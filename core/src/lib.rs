#![doc = include_str!("../../README.md")]
#![forbid(missing_docs)]

pub mod hardware;
pub mod hughes0488;
pub mod memory;
pub mod tms1100;

use arbitrary_int::u4;
use hardware::{Buzzer, Interface, Keyboard, Lcd, Rotary};
use hughes0488::{Hughes0488, LatchPulse, NotDataClock};
use memory::{Ram, Rom};
use tms1100::Tms1100;

/// An emulated Milton Bradley Microvision console/handheld.
#[derive(Default, Debug, Clone)]
pub struct Console {
    /// The on-cartridge TMS1100 micro-processor.
    pub cpu: Tms1100,
    /// The Hughes 0488 LCD driver.
    pub driver: Hughes0488,
    /// The on-cartridge 2kb ROM chip.
    pub rom: Rom,
    /// The on-cartridge 64b RAM chip.
    pub ram: Ram,

    /// The number of pulses sent to the speaker line.
    sound_pulses: usize,
    /// The starting time of the speaker line period.
    sound_start: usize,
    /// The ending time of the speaker line period.
    sound_end: usize,
    /// The current status of the speaker line.
    sound_status: bool,

    /// The amount of time until the charge supplied to the rotary
    /// controller ends.
    rotary_charge: usize,
    /// The current status of the rotary charge line.
    rotary_status: bool,

    /// The total amount of microseconds elapsed.
    elapsed: usize,
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
    ///
    /// This actually "runs" the console, updating the micro-processor,
    /// LCD display, etc. etc. To synchronize certain outputs like sound,
    /// use [sync](Self::sync).
    ///
    /// ## Timing
    ///
    /// This function should be called at a rate of 100khz, effectively every
    /// 10 **micro**-seconds.
    pub fn clock<L, B, K, R>(&mut self, hardware: Interface<L, B, K, R>)
    where
        L: Lcd,
        B: Buzzer,
        K: Keyboard,
        R: Rotary,
    {
        // The amount of microseconds every hz at 100khz takes.
        self.elapsed += 10;

        // Update the TMS1100 micro-processor.
        self.cpu.clock(&self.rom, &mut self.ram);

        // Update the K input of the TMS1100.
        let mut new = u4::new(0);
        // The 10th pin of the R output connects to the left column of the keyboard.
        if self.cpu.r.0.value() >> 10 & 1 != 0 {
            new |= read_column(hardware.keyboard, 0);
        }
        // The 9th pin of the R output connects to the middle column of the keyboard.
        if self.cpu.r.0.value() >> 9 & 1 != 0 {
            new |= read_column(hardware.keyboard, 1);
        }
        // The 8th pin of the R output connects to the right column of the keyboard.
        if self.cpu.r.0.value() >> 8 & 1 != 0 {
            new |= read_column(hardware.keyboard, 2);
        }
        if hardware.cartridge_specific.rotary_enabled {
            new &= u4::new(7);
            // If the charging circuit of the rotary controller has ended (timed out)
            // set the K8 line.
            if self.rotary_status && self.rotary_charge < self.elapsed {
                new |= u4::new(8);
            }
        }
        self.cpu.k.0 = new;

        // Update the Hughes 0488 LCD driver.
        self.driver.clock(
            hardware.cartridge_specific.output_pla.modify(self.cpu.o),
            LatchPulse(self.cpu.r.0.value() >> 6 & 1 != 0),
            NotDataClock(self.cpu.r.0.value() >> 7 & 1 != 0),
        );

        // Update the speaker line.
        let new = self.cpu.r.0.value() & 1 != 0;

        // On a 0->1 transition, update the speaker timings.
        if !self.sound_status && new {
            if self.sound_pulses == 0 {
                self.sound_start = self.elapsed;
            } else {
                self.sound_end = self.elapsed;
            }
            self.sound_pulses += 1;
        }
        self.sound_status = new;

        // Update the rotary control line.
        let new = self.cpu.r.0.value() >> 2 & 1 != 0;

        // On a 0->1 transition, update the rotary charge timings.
        if !self.rotary_status && new {
            let info = hardware.cartridge_specific.charge_info;
            self.rotary_charge =
                self.elapsed + info.offset + info.scale * hardware.rotary.get_value().get() / 10;
        }
        self.rotary_status = new;

        // Update the LCD display.
        //
        // If all the row indexes or the column data are zero, nothing will
        // be updated.
        if self.driver.row.0 == 0 || self.driver.col.0 == 0 {
            return;
        }

        for y in 0..=15 {
            // If the current row index is not set, nothing will be updated.
            if self.driver.row.0 >> y & 1 == 0 {
                continue;
            }

            for x in 0..=15 {
                // Pixels are not set/unset through the row/column lines,
                // instead they are enabled and eventually decay to off
                // over a brief period of time.
                if self.driver.col.0 >> x & 1 != 0 {
                    hardware.lcd.enable_pixel(x, y);
                }
            }
        }
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
    pub fn sync<L, B, K, R>(&mut self, hardware: Interface<L, B, K, R>)
    where
        L: Lcd,
        B: Buzzer,
        K: Keyboard,
        R: Rotary,
    {
        // Update the sound/buzzer output.
        //
        // We check if there have been 2 pulses to the speaker line, e.g.
        // a start and end time for the speaker period.
        if self.sound_pulses >= 2 {
            // The time of the sound period is microseconds.
            let ms = self.sound_end - self.sound_start;
            // The effective pitch of the buzzer.
            let pitch = (self.sound_pulses - 1) * 1000000 / ms;
            if (50..2400).contains(&pitch) {
                hardware.buzzer.enable_sound();
                hardware.buzzer.set_pitch(pitch);
            } else {
                hardware.buzzer.disable_sound();
            }
        } else {
            hardware.buzzer.disable_sound();
        }
        self.sound_pulses = 0;
    }
}

/// Read a column of keys as a 4-bit number from the given keyboard.
fn read_column<K>(kb: &K, col: usize) -> u4
where
    K: Keyboard,
{
    let k1 = if kb.get_key(0, col) { 8 } else { 0 };
    let k2 = if kb.get_key(1, col) { 4 } else { 0 };
    let k3 = if kb.get_key(2, col) { 2 } else { 0 };
    let k4 = if kb.get_key(3, col) { 1 } else { 0 };
    u4::new(k1 | k2 | k3 | k4)
}
