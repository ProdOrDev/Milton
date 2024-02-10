#![doc = include_str!("../../README.md")]
#![forbid(missing_docs)]

pub mod hardware;
pub mod hughes0488;
pub mod memory;
pub mod tms1100;

use hardware::{Buzzer, Interface, Keyboard, Lcd, Rotary};
use hughes0488::Hughes0488;
use memory::{Ram, Rom};
use tms1100::Tms1100;

/// An emulated Milton Bradley Microvision console/handheld.
#[derive(Default, Debug, Clone)]
pub struct Console {
    /// The TMS1100 micro-processor.
    pub cpu: Tms1100,
    /// The Hughes 0488 LCD driver.
    pub driver: Hughes0488,
    /// The on-cartridge 2kb ROM chip.
    pub rom: Rom,
    /// The on-cartridge 64b RAM chip.
    pub ram: Ram,
}

impl Console {
    /// Create a new Microvision console.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Completely reset this console.
    ///
    /// A real Microvision does not have external reset functionality,
    /// this function exists to simply clarify cases where the console
    /// is effectively reset for an event, e.g. loading a new ROM.
    pub fn reset(&mut self) {
        *self = Self::new()
    }

    /// Update this console.
    pub fn clock<L, B, K, R>(&mut self, _hardware: Interface<L, B, K, R>)
    where
        L: Lcd,
        B: Buzzer,
        K: Keyboard,
        R: Rotary,
    {
        todo!()
    }
}
