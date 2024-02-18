//! The frontend agnostic, #\[no-std\], emulator core of Milton.

#![forbid(missing_docs)]
#![no_std]

pub mod buzzer;
pub mod cartridge;
pub mod common;
pub mod display;
pub mod keypad;
pub mod rotary;
pub mod tms1100;

use buzzer::Buzzer;
use cartridge::Cartridge;
use common::{Interface, Ms};
use display::Hughes0488;
use keypad::Key;
use rotary::Rotary;
use tms1100::{pinio, Tms1100};

use arbitrary_int::u4;

/// An emulated (Milton Bradley) Microvision handheld.
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
            elapsed: Ms(0),
        }
    }

    /// Completely reset this console.
    ///
    /// A real Microvision does not have external reset functionality,
    /// this function exists to simply clarify cases where the console
    /// is effectively reset for an event, e.g. loading a new ROM.
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.driver.reset();
        self.buzzer.reset();
        self.rotary.reset();
        self.elapsed = Ms(0);
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
    #[allow(clippy::needless_pass_by_value)]
    pub fn clock<L, B, K, R>(&mut self, cart: &mut Cartridge, hardware: Interface<L, B, K, R>)
    where
        L: display::Api,
        B: buzzer::Api,
        K: keypad::Api,
        R: rotary::Api,
    {
        /// Read a column of keys from the given keyboard.
        #[rustfmt::skip]
        fn read_column<K>(kb: &K, k: &mut pinio::K, keys: [Key; 4])
        where
            K: keypad::Api,
        {
            if kb.get(keys[0]) { k.set(3, true) };
            if kb.get(keys[1]) { k.set(2, true) };
            if kb.get(keys[2]) { k.set(1, true) };
            if kb.get(keys[3]) { k.set(0, true) };
        }

        // The amount of microseconds every hz (clock) at 100khz takes.
        self.elapsed.offset(Ms(10));

        // Update the TMS1100 micro-processor.
        self.cpu.clock(&cart.rom, &mut cart.ram);

        // The R output of the TMS1100.
        let control = self.cpu.r;

        // Update the K input of the TMS1100.
        let mut k = pinio::K::new();

        // Pin 10 of the R output connects to the left column of the keyboard.
        if control.get(10) {
            read_column(
                hardware.keypad,
                &mut k,
                [Key::At0x0, Key::At0x1, Key::At0x2, Key::At0x3],
            );
        }
        // Pin 9 of the R output connects to the middle column of the keyboard.
        if control.get(9) {
            read_column(
                hardware.keypad,
                &mut k,
                [Key::At1x0, Key::At1x1, Key::At1x2, Key::At1x3],
            );
        }
        // Pin 8 of the R output connects to the right column of the keyboard.
        if control.get(8) {
            read_column(
                hardware.keypad,
                &mut k,
                [Key::At2x0, Key::At2x1, Key::At2x2, Key::At2x3],
            );
        }
        if cart.settings.rotary_enabled {
            k.0 &= u4::new(7);
            // If the charging circuit of the rotary controller has ended (timed out)
            // set the K8 line.
            if self.rotary.charge.value() && self.rotary.charge_end.is_before(self.elapsed) {
                k.set(3, true);
            }
        }

        // Update the Hughes 0488 LCD driver.
        self.driver.clock(
            cart.settings.output_pla.modify(self.cpu.o),
            control.get(6).into(),
            control.get(7).into(),
            hardware.display,
        );

        // Update the Piezo buzzer.
        self.buzzer.clock(control.get(0).into(), self.elapsed);

        // Update the rotary controller.
        self.rotary
            .clock(control.get(2).into(), self.elapsed, cart, hardware.rotary);
    }

    /// Synchronize this console.
    ///
    /// This does not "run" anything in the console, it simply synchronizes
    /// the values output to certain hardware. To actually "run" the console
    /// use [clock](Self::clock).
    ///
    /// # Timing
    ///
    /// This function should be called at the end of every frame.
    #[allow(clippy::needless_pass_by_value)]
    pub fn sync<L, B, K, R>(&mut self, hardware: Interface<L, B, K, R>)
    where
        L: display::Api,
        B: buzzer::Api,
        K: keypad::Api,
        R: rotary::Api,
    {
        self.buzzer.sync(hardware.buzzer);
    }
}
