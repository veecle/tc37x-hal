use tc37x_pac::can0::node::gfc;

use crate::can::{
    memory::{
        module_ram::{CanBuffer, NodeMemory},
        rx::CanRxFrame,
    },
    CanModule,
};

use super::{connection::Connected, CanNode, InConfiguration, Running};

mod states {
    use crate::can::{
        memory::{
            module_ram::{CanBuffer, NodeMemory},
            rx::CanRxFrame,
        },
        CanModuleRAM,
    };

    pub struct NoRx;

    pub struct RxFifo0<'a, B: CanBuffer, M: CanModuleRAM> {
        pub(super) memory: NodeMemory<'a, CanRxFrame<B>, M>,
    }
}

pub use states::*;

impl<'r, 'mem, AnyConnection, AnyTx, M: CanModule>
    CanNode<'r, AnyConnection, InConfiguration, AnyTx, NoRx, M>
{
    /// Setup fifo0 to receive all frames
    pub fn set_rx_fifo0<B: CanBuffer>(
        self,
        memory: NodeMemory<'mem, CanRxFrame<B>, M::RAM>,
    ) -> CanNode<'r, AnyConnection, InConfiguration, AnyTx, RxFifo0<'mem, B, M::RAM>, M> {
        self.node.set_buffer_dimension::<B>(
            memory.in_module_offset() as u16,
            memory.elements(),
            FifoBehavior::Blocking,
        );

        // When implementing fifo1 we need to refactor this to somewhere else
        self.node
            .gfc
            .modify(|_, w| w.anfs().variant(gfc::ANFS_A::ACCEPT_FIFO0));

        CanNode {
            rx_fifo0_config: RxFifo0 { memory },
            ..self
        }
    }
}

impl<'r, 'mem, C: Connected, B: CanBuffer, AnyTx, M: CanModule>
    CanNode<'r, C, Running, AnyTx, RxFifo0<'mem, B, M::RAM>, M>
{
    /// This will try to fetch a packet from the FIFO_0, returning None if no
    /// packets have been received
    pub fn try_receive_fifo0(&mut self) -> Option<CanRxFrame<B>> {
        // Do we have a message to read?
        if self.node.rx_fifo0_fill_level() == 0 {
            return None;
        }

        // Get the buffer index to read (shall be < 32, else we panic.. we asserted that
        // during configuration)... temporary to avoid working with 2 registers
        let index = self.node.rx_fifo0_index();

        // Buffer slot & read the frame
        let rx_buffer = &mut self.rx_fifo0_config.memory;

        let src = unsafe { rx_buffer.get(index) }
            .expect("Buffer out of range (again, shall not happen with proper configuration)");

        // BUG: We are having trouble to make 64 bit reads from can0, to avoid that we
        // copy the memory here
        let frame = unsafe { core::ptr::read_volatile(src as *const CanRxFrame<B>) };

        defmt::trace!(
            "Received message {} at index {} in {}",
            frame,
            index,
            rx_buffer
        );

        // Ack the data and return frame
        self.node.rx_fifo0_ack_index(index);
        Some(frame)
    }
}

mod fif0_helpers {
    use crate::can::memory::module_ram::CanBuffer;

    pub trait FifoState {
        /// Obtain the numbers of available can messages in the fifo
        fn rx_fifo0_fill_level(&self) -> u8;

        /// Obtain the index of the next to be fetched can frame
        ///
        /// This should usually be paired with a call to `rx_fifo0_fill_level`
        /// before actually reading the memory in the module ram from this address
        fn rx_fifo0_index(&self) -> u8;

        /// Acknowledge all frames up until the given level
        fn rx_fifo0_ack_index(&self, index: u8);

        /// Sets up fifo0.
        ///
        /// This will
        fn set_buffer_dimension<B: CanBuffer>(
            &self,
            in_module_offset: u16,
            buffer_count: u8,
            buffer_full_behavior: FifoBehavior,
        );
    }

    /// Sets up the behavior what happens when the Fifo is full.
    ///
    /// # CAUTION
    /// Operating the fifo in overwrite mode is not implemented, since we might
    /// create race conditions between accessing the buffer and the cpu writing
    pub enum FifoBehavior {
        Blocking,
    }

    impl FifoState for &tc37x_pac::can0::node::NODE {
        fn rx_fifo0_fill_level(&self) -> u8 {
            self.rxf0s.read().f0fl().bits()
        }

        fn rx_fifo0_index(&self) -> u8 {
            self.rxf0s.read().f0gi().bits()
        }
        /// FIFO-0 Ack frame in index
        fn rx_fifo0_ack_index(&self, index: u8) {
            // This automatically update the other FIFO registers since we ack
            self.rxf0a.modify(|_, w| w.f0ai().variant(index))
        }

        fn set_buffer_dimension<B: CanBuffer>(
            &self,
            in_module_offset: u16,
            buffer_count: u8,
            buffer_full_behavior: FifoBehavior,
        ) {
            self.rxf0c.modify(|_, w| {
                w.f0sa()
                    .variant(in_module_offset >> 2)
                    .f0s()
                    .variant(buffer_count)
            });

            self.rxesc
                .modify(|_, w| w.f0ds().variant(B::buffer_size().into()));

            match buffer_full_behavior {
                FifoBehavior::Blocking => {
                    self.rxf0c.modify(|_, w| w.f0om().clear_bit());
                }
            }

            self.rxf0c.modify(|_, w| {
                w.f0wm().variant(0) // no watermark
            })
        }
    }
}
use fif0_helpers::*;
