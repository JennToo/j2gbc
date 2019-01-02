use std::sync::Mutex;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record};

pub struct DebugLogger {
    pub log: Mutex<Vec<LogRecord>>,
    started: Instant,
}

pub struct LogRecord {
    pub timestamp: Duration,
    pub message: String,
}

lazy_static! {
    pub static ref DEBUG_LOGGER: DebugLogger = {
        set_max_level(LevelFilter::Debug);
        DebugLogger {
            log: Mutex::new(Vec::new()),
            started: Instant::now(),
        }
    };
}

impl Log for DebugLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let r = record.args().to_string();

        let timestamp = self.started.elapsed();
        let mut l = self.log.lock().unwrap();
        l.push(LogRecord {
            timestamp,
            message: r,
        });
    }

    fn flush(&self) {}
}

pub fn install_logger() {
    set_logger(&*DEBUG_LOGGER).expect("Failed to install logger");
}
