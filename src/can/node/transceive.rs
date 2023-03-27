//!
//! Transmit side for a CAN module
//!
use core::marker::PhantomData;

use tc37x_pac::can0;

use crate::can::{
    memory::{
        module_ram::{CanBuffer, NodeMemory},
        tx::CanTxFrame,
    },
    CanModule, CanModuleRAM,
};

use super::{connection::Connected, CanNode, InConfiguration, Running};

mod states {
    use crate::can::{
        memory::{
            module_ram::{CanBuffer, NodeMemory},
            tx::CanTxFrame,
        },
        CanModuleRAM,
    };

    pub struct NoTx;

    pub struct TxDedicated<'a, B: CanBuffer, M: CanModuleRAM> {
        pub(super) memory: NodeMemory<'a, CanTxFrame<B>, M>,
    }
}

pub use states::*;

impl<'r, C: Connected, R, M: CanModule> CanNode<'r, C, InConfiguration, NoTx, R, M> {
    /// Set TX parameters (addr & buffer) are super unsafe for now
    pub fn set_tx<'mem, B: CanBuffer>(
        self,
        memory: NodeMemory<'mem, CanTxFrame<B>, M::RAM>,
    ) -> CanNode<'r, C, InConfiguration, TxDedicated<'mem, B, M::RAM>, R, M> {
        defmt::assert!(
            memory.elements() < 32,
            "Cannot support more than 32 buffers for now"
        );

        self.node
            .txesc
            .modify(|_, w| w.tbds().variant(B::buffer_size().into()));

        let addr = memory.in_module_offset() as u16;
        let num = memory.elements();

        self.node
            .txbc
            .modify(|_, w| w.tbsa().variant(addr >> 2).ndtb().variant(num));

        CanNode {
            tx_dedicated_config: TxDedicated { memory },
            ..self
        }
    }
}

pub struct Uninitialized;

pub struct Initialized;

pub struct TransmitBuffer<'a, B: CanBuffer, S, M: CanModuleRAM> {
    node: &'a can0::NODE,
    buffer: &'a TxDedicated<'a, B, M>,
    in_buffer_index: u8,
    marker: PhantomData<S>,
}

impl<'a, B: CanBuffer, M: CanModuleRAM> TransmitBuffer<'a, B, Uninitialized, M> {
    pub fn set_frame(self, frame: CanTxFrame<B>) -> TransmitBuffer<'a, B, Initialized, M> {
        let dst = unsafe { self.buffer.memory.get(self.in_buffer_index) }.unwrap();
        // In theory we should not need unsafe here, but IFX crashes if we *dst = frame because of some
        // parallel access (we figure Rust's tries to write 16 bytes at once)
        unsafe {
            core::ptr::write_volatile(dst as *mut CanTxFrame<B>, frame);
            core::arch::asm!("dsync");
        }
        TransmitBuffer {
            marker: PhantomData,
            ..self
        }
    }
}

impl<'a, B: CanBuffer, M: CanModuleRAM> TransmitBuffer<'a, B, Initialized, M> {
    pub fn send(self) {
        defmt::trace!("Request sending of buffer index {:?}", self.in_buffer_index);
        // Set the bit of the corresponding buffer index
        self.node
            .txbar
            .write(|w| unsafe { w.bits(1 << self.in_buffer_index) });
    }
}

pub enum TransmitError {}

impl<'r, 'mem, C: Connected, B: CanBuffer, R, M: CanModule>
    CanNode<'r, C, Running, TxDedicated<'mem, B, M::RAM>, R, M>
{
    pub fn with_transmit_buffer<S, F: FnOnce(TransmitBuffer<'_, B, Uninitialized, M::RAM>) -> S>(
        &mut self,
        buffer_consume: F,
    ) -> Option<S> {
        let buffer = self.acquire_transmit_buffer()?;
        Some(buffer_consume(buffer))
    }

    pub fn acquire_transmit_buffer<'a>(
        &'a mut self,
    ) -> Option<TransmitBuffer<'a, B, Uninitialized, M::RAM>>
    where
        'r: 'a,
        'mem: 'a,
    {
        for buffer_index in 0..self.tx_dedicated_config.memory.elements() {
            let buffer_is_free = unsafe { self.node.txbrp.read().trp(buffer_index).bit_is_clear() };

            if buffer_is_free {
                return Some(TransmitBuffer {
                    node: self.node,
                    buffer: &self.tx_dedicated_config,
                    in_buffer_index: buffer_index,
                    marker: PhantomData,
                });
            }
        }

        None
    }
}
