use std::fmt;
use std::fmt::Display;

use emu::cpu::{Operand, Register16};
use emu::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Load {
    Ld(Operand, Operand),
    LdNA(i8),
    LdAN(i8),
    LdNCA,
    LdANC,
    LdRI16(Register16, u16),
    LdNIA16(Address),
    LdANI16(Address),
    LdNR16(Register16),
    LdRN16(Register16),
    LdHLSPI(i8),
    LdSPHL,
    Push(Register16),
    Pop(Register16),
}

impl Load {
    pub fn cycles(self) -> u8 {
        match self {
            Load::Ld(Operand::Register(_), Operand::Register(_)) => 4,

            Load::Ld(Operand::Register(_), Operand::Immediate(_))
            | Load::Ld(Operand::Register(_), Operand::IndirectRegister(_))
            | Load::Ld(Operand::IndirectRegister(_), Operand::Register(_)) => 8,

            Load::Ld(Operand::Register(_), Operand::IndirectAddress(_))
            | Load::Ld(Operand::IndirectAddress(_), Operand::Register(_))
            | Load::Ld(Operand::IndirectRegister(_), Operand::Immediate(_))
            | Load::LdRI16(_, _)
            | Load::LdHLSPI(_) => 12,
            Load::LdNA(_) | Load::LdAN(_) => 8,
            Load::LdNR16(_) | Load::LdRN16(_) => 8,
            Load::LdNIA16(_) | Load::LdANI16(_) => 16,
            Load::LdNCA => 8,
            Load::LdANC => 8,
            Load::LdSPHL => 8,
            Load::Push(_) => 16,
            Load::Pop(_) => 12,

            Load::Ld(_, _) => unimplemented!(),
        }
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Load::Ld(o1, o2) => write!(f, "ld {},{}", o1, o2),
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
            Load::LdHLSPI(v) => write!(f, "ld hl,sp+{:#x}", v),
            Load::LdSPHL => write!(f, "ld sp,hl"),
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
