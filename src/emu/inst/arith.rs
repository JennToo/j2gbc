use std::fmt::Display;
use std::fmt;

use emu::cpu::{Register16, Register8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    Cpl,
    IncN,
    IncR(Register8),
    DecR(Register8),
    AddN,
    AddR(Register8),
    AddI(u8),
    SubN,
    SubR(Register8),
    SubI(u8),
    Daa,
    IncR16(Register16),
    DecR16(Register16),
    AddRR16(Register16, Register16),
    SwapR(Register8),
    SlaR(Register8),
    RlR(Register8),
    Rra,
    Rrca,
    Rla,
    Rlca,
}

impl Arith {
    pub fn cycles(self) -> u8 {
        match self {
            Arith::Cpl | Arith::Daa => 4,
            Arith::SubR(_) | Arith::AddR(_) => 4,
            Arith::SubN | Arith::AddN => 8,
            Arith::SubI(_) | Arith::AddI(_) => 8,
            Arith::DecR16(_) | Arith::IncR16(_) => 8,
            Arith::IncN => 12,
            Arith::IncR(_) | Arith::DecR(_) => 4,
            Arith::AddRR16(_, _) => 16,
            Arith::SwapR(_) => 8,
            Arith::RlR(_) | Arith::SlaR(_) => 8,
            Arith::Rra | Arith::Rrca | Arith::Rla | Arith::Rlca => 4,
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arith::Cpl => write!(f, "cpl"),
            Arith::Daa => write!(f, "daa"),
            Arith::AddN => write!(f, "add [hl]"),
            Arith::AddR(r) => write!(f, "add {}", r),
            Arith::AddI(v) => write!(f, "add {:#x}", v),
            Arith::SubN => write!(f, "sub [hl]"),
            Arith::SubR(r) => write!(f, "sub {}", r),
            Arith::SubI(v) => write!(f, "sub {:#x}", v),
            Arith::DecR(r) => write!(f, "dec {}", r),
            Arith::IncR(r) => write!(f, "inc {}", r),
            Arith::IncN => write!(f, "inc [hl]"),
            Arith::DecR16(r) => write!(f, "dec {}", r),
            Arith::IncR16(r) => write!(f, "inc {}", r),
            Arith::AddRR16(r1, r2) => write!(f, "add {},{}", r1, r2),
            Arith::SwapR(r) => write!(f, "swap {}", r),
            Arith::SlaR(r) => write!(f, "sla {}", r),
            Arith::RlR(r) => write!(f, "rl {}", r),
            Arith::Rra => write!(f, "rra"),
            Arith::Rrca => write!(f, "rrca"),
            Arith::Rla => write!(f, "rla"),
            Arith::Rlca => write!(f, "rlca"),
        }
    }
}
