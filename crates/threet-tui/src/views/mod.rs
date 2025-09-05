use std::ops::Deref;

use async_trait::async_trait;
use ratatui::prelude::*;
use tokio::sync::mpsc::Sender;

mod authenticate;
mod chat;

pub use authenticate::AuthenticateView;
pub use chat::ChatView;

use crate::event::Event;
use crate::event::KeyCode;

/// each view has a single focuse area, users can change their focuse
/// usually when they are in Normal mode via TAB | j | k keys, this iterator
/// should yield a different enum variant matching the requested direction
trait FocuseIterator {
    fn previous(&mut self) -> Self;
    fn next(&mut self) -> Self;
}

#[derive(Debug)]
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
        Self::new(K::default())
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

#[derive(Default, Debug, Clone)]
pub enum ViewMode {
    #[default]
    Normal,
    Insert,
}

/// the view is the one who controls the mode, what key binds there are
/// what to display and where
#[async_trait]
pub trait View: Send + 'static {
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
    /// the returned boolean indicate if the app should rerender
    async fn handle_key(&mut self, keycode: KeyCode) -> bool;

    /// called on every tick so the view can update its internal state
    async fn tick(&mut self) {}
}

pub struct ViewContext {
    event_sender: Sender<Event>,
}

pub struct View_ {
    event_sender: Sender<Event>,
}
