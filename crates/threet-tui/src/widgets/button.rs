use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

use crate::conditional_build;

pub struct ButtonWidget<'a> {
    label: &'a str,
    focused: bool,
}

impl<'a> ButtonWidget<'a> {
    #[inline]
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            focused: false,
        }
    }

    #[inline]
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }
}

impl Widget for ButtonWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = conditional_build!(
            Block::bordered(),
            (self.focused, (style(Style::new().yellow())) else style(Style::new().dark_gray()))
        );
        Paragraph::new(self.label)
            .centered()
            .style(Style::new().bold())
            .block(block.border_type(ratatui::widgets::BorderType::Thick))
            .render(area, buf);
    }
}
