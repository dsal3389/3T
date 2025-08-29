use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

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
        Paragraph::new(self.label)
            .centered()
            .block(if self.focused {
                Block::bordered().style(Style::new().yellow())
            } else {
                Block::bordered()
            })
            .render(area, buf);
    }
}
