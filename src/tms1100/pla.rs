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

/// The instruction decode PLA of the TMS1100.
pub static PLA: [Entry; 0x100] = [
    /* 0x00 */ Entry(0),
    /* 0x01 */ Entry(0),
    /* 0x02 */ Entry(0),
    /* 0x03 */ Entry(0),
    /* 0x04 */ Entry(0),
    /* 0x05 */ Entry(0),
    /* 0x06 */ Entry(0),
    /* 0x07 */ Entry(0),
    /* 0x08 */ Entry(0),
    /* 0x09 */ Entry(0),
    /* 0x0a */ Entry(0),
    /* 0x0b */ Entry(0),
    /* 0x0c */ Entry(0),
    /* 0x0d */ Entry(0),
    /* 0x0e */ Entry(0),
    /* 0x0f */ Entry(0),
    /* 0x10 */ Entry(0),
    /* 0x11 */ Entry(0),
    /* 0x12 */ Entry(0),
    /* 0x13 */ Entry(0),
    /* 0x14 */ Entry(0),
    /* 0x15 */ Entry(0),
    /* 0x16 */ Entry(0),
    /* 0x17 */ Entry(0),
    /* 0x18 */ Entry(0),
    /* 0x19 */ Entry(0),
    /* 0x1a */ Entry(0),
    /* 0x1b */ Entry(0),
    /* 0x1c */ Entry(0),
    /* 0x1d */ Entry(0),
    /* 0x1e */ Entry(0),
    /* 0x1f */ Entry(0),
    /* 0x20 */ Entry(0),
    /* 0x21 */ Entry(0),
    /* 0x22 */ Entry(0),
    /* 0x23 */ Entry(0),
    /* 0x24 */ Entry(0),
    /* 0x25 */ Entry(0),
    /* 0x26 */ Entry(0),
    /* 0x27 */ Entry(0),
    /* 0x28 */ Entry(0),
    /* 0x29 */ Entry(0),
    /* 0x2a */ Entry(0),
    /* 0x2b */ Entry(0),
    /* 0x2c */ Entry(0),
    /* 0x2d */ Entry(0),
    /* 0x2e */ Entry(0),
    /* 0x2f */ Entry(0),
    /* 0x30 */ Entry(0),
    /* 0x31 */ Entry(0),
    /* 0x32 */ Entry(0),
    /* 0x33 */ Entry(0),
    /* 0x34 */ Entry(0),
    /* 0x35 */ Entry(0),
    /* 0x36 */ Entry(0),
    /* 0x37 */ Entry(0),
    /* 0x38 */ Entry(0),
    /* 0x39 */ Entry(0),
    /* 0x3a */ Entry(0),
    /* 0x3b */ Entry(0),
    /* 0x3c */ Entry(0),
    /* 0x3d */ Entry(0),
    /* 0x3e */ Entry(0),
    /* 0x3f */ Entry(0),
    /* 0x40 */ Entry(0),
    /* 0x41 */ Entry(0),
    /* 0x42 */ Entry(0),
    /* 0x43 */ Entry(0),
    /* 0x44 */ Entry(0),
    /* 0x45 */ Entry(0),
    /* 0x46 */ Entry(0),
    /* 0x47 */ Entry(0),
    /* 0x48 */ Entry(0),
    /* 0x49 */ Entry(0),
    /* 0x4a */ Entry(0),
    /* 0x4b */ Entry(0),
    /* 0x4c */ Entry(0),
    /* 0x4d */ Entry(0),
    /* 0x4e */ Entry(0),
    /* 0x4f */ Entry(0),
    /* 0x50 */ Entry(0),
    /* 0x51 */ Entry(0),
    /* 0x52 */ Entry(0),
    /* 0x53 */ Entry(0),
    /* 0x54 */ Entry(0),
    /* 0x55 */ Entry(0),
    /* 0x56 */ Entry(0),
    /* 0x57 */ Entry(0),
    /* 0x58 */ Entry(0),
    /* 0x59 */ Entry(0),
    /* 0x5a */ Entry(0),
    /* 0x5b */ Entry(0),
    /* 0x5c */ Entry(0),
    /* 0x5d */ Entry(0),
    /* 0x5e */ Entry(0),
    /* 0x5f */ Entry(0),
    /* 0x60 */ Entry(0),
    /* 0x61 */ Entry(0),
    /* 0x62 */ Entry(0),
    /* 0x63 */ Entry(0),
    /* 0x64 */ Entry(0),
    /* 0x65 */ Entry(0),
    /* 0x66 */ Entry(0),
    /* 0x67 */ Entry(0),
    /* 0x68 */ Entry(0),
    /* 0x69 */ Entry(0),
    /* 0x6a */ Entry(0),
    /* 0x6b */ Entry(0),
    /* 0x6c */ Entry(0),
    /* 0x6d */ Entry(0),
    /* 0x6e */ Entry(0),
    /* 0x6f */ Entry(0),
    /* 0x70 */ Entry(0),
    /* 0x71 */ Entry(0),
    /* 0x72 */ Entry(0),
    /* 0x73 */ Entry(0),
    /* 0x74 */ Entry(0),
    /* 0x75 */ Entry(0),
    /* 0x76 */ Entry(0),
    /* 0x77 */ Entry(0),
    /* 0x78 */ Entry(0),
    /* 0x79 */ Entry(0),
    /* 0x7a */ Entry(0),
    /* 0x7b */ Entry(0),
    /* 0x7c */ Entry(0),
    /* 0x7d */ Entry(0),
    /* 0x7e */ Entry(0),
    /* 0x7f */ Entry(0),
    /* 0x80 */ Entry(0),
    /* 0x81 */ Entry(0),
    /* 0x82 */ Entry(0),
    /* 0x83 */ Entry(0),
    /* 0x84 */ Entry(0),
    /* 0x85 */ Entry(0),
    /* 0x86 */ Entry(0),
    /* 0x87 */ Entry(0),
    /* 0x88 */ Entry(0),
    /* 0x89 */ Entry(0),
    /* 0x8a */ Entry(0),
    /* 0x8b */ Entry(0),
    /* 0x8c */ Entry(0),
    /* 0x8d */ Entry(0),
    /* 0x8e */ Entry(0),
    /* 0x8f */ Entry(0),
    /* 0x90 */ Entry(0),
    /* 0x91 */ Entry(0),
    /* 0x92 */ Entry(0),
    /* 0x93 */ Entry(0),
    /* 0x94 */ Entry(0),
    /* 0x95 */ Entry(0),
    /* 0x96 */ Entry(0),
    /* 0x97 */ Entry(0),
    /* 0x98 */ Entry(0),
    /* 0x99 */ Entry(0),
    /* 0x9a */ Entry(0),
    /* 0x9b */ Entry(0),
    /* 0x9c */ Entry(0),
    /* 0x9d */ Entry(0),
    /* 0x9e */ Entry(0),
    /* 0x9f */ Entry(0),
    /* 0xa0 */ Entry(0),
    /* 0xa1 */ Entry(0),
    /* 0xa2 */ Entry(0),
    /* 0xa3 */ Entry(0),
    /* 0xa4 */ Entry(0),
    /* 0xa5 */ Entry(0),
    /* 0xa6 */ Entry(0),
    /* 0xa7 */ Entry(0),
    /* 0xa8 */ Entry(0),
    /* 0xa9 */ Entry(0),
    /* 0xaa */ Entry(0),
    /* 0xab */ Entry(0),
    /* 0xac */ Entry(0),
    /* 0xad */ Entry(0),
    /* 0xae */ Entry(0),
    /* 0xaf */ Entry(0),
    /* 0xb0 */ Entry(0),
    /* 0xb1 */ Entry(0),
    /* 0xb2 */ Entry(0),
    /* 0xb3 */ Entry(0),
    /* 0xb4 */ Entry(0),
    /* 0xb5 */ Entry(0),
    /* 0xb6 */ Entry(0),
    /* 0xb7 */ Entry(0),
    /* 0xb8 */ Entry(0),
    /* 0xb9 */ Entry(0),
    /* 0xba */ Entry(0),
    /* 0xbb */ Entry(0),
    /* 0xbc */ Entry(0),
    /* 0xbd */ Entry(0),
    /* 0xbe */ Entry(0),
    /* 0xbf */ Entry(0),
    /* 0xc0 */ Entry(0),
    /* 0xc1 */ Entry(0),
    /* 0xc2 */ Entry(0),
    /* 0xc3 */ Entry(0),
    /* 0xc4 */ Entry(0),
    /* 0xc5 */ Entry(0),
    /* 0xc6 */ Entry(0),
    /* 0xc7 */ Entry(0),
    /* 0xc8 */ Entry(0),
    /* 0xc9 */ Entry(0),
    /* 0xca */ Entry(0),
    /* 0xcb */ Entry(0),
    /* 0xcc */ Entry(0),
    /* 0xcd */ Entry(0),
    /* 0xce */ Entry(0),
    /* 0xcf */ Entry(0),
    /* 0xd0 */ Entry(0),
    /* 0xd1 */ Entry(0),
    /* 0xd2 */ Entry(0),
    /* 0xd3 */ Entry(0),
    /* 0xd4 */ Entry(0),
    /* 0xd5 */ Entry(0),
    /* 0xd6 */ Entry(0),
    /* 0xd7 */ Entry(0),
    /* 0xd8 */ Entry(0),
    /* 0xd9 */ Entry(0),
    /* 0xda */ Entry(0),
    /* 0xdb */ Entry(0),
    /* 0xdc */ Entry(0),
    /* 0xdd */ Entry(0),
    /* 0xde */ Entry(0),
    /* 0xdf */ Entry(0),
    /* 0xe0 */ Entry(0),
    /* 0xe1 */ Entry(0),
    /* 0xe2 */ Entry(0),
    /* 0xe3 */ Entry(0),
    /* 0xe4 */ Entry(0),
    /* 0xe5 */ Entry(0),
    /* 0xe6 */ Entry(0),
    /* 0xe7 */ Entry(0),
    /* 0xe8 */ Entry(0),
    /* 0xe9 */ Entry(0),
    /* 0xea */ Entry(0),
    /* 0xeb */ Entry(0),
    /* 0xec */ Entry(0),
    /* 0xed */ Entry(0),
    /* 0xee */ Entry(0),
    /* 0xef */ Entry(0),
    /* 0xf0 */ Entry(0),
    /* 0xf1 */ Entry(0),
    /* 0xf2 */ Entry(0),
    /* 0xf3 */ Entry(0),
    /* 0xf4 */ Entry(0),
    /* 0xf5 */ Entry(0),
    /* 0xf6 */ Entry(0),
    /* 0xf7 */ Entry(0),
    /* 0xf8 */ Entry(0),
    /* 0xf9 */ Entry(0),
    /* 0xfa */ Entry(0),
    /* 0xfb */ Entry(0),
    /* 0xfc */ Entry(0),
    /* 0xfd */ Entry(0),
    /* 0xfe */ Entry(0),
    /* 0xff */ Entry(0),
];

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
