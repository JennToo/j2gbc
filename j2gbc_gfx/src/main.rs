use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use j2gbc::{System, SCREEN_SIZE};

mod audio;
mod debugger;
mod event;
mod loader;
mod logger;
mod save;
mod timer;

pub type SystemRef = Rc<RefCell<System>>;

pub fn main() {
    logger::install_logger();
    let application = Application::new(Some("org.nitori.j2gbc"), Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let args = loader::parse_args();
        let (system, mut saver, _) = loader::load_system(&args);
        let system = Rc::new(RefCell::new(system));

        let window = ApplicationWindow::new(app);
        window.set_title("j2gbc");
        window.set_default_size(600, 480);

        let pixbuf = gdk_pixbuf::Pixbuf::new(
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            SCREEN_SIZE.0 as i32,
            SCREEN_SIZE.1 as i32,
        )
        .unwrap();

        let image = gtk::Image::from_pixbuf(Some(&pixbuf));
        window.add(&image);

        let mut dt = timer::DeltaTimer::new();

        event::install_event_handlers(&window, &system);
        debugger::load_debugger(&system);

        glib::timeout_add_local(16, move || {
            saver.maybe_save(&system.borrow());
            event::run_frame(&image, &pixbuf, &system, &mut dt);
            glib::source::Continue(true)
        });

        window.show_all();
    });

    application.run(&[]);
}
