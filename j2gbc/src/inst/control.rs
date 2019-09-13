use std::fmt;
use std::fmt::Display;

use crate::cpu::ConditionCode;
use crate::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Control {
    JrCondI(i8, ConditionCode),
    JrI(i8),
    Ret,
    Reti,
    RetCond(ConditionCode),
    JpN,
    JpI(Address),
    JpCondI(Address, ConditionCode),
    CallI(Address),
    CallCondI(Address, ConditionCode),
    Rst(Address),
}

impl Control {
    pub fn cycles(self) -> u8 {
        match self {
            Control::Rst(_) => 16,
            Control::JpN => 4,
            Control::JpCondI(_, _) | Control::JpI(_) => 16,
            Control::CallI(_) | Control::CallCondI(_, _) => 24,
            // TODO: This is actually variable
            Control::JrCondI(_, _) | Control::JrI(_) => 12,
            Control::Ret | Control::Reti => 16,
            Control::RetCond(_) => 8,
        }
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::JrCondI(i, cond) => write!(f, "jr{} {}", cond, i),
            Control::JrI(i) => write!(f, "jr {}", i),
            Control::Ret => write!(f, "ret"),
            Control::Reti => write!(f, "reti"),
            Control::RetCond(cond) => write!(f, "ret {}", cond),
            Control::JpN => write!(f, "jmp [hl]"),
            Control::JpI(a) => write!(f, "jmp {}", a),
            Control::JpCondI(a, cond) => write!(f, "jmp{} {}", cond, a),
            Control::CallI(a) => write!(f, "call {}", a),
            Control::CallCondI(a, cond) => write!(f, "call{} {}", cond, a),
            Control::Rst(a) => write!(f, "rst {}", a),
        }
    }
}
