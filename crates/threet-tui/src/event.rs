use core::str;
use std::time::Duration;

use threet_storage::models::User;

use crate::notifications::Notification;

#[derive(Debug, Clone, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub enum KeyCode {
    Backspace,
    Enter,
    Space,
    Tab,
    Esc,
    Char(char),
}

impl KeyCode {
    pub fn from_char(value: u32) -> Option<Self> {
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

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Modifier(u32);

impl Modifier {
    const NONE: Modifier = Modifier(0x0);
    const SHIFT: Modifier = Modifier(0x1);
    const CTRL: Modifier = Modifier(0x2);

    #[inline(always)]
    pub fn contains(&self, modifier: Modifier) -> bool {
        (*self & modifier).0 != 0
    }
}

impl std::ops::BitOr for Modifier {
    type Output = Modifier;
    fn bitor(self, rhs: Self) -> Self::Output {
        Modifier(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Modifier {
    type Output = Modifier;
    fn bitand(self, rhs: Self) -> Self::Output {
        Modifier(self.0 & rhs.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Key {
    keycode: KeyCode,
    modifiers: Modifier,
}

impl Key {
    pub fn from_utf8(bytes: &[u8]) -> Key {
        let keycode = match bytes[0] {
            0x1b => KeyCode::Esc,
            0x7f => KeyCode::Backspace,
            0x9 => KeyCode::Tab,
            0x20 => KeyCode::Space,
            0xd | 0xa => KeyCode::Enter,
            _ => {
                let c = str::from_utf8(bytes).unwrap().chars().next().unwrap();
                KeyCode::Char(c)
            }
        };
        Key {
            keycode,
            modifiers: Modifier::NONE,
        }
    }
}

impl From<KeyCode> for Key {
    fn from(value: KeyCode) -> Self {
        Key {
            keycode: value,
            modifiers: Modifier::NONE,
        }
    }
}

impl AsRef<Key> for Key {
    fn as_ref(&self) -> &Key {
        self
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
