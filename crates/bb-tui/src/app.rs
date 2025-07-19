use std::io::Write;

use ratatui::Viewport;
use ratatui::widgets::{Paragraph, Block};
use ratatui::layout::Rect;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::events::AppEvent;

pub struct App<W>
where
    W: Write,
{
    event_receiver: UnboundedReceiver<AppEvent>,
    event_sender: UnboundedSender<AppEvent>,
    terminal: Terminal<CrosstermBackend<W>>
}

impl<W> App<W>
where
    W: Write
{
    pub fn new(stdout: W) -> anyhow::Result<Self> {
        let (event_sender, event_receiver) = unbounded_channel();
        let terminal = {
            let backend = CrosstermBackend::new(stdout);
            let options = TerminalOptions {
                viewport: Viewport::Fixed(Rect::default())
            };
            Terminal::with_options(backend, options)
        }?;

        Ok(App { event_sender, event_receiver, terminal })
    }

    pub fn event_sender(&self) -> UnboundedSender<AppEvent> {
        self.event_sender.clone()
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                AppEvent::Render => {
                    self.terminal.draw(|frame| {
                        let block = Block::bordered();
                        let p = Paragraph::new("hello world").block(block);
                        frame.render_widget(p, frame.area());
                    })?;
                }
                AppEvent::Resize((width, height)) => {
                    let _ = self.terminal.resize(Rect {
                        x: 0,
                        y: 0,
                        width,
                        height
                    });
                    // TODO just for testing
                    self.event_sender.send(AppEvent::Render).unwrap();
                }
                _ => unimplemented!()
            }
        }
        Ok(())
    }
}
