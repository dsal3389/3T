use tokio::sync::mpsc::Sender;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block,
    Paragraph,
};
use ratatui::symbols::border;

use crate::event::Event;
use super::View;

pub struct DevicesView {
    app_tx_event: Sender<Event>,
    count: u32
}

impl DevicesView {
    pub fn new(app_tx_event: Sender<Event>) -> DevicesView {
        DevicesView { app_tx_event, count: 0 }
    }
}

impl View for &mut DevicesView {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent) {
        log::info!("device view key event");
    }

    async fn on_tick(self) {
        log::info!("on  tick called");
        self.count += 1;
        self.app_tx_event.send(Event::ReDraw).await.unwrap();
    }
}

impl Widget for &DevicesView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        let block = Block::bordered()
            .title("devices")
            .border_set(border::ROUNDED);
        Paragraph::new(format!("this is devices view {}", self.count))
            .centered()
            .block(block)
            .render(area, buf);
    }
}
