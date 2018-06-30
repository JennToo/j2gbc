use std::cmp::max;
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

const LINE_CYCLE_TIME: u64 = CLOCK_RATE * 108_700 / 1_000_000_000; // Src: Official GB manual
const HBLANK_DURATION: u64 = CLOCK_RATE * 48_600 / 1_000_000_000; // Src: GBCPUMan.pdf
const MODE_10_DURATION: u64 = CLOCK_RATE * 19_000 / 1_000_000_000; // Src: GBCPUMan.pdf
const _VBLANK_DURATION: u64 = LINE_CYCLE_TIME * 10; // Src: Official GB manual
const SCREEN_CYCLE_TIME: u64 = 154 * LINE_CYCLE_TIME;
const BYTES_PER_CHAR: u16 = 16;
const BYTES_PER_ROW: u16 = 2;
const BG_CHARS_PER_ROW: u8 = 32;
const PIXEL_PER_CHAR: u8 = 8;
const CHARS_PER_BANK: u8 = 255;

const LYC_MATCH_INT_FLAG: u8 = 0b0100_0000;
const MODE_10_INT_FLAG: u8 = 0b0010_0000;
const _MODE_01_INT_FLAG: u8 = 0b0001_0000;
const MODE_00_INT_FLAG: u8 = 0b0000_1000;

const MODE_00_MASK: u8 = 0b00;
const MODE_01_MASK: u8 = 0b01;
const MODE_10_MASK: u8 = 0b10;
const _MODE_11_MASK: u8 = 0b11;

const LYC_MATCH_FLAG: u8 = 0b0000_0100;
const BG_ENABLED_FLAG: u8 = 0b0000_0001;
const WINDOW_ENABLED_FLAG: u8 = 0b0010_0000;
const OAM_ENABLED_FLAG: u8 = 0b0000_0010;
const OAM_TALL_FLAG: u8 = 0b0000_0100;
const BGD_CHAR_DAT_FLAG: u8 = 0b0001_0000;
const BGD_CODE_DAT_FLAG: u8 = 0b0000_1000;
const WINDOW_CODE_DAT_FLAG: u8 = 0b0100_0000;
const OBJ_PAL_FLAG: u8 = 0b0001_0000;
const OBJ_XFLIP_FLAG: u8 = 0b0010_0000;
const OBJ_YFLIP_FLAG: u8 = 0b0100_0000;
const OBJ_PRI_FLAG: u8 = 0b1000_0000;

const TILE_COUNT: usize = 384;
const OBJ_COUNT: usize = 40;

pub type Framebuffer = [FrameRow; SCREEN_SIZE.1];
type FrameRow = [Pixel; SCREEN_SIZE.0];
pub type BgBuffer = [BgRow; 256];
type BgRow = [Pixel; 256];
type TileRow = [u8; 8];

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

    next_hblank_start_cycle: u64,
    next_hblank_end_cycle: u64,
    next_vblank_start_cycle: u64,
    next_vblank_end_cycle: u64,
    next_mode_10_start_cycle: u64,
    next_mode_10_end_cycle: u64,

    running_until_cycle: u64,

    tiles: [Tile; TILE_COUNT],
    objs: [Obj; OBJ_COUNT],
}

#[derive(Copy, Clone)]
struct Tile([TileRow; 8]);

#[derive(Copy, Clone)]
struct Obj {
    x: u8,
    y: u8,
    char_: u8,
    flags: u8,
}

impl Obj {
    fn new() -> Obj {
        Obj {
            x: 0,
            y: 0,
            char_: 0,
            flags: 0,
        }
    }

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

impl Tile {
    pub fn new() -> Tile {
        Tile([[0; 8]; 8])
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

            next_hblank_start_cycle: LINE_CYCLE_TIME - HBLANK_DURATION - MODE_10_DURATION,
            next_hblank_end_cycle: LINE_CYCLE_TIME - MODE_10_DURATION,
            next_vblank_start_cycle: SCREEN_SIZE.1 as u64 * LINE_CYCLE_TIME,
            next_vblank_end_cycle: SCREEN_CYCLE_TIME,
            next_mode_10_start_cycle: LINE_CYCLE_TIME - HBLANK_DURATION,
            next_mode_10_end_cycle: LINE_CYCLE_TIME,
            running_until_cycle: 0,
            ly: 0,

            tiles: [Tile::new(); TILE_COUNT],
            objs: [Obj::new(); OBJ_COUNT],
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
        *[
            self.next_hblank_start_cycle,
            self.next_hblank_end_cycle,
            self.next_mode_10_start_cycle,
            self.next_mode_10_end_cycle,
            self.next_vblank_start_cycle,
            self.next_vblank_end_cycle,
        ].iter()
            .min()
            .unwrap()
    }

