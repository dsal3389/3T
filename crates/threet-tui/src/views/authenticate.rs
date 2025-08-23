use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding};

use crate::utils::get_middle_area;
use crate::widgets::{Field, FieldKind};

use super::{View, ViewMode};

#[derive(Default)]
enum Focuse {
    #[default]
    UsernameField,
    PasswordField,
}

impl Focuse {
    fn next(&self) -> Self {
        if matches!(self, Focuse::UsernameField) {
            Focuse::PasswordField
        } else {
            Focuse::UsernameField
        }
    }
}

pub struct AuthenticateView {
    username: Field,
    password: Field,
    focuse: Focuse,
    mode: ViewMode,
}

impl View for AuthenticateView {
    async fn handle_key(&mut self, key: char) {
        println!("key is {:x}", key as u32);
        match self.mode {
            ViewMode::Normal => match key {
                '\t' | 'j' | 'k' => {
                    self.focuse = self.focuse.next();
                }
                'i' | 'a' => {
                    self.mode = ViewMode::Insert;
                }
                _ => {}
            },
            ViewMode::Insert => {
                // if the key is ESC key
                if key as u32 == 0x1b {
                    self.mode = ViewMode::Normal;
                    return;
                }

                match self.focuse {
                    Focuse::UsernameField => self.username.push_char(key),
                    Focuse::PasswordField => self.password.push_char(key),
                }
            }
        }
    }

    #[inline]
    fn name(&self) -> &str {
        "Authentication"
    }

    #[inline]
    fn mode(&self) -> ViewMode {
        self.mode.clone()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let middle = get_middle_area((60, 10), area);
        let container = Block::bordered()
            .padding(Padding::symmetric(2, 1))
            .title("Authenticate");
        let [username_area, password_area] =
            Layout::vertical([Constraint::Length(3); 2]).areas(container.inner(middle));

        container.render(middle, buf);

        let [username_block, password_block] = if matches!(self.focuse, Focuse::UsernameField) {
            [
                Block::bordered()
                    .padding(Padding::left(1))
                    .style(Style::new().yellow()),
                Block::bordered().padding(Padding::left(1)),
            ]
        } else {
            [
                Block::bordered().padding(Padding::left(1)),
                Block::bordered()
                    .padding(Padding::left(1))
                    .style(Style::new().yellow()),
            ]
        };

        self.username
            .widget()
            .block(username_block)
            .render(username_area, buf);
        self.password
            .widget()
            .block(password_block)
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
            focuse: Focuse::default(),
        }
    }
}
