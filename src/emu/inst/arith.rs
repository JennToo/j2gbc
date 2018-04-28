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
    IncR16(Register16),
    DecR16(Register16),
    AddRR16(Register16, Register16),
}

impl Arith {
    pub fn cycles(self) -> u8 {
        match self {
            Arith::Cpl => 4,
            Arith::AddR(_) => 4,
            Arith::AddN => 8,
            Arith::DecR16(_) | Arith::IncR16(_) => 8,
            Arith::IncN => 12,
            Arith::IncR(_) | Arith::DecR(_) => 4,
            Arith::AddRR16(_, _) => 16,
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arith::Cpl => write!(f, "cpl"),
            Arith::AddN => write!(f, "add [hl]"),
            Arith::AddR(r) => write!(f, "add {}", r),
            Arith::DecR(r) => write!(f, "dec {}", r),
            Arith::IncR(r) => write!(f, "inc {}", r),
            Arith::IncN => write!(f, "inc [hl]"),
            Arith::DecR16(r) => write!(f, "dec {}", r),
            Arith::IncR16(r) => write!(f, "inc {}", r),
            Arith::AddRR16(r1, r2) => write!(f, "add {},{}", r1, r2),
        }
    }
}
