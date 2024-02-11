//! Emulation of the on-cartridge (TMS1100 embedded) ROM and RAM chips.

use arbitrary_int::{u1, u3, u4, u6};

/// The on-cartridge 2kb (2048 x 8-bit) Read Only Memory (ROM) chip.
#[derive(Debug, Clone)]
pub struct Rom {
    /// The inner (unguarded) memory data of this chip.
    pub data: [u8; 0x800],
}

impl Rom {
    /// Create a new (uninitialized) 2kb ROM chip.
    #[must_use]
    pub fn new() -> Self {
        Self { data: [0; 0x800] }
    }

    /// Read from this ROM chip.
    ///
    /// This takes a segmented memory address as input instead of
    /// a full 11-bit address. The full address is assembled like
    /// so: `0b[c][pppp][aaaaaa]`.
    #[must_use]
    pub fn read(&self, chapter: u1, page: u4, addr: u6) -> u8 {
        let full = (chapter.value() as usize & 0x1) << 10
            | (page.value() as usize & 0xf) << 6
            | (addr.value() as usize & 0x3f);

        self.data[full]
    }

    /// Return the checksum of the data contained on this ROM chip.
    ///
    /// This is an expensive operation to perform, so it is best to
    /// call this function once, then cache the value.
    #[must_use]
    pub fn checksum(&self) -> u16 {
        self.data
            .iter()
            .copied()
            .map(u16::from)
            .fold(0, u16::wrapping_add)
    }
}

impl Default for Rom {
    fn default() -> Self {
        Self::new()
    }
}

/// The on-cartridge 64b (128 x 4-bit) Random Access Memory (RAM) chip.
#[derive(Debug, Clone)]
pub struct Ram {
    /// The inner (unguarded) memory data of this chip.
    pub data: [u4; 0x80],
}

impl Ram {
    /// Create a new (uninitialized) 64b RAM chip.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: [u4::new(0); 0x80],
        }
    }

    /// Read from this RAM chip.
    ///
    /// This takes a segmented memory address as input instead of
    /// a full 7-bit address. The full address is assembled like
    /// so: `0b[xxx][yyyy]`.
    #[must_use]
    pub fn read(&self, x: u3, y: u4) -> u4 {
        let full = (x.value() as usize & 0x7) << 4 | (y.value() as usize & 0xf);

        self.data[full]
    }

    /// Write to this RAM chip.
    ///
    /// This takes a segmented memory address as input instead of
    /// a full 7-bit address. The full address is assembled like
    /// so: `0b[xxx][yyyy]`.
    pub fn write(&mut self, x: u3, y: u4, value: u4) {
        let full = (x.value() as usize & 0x7) << 4 | (y.value() as usize & 0xf);

        self.data[full] = value;
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}
