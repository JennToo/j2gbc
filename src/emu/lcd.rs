use std::cmp::min;
use std::num::Wrapping;

use super::cpu::{Interrupt, CLOCK_RATE};
use super::mem::{Address, MemDevice, RNG_LCD_BGDD1, RNG_LCD_BGDD2, Ram, RNG_CHAR_DAT, RNG_LCD_OAM};

const REG_LCDC: Address = Address(0xFF40);
const REG_STAT: Address = Address(0xFF41);
const REG_SCY: Address = Address(0xFF42);
const REG_SCX: Address = Address(0xFF43);
const REG_LY: Address = Address(0xFF44);
const REG_LYC: Address = Address(0xFF45);
const REG_BGP: Address = Address(0xFF47);
const REG_OBP0: Address = Address(0xFF48);
const REG_OBP1: Address = Address(0xFF49);
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
const BG_CHARS_PER_ROW: u8 = 32;
const PIXEL_PER_CHAR: u8 = 8;
const CHARS_PER_BANK: u8 = 255;

const LYC_MATCH_INT_FLAG: u8 = 0b0100_0000;
//const MODE_10_INT_FLAG: u8 = 0b0010_0000;
//const MODE_01_INT_FLAG: u8 = 0b0001_0000;
const MODE_00_INT_FLAG: u8 = 0b0000_1000;

const LYC_MATCH_FLAG: u8 = 0b0000_0100;
const BG_ENABLED_FLAG: u8 = 0b0000_0001;
const OAM_ENABLED_FLAG: u8 = 0b0000_0010;
const OAM_TALL_FLAG: u8 = 0b0000_0100;
const BGD_CHAR_DAT_FLAG: u8 = 0b0001_0000;
const BGD_CODE_DAT_FLAG: u8 = 0b0000_1000;
const OBJ_PAL_FLAG: u8 = 0b0001_0000;
const OBJ_XFLIP_FLAG: u8 = 0b0010_0000;
const OBJ_YFLIP_FLAG: u8 = 0b0100_0000;
const OBJ_PRI_FLAG: u8 = 0b1000_0000;

pub type Framebuffer = [FrameRow; SCREEN_SIZE.1];
type FrameRow = [Pixel; SCREEN_SIZE.0];
pub type BgBuffer = [BgRow; 256];
type BgRow = [Pixel; 256];
type CharRow = [u8; 8];

pub struct Lcd {
    lcdc: u8,
    stat: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wx: u8,
    wy: u8,
    sx: u8,
    sy: u8,
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

struct Obj {
    x: u8,
    y: u8,
    char_: u8,
    flags: u8,
}

impl Obj {
    fn high_palette(&self) -> bool {
        self.flags & OBJ_PAL_FLAG != 0
    }

    fn xflip(&self) -> bool {
        self.flags & OBJ_XFLIP_FLAG != 0
    }

    fn yflip(&self) -> bool {
        self.flags & OBJ_YFLIP_FLAG != 0
    }

    fn priority(&self) -> bool {
        self.flags & OBJ_PRI_FLAG != 0
    }
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcdc: 0x83,
            stat: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wx: 0,
            wy: 0,
            sx: 0,
            sy: 0,
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

            if (self.ly == self.lyc && self.is_lyc_int_enabled()) || self.is_hblank_int_enabled() {
                // TODO: This should actually happen at the start of the line I think
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else if cycle >= self.next_vblank_cycle {
            self.do_vblank(cycle);
            Some(Interrupt::VBlank)
        } else {
            None
        }
    }

    pub fn do_hblank(&mut self, cycle: u64) {
        // TODO: This is just a debug hack to display char data on the screen
        if self.ly < SCREEN_SIZE.1 as u8 {
            self.render_background_row();
            self.render_oam_row();
        }

        self.ly += 1;
        self.update_lyc();
        self.next_hblank_cycle = LINE_CYCLE_TIME + cycle;
    }

    pub fn do_vblank(&mut self, cycle: u64) {
        self.swap();
        self.ly = 0;
        self.update_lyc();
        self.next_vblank_cycle = 154 * LINE_CYCLE_TIME + cycle;
    }

