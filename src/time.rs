//!
//! Basic time utilities for testing
//!
use core::{
    ops::{Add, Sub},
    time::Duration,
};
use tc37x_pac::Peripherals;

/// Basic instant implementation based on [STM0]
///
/// At the moment, this gives no guarantees about the Instant quality
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    time_since_boot: Duration,
}

impl Instant {
    /// Return a new instant
    pub fn now() -> Self {
        let p = unsafe { Peripherals::steal() };
        let time = p.STM0.tim4.read().bits() as u64;

        const HZ_FREQUENCY_STM0: u64 = 100_000_000;

        // We are not reading tim0, but tim4, so our time is actually multiplied with 1 << 16
        let millis = (time * 1_000 * (1 << 16)) / (HZ_FREQUENCY_STM0);

        Instant {
            time_since_boot: Duration::from_millis(millis),
        }
    }
}

impl<'a> Add<Duration> for &'a Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant {
            time_since_boot: self.time_since_boot + rhs,
        }
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.time_since_boot - rhs.time_since_boot
    }
}

impl defmt::Format for Instant {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Instant {{ time_since_boot: {}s }}",
            self.time_since_boot.as_secs_f32()
        );
    }
}
