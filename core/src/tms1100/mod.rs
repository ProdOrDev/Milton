//! A (mostly) cycle-accurate implementation of the TMS1100 micro-processor.
//!
//! The usage of the term cycle-accuracy does not fully apply to internal pipelines
//! or processes, it mostly applies to the external output/input signal(s).
//!
//! # Links
//!
//! - MAME: <https://github.com/mamedev/mame/blob/master/src/devices/cpu/tms1000/tms1k_base.cpp>
//! - Data Manual: <http://www.bitsavers.org/components/ti/TMS1000/TMS_1000_Series_Data_Manual_Dec76.pdf>
//! - Programmers Reference: <https://en.wikichip.org/w/images/f/ff/TMS1000_Series_Programmer%27s_reference_manual.pdf>

pub mod mem;
pub mod pinio;
pub mod pla;

use mem::{Ram, RamAddr, Rom, RomAddr};
use pla::{
    instructions::{
        ATN, AUTA, AUTY, C8, CIN, CKM, CKN, CKP, FTN, MTN, MTP, NATN, NE, STO, STSL, YTP,
    },
    Entry, Fixed,
};

use arbitrary_int::{u1, u11, u3, u4, u5, u6, Number};

/// The internal adder circuit of the TMS1100.
///
/// Technically speaking, this can also be referred to as the Arithmetic Logic Unit
/// (ALU), however the documents provided about the TMS1100 refer to it as the adder.
#[derive(Debug, Clone)]
pub struct Adder {
    /// The 4-bit `P` input of the adder.
    ///
    /// This functions as the left hand side input of the adder.
    pub p: u4,
    /// The 4-bit `N` input of the adder.
    ///
    /// This functions as the right hand side input of the adder.
    pub n: u4,
    /// The 4-bit output of the adder.
    pub output: u4,
    /// The carry input of the adder.
    ///
    /// This enables the adder to perform ADC (Add With Carry) operations, or simply
    /// increment a number, unconditionally, in a compact manner.
    pub carry_in: bool,
    /// The status output of the adder.
    ///
    /// This acts as a flag output for the adder which can be modified through
    /// micro-instructions to represent the carry flag or a flag indicating
    /// the inequality of the adder inputs P and N.
    pub status_out: bool,
}

impl Adder {
    /// Create a new adder circuit.
    #[must_use]
    fn new() -> Self {
        Self {
            p: u4::new(0),
            n: u4::new(0),
            output: u4::new(0),
            carry_in: false,
            status_out: false,
        }
    }

    /// Reset the state of this adder circuit.
    fn reset(&mut self) {
        *self = Self::new();
        self.status_out = true;
    }

    /// Clock (update) this adder circuit.
    ///
    /// # Logic
    ///
    /// This performs an add (with optional carry) operation and copies the appropriate
    /// value to the status output of the adder.
    fn clock(&mut self, carry_to_status: bool, compare_to_status: bool) {
        let carry_in = u4::new(self.carry_in.into());

        let (res, c1) = self.p.overflowing_add(self.n);
        let (res, c2) = res.overflowing_add(carry_in);
        self.output = res;

        if carry_to_status {
            self.status_out &= c1 || c2;
        }
        if compare_to_status {
            self.status_out &= self.p != self.n;
        }
    }
}

/// The current sub-instruction cycle.
///
/// The TMS1100 operates on 6 oscillator cycles within a larger machine cycle,
/// with the general process going: fetch data from memory, then execute an
/// operation. Therefore, we need to represent each of 6 cycles as separate units.
#[derive(Debug, Clone, Copy)]
pub enum Cycle {
    /// The first sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the second part of the fixed instructions
    /// branch, call and return are executed; The K input of the processor is
    /// checked; The required data is read from RAM and the adder circuit is
    /// reset.
    On0,
    /// The second sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the `P` and `N` inputs of the adder circuit
    /// are filled with their corresponding data.
    On1,
    /// The third sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the adder circuit performs its operation;
    /// The reset of the fixed instructions finish executing and the required data
    /// is written back to RAM.
    On2,
    /// The fourth sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the first part of the register latch operations
    /// are executed. These operations can include both reading from a register into
    /// memory and storing the result of the adder circuit into a register.
    On3,
    /// The fifth sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the second part of the register latch operations
    /// are executed. These operations can include both reading from a register into
    /// memory and storing the result of the adder circuit into a register.
    On4,
    /// The sixth (and last) sub-instruction cycle.
    ///
    /// # Logic
    ///
    /// On this sub-instruction cycle, the next instruction is read from ROM and the
    /// first part of the fixed instructions branch, call and return are executed.
    On5,
}

