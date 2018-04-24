use emu::mem::Address;

// TODO: Eventually move the interrupt handler to in here

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interrupt {
    VBlank,
    LCDC,
    Timer,
    Controller,
}

impl Interrupt {
    pub fn is_enabled(self, reg: u8) -> bool {
        match self {
            Interrupt::VBlank => (reg & 0b0000_0001) != 0,
            Interrupt::LCDC => (reg & 0b0000_0010) != 0,
            Interrupt::Timer => (reg & 0b0000_0100) != 0,
            Interrupt::Controller => (reg & 0b0001_0000) != 0,
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
}
