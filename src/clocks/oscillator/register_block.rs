use tc37x_pac::{
    scu::{osccon, OSCCON},
    Peripherals,
};
use tc37x_rt::util::wait;

use super::config;

/// Oscillator control
pub struct OscillatorUnit<'r> {
    osccon: &'r OSCCON,
}

impl<'r> OscillatorUnit<'r> {
    pub fn new(p: &'r Peripherals) -> Self {
        Self {
            osccon: &p.SCU.osccon,
        }
    }
}

impl<'r> OscillatorUnit<'r> {
    /// Configure self (hardcoded to 20Mhz)
    pub fn configure(&self, config: config::Oscillator) -> Result<&Self, ()> {
        defmt::trace!(
            "Setting external oscillator configuration to {}Mhz",
            config.oscillator_speed()
        );
        defmt::flush();

        self.osccon.modify(|_, w| {
            // Using oscillator, no power saving
            w.mode()
                .variant(osccon::MODE_A::EXTERNAL_CRYSTAL)
                .oscval()
                .variant(config.oscval)
        });

        // Following, we wait that this get's enabled by checking PPLHV/PPLHL
        wait(|| {
            let pplhv = self.osccon.read().pllhv().bit();
            let ppllv = self.osccon.read().plllv().bit();
            pplhv && ppllv
        })?;

        Ok(self)
    }
}
