//! Emulation of the Programmable Logic Array (PLA) used for instruction decoding.

bitfield::bitfield! {
    /// An entry in the instruction decode PLA.
    ///
    /// This controls which micro-instructions are enabled for the given
    /// opcode/instruction. However, not every instruction uses the PLA
    /// for execution, some opcodes are decoded using a fixed (non-programmable)
    /// logic scheme ([Fixed]).
    #[derive(Clone, Copy)]
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
    /// however none of the PLA combinations actually make use of it.
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

/// See the corresponding [Entry] docs for more information.
const CKP: u16 = 1 << 0;
/// See the corresponding [Entry] docs for more information.
const YTP: u16 = 1 << 1;
/// See the corresponding [Entry] docs for more information.
const MTP: u16 = 1 << 2;
/// See the corresponding [Entry] docs for more information.
const ATN: u16 = 1 << 3;
/// See the corresponding [Entry] docs for more information.
const NATN: u16 = 1 << 4;
/// See the corresponding [Entry] docs for more information.
#[allow(unused)]
const MTN: u16 = 1 << 5;
/// See the corresponding [Entry] docs for more information.
const FTN: u16 = 1 << 6;
/// See the corresponding [Entry] docs for more information.
const CKN: u16 = 1 << 7;
/// See the corresponding [Entry] docs for more information.
const CIN: u16 = 1 << 8;
/// See the corresponding [Entry] docs for more information.
const NE: u16 = 1 << 9;
/// See the corresponding [Entry] docs for more information.
const C8: u16 = 1 << 10;
/// See the corresponding [Entry] docs for more information.
const STO: u16 = 1 << 11;
/// See the corresponding [Entry] docs for more information.
const CKM: u16 = 1 << 12;
/// See the corresponding [Entry] docs for more information.
const AUTA: u16 = 1 << 13;
/// See the corresponding [Entry] docs for more information.
const AUTY: u16 = 1 << 14;
/// See the corresponding [Entry] docs for more information.
const STSL: u16 = 1 << 15;
