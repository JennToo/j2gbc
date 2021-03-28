use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use cpal_audio::{CaptureConfig, CpalSink};
use frontend_utils::Saver;
use j2gbc::{AudioSink, NullSink, System};

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
