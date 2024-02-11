//! Emulation of the Microvision's ROM and RAM chips.
//!
//! These chips are embedded within the TMS1100 micro-processor and belong to
//! the specific game cartridges rather than the Microvision handheld itself.

use arbitrary_int::{u1, u11, u3, u4, u6, u7};
use rand::{thread_rng, Rng};

/// A segmented ROM address.
///
/// Rather than simply taking an 11-bit value as a ROM address, the TMS1100's
/// ROM chip takes a chapter (`c`), page (`p`) and address (`a`). All of these
/// inputs combine to form a full 11-bit address like so: `0b[c][pppp][aaaaaa]`.
#[derive(Debug, Clone, Copy)]
pub struct RomAddr {
    /// The chapter (`c`).
    chapter: u1,
    /// The page (`p`)
    page: u4,
    /// The address (`a`).
    addr: u6,
}

impl RomAddr {
    /// Create a new segmented ROM address.
    #[must_use]
    pub fn new(chapter: u1, page: u4, addr: u6) -> Self {
        Self {
            chapter,
            page,
            addr,
        }
    }

    /// Return the full 11-bit ROM address.
    #[must_use]
    pub fn full(&self) -> u11 {
        u11::from(self.chapter) << 10 | u11::from(self.page) << 6 | u11::from(self.addr)
    }
}

/// The TMS1100's 2kb (2048 x 8-bit) Read Only Memory (ROM) chip.
#[derive(Debug, Clone)]
pub struct Rom {
    /// The inner (unguarded) memory data of this chip.
    pub data: [u8; 0x800],
}

impl Rom {
    /// Create a new (data-zeroed) 2kb ROM chip.
    #[must_use]
    pub fn new_zeroed() -> Self {
        Self { data: [0; 0x800] }
    }

    /// Read from this ROM chip at the specified address.
    #[must_use]
    pub fn read(&self, addr: RomAddr) -> u8 {
        self.data[addr.full().value() as usize & 0x7ff]
    }

    /// Return the checksum of the data contained on this ROM chip.
    #[must_use]
    pub fn checksum(&self) -> u16 {
        self.data
            .iter()
            .copied()
            .map(u16::from)
            .fold(0, u16::wrapping_add)
    }
}

/// A segmented RAM address.
///
/// Rather than simply taking an 7-bit value as a RAM address, the TMS1100's
/// RAM chip takes a memory address (`x`) and a memory address (`y`). These
/// inputs combine to form a 7-bit (or more specifically a grid) index into
/// RAM data like so: `0b[xxx][yyyy]`.
#[derive(Debug, Clone, Copy)]
pub struct RamAddr {
    /// The memory address (`x`)
    x: u3,
    /// The memory address (`y`).
    y: u4,
}

impl RamAddr {
    /// Create a new segmented RAM address.
    #[must_use]
    pub fn new(x: u3, y: u4) -> Self {
        Self { x, y }
    }

    /// Return the full 7-bit RAM address.
    #[must_use]
    pub fn full(&self) -> u7 {
        u7::new(self.x.value() << 4 | self.y.value())
    }
}

/// The TMS1100's 64b (128 x 4-bit) Random Access Memory (RAM) chip.
#[derive(Debug, Clone)]
pub struct Ram {
    /// The inner (unguarded) memory data of this chip.
    pub data: [u4; 0x80],
}

impl Ram {
    /// Create a new (data-zeroed) 64b RAM chip.
    #[must_use]
    pub fn new_zeroed() -> Self {
        Self {
            data: [u4::new(0); 0x80],
        }
    }

    /// Create a new (data-randomized) 64b RAM chip.
    #[must_use]
    pub fn new_random() -> Self {
        let mut rng = thread_rng();
        let mut rom = Self::new_zeroed();

        for val in rom.data.iter_mut() {
            *val = u4::new(rng.gen_range(0..16))
        }

        rom
    }

    /// Read from this RAM chip at the specified address.
    #[must_use]
    pub fn read(&self, addr: RamAddr) -> u4 {
        self.data[addr.full().value() as usize & 0x7f]
    }

    /// Write to this RAM chip at the specified address.
    pub fn write(&mut self, addr: RamAddr, val: u4) {
        self.data[addr.full().value() as usize & 0x7f] = val;
    }
}
