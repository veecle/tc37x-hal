use tc37x_pac::Peripherals;
use tc37x_rt::call_without_endinit;

use crate::clocks::{
    ccu::{self, register_block::ClockControlUnit},
    oscillator::register_block::OscillatorUnit,
    pll::register_block::{PeripheralPLL, SystemPLL},
};

use super::config;

/// Trait to be implemented by peripherals that can configure the clocks
pub trait SetupClocks {
    /// Given a configuration, setup the clocks
    fn setup(&self, clocks: config::Clocks) -> Result<(), ()>;
}

impl SetupClocks for Peripherals {
    fn setup(&self, clocks: config::Clocks) -> Result<(), ()> {
        call_without_endinit(|| {
            let oscillator = OscillatorUnit::new(self);
            let ccu = ClockControlUnit::new(self);
            let syspll = SystemPLL::new(self);
            let pll = PeripheralPLL::new(self);

            oscillator.configure(clocks.oscillator)?;

            // Configure clock distribution mode for backup clock before configuring PLL
            ccu.set_source(ccu::register_block::ClockSource::BackupClock)?;

            // Configure system PLL
            defmt::info!("Configuring System PLL");

            // SAFETY: The CCU uses the backup clock for now so we can change the system PLL
            unsafe { syspll.initial_configuration(&clocks.system)? };

            // Configure peripherals PLL
            defmt::info!("Configuring Peripheral PLL");
            // SAFETY: The CCU uses the backup clock for now so we can change the system PLL
            unsafe { pll.initial_configuration(&clocks.peripheral)? };

            defmt::flush();
            ccu.distribute_clock(clocks.ccu)?;

            // Clock distribution is set up correctly, now switch input to PLL
            ccu.set_source(ccu::register_block::ClockSource::PLL)?;

            // Throttle system clock
            syspll.throttle(&clocks.system)?;
            Ok(())
        })
    }
}
