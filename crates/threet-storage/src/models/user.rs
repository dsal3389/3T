use sha2::Digest;

use crate::{Database, FromRow};

pub struct User {
    id: u32,
    username: String,
}

impl User {
    pub async fn by_username_password(
        db: Database,
        username: &str,
        password: &str,
    ) -> Option<User> {
        let username = String::from(username);
        let password = Self::digest_password(password);

        db.pool
            .conn(move |conn| {
                conn.query_one(
                    "SELECT id, username FROM \"User\" WHERE username = (1?) AND password = (2?)",
                    (username, password),
                    |row| Self::from_row(row),
                )
            })
            .await
            .ok()
    }

    // TODO: move this function to a better position
    fn digest_password(password: &str) -> String {
        sha2::Sha256::digest(password)
            .iter()
            .map(|byte| char::from_u32(*byte as u32).unwrap())
            .collect::<String>()
    }
}

impl FromRow for User {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let id = row.get("id")?;
        let username = row.get("username")?;
        Ok(User { id, username })
    }
}