    fn update_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= LYC_MATCH_FLAG;
        } else {
            self.stat &= !LYC_MATCH_FLAG;
        }
    }

    fn render_background_row(&mut self) {
        if !self.is_bg_enabled() {
            return;
        }

        let screen_y = self.ly;
        let translated_y = Wrapping(screen_y) + Wrapping(self.sy); // Implicit % 256
        for screen_x in 0..SCREEN_SIZE.0 {
            let translated_x = Wrapping(screen_x as u8) + Wrapping(self.sx); // Implicit % 256

            let char_y_offset = Wrapping(u16::from(translated_y.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR))
                * Wrapping(u16::from(BG_CHARS_PER_ROW));
            let char_offset = Wrapping(u16::from(translated_x.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR)) + char_y_offset;
            let char_addr = self.get_bg_code_dat_start() + Address(char_offset.0);
            let char_ = self.read(char_addr).unwrap();
            let (base_addr, signed) = self.get_bg_char_addr_start();
            let char_row =
                self.read_char_row_at(char_, (translated_y % Wrapping(8)).0, base_addr, signed);

            let color_index = char_row[(translated_x % Wrapping(8)).0 as usize];
            let corrected_index = palette_convert(color_index, self.bgp) as usize;
            self.get_back_framebuffer()[screen_y as usize][screen_x as usize] =
                COLORS[corrected_index];
        }
    }

    fn read_char_row_at(&self, char_: u8, row: u8, base_address: Address, signed: bool) -> CharRow {
        let char_address = if signed {
            let chari = i32::from(char_ as i8);
            let a = i32::from(base_address.0) + i32::from(BYTES_PER_CHAR) * chari;
            Address(a as u16)
        } else {
            base_address + Address(BYTES_PER_CHAR * u16::from(char_))
        };
        let row_address = char_address + Address(BYTES_PER_CHAR_ROW * u16::from(row));
        let b1 = self.read(row_address).unwrap();
        let b2 = self.read(row_address + Address(1)).unwrap();
        let mut row = [0; 8];
        for i in 0..8 {
            row[i] = read_bit(b1, (7 - i) as u8) | (read_bit(b2, (7 - i) as u8) << 1);
        }
        row
    }

    fn is_bg_enabled(&self) -> bool {
        self.lcdc & BG_ENABLED_FLAG != 0
    }

    fn is_oam_enabled(&self) -> bool {
        self.lcdc & OAM_ENABLED_FLAG != 0
    }

    fn is_lyc_int_enabled(&self) -> bool {
        self.stat & LYC_MATCH_INT_FLAG != 0
    }

    fn is_hblank_int_enabled(&self) -> bool {
        self.stat & MODE_00_INT_FLAG != 0
    }

    fn get_bg_char_addr_start(&self) -> (Address, bool) {
        if self.lcdc & BGD_CHAR_DAT_FLAG == 0 {
            (Address(0x9000), true)
        } else {
            (Address(0x8000), false)
        }
    }

    fn get_bg_code_dat_start(&self) -> Address {
        if self.lcdc & BGD_CODE_DAT_FLAG == 0 {
            Address(0x9800)
        } else {
            Address(0x9C00)
        }
    }

    pub fn render_char_dat(&self, high: bool) -> Box<Framebuffer> {
        let mut fb = Box::new([[Pixel(255, 255, 0, 255); SCREEN_SIZE.0]; SCREEN_SIZE.1]);
        let (start_addr, signed) = if high {
            (Address(0x9000), false)
        } else {
            (Address(0x8000), false)
        };

        const CHARS_PER_ROW: u8 = (SCREEN_SIZE.0 as u8 / PIXEL_PER_CHAR);
        for char_ in 0..CHARS_PER_BANK {
            let base_x = (char_ % CHARS_PER_ROW) * 8;
            let base_y = (char_ / CHARS_PER_ROW) * 8;

            for y in 0..PIXEL_PER_CHAR {
                let row = self.read_char_row_at(char_, y, start_addr, signed);
                for x in 0..PIXEL_PER_CHAR {
                    let color_index = row[x as usize];
                    let corrected_index = palette_convert(color_index, self.bgp) as usize;
                    fb[(base_y + y) as usize][(base_x + x) as usize] = COLORS[corrected_index];
                }
            }
        }

        fb
    }

    pub fn render_background(&self, first: bool) -> Box<BgBuffer> {
        let mut fb = Box::new([[Pixel(255, 255, 0, 255); 256]; 256]);

        let code_start = if first {
            Address(0x9800)
        } else {
            Address(0x9C00)
        };
        let (start_addr, signed) = self.get_bg_char_addr_start();

        for char_y in 0..BG_CHARS_PER_ROW {
            for char_x in 0..BG_CHARS_PER_ROW {
                let char_offset =
                    Address(u16::from(char_x) + u16::from(char_y) * u16::from(BG_CHARS_PER_ROW));
                let char_ = self.read(code_start + char_offset).unwrap();

                for y in 0..PIXEL_PER_CHAR {
                    let row = self.read_char_row_at(char_, y, start_addr, signed);
                    for x in 0..PIXEL_PER_CHAR {
                        let color_index = row[x as usize];
                        let corrected_index = palette_convert(color_index, self.bgp) as usize;
                        fb[(char_y * PIXEL_PER_CHAR + y) as usize]
                            [(char_x * PIXEL_PER_CHAR + x) as usize] = COLORS[corrected_index];
                    }
                }
            }
        }

        fb
    }

    fn get_obj(&self, index: u8) -> Obj {
        let a = RNG_LCD_OAM.0 + Address(u16::from(index) * 4);

        Obj {
            y: self.read(a).unwrap(),
            x: self.read(a + Address(1)).unwrap(),
            char_: self.read(a + Address(2)).unwrap(),
            flags: self.read(a + Address(3)).unwrap(),
        }
    }

    fn render_oam_row(&mut self) {
        if !self.is_oam_enabled() {
            return;
        }

        for i in 0..40 {
            let obj = self.get_obj(i);

            let (char_, hi_y) = if self.lcdc & OAM_TALL_FLAG != 0 {
                (obj.char_ & 0b1111_1110, 16)
            } else {
                (obj.char_, 8)
            };

            for y in 0..hi_y {
                let full_y = y as isize + obj.y as isize - 16;
                if full_y > SCREEN_SIZE.1 as isize || full_y < 0 || full_y != self.ly as isize {
                    continue;
                }

                let index_y = if obj.yflip() { hi_y - 1 - y } else { y };
                let row = self.read_char_row_at(char_, index_y, Address(0x8000), false);
                for x in 0..8 {
                    let full_x = x as isize + obj.x as isize - 8;

                    if full_x >= SCREEN_SIZE.0 as isize || full_x < 0 {
                        continue;
                    }

                    let index_x = if obj.xflip() { 7 - x } else { x };
                    let color_index = row[index_x as usize];
                    if color_index == 0 {
                        // 0 is always transparent
                        continue;
                    }
                    let pal = if obj.high_palette() {
                        self.obp1
                    } else {
                        self.obp0
                    };
                    let corrected_index = palette_convert(color_index, pal) as usize;
                    let color = COLORS[corrected_index];

                    if !obj.priority()
                        || self.get_back_framebuffer()[full_y as usize][full_x as usize]
                            == COLOR_WHITE
                    {
                        self.get_back_framebuffer()[full_y as usize][full_x as usize] = color;
                    }
                }
            }
        }
    }
}

