use defmt::{Debug2Format, Format};
use tc37x_pac::scu::{ccucon0, ccucon1, ccucon2, ccucon5};

/// Clock distribution configuration: contains all `ccu` values requires to correctly
/// configure the clocks
#[derive(Debug)]
pub struct ClockDistribution {
    pub(super) fsi2div: ccucon0::FSI2DIV_A,
    pub(super) fsidiv: ccucon0::FSIDIV_A,
    pub(super) bbbdiv: ccucon0::BBBDIV_A,
    pub(super) spbdiv: ccucon0::SPBDIV_A,
    pub(super) sridiv: ccucon0::SRIDIV_A,
    pub(super) gtmdiv: ccucon0::GTMDIV_A,
    pub(super) stmdiv: ccucon0::STMDIV_A,
    pub(super) clkselqspi: ccucon1::CLKSELQSPI_A,
    pub(super) qspidiv: ccucon1::QSPIDIV_A,
    pub(super) clkselmsc: ccucon1::CLKSELMSC_A,
    pub(super) mscdiv: ccucon1::MSCDIV_A,
    pub(super) i2cdiv: ccucon1::I2CDIV_A,
    pub(super) clkselmcan: ccucon1::CLKSELMCAN_A,
    pub(super) mcandiv: ccucon1::MCANDIV_A,
    pub(super) clkselasclins: ccucon2::CLKSELASCLINS_A,
    pub(super) asclinsdiv: ccucon2::ASCLINSDIV_A,
    pub(super) asclinfdiv: ccucon2::ASCLINFDIV_A,
    pub(super) mcanhdiv: ccucon5::MCANHDIV_A,
    pub(super) gethdiv: ccucon5::GETHDIV_A,
}

impl Format for ClockDistribution {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{}", Debug2Format(self))
    }
}

impl ClockDistribution {
    /// Return an hardcoded & working configuration
    pub fn new() -> Self {
        ClockDistribution {
            fsi2div: ccucon0::FSI2DIV_A::INHERIT_SRI,
            fsidiv: ccucon0::FSIDIV_A::MAYBE_DIV3,
            bbbdiv: ccucon0::BBBDIV_A::DIV2,
            spbdiv: ccucon0::SPBDIV_A::DIV3,
            sridiv: ccucon0::SRIDIV_A::DIV1,
            gtmdiv: ccucon0::GTMDIV_A::DIV1,
            stmdiv: ccucon0::STMDIV_A::DIV3,
            clkselqspi: ccucon1::CLKSELQSPI_A::USE_SOURCE2,
            qspidiv: ccucon1::QSPIDIV_A::DIV1,
            clkselmsc: ccucon1::CLKSELMSC_A::USE_SOURCE1,
            mscdiv: ccucon1::MSCDIV_A::DIV1,
            i2cdiv: ccucon1::I2CDIV_A::DIV2,
            clkselmcan: ccucon1::CLKSELMCAN_A::USE_MCANI,
            mcandiv: ccucon1::MCANDIV_A::DIV2,
            clkselasclins: ccucon2::CLKSELASCLINS_A::USE_ASCLINSI,
            asclinsdiv: ccucon2::ASCLINSDIV_A::DIV2,
            asclinfdiv: ccucon2::ASCLINFDIV_A::DIV1,
            mcanhdiv: ccucon5::MCANHDIV_A::DIV3,
            gethdiv: ccucon5::GETHDIV_A::DIV2,
        }
    }
}
