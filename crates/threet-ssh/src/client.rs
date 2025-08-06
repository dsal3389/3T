use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use russh::server::{Auth, Handle, Handler, Msg, Session};
use russh::{Channel, ChannelId};
use tokio::sync::mpsc::{Sender, channel};

use crate::events::{ClientEvent, ServerEvent};

pub struct Client {
    peer: SocketAddr,
    channel: Option<(Sender<ClientEvent>, Arc<ClientChannel>)>,
    server_sender: Sender<ServerEvent>,
}

impl Client {
    pub fn new(peer: SocketAddr, server_sender: Sender<ServerEvent>) -> Self {
        Client {
            peer,
            server_sender,
            channel: None,
        }
    }
}

impl Handler for Client {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        c: Channel<Msg>,
        session: &mut Session,
    ) -> anyhow::Result<bool> {
        if !self.channel.is_none() {
            anyhow::bail!("only 1 channel per user is allowed");
        }

        let (channel_tx, channel_rx) = channel(1);
        let channel = Arc::new(ClientChannel::new(c.id(), session.handle()));
        self.channel = Some((channel_tx, channel.clone()));

        // send event that a new channel was created with
        // the channel event receiver
        self.server_sender
            .send(ServerEvent::Channel((channel_rx, channel)))
            .await
            .unwrap();
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        id: ChannelId,
        _: &str,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        _: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> anyhow::Result<()> {
        if !self.channel.is_none() {
            anyhow::bail!("only 1 channel per user is allowed");
        }

        let (channel_tx, channel_rx) = channel(1);
        let channel = Arc::new(ClientChannel::new(id, session.handle()));
        self.channel = Some((channel_tx, channel.clone()));

        // send event that a new channel was created with
        // the channel event receiver
        self.server_sender
            .send(ServerEvent::Channel((channel_rx, channel)))
            .await
            .unwrap();
        Ok(())
    }

    async fn data(&mut self, _: ChannelId, data: &[u8], _: &mut Session) -> anyhow::Result<()> {
        match &self.channel {
            Some((sender, _)) => sender
                .send(ClientEvent::Stdin(data.to_vec()))
                .await
                .unwrap(),
            None => {
                todo!()
            }
        };
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

pub struct ClientChannel {
    id: ChannelId,
    session_handle: Handle,
    buffer: Vec<u8>,
}

impl ClientChannel {
    fn new(id: ChannelId, session_handle: Handle) -> ClientChannel {
        ClientChannel {
            id,
            session_handle,
            buffer: Vec::new(),
        }
    }
}

impl Write for ClientChannel {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let channel_id = self.id.clone();
        let buffer = self.buffer.clone();
        let handler = self.session_handle.clone();

        tokio::spawn(async move {
            let _ = handler.data(channel_id, buffer.into()).await;
        });

        self.buffer.clear();
        Ok(())
    }
}
