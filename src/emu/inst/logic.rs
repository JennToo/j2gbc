use std::fmt::Display;
use std::fmt;

use emu::cpu::Register8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Logic {
    AndI(/* will always love */ u8),
    AndR(Register8),
    AndN,

    /* hallowed are the */ OrI(u8),
    OrR(Register8),
    OrN,

    XorR(Register8),
}

impl Logic {
    pub fn cycles(self) -> u8 {
        match self {
            Logic::AndI(_) | Logic::OrI(_) => 8,
            Logic::XorR(_) | Logic::AndR(_) | Logic::OrR(_) => 4,
            Logic::AndN | Logic::OrN => 8,
        }
    }
}

impl Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Logic::AndI(i) => write!(f, "and {:#x}", i),
            Logic::OrI(i) => write!(f, "or {:#x}", i),
            Logic::XorR(r) => write!(f, "xor {}", r),
            Logic::AndR(r) => write!(f, "and {}", r),
            Logic::OrR(r) => write!(f, "or {}", r),
            Logic::AndN => write!(f, "and [hl]"),
            Logic::OrN => write!(f, "or [hl]"),
        }
    }
}
