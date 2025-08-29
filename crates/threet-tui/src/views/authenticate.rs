use std::time::Duration;

use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use threet_storage::get_database;
use threet_storage::models::User;

use crate::event::Event;
use crate::event::KeyCode;
use crate::utils::get_middle_area;
use crate::views::ViewKind;
use crate::widgets::ButtonWidget;
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

    // authentication task will contain the task handler for the
    // authentication, it is a separate task to not block the processor
    // and if we see we have a authentication task that is not
    // done, we display a loading screen
    authentication_task: Option<JoinHandle<()>>,
    username: Field,
    password: Field,
}

impl AuthenticateView {
    pub fn new(app_tx: Sender<Event>) -> Self {
        let username = FieldBuilder::default()
            .max(16)
            .kind(FieldKind::String)
            .build();
        let password = FieldBuilder::default().kind(FieldKind::Secret).build();

        AuthenticateView {
            app_tx,
            username,
            password,
            authentication_task: None,
            focuse: Focuse::default(),
            mode: ViewMode::default(),
        }
    }

    #[inline]
    fn start_authentication_task(&mut self) {
        self.authentication_task = Some(tokio::spawn({
            let username = self.username.value().to_string();
            let password = self.password.value().to_string();
            let app_tx = self.app_tx.clone();

            async move {
                let user = User::by_username_password(get_database(), &username, &password).await;
                println!("user is some {}", user.is_some());

                tokio::time::sleep(Duration::from_secs(3)).await;

                if user.is_some() {
                    app_tx.send(Event::SetUser(user.unwrap())).await.unwrap();
                    app_tx.send(Event::SetView(ViewKind::Chat)).await.unwrap();
                } else {
                    app_tx.send(Event::Render).await.unwrap();
                }
            }
        }));
    }

    #[inline]
    fn is_authentication_task_running(&self) -> bool {
        self.authentication_task
            .as_ref()
            .is_some_and(|handle| !handle.is_finished())
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

    async fn handle_key(&mut self, keycode: KeyCode) -> bool {
        // if authentication task is running we should not handle
        // any new key event and we don't need to rerender the screen
        if self.is_authentication_task_running() {
            return false;
        }

        // most of the actions in the view require rerendering, so
        // it will be easier to start with a truthy value
        let mut should_rerender = true;

        match self.mode {
            ViewMode::Normal => match keycode {
                KeyCode::Char('k') => self.focuse.previous(),
                KeyCode::Tab | KeyCode::Char('j') => self.focuse.next(),
                KeyCode::Char('i') | KeyCode::Char('a') => {
                    self.mode = ViewMode::Insert;
                }
                KeyCode::Space | KeyCode::Enter if self.focuse.is_authenticate_button() => {
                    self.start_authentication_task()
                }
                _ => {
                    should_rerender = false;
                }
            },
            ViewMode::Insert => match keycode {
                KeyCode::Esc => {
                    self.mode = ViewMode::Normal;
                }
                KeyCode::Char(..) | KeyCode::Space => match self.focuse.current() {
                    FocuseArea::UsernameField => self.username.push_char(keycode.into()),
                    FocuseArea::PasswordField => self.password.push_char(keycode.into()),
                    _ => {
                        should_rerender = false;
                    }
                },
                KeyCode::Backspace => match self.focuse.current() {
                    FocuseArea::UsernameField => self.username.remove_char(),
                    FocuseArea::PasswordField => self.password.remove_char(),
                    _ => {
                        should_rerender = false;
                    }
                },
                _ => {
                    should_rerender = false;
                }
            },
        };
        should_rerender
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        // if the authentication task is not done
        // we should show a loading page
        if self.is_authentication_task_running() {
            Paragraph::new("loading").centered().render(area, buf);
            return;
        }

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
                ButtonWidget::new("LOGIN"),
            ),
            FocuseArea::PasswordField => (
                self.username.widget(),
                self.password.widget().focused(),
                ButtonWidget::new("LOGIN"),
            ),
            FocuseArea::AuthenticateButton => (
                self.username.widget(),
                self.password.widget(),
                ButtonWidget::new("LOGIN").focused(),
            ),
        };

        username_widget
            .placeholder("username...")
            .render(username_area, buf);
        password_widget
            .placeholder("password...")
            .render(password_area, buf);
        btn_widget.render(btn_area, buf);
    }
}
