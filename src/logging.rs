use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::fs::{File, OpenOptions};

use log::{Metadata, Record};

pub struct AppLogger(Arc<Mutex<File>>);

impl AppLogger {
    pub fn new(file: File) -> Self {
        Self(Arc::new(Mutex::new(file)))
    }

    pub fn from_path<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(Self::new(file))
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut file = self.0.lock().unwrap();
        let _ = writeln!(file, "[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
