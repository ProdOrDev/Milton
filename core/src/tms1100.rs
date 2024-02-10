//! A (mostly) cycle-accurate implementation of the TMS1100 micro-processor.
//!
//! The cycle-accurate part only really applies to external signal output/input(s).
//!
//! ## Links
//!
//! - MAME: <https://github.com/mamedev/mame/blob/master/src/devices/cpu/tms1000/tms1k_base.cpp>
//! - Data Manual: <http://www.bitsavers.org/components/ti/TMS1000/TMS_1000_Series_Data_Manual_Dec76.pdf>
//! - Programmers Reference: <https://en.wikichip.org/w/images/f/ff/TMS1000_Series_Programmer%27s_reference_manual.pdf>

/// Emulation of the Programmable Logic Array (PLA) used by the TMS1100 for instruction decoding.
pub mod pla {
    /// The micro-instruction CKP.
    ///
    /// When enabled this transfers the data on the CKI bus to the P input of the
    /// adder.
    pub const CKP: u16 = 1 << 0;
    /// The micro-instruction YTP.
    ///
    /// When enabled this transfers the Y register to the P input of the adder.
    pub const YTP: u16 = 1 << 1;
    /// The micro-instruction MTP.
    ///
    /// When enabled this transfers the value in memory at (X, Y) to the P input
    /// of the adder.
    pub const MTP: u16 = 1 << 2;
    /// The micro-instruction ATN.
    ///
    /// When enabled this transfers the accumulator to the N input of the adder.
    pub const ATN: u16 = 1 << 3;
    /// The micro-instruction NATN.
    ///
    /// When enabled this transfers the negated value of the accumulator to the
    /// N input of the adder.
    pub const NATN: u16 = 1 << 4;
    /// The micro-instruction MTN.
    ///
    /// When enabled this transfers the value in memory at (X, Y) to the N input
    /// of the adder.
    ///
    /// This micro-instruction is slightly odd because it exists on the chip,
    /// however none of the PLA combinations actually make use of it. My best
    /// thought is that is was reserved for future revisions of the processor.
    pub const MTN: u16 = 1 << 5;
    /// The micro-instruction 15TN.
    ///
    /// When enabled this transfers the value $f to the N input of the adder.
    pub const FTN: u16 = 1 << 6;
    /// The micro-instruction CKN.
    ///
    /// When enabled this transfers the data on the CKI bus to the N input of
    /// the adder.
    pub const CKN: u16 = 1 << 7;
    /// The micro-instruction CIN.
    ///
    /// When enabled this instructs the adder to add an additional one to the P
    /// and N inputs.
    pub const CIN: u16 = 1 << 8;
    /// The micro-instruction NE.
    ///
    /// When enabled this instructs the adder to compare the P and N inputs,
    /// setting the output status of the adder to zero if they are equal.
    pub const NE: u16 = 1 << 9;
    /// The micro-instruction C8.
    ///
    /// When enabled this stores a potential adder carry to the internal status
    /// latch.
    pub const C8: u16 = 1 << 10;
    /// The micro-instruction STO.
    ///
    /// When enabled this writes the accumulator to memory.
    pub const STO: u16 = 1 << 11;
    /// The micro-instruction CKM.
    ///
    /// When enabled this writes the data on the CKI bus to memory.
    pub const CKM: u16 = 1 << 12;
    /// The micro-instruction AUTA.
    ///
    /// When enabled this stores the result of the adder into the accumulator.
    pub const AUTA: u16 = 1 << 13;
    /// The micro-instruction AUTY.
    ///
    /// When enabled this stores the result of the adder into the Y register.
    pub const AUTY: u16 = 1 << 14;
    /// The micro-instruction STSL.
    ///
    /// When enabled this stores the output status of the adder into the
    /// internal status latch.
    pub const STSL: u16 = 1 << 15;

    /// An entry in the instruction decode PLA.
    ///
    /// This controls which micro-instructions are enabled for the given opcode
    /// or instruction. However, not every instruction uses the PLA for execution,
    /// some opcodes are decoded using a fixed (non-programmable) logic scheme ([Fixed]).
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Entry(u16);

