use crate::mem::Address;

// TODO: Eventually move the interrupt handler to in here

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Interrupt {
    VBlank,
    LCDC,
    Timer,
    Controller,
}

const INT_VBLANK: u8 = 0b0000_0001;
const INT_LCDC: u8 = 0b0000_0010;
const INT_TIMER: u8 = 0b0000_0100;
const INT_CONTROLLER: u8 = 0b0001_0000;

const PRIORITY: [u8; 4] = [INT_VBLANK, INT_LCDC, INT_TIMER, INT_CONTROLLER];

impl Interrupt {
    pub fn bits(self) -> u8 {
        match self {
            Interrupt::VBlank => INT_VBLANK,
            Interrupt::LCDC => INT_LCDC,
            Interrupt::Timer => INT_TIMER,
            Interrupt::Controller => INT_CONTROLLER,
        }
    }

    pub fn table_address(self) -> Address {
        match self {
            Interrupt::VBlank => Address(0x0040),
            Interrupt::LCDC => Address(0x0048),
            Interrupt::Timer => Address(0x0050),
            Interrupt::Controller => Address(0x0060),
        }
    }

    pub fn from_bits(bit: u8) -> Interrupt {
        match bit {
            INT_VBLANK => Interrupt::VBlank,
            INT_LCDC => Interrupt::LCDC,
            INT_TIMER => Interrupt::Timer,
            INT_CONTROLLER => Interrupt::Controller,
            _ => panic!("Unsupported interrupt {}", bit),
        }
    }

    pub fn int_to_run(if_: u8, ie: u8) -> (Option<Interrupt>, u8) {
        for attempt in &PRIORITY {
            if ie & attempt != 0 && if_ & attempt != 0 {
                return (Some(Self::from_bits(*attempt)), if_ & (!attempt));
            }
        }

        (None, if_)
    }
}