impl Cycle {
    /// Go to the next sub-instruction cycle.
    ///
    /// If the current cycle is the sixth sub-instruction cycle, this will
    /// overflow back to the first cycle.
    fn next(&mut self) {
        *self = match self {
            Self::On0 => Self::On1,
            Self::On1 => Self::On2,
            Self::On2 => Self::On3,
            Self::On3 => Self::On4,
            Self::On4 => Self::On5,
            Self::On5 => Self::On0,
        }
    }
}

/// The branch/status flags of the TMS1100.
#[derive(Debug, Clone, Copy)]
pub struct Flags {
    /// The `C` call latch/flag.
    ///
    /// # Logic
    ///
    /// This is enabled when a call instruction is executed and is disabled
    /// when a return instruction is executed. This is needed to prevent
    /// branching to other memory pages because then the call's return
    /// address would be invalid.
    pub call: bool,
    /// The `SL` status latch/flag.
    ///
    /// # Logic
    ///
    /// This stores the results from adder operations and is used by instructions
    /// to conditionally perform branches or calls.
    pub status: bool,
}

/// A collection of data registers/latches on the TMS1100.
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    /// The 4-bit `A` accumulator.
    pub a: u4,
    /// The 3-bit `X` memory address register.
    pub x: u3,
    /// The 4-bit `Y` memory address register.
    pub y: u4,
    /// The 6-bit `PC` program counter.
    pub pc: u6,
    /// The 6-bit `SR` subroutine return register.
    ///
    /// This stores the previous value of the program counter when a call
    /// instruction is executed.
    pub sr: u6,
    /// The 4-bit `PA` page address register.
    pub pa: u4,
    /// The 4-bit `PB` page buffer register.
    pub pb: u4,
    /// The 1-bit `CA` chapter address latch.
    ///
    /// This stores the current chapter data.
    pub ca: u1,
    /// The 1-bit `CB` chapter buffer latch.
    ///
    /// This stores the succeeding chapter data and transfers to the CA pending
    /// the successful execution of a subsequent branch or call instruction.
    pub cb: u1,
    /// The 1-bit `CS` chapter subroutine latch.
    ///
    /// This stores the return address after successfully executing a call instruction
    /// (CA -> CS). CS transfers data back to CA when the return from subroutine (RETN)
    /// instruction occurs.
    pub cs: u1,
}

/// An emulated TMS1100 micro-processor.
#[derive(Debug, Clone)]
pub struct Tms1100 {
    /// The 11-bit pin output R\[0-10\].
    pub r: pinio::R,
    /// The 5-bit pin output O\[0-4\].
    pub o: pinio::O,
    /// The 4-bit pin input K\[1,2,4,8\].
    pub k: pinio::K,
    /// The internal adder circuit.
    pub adder: Adder,
    /// The branch/status flags.
    pub flags: Flags,
    /// The data registers/latches.
    pub regs: Registers,
    /// The current sub-instruction cycle.
    pub cycle: Cycle,
    /// The currently decoded (and executing) opcode.
    pub opcode: u8,
    /// The fixed instruction of the current opcode.
    pub fixed: Option<Fixed>,
    /// The micro-instruction PLA entry of the current opcode.
    pub micro: Entry,
    /// The lower 4-bit constant of the current opcode.
    constant: u4,
    /// A 4-bit workable value.
    ///
    /// This is filled with data from RAM on the first sub-instruction cycle
    /// and written back to RAM on the third sub-instruction cycle.
    ram_data: u4,
    /// The internal `CKI` data bus.
    ///
    /// The contents of this bus vary depending on the current instruction.
    cki_data: u4,
}