    pub fn set_running_until(&mut self, cycle: u64) {
        self.running_until_cycle = cycle;
    }

    pub fn pump_cycle(&mut self, cycle: u64) -> Option<Interrupt> {
        if cycle >= self.next_hblank_start_cycle {
            self.do_hblank_start(cycle);

            if self.is_hblank_int_enabled() && self.ly < SCREEN_SIZE.1 as u8 {
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else if cycle >= self.next_hblank_end_cycle {
            self.do_hblank_end();
            if self.ly == self.lyc && self.is_lyc_int_enabled() {
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else if cycle >= self.next_mode_10_start_cycle {
            self.next_mode_10_start_cycle = cycle + LINE_CYCLE_TIME;
            self.stat = (self.stat & 0b1111_1100) | MODE_10_MASK;

            if self.is_mode_10_int_enabled() {
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else if cycle >= self.next_mode_10_end_cycle {
            self.next_mode_10_end_cycle = cycle + LINE_CYCLE_TIME;
            self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
            None
        } else if cycle >= self.next_vblank_start_cycle {
            self.do_vblank_start();
            Some(Interrupt::VBlank)
        } else if cycle >= self.next_vblank_end_cycle {
            self.do_vblank_end();

            if self.ly == self.lyc && self.is_lyc_int_enabled() {
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn do_hblank_start(&mut self, cycle: u64) {
        if self.ly < SCREEN_SIZE.1 as u8 {
            if self.should_render_this_frame(cycle) {
                self.render_background_row();
                self.render_window_row();
                self.render_oam_row();
            }
            self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
        }
        self.next_hblank_start_cycle += LINE_CYCLE_TIME;
    }

    fn should_render_this_frame(&self, cycle: u64) -> bool {
        cycle >= self.running_until_cycle
            || self.running_until_cycle - cycle <= 2 * SCREEN_CYCLE_TIME
    }

    fn do_hblank_end(&mut self) {
        self.ly += 1;
        self.update_lyc();
        self.next_hblank_end_cycle += LINE_CYCLE_TIME;
    }

    pub fn do_vblank_start(&mut self) {
        self.swap();
        self.ly += 1;
        self.update_lyc();
        self.stat = (self.stat & 0b1111_1100) | MODE_01_MASK;
        self.next_vblank_start_cycle += SCREEN_CYCLE_TIME;
    }

    pub fn do_vblank_end(&mut self) {
        self.ly = 0;
        self.update_lyc();
        self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
        self.next_vblank_end_cycle += SCREEN_CYCLE_TIME;
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
        let row = self.render_tile_row(self.ly, self.sx, self.sy, self.get_bg_code_dat_start());
        for screen_x in 0..SCREEN_SIZE.0 {
            let screen_y = self.ly;
            self.get_back_framebuffer()[screen_y as usize][screen_x as usize] =
                row[screen_x as usize];
        }
    }

    fn render_window_row(&mut self) {
        if !self.is_window_enabled() {
            return;
        }

        let adjusted_wx = max(self.wx, 7) - 7;
        if self.wy > self.ly || adjusted_wx >= SCREEN_SIZE.0 as u8 {
            return;
        }

        let translated_y = self.ly - self.wy;
        let row = self.render_tile_row(translated_y, 0, 0, self.get_window_code_dat_start());

        let screen_y = self.ly;
        for screen_x in adjusted_wx..(SCREEN_SIZE.0 as u8) {
            self.get_back_framebuffer()[screen_y as usize][screen_x as usize] =
                row[screen_x as usize];
        }
    }

    fn render_tile_row(&self, screen_y: u8, scx: u8, scy: u8, code_dat_start: Address) -> FrameRow {
        let mut row = [COLOR_WHITE; SCREEN_SIZE.0];
        let translated_y = Wrapping(screen_y) + Wrapping(scy); // Implicit % 256
        for screen_x in 0..SCREEN_SIZE.0 {
            let translated_x = Wrapping(screen_x as u8) + Wrapping(scx); // Implicit % 256

            let char_y_offset = Wrapping(u16::from(translated_y.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR))
                * Wrapping(u16::from(BG_CHARS_PER_ROW));
            let char_offset = Wrapping(u16::from(translated_x.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR)) + char_y_offset;
            let char_addr = code_dat_start + Address(char_offset.0);
            let char_ = self.read(char_addr).unwrap();
            let signed = self.get_bg_char_addr_start();
            let char_row = self.read_char_row_at(char_, (translated_y % Wrapping(8)).0, signed);

            let color_index = char_row[(translated_x % Wrapping(8)).0 as usize];
            let corrected_index = palette_convert(color_index, self.bgp) as usize;
            row[screen_x as usize] = COLORS[corrected_index];
        }

        row
    }

    fn read_char_row_at(&self, char_: u8, row: u8, signed: bool) -> TileRow {
        let index = if signed {
            (256 + isize::from(char_ as i8)) as usize
        } else {
            char_ as usize
        };

        if row >= 8 {
            self.tiles[index + 1].0[row as usize - 8]
        } else {
            self.tiles[index].0[row as usize]
        }
    }

    fn is_bg_enabled(&self) -> bool {
        self.lcdc & BG_ENABLED_FLAG != 0
    }

    fn is_window_enabled(&self) -> bool {
        self.lcdc & WINDOW_ENABLED_FLAG != 0
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

    fn is_mode_10_int_enabled(&self) -> bool {
        self.stat & MODE_10_INT_FLAG != 0
    }

    fn get_bg_char_addr_start(&self) -> bool {
        self.lcdc & BGD_CHAR_DAT_FLAG == 0
    }

    fn get_bg_code_dat_start(&self) -> Address {
        if self.lcdc & BGD_CODE_DAT_FLAG == 0 {
            Address(0x9800)
        } else {
            Address(0x9C00)
        }
    }

    fn get_window_code_dat_start(&self) -> Address {
        if self.lcdc & WINDOW_CODE_DAT_FLAG == 0 {
            Address(0x9800)
        } else {
            Address(0x9C00)
        }
    }

    pub fn render_char_dat(&self, high: bool) -> Box<Framebuffer> {
        let mut fb = Box::new([[Pixel(255, 255, 0, 255); SCREEN_SIZE.0]; SCREEN_SIZE.1]);

        const CHARS_PER_ROW: u8 = (SCREEN_SIZE.0 as u8 / PIXEL_PER_CHAR);
        for char_ in 0..CHARS_PER_BANK {
            let base_x = (char_ % CHARS_PER_ROW) * 8;
            let base_y = (char_ / CHARS_PER_ROW) * 8;

            for y in 0..PIXEL_PER_CHAR {
                let row = self.read_char_row_at(char_, y, high);
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
        let signed = self.get_bg_char_addr_start();

        for char_y in 0..BG_CHARS_PER_ROW {
            for char_x in 0..BG_CHARS_PER_ROW {
                let char_offset =
                    Address(u16::from(char_x) + u16::from(char_y) * u16::from(BG_CHARS_PER_ROW));
                let char_ = self.read(code_start + char_offset).unwrap();

                for y in 0..PIXEL_PER_CHAR {
                    let row = self.read_char_row_at(char_, y, signed);
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

        for i in 0..OBJ_COUNT {
            let obj = self.objs[i];

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
                let row = self.read_char_row_at(char_, index_y, false);
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

    fn update_tile_at(&mut self, a: Address) {
        let byte_offset = a - RNG_CHAR_DAT.0;
        let char_offset = byte_offset.0 / BYTES_PER_CHAR;
        let row_offset = (byte_offset.0 % BYTES_PER_CHAR) / BYTES_PER_ROW;

        let b1 = self.cdata.read(byte_offset).unwrap();
        let b2 = self.cdata.read(byte_offset + Address(1)).unwrap();
        let mut row = [0; 8];
        for i in 0..8 {
            row[i] = read_bit(b1, (7 - i) as u8) | (read_bit(b2, (7 - i) as u8) << 1);
        }

        self.tiles[char_offset as usize].0[row_offset as usize] = row;
    }

    fn update_obj_at(&mut self, a: Address) {
        let byte_offset = a - RNG_LCD_OAM.0;
        let obj_index = byte_offset.0 / (RNG_LCD_OAM.len() / OBJ_COUNT) as u16;
        self.objs[obj_index as usize] = self.get_obj(obj_index as u8);
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
            self.cdata.write(a - RNG_CHAR_DAT.0, v)?;
            self.update_tile_at(Address(a.0 - a.0 % 2));
            Ok(())
        } else if a.in_(RNG_LCD_OAM) {
            self.oam.write(a - RNG_LCD_OAM.0, v)?;
            self.update_obj_at(a);
            Ok(())
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
