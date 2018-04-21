use mem::{Address, MemDevice, RNG_LCD_BGDD1, RNG_LCD_BGDD2, Ram, RNG_CHAR_DAT};

const REG_LCDC: Address = Address(0xFF40);
const REG_STAT: Address = Address(0xFF41);
const REG_LY: Address = Address(0xFF44);
const REG_LYC: Address = Address(0xFF45);
const REG_OBP0: Address = Address(0xFF48);
const REG_OBP1: Address = Address(0xFF49);
const REG_DMA: Address = Address(0xFF46);
const REG_WY: Address = Address(0xFF4A);
const REG_WX: Address = Address(0xFF4B);

pub struct Lcd {
    lcdc: u8,
    stat: u8,
    obp0: u8,
    obp1: u8,
    dma: u8,
    wx: u8,
    wy: u8,
    lyc: u8,
    cdata: Ram,
    bgdd1: Ram,
    bgdd2: Ram,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcdc: 0x83,
            stat: 0,
            obp0: 0,
            obp1: 0,
            dma: 0,
            wx: 0,
            wy: 0,
            lyc: 0,
            cdata: Ram::new(RNG_CHAR_DAT.len()),
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
        } else if a.in_(RNG_CHAR_DAT) {
            self.cdata.read(a - RNG_CHAR_DAT.0)
        } else {
            match a {
                REG_LY => {
                    println!("Warning: Reading from stub register LY");
                    Ok(145)
                }
                REG_LYC => {
                    println!("Warning: Reading from stub register LYC");
                    Ok(self.lyc)
                }
                REG_STAT => {
                    println!("Warning: Reading from stub register STAT");
                    Ok(self.stat)
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
                REG_DMA => {
                    println!("DMA register is write-only");
                    Err(())
                }
                REG_WX => {
                    println!("Warning: Reading from stub register WX");
                    Ok(self.wx)
                }
                REG_WY => {
                    println!("Warning: Reading from stub register WY");
                    Ok(self.wy)
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
        } else if a.in_(RNG_CHAR_DAT) {
            self.cdata.write(a - RNG_CHAR_DAT.0, v)
        } else {
            match a {
                REG_LY => {
                    println!("LY is a read only register!");
                    Err(())
                }
                REG_LYC => {
                    println!("Warning: Writing to stub register LYC");
                    self.lyc = v;
                    Ok(())
                }
                REG_LCDC => {
                    println!("Warning: Writing to stub register LCDC");
                    self.lcdc = v;
                    Ok(())
                }
                REG_STAT => {
                    println!("Warning: Writing to stub register STAT");
                    self.stat = v;
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
                REG_DMA => {
                    println!("Warning: Writing to stub register DMA");
                    self.dma = v;
                    Ok(())
                }
                REG_WX => {
                    println!("Warning: Writing to stub register WX");
                    self.wx = v;
                    Ok(())
                }
                REG_WY => {
                    println!("Warning: Writing to stub register WY");
                    self.wy = v;
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
