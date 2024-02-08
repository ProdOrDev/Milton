//! The execution of fixed instructions and micro-instructions.

use arbitrary_int::{u1, u11, u3, u4, u5, u6, Number};

use super::{pla::Fixed, Tms1100};

impl Tms1100 {
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

impl Tms1100 {
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
