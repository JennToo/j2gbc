use std::cmp::max;
use std::num::Wrapping;

use j2ds::{next_timer_event, Timer, TimerEvent};
use log::error;

use crate::{
    cpu::{Interrupt, CLOCK_RATE},
    mem::{Address, MemDevice, Ram, RNG_CHAR_DAT, RNG_LCD_BGDD1, RNG_LCD_BGDD2, RNG_LCD_OAM},
    system::SystemMode,
};

mod bg;
pub mod fb;
mod obj;
mod tile;

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
const REG_VBK: Address = Address(0xFF4F);
const REG_BCPS: Address = Address(0xFF68);
const REG_BCPD: Address = Address(0xFF69);
const REG_OCPS: Address = Address(0xFF6A);
const REG_OCPD: Address = Address(0xFF6B);

const LINE_CYCLE_TIME: u64 = CLOCK_RATE * 108_700 / 1_000_000_000; // Src: Official GB manual
const HBLANK_DURATION: u64 = CLOCK_RATE * 48_600 / 1_000_000_000; // Src: GBCPUMan.pdf
const MODE_10_DURATION: u64 = CLOCK_RATE * 19_000 / 1_000_000_000; // Src: GBCPUMan.pdf
const VBLANK_DURATION: u64 = LINE_CYCLE_TIME * 10; // Src: Official GB manual
const SCREEN_CYCLE_TIME: u64 = 154 * LINE_CYCLE_TIME;
const BYTES_PER_CHAR: u16 = 16;
const BYTES_PER_ROW: u16 = 2;
const BG_CHARS_PER_ROW: u8 = 32;
const PIXEL_PER_CHAR: u8 = 8;

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
const LCD_ENABLED_FLAG: u8 = 0b1000_0000;
const OAM_TALL_FLAG: u8 = 0b0000_0100;
const BGD_CHAR_DAT_FLAG: u8 = 0b0001_0000;
const BGD_CODE_DAT_FLAG: u8 = 0b0000_1000;
const WINDOW_CODE_DAT_FLAG: u8 = 0b0100_0000;

const TILE_COUNT: usize = 384 * 2;
const OBJ_COUNT: usize = 40;

const PAL_DATA_IDX: u8 = 0b11_1111;

type CgbPalette = [fb::Pixel; 4];

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
    bcps: u8,
    ocps: u8,
    bank_select: usize,
    cdata: Ram,
    bgdd1: Ram,
    bgdd2: Ram,
    oam: Ram,

    bcp: [u8; 0x40],
    ocp: [u8; 0x40],

    obj_palettes: [CgbPalette; 8],
    bg_palettes: [CgbPalette; 8],

    fbs: [fb::Framebuffer; 2],
    fbi: usize,

    hblank_timer: Timer,
    vblank_timer: Timer,
    mode10_timer: Timer,

    running_until_cycle: u64,

    tiles: [tile::MonoTile; TILE_COUNT],
    objs: [obj::Obj; OBJ_COUNT],

    system_mode: SystemMode,
}

