use ratatui::prelude::*;

mod devices;

pub use devices::DevicesView;

pub trait View: Widget + Sync + Send {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent);
}

pub enum AppView {
    DevicesView(DevicesView)
}

impl View for &AppView {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent) {
        match self {
            AppView::DevicesView(view) => view.handle_key_event(event).await,
        };
    }
}

impl Widget for &AppView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            AppView::DevicesView(view) => view.render(area, buf),
        }
    }
}
