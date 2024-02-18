//! Emulation of the Microvision's Piezo buzzer.

use crate::common::{line_type, Ms};

line_type! {
    /// The buzzer pulse line.
    ///
    /// # Logic
    ///
    /// When pulsed, this line instructs the buzzer hardware to wait until the
    /// next pulse, then calculate the effective pitch (frequency) to play at
    /// based upon the timing gap between the pulses.
    BuzzerPulse
}

/// An emulated Piezo buzzer.
#[derive(Debug, Clone)]
pub struct Buzzer {
    /// The buzzer pulse line.
    pub pulse: BuzzerPulse,
    /// The number of times the buzzer pulse line has been triggered.
    pub pulse_times: usize,
    /// The start of the buzzer pulse period.
    pub start: Ms,
    /// The end of the buzzer pulse period.
    pub end: Ms,
}

impl Buzzer {
    /// Create a new Piezo buzzer.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            pulse_times: 0,
            pulse: false.into(),
            start: Ms(0),
            end: Ms(0),
        }
    }

    /// Reset this Piezo buzzer.
    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this Piezo buzzer.
    ///
    /// # Logic
    ///
    /// This updates the buzzer's timing logic using the pulse line.
    pub(crate) fn clock(&mut self, pulse: BuzzerPulse, current_time: Ms) {
        if self.pulse.update_rising(pulse) {
            if self.pulse_times == 0 {
                self.start = current_time;
            } else {
                self.end = current_time;
            }
            self.pulse_times += 1;
        }
    }

    /// Synchronize this Piezo buzzer.
    ///
    /// # Logic
    ///
    /// Based upon the number of pules that have occurred this frame, the
    /// buzzer with instruct the frontend to play/stop sound, setting the
    /// desired frequency accordingly.
    pub(crate) fn sync<A>(&mut self, frontend: &mut A)
    where
        A: Api,
    {
        if self.pulse_times >= 2 {
            let period = self.start.elapsed(self.end);
            let pitch = (self.pulse_times - 1) * 1_000_000 / period.0;
            if (50..2400).contains(&pitch) {
                frontend.enable(pitch);
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
pub trait Api {
    /// Enable the sound output of this buzzer.
    ///
    /// This is given the pitch (or frequency) to play tje sound at.
    fn enable(&mut self, pitch: usize);

    /// Disable the sound output of this buzzer.
    fn disable(&mut self);
}
