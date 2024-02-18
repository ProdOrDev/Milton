//! Emulation of the Microvision's game cartridges.
//!
//! # Trivia
//!
//! The Microvision is the first console that used/supported interchangeable
//! cartridges and is therefore, in a sense, reprogrammable.

use crate::tms1100::mem::{Ram, Rom};

/// Cartridge-specific settings/features.
///
/// Due to the Microvision's cartridge-centric design, not all cartridges obey
/// the same "settings" as others. Some modify the charge supplied to chips like
/// the rotary controller and others reverse the output decoder of the TMS1100.
pub mod settings {
    use crate::{display::DataLine, tms1100::pinio};

    use arbitrary_int::u4;

    /// The settings of the charge line to the rotary controller.
    ///
    /// This is used to calculate the effective time until a charge supplied to
    /// the rotary controller/paddle would end.
    #[derive(Debug, Clone, Copy)]
    pub struct ChargeInfo {
        /// The value to offset the end time by.
        pub offset: usize,
        /// The value to scale the end time by.
        pub scale: usize,
    }

    impl Default for ChargeInfo {
        fn default() -> Self {
            Self {
                offset: 600,
                scale: 65,
            }
        }
    }

    /// The decode PLA for the O output of the TMS1100.
    ///
    /// This is used for decided what will end up on the [`DataLine`] lines of
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
        /// Modify the O output of the TMS1100 into the [`DataLine`] input of
        /// the LCD driver.
        #[must_use]
        pub(crate) fn modify(self, o: pinio::O) -> DataLine {
            match self {
                Self::Normal => DataLine(u4::new(o.0.value() & 0xf)),
                Self::Reversed => DataLine(u4::new(o.0.value() & 0xf).reverse_bits()),
            }
        }
    }

    /// The cartridge-specific settings.
    #[derive(Debug, Clone, Copy)]
    pub struct Settings {
        /// The settings of the charge line to the rotary controller.
        pub charge_info: ChargeInfo,
        /// The decode PLA for the O output of the TMS1100.
        pub output_pla: OutputPla,
        /// A flag determining if the rotary controller is enabled.
        pub rotary_enabled: bool,
    }
}

/// An interchangeable game-cartridge for the Microvision.
#[derive(Debug, Clone)]
pub struct Cartridge {
    /// The ROM data.
    pub rom: Rom,
    /// The RAM data.
    pub ram: Ram,
    /// The game-specific settings of this cartridge.
    pub settings: settings::Settings,
}
