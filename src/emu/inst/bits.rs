use std::fmt::Display;
use std::fmt;

use emu::cpu::Register8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bits {
    Cpl,
    SwapR(Register8),
    SlaR(Register8),
    RlR(Register8),
    Rra,
    Rrca,
    Rla,
    Rlca,
    Res(u8, Register8),
    Set(u8, Register8),
}

impl Bits {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Bits::Cpl => 4,
            Bits::Set(_, _) | Bits::Res(_, _) => 8,
            Bits::SwapR(_) => 8,
            Bits::RlR(_) | Bits::SlaR(_) => 8,
            Bits::Rra | Bits::Rrca | Bits::Rla | Bits::Rlca => 4,
        }
    }
}
impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Bits::Cpl => write!(f, "cpl"),
            Bits::Res(b, r) => write!(f, "res {},{}", b, r),
            Bits::Set(b, r) => write!(f, "set {},{}", b, r),
            Bits::SwapR(r) => write!(f, "swap {}", r),
            Bits::SlaR(r) => write!(f, "sla {}", r),
            Bits::RlR(r) => write!(f, "rl {}", r),
            Bits::Rra => write!(f, "rra"),
            Bits::Rrca => write!(f, "rrca"),
            Bits::Rla => write!(f, "rla"),
            Bits::Rlca => write!(f, "rlca"),
        }
    }
}
