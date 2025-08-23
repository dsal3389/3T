use ratatui::prelude::*;

mod authenticate;

pub use authenticate::AuthenticateView;

#[derive(Debug, Default, Clone)]
pub enum ViewMode {
    #[default]
    Normal,
    Insert,
}

pub trait View {
    async fn handle_key(&mut self, key: char);

    /// the view name
    fn name(&self) -> &str;

    /// view report what mod its currently in, the view is the type that controlls the
    /// mode at the time, since there is always only 1 view
    fn mode(&self) -> ViewMode;

    /// called when ratatui wants to render the view, the reason the trait is not bounded
    /// to `Widget` instead, is because we want to implement `View` on `T`, but if we want to implement
    /// `Widget` we need `&T` which is a different type
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// since the `AppView` is just a container of all type of `Views`
/// it gets repeatitive match each type and calling the method
/// on the type
macro_rules! proxy_view_call {
    ($self: expr, $($call: tt)*) => {
        match $self {
            AppView::Authenticate(view) => view.$($call)*,
        }
    };
}

pub enum AppView {
    Authenticate(AuthenticateView),
}

impl View for AppView {
    async fn handle_key(&mut self, key: char) {
        proxy_view_call!(self, handle_key(key).await);
    }

    fn name(&self) -> &str {
        proxy_view_call!(self, name())
    }

    fn mode(&self) -> ViewMode {
        proxy_view_call!(self, mode())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        proxy_view_call!(self, render(area, buf));
    }
}

impl Widget for &AppView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        View::render(self, area, buf);
    }
}
