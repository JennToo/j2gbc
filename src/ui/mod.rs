use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use std::time::{Duration, Instant};

use emu::system::System;
use emu::lcd::{Framebuffer, SCREEN_SIZE};

pub struct Window {
    ctx: sdl2::Sdl,
    window_canvas: sdl2::render::WindowCanvas,
    rendering_state: RenderingState,
}

enum RenderingState {
    LcdFramebuffer,
    CharDat(bool),
}

impl Window {
    pub fn new() -> Result<Window, String> {
        let ctx = try!(sdl2::init());
        let video = try!(ctx.video());
        let window = try!(
            video
                .window("j2gbc", 800, (800 * SCREEN_SIZE.1 / SCREEN_SIZE.0) as u32)
                .position_centered()
                .build()
                .map_err(|e| format!("{}", e))
        );
        let window_canvas = try!(
            window
                .into_canvas()
                .present_vsync()
                .build()
                .map_err(|e| format!("{}", e))
        );

        Ok(Window {
            ctx,
            window_canvas,
            rendering_state: RenderingState::LcdFramebuffer,
        })
    }

    pub fn run(&mut self, mut system: System) -> Result<(), String> {
        let texture_creator = self.window_canvas.texture_creator();
        let mut gb_screen = try!(
            texture_creator
                .create_texture_streaming(
                    sdl2::pixels::PixelFormatEnum::RGBA8888,
                    SCREEN_SIZE.0 as u32,
                    SCREEN_SIZE.1 as u32
                )
                .map_err(|e| format!("{}", e))
        );

        let mut dt = Instant::now();

        loop {
            for event in self.ctx.event_pump().unwrap().poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => return Ok(()),

                    Event::KeyDown {
                        keycode: Some(Keycode::F1),
                        ..
                    } => {
                        self.rendering_state = RenderingState::LcdFramebuffer;
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::F2),
                        ..
                    } => {
                        self.rendering_state = RenderingState::CharDat(false);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::F3),
                        ..
                    } => {
                        self.rendering_state = RenderingState::CharDat(true);
                    }

                    _ => {}
                }
            }

            self.window_canvas.clear();

            let elapsed = dt.elapsed();
            if elapsed > Duration::from_millis(17) {
                println!("Warning: Slow frame {:?}", elapsed);
            }
            system.run_for_duration(&elapsed);
            dt = Instant::now();

            try!(self.render(&system, &mut gb_screen));

            try!(self.window_canvas.copy(&gb_screen, None, None));
            self.window_canvas.present();
        }
    }

    fn render<'r>(&mut self, system: &System, gb_screen: &mut Texture<'r>) -> Result<(), String> {
        match self.rendering_state {
            RenderingState::LcdFramebuffer => {
                let fb = system.get_framebuffer();
                copy_framebuffer(fb, gb_screen)
            }
            RenderingState::CharDat(high) => {
                let fb = system.cpu.mmu.lcd.render_char_dat(high);
                copy_framebuffer(&fb, gb_screen)
            }
        }
    }
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
        })
        .map_err(|e| format!("{}", e))
}
