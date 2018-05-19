use std::fmt;
use std::fmt::Display;

use emu::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Register(Register8),
    IndirectRegister(Register16),
    IndirectAddress(Address),
    Immediate(u8),
}

impl Display for Register8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register8::A => write!(f, "a"),
            Register8::B => write!(f, "b"),
            Register8::C => write!(f, "c"),
            Register8::D => write!(f, "d"),
            Register8::E => write!(f, "e"),
            Register8::F => write!(f, "f"),
            Register8::H => write!(f, "h"),
            Register8::L => write!(f, "l"),
        }
    }
}

impl Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register16::AF => write!(f, "af"),
            Register16::BC => write!(f, "bc"),
            Register16::DE => write!(f, "de"),
            Register16::HL => write!(f, "hl"),
            Register16::SP => write!(f, "sp"),
            Register16::PC => write!(f, "pc"),
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Immediate(v) => write!(f, "{:#x}", v),
            Operand::IndirectAddress(a) => write!(f, "[{}]", a),
            Operand::IndirectRegister(r) => write!(f, "[{}]", r),
            Operand::Register(r) => r.fmt(f),
        }
    }
}
