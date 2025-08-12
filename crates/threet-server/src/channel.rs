use std::io::Write;

use russh::ChannelId;
use russh::server::Handle;
use tokio::sync::mpsc::Sender;

use threet_tui::Event;

enum ChannelState {
    NotReady,
    Ready { app_tx: Sender<Event> },
}

impl ChannelState {
    #[inline]
    fn is_ready(&self) -> bool {
        matches!(self, ChannelState::Ready { .. })
    }
}

pub struct ClientChannel {
    id: ChannelId,
    session_handle: Handle,
    state: ChannelState,
}

impl ClientChannel {
    pub fn new(id: ChannelId, session_handle: Handle) -> ClientChannel {
        ClientChannel {
            id,
            session_handle,
            state: ChannelState::NotReady,
        }
    }

    pub async fn pty_request(&mut self, size: (u16, u16)) -> anyhow::Result<()> {
        if self.state.is_ready() {
            anyhow::bail!("channel has already an app instance running");
        }
        let stdout = ChannelStdout {
            buffer: Vec::with_capacity(size.0 as usize * size.1 as usize),
            session_handle: self.session_handle.clone(),
            channel_id: self.id.clone(),
        };
        let (app, app_tx) = threet_tui::App::new(stdout, size);

        tokio::spawn(async move {
            app.run().await.unwrap();
        });

        self.state = ChannelState::Ready { app_tx };
        Ok(())
    }

    pub async fn resize(&mut self, dem: (u16, u16)) -> anyhow::Result<()> {
        let ChannelState::Ready { ref app_tx } = self.state else {
            anyhow::bail!("no application was created for this channel, request a pty")
        };
        app_tx.send(Event::Resize(dem)).await.unwrap();
        Ok(())
    }

    pub async fn data(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let ChannelState::Ready { ref app_tx } = self.state else {
            anyhow::bail!("no application was created for this channel, request a pty")
        };
        let event = Event::Stdin(data.to_vec());

        app_tx.send(event).await.unwrap();
        Ok(())
    }
}

pub struct ChannelStdout {
    buffer: Vec<u8>,
    channel_id: ChannelId,
    session_handle: Handle,
}

impl Write for ChannelStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let bytes = self.buffer.clone();
        let handler = self.session_handle.clone();
        let channel_id = self.channel_id.clone();

        tokio::spawn(async move {
            let _ = handler
                .data(channel_id, bytes.into())
                .await
                .inspect_err(|err| {
                    log::warn!("problem sending stdout data to remote client, {:?}", err)
                });
        });

        self.buffer.clear();
        Ok(())
    }
}
