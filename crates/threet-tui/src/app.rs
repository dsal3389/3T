use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

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
use crate::event::KeyCode;
use crate::notifications::NotificationServiceWidget;
use crate::views::AuthenticateView;
use crate::views::ChatView;
use crate::views::View;
use crate::widgets::StatusWidget;

pub struct App<W: Write> {
    events: Receiver<Event>,
    events_sender: Sender<Event>,

    compositor: Compositor<W>,

    // the app notification service for displaying notifications
    // to the user, this service is independent of the view, currently
    // the max possible notifications at the same time is set to 3
    notifications: NotificationServiceWidget<3>,

    // defines the authenticated user for the current app
    user: Option<User>,
}

impl<W: Write> App<W> {
    /// creates a new application instance that will write to the
    /// given stdout buffer, the returned value includes a channel sender
    /// to insert events to the app from outside
    pub fn new(stdout: W, size: (u16, u16)) -> (Self, Sender<Event>) {
        let (app_tx, app_rx) = channel(1);
        let mut compositor = Compositor::new(stdout, size);

        compositor.split_view(
            Box::new(AuthenticateView::new(app_tx.clone())),
            Layout::Horizontal,
        );
        compositor.split_view(
            Box::new(AuthenticateView::new(app_tx.clone())),
            Layout::Horizontal,
        );

        let app = App {
            events: app_rx,
            events_sender: app_tx.clone(),
            notifications: NotificationServiceWidget::new(app_tx.clone()),
            user: None,
            compositor,
        };
        (app, app_tx)
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // initial unconditiond application render
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
                Event::Resize(size) => {
                    self.compositor.resize(size);
                    self.render();
                }
                Event::Render => self.render(),
                _ => {}
            };
        }
        Ok(())
    }

    #[inline(always)]
    fn render(&mut self) {
        self.compositor.render();
    }
}
