use crate::system::SystemMode;

pub const SCREEN_SIZE: (usize, usize) = (160, 144);
pub const DMG_COLOR_WHITE: Pixel = [234, 255, 186];
pub const DMG_COLOR_LIGHT_GRAY: Pixel = [150, 187, 146];
pub const DMG_COLOR_DARK_GRAY: Pixel = [68, 106, 81];
pub const DMG_COLOR_BLACK: Pixel = [0, 14, 2];
pub const DMG_COLORS: [Pixel; 4] = [
    DMG_COLOR_WHITE,
    DMG_COLOR_LIGHT_GRAY,
    DMG_COLOR_DARK_GRAY,
    DMG_COLOR_BLACK,
];

pub type Pixel = [u8; 3];

#[derive(Clone)]
pub struct Framebuffer {
    data: Vec<Pixel>,
    size: (usize, usize),
}

impl Framebuffer {
    pub fn new((width, height): (usize, usize)) -> Framebuffer {
        let mut v = Vec::with_capacity(width * height);
        v.resize(width * height, DMG_COLOR_WHITE);
        Framebuffer {
            data: v,
            size: (width, height),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Pixel) {
        self.data[x + y * self.size.0] = color;
    }

    pub fn get(&self, x: usize, y: usize) -> Pixel {
        self.data[x + y * self.size.0]
    }

    pub fn raw(&self) -> &[Pixel] {
        &self.data
    }

    pub fn draw_wrapping_vline(&mut self, x: usize, y: usize, len: usize, color: Pixel) {
        for i in 0..len {
            let y = (y + i) % self.size.1;
            self.set(x, y, color);
        }
    }

    pub fn draw_wrapping_hline(&mut self, x: usize, y: usize, len: usize, color: Pixel) {
        for i in 0..len {
            let x = (x + i) % self.size.0;
            self.set(x, y, color);
        }
    }

    pub fn draw_wrapping_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: Pixel,
    ) {
        self.draw_wrapping_vline(x, y, height, color);
        self.draw_wrapping_hline(x, y, width, color);

        self.draw_wrapping_vline((x + width) % self.size.0, y, height, color);
        self.draw_wrapping_hline(x, (y + height) % self.size.1, width, color);
    }
}

#[derive(Copy, Clone)]
pub struct TentativePixel {
    color: Pixel,
    has_priority: bool,
    data_was_zero: bool,
}

impl TentativePixel {
    pub fn new(color: Pixel, has_priority: bool, data_was_zero: bool) -> TentativePixel {
        TentativePixel {
            color,
            has_priority,
            data_was_zero,
        }
    }

    pub fn color(self) -> Pixel {
        self.color
    }
}

pub fn resolve_pixel(mode: SystemMode, oam: Option<TentativePixel>, bg: TentativePixel) -> Pixel {
    match mode {
        SystemMode::DMG => resolve_pixel_dmg(oam, bg),
        SystemMode::CGB => resolve_pixel_cgb(oam, bg),
    }
}

// Based on a table from the Game Boy Programming Manual
pub fn resolve_pixel_cgb(oam: Option<TentativePixel>, bg: TentativePixel) -> Pixel {
    if let Some(oam) = oam {
        if bg.has_priority || !oam.has_priority {
            if bg.data_was_zero {
                oam.color
            } else {
                bg.color
            }
        } else {
            if oam.data_was_zero {
                bg.color
            } else {
                oam.color
            }
        }
    } else {
        bg.color
    }
}

pub fn resolve_pixel_dmg(oam: Option<TentativePixel>, bg: TentativePixel) -> Pixel {
    if let Some(oam) = oam {
        if oam.data_was_zero || !oam.has_priority {
            bg.color
        } else {
            oam.color
        }
    } else {
        bg.color
    }
}
