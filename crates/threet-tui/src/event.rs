pub enum Event {
    Stdin(Vec<u8>),
    Resize((u16, u16)),
    Render,
}
