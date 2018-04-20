use mem::{Address, MemDevice, RNG_LCD_BGDD1, RNG_LCD_BGDD2, Ram};

const REG_LCDC: Address = Address(0xFF40);
const REG_LY: Address = Address(0xFF44);
const REG_OBP0: Address = Address(0xFF48);
const REG_OBP1: Address = Address(0xFF49);

pub struct Lcd {
    lcdc: u8,
    obp0: u8,
    obp1: u8,
    bgdd1: Ram,
    bgdd2: Ram,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcdc: 0x83,
            obp0: 0,
            obp1: 0,
            bgdd1: Ram::new(RNG_LCD_BGDD1.len()),
            bgdd2: Ram::new(RNG_LCD_BGDD2.len()),
        }
    }
}

impl MemDevice for Lcd {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_LCD_BGDD1) {
            self.bgdd1.read(a - RNG_LCD_BGDD1.0)
        } else if a.in_(RNG_LCD_BGDD2) {
            self.bgdd2.read(a - RNG_LCD_BGDD2.0)
        } else {
            match a {
                REG_LY => {
                    println!("Warning: Reading from stub register LY");
                    Ok(145)
                }
                REG_LCDC => {
                    println!("Warning: Reading from stub register LCDC");
                    Ok(self.lcdc)
                }
                REG_OBP0 => {
                    println!("Warning: Reading from stub register OBP0");
                    Ok(self.obp0)
                }
                REG_OBP1 => {
                    println!("Warning: Reading from stub register OBP1");
                    Ok(self.obp1)
                }
                _ => {
                    println!("Unimplemented LCD register {:?}", a);
                    Err(())
                }
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_LCD_BGDD1) {
            self.bgdd1.write(a - RNG_LCD_BGDD1.0, v)
        } else if a.in_(RNG_LCD_BGDD2) {
            self.bgdd2.write(a - RNG_LCD_BGDD2.0, v)
        } else {
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
                REG_OBP0 => {
                    println!("Warning: Writing to stub register OBP0");
                    self.obp0 = v;
                    Ok(())
                }
                REG_OBP1 => {
                    println!("Warning: Writing to stub register OBP1");
                    self.obp1 = v;
                    Ok(())
                }
                _ => {
                    println!("Unimplemented LCD register {:?}", a);
                    Err(())
                }
            }
        }
    }
}
