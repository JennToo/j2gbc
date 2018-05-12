use std::fmt;
use std::fmt::Display;

use emu::cpu::{Register16, Register8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    IncN,
    IncR(Register8),
    DecN,
    DecR(Register8),

    AddN,
    AddR(Register8),
    AddI(u8),

    AdcN,
    AdcR(Register8),
    AdcI(u8),

    SubN,
    SubR(Register8),
    SubI(u8),

    SbcN,
    SbcR(Register8),
    SbcI(u8),

    Daa,
    IncR16(Register16),
    DecR16(Register16),
    AddRR16(Register16, Register16),
}

impl Arith {
    pub fn cycles(self) -> u8 {
        match self {
            Arith::Daa => 4,
            Arith::SubR(_) | Arith::AddR(_) | Arith::SbcR(_) | Arith::AdcR(_) => 4,
            Arith::SubN | Arith::AddN | Arith::SbcN | Arith::AdcN => 8,
            Arith::SubI(_) | Arith::AddI(_) | Arith::SbcI(_) | Arith::AdcI(_) => 8,
            Arith::DecR16(_) | Arith::IncR16(_) => 8,
            Arith::IncN | Arith::DecN => 12,
            Arith::IncR(_) | Arith::DecR(_) => 4,
            Arith::AddRR16(_, _) => 16,
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arith::Daa => write!(f, "daa"),
            Arith::AddN => write!(f, "add [hl]"),
            Arith::AddR(r) => write!(f, "add {}", r),
            Arith::AddI(v) => write!(f, "add {:#x}", v),
            Arith::AdcN => write!(f, "adc [hl]"),
            Arith::AdcR(r) => write!(f, "adc {}", r),
            Arith::AdcI(v) => write!(f, "adc {:#x}", v),
            Arith::SubN => write!(f, "sub [hl]"),
            Arith::SubR(r) => write!(f, "sub {}", r),
            Arith::SubI(v) => write!(f, "sub {:#x}", v),
            Arith::SbcN => write!(f, "sbc [hl]"),
            Arith::SbcR(r) => write!(f, "sbc {}", r),
            Arith::SbcI(v) => write!(f, "sbc {:#x}", v),
            Arith::DecR(r) => write!(f, "dec {}", r),
            Arith::IncR(r) => write!(f, "inc {}", r),
            Arith::DecN => write!(f, "dec [hl]"),
            Arith::IncN => write!(f, "inc [hl]"),
            Arith::DecR16(r) => write!(f, "dec {}", r),
            Arith::IncR16(r) => write!(f, "inc {}", r),
            Arith::AddRR16(r1, r2) => write!(f, "add {},{}", r1, r2),
        }
    }
}
