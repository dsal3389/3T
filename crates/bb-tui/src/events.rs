

pub enum AppEvent {
    Render,
    KeyPress(char),
    Resize((u16, u16)),
}
