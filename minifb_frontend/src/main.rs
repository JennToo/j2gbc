use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use std::fs::File;
use std::io::Read;

use cpal_audio::CpalSink;
use frontend_utils::Saver;
use j2gbc::{AudioSink, Button, NullSink, System, SCREEN_SIZE};

fn main() {
    let mut buffer: [u32; SCREEN_SIZE.0 * SCREEN_SIZE.1] = [255; SCREEN_SIZE.0 * SCREEN_SIZE.1];
    let options = WindowOptions {
        borderless: false,
        title: true,
        resize: true,
        scale: Scale::FitScreen,
        scale_mode: ScaleMode::AspectRatioStretch,
        topmost: false,
        transparency: false,
        none: false,
    };

    let mut window = Window::new("j2gbc", SCREEN_SIZE.0, SCREEN_SIZE.1, options).unwrap();
    let (mut system, mut saver) = load_system(&frontend_utils::parse_args());
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut timer = frontend_utils::DeltaTimer::default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        process_input(&window, &mut system);

        system.run_for_duration(&timer.elapsed());

        let framebuffer = system.get_framebuffer();
        for y in 0..SCREEN_SIZE.1 {
            for x in 0..SCREEN_SIZE.0 {
                let pixel = framebuffer.get(x, y);
                buffer[y * SCREEN_SIZE.0 + x] =
                    (pixel[2] as u32) | ((pixel[1] as u32) << 8) | ((pixel[0] as u32) << 16);
            }
        }
        window
            .update_with_buffer(&buffer, SCREEN_SIZE.0, SCREEN_SIZE.1)
            .unwrap();

        saver.maybe_save(&system);
    }
}

fn process_input(window: &Window, system: &mut System) {
    for (button, key) in &[
        (Button::Up, Key::Up),
        (Button::Down, Key::Down),
        (Button::Left, Key::Left),
        (Button::Right, Key::Right),
        (Button::A, Key::Z),
        (Button::B, Key::X),
        (Button::Start, Key::A),
        (Button::Select, Key::S),
    ] {
        if window.is_key_pressed(*key, KeyRepeat::No) {
            system.activate_button(*button);
        }
        if window.is_key_released(*key) {
            system.deactivate_button(*button);
        }
    }
}

fn load_system(args: &clap::ArgMatches<'static>) -> (System, Saver) {
    let cart_path = args.value_of("rom").unwrap();

    let cart_file = File::open(cart_path).unwrap();

    let sink: Box<dyn AudioSink + Send> = if !args.is_present("no-audio") {
        let sink = CpalSink::new().unwrap();
        Box::new(sink)
    } else {
        Box::new(NullSink)
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
    let saver = Saver::new(save_path.as_str());

    (system, saver)
}
