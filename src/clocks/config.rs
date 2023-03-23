use super::{ccu, oscillator, pll};

/// Contains all the clock configurations
pub struct Clocks {
    pub(super) oscillator: oscillator::config::Oscillator,
    pub(super) system: pll::config::SysPllConfig,
    pub(super) peripheral: pll::config::PeripheralPllConfig,
    pub(super) ccu: ccu::config::ClockDistribution,
}

impl Clocks {
    /// Return a new configuration given an oscillator
    pub fn new(oscillator_config: oscillator::config::Oscillator) -> Self {
        Clocks {
            oscillator: oscillator_config,
            system: pll::config::SysPllConfig::new(),
            peripheral: pll::config::PeripheralPllConfig::new(),
            ccu: ccu::config::ClockDistribution::new(),
        }
    }
}
