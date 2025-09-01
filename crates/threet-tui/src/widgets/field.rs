use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;

use crate::conditional_build;

/// since the `Field` type have a lot of initial parameters
/// it is easier to create a `Field` with the builder pattern
#[derive(Default)]
pub struct FieldBuilder {
    buffer: String,
    kind: FieldKind,
    style: FieldStyle,
    min: usize,
    max: usize,
}

impl FieldBuilder {
    #[inline]
    pub fn initial_buffer(mut self, value: String) -> Self {
        self.buffer = value;
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
    pub fn min(mut self, value: usize) -> Self {
        self.min = value;
        self
    }

    #[inline]
    pub fn max(mut self, value: usize) -> Self {
        self.max = value;
        self
    }

    #[inline]
    pub fn build(self) -> Field {
        Field {
            cursor: self.buffer.len(),
            buffer: self.buffer,
            kind: self.kind,
            style: self.style,
            min: self.min,
            max: self.max,
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
    Transparent,
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
    cursor: usize,
    style: FieldStyle,
    min: usize,
    max: usize,
}

impl Field {
    /// returns the kinds of the current field
    #[inline]
    pub fn kind(&self) -> FieldKind {
        self.kind.clone()
    }

    #[inline]
    pub fn valid(&self) -> bool {
        self.min > 0 && self.buffer.len() > self.min
    }

    /// pust a character into the field buffer, the char will be
    /// push in relevense to the cursor position, the returned
    /// bool indicate if the char was actually pushed
    pub fn push_char(&mut self, c: char) -> bool {
        if self.max > 0 && self.max <= self.buffer.len() {
            return false;
        }

        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
        true
    }

    /// return a boolean value indicating if a character was
    /// removed
    pub fn remove_char(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            true
        } else {
            false
        }
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
            label: None,
            focused: false,
            field_style: self.style.clone(),
            placeholder: None,
            max: self.max,
        }
    }
}

/// field widget representation, used in other widget `render` functions
pub struct FieldWidget<'a> {
    content: String,
    field_style: FieldStyle,
    placeholder: Option<&'a str>,
    label: Option<&'a str>,
    focused: bool,
    max: usize,
}

impl<'a> FieldWidget<'a> {
    #[inline]
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }

    #[inline]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
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
            Line::from(self.content.as_str())
        };

        match self.field_style {
            FieldStyle::Outline => {
                let block = conditional_build!(
                    Block::bordered().padding(Padding::left(1)),
                    (self.focused, (style(Style::new().yellow())) else style(Style::new().dark_gray())),
                    (self.label.is_some(), (title_top(self.label.unwrap()))),
                    (
                        self.max > 0,
                        (title_bottom(
                            Line::from(format!(" {}/{} ", self.content.len(), self.max))
                                .right_aligned(),
                        ))
                    )
                );

                Paragraph::new(line).block(block).render(area, buf);
            }
            FieldStyle::Transparent => {
                Paragraph::new(line).render(area, buf);
            }
        }
    }
}
