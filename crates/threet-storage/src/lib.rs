use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use async_sqlite::JournalMode;
use async_sqlite::Pool;
use async_sqlite::PoolBuilder;
use rusqlite::Row;

pub mod models;

use models::Model;

const database_schema: &str = include_str!("../schema.sql");

/// stores the global database instance so everyone can access it
static DATABASE: OnceLock<Database> = OnceLock::new();

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

pub fn set_database(db: Database) {
    DATABASE
        .set(db)
        .expect("couldn't set global database instance");
}

pub fn get_database() -> Database {
    DATABASE
        .get()
        .expect("couldn't get global database instance")
        .clone()
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database")
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
