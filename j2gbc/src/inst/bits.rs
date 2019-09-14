use std::fmt;
use std::fmt::Display;

use crate::cpu::Operand;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bits {
    Complement,
    RotateRightAccumulator,
    RotateRightCarryAccumulator,
    RotateLeftAccumulator,
    RotateLeftCarryAccumulator,
    RotateLeftCarry(Operand),
    RotateRightCarry(Operand),
    RotateLeft(Operand),
    RotateRight(Operand),
    ShiftLeftArithmetic(Operand),
    ShiftRightArithmetic(Operand),
    Swap(Operand),
    ShiftRightLogical(Operand),
    GetBit(u8, Operand),
    ResetBit(u8, Operand),
    SetBit(u8, Operand),
}

impl Bits {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Bits::Complement => 4,

            Bits::SetBit(_, Operand::Register(_))
            | Bits::GetBit(_, Operand::Register(_))
            | Bits::ResetBit(_, Operand::Register(_))
            | Bits::Swap(Operand::Register(_))
            | Bits::RotateLeft(Operand::Register(_))
            | Bits::RotateRight(Operand::Register(_))
            | Bits::RotateLeftCarry(Operand::Register(_))
            | Bits::RotateRightCarry(Operand::Register(_))
            | Bits::ShiftLeftArithmetic(Operand::Register(_))
            | Bits::ShiftRightArithmetic(Operand::Register(_))
            | Bits::ShiftRightLogical(Operand::Register(_)) => 8,

            Bits::SetBit(_, Operand::IndirectRegister(_))
            | Bits::GetBit(_, Operand::IndirectRegister(_))
            | Bits::ResetBit(_, Operand::IndirectRegister(_))
            | Bits::Swap(Operand::IndirectRegister(_))
            | Bits::RotateLeft(Operand::IndirectRegister(_))
            | Bits::RotateRight(Operand::IndirectRegister(_))
            | Bits::RotateLeftCarry(Operand::IndirectRegister(_))
            | Bits::RotateRightCarry(Operand::IndirectRegister(_))
            | Bits::ShiftLeftArithmetic(Operand::IndirectRegister(_))
            | Bits::ShiftRightArithmetic(Operand::IndirectRegister(_))
            | Bits::ShiftRightLogical(Operand::IndirectRegister(_)) => 8,

            Bits::RotateRightAccumulator
            | Bits::RotateRightCarryAccumulator
            | Bits::RotateLeftAccumulator
            | Bits::RotateLeftCarryAccumulator => 4,

            Bits::SetBit(_, _)
            | Bits::GetBit(_, _)
            | Bits::ResetBit(_, _)
            | Bits::Swap(_)
            | Bits::RotateLeft(_)
            | Bits::RotateRight(_)
            | Bits::RotateLeftCarry(_)
            | Bits::RotateRightCarry(_)
            | Bits::ShiftLeftArithmetic(_)
            | Bits::ShiftRightArithmetic(_)
            | Bits::ShiftRightLogical(_) => unimplemented!(),
        }
    }
}
impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Bits::Complement => write!(f, "cpl"),
            Bits::GetBit(b, o) => write!(f, "bit {},{}", b, o),
            Bits::ResetBit(b, o) => write!(f, "res {},{}", b, o),
            Bits::SetBit(b, o) => write!(f, "set {},{}", b, o),
            Bits::Swap(o) => write!(f, "swap {}", o),
            Bits::ShiftLeftArithmetic(o) => write!(f, "sla {}", o),
            Bits::ShiftRightArithmetic(o) => write!(f, "sra {}", o),
            Bits::RotateLeft(o) => write!(f, "rl {}", o),
            Bits::RotateRight(o) => write!(f, "rr {}", o),
            Bits::RotateLeftCarry(o) => write!(f, "rlc {}", o),
            Bits::RotateRightCarry(o) => write!(f, "rrc {}", o),
            Bits::ShiftRightLogical(o) => write!(f, "srl {}", o),
            Bits::RotateRightAccumulator => write!(f, "rra"),
            Bits::RotateRightCarryAccumulator => write!(f, "rrca"),
            Bits::RotateLeftAccumulator => write!(f, "rla"),
            Bits::RotateLeftCarryAccumulator => write!(f, "rlca"),
        }
    }
}
