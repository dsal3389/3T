use std::net::SocketAddr;

use russh::server::Server as SshServerTrait;
use threet_storage::Database;

use crate::client::Client;

pub struct Server {
    database: Database,
}

impl Server {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SshServerTrait for Server {
    type Handler = Client;
    fn new_client(&mut self, peer: Option<SocketAddr>) -> Self::Handler {
        Self::Handler::new(peer.unwrap())
    }
}
