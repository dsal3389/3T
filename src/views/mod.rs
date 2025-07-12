use ratatui::prelude::*;

mod devices;

pub use devices::DevicesView;

pub trait View: Sync + Send + Sized {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent);

    async fn on_tick(self);
}

pub enum AppView {
    DevicesView(DevicesView)
}

impl View for &mut AppView {
    async fn handle_key_event(self, event: crossterm::event::KeyEvent) {
        match self {
            AppView::DevicesView(view) => view.handle_key_event(event).await,
        };
    }

    async fn on_tick(self) {
        match self {
            AppView::DevicesView(view) => view.on_tick().await
        }
    }
}

impl Widget for &AppView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            AppView::DevicesView(view) => view.render(area, buf),
        };
    }
}
