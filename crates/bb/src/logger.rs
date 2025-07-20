use log::{Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{Sender, channel};

pub(crate) struct Logger {
    sender: Sender<String>,
}

impl Logger {
    pub(crate) fn new(mut file: File) -> Self {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                println!("{}", message);
                let _ = writeln!(file, "{}", message);
            }
        });
        Logger { sender: tx }
    }

    pub(crate) fn from_path<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        Ok(Self::new(file))
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.sender
            .send(format!("[{}] {}", record.level(), record.args()))
            .unwrap()
    }

    fn flush(&self) {}
}
