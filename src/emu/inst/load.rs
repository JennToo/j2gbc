use std::fmt;
use std::fmt::Display;

use emu::cpu::{Register16, Register8};
use emu::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Load {
    LdRM(Register8, Address),
    LdMR(Address, Register8),
    LdRR(Register8, Register8),
    LdRI(Register8, u8),
    LdRN(Register8),
    LdNR(Register8),
    LdNA(i8),
    LdAN(i8),
    LdNI(u8),
    LdNCA,
    LdANC,
    LdRI16(Register16, u16),
    LdNIA16(Address),
    LdANI16(Address),
    LdNR16(Register16),
    LdRN16(Register16),
    Push(Register16),
    Pop(Register16),
}

impl Load {
    pub fn cycles(self) -> u8 {
        match self {
            Load::LdRM(_, _) | Load::LdRI16(_, _) | Load::LdMR(_, _) => 12,
            Load::LdRR(_, _) => 4,
            Load::LdRI(_, _) => 8,
            Load::LdNA(_) | Load::LdAN(_) | Load::LdRN(_) | Load::LdNR(_) => 8,
            Load::LdNI(_) => 12,
            Load::LdNR16(_) | Load::LdRN16(_) => 8,
            Load::LdNIA16(_) | Load::LdANI16(_) => 16,
            Load::LdNCA => 8,
            Load::LdANC => 8,
            Load::Push(_) => 16,
            Load::Pop(_) => 12,
        }
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Load::LdRM(r, a) => write!(f, "ld {},[{}]", r, a),
            Load::LdMR(a, r) => write!(f, "ld [{}],{}", a, r),
            Load::LdRR(r1, r2) => write!(f, "ld {},{}", r1, r2),
            Load::LdRI(r, v) => write!(f, "ld {},{:#x}", r, v),
            Load::LdRN(r) => write!(f, "ld {},[hl]", r),
            Load::LdNR(r) => write!(f, "ld [hl],{}", r),
            Load::LdNA(i) => {
                if i > 0 {
                    write!(f, "ld [hl+],a")
                } else {
                    write!(f, "ld [hl-],a")
                }
            }
            Load::LdAN(i) => {
                if i > 0 {
                    write!(f, "ld a,[hl+]")
                } else {
                    write!(f, "ld a,[hl-]")
                }
            }
            Load::LdNI(v) => write!(f, "ld [hl],{:#x}", v),
            Load::LdNCA => write!(f, "ld [c+0xff00],a"),
            Load::LdANC => write!(f, "ld a,[c+0xff00]"),
            Load::LdRI16(r, v) => write!(f, "ld {},{:#x}", r, v),
            Load::LdNIA16(a) => write!(f, "ld [{}],a", a),
            Load::LdANI16(a) => write!(f, "ld a,[{}]", a),
            Load::LdNR16(r) => write!(f, "ld [{}],a", r),
            Load::LdRN16(r) => write!(f, "ld a,[{}]", r),
            Load::Push(r) => write!(f, "push {}", r),
            Load::Pop(r) => write!(f, "pop {}", r),
        }
    }
}
