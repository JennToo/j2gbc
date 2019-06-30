use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::Image;
use j2gbc::Button;

use crate::{timer::DeltaTimer, SystemRef};

pub fn install_event_handlers<W>(key_widget: &W, system: &SystemRef)
where
    W: WidgetExt,
{
    let key_press_system = system.clone();
    key_widget.connect_key_press_event(move |_, event| {
        if let Some(button) = keycode_to_button(event.get_keyval()) {
            key_press_system.borrow_mut().activate_button(button);
        }
        Inhibit(false)
    });
    let key_release_system = system.clone();
    key_widget.connect_key_release_event(move |_, event| {
        if let Some(button) = keycode_to_button(event.get_keyval()) {
            key_release_system.borrow_mut().deactivate_button(button);
        }
        Inhibit(false)
    });
}

fn keycode_to_button(keycode: gdk::enums::key::Key) -> Option<Button> {
    match keycode {
        gdk::enums::key::Up => Some(Button::Up),
        gdk::enums::key::Down => Some(Button::Down),
        gdk::enums::key::Left => Some(Button::Left),
        gdk::enums::key::Right => Some(Button::Right),
        gdk::enums::key::z => Some(Button::A),
        gdk::enums::key::x => Some(Button::B),
        gdk::enums::key::a => Some(Button::Select),
        gdk::enums::key::s => Some(Button::Start),
        _ => None,
    }
}

pub fn run_frame(image: &Image, pixbuf: &Pixbuf, system: &SystemRef, dt: &mut DeltaTimer) {
    let mut sys = system.borrow_mut();
    sys.run_for_duration(&dt.elapsed());

    let fb = sys.get_framebuffer();
    unsafe {
        let count = pixbuf.get_pixels().len();
        std::ptr::copy_nonoverlapping(
            fb.raw().as_ptr() as *const u8,
            pixbuf.get_pixels().as_mut_ptr(),
            count,
        );
    }
    let scaled = pixbuf
        .scale_simple(
            image.get_allocated_width(),
            image.get_allocated_height(),
            gdk_pixbuf::InterpType::Nearest,
        )
        .unwrap();

    image.set_from_pixbuf(Some(&scaled));
}
