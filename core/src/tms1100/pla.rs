//! Emulation of the Programmable Logic Array (PLA) used for instruction decoding.

use arbitrary_int::{u1, u11, u3, u4, u5, u6, Number};

bitfield::bitfield! {
    /// An entry in the instruction decode PLA.
    ///
    /// This controls which micro-instructions are enabled for the given
    /// opcode/instruction. However, not every instruction uses the PLA
    /// for execution, some opcodes are decoded using a fixed (non-programmable)
    /// logic scheme ([Fixed]).
    #[derive(Default, Clone, Copy)]
    pub struct Entry(u16);
    impl Debug;
    /// The micro-instruction CTP.
    ///
    /// When enabled this transfers the data on the CKI bus to the P input of the
    /// adder.
    pub ckp,  _:  0;
    /// The micro-instruction YTP.
    ///
    /// When enabled this transfers the Y register to the P input of the adder.
    pub ytp,  _:  1;
    /// The micro-instruction MTP.
    ///
    /// When enabled this transfers the value in memory at (X, Y) to the P input
    /// of the adder.
    pub mtp,  _:  2;
    /// The micro-instruction ATN.
    ///
    /// When enabled this transfers the accumulator to the N input of the adder.
    pub atn,  _:  3;
    /// The micro-instruction NATN.
    ///
    /// When enabled this transfers the negated value of the accumulator to the
    /// N input of the adder.
    pub natn, _:  4;
    /// The micro-instruction MTN.
    ///
    /// When enabled this transfers the value in memory at (X, Y) to the N input
    /// of the adder.
    ///
    /// This micro-instruction is slightly odd because it exists on the chip,
    /// however none of the PLA combinations actually make use of it. My best
    /// thought is that is was reserved for future revisions of the processor.
    pub mtn,  _:  5;
    /// The micro-instruction 15TN.
    ///
    /// When enabled this transfers the value $f to the N input of the adder.
    pub ftn,  _:  6;
    /// The micro-instruction CKN.
    ///
    /// When enabled this transfers the data on the CKI bus to the N input of
    /// the adder.
    pub ckn,  _:  7;
    /// The micro-instruction CIN.
    ///
    /// When enabled this instructs the adder to add an additional one to the P
    /// and N inputs.
    pub cin,  _:  8;
    /// The micro-instruction NE.
    ///
    /// When enabled this instructs the adder to compare the P and N inputs,
    /// setting the output status of the adder to zero if they are equal.
    pub ne,   _:  9;
    /// The micro-instruction C8.
    ///
    /// When enabled this stores a potential adder carry to the internal status
    /// latch.
    pub c8,   _: 10;
    /// The micro-instruction STO.
    ///
    /// When enabled this writes the accumulator to memory.
    pub sto,  _: 11;
    /// The micro-instruction CKM.
    ///
    /// When enabled this writes the data on the CKI bus to memory.
    pub ckm,  _: 12;
    /// The micro-instruction AUTA.
    ///
    /// When enabled this stores the result of the adder into the accumulator.
    pub auta, _: 13;
    /// The micro-instruction AUTY.
    ///
    /// When enabled this stores the result of the adder into the Y register.
    pub auty, _: 14;
    /// The micro-instruction STSL.
    ///
    /// When enabled this stores the output status of the adder into the
    /// internal status latch.
    pub stsl, _: 15;
}

impl Entry {
    /// Decode the TMS1100 PLA entry of the given opcode.
    #[must_use]
    pub fn decode(op: u8) -> Self {
        const CKP: u16 = 1 << 0;
        const YTP: u16 = 1 << 1;
        const MTP: u16 = 1 << 2;
        const ATN: u16 = 1 << 3;
        const NATN: u16 = 1 << 4;
        #[allow(unused)]
        const MTN: u16 = 1 << 5;
        const FTN: u16 = 1 << 6;
        const CKN: u16 = 1 << 7;
        const CIN: u16 = 1 << 8;
        const NE: u16 = 1 << 9;
        const C8: u16 = 1 << 10;
        const STO: u16 = 1 << 11;
        const CKM: u16 = 1 << 12;
        const AUTA: u16 = 1 << 13;
        const AUTY: u16 = 1 << 14;
        const STSL: u16 = 1 << 15;

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
    /// Decode a TMS1100 fixed instruction from the given opcode.
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
            0x80..=0xaf => Self::Br,
            0xc0..=0xff => Self::Call,
            _ => Self::None,
        }
    }
}

impl super::Tms1100 {
    /// Execute the fixed instruction BR.
    pub(super) fn fixed_br(&mut self) {
        if matches!(self.fixed, Fixed::Br) && self.status_latch {
            if !self.call_latch {
                self.pa = self.pb;
            }

            self.ca = self.cb;
            self.pc = u6::new(self.opcode & 0x3f);
        }
    }

    /// Execute the fixed instruction CALL.
    pub(super) fn fixed_call(&mut self) {
        if matches!(self.fixed, Fixed::Call) && self.status_latch {
            let prev_pa = self.pa;

            if !self.call_latch {
                self.call_latch = true;
                self.sr = self.pc;
                self.pa = self.pb;
                self.cs = self.ca;
            }

            self.ca = self.cb;
            self.pb = prev_pa;
            self.pc = u6::new(self.opcode & 0x3f);
        }
    }

