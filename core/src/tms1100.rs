//! A (mostly) cycle-accurate implementation of the TMS1100 micro-processor.
//!
//! The usage of the term cycle-accuracy does not fully apply to internal pipelines
//! or processes, it mostly applies to the external output/input signal(s).
//!
//! ## Links
//!
//! - MAME: <https://github.com/mamedev/mame/blob/master/src/devices/cpu/tms1000/tms1k_base.cpp>
//! - Data Manual: <http://www.bitsavers.org/components/ti/TMS1000/TMS_1000_Series_Data_Manual_Dec76.pdf>
//! - Programmers Reference: <https://en.wikichip.org/w/images/f/ff/TMS1000_Series_Programmer%27s_reference_manual.pdf>

/// The Programmable Logic Array (PLA) used by the TMS1100 for instruction decoding.
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

    /*
        Compile time verification ¯\_(ツ)_/¯

        Currently, Rust only allows primitive types for const-generics,
        therefore there needs to be some compile-time for enums represented
        as integers.
    */

    trait IsValid {}
    struct MicroInstruction<const M: u16>;

    impl IsValid for MicroInstruction<CKP> {}
    impl IsValid for MicroInstruction<YTP> {}
    impl IsValid for MicroInstruction<MTP> {}
    impl IsValid for MicroInstruction<ATN> {}
    impl IsValid for MicroInstruction<NATN> {}
    impl IsValid for MicroInstruction<MTN> {}
    impl IsValid for MicroInstruction<FTN> {}
    impl IsValid for MicroInstruction<CKN> {}
    impl IsValid for MicroInstruction<CIN> {}
    impl IsValid for MicroInstruction<NE> {}
    impl IsValid for MicroInstruction<C8> {}
    impl IsValid for MicroInstruction<STO> {}
    impl IsValid for MicroInstruction<CKM> {}
    impl IsValid for MicroInstruction<AUTA> {}
    impl IsValid for MicroInstruction<AUTY> {}
    impl IsValid for MicroInstruction<STSL> {}

    /// An entry in the instruction decode PLA.
    ///
    /// This controls which micro-instructions are enabled for the given opcode
    /// or instruction. However, not every instruction uses the PLA for execution,
    /// some opcodes are decoded using a fixed (non-programmable) logic scheme ([Fixed]).
    #[derive(Debug, Clone, Copy)]
    pub struct Entry(u16);

    impl Entry {
        /// The micro-instruction PLA entry used to initialize the TMS1100.
        pub(super) const INIT: Self = Self(0);

        /// Determine if this PLA entry enables a specific micro-instruction.
        #[must_use]
        #[allow(private_bounds)]
        pub fn enables<const MICRO: u16>(&self) -> bool
        where
            MicroInstruction<MICRO>: IsValid,
        {
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
    #[derive(Debug, Clone, Copy)]
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
    buzzer::BuzzerPulse,
    common::Ms,
    keyboard::{self, Key},
    lcd::{LatchPulse, NotDataClock},
    memory::{Ram, RamAddr, Rom, RomAddr},
    rotary::{ChargePulse, Rotary},
    settings::Settings,
};
use arbitrary_int::{u1, u11, u3, u4, u5, u6, Number};
use pla::{Entry, Fixed, *};

/// The 11-bit pin outputs R\[0-10\].
///
/// These are mapped, by the cartridge, to various components of the Microvision,
/// such as the rotary controller, Piezo buzzer, LCD driver, etc. etc.
#[derive(Debug, Clone, Copy)]
pub struct OutputR(pub u11);

impl OutputR {
    /// Return the latch pulse line of the LCD driver.
    #[must_use]
    pub fn latch_pulse(&self) -> LatchPulse {
        LatchPulse::new(self.0.value() >> 6 & 1 != 0)
    }

    /// Return the not data clock line of the LCD driver.
    #[must_use]
    pub fn not_clock(&self) -> NotDataClock {
        NotDataClock::new(self.0.value() >> 7 & 1 != 0)
    }

    /// Return the buzzer pulse line.
    #[must_use]
    pub fn buzzer_pulse(&self) -> BuzzerPulse {
        BuzzerPulse::new(self.0.value() & 1 != 0)
    }

    /// Return the rotary charge line.
    #[must_use]
    pub fn rotary_charge(&self) -> ChargePulse {
        ChargePulse::new(self.0.value() >> 2 & 1 != 0)
    }

    /// Check if the nth keyboard column is selected to be scanned.
    ///
    /// ## Panics
    ///
    /// This will panic if the associated constant `N` is not within the
    /// range of `0..=2`.
    #[must_use]
    pub fn nth_keyboard<const N: usize>(&self) -> bool {
        assert!(N < 3);
        self.0.value() >> (10 - N) & 1 != 0
    }
}

/// The 5-bit pin outputs O\[0-4\].
///
/// This is mapped to the data input of the LCD driver. Each cartridge wires the
/// output PLA of these pins differently so, this value may be reversed on some
/// cartridges and normal (un-reversed) on others.
#[derive(Debug, Clone, Copy)]
pub struct OutputO(pub u5);

/// The 4-bit pin inputs K\[1,2,4,8\].
///
/// These are mapped to the currently selected keyboard column and the rotary
/// controller, if it still has charge enabled.
#[derive(Debug, Clone, Copy)]
pub struct InputK(pub u4);

impl InputK {
    /// Update this input with the correct data.
    pub fn update<K>(
        &mut self,
        r: OutputR,
        elapsed: Ms,
        settings: Settings,
        rotary: &Rotary,
        keyboard: &K,
    ) where
        K: keyboard::Agnostic,
    {
        /// Read a column of keys as a 4-bit number from the given keyboard.
        fn read_column<K>(kb: &K, keys: [Key; 4]) -> u4
        where
            K: keyboard::Agnostic,
        {
            let k1 = if kb.get(keys[0]) { 8 } else { 0 };
            let k2 = if kb.get(keys[1]) { 4 } else { 0 };
            let k3 = if kb.get(keys[2]) { 2 } else { 0 };
            let k4 = if kb.get(keys[3]) { 1 } else { 0 };
            u4::new(k1 | k2 | k3 | k4)
        }

        self.0 = u4::MIN;

        // The 10th pin of the R output connects to the left column of the keyboard.
        if r.nth_keyboard::<0>() {
            self.0 |= read_column(keyboard, [Key::At0x0, Key::At0x1, Key::At0x2, Key::At0x3]);
        }
        // The 9th pin of the R output connects to the middle column of the keyboard.
        if r.nth_keyboard::<1>() {
            self.0 |= read_column(keyboard, [Key::At1x0, Key::At1x1, Key::At1x2, Key::At1x3]);
        }
        // The 8th pin of the R output connects to the right column of the keyboard.
        if r.nth_keyboard::<2>() {
            self.0 |= read_column(keyboard, [Key::At2x0, Key::At2x1, Key::At2x2, Key::At2x3]);
        }
        if settings.rotary_enabled {
            self.0 &= u4::new(7);
            // If the charging circuit of the rotary controller has ended (timed out)
            // set the K8 line.
            if rotary.charge.value() && rotary.charge_end < elapsed {
                self.0 |= u4::new(8);
            }
        }
    }
}

/// The Arithmetic Logic Unit (ALU) of the TMS1100.
#[derive(Debug, Clone)]
pub struct Alu {
    /// The 4-bit P input of the ALU.
    ///
    /// This represents the left hand side of the ALU.
    pub p: u4,
    /// The 4-bit N input of the ALU.
    ///
    /// This represents the right hand side of the ALU.
    pub n: u4,
    /// The 4-bit output of the ALU.
    pub res: u4,
    /// The carry input of the ALU.
    ///
    /// By setting this, the ALU can performs add with carry operations
    /// or simply increment a number by one (unconditionally).
    pub carry_in: bool,
    /// The status output of the ALU.
    ///
    /// This acts as a flag output for the ALU which can be modified through
    /// micro-instructions to represent the carry flag or a flag indicating
    /// the inequality of the ALU inputs P and N.
    pub status: bool,
}

impl Alu {
    /// Create a new ALU.
    #[must_use]
    pub fn new() -> Self {
        Self {
            p: u4::MIN,
            n: u4::MIN,
            res: u4::MIN,
            carry_in: false,
            status: false,
        }
    }

    /// Reset this ALU.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clock (update) this ALU.
    ///
    /// This returns a boolean value indicating if a carry has occurred.
    pub fn clock(&mut self) -> bool {
        let carry_in = u4::new(u8::from(self.carry_in));

        let (res, c1) = self.p.overflowing_add(self.n);
        let (res, c2) = res.overflowing_add(carry_in);

        self.res = res;
        self.status = true;

        c1 || c2
    }
}

/// The branch/status flags of the TMS1100.
#[derive(Debug, Clone, Copy)]
pub struct Flags {
    /// The call latch/flag C.
    ///
    /// This is enabled when a call instruction is executed and is disabled
    /// when a return instruction is executed. This is needed to prevent
    /// branching to other memory pages because then the call's return
    /// address would be invalid.
    pub call: bool,
    /// The status latch/flag S or SL.
    ///
    /// This stores the results from ALU operations and is used by instructions
    /// to conditionally perform branches or calls.
    pub status: bool,
}

/// A sub-instruction cycle counter.
///
/// ## Timing
///
/// The TMS1100 operates on 6 oscillator cycles within a larger machine cycle,
/// with the general process going: fetch data from memory, then execute an
/// operation.
#[derive(Debug, Clone, Copy)]
pub enum Counter {
    /// The first sub-instruction cycle.
    ///
    /// The second part of the branch/call/return instructions are executed,
    /// additionally the K inputs are checked, data is read from RAM, and
    /// the inputs to the ALU are cleared.
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
    /// Increment this counter to go to the next sub-instruction cycle.
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

/// An emulated TMS1100 micro-processor.
#[derive(Debug, Clone)]
pub struct Tms1100 {
    /// The 11-bit pin outputs R\[0-10\].
    pub r: OutputR,
    /// The 5-bit pin outputs O\[0-4\].
    pub o: OutputO,
    /// The 4-bit pin inputs K\[1,2,4,8\].
    pub k: InputK,
    /// The ALU.
    pub alu: Alu,
    /// The branch/status flags.
    pub flags: Flags,
    /// The 4-bit accumulator A.
    pub a: u4,
    /// The 3-bit memory address register X.
    pub x: u3,
    /// The 4-bit memory address register Y.
    pub y: u4,
    /// The 6-bit program counter PC.
    pub pc: u6,
    /// The 6-bit subroutine return register SR.
    ///
    /// This stores the previous value of the program counter when a call
    /// instruction is executed.
    pub sr: u6,
    /// The 4-bit page address register PA.
    pub pa: u4,
    /// The 4-bit page buffer register PB.
    pub pb: u4,
    /// The 1-bit chapter address latch CA.
    ///
    /// This stores the current chapter data.
    pub ca: u1,
    /// The 1-bit chapter buffer latch CB.
    ///
    /// This stores the succeeding chapter data and transfers to the CA pending
    /// the successful execution of a subsequent branch or call instruction.
    pub cb: u1,
    /// The 1-bit chapter subroutine latch CS.
    ///
    /// This stores the return address after successfully executing a call instruction
    /// (CA -> CS). CS transfers data back to CA when the return from subroutine (RETN)
    /// instruction occurs.
    pub cs: u1,
    /// A sub-instruction cycle counter.
    pub cycle: Counter,
    /// The currently executing opcode.
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
    /// Create a new TMS1100 micro-processor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            r: OutputR(u11::MIN),
            o: OutputO(u5::MIN),
            k: InputK(u4::MIN),
            alu: Alu::new(),
            flags: Flags {
                call: false,
                status: false,
            },
            a: u4::MIN,
            x: u3::MIN,
            y: u4::MIN,
            pc: u6::MIN,
            sr: u6::MIN,
            pa: u4::MIN,
            pb: u4::MIN,
            ca: u1::MIN,
            cb: u1::MIN,
            cs: u1::MIN,
            cycle: Counter::Cycle0,
            opcode: 0x00,
            fixed: Fixed::None,
            micro: Entry::INIT,
            constant: u4::MIN,
            ram_data: u4::MIN,
            cki_latch: u4::MIN,
        }
    }

    /// Reset this micro-processor.
    pub fn reset(&mut self) {
        *self = Self::new();
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

    /// Read the next opcode from ROM at the current program counter, then increment
    /// the program counter.
    fn next_opcode(&mut self, rom: &Rom) {
        self.opcode = rom.read(RomAddr::new(self.cs, self.pa, self.pc));

        // The lower 4-bits of the opcode is a constant value,
        // however most instructions expect this to be bit-swapped.
        self.constant = u4::new(self.opcode & 0xf).reverse_bits();

        self.fixed = Fixed::decode(self.opcode);
        self.micro = Entry::decode(self.opcode);

        self.next_pc();
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

    /// Clock (update) this micro-processor.
    ///
    /// This executes a single sub-instruction cycle, 1/6 of an instruction.
    pub fn clock(&mut self, rom: &Rom, ram: &mut Ram) {
        match self.cycle {
            Counter::Cycle0 => {
                match self.fixed {
                    Fixed::Br if self.flags.status => {
                        if !self.flags.call {
                            self.pa = self.pb;
                        }

                        self.ca = self.cb;
                        self.pc = u6::new(self.opcode & 0x3f);
                    }
                    Fixed::Call if self.flags.status => {
                        let prev_pa = self.pa;

                        if !self.flags.call {
                            self.flags.call = true;
                            self.sr = self.pc;
                            self.pa = self.pb;
                            self.cs = self.ca;
                        }

                        self.ca = self.cb;
                        self.pb = prev_pa;
                        self.pc = u6::new(self.opcode & 0x3f);
                    }
                    Fixed::Retn => {
                        if self.flags.call {
                            self.flags.call = false;
                            self.pc = self.sr;
                            self.ca = self.cs;
                        }

                        self.pa = self.pb;
                    }
                    _ => {}
                }

                self.read_cki();
                self.ram_data = ram.read(RamAddr::new(self.x, self.y));

                self.alu.reset();
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
                    self.alu.p |= self.y;
                }

                if self.micro.enables::<CIN>() {
                    self.alu.carry_in = true;
                }
            }
            Counter::Cycle2 => {
                let carry = self.alu.clock();

                if self.micro.enables::<C8>() {
                    self.alu.status &= carry;
                }
                if self.micro.enables::<NE>() {
                    self.alu.status &= self.alu.p != self.alu.n;
                }
                if self.micro.enables::<CKM>() {
                    self.ram_data = self.cki_latch;
                }

                if self.micro.enables::<STO>() {
                    self.ram_data = self.a;
                }

                match self.fixed {
                    Fixed::Comc => {
                        self.cb ^= u1::MAX;
                    }
                    Fixed::Comx => {
                        self.x ^= u3::MAX;
                    }
                    Fixed::Ldp => {
                        self.pb = self.constant;
                    }
                    Fixed::Ldx => {
                        self.x = u3::new(self.constant.value() >> 1);
                    }
                    Fixed::Rbit => {
                        self.ram_data &= self.cki_latch;
                    }
                    Fixed::Rstr => {
                        let idx = (self.x.value() >> 2) << 4 | self.y.value();
                        self.r.0 &= !(u11::new(1) << idx);
                    }
                    Fixed::Sbit => {
                        self.ram_data |= self.cki_latch ^ u4::new(0xf);
                    }
                    Fixed::Setr => {
                        let idx = (self.x.value() >> 2) << 4 | self.y.value();
                        self.r.0 |= u11::new(1) << idx;
                    }
                    Fixed::Tdo => {
                        self.o.0 = u5::new(u8::from(self.flags.status)) | u5::new(self.a.value());
                    }
                    _ => {}
                }

                ram.write(RamAddr::new(self.x, self.y), self.ram_data);
            }
            Counter::Cycle3 => { /* Idle */ }
            Counter::Cycle4 => {
                if self.micro.enables::<AUTA>() {
                    self.a = self.alu.res;
                }
                if self.micro.enables::<AUTY>() {
                    self.y = self.alu.res;
                }
                if self.micro.enables::<STSL>() {
                    self.flags.status = self.alu.status;
                }

                self.next_opcode(rom);
            }
            Counter::Cycle5 => { /* Idle */ }
        }

        self.cycle.next();
    }
}