fn read_bit(value: u8, bit: u8) -> u8 {
    let mask = 1 << bit;
    (value & mask) >> bit
}

fn palette_convert(v: u8, p: u8) -> u8 {
    (p >> (v * 2)) & 0b11
}

#[test]
fn test_palette_convert() {
    assert_eq!(0b11, palette_convert(0, 0b11));
    assert_eq!(0b00, palette_convert(3, 0b00111111));
    assert_eq!(0b01, palette_convert(1, 0b0100));
}

#[test]
fn test_read_bit() {
    assert_eq!(read_bit(0b0000_0100, 2), 1);
    assert_eq!(read_bit(0b0000_0100, 3), 0);
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
                REG_SCX => Ok(self.sx),
                REG_SCY => Ok(self.sy),
                REG_BGP => {
                    error!("Error: BGP is a write-only register");
                    Err(())
                }
                _ => {
                    error!("Unimplemented LCD register {:?}", a);
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
                    error!("LY is a read only register!");
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
                REG_WX => {
                    self.wx = v;
                    Ok(())
                }
                REG_WY => {
                    self.wy = v;
                    Ok(())
                }
                REG_SCX => {
                    self.sx = v;
                    Ok(())
                }
                REG_SCY => {
                    self.sy = v;
                    Ok(())
                }
                _ => {
                    error!("Unimplemented LCD register {:?}", a);
                    Err(())
                }
            }
        }
    }
}

impl Default for Lcd {
    fn default() -> Lcd {
        Lcd::new()
    }
}
