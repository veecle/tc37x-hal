//! Module contains implemented blocking traits from [`embedded_hal::blocking::delay`].
extern crate embedded_hal;
extern crate tc37x_pac;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use tc37x_pac::{Peripherals, STM0, STM1, STM2};
use tc37x_rt::asm_calls::read_cpu_core_id;

/// Stores one of the three system times.
pub enum SystemTimers {
    Timer0(STM0),
    Timer1(STM1),
    Timer2(STM2),
}

/// Generalized trait for each timer
trait DelayForTimers {
    /// Delay execution for a number of ticks
    fn delay(&self, ticks: u32);
}

macro_rules! implement_delay_for_timer {
    ($timer:path) => {
        impl DelayForTimers for $timer {
            fn delay(&self, ticks: u32) {
                let time = self.tim0.read().bits();
                let mut nt = time;
                while nt.wrapping_sub(time) < ticks {
                    nt = self.tim0.read().bits();
                }
            }
        }
    };
}

implement_delay_for_timer!(STM0);
implement_delay_for_timer!(STM1);
implement_delay_for_timer!(STM2);

/// Implementations for the DelayMs and DelayUs traits.
///
/// DelayMs<u8>, DelayMs<u16>, DeleayMs<u32>, DelayUs<u8>, DelayUs<u16>, DelasUs<32> traits implemented. In the implementation `tim0` register is
/// used to read system timer's value. `tim0` is also has 32 bits. If you use implemented functions, consider that 32 bit.
///
/// E.g. TC375 timers has frequency 100 000 000. You cannot store larger time interval than 2^32 ticks. 2^32 ticks = 4294967296 ticks = 42.94967296 secs.
/// You cannot add more millisenonds for delay_ms function than `42_9496_u32`.
pub struct Delay {
    frequency: u32,
    system_timer: SystemTimers,
}

impl Delay {
    /// Creates Delay structure for the system timer of current CPU cure.
    pub fn new(ticks: u32) -> Self {
        let p = unsafe { Peripherals::steal() };
        let cpu_core_id = read_cpu_core_id();
        match cpu_core_id {
            0 => Delay {
                frequency: ticks,
                system_timer: SystemTimers::Timer0(p.STM0),
            },
            1 => Delay {
                frequency: ticks,
                system_timer: SystemTimers::Timer1(p.STM1),
            },
            _ => Delay {
                frequency: ticks,
                system_timer: SystemTimers::Timer2(p.STM2),
            },
        }
    }

    /// Releases the system timer (SysTick) resource.
    pub fn free(self) -> SystemTimers {
        self.system_timer
    }
}

const CONVERSION_MULTIPLIER_FOR_MILI: u32 = 1000_u32;
const CONVERSION_MULTIPLIER_FOR_MICRO: u32 = 1_000_000_u32;

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * CONVERSION_MULTIPLIER_FOR_MILI);
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        let ticks = us * (self.frequency / CONVERSION_MULTIPLIER_FOR_MICRO);
        match &self.system_timer {
            SystemTimers::Timer0(stm) => stm.delay(ticks),
            SystemTimers::Timer1(stm) => stm.delay(ticks),
            SystemTimers::Timer2(stm) => stm.delay(ticks),
        }
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
