use mem::{MemDevice, Address};

const REG_LCDC: Address = Address(0xFF40);
const REG_LY: Address = Address(0xFF44);

pub struct Lcd {
    lcdc: u8,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcdc: 0x83,
        }
    }
}

impl MemDevice for Lcd {
    fn read(&self, a: Address) -> u8 {
        match a {
            REG_LY => {
                println!("Warning: Reading from stub register LY");
                145
            }
            REG_LCDC => {
                println!("Warning: Reading from stub register LCDC");
                self.lcdc
            }
            _ => {
                panic!("Unimplemented LCD register {:?}", a);
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) {
        match a {
            REG_LY => panic!("LY is a read only register!"),
            REG_LCDC => {
                println!("Warning: Writing to stub register LCDC");
                self.lcdc = v;
            }
            _ => panic!("Unimplemented LCD register {:?}", a),
        }
    }
}
