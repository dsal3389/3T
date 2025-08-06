use std::sync::Arc;

pub struct Client {
    ssh_events: threet_ssh::ChannelReceiver,
    channel: Arc<threet_ssh::ClientChannel>,
}

impl Client {
    pub fn new(
        ssh_events: threet_ssh::ChannelReceiver,
        channel: Arc<threet_ssh::ClientChannel>,
    ) -> Client {
        Client {
            ssh_events,
            channel,
        }
    }
}