    impl Entry {
        /// Determine if this PLA entry enables a specific micro-instruction.
        #[must_use]
        pub fn enables<const MICRO: u16>(&self) -> bool {
            self.0 & MICRO != 0
        }

        /// Return the PLA entry of the given opcode.
        #[must_use]
        pub fn decode(op: u8) -> Self {
            match op {
                0x00 => Self(MTP | ATN | NE),
                0x01 => Self(MTP | NATN | CIN | C8),
                0x02 => Self(YTP | ATN | NE | STSL),
                0x03 => Self(MTP | STO | AUTA),
                0x04 => Self(YTP | FTN | C8 | AUTY),
                0x05 => Self(YTP | CIN | C8 | AUTY),
                0x06 => Self(ATN | MTP | C8 | AUTA),
                0x07 => Self(MTP | FTN | C8 | AUTA),
                0x08 => Self(CKP | AUTA),
                0x0e => Self(CKP | NE),
                0x20 => Self(ATN | AUTY),
                0x21 => Self(MTP | AUTA),
                0x22 => Self(MTP | AUTY),
                0x23 => Self(YTP | AUTA),
                0x24 => Self(STO | YTP | FTN | C8 | AUTY),
                0x25 => Self(STO | YTP | CIN | C8 | AUTY),
                0x26 => Self(STO | AUTA),
                0x27 => Self(STO),
                0x38..=0x3b => Self(CKP | CKN | MTP | NE),
                0x3c => Self(MTP | NATN | CIN | C8 | AUTA),
                0x3d => Self(NATN | CIN | C8 | AUTA),
                0x3e => Self(MTP | CIN | C8 | AUTA),
                0x3f => Self(MTP | NE),
                0x40..=0x4f => Self(CKP | AUTY),
                0x50..=0x5f => Self(YTP | CKN | NE),
                0x60..=0x6f => Self(CKM | YTP | CIN | AUTY),
                0x70..=0x7e => Self(CKP | ATN | CIN | C8 | AUTA),
                0x7f => Self(CKP | CIN | C8 | AUTA),
                _ => Self(0),
            }
        }
    }

    /// A set of 12 fixed instructions.
    ///
    /// Unlike most instructions with can be modified via the PLA, certain
    /// instructions like branch/call have fixed (non-programmable) logic
    /// decoders. However, this does not mean the PLA slots for these opcodes
    /// are useless, in fact activating certain micro-instructions in the
    /// PLA for these fixed instructions can enable completely new instructions
    /// to be formed.
    #[derive(Default, Debug, Clone, Copy)]
    pub enum Fixed {
        /// A branch instruction.
        Br,
        /// A subroutine call instruction.
        Call,
        /// A complement instruction.
        ///
        /// This complements (inverts) the chapter buffer.
        Comc,
        /// A complement instruction.
        ///
        /// This complements (inverts) the X register.
        Comx,
        /// A load instruction.
        ///
        /// This loads the page buffer with a constant value.
        Ldp,
        /// A load instruction.
        ///
        /// This loads the X register with a file address.
        Ldx,
        /// A bit instruction.
        ///
        /// This resets the specified bit in RAM.
        Rbit,
        /// A subroutine return instruction.
        Retn,
        /// An output instruction.
        ///
        /// This resets the 11-bit output R addressed by the Y register.
        Rstr,
        /// A bit instruction.
        ///
        /// This sets the specified bit in RAM.
        Sbit,
        /// An output instruction.
        ///
        /// This sets the 11-bit output R addressed by the Y register.
        Setr,
        /// A transfer instruction.
        ///
        /// This transfers data from the accumulator and status latch into the O output.
        Tdo,
        /// The current opcode is not a fixed instruction.
        ///
        /// This is returned from [Fixed::decode] if the given opcode is not
        /// one of the 12 fixed instructions.
        #[default]
        None,
    }

    impl Fixed {
        /// Return the fixed instruction (if any) corresponding to the given opcode.
        #[must_use]
        pub fn decode(op: u8) -> Self {
            match op {
                0x09 => Self::Comx,
                0x0a => Self::Tdo,
                0x0b => Self::Comc,
                0x0c => Self::Rstr,
                0x0d => Self::Setr,
                0x0f => Self::Retn,
                0x30..=0x33 => Self::Sbit,
                0x34..=0x37 => Self::Rbit,
                0x01..=0x1f => Self::Ldp,
                0x28..=0x2f => Self::Ldx,
                0x80..=0xbf => Self::Br,
                0xc0..=0xff => Self::Call,
                _ => Self::None,
            }
        }
    }
}

