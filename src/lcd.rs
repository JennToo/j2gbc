use mem::{Address, MemDevice};

const REG_LCDC: Address = Address(0xFF40);
const REG_LY: Address = Address(0xFF44);

pub struct Lcd {
    lcdc: u8,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd { lcdc: 0x83 }
    }
}

impl MemDevice for Lcd {
    fn read(&self, a: Address) -> Result<u8, ()> {
        match a {
            REG_LY => {
                println!("Warning: Reading from stub register LY");
                Ok(145)
            }
            REG_LCDC => {
                println!("Warning: Reading from stub register LCDC");
                Ok(self.lcdc)
            }
            _ => {
                println!("Unimplemented LCD register {:?}", a);
                Err(())
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        match a {
            REG_LY => {
                println!("LY is a read only register!");
                Err(())
            }
            REG_LCDC => {
                println!("Warning: Writing to stub register LCDC");
                self.lcdc = v;
                Ok(())
            }
            _ =>  {
                println!("Unimplemented LCD register {:?}", a);
                Err(())
            }
        }
    }
}
