//! CCU-register stuff
#![allow(clippy::mistyped_literal_suffixes)]
use tc37x_pac::{scu::*, Peripherals};
use tc37x_rt::util::wait;

use super::config::ClockDistribution;

/// Clock source for source0, source1 and source2
#[derive(defmt::Format)]
pub enum ClockSource {
    BackupClock,
    PLL,
}

impl ClockSource {
    /// Return the raw register value given the variant
    pub fn as_variant(&self) -> ccucon0::CLKSEL_A {
        match self {
            ClockSource::BackupClock => ccucon0::CLKSEL_A::BACKUP,
            ClockSource::PLL => ccucon0::CLKSEL_A::PLL,
        }
    }
}

/// Clock control Unit (CCU0 - CCU5)
pub struct ClockControlUnit<'r> {
    ccucon0: &'r CCUCON0,
    ccucon1: &'r CCUCON1,
    ccucon2: &'r CCUCON2,
    ccucon5: &'r CCUCON5,
}

impl<'r> ClockControlUnit<'r> {
    pub fn new(p: &'r Peripherals) -> Self {
        ClockControlUnit {
            ccucon0: &p.SCU.ccucon0,
            ccucon1: &p.SCU.ccucon1,
            ccucon2: &p.SCU.ccucon2,
            ccucon5: &p.SCU.ccucon5,
        }
    }
}

impl<'r> ClockControlUnit<'r> {
    /// Set the clock source for the unit
    pub fn set_source(&self, source: ClockSource) -> Result<&Self, ()> {
        defmt::trace!("Configuring clock source for system for {:?}", &source);

        wait(|| self.ccucon0.read().lck().bit_is_clear())?;
        self.ccucon0
            .modify(|_, w| w.clksel().variant(source.as_variant()).up().set_bit());
        wait(|| self.ccucon0.read().lck().bit_is_clear())?;

        Ok(self)
    }

    /// Set up the clock distribution
    pub fn distribute_clock(&self, clock_distribution: ClockDistribution) -> Result<&Self, ()> {
        defmt::info!("Configuring clock distribution {:?}", clock_distribution);
        defmt::flush();

        // CCU0
        wait(|| self.ccucon0.read().lck().bit_is_clear())?;
        self.ccucon0.modify(|_, w| {
            w.fsi2div()
                .variant(clock_distribution.fsi2div)
                .fsidiv()
                .variant(clock_distribution.fsidiv)
                .bbbdiv()
                .variant(clock_distribution.bbbdiv)
                .spbdiv()
                .variant(clock_distribution.spbdiv)
                .sridiv()
                .variant(clock_distribution.sridiv)
                .gtmdiv()
                .variant(clock_distribution.gtmdiv)
                .stmdiv()
                .variant(clock_distribution.stmdiv)
        });
        wait(|| self.ccucon0.read().lck().bit_is_clear())?;

        // CCU1
        wait(|| self.ccucon1.read().lck().bit_is_clear())?;
        // Disable all clocks before doing anything with the register
        self.ccucon1.modify(|_, w| {
            w.clkselmcan()
                .variant(ccucon1::CLKSELMCAN_A::STOPPED)
                .clkselmsc()
                .variant(ccucon1::CLKSELMSC_A::STOPPED)
                .clkselqspi()
                .variant(ccucon1::CLKSELQSPI_A::STOPPED)
        });
        wait(|| self.ccucon1.read().lck().bit_is_clear())?;
        // iLLD ignores PLL1DIVDIS
        self.ccucon1.modify(|_, w| {
            w.clkselqspi()
                .variant(clock_distribution.clkselqspi)
                .qspidiv()
                .variant(clock_distribution.qspidiv)
                .clkselmsc()
                .variant(clock_distribution.clkselmsc)
                .mscdiv()
                .variant(clock_distribution.mscdiv)
                .i2cdiv()
                .variant(clock_distribution.i2cdiv)
                .clkselmcan()
                .variant(clock_distribution.clkselmcan)
                .mcandiv()
                .variant(clock_distribution.mcandiv)
        });
        wait(|| self.ccucon1.read().lck().bit_is_clear())?;

        // CCU2
        wait(|| self.ccucon2.read().lck().bit_is_clear())?;
        // Disable asclins clock before doing anything with the register
        self.ccucon2
            .modify(|_, w| w.clkselasclins().variant(ccucon2::CLKSELASCLINS_A::STOPPED));
        wait(|| self.ccucon2.read().lck().bit_is_clear())?;
        // iLLD ignores HSPDMPERON, ERAYPERON, EBUPERON
        self.ccucon2.modify(|_, w| {
            w.clkselasclins()
                .variant(clock_distribution.clkselasclins)
                .asclinsdiv()
                .variant(clock_distribution.asclinsdiv)
                .asclinfdiv()
                .variant(clock_distribution.asclinfdiv)
        });
        wait(|| self.ccucon1.read().lck().bit_is_clear())?;

        // CCU5
        wait(|| self.ccucon5.read().lck().bit_is_clear())?;
        // iLLD ignores ADASDIV
        self.ccucon5.modify(|_, w| {
            w.mcanhdiv()
                .variant(clock_distribution.mcanhdiv)
                .gethdiv()
                .variant(clock_distribution.gethdiv)
        });
        wait(|| self.ccucon5.read().lck().bit_is_clear())?;

        Ok(self)
    }
}
