use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use j2gbc::{AudioSink, Button, NullSink, System, SCREEN_SIZE};

mod audio;
mod logger;
mod save;
mod timer;

fn load_system(
    args: &clap::ArgMatches<'static>,
) -> (System, save::Saver, Arc<audio::CaptureConfig>) {
    let cart_path = args.value_of("rom").unwrap();

    let cart_file = File::open(cart_path).unwrap();

    let (sink, capture_config): (Box<AudioSink + Send>, _) = if !args.is_present("no-audio") {
        let sink = audio::CpalSink::new().unwrap();
        let config = sink.get_capture_config();
        (Box::new(sink), config)
    } else {
        (
            Box::new(NullSink),
            Arc::new(audio::CaptureConfig::default()),
        )
    };

    let cgb_mode = if let Some(m) = args.value_of("mode") {
        m == "cgb"
    } else {
        true
    };

    let mut system = System::new(cart_file, sink, cgb_mode).unwrap();
    system.set_mmu_pedantic(!args.is_present("no-pedantic-mmu"));

    let save_path = format!("{}.sav", cart_path);
    if let Ok(mut f) = File::open(&save_path) {
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_ok() {
            println!("Loaded save file {}", save_path);
        }
        system.load_cart_sram(buf.as_slice());
    }
    let saver = save::Saver::new(save_path.as_str());

    (system, saver, capture_config)
}

fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new("j2gbc -- DMG and CGB emulator")
        .author("Jennifer Wilcox <jennifer@nitori.org>")
        .arg(
            clap::Arg::with_name("mode")
                .short("m")
                .long("mode")
                .takes_value(true)
                .help("Operate as a DMG or CGB [default: cgb]")
                .possible_values(&["dmg", "cgb"]),
        )
        .arg(clap::Arg::with_name("no-pedantic-mmu")
            .long("no-pedantic-mmu")
            .help("Disable pedantic MMU. Otherwise by default the MMU will trap if an invalid memory access occurs.")
        )
        .arg(clap::Arg::with_name("no-audio")
             .long("no-audio")
             .help("Disable audio")
        )
        .arg(
            clap::Arg::with_name("rom")
                .help("ROM file to load")
                .required(true),
        ).get_matches()
}

type SystemRef = Rc<RefCell<System>>;

pub fn main() {
    logger::install_logger();
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let args = parse_args();
        let (system, mut saver, _) = load_system(&args);
        let system = Rc::new(RefCell::new(system));

        let window = ApplicationWindow::new(app);
        window.set_title("First GTK+ Program");
        window.set_default_size(600, 480);

        let pixbuf = gdk_pixbuf::Pixbuf::new(
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            SCREEN_SIZE.0 as i32,
            SCREEN_SIZE.1 as i32,
        )
        .unwrap();

        let image = gtk::Image::new_from_pixbuf(Some(&pixbuf));
        window.add(&image);

        let mut dt = timer::DeltaTimer::new();

        let key_press_system = system.clone();
        window.connect_key_press_event(move |_, event| {
            if let Some(button) = keycode_to_button(event.get_keyval()) {
                key_press_system.borrow_mut().activate_button(button);
            }
            Inhibit(false)
        });
        let key_release_system = system.clone();
        window.connect_key_release_event(move |_, event| {
            if let Some(button) = keycode_to_button(event.get_keyval()) {
                key_release_system.borrow_mut().deactivate_button(button);
            }
            Inhibit(false)
        });

        gtk::timeout_add(16, move || {
            saver.maybe_save(&system.borrow());
            system.borrow_mut().run_for_duration(&dt.elapsed());

            let sys = system.borrow_mut();
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

            glib::source::Continue(true)
        });

        window.show_all();
    });

    application.run(&[]);
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
