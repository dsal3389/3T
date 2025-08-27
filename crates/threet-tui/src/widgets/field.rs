use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;

#[derive(Default)]
pub struct FieldBuilder {
    buffer: String,
    kind: FieldKind,
    placeholder: Option<String>,
    style: FieldStyle,
    min: u16,
    max: u16,
}

impl FieldBuilder {
    #[inline]
    pub fn initial_buffer(mut self, value: String) -> Self {
        self.buffer = value;
        self
    }

    #[inline]
    pub fn placeholder(mut self, value: String) -> Self {
        self.placeholder = Some(value);
        self
    }

    #[inline]
    pub fn kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }

    #[inline]
    pub fn style(mut self, style: FieldStyle) -> Self {
        self.style = style;
        self
    }

    #[inline]
    pub fn min(mut self, value: u16) -> Self {
        self.min = value;
        self
    }

    #[inline]
    pub fn max(mut self, value: u16) -> Self {
        self.max = value;
        self
    }

    #[inline]
    pub fn build(self) -> Field {
        Field {
            buffer: self.buffer,
            kind: self.kind,
            placeholder: self.placeholder,
            style: self.style,
        }
    }
}

/// represent what kind of string the field type expect /// this effect the way the field will repesent its inner buffer
/// to the endpoint user
#[derive(Default, Clone)]
pub enum FieldKind {
    #[default]
    String,
    Secret,
}

#[derive(Clone, Default)]
pub enum FieldStyle {
    #[default]
    Outline,
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
    style: FieldStyle,
}

impl Field {
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
            focused: false,
            field_style: self.style.clone(),
            placeholder: self.placeholder.as_ref().map(|v| v.as_str()),
        }
    }
}

/// field widget representation, used in other widget `render` functions
pub struct FieldWidget<'a> {
    content: String,
    field_style: FieldStyle,
    placeholder: Option<&'a str>,
    focused: bool,
}

impl<'a> FieldWidget<'a> {
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }
}

impl<'a> Widget for FieldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let line = if self.content.is_empty() {
            Line::styled(
                self.placeholder.unwrap_or_default(),
                Style::new().dark_gray().italic(),
            )
        } else {
            Line::from(self.content)
        };

        let p = match self.field_style {
            FieldStyle::Outline => Paragraph::new(line).block(if self.focused {
                Block::bordered()
                    .padding(Padding::left(1))
                    .style(Style::new().yellow())
            } else {
                Block::bordered().padding(Padding::left(1))
            }),
        };
        p.render(area, buf);
    }
}
