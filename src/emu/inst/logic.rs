use std::fmt;
use std::fmt::Display;

use emu::cpu::Register8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Logic {
    AndI(/* will always love */ u8),
    AndR(Register8),
    AndN,

    /* hallowed are the */ OrI(u8),
    OrR(Register8),
    OrN,

    XorI(u8),
    XorR(Register8),
    XorN,
}

impl Logic {
    pub fn cycles(self) -> u8 {
        match self {
            Logic::AndI(_) | Logic::OrI(_) | Logic::XorI(_) => 8,
            Logic::XorR(_) | Logic::AndR(_) | Logic::OrR(_) => 4,
            Logic::AndN | Logic::OrN | Logic::XorN => 8,
        }
    }
}

impl Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Logic::AndI(i) => write!(f, "and {:#x}", i),
            Logic::OrI(i) => write!(f, "or {:#x}", i),
            Logic::XorI(i) => write!(f, "xor {:#x}", i),
            Logic::XorR(r) => write!(f, "xor {}", r),
            Logic::AndR(r) => write!(f, "and {}", r),
            Logic::OrR(r) => write!(f, "or {}", r),
            Logic::AndN => write!(f, "and [hl]"),
            Logic::OrN => write!(f, "or [hl]"),
            Logic::XorN => write!(f, "xor [hl]"),
        }
    }
}
