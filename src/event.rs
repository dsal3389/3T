
#[derive(Debug)]
pub enum Event {
    ReDraw,
    KeyEvent(crossterm::event::KeyEvent)
}
