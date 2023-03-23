/// Type-safe wrapper to work with frequencies
///
/// This is not **proper** :)
pub struct Frequency {
    _hz: u32,
}

impl Frequency {
    /// A new frequency for Hz
    pub const fn hz(frequency: u32) -> Self {
        Frequency { _hz: frequency }
    }
}

pub trait FreqExt {
    /// Return the frequency in MHz
    fn mhz(&self) -> Frequency;

    /// Return the frequency in MHz
    fn khz(&self) -> Frequency;

    /// Return the frequency in Hz
    fn hz(&self) -> Frequency;
}

impl FreqExt for u32 {
    fn mhz(&self) -> Frequency {
        Frequency {
            _hz: (*self).checked_mul(1_000_000).unwrap(),
        }
    }

    fn khz(&self) -> Frequency {
        Frequency {
            _hz: (*self).checked_mul(1_000).unwrap(),
        }
    }

    fn hz(&self) -> Frequency {
        Frequency { _hz: *self }
    }
}
