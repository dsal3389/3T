use std::io::Write;

use ratatui::Viewport;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::events::AppEvent;
use crate::views::{AppView, AuthView, View};

pub struct App<W>
where
    W: Write,
{
    event_receiver: UnboundedReceiver<AppEvent>,
    terminal: Terminal<CrosstermBackend<W>>,
    view: AppView,
}

impl<W> App<W>
where
    W: Write,
{
    /// creates a new application instance and will return the app event sender
    /// to the caller so he can send calls from outside into the application
    pub fn new(stdout: W) -> anyhow::Result<(Self, UnboundedSender<AppEvent>)> {
        let (event_sender, event_receiver) = unbounded_channel();
        let terminal = {
            let backend = CrosstermBackend::new(stdout);
            let options = TerminalOptions {
                viewport: Viewport::Fixed(Rect::default()),
            };
            Terminal::with_options(backend, options)
        }?;

        let app = App {
            event_receiver,
            terminal,
            view: AppView::Auth(AuthView::default()),
        };
        Ok((app, event_sender))
    }

    /// will start to run the application instance main loop, read application
    /// events and act accordily
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
                AppEvent::KeyPress(key) => {
                    // TODO: forward the keypress to the services
                    let _ = self.view.handle_keypress(key).await;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|frame| {
            frame.render_widget(&self.view, frame.area());
        })?;
        Ok(())
    }
}