    /// Execute the fixed instruction RETN.
    pub(super) fn fixed_retn(&mut self) {
        if matches!(self.fixed, Fixed::Retn) {
            if self.call_latch {
                self.call_latch = false;
                self.pc = self.sr;
                self.ca = self.cs;
            }

            self.pa = self.pb;
        }
    }

    /// Execute the fixed instruction SBIT.
    pub(super) fn fixed_sbit(&mut self) {
        if matches!(self.fixed, Fixed::Sbit) {
            self.ram_data |= self.cki_bus ^ u4::new(0xf);
        }
    }

    /// Execute the fixed instruction RBIT.
    pub(super) fn fixed_rbit(&mut self) {
        if matches!(self.fixed, Fixed::Rbit) {
            self.ram_data &= self.cki_bus;
        }
    }

    /// Execute the fixed instruction SETR.
    pub(super) fn fixed_setr(&mut self) {
        if matches!(self.fixed, Fixed::Setr) {
            let idx = (self.x.value() >> 2) << 4 | self.y.value();
            self.r |= u11::new(1) << idx;
        }
    }

    /// Execute the fixed instruction RSTR.
    pub(super) fn fixed_rstr(&mut self) {
        if matches!(self.fixed, Fixed::Rstr) {
            let idx = (self.x.value() >> 2) << 4 | self.y.value();
            self.r &= !(u11::new(1) << idx);
        }
    }

    /// Execute the fixed instruction TDO.
    pub(super) fn fixed_tdo(&mut self) {
        if matches!(self.fixed, Fixed::Tdo) {
            self.o = u5::new(u8::from(self.status_latch)) | u5::new(self.a.value());
        }
    }

    /// Execute the fixed instruction LDX.
    pub(super) fn fixed_ldx(&mut self) {
        if matches!(self.fixed, Fixed::Ldx) {
            self.x = u3::new(self.c.value() >> 1);
        }
    }

    /// Execute the fixed instruction COMX.
    pub(super) fn fixed_comx(&mut self) {
        if matches!(self.fixed, Fixed::Comx) {
            self.x ^= u3::MAX;
        }
    }

    /// Execute the fixed instruction LDP.
    pub(super) fn fixed_ldp(&mut self) {
        if matches!(self.fixed, Fixed::Ldp) {
            self.pb = self.c;
        }
    }

    /// Execute the fixed instruction COMC.
    pub(super) fn fixed_comc(&mut self) {
        if matches!(self.fixed, Fixed::Comc) {
            self.cb ^= u1::MAX;
        }
    }
}

impl super::Tms1100 {
    /// Execute the micro-instruction 15TN.
    pub(super) fn op_ftn(&mut self) {
        if self.micro.ftn() {
            self.alu.n |= u4::MAX;
        }
    }

    /// Execute the micro-instruction ATN.
    pub(super) fn op_atn(&mut self) {
        if self.micro.atn() {
            self.alu.n |= self.a;
        }
    }
    /// Execute the micro-instruction NATN.
    pub(super) fn op_natn(&mut self) {
        if self.micro.natn() {
            self.alu.n |= !self.a;
        }
    }
    /// Execute the micro-instruction CKN.
    pub(super) fn op_ckn(&mut self) {
        if self.micro.ckn() {
            self.alu.n |= self.cki_bus;
        }
    }
    /// Execute the micro-instruction MTN.
    pub(super) fn op_mtn(&mut self) {
        if self.micro.mtn() {
            self.alu.n |= self.ram_data;
        }
    }

    /// Execute the micro-instruction CKP.
    pub(super) fn op_ckp(&mut self) {
        if self.micro.ckp() {
            self.alu.p |= self.cki_bus;
        }
    }
    /// Execute the micro-instruction MTP.
    pub(super) fn op_mtp(&mut self) {
        if self.micro.mtp() {
            self.alu.p |= self.ram_data;
        }
    }
    /// Execute the micro-instruction YTP.
    pub(super) fn op_ytp(&mut self) {
        if self.micro.ytp() {
            self.alu.p |= self.y;
        }
    }

    /// Execute the micro-instruction CIN.
    pub(super) fn op_cin(&mut self) {
        if self.micro.cin() {
            self.alu.carry_in = true;
        }
    }

    /// Execute the micro-instruction C8.
    pub(super) fn op_c8(&mut self) {
        if self.micro.c8() {
            self.alu.status &= self.alu.carry_out;
        }
    }

    /// Execute the micro-instruction NE.
    pub(super) fn op_ne(&mut self) {
        if self.micro.ne() {
            self.alu.status &= self.alu.p != self.alu.n;
        }
    }

    /// Execute the micro-instruction CKM.
    pub(super) fn op_ckm(&mut self) {
        if self.micro.ckm() {
            self.ram_data = self.cki_bus;
        }
    }

    /// Execute the micro-instruction STO.
    pub(super) fn op_sto(&mut self) {
        if self.micro.sto() {
            self.ram_data = self.a;
        }
    }

    /// Execute the micro-instruction AUTA.
    pub(super) fn op_auta(&mut self) {
        if self.micro.auta() {
            self.a = self.alu.res;
        }
    }

    /// Execute the micro-instruction AUTY.
    pub(super) fn op_auty(&mut self) {
        if self.micro.auty() {
            self.y = self.alu.res;
        }
    }

    /// Execute the micro-instruction STSL.
    pub(super) fn op_stsl(&mut self) {
        if self.micro.stsl() {
            self.status_latch = self.alu.status;
        }
    }
}
