use std::fs::File;
use std::io::Read;

use j2gbc::system::System;
use log::info;

mod audio;
mod event;
mod logger;
mod render;
mod save;
mod timer;

fn load_system(args: &clap::ArgMatches<'static>) -> (System, save::Saver) {
    let cart_path = args.value_of("rom").unwrap();

    let cart_file = File::open(cart_path.clone()).unwrap();
    let mut c = j2gbc::cart::Cart::load(cart_file).unwrap();
    let save_path = format!("{}.sav", cart_path);
    if let Ok(mut f) = File::open(&save_path) {
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_ok() {
            println!("Loaded save file {}", save_path);
        }
        c.set_sram(buf.as_slice());
    }
    let saver = save::Saver::new(save_path.as_str());

    info!("Loaded cart {}:", cart_path);
    info!("Name: {}", c.name());
    info!("File Size: {} bytes", c.data.len());
    info!("Cart type: {}", c.type_());
    info!("ROM Size: {} bytes", c.rom_size());
    info!("RAM Size: {} bytes", c.ram_size());

    let sink = audio::CpalSink::new().unwrap();

    let cgb_mode = if let Some(m) = args.value_of("mode") {
        m == "cgb"
    } else {
        true
    };

    let mut cpu = j2gbc::cpu::Cpu::new(c, Box::new(sink), cgb_mode);
    cpu.mmu.pedantic = !args.is_present("no-pedantic-mmu");
    (System::new(cpu), saver)
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
        .arg(
            clap::Arg::with_name("rom")
                .help("ROM file to load")
                .required(true),
        ).get_matches()
}

pub fn main() {
    logger::install_logger();
    let args = parse_args();
    let (mut system, mut saver) = load_system(&args);

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
        events.handle_events(&mut system, &mut renderer);
        system.run_for_duration(&events.elapsed);
        saver.maybe_save(&system);
        renderer.draw(&mut system, events.elapsed);
    }
}
