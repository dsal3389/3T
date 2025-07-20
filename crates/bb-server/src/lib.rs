use std::net::SocketAddr;

pub async fn main(address: SocketAddr) -> anyhow::Result<()> {
    bb_ssh::main(address).await
}
