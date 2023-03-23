//! Basic CAN module implementation
use defmt::Format;

pub mod can0;

pub mod memory {
    pub mod module_ram;
    pub mod rx;
    pub mod tx;
}

pub mod node;
pub mod timing;

pub use memory::rx::CanRxFrame;
pub use memory::tx::CanTxFrame;

/// A Can module (CAN0, CAN1...); Used to track RAM properties
///
/// # Safety
/// Unsafe as wrong data here will result in hard faults
pub unsafe trait CanModuleRAM {
    /// Address of the RAM
    const RAM_LOCATION: *mut u8;
    /// The size of the RAM, in bytes
    const RAM_SIZE: usize;
}

/// Marker for a CAN peripheral
pub trait CanModule {
    /// Associated RAM module
    type RAM: CanModuleRAM;
}

/// Type-aware wrapper around a CAN identifier
#[derive(PartialEq, Eq)]
pub enum CanID {
    /// A standard ID
    Standard(u16),
    /// An extended ID
    Extended(u32),
}

impl Format for CanID {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            CanID::Standard(id) => defmt::write!(fmt, "CanID::Standard(0x{:X})", id),
            CanID::Extended(id) => defmt::write!(fmt, "CanID::Extended(0x{:X})", id),
        }
    }
}
