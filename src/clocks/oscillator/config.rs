pub struct Oscillator {
    pub(super) oscval: u8,
}

impl Oscillator {
    pub fn new(frequency_mhz: u8) -> Oscillator {
        assert!(frequency_mhz <= 40);
        assert!(frequency_mhz >= 16);
        Oscillator {
            oscval: frequency_mhz - 15,
        }
    }

    pub fn oscillator_speed(&self) -> u8 {
        self.oscval + 15
    }
}
