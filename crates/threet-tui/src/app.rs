use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use ratatui::TerminalOptions;
use ratatui::Viewport;
use ratatui::prelude::*;

use threet_storage::models::User;

use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;

use crate::compositor::Compositor;
use crate::compositor::Layout;
use crate::event::Event;
use crate::event::Key;
use crate::event::KeyCode;
use crate::views::AuthenticateView;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
}

/// App context is used to passed to the compositor, and the
/// compositor will pass the app context to the currently focused view
pub struct AppContext<'a> {
    dispatcher: Sender<Event>,
    user: Option<&'a User>,
}

pub struct App<W: Write> {
    events: Receiver<Event>,
    events_sender: Sender<Event>,
    terminal: Terminal<CrosstermBackend<W>>,
    mode: Mode,

    compositor: Compositor,

    // defines the authenticated user for the current app
    user: Option<User>,
}

impl<W: Write> App<W> {
    /// creates a new application instance that will write to the
    /// given stdout buffer, the returned value includes a channel sender
    /// to insert events to the app from outside
    pub fn new(stdout: W, size: (u16, u16)) -> (Self, Sender<Event>) {
        let area = Rect::new(0, 0, size.0, size.1);
        let (app_tx, app_rx) = channel(1);
        let terminal = Terminal::with_options(
            CrosstermBackend::new(stdout),
            TerminalOptions {
                viewport: Viewport::Fixed(area),
            },
        )
        .unwrap();

        let mut compositor = Compositor::new(area);

        compositor.split_view(
            Box::new(AuthenticateView::new(app_tx.clone())),
            Layout::Vertical,
        );
        compositor.split_view(
            Box::new(AuthenticateView::new(app_tx.clone())),
            Layout::Vertical,
        );

        let app = App {
            events: app_rx,
            events_sender: app_tx.clone(),
            user: None,
            mode: Mode::Normal,
            compositor,
            terminal,
        };
        (app, app_tx)
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // initial unconditiond application render
        self.terminal.clear().unwrap();
        self.render();

        // a boolean value indicating if the tick event was comsumed, the tick
        // event task won't place more `Tick` events on the channel
        // if the last `Tick` event was not consumed
        // TODO: maybe should this be atomic bool?
        let tick_consumed = Arc::new(Mutex::new(true));

        tokio::spawn({
            let tick_consumed = tick_consumed.clone();
            let app_tx = self.events_sender.clone();

            async move {
                let mut interval_ = interval(Duration::from_millis(350));
                interval_.set_missed_tick_behavior(MissedTickBehavior::Skip);

                // FIXME!: need to kill that loop when app instance
                // is dropped!!!
                loop {
                    interval_.tick().await;

                    let mut tick_consumed = tick_consumed.lock().await;

                    if *tick_consumed {
                        // FIXME: this will break if the app drop
                        app_tx.send(Event::Tick).await.unwrap();
                        *tick_consumed = false;
                    }
                }
            }
        });

        while let Some(event) = self.events.recv().await {
            match event {
                Event::Stdin(bytes) => {
                    let keys: Vec<Key> = bytes
                        .utf8_chunks()
                        .flat_map(|chunk| {
                            chunk
                                .valid()
                                .chars()
                                .inspect(|c| println!("c {} {:x}", c, *c as u32))
                                .map(|c| KeyCode::from_char(c as u32).unwrap())
                                // this is for quick fix until
                                // a good `Key` implementation is made
                                .map(|keycode| keycode.into())
                                .collect::<Vec<Key>>()
                        })
                        .collect();
                    let cx = AppContext {
                        dispatcher: self.events_sender.clone(),
                        user: self.user.as_ref(),
                    };
                    let should_rerender = self.compositor.handle_keys(keys.as_slice(), cx).await;
                    if should_rerender {
                        self.render();
                    }
                }
                Event::Resize(size) => {
                    self.terminal
                        .resize(Rect::new(0, 0, size.0, size.1))
                        .unwrap();
                    // resize the compositor which wil trigger a recalculation
                    // and unconditional render
                    self.compositor.resize(size);
                    self.render();
                }
                Event::Render => self.render(),
                _ => {}
            };
        }
        Ok(())
    }

    #[inline]
    fn render(&mut self) {
        self.terminal
            .draw(|frame| self.compositor.render(frame.buffer_mut()))
            .unwrap();
    }
}
