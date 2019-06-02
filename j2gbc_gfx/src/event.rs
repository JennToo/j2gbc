use std;
use std::time::Duration;

use glutin::{ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent};
use j2gbc::{Button, System};
use log::info;

use crate::{render::Renderer, timer::DeltaTimer};

pub struct EventHandler {
    events_loop: EventsLoop,
    pub elapsed: Duration,
    timer: DeltaTimer,
}

impl EventHandler {
    pub fn new(events_loop: EventsLoop) -> EventHandler {
        EventHandler {
            events_loop,
            timer: DeltaTimer::new(),
            elapsed: Duration::new(0, 0),
        }
    }

    pub fn handle_events(&mut self, system: &mut System, renderer: &mut Renderer) -> bool {
        let mut should_exit = false;
        self.events_loop.poll_events(|event| {
            renderer.ui.handle_event(&event);
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => should_exit = true,
                    WindowEvent::KeyboardInput { input, .. } => {
                        handle_input(&input, system);
                    }
                    WindowEvent::Resized(size) => {
                        renderer.resize(size);
                    }
                    _ => (),
                }
            }
        });

        self.elapsed = self.timer.elapsed();
        if self.elapsed > Duration::from_millis(17) {
            info!(target: "events", "Slow frame {:?}", self.elapsed);
        }
        should_exit
    }
}

fn handle_input(input: &KeyboardInput, system: &mut System) {
    if let Some(b) = keycode_to_button(input.virtual_keycode.unwrap()) {
        match input.state {
            ElementState::Pressed => {
                system.activate_button(b);
            }
            ElementState::Released => {
                system.deactivate_button(b);
            }
        }
    }
}

fn keycode_to_button(keycode: VirtualKeyCode) -> Option<Button> {
    match keycode {
        VirtualKeyCode::Up => Some(Button::Up),
        VirtualKeyCode::Down => Some(Button::Down),
        VirtualKeyCode::Left => Some(Button::Left),
        VirtualKeyCode::Right => Some(Button::Right),
        VirtualKeyCode::Z => Some(Button::A),
        VirtualKeyCode::X => Some(Button::B),
        VirtualKeyCode::A => Some(Button::Select),
        VirtualKeyCode::S => Some(Button::Start),
        _ => None,
    }
}
