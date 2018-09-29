use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use emu::input::Button;
use emu::lcd::SCREEN_SIZE;
use emu::system::System;

pub mod audio;
pub mod debug;
mod fb;

use self::fb::{Framebuffers, RenderingState};

pub struct Window {
    ctx: sdl2::Sdl,
    window_canvas: sdl2::render::WindowCanvas,
    save_file: String,
}

impl Window {
    pub fn new(save_file: String) -> Result<Window, String> {
        let ctx = sdl2::init()?;
        let video = ctx.video()?;
        let window = video
            .window("j2gbc", 800, (800 * SCREEN_SIZE.1 / SCREEN_SIZE.0) as u32)
            .position_centered()
            .resizable()
            .maximized()
            .build()
            .map_err(|e| format!("{}", e))?;
        let window_canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| format!("{}", e))?;

        Ok(Window {
            ctx,
            window_canvas,
            save_file,
        })
    }

    pub fn run(&mut self, mut system: System) -> Result<(), String> {
        let texture_creator = self.window_canvas.texture_creator();
        let mut fbs = Framebuffers::new(&texture_creator)?;
        let mut dt = Instant::now();
        let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

        let mut debug = debug::Debug::new(&ttf_ctx)?;
        let mut super_speed = false;
        let mut save_timer = Duration::from_secs(0);

        //fbs.rendering_state = RenderingState::Debug;
        //system.cpu.debug_halted = true;
        //debug.start_debugging(&mut system);

        loop {
            for event in self.ctx.event_pump().unwrap().poll_iter() {
                handle_buttons(&event, &mut system);
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
                        if system.cpu.debug_halted {
                            debug.command_backspace();
                        }
                    }
                    Event::TextInput { text, .. } => {
                        if system.cpu.debug_halted {
                            debug.command_keystroke(text.as_str());
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        if system.cpu.debug_halted {
                            debug.run_command(&mut system);
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        if system.cpu.debug_halted {
                            debug.scroll_up(1);
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        if system.cpu.debug_halted {
                            debug.scroll_down(1);
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Tab),
                        ..
                    } => {
                        super_speed = !super_speed;
                    }
                    _ => {}
                }
            }

            let elapsed = if super_speed {
                dt.elapsed() * 8
            } else {
                dt.elapsed()
            };
            dt = Instant::now();

            if elapsed > Duration::from_millis(17) {
                info!(target: "events", "Slow frame {:?}", elapsed);
            }
            let was_debugging = system.cpu.debug_halted;
            system.run_for_duration(&elapsed);
            if !was_debugging && system.cpu.debug_halted {
                debug.start_debugging(&mut system);
            }

            save_timer += elapsed;
            if save_timer > Duration::from_secs(1) {
                save_timer = Duration::from_secs(0);

                let mut f = File::create(self.save_file.as_str()).map_err(|e| e.to_string())?;
                f.write(system.cpu.mmu.cart.get_sram())
                    .map_err(|e| e.to_string())?;
            }

            self.window_canvas.set_draw_color(Color::RGB(100, 100, 100));
            self.window_canvas.fill_rect(None)?;
            fbs.render(&system, &mut self.window_canvas)?;
            if fbs.rendering_state == RenderingState::Debug && system.cpu.debug_halted {
                debug.draw(&mut self.window_canvas, &texture_creator, &system)?;
            }
            self.window_canvas.present();
        }
    }
}

fn handle_buttons(e: &Event, system: &mut System) {
    match e {
        Event::KeyDown {
            keycode: Some(k), ..
        } => {
            if let Some(b) = keycode_to_button(k) {
                system.cpu.mmu.input.activate_button(b);
                system.cpu.request_p1_int();
            }
        }
        Event::KeyUp {
            keycode: Some(k), ..
        } => {
            if let Some(b) = keycode_to_button(k) {
                system.cpu.mmu.input.deactivate_button(b);
            }
        }

        _ => {}
    }
}

fn keycode_to_button(keycode: &Keycode) -> Option<Button> {
    match keycode {
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::A => Some(Button::Select),
        Keycode::S => Some(Button::Start),
        _ => None,
    }
}