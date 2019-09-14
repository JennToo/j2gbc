use std::fmt;
use std::fmt::Display;

use crate::cpu::{Operand, Register16};
use crate::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Load {
    Load(Operand, Operand),
    LoadIndirectFromA(i8),
    LoadAFromIndirect(i8),
    LoadIndirectHiFromA,
    LoadAFromIndirectHi,
    LoadRegisterImmediate16(Register16, u16),
    LoadMemoryFromA(Address),
    LoadAFromMemory(Address),
    LoadIndirectRegisterFromA(Register16),
    LoadAFromIndirectRegister(Register16),
    LoadHLFromSP(i8),
    LoadSPFromHL,
    LoadMemoryFromSP(Address),
    Push(Register16),
    Pop(Register16),
}

impl Load {
    pub fn cycles(self) -> u8 {
        match self {
            Load::Load(Operand::Register(_), Operand::Register(_)) => 4,

            Load::Load(Operand::Register(_), Operand::Immediate(_))
            | Load::Load(Operand::Register(_), Operand::IndirectRegister(_))
            | Load::Load(Operand::IndirectRegister(_), Operand::Register(_)) => 8,

            Load::Load(Operand::Register(_), Operand::IndirectAddress(_))
            | Load::Load(Operand::IndirectAddress(_), Operand::Register(_))
            | Load::Load(Operand::IndirectRegister(_), Operand::Immediate(_))
            | Load::LoadRegisterImmediate16(_, _)
            | Load::LoadHLFromSP(_) => 12,
            Load::LoadIndirectFromA(_) | Load::LoadAFromIndirect(_) => 8,
            Load::LoadIndirectRegisterFromA(_) | Load::LoadAFromIndirectRegister(_) => 8,
            Load::LoadMemoryFromA(_) | Load::LoadAFromMemory(_) => 16,
            Load::LoadIndirectHiFromA => 8,
            Load::LoadAFromIndirectHi => 8,
            Load::LoadSPFromHL => 8,
            Load::Push(_) => 16,
            Load::Pop(_) => 12,

            Load::LoadMemoryFromSP(_) => 20,

            Load::Load(_, _) => unimplemented!(),
        }
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Load::Load(o1, o2) => write!(f, "ld {},{}", o1, o2),
            Load::LoadIndirectFromA(i) => {
                if i > 0 {
                    write!(f, "ld [hl+],a")
                } else {
                    write!(f, "ld [hl-],a")
                }
            }
            Load::LoadAFromIndirect(i) => {
                if i > 0 {
                    write!(f, "ld a,[hl+]")
                } else {
                    write!(f, "ld a,[hl-]")
                }
            }
            Load::LoadHLFromSP(v) => write!(f, "ld hl,sp+{:#x}", v),
            Load::LoadSPFromHL => write!(f, "ld sp,hl"),
            Load::LoadIndirectHiFromA => write!(f, "ld [c+0xff00],a"),
            Load::LoadAFromIndirectHi => write!(f, "ld a,[c+0xff00]"),
            Load::LoadRegisterImmediate16(r, v) => write!(f, "ld {},{:#x}", r, v),
            Load::LoadMemoryFromA(a) => write!(f, "ld [{}],a", a),
            Load::LoadAFromMemory(a) => write!(f, "ld a,[{}]", a),
            Load::LoadIndirectRegisterFromA(r) => write!(f, "ld [{}],a", r),
            Load::LoadAFromIndirectRegister(r) => write!(f, "ld a,[{}]", r),
            Load::LoadMemoryFromSP(a) => write!(f, "ld [{}],sp", a),
            Load::Push(r) => write!(f, "push {}", r),
            Load::Pop(r) => write!(f, "pop {}", r),
        }
    }
}
