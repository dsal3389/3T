use std::net::SocketAddr;

use russh::Channel;
use russh::ChannelId;
use russh::server::Auth;
use russh::server::Handler;
use russh::server::Msg;
use russh::server::Session;

use crate::channel::ClientChannel;

macro_rules! channel_mut {
    ($maybe_channel: expr) => {
        match $maybe_channel.as_mut() {
            Some(channel) => channel,
            None => anyhow::bail!("no channel was created for the current session"),
        }
    };
}

macro_rules! channel_op_state {
    ($action: expr, $session: expr, $channel_id: expr) => {
        match $action {
            Ok(_) => $session.channel_success($channel_id)?,
            Err(_) => $session.channel_failure($channel_id)?,
        }
    };
}

pub struct Client {
    peer: SocketAddr,
    channel: Option<ClientChannel>,
}

impl Client {
    pub fn new(peer: SocketAddr) -> Self {
        Client {
            peer,
            channel: None,
        }
    }
}

impl Handler for Client {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> anyhow::Result<bool> {
        if self.channel.is_some() {
            anyhow::bail!("only 1 channel per user is allowed");
        }

        let channel = ClientChannel::new(channel.id(), session.handle());
        self.channel = Some(channel);
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> anyhow::Result<()> {
        channel_op_state!(
            channel_mut!(self.channel)
                .pty_request((col_width as u16, row_height as u16))
                .await,
            session,
            channel
        );
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> anyhow::Result<()> {
        channel_op_state!(
            channel_mut!(self.channel).data(data).await,
            session,
            channel
        );
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        session: &mut Session,
    ) -> anyhow::Result<()> {
        channel_op_state!(
            channel_mut!(self.channel)
                .resize((col_width as u16, row_height as u16))
                .await,
            session,
            channel
        );
        Ok(())
    }

    async fn auth_publickey(
        &mut self,
        _: &str,
        _: &russh::keys::ssh_key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _: &str, _: &str) -> anyhow::Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_none(&mut self, _: &str) -> anyhow::Result<Auth> {
        Ok(Auth::Accept)
    }
}
