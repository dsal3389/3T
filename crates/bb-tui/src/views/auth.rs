use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use super::View;

#[derive(Default)]
pub enum FocuseField {
    #[default]
    Username,
    Password,
}

#[derive(Default)]
pub struct AuthView {
    username: String,
    password: String,
    focuse: FocuseField,
}

impl View for AuthView {
    async fn handle_keypress(&mut self, key: char) -> bool {
        match self.focuse {
            FocuseField::Username => self.username.push(key),
            FocuseField::Password => self.password.push(key),
        };
        true
    }
}

impl Widget for &AuthView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [_, middle, _] = Layout::vertical([Constraint::Ratio(1, 3); 3]).areas(area);
        let [_, middle, _] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(middle);

        let username_block = Block::bordered();
        let password_block = Block::bordered();

        let [username_area, password_area] =
            Layout::vertical([Constraint::Length(3); 2]).areas(middle);

        Paragraph::new(self.username.clone())
            .block(username_block)
            .render(username_area, buf);
        Paragraph::new("*".repeat(self.password.len()))
            .block(password_block)
            .render(password_area, buf);
    }
}
