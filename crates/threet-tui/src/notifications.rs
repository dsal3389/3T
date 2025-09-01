use std::collections::VecDeque;
use std::iter::zip;
use std::time::Duration;
use std::time::Instant;

use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use tokio::sync::mpsc::Sender;

use crate::Event;

#[derive(Debug, Clone)]
pub enum NotificationKind {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct Notification {
    title: String,
    content: String,
    kind: NotificationKind,
}

impl Notification {
    #[inline]
    pub fn new(title: String, content: String, kind: NotificationKind) -> Self {
        Self {
            title,
            content,
            kind,
        }
    }

    #[inline]
    pub fn info(title: String, content: String) -> Self {
        Self::new(title, content, NotificationKind::Info)
    }

    #[inline]
    pub fn warning(title: String, content: String) -> Self {
        Self::new(title, content, NotificationKind::Warning)
    }

    #[inline]
    pub fn error(title: String, content: String) -> Self {
        Self::new(title, content, NotificationKind::Error)
    }
}

impl Widget for &Notification {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(Clear, area, buf);
        Paragraph::new(self.content.as_str())
            .block(
                Block::bordered()
                    .border_type(BorderType::Thick)
                    .title_top(self.title.as_str()),
            )
            .render(area, buf);
    }
}

/// notification service is used to push notifications to the user application
/// each app has a single notification service that can receive and display
/// the notifications
///
/// the const `N` indicate the max notifications possible for the service
/// to display at the same time, if more notifications are pushed, the oldest
/// notification will be removed
///
/// the notification service is also a widget which means is can be rendered
/// directly to ratatui by reference
pub struct NotificationServiceWidget<const N: usize> {
    stack: VecDeque<(Notification, Duration, Instant)>,
    app_tx: Sender<Event>,
}

impl<const N: usize> NotificationServiceWidget<N> {
    pub fn new(app_tx: Sender<Event>) -> Self {
        Self {
            stack: VecDeque::with_capacity(N),
            app_tx,
        }
    }

    /// push new notification to the stack, if the stack is at its limit, the oldest
    /// notification will be removed
    pub fn push_notification(&mut self, notification: Notification, duration: Duration) {
        if self.stack.len() >= N {
            self.stack.pop_back();
        }
        self.stack
            .push_front((notification, duration, Instant::now()));
    }

    #[inline]
    pub async fn tick(&mut self) {
        let start = self.stack.len();
        self.stack
            .retain(|(_, duration, instant)| instant.elapsed() < *duration);

        // if the start length is different it is because
        // we removed notifications so we should rerender
        // the screen
        if start != self.stack.len() {
            self.app_tx.send(Event::Render).await.unwrap();
        }
    }

    #[inline]
    pub fn should_render(&self) -> bool {
        self.stack.len() != 0
    }
}

impl<const N: usize> Widget for &NotificationServiceWidget<N> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [_, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Max(84)]).areas(area);
        let constraints = (0..self.stack.len()).map(|_| Constraint::Ratio(1, N as u32));
        let areas = Layout::vertical(constraints).split(right_area);

        for (area, notification) in zip(
            areas.iter(),
            self.stack.iter().map(|(notification, _, _)| notification),
        ) {
            notification.render(*area, buf);
        }
    }
}
