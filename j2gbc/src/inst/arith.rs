use std::fmt;
use std::fmt::Display;

use crate::cpu::{Operand, Register16};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    Increment(Operand),
    Decrement(Operand),
    Add(Operand),
    AddWithCarry(Operand),
    Subtract(Operand),
    SubtractWithCarry(Operand),

    DecimalAdjustAccumulator,

    IncrementRegister16(Register16),
    DecrementRegister16(Register16),

    AddRegisterRegister16(Register16, Register16),

    AddSP(i8),
}

impl Arith {
    pub fn cycles(self) -> u8 {
        match self {
            Arith::DecimalAdjustAccumulator => 4,

            Arith::Subtract(Operand::Register(_))
            | Arith::Add(Operand::Register(_))
            | Arith::SubtractWithCarry(Operand::Register(_))
            | Arith::AddWithCarry(Operand::Register(_))
            | Arith::Decrement(Operand::Register(_))
            | Arith::Increment(Operand::Register(_)) => 4,

            Arith::Subtract(Operand::IndirectRegister(_))
            | Arith::Add(Operand::IndirectRegister(_))
            | Arith::SubtractWithCarry(Operand::IndirectRegister(_))
            | Arith::AddWithCarry(Operand::IndirectRegister(_)) => 8,

            Arith::Decrement(Operand::IndirectRegister(_))
            | Arith::Increment(Operand::IndirectRegister(_)) => 12,

            Arith::Subtract(Operand::Immediate(_))
            | Arith::Add(Operand::Immediate(_))
            | Arith::SubtractWithCarry(Operand::Immediate(_))
            | Arith::AddWithCarry(Operand::Immediate(_)) => 8,

            Arith::DecrementRegister16(_) | Arith::IncrementRegister16(_) => 8,
            Arith::AddRegisterRegister16(_, _) => 16,

            Arith::AddSP(_) => 16,

            Arith::Subtract(_)
            | Arith::Add(_)
            | Arith::SubtractWithCarry(_)
            | Arith::AddWithCarry(_)
            | Arith::Decrement(_)
            | Arith::Increment(_) => unimplemented!(),
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arith::DecimalAdjustAccumulator => write!(f, "daa"),
            Arith::Add(o) => write!(f, "add {}", o),
            Arith::AddWithCarry(o) => write!(f, "adc {}", o),
            Arith::Subtract(o) => write!(f, "sub {}", o),
            Arith::SubtractWithCarry(o) => write!(f, "sbc {}", o),
            Arith::Decrement(o) => write!(f, "dec {}", o),
            Arith::Increment(o) => write!(f, "inc {}", o),
            Arith::DecrementRegister16(r) => write!(f, "dec {}", r),
            Arith::IncrementRegister16(r) => write!(f, "inc {}", r),
            Arith::AddRegisterRegister16(r1, r2) => write!(f, "add {},{}", r1, r2),
            Arith::AddSP(v) => write!(f, "add sp,{}", v),
        }
    }
}
