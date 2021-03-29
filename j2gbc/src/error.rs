use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum ExecutionError {
    BusError,
    StopWithoutSpeed,
    InvalidInstruction,
    ProtectionFault,
    // It's not clear if these should really be treated as "errors"
    Breakpoint,
    MmuException,
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::BusError => write!(f, "Bus Error"),
            ExecutionError::MmuException => write!(f, "MMU Exception (watchpoint)"),
            ExecutionError::InvalidInstruction => {
                write!(f, "Attempt to decode invalid instruction")
            }
            ExecutionError::ProtectionFault => write!(f, "RAM protection fault"),
            ExecutionError::StopWithoutSpeed => write!(f, "STOP instruction without speed prep"),
            ExecutionError::Breakpoint => write!(f, "Breakpoint"),
        }
    }
}

impl Error for ExecutionError {}
