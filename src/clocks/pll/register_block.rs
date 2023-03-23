//!
//! Helpers for PLL registers
//!
use tc37x_pac::{scu::*, Peripherals};
use tc37x_rt::util::wait;

use crate::clocks::pll::config::SysPllConfig;

use super::config::PeripheralPllConfig;

/// System PLL control register access with stat
pub struct SystemPLL<'r> {
    syspllcon0: &'r SYSPLLCON0,
    syspllcon1: &'r SYSPLLCON1,
    syspllstat: &'r SYSPLLSTAT,
}

impl<'r> SystemPLL<'r> {
    pub fn new(p: &'r Peripherals) -> Self {
        SystemPLL {
            syspllcon0: &p.SCU.syspllcon0,
            syspllcon1: &p.SCU.syspllcon1,
            syspllstat: &p.SCU.syspllstat,
        }
    }
}

impl<'r> SystemPLL<'r> {
    /// Turn the register on & wait until operation is completed
    fn on(&self) -> Result<&Self, ()> {
        defmt::trace!("SystemPLL -> ON");
        defmt::flush();

        self.syspllcon0.modify(|_, w| w.pllpwd().set_bit());
        wait(|| self.syspllstat.read().pwdstat().bit_is_clear())?;

        // Write K2 factor to the highest value to reduce current spike when the
        // clock turns on
        // TODO: Can we do that before turning on the system PLL?
        self.configure_k(SysPllConfig::k2_max())?;

        // Make sure we can still detect the lock
        self.syspllcon0.modify(|_, w| w.resld().set_bit());
        wait(|| self.syspllstat.read().lock().bit_is_set())?;

        Ok(self)
    }

    /// Turn off the system PLL
    ///
    /// # Safety
    /// Turning of the system PLL can make access to memory/registers that depend
    /// on this clock undefined behavior. The user must make sure that the system PLL is not
    /// used anywhere else, e.g. by selecting the backup clock as an input for
    /// the clock distribution
    unsafe fn off(&self) -> Result<&Self, ()> {
        defmt::trace!("SystemPLL -> OFF");
        defmt::flush();

        self.syspllcon0.modify(|_, w| w.pllpwd().clear_bit());
        wait(|| self.syspllstat.read().pwdstat().bit_is_set())?;

        Ok(self)
    }

    /// Configure, without waiting the P/N and input clock (hardcoded to oscillator)
    fn configure_pn(&self, config: &SysPllConfig) -> Result<&Self, ()> {
        defmt::trace!(
            "SystemPLL: Configure p={}, n={}",
            config.effective_p(),
            config.effective_n()
        );
        defmt::flush();

        self.syspllcon0.modify(|_, w| {
            w.ndiv()
                .variant(config.n)
                .pdiv()
                .variant(config.p)
                // Hard-code oscillator as input source for now
                .insel()
                .variant(syspllcon0::INSEL_A::OSCILLATOR)
        });

        Ok(self)
    }

    /// Configure and wait the k-divider
    fn configure_k(&self, k2: u8) -> Result<&Self, ()> {
        wait(|| self.syspllstat.read().k2rdy().bit_is_set())?;

        self.syspllcon1.modify(|_, w| w.k2div().variant(k2));

        wait(|| self.syspllstat.read().k2rdy().bit_is_set())?;

        Ok(self)
    }

    /// Start setting up the system PLL to the given configuration
    ///
    /// # Safety
    /// This temporarily turns of the system PLL, and potentially changes the frequency.
    /// This means that access to entities that are configured to depend on this clock
    /// will show undefined behvior. The user must configure the CCU to
    /// a backup clock before calling this function.
    pub unsafe fn initial_configuration(&self, config: &SysPllConfig) -> Result<(), ()> {
        self.off()?.configure_pn(config)?.on()?;

        Ok(())
    }

