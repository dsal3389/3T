use std::path::{Path, PathBuf};

use async_sqlite::{JournalMode, Pool, PoolBuilder};
use rusqlite::Row;

pub mod models;

use models::Model;

const database_schema: &str = include_str!("../schema.sql");

/// implemented on types that can be created
/// from a database row, usually types that implement
/// this triat are models
trait FromRow: Sized {
    fn from_row(row: &Row) -> rusqlite::Result<Self>;
}

fn preper_select_statement_string<T: Model>() -> String {
    let fields = T::fields().join(",");
    format!("SELECT {} from {}", fields, T::table_name())
}

pub struct Database {
    pool: Pool,
}

impl Database {
    fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// returns a database handler that can execute
    /// sql queries
    pub fn handler(&self) -> DatabaseHandler {
        DatabaseHandler {
            pool: self.pool.clone(),
        }
    }
}

pub struct DatabaseHandler {
    pool: Pool,
}

impl DatabaseHandler {
    pub(crate) async fn fetch_entries<T: Model + FromRow + 'static>(
        self,
    ) -> anyhow::Result<Vec<T>> {
        let entries = self
            .pool
            .conn(|conn| {
                let stmt = preper_select_statement_string::<T>();
                let mut stmt = conn.prepare(&stmt)?;

                stmt.query_map([], |row| T::from_row(row))?
                    .collect::<rusqlite::Result<Vec<T>>>()
            })
            .await?;
        Ok(entries)
    }
}

#[derive(Default)]
pub struct DatabaseBuilder {
    path: Option<PathBuf>,
    num_connections: Option<usize>,
}

impl DatabaseBuilder {
    /// to defines the file system path to the database
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_owned());
        self
    }

    /// to define how many connections can be opened to the remote
    /// database for the internal pool
    pub fn num_connections(mut self, n: usize) -> Self {
        self.num_connections = Some(n);
        self
    }

    pub async fn build(self) -> Database {
        let pool = PoolBuilder::new()
            .path(self.path.expect("cannot create database without a path"))
            .num_conns(self.num_connections.unwrap_or(1))
            .journal_mode(JournalMode::Wal)
            .open()
            .await
            .expect("couldn't connect to database");
        pool.conn(|conn| conn.execute_batch(database_schema))
            .await
            .expect("problem executing database schema");
        Database::new(pool)
    }
}
