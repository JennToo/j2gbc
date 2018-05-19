use std::fmt;
use std::fmt::Display;

use emu::cpu::Operand;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bits {
    Cpl,
    Swap(Operand),
    Sla(Operand),
    Rl(Operand),
    Rr(Operand),
    Srl(Operand),
    Rra,
    Rrca,
    Rla,
    Rlca,
    Res(u8, Operand),
    Set(u8, Operand),
}

impl Bits {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Bits::Cpl => 4,

            Bits::Set(_, Operand::Register(_))
            | Bits::Res(_, Operand::Register(_))
            | Bits::Swap(Operand::Register(_))
            | Bits::Rl(Operand::Register(_))
            | Bits::Rr(Operand::Register(_))
            | Bits::Sla(Operand::Register(_))
            | Bits::Srl(Operand::Register(_)) => 8,

            Bits::Set(_, Operand::IndirectRegister(_))
            | Bits::Res(_, Operand::IndirectRegister(_))
            | Bits::Swap(Operand::IndirectRegister(_))
            | Bits::Rl(Operand::IndirectRegister(_))
            | Bits::Rr(Operand::IndirectRegister(_))
            | Bits::Sla(Operand::IndirectRegister(_))
            | Bits::Srl(Operand::IndirectRegister(_)) => 8,

            Bits::Rra | Bits::Rrca | Bits::Rla | Bits::Rlca => 4,

            Bits::Set(_, _)
            | Bits::Res(_, _)
            | Bits::Swap(_)
            | Bits::Rl(_)
            | Bits::Rr(_)
            | Bits::Sla(_)
            | Bits::Srl(_) => unimplemented!(),
        }
    }
}
impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Bits::Cpl => write!(f, "cpl"),
            Bits::Res(b, o) => write!(f, "res {},{}", b, o),
            Bits::Set(b, o) => write!(f, "set {},{}", b, o),
            Bits::Swap(o) => write!(f, "swap {}", o),
            Bits::Sla(o) => write!(f, "sla {}", o),
            Bits::Rl(o) => write!(f, "rl {}", o),
            Bits::Rr(o) => write!(f, "rr {}", o),
            Bits::Srl(o) => write!(f, "srl {}", o),
            Bits::Rra => write!(f, "rra"),
            Bits::Rrca => write!(f, "rrca"),
            Bits::Rla => write!(f, "rla"),
            Bits::Rlca => write!(f, "rlca"),
        }
    }
}
