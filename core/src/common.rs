//! A collection of common (project-wide) data types and utilities.

use crate::{buzzer, display, keypad, rotary};

/// Define a new type-alias representing a distinct signal line.
///
/// This allows function to take a "custom" signal line type, e.g. `LcdPulse`, which
/// allows for easier understanding of the code without much logical overhead.
macro_rules! line_type {
    {$(#[$($doc:tt)*])* $alias:ident} => {
        $(#[$($doc)*])*
        #[allow(clippy::module_name_repetitions)]
        pub type $alias = crate::common::Line;
    };
}
pub(crate) use line_type;

/// A 1-bit (boolean) input/output signal line.
///
/// This is used for inter-chip communication and to transfer state from one
/// component to another in an elegant and robust manner.
#[derive(Debug, Clone, Copy)]
pub struct Line(pub(crate) bool);

impl Line {
    /// Return the inner boolean value representing this signal line.
    #[must_use]
    pub fn value(&self) -> bool {
        self.0
    }

    /// Update the state of this signal line with the signal from another line.
    ///
    /// This returns a boolean indicating if a rising edge, a `0->1` transition,
    /// has occurred.
    #[must_use]
    pub(crate) fn update_rising(&mut self, other: Self) -> bool {
        let rising = !self.0 && other.0;
        self.0 = other.0;
        rising
    }
}

impl From<bool> for Line {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// A unit of time, measured in **micro**-seconds.
///
/// # Note
///
/// This should not be confused with **milli**-seconds, which are `1000`
/// times larger than **micro**-seconds.
#[derive(Debug, Clone, Copy)]
pub struct Ms(pub(crate) usize);

impl Ms {
    /// Return the inner numerical value representing this number of **micro**-seconds.
    #[must_use]
    pub fn value(&self) -> usize {
        self.0
    }

    /// Check if the given time, `self`, comes before `other`.
    #[must_use]
    pub(crate) fn is_before(self, other: Self) -> bool {
        self.0 <= other.0
    }

    /// Return the total amount of **micro**-seconds elapsed between `self` and `end`.
    ///
    /// # Panics
    ///
    /// This function will panic if the given `end` time comes before the time
    /// represented by `self`.
    #[must_use]
    pub(crate) fn elapsed(self, end: Self) -> Self {
        assert!(self.is_before(end));

        Self(end.0 - self.0)
    }

    /// Increment this unit of time by the given amount of **micro**-seconds.
    pub(crate) fn offset(&mut self, amount: Self) {
        self.0 += amount.0;
    }
}

/// An abstract (frontend agnostic) hardware interface.
#[must_use]
pub struct Interface<'a, L, B, K, R>
where
    L: display::Api,
    B: buzzer::Api,
    K: keypad::Api,
    R: rotary::Api,
{
    /// The 16x16 LCD display.
    pub display: &'a mut L,
    /// The Piezo buzzer.
    pub buzzer: &'a mut B,
    /// The 3x4 keypad.
    pub keypad: &'a K,
    /// The rotary controller.
    pub rotary: &'a R,
}
