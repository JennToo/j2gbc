use std::cmp::min;

use super::mem::{Address, MemDevice, RNG_LCD_BGDD1, RNG_LCD_BGDD2, Ram, RNG_CHAR_DAT, RNG_LCD_OAM};
use super::cpu::{Interrupt, CLOCK_RATE};

const REG_LCDC: Address = Address(0xFF40);
const REG_STAT: Address = Address(0xFF41);
const REG_LY: Address = Address(0xFF44);
const REG_LYC: Address = Address(0xFF45);
const REG_BGP: Address = Address(0xFF47);
const REG_OBP0: Address = Address(0xFF48);
const REG_OBP1: Address = Address(0xFF49);
const REG_DMA: Address = Address(0xFF46);
const REG_WY: Address = Address(0xFF4A);
const REG_WX: Address = Address(0xFF4B);

pub const SCREEN_SIZE: (usize, usize) = (160, 144);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Pixel(pub u8, pub u8, pub u8, pub u8);

const COLOR_WHITE: Pixel = Pixel(234, 255, 186, 255);
const COLOR_LIGHT_GRAY: Pixel = Pixel(150, 187, 146, 255);
const COLOR_DARK_GRAY: Pixel = Pixel(68, 106, 81, 255);
const COLOR_BLACK: Pixel = Pixel(0, 14, 2, 255);
const COLORS: [Pixel; 4] = [COLOR_WHITE, COLOR_LIGHT_GRAY, COLOR_DARK_GRAY, COLOR_BLACK];

const LINE_CYCLE_TIME: u64 = CLOCK_RATE * 180_700 / 1_000_000_000;
const BYTES_PER_CHAR: u16 = 16;
const BYTES_PER_CHAR_ROW: u16 = 2;

pub type Framebuffer = [[Pixel; SCREEN_SIZE.0]; SCREEN_SIZE.1];
type CharRow = [usize; 8];

pub struct Lcd {
    lcdc: u8,
    stat: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    dma: u8,
    wx: u8,
    wy: u8,
    lyc: u8,
    ly: u8,
    cdata: Ram,
    bgdd1: Ram,
    bgdd2: Ram,
    oam: Ram,

    fbs: [Framebuffer; 2],
    fbi: usize,

    next_hblank_cycle: u64,
    next_vblank_cycle: u64,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcdc: 0x83,
            stat: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            dma: 0,
            wx: 0,
            wy: 0,
            lyc: 0,
            cdata: Ram::new(RNG_CHAR_DAT.len()),
            bgdd1: Ram::new(RNG_LCD_BGDD1.len()),
            bgdd2: Ram::new(RNG_LCD_BGDD2.len()),
            oam: Ram::new(RNG_LCD_OAM.len()),
            fbs: [[[COLOR_WHITE; SCREEN_SIZE.0]; SCREEN_SIZE.1]; 2],
            fbi: 0,

