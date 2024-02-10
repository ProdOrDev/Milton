#![doc = include_str!("../../README.md")]
#![forbid(missing_docs)]

pub mod hardware;
pub mod hughes0488;
pub mod memory;
pub mod tms1100;

use hughes0488::Hughes0488;
use tms1100::Tms1100;

/// An emulated Milton Bradley Microvision console/handheld.
#[derive(Default, Debug, Clone)]
pub struct Console {
    /// The TMS1100 micro-processor.
    pub cpu: Tms1100,
    /// The Hughes 0488 LCD driver.
    pub lcd_driver: Hughes0488,
}
