use russh::ChannelId;
use russh::server::Handle;
use tokio::sync::mpsc::{UnboundedSender, Sender, channel};
use bb_tui::{App, AppEvent};

enum AppChannelState {
    NotReady,
    Ready {
        app_event_sender: UnboundedSender<AppEvent>,
    }
}

pub(crate) struct AppChannel {
    id: ChannelId,
    state: AppChannelState
}

impl AppChannel {
    pub(crate) fn new(id: ChannelId) -> Self {
        Self {
            id,
            state: AppChannelState::NotReady
        }
    }

    /// returns a boolean value indicating if the app channel
    /// is ready and data can be sent to/from it
    #[inline]
    pub fn is_ready(&self) -> bool {
        !matches!(self.state, AppChannelState::NotReady)
    }

    pub async fn create_pty(
        &mut self,
        session_handle: Handle,
        width: u16,
        height: u16,
    ) -> anyhow::Result<()> {
        if self.is_ready() {
            anyhow::bail!("cannot create more then 1 pty per channel");
        }

        let (stdout_tx, mut stdout_rx) = channel::<Vec<u8>>(1);
        let channel_id = self.id.clone();

        tokio::spawn(async move {
            while let Some(data) = stdout_rx.recv().await {
                let _ = session_handle.data(channel_id, data.into()).await
                    .inspect_err(|_| log::error!("couldn't write data to remote peer"));
            }
        });

        let ssh_stdout = AppChannelWriter {
            buffer: Vec::new(),
            sender: stdout_tx
        };

        let app = App::new(ssh_stdout)?;
        let event_sender = app.event_sender();

        // we put the first event in the queue for resize
        // to be ready before we event start the application
        event_sender.send(AppEvent::Resize((width, height))).unwrap();

        // update the channel state to a ready state
        // and get an event sender from the application instance
        // so we can send events from the ssh connection
        self.state = AppChannelState::Ready {
            app_event_sender: event_sender
        };

        // spawn the application to run in the background and
        // listen for incoming events
        tokio::spawn(async move {
            app.run()
               .await
               .inspect_err(|err| log::error!("app run failed with error {}", err))
        });
        Ok(())
    }

    /// sends resize event to the application to handle
    pub fn resize(&self, width: u16, height: u16) -> anyhow::Result<()> {
        match self.state {
            AppChannelState::Ready { ref app_event_sender } => {
                app_event_sender.send(AppEvent::Resize((width, height))).unwrap();
                Ok(())
            },
            AppChannelState::NotReady => Err(anyhow::anyhow!("channel is not ready, pty not created"))
        }
    }
}

/// a simple wrapper around a sender, the receiver will forward
/// the written bytes to the remote channel stdout
struct AppChannelWriter {
    buffer: Vec<u8>,
    sender: Sender<Vec<u8>>
}

impl std::io::Write for AppChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let buffer = self.buffer.clone();
        let sender = self.sender.clone();

        tokio::spawn(async move {
            // TODO: think about something better then this blocking_send
            // and see what are the limitations of this
            sender.send(buffer).await.unwrap();
        });

        self.buffer.clear();
        Ok(())
    }
}
