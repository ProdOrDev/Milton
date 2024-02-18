//! Emulation of the Microvision's rotary controller.

use crate::{
    cartridge::{settings::ChargeInfo, Cartridge},
    common::{line_type, Ms},
};

line_type! {
    /// The rotary charge line.
    ///
    /// # Logic
    ///
    /// When pulsed, this lines will supply charge to the rotary controller for
    /// a cartridge-specific amount of time (in microseconds).
    ChargePulse
}

/// An emulated rotary controller.
#[derive(Debug, Clone)]
pub struct Rotary {
    /// The point in time when the charge supplied to this rotary controller is
    /// expected to end.
    pub charge_end: Ms,
    /// The rotary charge line.
    pub charge: ChargePulse,
}

impl Rotary {
    /// Create a new rotary controller.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            charge_end: Ms(0),
            charge: false.into(),
        }
    }

    /// Reset this rotary controller.
    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this rotary controller.
    ///
    /// # Logic
    ///
    /// This updates the rotary controller's charging logic and timing circuit.
    pub fn clock<A>(
        &mut self,
        charge: ChargePulse,
        current_time: Ms,
        cart: &Cartridge,
        frontend: &A,
    ) where
        A: Api,
    {
        if self.charge.update_rising(charge) {
            let ChargeInfo { offset, scale } = cart.settings.charge_info;

            self.charge_end = Ms(current_time.0 + offset + scale * frontend.turn().0 / 10);
        }
    }
}

/// The turn percentage (`0-100`) of a rotary controller.
#[derive(Debug, Clone, Copy)]
pub struct Percentage(pub(crate) usize);

impl Percentage {
    /// Create a new percentage value.
    ///
    /// # Panics
    ///
    /// If the given value does not fall within the range of `0..=100`, e.g. the
    /// value is greater than `100`, this function will panic.
    #[must_use]
    pub fn new(amount: usize) -> Self {
        assert!(amount < 101, "The given percentage value is too large");

        Self(amount)
    }

    /// Return the inner value of this percentage.
    #[must_use]
    pub fn value(&self) -> usize {
        self.0
    }
}

/// An abstract (frontend agnostic) rotary controller.
pub trait Api {
    /// Return the current turn percentage of this controller.
    #[must_use]
    fn turn(&self) -> Percentage;
}
