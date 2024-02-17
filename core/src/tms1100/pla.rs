//! Emulation of the TMS1100's Programmable Logic Array (PLA) used for instruction decoding.
//!
//! The very nature of the term PLA suggests something that is modifiable per-chip, however
//! this PLA implementation is fine-tuned to execute the standard configuration of the
//! TMS1100's PLA, which every (official) Microvision cartridge uses.

/// The standard set of PLA micro-instructions.
pub mod instructions {
    /// The micro-instruction `CKP`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data on the internal `CKI` data bus,
    /// which varies depending on the current instruction, is latched to the `P` input of
    /// the adder circuit.
    pub const CKP: u16 = 1 << 0;

    /// The micro-instruction `YTP`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data contained within the the `Y`
    /// register is latched to the `P` input of the adder circuit.
    pub const YTP: u16 = 1 << 1;

    /// The micro-instruction `MTP`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data in RAM at `(X, Y)` is latched to
    /// the `P` input of the adder circuit.
    pub const MTP: u16 = 1 << 2;

    /// The micro-instruction `ATN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data contained within the `A` accumulator
    /// is latched to the `N` input of the adder circuit.
    pub const ATN: u16 = 1 << 3;

    /// The micro-instruction `NATN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data contained within the `A` accumulator
    /// is inverted, without modifying the accumulator, then latched to the `N` input of the
    /// adder circuit.
    pub const NATN: u16 = 1 << 4;

    /// The micro-instruction `MTN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data in RAM at `(X, Y)` is latched to
    /// the `N` input of the adder circuit.
    ///
    /// # Notes
    ///
    /// This micro-instruction is slightly odd because it exists on the chip however, none
    // of the PLA opcode combinations actually make use of it. My best thought is that is
    /// reserved for future revisions of the processor.
    pub const MTN: u16 = 1 << 5;

    /// The micro-instruction `15TN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, a value of `15` or `0xf` is latched to the
    /// `N` input of the adder circuit.
    ///
    /// # Notes
    ///
    /// Due to the naming restrictions of Rust, this value has to be called `FTN`, which is
    /// still accurate because `0xf` means `15`.
    pub const FTN: u16 = 1 << 6;

    /// The micro-instruction `CKN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data on the internal `CKI` data bus,
    /// which varies depending on the current instruction, is latched to the `N` input of
    /// the adder circuit.
    pub const CKN: u16 = 1 << 7;

    /// The micro-instruction `CIN`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the carry input line of the adder circuit
    /// is pulled high, which allows incrementing a value by one.
    pub const CIN: u16 = 1 << 8;

    /// The micro-instruction `NE`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the adder circuit is instructed, in addition
    /// to the normal add operation, compare the `P` and `N` inputs, setting the status output
    /// to one if they are non-equal.
    pub const NE: u16 = 1 << 9;

    /// The micro-instruction `C8`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the adder circuit copies the carry result of
    /// its add operation to the status output.
    pub const C8: u16 = 1 << 10;

    /// The micro-instruction `STO`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the `A` accumulator is written into RAM at
    /// `(X, Y)`.
    pub const STO: u16 = 1 << 11;

    /// The micro-instruction `CKM`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the data on the internal `CKI` data bus,
    /// which varies depending on the current instruction, is written into RAM at `(X, Y)`.
    pub const CKM: u16 = 1 << 12;

    /// The micro-instruction `AUTA`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the adder circuit copies the result of its
    /// add operation into the `A` accumulator.
    pub const AUTA: u16 = 1 << 13;

    /// The micro-instruction `AUTY`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the adder circuit copies the result of its
    /// add operation into the `Y` register.
    pub const AUTY: u16 = 1 << 14;

    /// The micro-instruction `STSL`.
    ///
    /// # Logic
    ///
    /// When this micro-instruction is enabled, the adder circuit copies its status output
    /// into the internal `SL` status latch.
    pub const STSL: u16 = 1 << 15;

    /// A compile-time validator for micro-instruction values.
    ///
    /// # Why?
    ///
    /// Currently, Rust only allows primitive types for const-generics, therefore there
    /// needs to be some compile-time check for enums represented as integers.
    pub(crate) trait IsValid {}

    /// A compile-time reference to a micro-instruction.
    pub(crate) struct InstructionRef<const M: u16>;

