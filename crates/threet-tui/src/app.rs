use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::LazyLock;
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

use crate::combo::ComboRecorder;
use crate::combo::ComboRegister;
use crate::compositor::Compositor;
use crate::compositor::Layout;
use crate::event::Event;
use crate::event::Key;
use crate::event::KeyCode;
use crate::views::AuthenticateView;

static NORMAL_COMBOS: LazyLock<ComboRegister> = LazyLock::new(|| {
    let mut combo = ComboRegister::new();
    combo.add([KeyCode::Char('a'); 1], new_vertical);
    combo
});

fn new_vertical<'a>(cx: Context<'a>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        // TODO: a default view
        cx.compositor.split_view(
            Box::new(AuthenticateView::new(cx.dispatcher.clone())),
            Layout::Vertical,
        );
        cx.dispatcher.send(Event::Render).await.unwrap();
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
}

/// the Context is passed to the callback that is returned by the view `handle_keys`, the returned
/// call back need context about the application like the ability to manipulate the app `state`, interact
/// with the view compositor or even send events to the application thus this object contain
/// all the object the callback can interact
///
/// since the app processes each event one after the other, we can pass to the context mutable
/// references.
pub struct Context<'a> {
    pub state: &'a mut AppState,
    pub compositor: &'a mut Compositor,
    pub dispatcher: Sender<Event>,
}

/// contains the application state that is share able, this is mostly used for
/// the `Context` type so other callbacks can take a mutable reference to the `AppState`
/// and change properties that will effect the app overall behaviour
pub struct AppState {
    pub mode: Mode,

    /// vector of the current keys pressed by the user
    /// to match with the combo, this vector is filled when
    /// the app mode is in `Normal` and the vector is emptied
    /// when a `ESC` key is recieved
    pub recorder: ComboRecorder,

    /// defines the authenticated user for the current app
    pub user: Option<User>,
}

pub struct App<W: Write> {
    events: Receiver<Event>,
    events_sender: Sender<Event>,
    terminal: Terminal<CrosstermBackend<W>>,
    compositor: Compositor,
    state: AppState,
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

        let state = AppState {
            mode: Mode::Normal,
            recorder: ComboRecorder::new(),
            user: None,
        };

        let app = App {
            events: app_rx,
            events_sender: app_tx.clone(),
            compositor,
            terminal,
            state,
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
                Event::Stdin(bytes) => self.handle_stdin(bytes).await,
                Event::Resize(mut size) => {
                    self.terminal
                        .resize(Rect::new(0, 0, size.0, size.1))
                        .unwrap();

                    // reduce 1 from the area hight because the app will use that line
                    // to render the status bar
                    size.1 -= 1;

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
    async fn handle_stdin(&mut self, bytes: Vec<u8>) {
        let Some(key) = Key::from_bytes(bytes.as_slice()) else {
            return;
        };

        // if the key was not pushed for some reason, or if the recorder
        // is empty, we have no point processing the record
        if !self.state.recorder.push(key) || self.state.recorder.is_mepty() {
            return;
        }

        let mut app_callback = match self.state.mode {
            Mode::Normal => NORMAL_COMBOS.get(self.state.recorder.as_ref()),
            Mode::Insert => None,
        };

        if app_callback.is_none() {
            app_callback = self
                .compositor
                .current_view_mut()
                .handle_keys(self.state.recorder.as_ref(), self.state.mode)
                .await;
        }

        if let Some(callback) = app_callback {
            let cx = Context {
                state: &mut self.state,
                compositor: &mut self.compositor,
                dispatcher: self.events_sender.clone(),
            };
            callback(cx).await;
            self.state.recorder.clear();
        }
    }

    #[inline]
    fn render(&mut self) {
        self.terminal
            .draw(|frame| self.compositor.render(frame.area(), frame.buffer_mut()))
            .unwrap();
    }
}
