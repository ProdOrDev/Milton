#![doc = include_str!("../../README.md")]
#![forbid(missing_docs)]
#![allow(clippy::new_without_default)]

pub mod buzzer;
pub mod common;
pub mod keyboard;
pub mod lcd;
pub mod memory;
pub mod rotary;
pub mod settings;
pub mod tms1100;

use buzzer::Buzzer;
use common::{Hardware, Ms};
use lcd::Hughes0488;
use memory::{Ram, Rom};
use rotary::Rotary;
use settings::Settings;
use tms1100::Tms1100;

/// An emulated Milton Bradley Microvision console/handheld.
#[derive(Debug, Clone)]
pub struct Console {
    /// The on-cartridge TMS1100 micro-processor.
    pub cpu: Tms1100,
    /// The Hughes 0488 LCD driver.
    pub driver: Hughes0488,
    /// The Piezo buzzer.
    pub buzzer: Buzzer,
    /// The rotary controller.
    pub rotary: Rotary,
    /// The total amount of microseconds elapsed.
    pub elapsed: Ms,
}

impl Console {
    /// Create a new Microvision console.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cpu: Tms1100::new(),
            driver: Hughes0488::new(),
            buzzer: Buzzer::new(),
            rotary: Rotary::new(),
            elapsed: 0,
        }
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
    ///
    /// This actually "runs" the console, updating the micro-processor,
    /// LCD display, etc. etc. To synchronize certain outputs like sound,
    /// use [sync](Self::sync).
    ///
    /// ## Timing
    ///
    /// This function should be called at a rate of 100khz, effectively every
    /// 10 **micro**-seconds.
    pub fn clock<L, B, K, R>(
        &mut self,
        rom: &Rom,
        ram: &mut Ram,
        settings: Settings,
        hardware: Hardware<L, B, K, R>,
    ) where
        L: lcd::Agnostic,
        B: buzzer::Agnostic,
        K: keyboard::Agnostic,
        R: rotary::Agnostic,
    {
        // The amount of microseconds every hz (clock) at 100khz takes.
        self.elapsed += 10;

        // Update the TMS1100 micro-processor.
        self.cpu.clock(rom, ram);

        // Update the K input of the TMS1100.
        self.cpu.k.update(
            self.cpu.r,
            self.elapsed,
            settings,
            &self.rotary,
            hardware.keyboard,
        );

        // Update the Hughes 0488 LCD driver.
        self.driver.clock(
            settings.output_pla.modify(self.cpu.o),
            self.cpu.r.latch_pulse(),
            self.cpu.r.not_clock(),
            hardware.lcd,
        );

        // Update the Piezo buzzer.
        self.buzzer.clock(self.cpu.r.buzzer_pulse(), self.elapsed);

        // Update the rotary controller.
        self.rotary.clock(
            self.cpu.r.rotary_charge(),
            self.elapsed,
            settings.charge,
            hardware.rotary,
        );
    }

    /// Synchronize this console.
    ///
    /// This does not "run" anything in the console, it simply synchronizes
    /// the values output to certain hardware. To actually "run" the console
    /// use [clock](Self::clock).
    ///
    /// ## Timing
    ///
    /// This function should be called at the end of every frame.
    pub fn sync<L, B, K, R>(&mut self, hardware: Hardware<L, B, K, R>)
    where
        L: lcd::Agnostic,
        B: buzzer::Agnostic,
        K: keyboard::Agnostic,
        R: rotary::Agnostic,
    {
        self.buzzer.sync(hardware.buzzer);
    }
}
