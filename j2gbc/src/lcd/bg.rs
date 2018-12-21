use crate::system::SystemMode;

const BG_YFLIP_FLAG: u8 = 0b0100_0000;
const BG_XFLIP_FLAG: u8 = 0b0010_0000;
const BG_PRI_FLAG: u8 = 0b1000_0000;
const BG_BANK_SEL_MASK: u8 = 0b0000_1000;
const BG_PAL_SEL_MASK: u8 = 0b0000_0111;

#[derive(Copy, Clone)]
pub struct BgFlags {
    flags: u8,
}

impl BgFlags {
    pub fn new(flags: u8, mode: SystemMode) -> BgFlags {
        BgFlags {
            flags: match mode {
                SystemMode::CGB => flags,
                SystemMode::DMG => 0,
            },
        }
    }

    pub fn xflip(self) -> bool {
        self.flags & BG_XFLIP_FLAG != 0
    }

    pub fn yflip(self) -> bool {
        self.flags & BG_YFLIP_FLAG != 0
    }

    pub fn priority(self) -> bool {
        self.flags & BG_PRI_FLAG != 0
    }

    pub fn cgb_pallete(self) -> u8 {
        self.flags & BG_PAL_SEL_MASK
    }

    pub fn bank(self) -> u8 {
        (self.flags & BG_BANK_SEL_MASK) >> 3
    }
}
