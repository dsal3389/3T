use std::ops::Deref;

use ratatui::prelude::*;

mod authenticate;

pub use authenticate::AuthenticateView;

trait FocuseIterator {
    fn previous(&mut self) -> Self;
    fn next(&mut self) -> Self;
}

#[repr(transparent)]
struct Focuse<K: FocuseIterator + Clone>(K);

impl<K> Focuse<K>
where
    K: FocuseIterator + Clone,
{
    #[inline]
    pub fn new(focuse: K) -> Self {
        Self(focuse)
    }

    #[inline]
    pub fn current(&self) -> K {
        self.0.clone()
    }

    #[inline]
    pub fn previous(&mut self) {
        self.0 = self.0.previous();
    }

    #[inline]
    pub fn next(&mut self) {
        self.0 = self.0.next();
    }
}

impl<K> Default for Focuse<K>
where
    K: FocuseIterator + Clone + Default,
{
    fn default() -> Self {
        Self(K::default())
    }
}

impl<K> Deref for Focuse<K>
where
    K: FocuseIterator + Clone,
{
    type Target = K;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub enum ViewMode {
    #[default]
    Normal,
    Insert,
}

pub trait View {
    /// the view name
    fn name(&self) -> &str;

    /// view report what mod its currently in, the view is the type that controlls the
    /// mode at the time, since there is always only 1 view
    fn mode(&self) -> ViewMode;

    /// called when ratatui wants to render the view, the reason the trait is not bounded
    /// to `Widget` instead, is because we want to implement `View` on `T`, but if we want to implement
    /// `Widget` we need `&T` which is a different type
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// called when an input received, the viewer
    /// will decide how to handle it and what to do with it
    async fn handle_key(&mut self, key: char);

    /// called on every tick so the view can update its internal state
    async fn on_tick(&mut self) {}
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

pub enum ViewKind {
    Authenticate,
}

pub enum AppView {
    Authenticate(AuthenticateView),
}

impl View for AppView {
    fn name(&self) -> &str {
        proxy_view_call!(self, name())
    }

    fn mode(&self) -> ViewMode {
        proxy_view_call!(self, mode())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        proxy_view_call!(self, render(area, buf));
    }

    async fn handle_key(&mut self, key: char) {
        proxy_view_call!(self, handle_key(key).await);
    }

    async fn on_tick(&mut self) {
        proxy_view_call!(self, on_tick().await)
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
