use std::time::Duration;

use threet_storage::models::User;

use crate::notifications::Notification;

#[derive(Debug, Clone)]
pub enum KeyCode {
    Backspace,
    Enter,
    Space,
    Tab,
    Esc,
    Char(char),
}

impl KeyCode {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x1b => Some(KeyCode::Esc),
            0x7f => Some(KeyCode::Backspace),
            0x9 => Some(KeyCode::Tab),
            0x20 => Some(KeyCode::Space),
            0xd | 0xa => Some(KeyCode::Enter),
            _ => char::from_u32(value).map(|c| KeyCode::Char(c)),
        }
    }
}

impl From<KeyCode> for char {
    fn from(value: KeyCode) -> Self {
        let code = match value {
            KeyCode::Backspace => 0x7f as u32,
            KeyCode::Esc => 0x1b as u32,
            KeyCode::Tab => 0x9 as u32,
            KeyCode::Enter => 0xd as u32,
            KeyCode::Space => return ' ',
            KeyCode::Char(c) => return c,
        };

        // safety: this should be safe because `KeyCode` can be created
        // only from a valid char, and we control in the match case
        // what is the ascii `code`
        unsafe { char::from_u32_unchecked(code) }
    }
}

#[derive(Debug)]
pub enum Event {
    Stdin(Vec<u8>),
    Resize((u16, u16)),

    Tick,

    /// push a new notification to the app instance to display
    /// to the user
    Notification((Notification, Duration)),

    /// allow setting the user from outside the application
    /// or from a view
    SetUser(User),
    Render,
}
