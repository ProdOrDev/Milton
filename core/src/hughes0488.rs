//! Emulation of the Hughes 0488 LCD driver.
//!
//! ## Links
//!
//! - Random Notes: <http://studio2.org.uk/studio2/mv/HughesNotes.pdf>
//! - Driver Manual: <http://studio2.org.uk/studio2/mv/Hughes0488LCDDriver.pdf>

use arbitrary_int::{u3, u4};

/// The 16 row output connections of the Hughes 0488.
///
/// To update the pixels of row N, the corresponding bit is enabled
/// in this output line, then the current [Column] data in copied onto
/// row N of the LCD.
#[derive(Default, Debug, Clone, Copy)]
pub struct Row(pub u16);

/// The 16 column output connections of the Hughes 0488.
///
/// Alone this output does nothing, however when it is combined
/// with the [Row] output, the current row will be updated using
/// the data on this output connection.
#[derive(Default, Debug, Clone, Copy)]
pub struct Column(pub u16);

/// The 4 data input lines D\[0-3\].
///
/// These control the value that is next written to the internal
/// address latches of the Hughes 0488.
#[derive(Default, Debug, Clone, Copy)]
pub struct Data(pub u4);

/// The 8 sets of 4-bit internal address latches.
///
/// This acts as an intermediary storage for the 16 row drivers
/// and the 16 column drivers before they are output by the driver.
#[derive(Default, Debug, Clone)]
pub struct Latches {
    /// The inner latch data.
    pub data: [u4; 8],
    /// The current address latch.
    pub counter: u3,
}

/// The latch pulse input line.
///
/// When this line is pulled high the 8 latch address counter is reset.
#[derive(Default, Debug, Clone, Copy)]
pub struct LatchPulse(pub bool);

/// The not data clock input line.
///
/// When this line is disabled (set to 0) the value of the [Data] line
/// is written to the next available address latch.
#[derive(Default, Debug, Clone, Copy)]
pub struct NotDataClock(pub bool);

/// An emulated Hughes 0488 LCD driver.
#[derive(Default, Debug, Clone)]
pub struct Hughes0488 {
    /// The 4 data input lines D\[0-3\].
    pub data: Data,
    /// The latch pulse input line.
    pub pulse: LatchPulse,
    /// The not data clock input line.
    pub not_clock: NotDataClock,

    /// The 8 sets of 4-bit internal address latches.
    pub latches: Latches,

    /// The 16 row output connections.
    pub row: Row,
    /// The 16 column output connections.
    pub col: Column,
}

impl Hughes0488 {
    /// Update the Hughes 0488 LCD driver.
    pub fn clock(&mut self, data: Data, pulse: LatchPulse, not_clock: NotDataClock) {
        // On the rising edge of !DATA CLK, bump the address latch counter.
        if !self.not_clock.0 && not_clock.0 {
            self.latches.counter = self.latches.counter.wrapping_add(u3::new(1))
        }
        self.not_clock = not_clock;

        self.pulse = pulse;

        // If !DATA CLK is low, load the address latch.
        if !self.not_clock.0 {
            self.latches.data[self.latches.counter.value() as usize & 7] = data.0;
        }

        // If latch pulse is high and !DATA CLK is high, transfer the
        // address latches to the row and column output latches.
        if self.pulse.0 && self.not_clock.0 {
            self.row.0 = self.latches.data[0..4]
                .iter()
                .fold(0u16, |acc, x| (acc << 4) | x.value() as u16);

            self.col.0 = self.latches.data[4..8]
                .iter()
                .fold(0u16, |acc, x| (acc << 4) | x.value() as u16);
        }

        // If latch pulse is high, reset the address latch counter.
        if self.pulse.0 {
            self.latches.counter = u3::new(0);
        }
    }
}