    impl IsValid for InstructionRef<CKP> {}
    impl IsValid for InstructionRef<YTP> {}
    impl IsValid for InstructionRef<MTP> {}
    impl IsValid for InstructionRef<ATN> {}
    impl IsValid for InstructionRef<NATN> {}
    impl IsValid for InstructionRef<MTN> {}
    impl IsValid for InstructionRef<FTN> {}
    impl IsValid for InstructionRef<CKN> {}
    impl IsValid for InstructionRef<CIN> {}
    impl IsValid for InstructionRef<NE> {}
    impl IsValid for InstructionRef<C8> {}
    impl IsValid for InstructionRef<STO> {}
    impl IsValid for InstructionRef<CKM> {}
    impl IsValid for InstructionRef<AUTA> {}
    impl IsValid for InstructionRef<AUTY> {}
    impl IsValid for InstructionRef<STSL> {}
}

use instructions::{
    InstructionRef, IsValid, ATN, AUTA, AUTY, C8, CIN, CKM, CKN, CKP, FTN, MTP, NATN, NE, STO,
    STSL, YTP,
};

/// A micro-instruction entry in the TMS1100's instruction decode PLA.
///
/// These entries control which micro-instructions are enabled for the given opcode
/// or instruction. However, not every instruction uses the PLA for execution, some
/// opcodes are decoded using a [Fixed] (non-programmable) logic scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry(pub(crate) u16);

impl Entry {
    /// An empty micro-instruction PLA entry.
    ///
    /// This is used when first running the TSM1100, during its initialization stage.
    pub(crate) const EMPTY: Self = Self(0);

    /// Check if the PLA entry enables a micro-instruction.
    #[must_use]
    #[allow(private_bounds)]
    pub fn enables<const M: u16>(&self) -> bool
    where
        InstructionRef<M>: IsValid,
    {
        self.0 & M != 0
    }

    /// Return the PLA entry for the given opcode.
    ///
    /// Instead of mimicking a full PLA, e.g. using `AND`/`OR` gates, we simply use
    /// a function that maps opcodes to their known micro-instructions.
    #[must_use]
    pub fn decode(opcode: u8) -> Self {
        match opcode {
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
            _ => Self::EMPTY,
        }
    }
}

impl From<u16> for Entry {
    fn from(val: u16) -> Self {
        Self(val)
    }
}

/// A set of 12 fixed-instructions.
///
/// Unlike most instructions with can be modified via the PLA, certain instructions
/// like branch/call have fixed (non-programmable) logic decoders. However, this
/// does not mean the PLA slots for these opcodes are useless, in fact activating
/// certain micro-instructions in the PLA for these fixed-instructions can enable
/// completely new instructions to be formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixed {
    /// The fixed-instruction `BR`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction performs a branch operation to another point in ROM.
    Br,
    /// The fixed-instruction `CALL`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction performs a subroutine call to another point in ROM.
    Call,
    /// The fixed-instruction `RETN`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction performs a subroutine return to another point in ROM.
    Retn,
    /// The fixed-instruction `COMC`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction complements (inverts) the `CB` chapter buffer.
    Comc,
    /// The fixed-instruction `COMX`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction complements (inverts) the `X` register.
    Comx,
    /// The fixed-instruction `LDP`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction loads the `PB` page buffer with a constant value.
    Ldp,
    /// The fixed-instruction `LDX`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction loads the `X` register with a constant value.
    Ldx,
    /// The fixed-instruction `RBIT`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction resets the bit, denoted by a constant value, back to
    /// zero in RAM at `(X, Y)`.
    Rbit,
    /// The fixed-instruction `SBIT`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction sets the bit, denoted by a constant value, to one in
    /// RAM at `(X, Y)`.
    Sbit,
    /// The fixed-instruction `RSTR`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction resets the bit, denoted by the `X` and `Y` registers,
    /// back to zero in the `R` pin output.
    Rstr,
    /// The fixed-instruction `SETR`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction sets the bit, denoted by the `X` and `Y` registers, to
    /// one in the `R` pin output.
    Setr,
    /// The fixed-instruction `TDO`.
    ///
    /// # Logic
    ///
    /// This fixed-instruction transfers the data from the accumulator and status latch
    /// into the `O` pin output.
    Tdo,
}

