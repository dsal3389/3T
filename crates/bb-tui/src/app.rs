use std::io::Write;

use ratatui::Viewport;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Terminal, TerminalOptions};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::events::AppEvent;

pub struct App<W>
where
    W: Write,
{
    event_receiver: UnboundedReceiver<AppEvent>,
    event_sender: UnboundedSender<AppEvent>,
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W> App<W>
where
    W: Write,
{
    pub fn new(stdout: W) -> anyhow::Result<Self> {
        let (event_sender, event_receiver) = unbounded_channel();
        let terminal = {
            let backend = CrosstermBackend::new(stdout);
            let options = TerminalOptions {
                viewport: Viewport::Fixed(Rect::default()),
            };
            Terminal::with_options(backend, options)
        }?;

        Ok(App {
            event_sender,
            event_receiver,
            terminal,
        })
    }

    #[inline]
    pub fn event_sender(&self) -> UnboundedSender<AppEvent> {
        self.event_sender.clone()
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                AppEvent::Render => self.render()?,
                AppEvent::Resize((width, height)) => {
                    let _ = self.terminal.resize(Rect {
                        x: 0,
                        y: 0,
                        width,
                        height,
                    });

                    // when the screen is resized we want to immidatly redraw the screen
                    // use the user and not put it in the app event and wait for
                    // the eventloop to comeback to use to handle that event
                    self.render()?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|frame| {
            let block = Block::bordered();
            let p = Paragraph::new("hello world").block(block);
            frame.render_widget(p, frame.area());
        })?;
        Ok(())
    }
}
