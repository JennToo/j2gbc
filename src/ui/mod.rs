use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use emu::system::System;
use emu::lcd::SCREEN_SIZE;

mod fb;
pub mod debug;

use self::fb::{Framebuffers, RenderingState};

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
                .resizable()
                .maximized()
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
        let mut fbs = Framebuffers::new(&texture_creator)?;
        let mut dt = Instant::now();
        let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

        let mut debug = debug::Debug::new(&ttf_ctx)?;

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
                        fbs.rendering_state = RenderingState::Normal;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::F2),
                        ..
                    } => {
                        fbs.rendering_state = RenderingState::Debug;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::F4),
                        ..
                    } => {
                        fbs.rendering_state = RenderingState::Debug;
                        system.cpu.debug_halted = true;
                        debug.start_debugging(&mut system);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Backspace),
                        ..
                    } => {
                        debug.command_backspace();
                    }
                    Event::TextInput { text, .. } => {
                        debug.command_keystroke(text.as_str());
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        debug.run_command(&mut system);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        debug.scroll_up(1);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        debug.scroll_down(1);
                    }
                    _ => {}
                }
            }

            let elapsed = dt.elapsed();
            if elapsed > Duration::from_millis(17) {
                //println!("Warning: Slow frame {:?}", elapsed);
            }
            let was_debugging = system.cpu.debug_halted;
            system.run_for_duration(&elapsed);
            dt = Instant::now();
            if !was_debugging && system.cpu.debug_halted {
                debug.start_debugging(&mut system);
            }

            self.window_canvas.set_draw_color(Color::RGB(100, 100, 100));
            try!(self.window_canvas.fill_rect(None));
            try!(fbs.render(&system, &mut self.window_canvas));
            if fbs.rendering_state == RenderingState::Debug && system.cpu.debug_halted {
                debug.draw(&mut self.window_canvas, &texture_creator, &system)?;
            }
            self.window_canvas.present();
        }
    }
}
