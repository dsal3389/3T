use std::io::Write;

use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::{TerminalOptions, Viewport};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::Event;

pub struct App<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
    events: Receiver<Event>,
}

impl<W: Write> App<W> {
    /// creates a new application instance that will write to the
    /// given stdout buffer, the returned value includes a channel sender
    /// to insert events to the app from outside
    pub fn new(stdout: W, size: (u16, u16)) -> (Self, Sender<Event>) {
        let (app_tx, app_rx) = channel(1);
        let terminal = Terminal::with_options(
            CrosstermBackend::new(stdout),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, size.0, size.1)),
            },
        )
        .unwrap();
        let app = App {
            terminal,
            events: app_rx,
        };
        (app, app_tx)
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // initial application render
        self.render();

        while let Some(event) = self.events.recv().await {
            match event {
                Event::Render => self.render(),
                Event::Resize((width, height)) => {
                    self.terminal
                        .resize(Rect::new(0, 0, width, height))
                        .unwrap();
                    self.render();
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn render(&mut self) {
        self.terminal
            .draw(|frame| {
                frame.render_widget(Block::bordered(), frame.area());
            })
            .unwrap();
    }
}
