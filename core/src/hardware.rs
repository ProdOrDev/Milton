//! Emulation of the Microvision's external (user perceived) hardware.

use crate::{hughes0488::Data, tms1100::OutputO};
use arbitrary_int::u4;

/// The turn percentage of a rotary controller.
#[derive(Default, Debug, Clone, Copy)]
pub struct Percentage(pub(crate) usize);

impl Percentage {
    /// Create a new rotary percentage value.
    ///
    /// This interface expects the end user to implement the rotary controller
    /// in terms of `0%` to `100%`. If the value returned is greater than `100`
    /// it will be truncated back to `100`.
    #[must_use]
    pub fn new(amount: usize) -> Percentage {
        Percentage(amount.min(100))
    }

    /// Return the inner percentage value.
    #[must_use]
    pub fn get(&self) -> usize {
        self.0
    }
}

/// An abstract (non implementation specific) rotary controller/paddle.
pub trait Rotary {
    /// Return the turn percentage of the controller.
    #[must_use]
    fn get_value(&self) -> Percentage;
}

/// An abstract (non implementation specific) 4x3 keyboard.
pub trait Keyboard {
    /// Return the status of the key at the given row/column position.
    #[must_use]
    fn get_key(&self, row: usize, col: usize) -> bool;
}

/// An abstract (non implementation specific) Piezo buzzer.
pub trait Buzzer {
    /// Disable the sound coming out of this buzzer.
    fn disable_sound(&mut self);

    /// Enable the sound coming out of this buzzer.
    fn enable_sound(&mut self);

    /// Set the pitch (frequency) of this buzzer.
    fn set_pitch(&mut self, pitch: usize);
}

/// An abstract (non implementation specific) 16x16 LCD display.
pub trait Lcd {
    /// Enable the pixel at the given X and Y screen coordinates.
    ///
    /// This coordinates start at the upper left corner of the LCD display,
    /// so X = 2, Y = 3 would be the pixel on the 4th row and 3rd column.
    fn enable_pixel(&mut self, x: usize, y: usize);
}

/// The charge/pulse information of the rotary controller.
///
/// This is used to calculate the effective time until a charge supplied
/// to the rotary controller/paddle charge would end.
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
    pub fn modify(&self, o: OutputO) -> Data {
        match self {
            OutputPla::Normal => Data(u4::new(o.0.value() & 0xf)),
            OutputPla::Reversed => Data(u4::new(o.0.value() & 0xf).reverse_bits()),
        }
    }
}

/// The cartridge-specific hardware wirings of certain features.
#[derive(Debug, Clone, Copy)]
pub struct CartridgeSpecific {
    /// The charge/pulse information of the rotary controller.
    pub charge_info: ChargeInfo,
    /// The decode PLA for the O output of the TMS1100.
    pub output_pla: OutputPla,
    /// A flag determining if the rotary controller is enabled.
    pub rotary_enabled: bool,
}

impl Default for CartridgeSpecific {
    fn default() -> Self {
        Self {
            charge_info: Default::default(),
            output_pla: Default::default(),
            rotary_enabled: true,
        }
    }
}

/// An abstract (non implementation specific) hardware interface.
#[derive(Debug)]
pub struct Interface<'a, L, B, K, R>
where
    L: Lcd,
    B: Buzzer,
    K: Keyboard,
    R: Rotary,
{
    /// The 16x16 LCD display.
    pub lcd: &'a mut L,
    /// The Piezo buzzer.
    pub buzzer: &'a mut B,
    /// The 4x3 keyboard.
    pub keyboard: &'a K,
    /// The rotary controller/paddle.
    pub rotary: &'a R,
    /// The cartridge-specific hardware wirings.
    pub cartridge_specific: CartridgeSpecific,
}
