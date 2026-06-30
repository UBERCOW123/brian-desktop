//! SQLite schema parity with CORE mobile [`DatabaseHelper`] (schema v14 target).
//!
//! Source of truth: `core/lib/data/database_helper.dart` in `vendor/core`.

mod schema;

pub use schema::SCHEMA_VERSION;

pub use rusqlite::Connection;
use rusqlite::Connection as SqliteConnection;
use std::path::Path;
use thiserror::Error;

pub const DEFAULT_DB_FILENAME: &str = "core_os.db";

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersion { expected: i32, actual: i32 },
}

/// Open (or create) the local CORE database and ensure schema is current.
pub fn open(path: impl AsRef<Path>) -> Result<Connection, DbError> {
    let conn = SqliteConnection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Apply fresh schema — used for new databases and tests.
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if version == 0 {
        conn.execute_batch(schema::CREATE_ALL)?;
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
        tracing::info!(version = SCHEMA_VERSION, "created CORE sqlite schema");
        return Ok(());
    }

    if version != SCHEMA_VERSION {
        // Incremental migrations land in plan Phase 0 — fail loud until implemented.
        return Err(DbError::SchemaVersion {
            expected: SCHEMA_VERSION,
            actual: version,
        });
    }

    Ok(())
}

/// In-memory database for unit tests.
pub fn open_in_memory() -> Result<Connection, DbError> {
    open(":memory:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_core_tables() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='core_records'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
