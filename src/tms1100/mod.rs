//! A (mostly) cycle-accurate implementation of the TMS1100 micro-processor.
//!
//! ## Links
//!
//! - MAME: <https://github.com/mamedev/mame/blob/master/src/devices/cpu/tms1000/tms1k_base.cpp>
//! - Data Manual: <http://www.bitsavers.org/components/ti/TMS1000/TMS_1000_Series_Data_Manual_Dec76.pdf>
//! - Programmers Reference: <https://en.wikichip.org/w/images/f/ff/TMS1000_Series_Programmer%27s_reference_manual.pdf>

pub mod mem;
pub mod pla;

use arbitrary_int::{u1, u11, u3, u4, u5, u6, u7, Number};

use mem::{Ram, Rom};
use pla::{Entry, Fixed};

/// An emulated TMS1100 micro-processor.
#[derive(Default, Debug, Clone)]
pub struct Tms1100 {
    /// The K\[1,2,4,8\] pin inputs.
    pub k: u4,
    /// The R\[0-10\] pin outputs.
    pub r: u11,
    /// The O\[0-7\] pin outputs.
    ///
    /// ## Note
    ///
    /// An important thing to note here is that this is not actually
    /// the 8-bit O value, instead this the un-PLAed value of the O
    /// pins. To make use of this value it must be put through your
    /// custom PLA first.
    pub o: u5,

    /// The accumulator A.
    pub a: u4,
    /// The address register X.
    pub x: u3,
    /// The address register Y.
    pub y: u4,
    /// The program counter PC.
    pub pc: u6,
    /// The subroutine return register SR.
    ///
    /// This stores the previous program counter.
    pub sr: u6,
    /// The page address register PA.
    pub pa: u4,
    /// The page buffer register PB.
    pub pb: u4,
    /// The chapter address latch CA.
    ///
    /// This stores the current chapter data.
    pub ca: u1,
    /// The chapter buffer latch CB.
    ///
    /// This stores the succeeding chapter data and transfers to the CA pending
    /// the successful execution of a subsequent branch or call instruction.
    pub cb: u1,
    /// The chapter subroutine latch CS.
    ///
    /// This stores the return address after successfully executing a call instruction
    /// (CA -> CS). CS transfers data back to CA when the return from subroutine (RETN)
    /// instruction occurs.
    pub cs: u1,
    /// The call latch C.
    pub call_latch: bool,
    /// The status latch S.
    ///
    /// This stores the result from adder operations/comparisons and can be used to
    /// conditionally branch.
    pub status_latch: bool,

    /// A sub-instruction cycle counter.
    pub cycle: Counter,
    /// The current opcode.
    pub opcode: u8,
    /// The fixed instruction of the current opcode.
    pub fixed: Fixed,
    /// The micro-instructions of the current opcode.
    pub micro: Entry,
    /// The CKI data bus.
    ///
    /// The contents of this bus vary depending on the currently executing opcode.
    pub cki_bus: u4,
    /// The lower 4-bit constant of the current opcode.
    c: u4,
    /// A value read from, or to be written to, RAM.
    ram_data: u4,
    /// The internal ALU (or adder).
    pub alu: Alu,

    /// The onboard (2048 x 8bit) Read Only Memory (ROM) chip.
    pub rom: Rom,
    /// The onboard (128 x 4bit) Random Access Memory (RAM) chip.
    pub ram: Ram,
}

impl Tms1100 {
    /// Create a new TMS1100 micro-processor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a single sub-instruction cycle.
    pub fn clock(&mut self) {
        #[allow(clippy::match_same_arms)]
        match self.cycle {
            Counter::Cycle0 => {
                // Execute the second part of the branch/call/return fixed instructions.
                self.fixed_br();
                self.fixed_call();
                self.fixed_retn();

                // Read the constant or K input for the current opcode.
                self.read_cki_bus();

                // Read data from RAM.
                self.ram_data = self.ram.read();

                // Clear the inputs and outputs of the ALU.
                self.alu.reset();
            }
            Counter::Cycle1 => {
                // Update the ROM address.
                self.rom.addr =
                    u11::from(self.ca) << 10 | u11::from(self.pa) << 6 | u11::from(self.pc);

                // Update the N input of the ALU.
                self.op_ftn();
                self.op_atn();
                self.op_natn();
                self.op_ckn();
                self.op_mtn();

                // Update the P inout of the ALU.
                self.op_ckp();
                self.op_mtp();
                self.op_ytp();

                // Update the carry input of the ALU.
                self.op_cin();
            }
            Counter::Cycle2 => {
                // Perform the ALU logic.
                self.alu.clock();

                // Update the ALU status.
                self.op_c8();
                self.op_ne();
                self.op_ckm();

                // Perform the rest of the fixed instructions and decide
                // which value to write back to RAM.
                self.op_sto();

                self.fixed_sbit();
                self.fixed_rbit();
                self.fixed_setr();
                self.fixed_rstr();
                self.fixed_tdo();
                self.fixed_ldx();
                self.fixed_comx();
                self.fixed_ldp();
                self.fixed_comc();

                // Write the (potentially modified) RAM data back to RAM.
                self.ram.write(self.ram_data);
            }
            Counter::Cycle3 => {}
            Counter::Cycle4 => {
                // Store any potential ALU outputs into registers.
                self.op_auta();
                self.op_auty();
                self.op_stsl();

                // Read the next opcode.
                self.read_opcode();

                // Update the RAM address.
                self.ram.addr = u7::new(self.x.value() << 4 | self.y.value());
            }
            Counter::Cycle5 => {}
        }

        self.cycle.next();
    }

