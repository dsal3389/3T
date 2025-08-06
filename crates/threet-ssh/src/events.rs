use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub enum ServerEvent {
    Channel((Receiver<ClientEvent>, Arc<crate::client::ClientChannel>)),
}

pub enum ClientEvent {
    Stdin(Vec<u8>),
    Resize((u16, u16)),
}
