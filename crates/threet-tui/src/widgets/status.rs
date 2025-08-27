use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::text::Span;

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

    fn mode_spans(&self) -> [Span; 2] {
        match self.current_mode {
            ViewMode::Insert => [
                Span::from(" Insert ").style(Style::new().bold().black().on_green()),
                Span::from("\u{e0b0}").style(Style::new().green().on_blue()),
            ],
            ViewMode::Normal => [
                Span::from(" Normal ").style(Style::new().bold().black().on_magenta()),
                Span::from("\u{e0b0}").style(Style::new().magenta().on_blue()),
            ],
        }
    }

    fn view_spans(&self) -> [Span; 2] {
        [
            Span::from(format!(" {} ", self.view_name)).style(Style::new().on_blue()),
            Span::from("\u{e0b0}").style(Style::new().blue()),
        ]
    }
}

impl Widget for StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut spans = Vec::with_capacity(4);
        spans.extend(self.mode_spans());
        spans.extend(self.view_spans());
        Line::from_iter(spans).render(area, buf);
    }
}