impl Fixed {
    /// Return the fixed-instruction for the given opcode.
    ///
    /// In the case that the given opcode does not represent one of the 12 fixed-instructions,
    /// a [None] value will be returned.
    #[must_use]
    pub fn decode(opcode: u8) -> Option<Self> {
        let fixed = match opcode {
            0x09 => Self::Comx,
            0x0a => Self::Tdo,
            0x0b => Self::Comc,
            0x0c => Self::Rstr,
            0x0d => Self::Setr,
            0x0f => Self::Retn,
            0x30..=0x33 => Self::Sbit,
            0x34..=0x37 => Self::Rbit,
            0x10..=0x1f => Self::Ldp,
            0x28..=0x2f => Self::Ldx,
            0x80..=0xbf => Self::Br,
            0xc0..=0xff => Self::Call,
            _ => return None,
        };

        Some(fixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Define a test function for instruction decoding.
    ///
    /// This verifies if the correct micro/fixed-instructions are enabled for a given opcode.
    macro_rules! opcode {
        ($name:ident, $op:literal, $entry:expr, $fixed:expr) => {
            #[test]
            fn $name() {
                assert_eq!(Entry::decode($op), Entry::from($entry));
                assert_eq!(Fixed::decode($op), $fixed);
            }
        };
    }

    /// Define a test function for instruction decoding.
    ///
    /// This verifies if the correct micro/fixed-instructions are enabled for a given opcode range.
    macro_rules! opcode_range {
        ($name:ident, $range:expr, $entry:expr, $fixed:expr) => {
            #[test]
            fn $name() {
                for op in $range {
                    assert_eq!(Entry::decode(op), Entry::from($entry));
                    assert_eq!(Fixed::decode(op), $fixed);
                }
            }
        };
    }

    opcode!(mnea, 0x00, MTP | ATN | NE, None);
    opcode!(alem, 0x01, MTP | NATN | CIN | C8, None);
    opcode!(ynea, 0x02, YTP | ATN | NE | STSL, None);
    opcode!(xma, 0x03, MTP | STO | AUTA, None);
    opcode!(r#dyn, 0x04, YTP | FTN | C8 | AUTY, None);
    opcode!(iyc, 0x05, YTP | CIN | C8 | AUTY, None);
    opcode!(amaac, 0x06, ATN | MTP | C8 | AUTA, None);
    opcode!(dman, 0x07, MTP | FTN | C8 | AUTA, None);
    opcode!(tka, 0x08, CKP | AUTA, None);
    opcode!(comx, 0x09, Entry::EMPTY, Some(Fixed::Comx));
    opcode!(tdo, 0x0a, Entry::EMPTY, Some(Fixed::Tdo));
    opcode!(comc, 0x0b, Entry::EMPTY, Some(Fixed::Comc));
    opcode!(rstr, 0x0c, Entry::EMPTY, Some(Fixed::Rstr));
    opcode!(setr, 0x0d, Entry::EMPTY, Some(Fixed::Setr));
    opcode!(knez, 0x0e, CKP | NE, None);
    opcode!(retn, 0x0f, Entry::EMPTY, Some(Fixed::Retn));
    opcode_range!(ldp, 0x10..=0x1f, Entry::EMPTY, Some(Fixed::Ldp));
    opcode!(tay, 0x20, ATN | AUTY, None);
    opcode!(tma, 0x21, MTP | AUTA, None);
    opcode!(tmy, 0x22, MTP | AUTY, None);
    opcode!(tya, 0x23, YTP | AUTA, None);
    opcode!(tamdyn, 0x24, STO | YTP | FTN | C8 | AUTY, None);
    opcode!(tamiyc, 0x25, STO | YTP | CIN | C8 | AUTY, None);
    opcode!(tamza, 0x26, STO | AUTA, None);
    opcode!(tam, 0x27, STO, None);
    opcode_range!(ldx, 0x28..=0x2f, Entry::EMPTY, Some(Fixed::Ldx));
    opcode_range!(sbit, 0x30..=0x33, Entry::EMPTY, Some(Fixed::Sbit));
    opcode_range!(rbit, 0x34..=0x37, Entry::EMPTY, Some(Fixed::Rbit));
    opcode_range!(tbit1, 0x38..=0x3b, CKP | CKN | MTP | NE, None);
    opcode!(saman, 0x3c, MTP | NATN | CIN | C8 | AUTA, None);
    opcode!(cpaiz, 0x3d, NATN | CIN | C8 | AUTA, None);
    opcode!(imac, 0x3e, MTP | CIN | C8 | AUTA, None);
    opcode!(mnez, 0x3f, MTP | NE, None);
    opcode_range!(tcy, 0x40..=0x4f, CKP | AUTY, None);
    opcode_range!(ynec, 0x50..=0x5f, YTP | CKN | NE, None);
    opcode_range!(tcmiy, 0x60..=0x6f, CKM | YTP | CIN | AUTY, None);
    opcode_range!(ac1ac, 0x70..=0x7e, CKP | ATN | CIN | C8 | AUTA, None);
    opcode!(cla, 0x7f, CKP | CIN | C8 | AUTA, None);
    opcode_range!(br, 0x80..=0xbf, Entry::EMPTY, Some(Fixed::Br));
    opcode_range!(call, 0xc0..=0xff, Entry::EMPTY, Some(Fixed::Call));
}
