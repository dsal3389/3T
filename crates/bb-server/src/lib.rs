use std::sync::Arc;
use std::net::SocketAddr;

use rand::rngs::OsRng;
use russh::server::{Server, Config};

mod channel;
mod server;
mod client;

pub async fn main(address: SocketAddr) -> anyhow::Result<()> {
    let config = Arc::new(
        Config {
            keys: vec![
                russh::keys::PrivateKey::random(&mut OsRng, russh::keys::ssh_key::Algorithm::Ed25519).unwrap()
            ],
            ..Config::default()
        }
    );
    let mut server = server::AppServer::default();
    server.run_on_address(config, address).await
        .map_err(|err| err.into())
}
