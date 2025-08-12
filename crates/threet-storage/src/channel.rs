use rusqlite::Row;

use crate::{DatabaseEntry, DatabaseHandler};

#[derive(Debug)]
pub struct Channel {
    id: i32,
    name: String,
}

impl Channel {
    pub async fn load_all(handler: DatabaseHandler) -> anyhow::Result<Vec<Channel>> {
        handler.fetch_entries().await
    }
}

impl DatabaseEntry for Channel {
    fn table_name() -> &'static str {
        "Channel"
    }

    fn fields() -> Vec<String> {
        vec!["id".to_string(), "name".to_string()]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let id = row.get("id")?;
        let name = row.get("name")?;
        Ok(Channel { id, name })
    }
}
