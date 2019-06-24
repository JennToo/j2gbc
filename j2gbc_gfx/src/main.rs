use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use j2gbc::{AudioSink, NullSink, System};

mod audio;
mod event;
mod logger;
mod render;
mod save;
mod timer;

fn load_system(
    args: &clap::ArgMatches<'static>,
) -> (System, save::Saver, Arc<audio::CaptureConfig>) {
    let cart_path = args.value_of("rom").unwrap();

    let cart_file = File::open(cart_path).unwrap();

    let (sink, capture_config): (Box<AudioSink>, _) = if !args.is_present("no-audio") {
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

pub fn main() {
    logger::install_logger();
    let args = parse_args();
    let (mut system, mut saver, audio_capture_config) = load_system(&args);

    let events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title(format!("j2gbc -- {}", args.value_of("rom").unwrap()))
        .with_maximized(true);
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window_config, context, &events_loop).unwrap();

    let mut events = event::EventHandler::new(events_loop);
    let mut renderer = render::Renderer::new(gl_window);

    loop {
        if events.handle_events(&mut system, &mut renderer) {
            break;
        }
        system.run_for_duration(&events.elapsed);
        saver.maybe_save(&system);
        renderer.draw(&mut system, &audio_capture_config, events.elapsed);
    }
}
