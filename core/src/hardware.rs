//! Emulation of the Microvision's external (user perceived) hardware, both visible and audible.

/// An abstract (non implementation specific) 16x16 LCD display.
pub trait Lcd {
    /// Enable the pixel at the given X and Y screen coordinates.
    ///
    /// This coordinates start at the upper left corner of the LCD display,
    /// so X = 2, Y = 3 would be the pixel on the 4th row and 3rd column.
    fn enable_pixel(&mut self, x: usize, y: usize);
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

/// An abstract (non implementation specific) 4x3 keyboard.
pub trait Keyboard {
    /// Return the status of the key at the given row/column position.
    fn get_key(&mut self, row: usize, col: usize) -> bool;
}

/// An abstract (non implementation specific) hardware interface.
#[derive(Debug)]
pub struct Interface<'a, L, B, K>
where
    L: Lcd,
    B: Buzzer,
    K: Keyboard,
{
    /// The 16x16 LCD display.
    pub lcd: &'a mut L,
    /// The Piezo buzzer.
    pub buzzer: &'a mut B,
    /// The 4x3 keyboard.
    pub keyboard: &'a mut K,
}