    /// Read the correct value for the current opcode onto the CKI data bus.
    fn read_cki_bus(&mut self) {
        self.cki_bus = match self.opcode & 0xf8 {
            // Opcode: 00001XXX, reads the K inputs.
            0x08 => self.k,
            // Opcode: 0011XXXX, select the bit to modify.
            0x30 | 0x38 => u4::new(1) << ((self.c.value() >> 2) ^ 0xf),
            // Opcode: 01XXXXXX, a constant value.
            0x00 | 0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => self.c,
            _ => u4::new(0),
        }
    }

    /// Increment the program counter.
    fn next_pc(&mut self) {
        // The program counter is Linear Feedback Shift Register (LFSR).
        //
        // This means that a feedback bit exists which is a XOR of the
        // highest two bits. However, this bit does make an exception
        // when all the low bits of the program counter are set.

        let mut feedback = (self.pc << 1) >> 5 & self.pc >> 5;

        if self.pc == u6::MAX >> 1 {
            feedback = u6::new(1);
        } else if self.pc == u6::MAX {
            feedback = u6::new(0);
        }

        self.pc = self.pc << 1 | feedback;
    }

    /// Read the opcode at the current program counter, then increment
    /// the program counter.
    fn read_opcode(&mut self) {
        let op = self.rom.read();
        self.opcode = op;

        // The lower 4-bits of the opcode is a constant value,
        // however most instructions expect this to be bitswapped.
        self.c = u4::new(op & 0xf).reverse_bits();

        self.fixed = Fixed::decode(op);
        self.micro = Entry::decode(op);

        self.next_pc();
    }
}

/// A sub-instruction cycle counter.
///
/// The TMS1100 operates on 6 oscillator cycles within a larger machine cycle,
/// with the general process going: fetch data from memory, then execute an
/// operation.
#[derive(Default, Debug, Clone, Copy)]
pub enum Counter {
    /// The first sub-instruction cycle.
    ///
    /// The second part of the branch/call/return instructions are executed,
    /// additionally the K inputs are checked, data is read from RAM, and
    /// the inputs to the ALU are cleared.
    #[default]
    Cycle0,
    /// The second sub-instruction cycle.
    ///
    /// The inputs to the ALU are supplied with their corresponding data.
    Cycle1,
    /// The third sub-instruction cycle.
    ///
    /// The ALU performs its specified logic operation, the remaining fixed
    /// instructions are executed, and data is written back to RAM.
    Cycle2,
    /// The fourth sub-instruction cycle.
    ///
    /// The first part of the register store operations are executed.
    Cycle3,
    /// The fifth sub-instruction cycle.
    ///
    /// The second part of the register store operations are executed.
    Cycle4,
    /// The sixth sub-instruction cycle.
    ///
    /// The next instruction is decoded and the first part of the branch/call/return
    /// instruction are executed.
    Cycle5,
}

impl Counter {
    /// Go to the next sub-instruction cycle.
    pub fn next(&mut self) {
        *self = match self {
            Self::Cycle0 => Self::Cycle1,
            Self::Cycle1 => Self::Cycle2,
            Self::Cycle2 => Self::Cycle3,
            Self::Cycle3 => Self::Cycle4,
            Self::Cycle4 => Self::Cycle5,
            Self::Cycle5 => Self::Cycle0,
        }
    }
}

/// The internal ALU (or adder) of the TMS1100.
#[derive(Default, Debug, Clone)]
pub struct Alu {
    /// The P (lhs) input of the ALU.
    pub p: u4,
    /// The N (rhs) input of the ALU.
    pub n: u4,
    /// The result output of the ALU.
    pub res: u4,

    /// The CIN (carry) input of the ALU.
    pub carry_in: bool,
    /// The COUT (carry) output of the ALU.
    pub carry_out: bool,

    /// The status output of the ALU.
    pub status: bool,
}

impl Alu {
    /// Create a new ALU for the TMS1100.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the current state of the ALU.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Clock the ALU.
    ///
    /// This performs an add (with optional carry) operation.
    pub fn clock(&mut self) {
        let carry_in = u4::from(u8::from(self.carry_in));

        let (res, c1) = self.p.overflowing_add(self.n);
        let (res, c2) = res.overflowing_add(carry_in);

        self.carry_out = c1 || c2;
        self.res = res;

        // This will be modified by micro-instructions.
        self.status = true;
    }
}
