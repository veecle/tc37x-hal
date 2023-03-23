use bitfield_struct::bitfield;
use defmt::Format;

use crate::can::CanID;

use super::{module_ram::CanBuffer, tx::TxMessageT0};

/// Represents a received can frame laid out in the can module ram.
///
/// From https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/Infra/Sfr/TC37A/_Reg/IfxCan_regdef.h#L2534
#[derive(Default, Clone)]
#[repr(C)]
pub struct CanRxFrame<B: CanBuffer> {
    transmit_buffer_0: RxMessageT0,
    transmit_buffer_1: RxMessageT1,
    /// Up to 8 bytes payload, actual length of data defined by [TxMessageT1::dlc]
    buffer: B,
}

/// They appear to be the same
type RxMessageT0 = TxMessageT0;

/// Represents a to be transceived can frame laid out in the can module ram.
///
/// From https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/Infra/Sfr/TC37A/_Reg/IfxCan_regdef.h#L1393
#[bitfield(u32)]
#[derive(Default)]
pub struct RxMessageT1 {
    rx_timestamp: u16,
    /// Indicates the actual length of the array in the parenting structure `CanFrame`
    #[bits(4)]
    dlc: u8,
    bitrate_switching: bool,
    is_fd_format: bool,
    #[bits(2)]
    reserved_22: u32,
    #[bits(7)]
    filter_index: u8,
    accepted_non_matching_frame: bool,
}

impl<B: CanBuffer> Format for CanRxFrame<B> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CanFrame {{ id: 0x{:X}, data: {}, rtr: {}, error_state: {}, bitrate_switching: {}, is_fd_format: {}, filter_index: {}, accepted_non_matching_frame: {} }}",
            self.get_id(),
            self.data(),
            self.transmit_buffer_0.rtr(),
            self.transmit_buffer_0.error_state(),
            self.transmit_buffer_1.bitrate_switching(),
            self.transmit_buffer_1.is_fd_format(),
            self.transmit_buffer_1.filter_index(),
            self.transmit_buffer_1.accepted_non_matching_frame(),
        )
    }
}

impl<B: CanBuffer> CanRxFrame<B> {
    pub fn get_id(&self) -> CanID {
        let id_field = self.transmit_buffer_0.id();
        if self.transmit_buffer_0.is_extended() {
            CanID::Extended(id_field)
        } else {
            CanID::Standard((id_field >> 18) as u16)
        }
    }

    pub fn data(&self) -> &[u8] {
        let length = self.transmit_buffer_1.dlc() as usize;
        &(self.buffer.as_ref()[..length])
    }
}
