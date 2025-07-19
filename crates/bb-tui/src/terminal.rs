use tokio::sync::mpsc::{Sender, Receiver, channel};

pub struct Terminal {
    size: (u16, u16),
}

impl Terminal {
    pub fn new(width: u16, height: u16) -> Terminal {
        let (stdin_tx, stdin_rx) = channel::<u8>(1);
        let (stdout_tx, stdout_rx) = channel::<Vec<u8>>(1);

        Terminal {
            size: (width, height)
        }
    }
}
