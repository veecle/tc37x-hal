/// PLL Configuration
#[derive(defmt::Format)]
pub struct SysPllConfig {
    pub p: u8,
    pub n: u8,
    pub k2: u8,
}

impl SysPllConfig {
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

    pub fn effective_p(&self) -> u8 {
        self.p + 1
    }

    pub fn effective_n(&self) -> u8 {
        self.n + 1
    }

    pub fn effective_k2(&self) -> u8 {
        self.k2 + 1
    }
}
#[derive(defmt::Format)]
pub struct PeripheralPllConfig {
    pub p: u8,
    pub n: u8,
    // Whether to bypass the k3
    pub k_bypass: bool,
    pub k2: u8,
    pub k3: u8,
}

impl PeripheralPllConfig {
    pub const fn new() -> Self {
        PeripheralPllConfig {
            p: 0,
            n: 31,
            k_bypass: false,
            k2: 1,
            k3: 1,
        }
    }

    pub fn effective_p(&self) -> u8 {
        self.p + 1
    }

    pub fn effective_n(&self) -> u8 {
        self.n + 1
    }

    pub fn effective_k2(&self) -> u8 {
        self.k2 + 1
    }

    pub fn effective_k3(&self) -> u8 {
        self.k3 + 1
    }

    pub fn pre_k3_divider(&self) -> f32 {
        if self.k_bypass {
            2.0
        } else {
            1.6
        }
    }
}
