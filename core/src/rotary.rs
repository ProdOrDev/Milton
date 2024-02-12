//! Emulation of the Microvision's rotary controller.

use crate::{
    common::{Line, Ms},
    settings,
};

/// The rotary charge line.
///
/// When pulsed, this lines will supply charge to the rotary controller for
/// a cartridge-specific amount of time (in microseconds).
pub type ChargePulse = Line;

/// An emulated rotary controller.
#[derive(Debug, Clone)]
pub struct Rotary {
    /// The exact point in time when the charge supplied to this rotary
    /// controller with end.
    pub charge_end: Ms,
    /// The rotary charge line.
    pub charge: ChargePulse,
}

impl Rotary {
    /// Create a new rotary controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            charge_end: 0,
            charge: ChargePulse::new(false),
        }
    }

    /// Reset this rotary controller.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this rotary controller.
    pub fn clock<A>(
        &mut self,
        charge: ChargePulse,
        current_time: Ms,
        settings: settings::Charge,
        frontend: &A,
    ) where
        A: Agnostic,
    {
        if self.charge.update_rising(charge) {
            self.charge_end =
                current_time + settings.offset + settings.scale * frontend.turn().value() / 10;
        }
    }
}

/// The turn percentage (0-100) of a rotary controller.
#[derive(Debug, Clone, Copy)]
pub struct Percentage(usize);

impl Percentage {
    /// Create a new percentage value.
    ///
    /// ## Panics
    ///
    /// If the given value does not fall within the range of `0..=100`, e.g. the
    /// value is greater than `100`, this function will panic.
    #[must_use]
    pub fn new(amount: usize) -> Percentage {
        assert!(amount <= 100, "The given percentage value is too large");
        Percentage(amount)
    }

    /// Return the inner value of this percentage.
    #[must_use]
    pub fn value(&self) -> usize {
        self.0
    }
}

/// An abstract (frontend agnostic) rotary controller.
pub trait Agnostic {
    /// Return the turn percentage of this controller.
    #[must_use]
    fn turn(&self) -> Percentage;
}
