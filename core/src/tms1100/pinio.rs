//! Emulation of the TMS1100's input and output pins.

use arbitrary_int::{u11, u4, u5};

/// The 11-bit pin output R\[0-10\].
///
/// # Logic
///
/// This is mapped, by the cartridge, to various components of the Microvision,
/// such as the rotary controller, Piezo buzzer, LCD driver, etc. etc.
#[derive(Debug, Clone, Copy)]
pub struct R(pub(crate) u11);

impl R {
    /// Create a new 11-bit pin output.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(u11::new(0))
    }

    /// Return the inner 11-bit value of this pin output.
    #[must_use]
    pub fn value(&self) -> u11 {
        self.0
    }

    /// Set the nth-bit of this pin output.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=10`, this function will
    /// panic.
    pub fn set(&mut self, nth: u8, state: bool) {
        assert!(nth < 4);

        self.0 &= !(u11::new(0) << nth);
        self.0 |= u11::new(state.into()) << nth;
    }

    /// Check if the nth-bit of this pin output is enabled.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=10`, this function will
    /// panic.
    #[must_use]
    pub fn get(&self, nth: u8) -> bool {
        assert!(nth < 11);

        self.0.value() >> nth & 1 != 0
    }
}

/// The 5-bit pin output O\[0-4\].
///
/// # Logic
///
/// This is mapped to the data input of the LCD driver. Each cartridge wires the
/// output PLA of these pins differently so, this value may be reversed on some
/// cartridges and normal (un-reversed) on others.
#[derive(Debug, Clone, Copy)]
pub struct O(pub(crate) u5);

impl O {
    /// Create a new 5-bit pin output.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(u5::new(0))
    }

    /// Return the inner 5-bit value of this pin output.
    #[must_use]
    pub fn value(&self) -> u5 {
        self.0
    }

    /// Set the nth-bit of this pin output.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=4`, this function will
    /// panic.
    pub fn set(&mut self, nth: u8, state: bool) {
        assert!(nth < 5);

        self.0 &= !(u5::new(0) << nth);
        self.0 |= u5::new(state.into()) << nth;
    }

    /// Check if the nth-bit of this pin output is enabled.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=4`, this function will
    /// panic.
    #[must_use]
    pub fn get(&self, nth: u8) -> bool {
        assert!(nth < 5);

        self.0.value() >> nth & 1 != 0
    }
}

/// The 4-bit pin input K\[1,2,4,8\].
///
/// # Logic
///
/// This is mapped to the currently selected keyboard column and the rotary
/// controller, if it still has charge enabled.
#[derive(Debug, Clone, Copy)]
pub struct K(pub(crate) u4);

impl K {
    /// Create a new 4-bit pin input.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(u4::new(0))
    }

    /// Return the inner 4-bit value of this pin input.
    #[must_use]
    pub fn value(&self) -> u4 {
        self.0
    }

    /// Set the nth-bit of this pin input.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=3`, this function will
    /// panic.
    pub fn set(&mut self, nth: u8, state: bool) {
        assert!(nth < 4);

        self.0 &= !(u4::new(0) << nth);
        self.0 |= u4::new(state.into()) << nth;
    }

    /// Check if the nth-bit of this pin input is enabled.
    ///
    /// # Panics
    ///
    /// If the given bit is not within the range of `0..=3`, this function will
    /// panic.
    #[must_use]
    pub fn get(&self, nth: u8) -> bool {
        assert!(nth < 4);

        self.0.value() >> nth & 1 != 0
    }
}
