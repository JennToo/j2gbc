use std::fmt;
use std::fmt::Display;

use crate::cpu::Register8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Logic {
    AndImmediate(u8),
    AndRegister(Register8),
    AndIndirect,

    OrImmediate(u8),
    OrRegister(Register8),
    OrIndirect,

    XorImmediate(u8),
    XorRegister(Register8),
    XorIndirect,
}

impl Logic {
    pub fn cycles(self) -> u8 {
        match self {
            Logic::AndImmediate(_) | Logic::OrImmediate(_) | Logic::XorImmediate(_) => 8,
            Logic::XorRegister(_) | Logic::AndRegister(_) | Logic::OrRegister(_) => 4,
            Logic::AndIndirect | Logic::OrIndirect | Logic::XorIndirect => 8,
        }
    }
}

impl Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Logic::AndImmediate(i) => write!(f, "and {:#x}", i),
            Logic::OrImmediate(i) => write!(f, "or {:#x}", i),
            Logic::XorImmediate(i) => write!(f, "xor {:#x}", i),
            Logic::XorRegister(r) => write!(f, "xor {}", r),
            Logic::AndRegister(r) => write!(f, "and {}", r),
            Logic::OrRegister(r) => write!(f, "or {}", r),
            Logic::AndIndirect => write!(f, "and [hl]"),
            Logic::OrIndirect => write!(f, "or [hl]"),
            Logic::XorIndirect => write!(f, "xor [hl]"),
        }
    }
}
