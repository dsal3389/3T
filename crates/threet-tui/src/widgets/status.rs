use ratatui::prelude::*;
use ratatui::text::{Line, Span};

use crate::views::ViewMode;

pub struct StatusWidget<'a> {
    view_name: &'a str,
    current_mode: ViewMode,
}

impl<'a> StatusWidget<'a> {
    pub fn new(view_name: &'a str, current_mode: ViewMode) -> Self {
        StatusWidget {
            view_name,
            current_mode,
        }
    }
}

impl Widget for StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Line::from_iter([Span::from(self.view_name).style(Style::new().bg(Color::Magenta))])
            .render(area, buf);
    }
}
