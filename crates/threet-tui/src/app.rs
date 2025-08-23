use std::io::Write;

use ratatui::prelude::*;
use ratatui::{TerminalOptions, Viewport};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::Event;
use crate::views::{AppView, AuthenticateView, View};
use crate::widgets::StatusWidget;

pub struct App<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
    events: Receiver<Event>,
    view: AppView,
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
        let view = AuthenticateView::default();
        let app = App {
            terminal,
            events: app_rx,
            view: AppView::Authenticate(view),
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
                Event::Stdin(data) => {
                    let stdin = match String::from_utf8(data) {
                        Ok(string) => string,
                        Err(err) => {
                            log::warn!("warning converting to utf-8 {}", err);
                            continue;
                        }
                    };
                    for c in stdin.chars() {
                        self.view.handle_key(c).await;
                    }
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
                let [view_area, status_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                        .areas(frame.area());

                frame.render_widget(&self.view, view_area);
                frame.render_widget(
                    StatusWidget::new(self.view.name(), self.view.mode()),
                    status_area,
                );
            })
            .unwrap();
    }
}
