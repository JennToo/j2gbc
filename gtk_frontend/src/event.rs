use enclose::enclose;
use frontend_utils::DeltaTimer;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::Image;
use j2gbc::Button;

use crate::SystemRef;

pub fn install_event_handlers<W>(key_widget: &W, system: &SystemRef)
where
    W: WidgetExt,
{
    key_widget.connect_key_press_event(enclose!((system) move |_, event| {
        if let Some(button) = keycode_to_button(event.get_keyval()) {
            system.borrow_mut().activate_button(button);
        }
        Inhibit(false)
    }));
    key_widget.connect_key_release_event(enclose!((system) move |_, event| {
        if let Some(button) = keycode_to_button(event.get_keyval()) {
            system.borrow_mut().deactivate_button(button);
        }
        Inhibit(false)
    }));
}

fn keycode_to_button(keycode: gdk::keys::Key) -> Option<Button> {
    match keycode {
        gdk::keys::constants::Up => Some(Button::Up),
        gdk::keys::constants::Down => Some(Button::Down),
        gdk::keys::constants::Left => Some(Button::Left),
        gdk::keys::constants::Right => Some(Button::Right),
        gdk::keys::constants::z => Some(Button::A),
        gdk::keys::constants::x => Some(Button::B),
        gdk::keys::constants::a => Some(Button::Select),
        gdk::keys::constants::s => Some(Button::Start),
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
