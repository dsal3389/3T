use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

/// represent what kind of string the field type expect
/// this effect the way the field will repesent its inner buffer
/// to the endpoint user
#[derive(Default, Clone)]
pub enum FieldKind {
    #[default]
    String,
    Secret,
}

/// represent a single line text field that can hold
/// a string value
///
/// the type support cursor position for inserting characters between
/// other characters, also will render its character with respect to the
/// given field type
#[derive(Default)]
pub struct Field {
    buffer: String,
    kind: FieldKind,
    placeholder: Option<String>,
}

impl Field {
    #[inline]
    pub fn new(kind: FieldKind, placeholder: Option<String>) -> Field {
        Self::with_value("", kind, placeholder)
    }

    #[inline]
    pub fn with_value(initial_value: &str, kind: FieldKind, placeholder: Option<String>) -> Field {
        Field {
            buffer: String::from(initial_value),
            placeholder,
            kind,
        }
    }

    /// returns the kinds of the current field
    #[inline]
    pub fn kind(&self) -> FieldKind {
        self.kind.clone()
    }

    /// pust a character into the field buffer, the char will be
    /// push in relevense to the cursor position
    #[inline]
    pub fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// returns the field current value
    #[inline]
    pub fn value(&self) -> &str {
        &self.buffer
    }

    /// returns a widget that represent the current field, can
    /// be used in ratatui render
    #[inline]
    pub fn widget(&self) -> FieldWidget {
        let content = match self.kind {
            FieldKind::String => self.buffer.clone(),
            FieldKind::Secret => self.buffer.chars().map(|_| "*").collect::<String>(),
        };
        FieldWidget {
            content,
            block: None,
            placeholder: self.placeholder.as_ref().map(|v| v.as_str()),
        }
    }
}

/// field widget representation, used in other widget `render` functions
pub struct FieldWidget<'a> {
    content: String,
    placeholder: Option<&'a str>,
    block: Option<Block<'a>>,
}

impl<'a> FieldWidget<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for FieldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut line = Line::default();
        let content_span = if self.content.is_empty() {
            Span::from(self.placeholder.unwrap_or_default())
                .style(Style::new().dark_gray().italic())
        } else {
            Span::from(self.content)
        };

        line.push_span(content_span);

        let p = if let Some(block) = self.block {
            Paragraph::new(line).block(block)
        } else {
            Paragraph::new(line)
        };

        p.render(area, buf);
    }
}
