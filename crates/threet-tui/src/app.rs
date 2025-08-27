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

use crate::Event;
use crate::views::AppView;
use crate::views::AuthenticateView;
use crate::views::View;
use crate::views::ViewKind;
use crate::widgets::StatusWidget;

pub struct App<W: Write> {
    events: Receiver<Event>,
    events_sender: Sender<Event>,
    terminal: Terminal<CrosstermBackend<W>>,
    user: Option<User>,
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
        let view = AuthenticateView::new(app_tx.clone());
        let app = App {
            terminal,
            events: app_rx,
            events_sender: app_tx.clone(),
            view: AppView::Authenticate(view),
            user: None,
        };
        (app, app_tx)
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // initial unconditiond application render
        self.render();

        // disgusting naming, but I just want to make it work for now
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
                Event::Tick => {
                    self.view.on_tick().await;

                    let mut tick_consumed = tick_consumed.lock().await;
                    *tick_consumed = true;
                }
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

                    let mut should_rerender = false;

                    // we iterate over each char because the view
                    // needs to handle each character separatly
                    for c in stdin.chars() {
                        should_rerender = self.view.handle_key(c).await || should_rerender;
                    }

                    if should_rerender {
                        self.render();
                    }
                }
                Event::SetView(view_kind) => self.set_view(view_kind),
                Event::SetUser(user) => self.user = Some(user),
                _ => {}
            };
        }
        Ok(())
    }

    fn render(&mut self) {
        println!("render");
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

    fn set_view(&mut self, view_kind: ViewKind) {
        match view_kind {
            AuthenticateView => {
                todo!()
            }
        }
    }
}