            next_hblank_cycle: 0,
            next_vblank_cycle: 0,
            ly: 0,
        }
    }

    pub fn get_framebuffer(&self) -> &Framebuffer {
        &self.fbs[self.fbi]
    }

    fn get_back_framebuffer(&mut self) -> &mut Framebuffer {
        if self.fbi == 0 {
            &mut self.fbs[1]
        } else {
            &mut self.fbs[0]
        }
    }

    fn swap(&mut self) {
        if self.fbi == 0 {
            self.fbi = 1;
        } else {
            self.fbi = 0;
        }
    }

    pub fn get_next_event_cycle(&self) -> u64 {
        min(self.next_hblank_cycle, self.next_vblank_cycle)
    }

    pub fn pump_cycle(&mut self, cycle: u64) -> Option<Interrupt> {
        if cycle >= self.next_hblank_cycle {
            self.do_hblank(cycle);
            Some(Interrupt::HBlank)
        } else if cycle >= self.next_vblank_cycle {
            self.do_vblank(cycle);
            Some(Interrupt::VBlank)
        } else {
            None
        }
    }

    pub fn do_hblank(&mut self, cycle: u64) {
        // TODO: This is just a debug hack to display char data on the screen
        const CHARS_PER_ROW: u8 = (SCREEN_SIZE.0 as u8 / 8);
        let current_char_start = (self.ly as u16 * CHARS_PER_ROW as u16 / 8) as u8;
        if current_char_start < (255 - CHARS_PER_ROW) && self.ly < SCREEN_SIZE.1 as u8 {
            for i in 0..CHARS_PER_ROW {
                let char_ = current_char_start + i;
                let row = self.read_char_row_at(char_, self.ly % 8, true);
                for j in 0..8 {
                    let x = (i + j) as usize;
                    let y = self.ly as usize;
                    let color_index = row[j as usize] as usize;
                    self.get_back_framebuffer()[y][x] = COLORS[color_index];
                }
            }
        }

        self.ly += 1;
        self.next_hblank_cycle = LINE_CYCLE_TIME + cycle;
    }

    pub fn do_vblank(&mut self, cycle: u64) {
        self.swap();
        self.ly = 0;
        self.next_vblank_cycle = 154 * LINE_CYCLE_TIME + cycle;
    }

    fn read_char_row_at(&self, char_: u8, row: u8, high: bool) -> CharRow {
        let base_address = if high {
            Address(0x9000)
        } else {
            Address(0x8000)
        };
        let char_address = base_address + Address(BYTES_PER_CHAR * (char_ as u16));
        let row_address = char_address + Address(BYTES_PER_CHAR_ROW * (row as u16));
        let b1 = self.read(row_address).unwrap();
        let b2 = self.read(row_address + Address(1)).unwrap();
        let mut row = [0; 8];
        for i in 0..8 {
            row[i] = (read_bit(b1, i as u8) | (read_bit(b2, i as u8) << 1)) as usize;
        }
        row
    }
}

fn read_bit(value: u8, bit: u8) -> u8 {
    let mask = 1 << bit;
    (value & mask) >> bit
}

impl MemDevice for Lcd {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_LCD_BGDD1) {
            self.bgdd1.read(a - RNG_LCD_BGDD1.0)
        } else if a.in_(RNG_LCD_BGDD2) {
            self.bgdd2.read(a - RNG_LCD_BGDD2.0)
        } else if a.in_(RNG_CHAR_DAT) {
            self.cdata.read(a - RNG_CHAR_DAT.0)
        } else if a.in_(RNG_LCD_OAM) {
            self.oam.read(a - RNG_LCD_OAM.0)
        } else {
            match a {
                REG_LY => Ok(self.ly),
                REG_LYC => Ok(self.lyc),
                REG_STAT => Ok(self.stat),
                REG_LCDC => Ok(self.lcdc),
                REG_OBP0 => Ok(self.obp0),
                REG_OBP1 => Ok(self.obp1),
                REG_WX => Ok(self.wx),
                REG_WY => Ok(self.wy),
                REG_BGP => {
                    println!("Error: BGP is a write-only register");
                    Err(())
                }
                REG_DMA => {
                    println!("DMA register is write-only");
                    Err(())
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
        } else if a.in_(RNG_LCD_OAM) {
            self.oam.write(a - RNG_LCD_OAM.0, v)
        } else {
            match a {
                REG_LY => {
                    println!("LY is a read only register!");
                    Err(())
                }
                REG_LYC => {
                    self.lyc = v;
                    Ok(())
                }
                REG_LCDC => {
                    self.lcdc = v;
                    Ok(())
                }
                REG_STAT => {
                    self.stat = v;
                    Ok(())
                }
                REG_BGP => {
                    self.bgp = v;
                    Ok(())
                }
                REG_OBP0 => {
                    self.obp0 = v;
                    Ok(())
                }
                REG_OBP1 => {
                    self.obp1 = v;
                    Ok(())
                }
                REG_DMA => {
                    self.dma = v;
                    Ok(())
                }
                REG_WX => {
                    self.wx = v;
                    Ok(())
                }
                REG_WY => {
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
