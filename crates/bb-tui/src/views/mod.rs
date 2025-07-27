use ratatui::prelude::*;

mod auth;

pub use auth::AuthView;

pub trait View {
    /// handles a stdin from the user, also return a boolean
    /// value indicating if the input was handled by the
    /// current view, this will indicate to the app to forward the stdin
    /// to the services if stdin wasn't handled by the current view
    async fn handle_keypress(&mut self, key: char) -> bool
    where
        Self: Sized,
    {
        false
    }
}

pub enum AppView {
    Auth(AuthView),
}

impl View for AppView {
    async fn handle_keypress(&mut self, key: char) -> bool {
        match self {
            AppView::Auth(view) => view.handle_keypress(key).await,
        }
    }
}

impl Widget for &AppView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            AppView::Auth(view) => view.render(area, buf),
        }
    }
}
