use std::fmt::Display;
use std::fmt;

use emu::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Control {
    JrNZI(i8),
    JrI(i8),
    JrNCI(i8),
    JrZI(i8),
    JrCI(i8),
    Ret,
    JpI(Address),
    JpCI(Address),
    JpZI(Address),
    JpNCI(Address),
    JpNZI(Address),
    CallI(Address),
}

impl Control {
    pub fn cycles(self) -> u8 {
        match self {
            Control::JpCI(_)
            | Control::JpI(_)
            | Control::JpZI(_)
            | Control::JpNCI(_)
            | Control::JpNZI(_) => 16,
            Control::CallI(_) => 24,
            // TODO: This is actually variable
            Control::JrNZI(_)
            | Control::JrNCI(_)
            | Control::JrI(_)
            | Control::JrCI(_)
            | Control::JrZI(_) => 12,
            Control::Ret => 16,
        }
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::JrNZI(i) => write!(f, "jrnz {}", i),
            Control::JrI(i) => write!(f, "jr {}", i),
            Control::JrNCI(i) => write!(f, "jrnc {}", i),
            Control::JrZI(i) => write!(f, "jrz {}", i),
            Control::JrCI(i) => write!(f, "jrc {}", i),
            Control::Ret => write!(f, "ret"),
            Control::JpI(a) => write!(f, "jmp {}", a),
            Control::JpCI(a) => write!(f, "jmpc {}", a),
            Control::JpZI(a) => write!(f, "jmpz {}", a),
            Control::JpNCI(a) => write!(f, "jmpnc {}", a),
            Control::JpNZI(a) => write!(f, "jmpnz {}", a),
            Control::CallI(a) => write!(f, "call {}", a),
        }
    }
}
