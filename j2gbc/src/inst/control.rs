use std::fmt;
use std::fmt::Display;

use crate::cpu::ConditionCode;
use crate::mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Control {
    JumpRelativeConditional(i8, ConditionCode),
    JumpRelative(i8),
    Return,
    InterruptReturn,
    ReturnConditional(ConditionCode),
    JumpIndirect,
    Jump(Address),
    JumpConditional(Address, ConditionCode),
    Call(Address),
    CallConditional(Address, ConditionCode),
    Reset(Address),
}

impl Control {
    pub fn cycles(self) -> u8 {
        match self {
            Control::Reset(_) => 16,
            Control::JumpIndirect => 4,
            Control::JumpConditional(_, _) | Control::Jump(_) => 16,
            Control::Call(_) | Control::CallConditional(_, _) => 24,
            // TODO: This is actually variable
            Control::JumpRelativeConditional(_, _) | Control::JumpRelative(_) => 12,
            Control::Return | Control::InterruptReturn => 16,
            Control::ReturnConditional(_) => 8,
        }
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::JumpRelativeConditional(i, cond) => write!(f, "jr{} {}", cond, i),
            Control::JumpRelative(i) => write!(f, "jr {}", i),
            Control::Return => write!(f, "ret"),
            Control::InterruptReturn => write!(f, "reti"),
            Control::ReturnConditional(cond) => write!(f, "ret {}", cond),
            Control::JumpIndirect => write!(f, "jmp [hl]"),
            Control::Jump(a) => write!(f, "jmp {}", a),
            Control::JumpConditional(a, cond) => write!(f, "jmp{} {}", cond, a),
            Control::Call(a) => write!(f, "call {}", a),
            Control::CallConditional(a, cond) => write!(f, "call{} {}", cond, a),
            Control::Reset(a) => write!(f, "rst {}", a),
        }
    }
}
