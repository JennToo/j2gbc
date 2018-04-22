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
                .window("j2gbc", 800, 800 * SCREEN_SIZE.1 / SCREEN_SIZE.0)
                .position_centered()
                .build()
                .map_err(|e| format!("{}", e))
        );
        let window_canvas = try!(
            window
                .into_canvas()
                .software()
                .build()
                .map_err(|e| format!("{}", e))
        );

        Ok(Window { ctx, window_canvas })
    }

    pub fn run(&mut self, mut system: System) -> Result<(), String> {
        let texture_creator = self.window_canvas.texture_creator();
        let gb_screen = try!(
            texture_creator
                .create_texture_target(
                    sdl2::pixels::PixelFormatEnum::RGB24,
                    SCREEN_SIZE.0,
                    SCREEN_SIZE.1
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

            let elapsed = dt.elapsed();
            system.run_for_duration(&elapsed);
            dt = Instant::now();

            self.window_canvas.clear();
            try!(self.window_canvas.copy(&gb_screen, None, None));
            self.window_canvas.present();
        }
    }
}
