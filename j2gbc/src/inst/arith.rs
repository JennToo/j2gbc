use std::fmt;
use std::fmt::Display;

use cpu::{Operand, Register16};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    Inc(Operand),
    Dec(Operand),
    Add(Operand),
    Adc(Operand),
    Sub(Operand),
    Sbc(Operand),

    Daa,

    IncR16(Register16),
    DecR16(Register16),

    AddRR16(Register16, Register16),

    AddSP(i8),
}

impl Arith {
    pub fn cycles(self) -> u8 {
        match self {
            Arith::Daa => 4,

            Arith::Sub(Operand::Register(_))
            | Arith::Add(Operand::Register(_))
            | Arith::Sbc(Operand::Register(_))
            | Arith::Adc(Operand::Register(_))
            | Arith::Dec(Operand::Register(_))
            | Arith::Inc(Operand::Register(_)) => 4,

            Arith::Sub(Operand::IndirectRegister(_))
            | Arith::Add(Operand::IndirectRegister(_))
            | Arith::Sbc(Operand::IndirectRegister(_))
            | Arith::Adc(Operand::IndirectRegister(_)) => 8,

            Arith::Dec(Operand::IndirectRegister(_)) | Arith::Inc(Operand::IndirectRegister(_)) => {
                12
            }

            Arith::Sub(Operand::Immediate(_))
            | Arith::Add(Operand::Immediate(_))
            | Arith::Sbc(Operand::Immediate(_))
            | Arith::Adc(Operand::Immediate(_)) => 8,

            Arith::DecR16(_) | Arith::IncR16(_) => 8,
            Arith::AddRR16(_, _) => 16,

            Arith::AddSP(_) => 16,

            Arith::Sub(_)
            | Arith::Add(_)
            | Arith::Sbc(_)
            | Arith::Adc(_)
            | Arith::Dec(_)
            | Arith::Inc(_) => unimplemented!(),
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arith::Daa => write!(f, "daa"),
            Arith::Add(o) => write!(f, "add {}", o),
            Arith::Adc(o) => write!(f, "adc {}", o),
            Arith::Sub(o) => write!(f, "sub {}", o),
            Arith::Sbc(o) => write!(f, "sbc {}", o),
            Arith::Dec(o) => write!(f, "dec {}", o),
            Arith::Inc(o) => write!(f, "inc {}", o),
            Arith::DecR16(r) => write!(f, "dec {}", r),
            Arith::IncR16(r) => write!(f, "inc {}", r),
            Arith::AddRR16(r1, r2) => write!(f, "add {},{}", r1, r2),
            Arith::AddSP(v) => write!(f, "add sp,{}", v),
        }
    }
}
