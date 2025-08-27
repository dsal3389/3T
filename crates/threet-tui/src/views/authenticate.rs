use anyhow::Context;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding};
use tokio::sync::mpsc::Sender;

use threet_storage::get_database;
use threet_storage::models::User;

use crate::Event;
use crate::utils::get_middle_area;
use crate::widgets::{Field, FieldKind};

use super::{Focuse, FocuseIterator, View, ViewMode};

#[derive(Default, Clone)]
enum FocuseKind {
    #[default]
    UsernameField,
    PasswordField,
    AuthenticateButton,
}

impl FocuseKind {
    #[inline]
    fn is_username_field(&self) -> bool {
        matches!(self, FocuseKind::UsernameField)
    }

    #[inline]
    fn is_password_field(&self) -> bool {
        matches!(self, FocuseKind::PasswordField)
    }

    #[inline]
    fn is_authenticate_button(&self) -> bool {
        matches!(self, FocuseKind::AuthenticateButton)
    }
}

impl FocuseIterator for FocuseKind {
    fn previous(&mut self) -> Self {
        match self {
            FocuseKind::AuthenticateButton => FocuseKind::PasswordField,
            FocuseKind::PasswordField => FocuseKind::UsernameField,
            FocuseKind::UsernameField => FocuseKind::AuthenticateButton,
        }
    }

    fn next(&mut self) -> Self {
        match self {
            FocuseKind::UsernameField => FocuseKind::PasswordField,
            FocuseKind::PasswordField => FocuseKind::AuthenticateButton,
            FocuseKind::AuthenticateButton => FocuseKind::UsernameField,
        }
    }
}

pub struct AuthenticateView {
    app_tx: Sender<Event>,
    focuse: Focuse<FocuseKind>,
    mode: ViewMode,

    username: Field,
    password: Field,
}

impl AuthenticateView {
    pub fn new(app_tx: Sender<Event>) -> Self {
        let username = Field::new(FieldKind::String, Some(String::from("username...")));
        let password = Field::new(FieldKind::Secret, Some(String::from("password...")));
        AuthenticateView {
            app_tx,
            username,
            password,
            mode: ViewMode::default(),
            focuse: Focuse::default(),
        }
    }

    pub async fn try_authenticate(&self) -> anyhow::Result<User> {
        User::by_username_password(get_database(), self.username.value(), self.password.value())
            .await
            .context("username or password incorrect")
    }
}

impl View for AuthenticateView {
    #[inline]
    fn name(&self) -> &str {
        "Authentication"
    }

    #[inline]
    fn mode(&self) -> ViewMode {
        self.mode.clone()
    }

    async fn on_tick(&mut self) {
        println!("tick")
    }

    async fn handle_key(&mut self, key: char) {
        println!("key is {:x}", key as u32);
        match self.mode {
            ViewMode::Normal => match key {
                '\t' | 'j' | 'k' => {
                    self.focuse.next();
                }
                'i' | 'a' => {
                    self.mode = ViewMode::Insert;
                }
                ' ' | '\n' if self.focuse.is_authenticate_button() => {
                    todo!()
                }
                _ => {}
            },
            ViewMode::Insert => {
                // if the key is ESC key
                if key as u32 == 0x1b {
                    self.mode = ViewMode::Normal;
                    return;
                }

                match self.focuse.current() {
                    FocuseKind::UsernameField => self.username.push_char(key),
                    FocuseKind::PasswordField => self.password.push_char(key),
                    _ => {}
                }
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let middle = get_middle_area((60, 13), area);
        let container = Block::bordered()
            .padding(Padding::symmetric(2, 1))
            .title("Authenticate");
        let [username_area, password_area] =
            Layout::vertical([Constraint::Length(3); 2]).areas(container.inner(middle));

        container.render(middle, buf);

        let [username_block, password_block] = if self.focuse.is_username_field() {
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