use crate::{
    hughes0488::{LatchPulse, NotDataClock},
    memory::{Ram, Rom},
};
use arbitrary_int::{u1, u11, u3, u4, u5, u6, u7, Number};
use pla::{Entry, Fixed, *};

/// The internal Arithmetic Logic Unit (ALU) of the TMS1100.
#[derive(Default, Debug, Clone)]
pub struct Alu {
    /// The P (left hand side) input of the ALU.
    pub p: u4,
    /// The N (right hand side) input of the ALU.
    pub n: u4,
    /// The output result of the ALU.
    pub result: u4,

    /// The carry input of the ALU.
    ///
    /// This allows the ALU to perform add with carry operations,
    /// or to simply increment a number.
    pub carry_in: bool,
    /// The carry output of the ALU.
    ///
    /// This can be copied to the status output of the ALU by
    /// executing certain micro-instructions.
    pub carry_out: bool,

    /// The status output of the ALU.
    ///
    /// This can be modified by various operations in the ALU to
    /// output the in-equality of the ALU inputs or a flag indicating
    /// if a carry occurred while perfoming addition on the ALU inputs.
    pub status: bool,
}

impl Alu {
    /// Instruct this ALU to perform an add (with optional carry) operation.
    fn add(&mut self) {
        let carry_in = u4::new(u8::from(self.carry_in));

        let (res, c1) = self.p.overflowing_add(self.n);
        let (res, c2) = res.overflowing_add(carry_in);

        self.carry_out = c1 || c2;
        self.result = res;

        self.status = true;
    }

    /// Instruct this ALU to copy the carry output to the status output.
    fn carry_status(&mut self) {
        self.status &= self.carry_out;
    }

    /// Instruct this ALU to compare the P and N inputs for inequality.
    fn compare(&mut self) {
        self.status &= self.p != self.n;
    }
}

/// The internal branch and/or status flags of the TMS1100.
#[derive(Default, Debug, Clone)]
pub struct Flags {
    /// The call latch/flag C.
    ///
    /// This is enabled when a call instruction (CALL) is executed and is
    /// disabled when a return instruction (RETN) is executed. This is needed
    /// to prevent branching to other memory pages because then the call's
    /// return address would be invalid.
    pub call: bool,
    /// The status latch/flag S or SL.
    ///
    /// This stores the results from ALU operations and is used by instructions
    /// to conditionally perform branches or calls.
    pub status: bool,
}

/// The internal RAM address latches of the TMS1100.
#[derive(Default, Debug, Clone)]
pub struct RamAddr {
    /// The memory address register X.
    pub x: u3,
    /// The memory address register Y.
    pub y: u4,
}

impl RamAddr {
    /// Return the full 7bit RAM address.
    #[must_use]
    pub fn full(&self) -> u7 {
        u7::new(self.x.value() << 4 | self.y.value())
    }
}

/// The internal ROM address latches of the TMS1100.
#[derive(Default, Debug, Clone)]
pub struct RomAddr {
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
}

impl RomAddr {
    /// Return the full 11bit ROM address.
    #[must_use]
    pub fn full(&self) -> u11 {
        u11::from(self.ca) << 10 | u11::from(self.pa) << 6 | u11::from(self.pc)
    }

    /// Increment the program counter and go to the next instruction.
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
}

