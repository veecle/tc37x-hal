use defmt::Format;

/// A helper structure to configure nominal bit timing for can
///
/// See https://www.infineon.com/dgdl/Infineon-AURIX_TC3xx_Part2-UserManual-v02_00-EN.pdf?fileId=5546d462712ef9b701717d35f8541d94
pub struct CanBitrate {
    sync_jump_width: u8,
    pre_scaler: u16,
    time_segment1: u8,
    time_segment2: u8,
}

/// Manually crunched numbers to yield a 50khz bitrate for a 80Mhz CAN clock
const BITRATE_50KHZ: CanBitrate = CanBitrate {
    sync_jump_width: 1,
    pre_scaler: 200,
    time_segment1: 6,
    time_segment2: 1,
};

/// Manually crunched numbers to yield a 500khz bitrate for a 80Mhz CAN clock
const BITRATE_500KHZ: CanBitrate = CanBitrate {
    sync_jump_width: 1,
    pre_scaler: 10,
    time_segment1: 13,
    time_segment2: 2,
};

/// Zero-cost abstraction to express the unit "kilobits per second" through the type
#[derive(PartialEq, Eq)]
pub struct Kbps(u32);

pub trait U32Ext {
    /// Interpret the given value as kilobits per seconds
    fn kbps(&self) -> Kbps;
}

impl U32Ext for u32 {
    fn kbps(&self) -> Kbps {
        Kbps(*self)
    }
}

impl CanBitrate {
    /// Infer can bitrate timings for the given frequency. This assumes an 80Mhz
    /// f_async CAN clock.
    pub fn from_frequency(bitrate: Kbps) -> Result<Self, ()> {
        // Here a more sophisticated algorithm may be implemented, but for now
        // we only have hardcoded values for 50khz and 500khz
        if bitrate == 50u32.kbps() {
            Ok(BITRATE_50KHZ)
        } else if bitrate == 500u32.kbps() {
            Ok(BITRATE_500KHZ)
        } else {
            Err(())
        }
    }

    pub fn sync_jump_width(&self) -> u8 {
        self.sync_jump_width
    }

    pub fn pre_scaler(&self) -> u16 {
        self.pre_scaler
    }

    pub fn tseg1(&self) -> u8 {
        self.time_segment1
    }

    pub fn tseg2(&self) -> u8 {
        self.time_segment2
    }
}

impl Format for CanBitrate {
    fn format(&self, fmt: defmt::Formatter) {
        let ccu_can_frequency = 80_000_000.0f32;

        let clock_period = 1.0 / ccu_can_frequency;

        let time_quanta = clock_period * self.pre_scaler as f32;

        let total_bit_time =
            (self.sync_jump_width as f32 + self.time_segment1 as f32 + self.time_segment2 as f32)
                * time_quanta;

        let sample_point = (self.sync_jump_width as f32 + self.time_segment1 as f32)
            / (self.sync_jump_width as f32 + self.time_segment1 as f32 + self.time_segment2 as f32);

        defmt::write!(
            fmt,
            "BitRate {{ frequency: {}hz, sample_point: {}% }}",
            1.0 / total_bit_time,
            100.0 * sample_point
        );
    }
}
