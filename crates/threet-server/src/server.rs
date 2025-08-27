use std::net::SocketAddr;

use russh::server::Server as SshServerTrait;

use crate::client::Client;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }
}

impl SshServerTrait for Server {
    type Handler = Client;
    fn new_client(&mut self, peer: Option<SocketAddr>) -> Self::Handler {
        Self::Handler::new(peer.unwrap())
    }
}
