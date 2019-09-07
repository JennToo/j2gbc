use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use j2gbc::{AudioSink, NullSink, System};

use crate::{
    audio::{CaptureConfig, CpalSink},
    save::Saver,
};

pub fn load_system(args: &clap::ArgMatches<'static>) -> (System, Saver, Arc<CaptureConfig>) {
    let cart_path = args.value_of("rom").unwrap();

    let cart_file = File::open(cart_path).unwrap();

    let (sink, capture_config): (Box<dyn AudioSink + Send>, _) = if !args.is_present("no-audio") {
        let sink = CpalSink::new().unwrap();
        let config = sink.get_capture_config();
        (Box::new(sink), config)
    } else {
        (Box::new(NullSink), Arc::new(CaptureConfig::default()))
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

    (system, saver, capture_config)
}

pub fn parse_args() -> clap::ArgMatches<'static> {
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
