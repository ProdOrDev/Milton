//! Emulation of the Microvision's 3x4 keypad.

/// The location of a key on the Microvision's 3x4 keypad.
#[derive(Debug, Clone, Copy)]
pub enum Key {
    /// The key in column `0` on row `0`.
    At0x0,
    /// The key in column `0` on row `1`.
    At0x1,
    /// The key in column `0` on row `2`.
    At0x2,
    /// The key in column `0` on row `3`.
    At0x3,
    /// The key in column `1` on row `0`.
    At1x0,
    /// The key in column `1` on row `1`.
    At1x1,
    /// The key in column `1` on row `2`.
    At1x2,
    /// The key in column `1` on row `3`.
    At1x3,
    /// The key in column `2` on row `0`.
    At2x0,
    /// The key in column `2` on row `1`.
    At2x1,
    /// The key in column `2` on row `2`.
    At2x2,
    /// The key in column `2` on row `3`.
    At2x3,
}

impl Key {
    /// Return the row/column offsets of this key location.
    ///
    /// The return value of this function is structured as `(row, col)`.
    #[must_use]
    pub fn pos(&self) -> (usize, usize) {
        match self {
            Self::At0x0 => (0, 0),
            Self::At0x1 => (1, 0),
            Self::At0x2 => (2, 0),
            Self::At0x3 => (3, 0),
            Self::At1x0 => (0, 1),
            Self::At1x1 => (1, 1),
            Self::At1x2 => (2, 1),
            Self::At1x3 => (3, 1),
            Self::At2x0 => (0, 2),
            Self::At2x1 => (1, 2),
            Self::At2x2 => (2, 2),
            Self::At2x3 => (3, 2),
        }
    }
}

/// An abstract (frontend agnostic) 3x4 keypad.
pub trait Api {
    /// Return the status of the given [`Key`].
    #[must_use]
    fn get(&self, key: Key) -> bool;
}
