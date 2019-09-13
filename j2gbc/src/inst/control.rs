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
    CallINZ(Address),
    CallINC(Address),
    CallIC(Address),
    CallIZ(Address),
    Rst(Address),
}

impl Control {
    pub fn cycles(self) -> u8 {
        match self {
            Control::Rst(_) => 16,
            Control::JpN => 4,
            Control::JpCondI(_, _) | Control::JpI(_) => 16,
            Control::CallI(_)
            | Control::CallIC(_)
            | Control::CallINC(_)
            | Control::CallINZ(_)
            | Control::CallIZ(_) => 24,
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
            Control::CallIC(a) => write!(f, "callc {}", a),
            Control::CallIZ(a) => write!(f, "callz {}", a),
            Control::CallINC(a) => write!(f, "callnc {}", a),
            Control::CallINZ(a) => write!(f, "callnz {}", a),
            Control::Rst(a) => write!(f, "rst {}", a),
        }
    }
}
