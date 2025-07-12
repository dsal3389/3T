
#[derive(Debug)]
pub enum Event {
    /// tells the app to gracefully exit, this event should not be called
    /// by any view, it is called automatically when `CTRL+C` or `ESC` is pressed
    Exit,

    /// triggered by the app tick async task, every X time, this will
    /// let the view refresh its own state and trigger a `ReDraw`
    /// if it has new state it needs to show
    Tick,

    /// the redraw event is triggered by the app on start
    /// after that the app never trigger the `ReDraw` event by it self
    /// it is the responsibility of the app `View` to trigger it if it
    /// needs to redraw itself
    ReDraw,

    /// triggered every time a new key is pressed
    KeyEvent(crossterm::event::KeyEvent)
}
