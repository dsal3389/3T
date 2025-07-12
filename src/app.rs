use std::sync::Arc;
use std::cell::{Cell, RefCell};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::Mutex;

use crate::views::{View, AppView};
use crate::event::Event;

/// a shared state of the app across different views, since it is gurenteed
/// that the app will not have more than 1 view at a time, this type
/// does not implement `Sync` which allows for "lock free" fields
/// for mutations
pub struct AppState {
    pub app_tx_events: Sender<Event>,
    pub session: RefCell<Option<bluer::Session>>,
}

impl AppState {
    fn new(app_tx_events: Sender<Event>) -> AppState {
        AppState {
            app_tx_events,
            session: RefCell::new(None),
        }
    }
}

/// the app type which runs the main loop for events
/// and triggering views, the app is not shared between threads
/// but views need to have shared state between themselves, for that
/// they have the `AppState` which is shared
pub struct App {
    app_tx_events: Sender<Event>,
    app_rx_events: Receiver<Event>,
    view: Option<AppView>,

    /// indicator for the `tick` async task to tell if the
    /// `Event::Tick` was consumed and it can schedule another one
    ticked: Arc<Mutex<Cell<bool>>>,

    /// a shared app state across different app views
    state: Arc<AppState>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        let state = AppState::new(tx.clone());

        Self {
            app_tx_events: tx,
            app_rx_events: rx,
            view: None,
            state: Arc::new(state),
            ticked: Arc::new(Mutex::new(Cell::new(false)))
        }
    }

    pub async fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> anyhow::Result<()> {
        self.preper_for_run();

        // fire off the first uncoditioned redraw event and tick even
        // the redraw event must be called before the tick event because the view
        // can trigger its own `ReDraw` event (it is not required),
        // if it does we will have 2 redraw events
        // if it doesn't the app will might not be drawn
        self.app_tx_events.send(Event::ReDraw).await.unwrap();
        self.app_tx_events.send(Event::Tick).await.unwrap();

        // set the intialized view
        let _ = self.view
            .insert(
                AppView::DevicesView(crate::views::DevicesView::new(self.state.clone()))
            );

        // the current run loop should be the only owner of this
        // mutex, so this lock will never be released until the `run`
        // method is finished

        while let Some(event) = self.app_rx_events.recv().await {
            match event {
                Event::Exit => break,
                Event::ReDraw => {
                    terminal.draw(|frame| {
                        frame.render_widget(
                            self.view.as_ref().unwrap(),
                            frame.area()
                        );
                    })?;
                },
                Event::Tick => {
                    self.view.as_mut().unwrap().on_tick().await ;
                    self.ticked.lock().await.set(true);
                },

                Event::KeyEvent(event) => {
                    self.view.as_mut().unwrap().handle_key_event(event).await;
                },
            }
        }
        Ok(())
    }

    fn preper_for_run(&self) {
        let app_tx_events = self.app_tx_events.clone();
        let ticked = Arc::downgrade(&self.ticked);

        tokio::spawn(async move {
            loop {
                match ticked.upgrade() {
                    Some(ticked) => {
                        let ticked = ticked.lock().await;

                        // if the `redrawn` value was set `true` it means
                        // we can schedule another `ReDraw` event for the app
                        // else we will sleep again for 250ms
                        if ticked.get() {
                            ticked.set(false);
                            app_tx_events.send(Event::Tick).await.unwrap();
                        }
                    },
                    None => break
                };

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let app_tx_events = self.app_tx_events.clone();

        // create an async task to listen for crossterm
        // events and trigger them with the apropriate event kind
        tokio::spawn(async move {
            loop {
                if !crossterm::event::poll(Duration::from_secs(0)).unwrap() {
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
                        app_tx_events.send(app_event).await.unwrap()
                    }
                    _ => {}
                }
            }
        });
    }
}
