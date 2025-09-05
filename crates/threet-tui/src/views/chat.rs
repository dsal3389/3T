use async_trait::async_trait;

use super::View;
use super::ViewMode;
use crate::event::KeyCode;

pub struct ChatView {
    mode: ViewMode,
}

impl ChatView {
    pub fn new() -> Self {
        ChatView {
            mode: ViewMode::default(),
        }
    }
}

#[async_trait]
impl View for ChatView {
    fn name(&self) -> &str {
        "chat"
    }

    fn mode(&self) -> ViewMode {
        self.mode.clone()
    }

    fn render(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {}

    async fn handle_key(&mut self, keycode: KeyCode) -> bool {
        false
    }
}