impl Lcd {
    pub fn new(cgb_mode: bool) -> Lcd {
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
            bcps: 0,
            ocps: 0,
            bank_select: 0,
            cdata: Ram::new(RNG_CHAR_DAT.len() * 2),
            bgdd1: Ram::new(RNG_LCD_BGDD1.len() * 2),
            bgdd2: Ram::new(RNG_LCD_BGDD2.len() * 2),
            oam: Ram::new(RNG_LCD_OAM.len()),
            fbs: [fb::Framebuffer::default(); 2],
            fbi: 0,

            bcp: [0; 0x40],
            ocp: [0; 0x40],

            obj_palettes: [[fb::DMG_COLOR_WHITE; 4]; 8],
            bg_palettes: [[fb::DMG_COLOR_WHITE; 4]; 8],

            hblank_timer: Timer::new(
                LINE_CYCLE_TIME,
                LINE_CYCLE_TIME - HBLANK_DURATION - MODE_10_DURATION,
                HBLANK_DURATION,
            ),
            vblank_timer: Timer::new(
                SCREEN_CYCLE_TIME,
                fb::SCREEN_SIZE.1 as u64 * LINE_CYCLE_TIME,
                VBLANK_DURATION,
            ),
            mode10_timer: Timer::new(
                LINE_CYCLE_TIME,
                LINE_CYCLE_TIME - HBLANK_DURATION,
                HBLANK_DURATION,
            ),
            running_until_cycle: 0,
            ly: 0,

            tiles: [tile::MonoTile::default(); TILE_COUNT],
            objs: [obj::Obj::default(); OBJ_COUNT],

            system_mode: if cgb_mode {
                SystemMode::CGB
            } else {
                SystemMode::DMG
            },
        }
    }

    pub fn get_framebuffer(&self) -> &fb::Framebuffer {
        &self.fbs[self.fbi]
    }

    fn get_back_framebuffer(&mut self) -> &mut fb::Framebuffer {
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
        next_timer_event(&[self.hblank_timer, self.vblank_timer, self.mode10_timer])
    }

    pub fn set_running_until(&mut self, cycle: u64) {
        self.running_until_cycle = cycle;
    }

    pub fn pump_cycle(&mut self, cycle: u64) -> Option<Interrupt> {
        match self.hblank_timer.update(cycle) {
            Some(TimerEvent::RisingEdge) => {
                self.do_hblank_start(cycle);
                if self.is_hblank_int_enabled() && self.ly < fb::SCREEN_SIZE.1 as u8 {
                    return Some(Interrupt::LCDC);
                }
            }
            Some(TimerEvent::FallingEdge) => {
                self.do_hblank_end();
                if self.ly == self.lyc && self.is_lyc_int_enabled() {
                    return Some(Interrupt::LCDC);
                }
            }
            None => {}
        }

        match self.mode10_timer.update(cycle) {
            Some(TimerEvent::RisingEdge) => {
                self.stat = (self.stat & 0b1111_1100) | MODE_10_MASK;

                if self.is_mode_10_int_enabled() {
                    return Some(Interrupt::LCDC);
                }
            }
            Some(TimerEvent::FallingEdge) => {
                self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
            }
            None => {}
        }

        match self.vblank_timer.update(cycle) {
            Some(TimerEvent::RisingEdge) => {
                self.do_vblank_start();
                return Some(Interrupt::VBlank);
            }
            Some(TimerEvent::FallingEdge) => {
                self.do_vblank_end();

                if self.ly == self.lyc && self.is_lyc_int_enabled() {
                    return Some(Interrupt::LCDC);
                }
            }
            None => {}
        }

        None
    }

    fn do_hblank_start(&mut self, cycle: u64) {
        if self.ly < fb::SCREEN_SIZE.1 as u8 {
            if self.should_render_this_frame(cycle) {
                self.render_screen_row();
            }
            self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
        }
    }

    fn should_render_this_frame(&self, cycle: u64) -> bool {
        cycle >= self.running_until_cycle
            || self.running_until_cycle - cycle <= 2 * SCREEN_CYCLE_TIME
    }

    fn do_hblank_end(&mut self) {
        self.ly += 1;
        self.update_lyc();
    }

    pub fn do_vblank_start(&mut self) {
        self.swap();
        self.ly += 1;
        self.update_lyc();
        self.stat = (self.stat & 0b1111_1100) | MODE_01_MASK;
    }

    pub fn do_vblank_end(&mut self) {
        self.ly = 0;
        self.update_lyc();
        self.stat = (self.stat & 0b1111_1100) | MODE_00_MASK;
    }

    fn update_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= LYC_MATCH_FLAG;
        } else {
            self.stat &= !LYC_MATCH_FLAG;
        }
    }

    fn render_screen_row(&mut self) {
        if !self.is_lcd_enabled() {
            let y = self.ly as usize;
            for x in 0..(fb::SCREEN_SIZE.0 as usize) {
                self.get_back_framebuffer().set(x, y, fb::DMG_COLOR_WHITE);
            }
            return;
        }

        let mut bg_screen_row =
            [fb::TentativePixel::new(fb::DMG_COLOR_WHITE, false, true); fb::SCREEN_SIZE.0];
        let mut oam_screen_row = [None; fb::SCREEN_SIZE.0];
        self.render_background_row(&mut bg_screen_row);
        self.render_window_row(&mut bg_screen_row);
        self.render_oam_row(&mut oam_screen_row);

        let y = self.ly as usize;
        for x in 0..(fb::SCREEN_SIZE.0 as usize) {
            let color = fb::resolve_pixel(self.system_mode, oam_screen_row[x], bg_screen_row[x]);

            self.get_back_framebuffer().set(x, y, color);
        }
    }

    fn render_background_row(&self, screen_row: &mut [fb::TentativePixel]) {
        if !self.is_bg_enabled() {
            return;
        }
        self.render_tile_row(
            self.ly,
            self.sx,
            self.sy,
            self.get_bg_code_dat_start(),
            screen_row,
        );
    }

    fn render_window_row(&self, screen_row: &mut [fb::TentativePixel]) {
        if !self.is_window_enabled() {
            return;
        }

        let adjusted_wx = max(self.wx, 7) - 7;
        if self.wy > self.ly || adjusted_wx >= fb::SCREEN_SIZE.0 as u8 {
            return;
        }

        let translated_y = self.ly - self.wy;
        self.render_tile_row(
            translated_y,
            0,
            0,
            self.get_window_code_dat_start(),
            screen_row,
        );
    }

    fn render_tile_row(
        &self,
        screen_y: u8,
        scx: u8,
        scy: u8,
        code_dat_start: Address,
        screen_row: &mut [fb::TentativePixel],
    ) {
        let translated_y = Wrapping(screen_y) + Wrapping(scy); // Implicit % 256
        for screen_x in 0..fb::SCREEN_SIZE.0 {
            let translated_x = Wrapping(screen_x as u8) + Wrapping(scx); // Implicit % 256

            let char_y_offset = Wrapping(u16::from(translated_y.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR))
                * Wrapping(u16::from(BG_CHARS_PER_ROW));
            let char_offset = Wrapping(u16::from(translated_x.0))
                / Wrapping(u16::from(PIXEL_PER_CHAR))
                + char_y_offset;
            let (char_, flags) = if code_dat_start == RNG_LCD_BGDD1.0 {
                (
                    self.bgdd1.read(Address(char_offset.0)).unwrap(),
                    self.bgdd1
                        .read(Address(char_offset.0 + (RNG_LCD_BGDD1.len() as u16)))
                        .unwrap(),
                )
            } else {
                (
                    self.bgdd2.read(Address(char_offset.0)).unwrap(),
                    self.bgdd2
                        .read(Address(char_offset.0 + (RNG_LCD_BGDD2.len() as u16)))
                        .unwrap(),
                )
            };
            let flags = bg::BgFlags::new(flags, self.system_mode);

            let maybe_flipped_y = if flags.yflip() {
                Wrapping(7) - (translated_y % Wrapping(8))
            } else {
                translated_y % Wrapping(8)
            };

            let signed = self.get_bg_char_addr_start();
            let char_row = self.read_char_row_at(char_, maybe_flipped_y.0, signed, flags.bank());

            let (color, data) = match self.system_mode {
                SystemMode::CGB => {
                    let maybe_flipped_x = if flags.xflip() {
                        Wrapping(7) - (translated_x % Wrapping(8))
                    } else {
                        translated_x % Wrapping(8)
                    };
                    let color_index = char_row[maybe_flipped_x.0 as usize];
                    (
                        self.bg_palettes[flags.cgb_pallete() as usize][color_index as usize],
                        color_index,
                    )
                }
                SystemMode::DMG => {
                    let color_index = char_row[(translated_x % Wrapping(8)).0 as usize];
                    let corrected_index = palette_convert(color_index, self.bgp) as usize;
                    (fb::DMG_COLORS[corrected_index], color_index)
                }
            };

            screen_row[screen_x as usize] =
                fb::TentativePixel::new(color, flags.priority(), data == 0);
        }
    }

    fn read_char_row_at(&self, char_: u8, row: u8, signed: bool, bank: u8) -> tile::MonoTileRow {
        let index = if signed {
            (256 + isize::from(char_ as i8)) as usize
        } else {
            char_ as usize
        } + (bank as usize) * (256 + 128);

        if row >= 8 {
            self.tiles[index + 1].read_row(row as usize - 8)
        } else {
            self.tiles[index].read_row(row as usize)
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

    fn is_lcd_enabled(&self) -> bool {
        self.lcdc & LCD_ENABLED_FLAG != 0
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

    fn read_obj(&self, index: u8) -> obj::Obj {
        let a = RNG_LCD_OAM.0 + Address(u16::from(index) * 4);

        obj::Obj::new(
            self.read(a + Address(1)).unwrap(),
            self.read(a).unwrap(),
            self.read(a + Address(2)).unwrap(),
            self.read(a + Address(3)).unwrap(),
            self.system_mode,
        )
    }

    fn render_oam_row(&self, screen_row: &mut [Option<fb::TentativePixel>]) {
        if !self.is_oam_enabled() {
            return;
        }

        for i in 0..OBJ_COUNT {
            let obj = self.objs[OBJ_COUNT - i - 1];

            let (char_, hi_y) = if self.lcdc & OAM_TALL_FLAG != 0 {
                (obj.char_ & 0b1111_1110, 16)
            } else {
                (obj.char_, 8)
            };

            for y in 0..hi_y {
                let full_y = y as isize + obj.y as isize - 16;
                if full_y > fb::SCREEN_SIZE.1 as isize || full_y < 0 || full_y != self.ly as isize {
                    continue;
                }

                let index_y = if obj.yflip() { hi_y - 1 - y } else { y };
                let row = self.read_char_row_at(char_, index_y, false, obj.bank());
                for x in 0..8 {
                    let full_x = x as isize + obj.x as isize - 8;

                    if full_x >= fb::SCREEN_SIZE.0 as isize || full_x < 0 {
                        continue;
                    }

                    let index_x = if obj.xflip() { 7 - x } else { x };
                    let color_index = row[index_x as usize];
                    if color_index == 0 {
                        // 0 is always transparent
                        continue;
                    }
                    let color = match self.system_mode {
                        SystemMode::CGB => {
                            self.obj_palettes[obj.cgb_palette() as usize][color_index as usize]
                        }
                        SystemMode::DMG => {
                            let pal = if obj.high_palette() {
                                self.obp1
                            } else {
                                self.obp0
                            };
                            let corrected_index = palette_convert(color_index, pal) as usize;
                            fb::DMG_COLORS[corrected_index]
                        }
                    };

                    screen_row[full_x as usize] = Some(fb::TentativePixel::new(
                        color,
                        !obj.priority(),
                        color_index == 0,
                    ));
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
        self.tiles[char_offset as usize].update_row(row_offset as usize, b1, b2);
    }

    fn update_obj_at(&mut self, a: Address) {
        let byte_offset = a - RNG_LCD_OAM.0;
        let obj_index = byte_offset.0 / (RNG_LCD_OAM.len() / OBJ_COUNT) as u16;
        self.objs[obj_index as usize] = self.read_obj(obj_index as u8);
    }
}

fn load_color_from_data(data: &[u8], pal_out: &mut [CgbPalette]) {
    let mut i = 0;
    for pal in 0..8 {
        for color_index in 0..4 {
            let l = data[i];
            let h = data[i + 1];

            let r = (l & 0b0001_1111) as u16;
            let g = ((l >> 5) | ((h & 0b11) << 3)) as u16;
            let b = ((h >> 2) & 0b0001_1111) as u16;
            let a = 255;

            let ar = (r * 255 / 0x1F) as u8;
            let ag = (g * 255 / 0x1F) as u8;
            let ab = (b * 255 / 0x1F) as u8;

            pal_out[pal as usize][color_index as usize] = [ar, ag, ab, a];

            i += 2;
        }
    }
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

impl MemDevice for Lcd {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_LCD_BGDD1) {
            self.bgdd1.read(
                a - RNG_LCD_BGDD1.0 + Address((self.bank_select * RNG_LCD_BGDD1.len()) as u16),
            )
        } else if a.in_(RNG_LCD_BGDD2) {
            self.bgdd2.read(
                a - RNG_LCD_BGDD2.0 + Address((self.bank_select * RNG_LCD_BGDD2.len()) as u16),
            )
        } else if a.in_(RNG_CHAR_DAT) {
            self.cdata
                .read(a - RNG_CHAR_DAT.0 + Address((self.bank_select * RNG_CHAR_DAT.len()) as u16))
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
                REG_VBK => Ok(self.bank_select as u8),
                REG_BCPS => Ok(self.bcps),
                REG_BCPD => Ok(self.bcp[(self.bcps & PAL_DATA_IDX) as usize]),
                REG_OCPS => Ok(self.ocps),
                REG_OCPD => Ok(self.ocp[(self.ocps & PAL_DATA_IDX) as usize]),
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
            let adjusted = a + Address((self.bank_select * RNG_LCD_BGDD1.len()) as u16);
            self.bgdd1.write(adjusted - RNG_LCD_BGDD1.0, v)
        } else if a.in_(RNG_LCD_BGDD2) {
            let adjusted = a + Address((self.bank_select * RNG_LCD_BGDD2.len()) as u16);
            self.bgdd2.write(adjusted - RNG_LCD_BGDD2.0, v)
        } else if a.in_(RNG_CHAR_DAT) {
            let adjusted = a + Address((self.bank_select * RNG_CHAR_DAT.len()) as u16);
            self.cdata.write(adjusted - RNG_CHAR_DAT.0, v)?;
            self.update_tile_at(Address(adjusted.0 - adjusted.0 % 2));
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
                REG_VBK => {
                    self.bank_select = usize::from(v & 0b1);
                    Ok(())
                }
                REG_BCPS => {
                    self.bcps = v & 0b1011_1111;
                    Ok(())
                }
                REG_BCPD => {
                    let mut idx = (self.bcps & PAL_DATA_IDX) as usize;
                    self.bcp[idx] = v;
                    if self.bcps & 0b1000_0000 != 0 {
                        idx += 1;
                        if idx >= 0x40 {
                            idx = 0;
                        }
                        self.bcps = (self.bcps & !PAL_DATA_IDX) | (idx as u8);
                    }
                    load_color_from_data(&self.bcp, &mut self.bg_palettes);
                    Ok(())
                }
                REG_OCPS => {
                    self.ocps = v & 0b1011_1111;
                    Ok(())
                }
                REG_OCPD => {
                    let mut idx = (self.ocps & PAL_DATA_IDX) as usize;
                    self.ocp[idx] = v;
                    if self.ocps & 0b1000_0000 != 0 {
                        idx += 1;
                        if idx >= 0x40 {
                            idx = 0;
                        }
                        self.ocps = (self.ocps & !PAL_DATA_IDX) | (idx as u8);
                    }
                    load_color_from_data(&self.ocp, &mut self.obj_palettes);
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
