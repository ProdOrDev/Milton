//! Emulation of the Microvision's keyboard (pad?).

/// An abstract key location on a 3x4 keyboard.
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
    /// Return the row and column position of this key.
    ///
    /// The returned value is structured: `(row, col)`.
    #[must_use]
    pub fn pos(&self) -> (usize, usize) {
        match self {
            Key::At0x0 => (0, 0),
            Key::At0x1 => (1, 0),
            Key::At0x2 => (2, 0),
            Key::At0x3 => (3, 0),
            Key::At1x0 => (0, 1),
            Key::At1x1 => (1, 1),
            Key::At1x2 => (2, 1),
            Key::At1x3 => (3, 1),
            Key::At2x0 => (0, 2),
            Key::At2x1 => (1, 2),
            Key::At2x2 => (2, 2),
            Key::At2x3 => (3, 2),
        }
    }
}

/// An abstract (frontend agnostic) 3x4 keyboard.
pub trait Agnostic {
    /// Return the state of the given [Key].
    #[must_use]
    fn get(&self, key: Key) -> bool;
}