/// An (emulator-fictional) sub-instruction cycle counter.
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
    /// Increment this counter to the next sub-instruction cycle.
    fn next(&mut self) {
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

/// The R\[0-10\] pin outputs.
#[derive(Default, Debug, Clone, Copy)]
pub struct OutputR(pub u11);

impl OutputR {
    /// Return the value of the LCD drivers latch pulse line.
    #[must_use]
    pub fn latch_pulse(&self) -> LatchPulse {
        LatchPulse(self.0.value() >> 6 & 1 != 0)
    }

    /// Return the value of the LCD drivers not data clock line.
    #[must_use]
    pub fn not_clock(&self) -> NotDataClock {
        NotDataClock(self.0.value() >> 7 & 1 != 0)
    }

    /// Return the value of the speaker line.
    #[must_use]
    pub fn speaker(&self) -> bool {
        self.0.value() & 1 != 0
    }

    /// Return the value of the rotary control line.
    #[must_use]
    pub fn rotary_control(&self) -> bool {
        self.0.value() >> 2 & 1 != 0
    }

    /// Check if the nth keyboard column is selected.
    #[must_use]
    pub fn keyboard<const N: usize>(&self) -> bool {
        assert!(N < 3);
        self.0.value() >> (10 - N) & 1 != 0
    }
}

/// The O\[0-7\] pin outputs.
///
/// An important thing to note here is that this is not actually
/// the 8-bit O value, instead this the un-PLAed value of the O
/// pins. To make use of this value it must be put through your
/// custom PLA first.
#[derive(Default, Debug, Clone, Copy)]
pub struct OutputO(pub u5);

/// The K\[1,2,4,8\] pin inputs.
#[derive(Default, Debug, Clone, Copy)]
pub struct InputK(pub u4);

/// An emulated TMS1100 micro-processor.
#[derive(Default, Debug, Clone)]
pub struct Tms1100 {
    /// The R\[0-10\] pin outputs.
    pub r: OutputR,
    /// The O\[0-7\] pin outputs.
    pub o: OutputO,
    /// The K\[1,2,4,8\] pin inputs.
    pub k: InputK,

    /// The RAM address latches.
    pub ram_addr: RamAddr,
    /// The ROM address latches.
    pub rom_addr: RomAddr,
    /// The branch and/or status flags.
    pub flags: Flags,
    /// The Arithmetic Logic Unit (ALU).
    pub alu: Alu,
    /// The accumulator A.
    pub a: u4,

    /// A sub-instruction cycle counter.
    pub counter: Counter,
    /// The current opcode.
    pub opcode: u8,
    /// The fixed instruction of the current opcode.
    pub fixed: Fixed,
    /// The micro-instructions of the current opcode.
    pub micro: Entry,

    /// The lower 4-bit constant of the current opcode.
    constant: u4,
    /// A value read from, or to be written to, RAM.
    ram_data: u4,
    /// The CKI data latch.
    ///
    /// The contents of this latch vary depending on the currently executing opcode.
    cki_latch: u4,
}

impl Tms1100 {
    /// Execute a single sub-instruction cycle.
    pub fn clock(&mut self, rom: &Rom, ram: &mut Ram) {
        match self.counter {
            Counter::Cycle0 => {
                match self.fixed {
                    Fixed::Br if self.flags.status => {
                        if !self.flags.call {
                            self.rom_addr.pa = self.rom_addr.pb;
                        }

                        self.rom_addr.ca = self.rom_addr.cb;
                        self.rom_addr.pc = u6::new(self.opcode & 0x3f);
                    }
                    Fixed::Call if self.flags.status => {
                        let prev_pa = self.rom_addr.pa;

                        if !self.flags.call {
                            self.flags.call = true;
                            self.rom_addr.sr = self.rom_addr.pc;
                            self.rom_addr.pa = self.rom_addr.pb;
                            self.rom_addr.cs = self.rom_addr.ca;
                        }

                        self.rom_addr.ca = self.rom_addr.cb;
                        self.rom_addr.pb = prev_pa;
                        self.rom_addr.pc = u6::new(self.opcode & 0x3f);
                    }
                    Fixed::Retn => {
                        if self.flags.call {
                            self.flags.call = false;
                            self.rom_addr.pc = self.rom_addr.sr;
                            self.rom_addr.ca = self.rom_addr.cs;
                        }

                        self.rom_addr.pa = self.rom_addr.pb;
                    }
                    _ => {}
                }

                self.read_cki();
                self.ram_data = ram.read(self.ram_addr.full());

                self.alu = Alu::default();
            }
            Counter::Cycle1 => {
                if self.micro.enables::<FTN>() {
                    self.alu.n |= u4::MAX;
                }
                if self.micro.enables::<ATN>() {
                    self.alu.n |= self.a;
                }
                if self.micro.enables::<NATN>() {
                    self.alu.n |= !self.a;
                }
                if self.micro.enables::<CKN>() {
                    self.alu.n |= self.cki_latch;
                }
                if self.micro.enables::<MTN>() {
                    self.alu.n |= self.ram_data;
                }

                if self.micro.enables::<CKP>() {
                    self.alu.p |= self.cki_latch;
                }
                if self.micro.enables::<MTP>() {
                    self.alu.p |= self.ram_data;
                }
                if self.micro.enables::<YTP>() {
                    self.alu.p |= self.ram_addr.y;
                }

                if self.micro.enables::<CIN>() {
                    self.alu.carry_in = true;
                }
            }
            Counter::Cycle2 => {
                self.alu.add();

                if self.micro.enables::<C8>() {
                    self.alu.carry_status();
                }
                if self.micro.enables::<NE>() {
                    self.alu.compare()
                }
                if self.micro.enables::<CKM>() {
                    self.ram_data = self.cki_latch;
                }

                if self.micro.enables::<STO>() {
                    self.ram_data = self.a;
                }

                match self.fixed {
                    Fixed::Comc => {
                        self.rom_addr.cb ^= u1::MAX;
                    }
                    Fixed::Comx => {
                        self.ram_addr.x ^= u3::MAX;
                    }
                    Fixed::Ldp => {
                        self.rom_addr.pb = self.constant;
                    }
                    Fixed::Ldx => {
                        self.ram_addr.x = u3::new(self.constant.value() >> 1);
                    }
                    Fixed::Rbit => {
                        self.ram_data &= self.cki_latch;
                    }
                    Fixed::Rstr => {
                        let idx = (self.ram_addr.x.value() >> 2) << 4 | self.ram_addr.y.value();
                        self.r.0 &= !(u11::new(1) << idx);
                    }
                    Fixed::Sbit => {
                        self.ram_data |= self.cki_latch ^ u4::new(0xf);
                    }
                    Fixed::Setr => {
                        let idx = (self.ram_addr.x.value() >> 2) << 4 | self.ram_addr.y.value();
                        self.r.0 |= u11::new(1) << idx;
                    }
                    Fixed::Tdo => {
                        self.o.0 = u5::new(u8::from(self.flags.status)) | u5::new(self.a.value());
                    }
                    _ => {}
                }

                ram.write(self.ram_addr.full(), self.ram_data);
            }
            Counter::Cycle3 => { /* Idle */ }
            Counter::Cycle4 => {
                if self.micro.enables::<AUTA>() {
                    self.a = self.alu.result;
                }
                if self.micro.enables::<AUTY>() {
                    self.ram_addr.y = self.alu.result;
                }
                if self.micro.enables::<STSL>() {
                    self.flags.status = self.alu.status;
                }

                self.next_opcode(rom);
            }
            Counter::Cycle5 => { /* Idle */ }
        }

        self.counter.next();
    }

    /// Read the next opcode from ROM at the current program counter, then increment
    /// the program counter.
    fn next_opcode(&mut self, rom: &Rom) {
        self.opcode = rom.read(self.rom_addr.full());

        // The lower 4-bits of the opcode is a constant value,
        // however most instructions expect this to be bit-swapped.
        self.constant = u4::new(self.opcode & 0xf).reverse_bits();

        self.fixed = Fixed::decode(self.opcode);
        self.micro = Entry::decode(self.opcode);

        self.rom_addr.next_pc();
    }

    /// Read a value into the CKI data latch.
    ///
    /// The value going into the latch depends on the current opcode.
    fn read_cki(&mut self) {
        self.cki_latch = match self.opcode & 0xf8 {
            // Opcode: 00001XXX, reads the K inputs.
            0x08 => self.k.0,
            // Opcode: 0011XXXX, select the bit to modify.
            0x30 | 0x38 => u4::new(1) << ((self.constant.value() >> 2) ^ 0xf),
            // Opcode: 01XXXXXX, a constant value.
            0x00 | 0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => self.constant,
            _ => u4::new(0),
        }
    }
}
