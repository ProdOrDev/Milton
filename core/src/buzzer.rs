//! Emulation of the Microvision's Piezo buzzer.

use crate::common::{Line, Ms};

/// The buzzer pulse line.
///
/// When pulsed, this line instructs the buzzer hardware to wait until the
/// next pulse, then calculate the effective pitch (frequency) to play at
/// based upon the timing gap between the pulses.
pub type BuzzerPulse = Line;

/// An emulated Piezo buzzer.
#[derive(Debug, Clone)]
pub struct Buzzer {
    /// The number of times the buzzer pulse line has been triggered.
    ///
    /// This resets once the Microvision synchronizes on a new frame.
    pub pulse_times: usize,
    /// The buzzer pulse line.
    pub pulse: BuzzerPulse,
    /// The start of the buzzer period.
    pub start: Ms,
    /// The end of the buzzer period.
    pub end: Ms,
}

impl Buzzer {
    /// Create a new Piezo buzzer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pulse_times: 0,
            pulse: BuzzerPulse::new(false),
            start: 0,
            end: 0,
        }
    }

    /// Reset this Piezo buzzer.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this Piezo buzzer.
    pub fn clock(&mut self, pulse: BuzzerPulse, current_time: Ms) {
        if self.pulse.update_rising(pulse) {
            if self.pulse_times == 0 {
                self.start = current_time;
            } else {
                self.end = current_time;
            }
            self.pulse_times += 1;
        }
    }

    /// Synchronize this Piezo buzzer to a frontend implementation.
    pub fn sync<A>(&mut self, frontend: &mut A)
    where
        A: Agnostic,
    {
        if self.pulse_times >= 2 {
            let period = self.end - self.start;
            let pitch = (self.pulse_times - 1) * 1000000 / period;
            if (50..2400).contains(&pitch) {
                frontend.set_pitch(pitch);
                frontend.enable();
            } else {
                frontend.disable();
            }
        } else {
            frontend.disable();
        }
        self.pulse_times = 0;
    }
}

/// An abstract (frontend agnostic) Piezo buzzer.
pub trait Agnostic {
    /// Enable the sound output of this buzzer.
    fn enable(&mut self);

    /// Disable the sound output of this buzzer.
    fn disable(&mut self);

    /// Set the pitch (or frequency) of this buzzer.
    ///
    /// This is always followed by a call to [Agnostic::enable] so sound
    /// could be enabled here if a framework deems it necessary.
    fn set_pitch(&mut self, pitch: usize);
}
