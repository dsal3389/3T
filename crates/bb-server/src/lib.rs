use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use rand::rngs::OsRng;
use russh::keys::PrivateKey;
use russh::server::{Config, Server};

mod server;
mod ssh;

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

pub async fn main(address: SocketAddr) -> anyhow::Result<()> {
    let private_key = load_server_private_key("./key.pem")?;
    let config = Arc::new(Config {
        keys: vec![private_key],
        ..Config::default()
    });

    server::AppServer::default()
        .run_on_address(config, address)
        .await?;
    Ok(())
}
