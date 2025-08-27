use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

pub struct Button {
    clicked: bool,
    label: String,
    disabled: bool,
}

impl Button {
    #[inline]
    pub fn new(label: String, disabled: bool) -> Button {
        Button {
            label,
            disabled,
            clicked: false,
        }
    }

    #[inline]
    pub fn click(&mut self) {
        self.clicked = true;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.clicked = false;
    }

    #[inline]
    pub fn widget(&self) -> ButtonWidget {
        ButtonWidget {
            label: self.label.clone(),
            focused: false,
        }
    }
}

pub struct ButtonWidget {
    label: String,
    focused: bool,
}

impl ButtonWidget {
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }
}

impl Widget for ButtonWidget {
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
