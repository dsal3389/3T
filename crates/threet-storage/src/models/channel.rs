use super::Model;
use crate::FromRow;

#[derive(Debug)]
pub struct Channel {
    id: i32,
    name: String,
}

impl FromRow for Channel {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let id = row.get("id")?;
        let name = row.get("name")?;
        Ok(Channel { id, name })
    }
}

impl Model for Channel {
    fn table_name() -> &'static str {
        "Channel"
    }

    fn fields() -> Vec<String> {
        vec!["id".to_string(), "name".to_string()]
    }
}
