//! CAN0 implementation
use core::marker::PhantomData;

use tc37x_pac::CAN0;
use tc37x_rt::{util::wait, wdtcon::*};

use super::node::{
    connection::{DefaultDisconnected, Node0Pin, Node1Pin},
    receive::NoRx,
    transceive::NoTx,
    CanNode, InConfiguration,
};

/// Implementation for CAN0
///
/// Later this can be generalized for a <T: CanInstance> for T: {CAN0, CAN1} like in our HAL; \
/// similarly, register and access shall be protected via trait access
pub struct CanModule0<'r, Node0, Node1> {
    /// Generic CAN access (Hardcoded CAN-0 for now)
    can: &'r CAN0,
    marker: PhantomData<(Node0, Node1)>,
}

/// States tracking the availability of a CAN node within a module
mod states {
    pub struct Available;
    pub struct Taken;
}
pub use states::*;

mod mem {
    use crate::can::{CanModule, CanModuleRAM};
    use super::CanModule0;

    impl<'r, Node0, Node1> CanModule for CanModule0<'r, Node0, Node1> {
        type RAM = CanModule0RAM;
    }

    /// Type defining the RAM for can module 0
    pub struct CanModule0RAM;

    // RAM definition for CAN0
    //
    // # Safety
    // Data comes from the reference manual
    unsafe impl CanModuleRAM for CanModule0RAM {
        const RAM_LOCATION: *mut u8 = 0xF0_20_00_00 as *mut u8;
        const RAM_SIZE: usize = 0x8000;
    }
}
pub use mem::*;

//
// Only run this if the module is not taken
//
impl<'r> CanModule0<'r, Available, Available> {
    /// New from peripherals
    pub fn new(p: &'r mut CAN0) -> Self {
        // We do it manually since it seems we need only for this operation
        clear_cpu_endinit();

        p.clc.modify(|_, w| w.disr().clear_bit());
        wait(|| p.clc.read().diss().bit_is_clear()).unwrap();

        // We do it manually since it seems we need only for this operation
        set_cpu_endinit();
        CanModule0 {
            can: p,
            marker: PhantomData,
        }
    }
}

// 
impl<'r, N1> CanModule0<'r, Available, N1> {
    pub fn node0(
        self,
    ) -> (
        CanModule0<'r, Taken, N1>,
        CanNode<'r, DefaultDisconnected<Node0Pin>, InConfiguration, NoTx, NoRx, Self>,
    ) {
        self.enable_clock_source(clock_helpers::CanNode::Node0);

        (
            CanModule0 {
                can: self.can,
                marker: PhantomData,
            },
            CanNode::node0(self.can),
        )
    }
}

impl<'r, N0> CanModule0<'r, N0, Available> {
    pub fn node1(
        self,
    ) -> (
        CanModule0<'r, N0, Taken>,
        CanNode<'r, DefaultDisconnected<Node1Pin>, InConfiguration, NoTx, NoRx, Self>,
    ) {
        self.enable_clock_source(clock_helpers::CanNode::Node1);

        (
            CanModule0 {
                can: self.can,
                marker: PhantomData,
            },
            CanNode::node1(self.can),
        )
    }
}

/// Structures to help with setting up the clock for a specific can node
mod clock_helpers {
    use tc37x_rt::util::wait;

    use super::CanModule0;

    impl<'r, N0, N1> CanModule0<'r, N0, N1> {
        /// Set the (hardcoded to NODE-0) can source
        pub(super) fn enable_clock_source(&self, node: CanNode) -> &Self {
            defmt::trace!("CAN0: Enabling clock source for node {:?}", node);

            // Unlock MCR
            self.can
                .mcr
                .modify(|_, w| w.ci().set_bit().ccce().set_bit());

            wait(|| {
                let mcr_read = self.can.mcr.read();
                mcr_read.ci().bit_is_set() && mcr_read.ccce().bit_is_set()
            })
            .unwrap();

            self.can.mcr.modify(|_, w| w.enable_clock(node));

            self.can.mcr.modify(|_, w| {
                w
                    // These values will not have propagated until we clear CCCE and CI
                    .enable_clock(node)
                    .ccce()
                    .clear_bit()
                    .ci()
                    .clear_bit()
            });

            wait(|| {
                let mcr_read = self.can.mcr.read();
                mcr_read.ci().bit_is_clear() && mcr_read.ccce().bit_is_clear()
            })
            .unwrap();

            defmt::assert!(self.can.mcr.read().is_enabled(node));

            self
        }
    }

    /// Clock select helper
    #[allow(unused)]
    #[derive(defmt::Format, Clone, Copy)]
    pub(super) enum CanNode {
        Node0,
        Node1,
        Node2,
        Node3,
    }

    trait ClockSelect {
        fn enable_clock(self, clock_enable: CanNode) -> Self;
    }

    trait GetClockSelect {
        fn is_enabled(self, clock_to_check: CanNode) -> bool;
    }

    impl ClockSelect for &mut tc37x_pac::can0::mcr::W {
        fn enable_clock(self, clock_enable: CanNode) -> Self {
            match clock_enable {
                CanNode::Node0 => self.clksel0().both_on(),
                CanNode::Node1 => self.clksel1().both_on(),
                CanNode::Node2 => self.clksel2().both_on(),
                CanNode::Node3 => self.clksel3().both_on(),
            }
        }
    }

    impl GetClockSelect for &tc37x_pac::can0::mcr::R {
        fn is_enabled(self, clock_to_check: CanNode) -> bool {
            match clock_to_check {
                CanNode::Node0 => self.clksel0().is_both_on(),
                CanNode::Node1 => self.clksel1().is_both_on(),
                CanNode::Node2 => self.clksel2().is_both_on(),
                CanNode::Node3 => self.clksel3().is_both_on(),
            }
        }
    }
}
