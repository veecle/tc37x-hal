use core::fmt::Debug;
use defmt::{Debug2Format, Format};
use tc37x_pac::can0::node::psr::ACT_A;

use crate::can::CanModule;

use super::{CanNode, Running};

#[derive(Format)]
pub struct NodeErrorState {
    transmit_error_counter: u8,
    receive_error_counter: u8,
    protocol_status: ProtocolStatus,
}

struct DebugWrapper<T>(T);

impl<T: Debug> Format for DebugWrapper<T> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{}", Debug2Format(&self.0))
    }
}

#[derive(Format)]
pub struct ProtocolStatus {
    activity: DebugWrapper<ACT_A>,
    /// Indicates if at least one of error counter (REC/TEC) has reached the Error_Warning limit of 96
    warning_status: bool,
    in_error_passive: bool,
    bus_is_off: bool,
}

impl<'r, 'mem, AnyConnection, AnyTx, AnyRx, M: CanModule>
    CanNode<'r, AnyConnection, Running, AnyTx, AnyRx, M>
{
    pub fn clear_error(&self) -> NodeErrorState {
        let psr_value = self.node.psr.read();
        let ecr_value = self.node.ecr.read();

        NodeErrorState {
            transmit_error_counter: ecr_value.tec().bits(),
            receive_error_counter: ecr_value.rec().bits(),
            protocol_status: ProtocolStatus::from_register_values(
                psr_value.act().variant(),
                psr_value.ew().bit(),
                psr_value.ep().bit(),
                psr_value.bo().bit(),
            ),
        }
    }
}

impl ProtocolStatus {
    pub fn from_register_values(
        activity: ACT_A,
        warning_status: bool,
        in_error_passive: bool,
        bus_is_off: bool,
    ) -> Self {
        Self {
            activity: DebugWrapper(activity),
            warning_status,
            in_error_passive,
            bus_is_off,
        }
    }

    /// Check whether the status indicates that something is wrong
    pub fn indicates_error(&self) -> bool {
        self.warning_status || self.in_error_passive || self.bus_is_off
    }
}
