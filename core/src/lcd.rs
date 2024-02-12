//! Emulation of the Microvision's LCD display and Hughes 0488 LCD driver.
//!
//! ## Links
//!
//! - Random Notes: <http://studio2.org.uk/studio2/mv/HughesNotes.pdf>
//! - Driver Manual: <http://studio2.org.uk/studio2/mv/Hughes0488LCDDriver.pdf>

use crate::common::Line;
use arbitrary_int::{u3, u4};

/// The 16 row output connections of the Hughes 0488.
///
/// To update the pixels of row N, the corresponding bit is enabled
/// in this output line, then the current [Column] data in copied onto
/// row N of the LCD.
#[derive(Debug, Clone, Copy)]
pub struct Row(pub u16);

/// The 16 column output connections of the Hughes 0488.
///
/// Alone, this output does nothing, however when it is combined
/// with the [Row] output, the current row will be updated using
/// the data on this output connection.
#[derive(Debug, Clone, Copy)]
pub struct Column(pub u16);

/// The 4 data input/control lines D\[0-3\].
///
/// The value on these control lines is written to the internal
/// address latches of the Hughes 0488 on the next data clock.
#[derive(Debug, Clone, Copy)]
pub struct DataLine(pub u4);

/// The latch pulse input line.
///
/// When pulsed, this instructs the Hughes 0488 to move the address
/// latches to the [Row] and [Column] outputs if the data clock is
/// enabled, additionally resetting the address latch counter.
pub type LatchPulse = Line;

/// The not data clock input line.
///
/// When this line is pulled low (set to 0), the value on the [DataLine]
/// is written to the next available address latch.
pub type NotDataClock = Line;

/// The 8 internal 4-bit address latches of the Hughes 0488.
///
/// These act as an intermediary storage for the [Row] and [Column] outputs
/// before they are clocked to the output lines by the driver.
#[derive(Debug, Clone)]
pub struct Latches {
    /// The inner (unguarded) latch data.
    pub data: [u4; 8],
    /// The current address latch.
    pub counter: u3,
}

/// An emulated Hughes 0488 LCD driver.
#[derive(Debug, Clone)]
pub struct Hughes0488 {
    /// The 4 data input/control lines D\[0-3\].
    pub data: DataLine,
    /// The latch pulse input line.
    pub pulse: LatchPulse,
    /// The not data clock input line.
    pub not_clock: NotDataClock,
    /// The 8 internal 4-bit address latches.
    pub latches: Latches,
    /// The 16 row output connections.
    pub row: Row,
    /// The 16 column output connections.
    pub col: Column,
}

impl Hughes0488 {
    /// Create a new LCD driver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: DataLine(u4::new(0)),
            pulse: LatchPulse::new(false),
            not_clock: NotDataClock::new(false),
            latches: Latches {
                data: [u4::new(0); 8],
                counter: u3::new(0),
            },
            row: Row(0),
            col: Column(0),
        }
    }

    /// Reset this LCD driver.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this LCD driver.
    pub fn clock<A>(
        &mut self,
        data: DataLine,
        pulse: LatchPulse,
        not_clock: NotDataClock,
        frontend: &mut A,
    ) where
        A: Agnostic,
    {
        if self.not_clock.update_rising(not_clock) {
            self.latches.counter = self.latches.counter.wrapping_add(u3::new(1))
        }

        self.pulse = pulse;

        if !self.not_clock.value() {
            self.latches.data[self.latches.counter.value() as usize & 7] = data.0;
        }

        if self.pulse.value() && self.not_clock.value() {
            self.row.0 = self.latches.data[0..4]
                .iter()
                .fold(0u16, |acc, x| (acc << 4) | x.value() as u16);

            self.col.0 = self.latches.data[4..8]
                .iter()
                .fold(0u16, |acc, x| (acc << 4) | x.value() as u16);

            // If all the row indexes or the column data are zero, nothing will
            // be updated.
            if self.row.0 == 0 || self.col.0 == 0 {
                return;
            }

            for y in 0..=15 {
                // If the current row index is not set, nothing will be updated.
                if self.row.0 >> y & 1 == 0 {
                    continue;
                }

                for x in 0..=15 {
                    // Pixels are not set/unset through the row/column lines,
                    // instead they are enabled and eventually decay to off
                    // over a brief period of time.
                    if self.col.0 >> x & 1 != 0 {
                        frontend.enable_pixel(x, y);
                    }
                }
            }
        }

        if self.pulse.value() {
            self.latches.counter = u3::new(0);
        }
    }
}

/// An abstract (frontend agnostic) 16x16 LCD display.
pub trait Agnostic {
    /// Enable the pixel at the given X and Y screen coordinates.
    ///
    /// ## Screen Mapping
    ///
    /// The coordinates supplied to this function begin at the upper left
    /// corner of the LCD display, so X = 2, Y = 3 would be the pixel on
    /// the 4th row and 3rd column.
    fn enable_pixel(&mut self, x: usize, y: usize);
}
