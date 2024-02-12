//! Common data types and utilities.

use crate::{buzzer, keyboard, lcd, rotary};

/// A boolean input/output signal line.
///
/// This is used for inter-chip communication and to transfer state from one
/// component to another in an elegant and robust manner.
#[derive(Debug, Clone, Copy)]
pub struct Line(bool);

impl Line {
    /// Initialize a new input/output line with the given signal.
    #[must_use]
    pub fn new(signal: bool) -> Self {
        Self(signal)
    }

    /// Return the inner signal value of this line.
    #[must_use]
    pub fn value(&self) -> bool {
        self.0
    }

    /// Update the signal of the current input/output line with the signal
    /// from another line.
    ///
    /// This returns a boolean indicating if a rising edge has occurred.
    #[must_use]
    pub fn update_rising(&mut self, other: Self) -> bool {
        let rising = !self.0 && other.0;
        self.0 = other.0;
        rising
    }
}

/// A unit of time, measured in **micro**-seconds.
pub type Ms = usize;

/// An abstract (frontend agnostic) hardware interface.
pub struct Hardware<'a, L, B, K, R>
where
    L: lcd::Agnostic,
    B: buzzer::Agnostic,
    K: keyboard::Agnostic,
    R: rotary::Agnostic,
{
    /// The 16x16 LCD display.
    pub lcd: &'a mut L,
    /// The Piezo buzzer.
    pub buzzer: &'a mut B,
    /// The 3x4 keyboard.
    pub keyboard: &'a K,
    /// The rotary controller/paddle.
    pub rotary: &'a R,
}
