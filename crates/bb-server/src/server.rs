use std::net::SocketAddr;

use russh::server::Server;

use crate::ssh::AppClient;

#[derive(Default)]
pub struct AppServer {}

impl Server for AppServer {
    type Handler = AppClient;

    fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self::Handler {
        log::info!(
            "new client connected {}",
            peer_addr
                .map(|addr| addr.to_string())
                .unwrap_or("????:????".to_string())
        );
        AppClient::default()
    }
}