    /// Throttle PLL in steps?
    pub fn throttle(&self, config: &SysPllConfig) -> Result<&Self, ()> {
        for step in (config.k2..SysPllConfig::k2_max()).rev() {
            defmt::trace!("PLL Throttle K -> {}", step);
            defmt::flush();
            wait(|| self.syspllstat.read().k2rdy().bit_is_set())?;
            self.syspllcon1.modify(|_, w| w.k2div().variant(step));
            wait(|| self.syspllstat.read().k2rdy().bit_is_set())?;
        }

        defmt::trace!("Throttling done!");
        defmt::flush();
        Ok(self)
    }
}

/// System PLL control register access with stat
pub struct PeripheralPLL<'r> {
    perpllcon0: &'r PERPLLCON0,
    perpllcon1: &'r PERPLLCON1,
    perpllstat: &'r PERPLLSTAT,
}

impl<'r> PeripheralPLL<'r> {
    pub fn new(p: &'r Peripherals) -> Self {
        PeripheralPLL {
            perpllcon0: &p.SCU.perpllcon0,
            perpllcon1: &p.SCU.perpllcon1,
            perpllstat: &p.SCU.perpllstat,
        }
    }
}

impl<'r> PeripheralPLL<'r> {
    /// Turn the PLL on & wait until operation is completed
    fn on(&self) -> Result<&Self, ()> {
        defmt::trace!("PeripheralPLL: ON");
        defmt::flush();

        self.perpllcon0.modify(|_, w| w.pllpwd().set_bit());
        wait(|| self.perpllstat.read().pwdstat().bit_is_clear())?;

        // Fail here if the lock detection does not succeed (cause of invalid
        // configuration like wrong input selection or p/n out of bounds)
        self.perpllcon0.modify(|_, w| w.resld().set_bit());
        wait(|| self.perpllstat.read().lock().bit_is_set())?;

        Ok(self)
    }

    /// Turns the PLL off
    ///
    /// # Safety
    /// Turning of the peripheral PLL can make access to memory/registers that depend
    /// on this clock undefined behavior. The user must make sure that the peripheral PLL is not
    /// used anywhere else, e.g. by selecting the backup clock as an input for
    /// the clock distribution
    unsafe fn off(&self) -> Result<&Self, ()> {
        defmt::trace!("PeripheralPLL: OFF");
        defmt::flush();

        self.perpllcon0.modify(|_, w| w.pllpwd().clear_bit());
        wait(|| self.perpllstat.read().pwdstat().bit_is_set())?;

        Ok(self)
    }

    /// Configure, without waiting the P/N and input clock (hardcoded to oscillator)
    fn configure_pn(&self, config: &PeripheralPllConfig) -> &Self {
        defmt::trace!(
            "PeripheralPLL: Configure p={}, n={}, pre-k3-divider={}",
            config.effective_p(),
            config.effective_n(),
            config.pre_k3_divider()
        );
        defmt::flush();

        self.perpllcon0.modify(|_, w| {
            w.ndiv()
                .variant(config.n)
                .pdiv()
                .variant(config.p)
                .divby()
                .variant(config.k_bypass)
        });

        self
    }

    /// Configure the k-dividers
    fn configure_dividers(&self, config: &PeripheralPllConfig) -> Result<&Self, ()> {
        defmt::trace!(
            "PeripheralPLL: Configure k2={}, k3={}",
            config.effective_k2(),
            config.effective_k3()
        );
        defmt::flush();

        // TODO: We should implement throttling here as well
        let r = self.perpllstat.read();
        wait(|| r.k2rdy().bit_is_set() && r.k3rdy().bit_is_set())?;
        self.perpllcon1
            .modify(|_, w| w.k2div().variant(config.k2).k3div().variant(config.k3));
        wait(|| r.k2rdy().bit_is_set() && r.k3rdy().bit_is_set())?;

        Ok(self)
    }

    /// Start setting up the system PLL to the given configuration
    ///
    /// # Safety
    /// This temporarily turns of the system PLL, and potentially changes the frequency.
    /// This means that access to entities that are configured to depend on this clock
    /// will show undefined behvior. The user must configure the CCU to
    /// a backup clock before calling this function.
    pub unsafe fn initial_configuration(&self, config: &PeripheralPllConfig) -> Result<(), ()> {
        self.off()?
            .configure_pn(&config)
            .on()?
            .configure_dividers(&config)?;

        Ok(())
    }
}
