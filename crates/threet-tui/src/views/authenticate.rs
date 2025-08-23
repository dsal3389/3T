use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding};

use crate::widgets::{Field, FieldKind};

use super::{View, ViewMode};

#[derive(Default)]
enum FocuseField {
    #[default]
    Username,
    Password,
}

impl Iterator for FocuseField {
    type Item = FocuseField;

    fn next(&mut self) -> Option<Self::Item> {
        if matches!(self, FocuseField::Username) {
            Some(FocuseField::Password)
        } else {
            Some(FocuseField::Username)
        }
    }
}

pub struct AuthenticateView {
    username: Field,
    password: Field,
    focuse: FocuseField,
    mode: ViewMode,
}

impl View for AuthenticateView {
    async fn handle_key(&mut self, key: char) {
        println!("key is {:x}", key as u32);
        match key {
            '\t' => {
                self.focuse = self.focuse.next().unwrap();
            }
            _ => match self.focuse {
                FocuseField::Username => self.username.push_char(key),
                FocuseField::Password => self.password.push_char(key),
            },
        }
    }

    #[inline]
    fn name(&self) -> &str {
        " Authentication "
    }

    #[inline]
    fn mode(&self) -> ViewMode {
        self.mode.clone()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [_, middle, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(60),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, middle, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .areas(middle);

        let container = Block::bordered()
            .title("Authenticate")
            .padding(Padding::symmetric(2, 1));
        let [username_area, password_area] =
            Layout::vertical([Constraint::Length(3); 2]).areas(container.inner(middle));

        container.render(middle, buf);
        self.username
            .widget()
            .block(Block::bordered())
            .render(username_area, buf);
        self.password
            .widget()
            .block(Block::bordered())
            .render(password_area, buf);
    }
}

impl Default for AuthenticateView {
    fn default() -> Self {
        let username = Field::new(FieldKind::String, Some(String::from("username...")));
        let password = Field::new(FieldKind::Secret, Some(String::from("password...")));
        AuthenticateView {
            username,
            password,
            mode: ViewMode::default(),
            focuse: FocuseField::default(),
        }
    }
}
