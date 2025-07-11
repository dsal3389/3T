use ratatui::prelude::*;

use super::View;


#[derive(Default)]
pub struct DevicesView {

}

impl View for &DevicesView {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent) {
        log::info!("device view key event");
    }
}

impl Widget for &DevicesView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        log::info!("device view render");
    }
}
