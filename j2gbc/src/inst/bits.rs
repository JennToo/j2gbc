use std::fmt;
use std::fmt::Display;

use cpu::Operand;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bits {
    Cpl,
    Rra,
    Rrca,
    Rla,
    Rlca,
    Rlc(Operand),
    Rrc(Operand),
    Rl(Operand),
    Rr(Operand),
    Sla(Operand),
    Sra(Operand),
    Swap(Operand),
    Srl(Operand),
    Bit(u8, Operand),
    Res(u8, Operand),
    Set(u8, Operand),
}

impl Bits {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Bits::Cpl => 4,

            Bits::Set(_, Operand::Register(_))
            | Bits::Bit(_, Operand::Register(_))
            | Bits::Res(_, Operand::Register(_))
            | Bits::Swap(Operand::Register(_))
            | Bits::Rl(Operand::Register(_))
            | Bits::Rr(Operand::Register(_))
            | Bits::Rlc(Operand::Register(_))
            | Bits::Rrc(Operand::Register(_))
            | Bits::Sla(Operand::Register(_))
            | Bits::Sra(Operand::Register(_))
            | Bits::Srl(Operand::Register(_)) => 8,

            Bits::Set(_, Operand::IndirectRegister(_))
            | Bits::Bit(_, Operand::IndirectRegister(_))
            | Bits::Res(_, Operand::IndirectRegister(_))
            | Bits::Swap(Operand::IndirectRegister(_))
            | Bits::Rl(Operand::IndirectRegister(_))
            | Bits::Rr(Operand::IndirectRegister(_))
            | Bits::Rlc(Operand::IndirectRegister(_))
            | Bits::Rrc(Operand::IndirectRegister(_))
            | Bits::Sla(Operand::IndirectRegister(_))
            | Bits::Sra(Operand::IndirectRegister(_))
            | Bits::Srl(Operand::IndirectRegister(_)) => 8,

            Bits::Rra | Bits::Rrca | Bits::Rla | Bits::Rlca => 4,

            Bits::Set(_, _)
            | Bits::Bit(_, _)
            | Bits::Res(_, _)
            | Bits::Swap(_)
            | Bits::Rl(_)
            | Bits::Rr(_)
            | Bits::Rlc(_)
            | Bits::Rrc(_)
            | Bits::Sla(_)
            | Bits::Sra(_)
            | Bits::Srl(_) => unimplemented!(),
        }
    }
}
impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Bits::Cpl => write!(f, "cpl"),
            Bits::Bit(b, o) => write!(f, "bit {},{}", b, o),
            Bits::Res(b, o) => write!(f, "res {},{}", b, o),
            Bits::Set(b, o) => write!(f, "set {},{}", b, o),
            Bits::Swap(o) => write!(f, "swap {}", o),
            Bits::Sla(o) => write!(f, "sla {}", o),
            Bits::Sra(o) => write!(f, "sra {}", o),
            Bits::Rl(o) => write!(f, "rl {}", o),
            Bits::Rr(o) => write!(f, "rr {}", o),
            Bits::Rlc(o) => write!(f, "rlc {}", o),
            Bits::Rrc(o) => write!(f, "rrc {}", o),
            Bits::Srl(o) => write!(f, "srl {}", o),
            Bits::Rra => write!(f, "rra"),
            Bits::Rrca => write!(f, "rrca"),
            Bits::Rla => write!(f, "rla"),
            Bits::Rlca => write!(f, "rlca"),
        }
    }
}
