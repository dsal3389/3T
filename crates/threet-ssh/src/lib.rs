use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use rand::rngs::OsRng;
use russh::keys::PrivateKey;
use russh::server::{Config, Server as _};
use tokio::sync::mpsc::Receiver;

mod client;
mod events;
mod server;

pub use client::ClientChannel;
pub use events::{ClientEvent, ServerEvent};

pub type ChannelReceiver = Receiver<ClientEvent>;
pub type ServerReceiver = Receiver<ServerEvent>;

/// loads the ssh server private keys from the given path, if coudln't
/// find a private file at the given path, will create one and save
/// it in the given path for next time
fn load_server_private_key<P>(path: P) -> anyhow::Result<PrivateKey>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let key = if path.exists() {
        PrivateKey::read_openssh_file(path)?
    } else {
        let private = PrivateKey::random(&mut OsRng, russh::keys::ssh_key::Algorithm::Ed25519)?;
        private.write_openssh_file(path, base64ct::LineEnding::default())?;
        private
    };
    Ok(key)
}

pub async fn run_server(addr: SocketAddr) -> anyhow::Result<ServerReceiver> {
    let private_key = load_server_private_key("./key.pem")?;
    let config = Arc::new(Config {
        keys: vec![private_key],
        ..Config::default()
    });
    let (mut server, events) = server::Server::new();

    tokio::spawn(async move {
        let _ = server.run_on_address(config, addr).await;
    });
    Ok(events)
}
