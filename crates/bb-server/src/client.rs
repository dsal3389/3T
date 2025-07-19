use russh::{Pty, Channel, ChannelId, Sig};
use russh::server::{Auth, Handler, Msg, Session};
use russh::keys::PublicKey;
use anyhow::Context;

use crate::channel::AppChannel;

#[derive(Default)]
pub(crate) struct AppClient {
    app_channel: Option<AppChannel>
}

impl Handler for AppClient {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _: &mut Session,
    ) -> anyhow::Result<bool> {
        if self.app_channel.is_some() {
            anyhow::bail!("only a single session channel can be created");
        }

        self.app_channel = Some(AppChannel::new(channel.id()));
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _: &mut Session
    ) -> anyhow::Result<()> {
        println!("recv data {}: {:?}", channel, data);
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session
    ) -> anyhow::Result<()> {
        let app_channel = self.app_channel
                              .as_mut()
                              .context("expected `channel_open_session` to already be called")?;
        match app_channel.create_pty(session.handle(), col_width as u16, row_height as u16).await {
            Ok(()) => session.channel_success(channel)?,
            Err(_) => session.channel_failure(channel)?
        };
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        session: &mut Session
    ) -> anyhow::Result<()> {
        let app_channel = self.app_channel
                              .as_ref()
                              .context("expected `channel_open_session` to already be called")?;
        match app_channel.resize(col_width as u16, row_height as u16) {
            Ok(()) => session.channel_success(channel)?,
            Err(_) => session.channel_failure(channel)?,
        };
        Ok(())
    }

    async fn signal(
        &mut self,
        _: ChannelId,
        signal: Sig,
        _: &mut Session
    ) -> anyhow::Result<()> {
        println!("recv sig {:?}", signal);
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _: ChannelId,
        _: &mut Session
    ) -> anyhow::Result<()> {
        println!("eof recved");
        Ok(())
    }

    async fn auth_none(&mut self, _: &str) -> anyhow::Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _: &str, _: &str) -> anyhow::Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> anyhow::Result<Auth> {
        Ok(Auth::Accept)
    }
}
