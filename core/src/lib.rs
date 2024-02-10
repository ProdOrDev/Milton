#![doc = include_str!("../../README.md")]
#![forbid(missing_docs)]

pub mod hardware;
pub mod hughes0488;
pub mod memory;
pub mod tms1100;

/// An emulated Milton Bradley Microvision.
#[derive(Default, Debug, Clone)]
pub struct Console {}
