use std::net::SocketAddr;

use russh::server::{Config, Server as SshServerTrait};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::client::{Client, ClientChannel};
use crate::events::{ClientEvent, ServerEvent};

pub struct Server {
    sender: Sender<ServerEvent>,
}

impl Server {
    pub fn new() -> (Server, Receiver<ServerEvent>) {
        let (tx, rx) = channel(1);
        let server = Server { sender: tx };
        (server, rx)
    }
}

impl SshServerTrait for Server {
    type Handler = Client;
    fn new_client(&mut self, peer: Option<SocketAddr>) -> Self::Handler {
        Self::Handler::new(peer.unwrap(), self.sender.clone())
    }
}
