use std::sync::Arc;
use std::cell::Cell;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::Mutex;

use crate::views::{View, AppView};
use crate::event::Event;

pub struct App {
    app_tx_events: Sender<Event>,
    app_rx_events: Mutex<Receiver<Event>>,

    view: Mutex<Option<AppView>>,

    /// indicator for the `tick` async task to tell if the
    /// `Event::Tick` was consumed and it can schedule another one
    ticked: Mutex<Cell<bool>>
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        Self {
            app_tx_events: tx,
            app_rx_events: Mutex::new(rx),
            view: Mutex::new(None),
            ticked: Mutex::new(Cell::new(false))
        }
    }

    pub async fn run<B: Backend>(self: Arc<Self>, mut terminal: Terminal<B>) -> anyhow::Result<()> {
        self.clone().preper_for_run();

        // set the intialized view
        let _ = self.view
            .lock()
            .await
            .insert(
                AppView::DevicesView(crate::views::DevicesView::new(self.app_tx_events.clone()))
            );

        // the current run loop should be the only owner of this
        // mutex, so this lock will never be released until the `run`
        // method is finished
        let mut app_rx_events = self.app_rx_events.lock().await;

        while let Some(event) = app_rx_events.recv().await {
            let mut view = self.view.lock().await;

            match event {
                Event::Exit => break,
                Event::ReDraw => {
                    terminal.draw(|frame| {
                        frame.render_widget(
                            view.as_ref().unwrap(),
                            frame.area()
                        );
                    })?;
                },
                Event::Tick => {
                    view.as_mut().unwrap().on_tick().await ;
                    self.ticked.lock().await.set(true);
                },

                Event::KeyEvent(event) => {
                    view.as_mut().unwrap().handle_key_event(event).await;
                },
            }
        }
        Ok(())
    }

    fn preper_for_run(self: Arc<Self>) {
        let app_wref = Arc::downgrade(&self);

        tokio::spawn(async move {
            // fire off the first uncoditioned redraw event and tick even
            // the redraw event must be called before the tick event because the view
            // can trigger its own `ReDraw` event (it is not required),
            // if it does we will have 2 redraw events
            // if it doesn't the app will might not be drawn
            match app_wref.upgrade() {
                Some(app) => {
                    app.app_tx_events.send(Event::ReDraw).await.unwrap();
                    app.app_tx_events.send(Event::Tick).await.unwrap();
                },
                None => return
            };

            loop {
                match app_wref.upgrade() {
                    Some(app) => {
                        let ticked = app.ticked.lock().await;

                        // if the `redrawn` value was set `true` it means
                        // we can schedule another `ReDraw` event for the app
                        // else we will sleep again for 250ms
                        if ticked.get() {
                            ticked.set(false);
                            app.app_tx_events.send(Event::Tick).await.unwrap();
                        }
                    },
                    None => break
                };

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let app_wref = Arc::downgrade(&self);

        // create an async task to listen for crossterm
        // events and trigger them with the apropriate event kind
        tokio::spawn(async move {
            loop {
                let Some(app) = app_wref.upgrade() else {
                    break;
                };

                if !crossterm::event::poll(Duration::from_secs(0)).unwrap() {
                    // drop the strong ref because we don't know
                    // how much time it will take to tokio to come back to us
                    // and we don't want to hold a strong ref to `app`
                    std::mem::drop(app);
                    tokio::task::yield_now().await;
                    continue
                }

                match crossterm::event::read().unwrap() {
                    crossterm::event::Event::Key(event) => {
                        let app_event = match event {
                            crossterm::event::KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } | crossterm::event::KeyEvent {
                                code: KeyCode::Esc,
                                ..
                            } => Event::Exit,
                            event => Event::KeyEvent(event)
                        };
                        app.app_tx_events.send(app_event).await.unwrap()
                    }
                    _ => {}
                }
            }
        });
    }
}
