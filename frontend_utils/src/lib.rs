use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use j2gbc::System;

pub struct DeltaTimer {
    last_time: Instant,
}

impl Default for DeltaTimer {
    fn default() -> DeltaTimer {
        DeltaTimer {
            last_time: Instant::now(),
        }
    }
}

impl DeltaTimer {
    pub fn elapsed(&mut self) -> Duration {
        let new_now = Instant::now();
        let d = new_now - self.last_time;
        self.last_time = new_now;
        d
    }
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

pub struct Saver {
    path: PathBuf,
    timer: Instant,
}

impl Saver {
    pub fn new(path: &str) -> Saver {
        Saver {
            path: PathBuf::from(path),
            timer: Instant::now(),
        }
    }

    pub fn maybe_save(&mut self, system: &System) {
        if self.timer.elapsed().as_secs() > 0 {
            self.timer = Instant::now();
            let mut f = File::create(&self.path).unwrap();
            f.write_all(system.read_cart_sram()).unwrap();
        }
    }
}
