use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

use emu::system::System;
use emu::lcd::SCREEN_SIZE;

pub struct Window {
    ctx: sdl2::Sdl,
    window_canvas: sdl2::render::WindowCanvas,
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

        Ok(Window { ctx, window_canvas })
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
                    _ => {}
                }
            }

            self.window_canvas.clear();

            let elapsed = dt.elapsed();
            system.run_for_duration(&elapsed);
            dt = Instant::now();

            let fb = system.get_framebuffer();
            try!(
                gb_screen
                    .with_lock(None, |outfb, _| for y in 0..SCREEN_SIZE.1 {
                        for x in 0..SCREEN_SIZE.0 {
                            let index = 4 * x + 4 * y * SCREEN_SIZE.0;

                            let pixel = fb[y][x];
                            outfb[index] = pixel.3;
                            outfb[index + 1] = pixel.2;
                            outfb[index + 2] = pixel.1;
                            outfb[index + 3] = pixel.0;
                        }
                    })
                    .map_err(|e| format!("{}", e))
            );

            try!(self.window_canvas.copy(&gb_screen, None, None));
            self.window_canvas.present();
        }
    }
}