impl Tms1100 {
    /// Create a new TMS1100 micro-processor.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            r: pinio::R::new(),
            o: pinio::O::new(),
            k: pinio::K::new(),
            adder: Adder::new(),
            flags: Flags {
                call: false,
                status: false,
            },
            regs: Registers {
                a: u4::new(0),
                x: u3::new(0),
                y: u4::new(0),
                pc: u6::new(0),
                sr: u6::new(0),
                pa: u4::new(0),
                pb: u4::new(0),
                ca: u1::new(0),
                cb: u1::new(0),
                cs: u1::new(0),
            },
            cycle: Cycle::On0,
            opcode: 0x00,
            fixed: None,
            micro: Entry::EMPTY,
            constant: u4::new(0),
            ram_data: u4::new(0),
            cki_data: u4::new(0),
        }
    }

    /// Reset this micro-processor.
    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    /// Increment the `PC` program counter.
    fn next_pc(&mut self) {
        // The program counter is Linear Feedback Shift Register (LFSR).
        //
        // This means that a feedback bit exists which is a XOR of the
        // highest two bits. However, this bit does make an exception
        // when all the low bits of the program counter are set.

        let mut feedback = (self.regs.pc << 1) >> 5 & self.regs.pc >> 5;

        if self.regs.pc == u6::MAX >> 1 {
            feedback = u6::new(1);
        } else if self.regs.pc == u6::MAX {
            feedback = u6::new(0);
        }

        self.regs.pc = self.regs.pc << 1 | feedback;
    }

    /// Read the next opcode from ROM.
    fn next_opcode(&mut self, rom: &Rom) {
        self.opcode = rom.read(RomAddr::new(self.regs.cs, self.regs.pa, self.regs.pc));

        // The lower 4-bits of the opcode is a constant value,
        // however most instructions expect this to be bit-swapped.
        self.constant = u4::new(self.opcode & 0xf).reverse_bits();

        self.fixed = Fixed::decode(self.opcode);
        self.micro = Entry::decode(self.opcode);

        self.next_pc();
    }

    /// Read a value onto the `CKI` data bus.
    fn read_cki(&mut self) {
        self.cki_data = match self.opcode & 0xf8 {
            // Opcode: 00001XXX, reads the K inputs.
            0x08 => self.k.0,
            // Opcode: 0011XXXX, select the bit to modify.
            0x30 | 0x38 => u4::new(1) << ((self.constant.value() >> 2) ^ 0xf),
            // Opcode: 01XXXXXX, a constant value.
            0x00 | 0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => self.constant,
            _ => u4::new(0),
        }
    }

    /// Execute the first sub-instruction cycle.
    fn exec_0(&mut self, ram: &Ram) {
        match self.fixed {
            Some(Fixed::Br) if self.flags.status => {
                if !self.flags.call {
                    self.regs.pa = self.regs.pb;
                }

                self.regs.ca = self.regs.cb;
                self.regs.pc = u6::new(self.opcode & 0x3f);
            }
            Some(Fixed::Call) if self.flags.status => {
                let prev_pa = self.regs.pa;

                if !self.flags.call {
                    self.flags.call = true;
                    self.regs.sr = self.regs.pc;
                    self.regs.pa = self.regs.pb;
                    self.regs.cs = self.regs.ca;
                }

                self.regs.ca = self.regs.cb;
                self.regs.pb = prev_pa;
                self.regs.pc = u6::new(self.opcode & 0x3f);
            }
            Some(Fixed::Retn) => {
                if self.flags.call {
                    self.flags.call = false;
                    self.regs.pc = self.regs.sr;
                    self.regs.ca = self.regs.cs;
                }

                self.regs.pa = self.regs.pb;
            }
            _ => {}
        }

        self.read_cki();
        self.ram_data = ram.read(RamAddr::new(self.regs.x, self.regs.y));

        self.adder.reset();
    }

    /// Execute the second sub-instruction cycle.
    fn exec_1(&mut self) {
        if self.micro.enables::<FTN>() {
            self.adder.n |= u4::MAX;
        }
        if self.micro.enables::<ATN>() {
            self.adder.n |= self.regs.a;
        }
        if self.micro.enables::<NATN>() {
            self.adder.n |= !self.regs.a;
        }
        if self.micro.enables::<CKN>() {
            self.adder.n |= self.cki_data;
        }
        if self.micro.enables::<MTN>() {
            self.adder.n |= self.ram_data;
        }
        if self.micro.enables::<CKP>() {
            self.adder.p |= self.cki_data;
        }
        if self.micro.enables::<MTP>() {
            self.adder.p |= self.ram_data;
        }
        if self.micro.enables::<YTP>() {
            self.adder.p |= self.regs.y;
        }
        if self.micro.enables::<CIN>() {
            self.adder.carry_in = true;
        }
    }

    /// Execute the third sub-instruction cycle.
    fn exec_2(&mut self, ram: &mut Ram) {
        self.adder
            .clock(self.micro.enables::<C8>(), self.micro.enables::<NE>());

        if self.micro.enables::<CKM>() {
            self.ram_data = self.cki_data;
        }
        if self.micro.enables::<STO>() {
            self.ram_data = self.regs.a;
        }

        match self.fixed {
            Some(Fixed::Comc) => {
                self.regs.cb ^= u1::MAX;
            }
            Some(Fixed::Comx) => {
                self.regs.x ^= u3::MAX;
            }
            Some(Fixed::Ldp) => {
                self.regs.pb = self.constant;
            }
            Some(Fixed::Ldx) => {
                self.regs.x = u3::new(self.constant.value() >> 1);
            }
            Some(Fixed::Rbit) => {
                self.ram_data &= self.cki_data;
            }
            Some(Fixed::Rstr) => {
                let idx = (self.regs.x.value() >> 2) << 4 | self.regs.y.value();
                self.r.0 &= !(u11::new(1) << idx);
            }
            Some(Fixed::Sbit) => {
                self.ram_data |= self.cki_data ^ u4::new(0xf);
            }
            Some(Fixed::Setr) => {
                let idx = (self.regs.x.value() >> 2) << 4 | self.regs.y.value();
                self.r.0 |= u11::new(1) << idx;
            }
            Some(Fixed::Tdo) => {
                self.o.0 = u5::new(u8::from(self.flags.status)) | u5::new(self.regs.a.value());
            }
            _ => {}
        }

        ram.write(RamAddr::new(self.regs.x, self.regs.y), self.ram_data);
    }

    /// Execute the fifth sub-instruction cycle.
    fn exec_4(&mut self, rom: &Rom) {
        if self.micro.enables::<AUTA>() {
            self.regs.a = self.adder.output;
        }
        if self.micro.enables::<AUTY>() {
            self.regs.y = self.adder.output;
        }
        if self.micro.enables::<STSL>() {
            self.flags.status = self.adder.status_out;
        }

        self.next_opcode(rom);
    }

    /// Clock (update) this micro-processor.
    ///
    /// # Logic
    ///
    /// This executes a single sub-instruction cycle, 1/6 of a whole instruction.
    #[allow(clippy::similar_names)]
    pub(crate) fn clock(&mut self, rom: &Rom, ram: &mut Ram) {
        match self.cycle {
            Cycle::On0 => self.exec_0(ram),
            Cycle::On1 => self.exec_1(),
            Cycle::On2 => self.exec_2(ram),
            Cycle::On4 => self.exec_4(rom),
            Cycle::On3 | Cycle::On5 => {
                // These sub-instruction cycles are idle in this emulation.
            }
        }

        self.cycle.next();
    }
}
