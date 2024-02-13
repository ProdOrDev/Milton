//! Emulation of the Microvision's cartridge-specific features.
//!
//! Due to the Microvision's cartridge-centric design, not all cartridges obey
//! the same "settings" as others. Some modify the charge supplied to chips like
//! the rotary controller and others reverse the output decoder of the TMS1100.

use arbitrary_int::u4;

use crate::{lcd::DataLine, tms1100::OutputO};

/// The charge settings of a rotary controller.
///
/// This is used to calculate the effective time until a charge supplied to
/// the rotary controller/paddle would end.
#[derive(Debug, Clone, Copy)]
pub struct Charge {
    /// The value to offset the end time by.
    pub offset: usize,
    /// The value to scale the end time by.
    pub scale: usize,
}

impl Default for Charge {
    fn default() -> Self {
        Self {
            offset: 600,
            scale: 65,
        }
    }
}

/// The decode PLA for the O output of the TMS1100.
///
/// This is used for decided what will end up on the [Data] lines of
/// the Hughes 0488 LCD driver.
#[derive(Default, Debug, Clone, Copy)]
pub enum OutputPla {
    /// The O output is simply forwarded through to the LCD driver.
    Normal,
    /// The O output is reversed then sent to the LCD driver.
    #[default]
    Reversed,
}

impl OutputPla {
    /// Modify the O output of the TMS1100 into the [Data] input of the LCD driver.
    #[must_use]
    pub fn modify(&self, o: OutputO) -> DataLine {
        match self {
            OutputPla::Normal => DataLine(u4::new(o.0.value() & 0xf)),
            OutputPla::Reversed => DataLine(u4::new(o.0.value() & 0xf).reverse_bits()),
        }
    }
}

/// The cartridge-specific settings.
#[derive(Debug, Clone, Copy)]
pub struct CartridgeSpecific {
    /// The charge/pulse information of the rotary controller.
    pub charge: Charge,
    /// The decode PLA for the O output of the TMS1100.
    pub output_pla: OutputPla,
    /// A flag determining if the rotary controller is enabled.
    pub rotary_enabled: bool,
}

impl Default for CartridgeSpecific {
    fn default() -> Self {
        Self {
            charge: Default::default(),
            output_pla: Default::default(),
            rotary_enabled: true,
        }
    }
}
