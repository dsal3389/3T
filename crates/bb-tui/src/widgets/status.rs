use ratatui::prelude::*;


pub struct StatusBar {
}

impl Widget for &StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
       buf.set_style(area, Style::default().bg(Color::LightGreen).fg(Color::Black));
    }
}
