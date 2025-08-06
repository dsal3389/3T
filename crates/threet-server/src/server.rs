use std::net::SocketAddr;

use crate::client::Client;

pub struct AppServer {}

impl AppServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self, addr: SocketAddr) -> anyhow::Result<()> {
        let mut ssh_events = threet_ssh::run_server(addr).await?;

        // start the ssh server and wait for incoming channel events
        // for each new channel a server client is created
        while let Some(event) = ssh_events.recv().await {
            match event {
                threet_ssh::ServerEvent::Channel((channel_events, channel)) => {
                    let client = Client::new(channel_events, channel);
                    log::info!("client connected");
                }
            }
        }
        Ok(())
    }
}
