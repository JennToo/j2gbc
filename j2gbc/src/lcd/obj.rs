use crate::system::SystemMode;

const OBJ_PAL_FLAG: u8 = 0b0001_0000;
const OBJ_XFLIP_FLAG: u8 = 0b0010_0000;
const OBJ_YFLIP_FLAG: u8 = 0b0100_0000;
const OBJ_PRI_FLAG: u8 = 0b1000_0000;
const OBJ_BANK_FLAG: u8 = 0b0000_1000;
const OBJ_PAL_SELECT: u8 = 0b0000_0111;

#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub struct Obj {
    pub x: u8,
    pub y: u8,
    pub char_: u8,
    flags: u8,
}

impl Obj {
    pub fn new(x: u8, y: u8, char_: u8, flags: u8, mode: SystemMode) -> Obj {
        Obj {
            x,
            y,
            char_,
            flags: match mode {
                SystemMode::CGB => flags & !OBJ_PAL_FLAG,
                SystemMode::DMG => flags & !OBJ_BANK_FLAG & !OBJ_PAL_SELECT,
            },
        }
    }

    pub fn high_palette(self) -> bool {
        self.flags & OBJ_PAL_FLAG != 0
    }

    pub fn xflip(self) -> bool {
        self.flags & OBJ_XFLIP_FLAG != 0
    }

    pub fn yflip(self) -> bool {
        self.flags & OBJ_YFLIP_FLAG != 0
    }

    pub fn priority(self) -> bool {
        self.flags & OBJ_PRI_FLAG != 0
    }

    pub fn cgb_palette(self) -> u8 {
        self.flags & OBJ_PAL_SELECT
    }

    pub fn bank(self) -> u8 {
        (self.flags & OBJ_BANK_FLAG) >> 3
    }
}
