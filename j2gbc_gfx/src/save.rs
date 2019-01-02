use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use j2gbc::System;

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
