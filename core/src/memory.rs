//! Emulation of the on-cartridge (more specifically on the TMS1100) ROM/RAM chips.

use arbitrary_int::{u11, u4, u7};

/// The onboard 2kb (2048 x 8bit) Read Only Memory (ROM) chip.
#[derive(Debug, Clone)]
pub struct Rom {
    /// The memory data within the chip.
    pub data: [u8; 0x800],
}

impl Default for Rom {
    /// Create a new (uninitialized) ROM chip.
    fn default() -> Self {
        Self { data: [0; 0x800] }
    }
}

impl Rom {
    /// Read from this ROM chip at the specified memory address.
    #[must_use]
    pub fn read(&self, addr: u11) -> u8 {
        self.data[addr.value() as usize & 0x7ff]
    }

    /// Return the checksum of this ROM image.
    #[must_use]
    pub fn checksum(&self) -> u16 {
        self.data
            .iter()
            .copied()
            .map(u16::from)
            .fold(0, u16::wrapping_add)
    }
}

/// The onboard 64b (128 x 4bit) Random Access Memory (RAM) chip.
#[derive(Debug, Clone)]
pub struct Ram {
    /// The memory data within the chip.
    pub data: [u4; 0x80],
}

impl Default for Ram {
    /// Create a new (uninitialized) RAM chip.
    fn default() -> Self {
        Self {
            data: [u4::new(0); 0x80],
        }
    }
}

impl Ram {
    /// Read from this RAM chip at the specified memory address.
    #[must_use]
    pub fn read(&self, addr: u7) -> u4 {
        self.data[addr.value() as usize & 0x7f]
    }

    /// Write to this RAM chip at the specified memory address.
    pub fn write(&mut self, addr: u7, value: u4) {
        self.data[addr.value() as usize & 0x7f] = value;
    }
}
