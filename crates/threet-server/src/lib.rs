use std::net::SocketAddr;

mod client;
mod server;

#[inline]
pub async fn main(addr: SocketAddr) -> anyhow::Result<()> {
    server::AppServer::new().run(addr).await
}
