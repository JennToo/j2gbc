pub const SCREEN_SIZE: (usize, usize) = (160, 144);
pub const DMG_COLOR_WHITE: Pixel = [234, 255, 186, 255];
pub const DMG_COLOR_LIGHT_GRAY: Pixel = [150, 187, 146, 255];
pub const DMG_COLOR_DARK_GRAY: Pixel = [68, 106, 81, 255];
pub const DMG_COLOR_BLACK: Pixel = [0, 14, 2, 255];
pub const DMG_COLORS: [Pixel; 4] = [
    DMG_COLOR_WHITE,
    DMG_COLOR_LIGHT_GRAY,
    DMG_COLOR_DARK_GRAY,
    DMG_COLOR_BLACK,
];

pub type Pixel = [u8; 4];

#[derive(Copy, Clone)]
pub struct Framebuffer([Pixel; SCREEN_SIZE.0 * SCREEN_SIZE.1]);

impl Default for Framebuffer {
    fn default() -> Framebuffer {
        Framebuffer([DMG_COLOR_WHITE; SCREEN_SIZE.0 * SCREEN_SIZE.1])
    }
}

impl Framebuffer {
    pub fn set(&mut self, x: usize, y: usize, color: Pixel) {
        self.0[x + y * SCREEN_SIZE.0] = color;
    }

    pub fn get(&self, x: usize, y: usize) -> Pixel {
        self.0[x + y * SCREEN_SIZE.0]
    }

    pub fn raw(&self) -> &[Pixel] {
        &self.0
    }
}
