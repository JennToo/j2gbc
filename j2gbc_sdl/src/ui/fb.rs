use sdl2;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

use j2gbc::lcd::{BgBuffer, Framebuffer, SCREEN_SIZE};
use j2gbc::system::System;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RenderingState {
    Normal,
    Debug,
}

pub struct Framebuffers<'r> {
    lcd_screen: Texture<'r>,
    bg_screen: Texture<'r>,
    pub rendering_state: RenderingState,
}

impl<'r> Framebuffers<'r> {
    pub fn new(
        texture_creator: &'r TextureCreator<WindowContext>,
    ) -> Result<Framebuffers<'r>, String> {
        let lcd_screen =
            Framebuffers::make_tex(texture_creator, SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32)?;
        let bg_screen = Framebuffers::make_tex(texture_creator, 256, 256)?;

        Ok(Framebuffers {
            lcd_screen,
            bg_screen,
            rendering_state: RenderingState::Normal,
        })
    }

    fn make_tex(
        texture_creator: &'r TextureCreator<WindowContext>,
        width: u32,
        height: u32,
    ) -> Result<Texture<'r>, String> {
        texture_creator
            .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA8888, width, height)
            .map_err(|e| format!("{}", e))
    }

    pub fn render(
        &mut self,
        system: &System,
        window_canvas: &mut WindowCanvas,
    ) -> Result<(), String> {
        let window_size = window_canvas.output_size()?;

        match self.rendering_state {
            RenderingState::Normal => {
                let fb = system.get_framebuffer();
                copy_framebuffer(fb, &mut self.lcd_screen)?;

                let lcd_prop_size =
                    proportional_subset((SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32), window_size);
                let lcd_center_offset = center_in(lcd_prop_size, window_size);
                let target = Rect::new(
                    lcd_center_offset.0 as i32,
                    lcd_center_offset.1 as i32,
                    lcd_prop_size.0,
                    lcd_prop_size.1,
                );
                window_canvas.copy(&self.lcd_screen, None, target)?;
            }
            RenderingState::Debug => {
                let fb = system.get_framebuffer();
                copy_framebuffer(&fb, &mut self.lcd_screen)?;
                let target = Rect::new(4, 4, SCREEN_SIZE.0 as u32 * 2, SCREEN_SIZE.1 as u32 * 2);
                window_canvas.copy(&self.lcd_screen, None, target)?;

                let fb = system.cpu.mmu.lcd.render_char_dat(false);
                copy_framebuffer(&fb, &mut self.lcd_screen)?;
                let target = Rect::new(
                    4,
                    8 + (SCREEN_SIZE.1 as i32) * 2,
                    SCREEN_SIZE.0 as u32 * 2,
                    SCREEN_SIZE.1 as u32 * 2,
                );
                window_canvas.copy(&self.lcd_screen, None, target)?;

                let fb = system.cpu.mmu.lcd.render_char_dat(true);
                copy_framebuffer(&fb, &mut self.lcd_screen)?;
                let target = Rect::new(
                    4,
                    4 + (4 + (SCREEN_SIZE.1 as i32) * 2) * 2,
                    SCREEN_SIZE.0 as u32 * 2,
                    SCREEN_SIZE.1 as u32 * 2,
                );
                window_canvas.copy(&self.lcd_screen, None, target)?;

                let fb = system.cpu.mmu.lcd.render_background(false);
                copy_bgbuffer(&fb, &mut self.bg_screen)?;
                let target = Rect::new(8 + (SCREEN_SIZE.0 as i32) * 2, 4, 256 * 2, 256 * 2);
                window_canvas.copy(&self.bg_screen, None, target)?;

                let fb = system.cpu.mmu.lcd.render_background(true);
                copy_bgbuffer(&fb, &mut self.bg_screen)?;
                let target = Rect::new(
                    8 + (SCREEN_SIZE.0 as i32) * 2,
                    4 + (4 + 256 * 2),
                    256 * 2,
                    256 * 2,
                );
                window_canvas.copy(&self.bg_screen, None, target)?;
            }
        }

        Ok(())
    }
}

fn proportional_subset(proportions: (u32, u32), max_size: (u32, u32)) -> (u32, u32) {
    let y_prop_corrected = max_size.0 * proportions.1 / proportions.0;
    let x_prop_corrected = max_size.1 * proportions.0 / proportions.1;

    if x_prop_corrected < y_prop_corrected {
        (x_prop_corrected, max_size.1)
    } else {
        (max_size.0, y_prop_corrected)
    }
}

fn center_in(inner: (u32, u32), outer: (u32, u32)) -> (u32, u32) {
    ((outer.0 - inner.0) / 2, (outer.1 - inner.1) / 2)
}

fn copy_framebuffer<'r>(fb: &Framebuffer, gb_screen: &mut Texture<'r>) -> Result<(), String> {
    gb_screen
        .with_lock(None, |outfb, _| {
            for y in 0..SCREEN_SIZE.1 {
                for x in 0..SCREEN_SIZE.0 {
                    let index = 4 * x + 4 * y * SCREEN_SIZE.0;

                    let pixel = fb[y][x];
                    outfb[index] = pixel.3;
                    outfb[index + 1] = pixel.2;
                    outfb[index + 2] = pixel.1;
                    outfb[index + 3] = pixel.0;
                }
            }
        }).map_err(|e| e.to_string())
}

fn copy_bgbuffer<'r>(fb: &BgBuffer, gb_screen: &mut Texture<'r>) -> Result<(), String> {
    gb_screen
        .with_lock(None, |outfb, _| {
            for y in 0..256 {
                for x in 0..256 {
                    let index = 4 * x + 4 * y * 256;

                    let pixel = fb[y][x];
                    outfb[index] = pixel.3;
                    outfb[index + 1] = pixel.2;
                    outfb[index + 2] = pixel.1;
                    outfb[index + 3] = pixel.0;
                }
            }
        }).map_err(|e| e.to_string())
}
