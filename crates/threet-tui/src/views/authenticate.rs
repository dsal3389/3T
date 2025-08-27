use anyhow::Context;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use tokio::sync::mpsc::Sender;

use threet_storage::get_database;
use threet_storage::models::User;

use crate::Event;
use crate::utils::get_middle_area;
use crate::widgets::Button;
use crate::widgets::Field;
use crate::widgets::FieldBuilder;
use crate::widgets::FieldKind;

use super::Focuse;
use super::FocuseIterator;
use super::View;
use super::ViewMode;

#[derive(Default, Clone)]
enum FocuseArea {
    #[default]
    UsernameField,
    PasswordField,
    AuthenticateButton,
}

impl FocuseArea {
    #[inline]
    fn is_username_field(&self) -> bool {
        matches!(self, FocuseArea::UsernameField)
    }

    #[inline]
    fn is_password_field(&self) -> bool {
        matches!(self, FocuseArea::PasswordField)
    }

    #[inline]
    fn is_authenticate_button(&self) -> bool {
        matches!(self, FocuseArea::AuthenticateButton)
    }
}

impl FocuseIterator for FocuseArea {
    fn previous(&mut self) -> Self {
        match self {
            FocuseArea::AuthenticateButton => FocuseArea::PasswordField,
            FocuseArea::PasswordField => FocuseArea::UsernameField,
            FocuseArea::UsernameField => FocuseArea::AuthenticateButton,
        }
    }

    fn next(&mut self) -> Self {
        match self {
            FocuseArea::UsernameField => FocuseArea::PasswordField,
            FocuseArea::PasswordField => FocuseArea::AuthenticateButton,
            FocuseArea::AuthenticateButton => FocuseArea::UsernameField,
        }
    }
}

pub struct AuthenticateView {
    app_tx: Sender<Event>,
    focuse: Focuse<FocuseArea>,
    mode: ViewMode,

    username: Field,
    password: Field,
    auth_btn: Button,
}

impl AuthenticateView {
    pub fn new(app_tx: Sender<Event>) -> Self {
        let username = FieldBuilder::default()
            .kind(FieldKind::String)
            .placeholder("username...".to_string())
            .build();
        let password = FieldBuilder::default()
            .kind(FieldKind::Secret)
            .placeholder("password...".to_string())
            .build();

        AuthenticateView {
            app_tx,
            username,
            password,
            auth_btn: Button::new("LOGIN".to_string(), false),
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

    async fn handle_key(&mut self, key: char) -> bool {
        // most of the actions in the view require rerendering, so
        // it will be easier to start with a truthy value
        let mut should_rerender = true;

        match self.mode {
            ViewMode::Normal => match key {
                'k' => self.focuse.previous(),
                '\t' | 'j' => self.focuse.next(),
                'i' | 'a' => {
                    self.mode = ViewMode::Insert;
                }
                ' ' | '\n' if self.focuse.is_authenticate_button() => {
                    todo!()
                }
                _ => {
                    should_rerender = false;
                }
            },
            ViewMode::Insert => {
                // if the key is ESC key
                if key as u32 == 0x1b {
                    self.mode = ViewMode::Normal;
                } else {
                    match self.focuse.current() {
                        FocuseArea::UsernameField => self.username.push_char(key),
                        FocuseArea::PasswordField => self.password.push_char(key),
                        _ => {
                            should_rerender = false;
                        }
                    }
                }
            }
        };
        should_rerender
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let middle = get_middle_area((60, 13), area);
        let container = Block::bordered()
            .padding(Padding::symmetric(2, 1))
            .title("Authenticate");
        let [username_area, password_area, btn_area] =
            Layout::vertical([Constraint::Length(3); 3]).areas(container.inner(middle));

        container.render(middle, buf);

        let (username_widget, password_widget, btn_widget) = match self.focuse.current() {
            FocuseArea::UsernameField => (
                self.username.widget().focused(),
                self.password.widget(),
                self.auth_btn.widget(),
            ),
            FocuseArea::PasswordField => (
                self.username.widget(),
                self.password.widget().focused(),
                self.auth_btn.widget(),
            ),
            FocuseArea::AuthenticateButton => (
                self.username.widget(),
                self.password.widget(),
                self.auth_btn.widget().focused(),
            ),
        };
        username_widget.render(username_area, buf);
        password_widget.render(password_area, buf);
        btn_widget.render(btn_area, buf);
    }
}
