//! Emulation of the TMS1100's internal (onboard) ROM and RAM chips.

use arbitrary_int::{u11, u4, u7};

/// An onboard (2048 x 8bit) Read Only Memory (ROM) chip.
#[derive(Debug, Clone)]
pub struct Rom {
    /// The inner chip data.
    pub data: [u8; 0x800],
    /// The current memory address of the chip.
    pub addr: u11,
}

impl Rom {
    /// Create a new (uninitialized) ROM chip.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: [0; 0x800],
            addr: u11::new(0),
        }
    }

    /// Read from this ROM chip at its specified memory address.
    #[must_use]
    pub fn read(&self) -> u8 {
        self.data[self.addr.value() as usize & 0x7ff]
    }
}

impl Default for Rom {
    fn default() -> Self {
        Self::new()
    }
}

/// An onboard (128 x 4bit) Random Access Memory (RAM) chip.
#[derive(Debug, Clone)]
pub struct Ram {
    /// The inner chip data.
    pub data: [u4; 0x80],
    /// The current memory address of the chip.
    pub addr: u7,
}

impl Ram {
    /// Create a new (uninitialized) RAM chip.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: [u4::new(0); 0x80],
            addr: u7::new(0),
        }
    }

    /// Read from this RAM chip at its specified memory address.
    #[must_use]
    pub fn read(&self) -> u4 {
        self.data[self.addr.value() as usize & 0x7f]
    }

    /// Write to this ROM chip at the specified memory address.
    pub fn write(&mut self, value: u4) {
        self.data[self.addr.value() as usize & 0x7f] = value;
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}
