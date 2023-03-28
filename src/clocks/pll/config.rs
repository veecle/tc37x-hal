/// PLL Configuration
#[derive(defmt::Format)]
pub struct SysPllConfig {
    /// P-divider
    pub p: u8,
    /// N-divider
    pub n: u8,
    /// K2-divider
    pub k2: u8,
}

impl SysPllConfig {
    /// A new default configuration
    pub const fn new() -> Self {
        SysPllConfig { p: 0, n: 29, k2: 1 }
    }

    /// Slowest divider for the system PLL
    ///
    /// TODO: Does this depend on the overall configuration, e.g. can the system
    /// pll be too slow?
    pub const fn k2_max() -> u8 {
        // Three bits k2 value for the system PLL, so this is the biggest valid divider
        0b111
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_p(&self) -> u8 {
        self.p + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_n(&self) -> u8 {
        self.n + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_k2(&self) -> u8 {
        self.k2 + 1
    }
}

/// Peripheral PLL Configuration
#[derive(defmt::Format)]
pub struct PeripheralPllConfig {
    /// P-divider
    pub p: u8,
    /// N-divider
    pub n: u8,
    // Whether to bypass the k3
    pub k_bypass: bool,
    /// K2-divider
    pub k2: u8,
    /// K3-divider
    pub k3: u8,
}

impl PeripheralPllConfig {
    /// A new default configuration
    pub const fn new() -> Self {
        PeripheralPllConfig {
            p: 0,
            n: 31,
            k_bypass: false,
            k2: 1,
            k3: 1,
        }
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_p(&self) -> u8 {
        self.p + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_n(&self) -> u8 {
        self.n + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_k2(&self) -> u8 {
        self.k2 + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn effective_k3(&self) -> u8 {
        self.k3 + 1
    }

    /// Return the actual value of the divider, and not the one stored in the register
    pub fn pre_k3_divider(&self) -> f32 {
        if self.k_bypass {
            2.0
        } else {
            1.6
        }
    }
}
