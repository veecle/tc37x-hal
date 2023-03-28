//!
//! Helper structure for TXFrame memory descriptor
//!
use bitfield_struct::bitfield;
use defmt::Format;

use crate::can::CanID;

use super::module_ram::CanBuffer;

/// Simplified CanFrame structure in C representation to allow easy mem-copy... used only for printing and mem-copy
///
/// Later on we can change is `nos::CanFrame`
///
/// From https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/Infra/Sfr/TC37A/_Reg/IfxCan_regdef.h#L2534
#[derive(Default, Clone)]
#[repr(C)]
pub struct CanTxFrame<B: CanBuffer> {
    transmit_buffer_0: TxMessageT0,
    transmit_buffer_1: TxMessageT1,
    /// Up to 8 bytes payload, actual length of data defined by [TxMessageT1::dlc]
    buffer: B,
}

/// From https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/Infra/Sfr/TC37A/_Reg/IfxCan_regdef.h#L1463
#[bitfield(u32)]
#[derive(Default)]
pub struct TxMessageT0 {
    /// Interpretation of this field depends on `is_extended`, a standard 11 bit
    /// identifier is left shifted by 18 bits.
    ///
    /// See https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/iLLD/TC37A/Tricore/Can/Std/IfxCan.h#L2183
    #[bits(29)]
    pub id: u32,
    pub rtr: bool,
    pub is_extended: bool,
    pub error_state: bool,
}

/// From https://github.com/Infineon/AURIX_code_examples/blob/f1a75eea6a9cf939d6052a3cf9463ab338a17df3/code_examples/MCMCAN_1_KIT_TC375_LK/Libraries/Infra/Sfr/TC37A/_Reg/IfxCan_regdef.h#L1472
#[bitfield(u32)]
#[derive(Default)]
pub struct TxMessageT1 {
    #[bits(16)]
    reserved_0: u32,
    /// Indicates the actual length of the array in the parenting structure `CanFrame`
    #[bits(4)]
    dlc: u8,
    bitrate_switching: bool,
    is_fd_format: bool,
    #[bits(1)]
    reserved_22: u32,
    event_fifo_control: bool,
    message_marker: u8,
}

impl<B: CanBuffer> Format for CanTxFrame<B> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CanFrame {{ id: {}, data: {}, rtr: {}, error_state: {}, bitrate_switching: {}, is_fd_format: {}, event_fifo_control: {}, message_marker: 0x{:X} }}",
            self.get_id(),
            self.data(),
            self.transmit_buffer_0.rtr(),
            self.transmit_buffer_0.error_state(),
            self.transmit_buffer_1.bitrate_switching(),
            self.transmit_buffer_1.is_fd_format(),
            self.transmit_buffer_1.event_fifo_control(),
            self.transmit_buffer_1.message_marker(),
        )
    }
}

impl<B: CanBuffer> CanTxFrame<B> {
    pub fn set_id(&mut self, id: CanID) {
        let id_field = match id {
            CanID::Standard(id) => {
                self.transmit_buffer_0.set_is_extended(false);
                (id as u32) << 18
            }
            CanID::Extended(id) => {
                self.transmit_buffer_0.set_is_extended(true);
                id
            }
        };
        self.transmit_buffer_0.set_id(id_field);
    }

    pub fn get_id(&self) -> CanID {
        let id_field = self.transmit_buffer_0.id();
        if self.transmit_buffer_0.is_extended() {
            CanID::Extended(id_field)
        } else {
            CanID::Standard((id_field >> 18) as u16)
        }
    }

    pub fn set_data(&mut self, data: &[u8]) {
        if data.len() > 8 {
            defmt::panic!("CAN data length exceeds 8 ({})", data.len())
        }
        self.buffer.as_mut()[..data.len()].copy_from_slice(data);

        self.transmit_buffer_1.set_dlc(data.len() as u8);
    }

    pub fn data(&self) -> &[u8] {
        let length = self.transmit_buffer_1.dlc() as usize;
        &(self.buffer.as_ref()[..length])
    }
}
