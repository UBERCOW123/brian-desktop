//! Device-local preferences (mirrors mobile SharedPreferences keys for sync).

use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum LocalPrefKey {
    CloudDeviceId,
    SyncLastPullAt,
}

impl LocalPrefKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CloudDeviceId => "core_cloud_device_id",
            Self::SyncLastPullAt => "core_sync_last_pull_at",
        }
    }
}

pub struct LocalPrefs<'a> {
    conn: &'a Connection,
}

impl<'a> LocalPrefs<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get(&self, key: LocalPrefKey) -> Result<Option<String>, LocalPrefError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM local_prefs WHERE key = ?1")?;
        let mut rows = stmt.query(params![key.as_str()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, key: LocalPrefKey, value: &str) -> Result<(), LocalPrefError> {
        self.conn.execute(
            "INSERT INTO local_prefs (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key.as_str(), value],
        )?;
        Ok(())
    }

    pub fn remove(&self, key: LocalPrefKey) -> Result<(), LocalPrefError> {
        self.conn
            .execute("DELETE FROM local_prefs WHERE key = ?1", params![key.as_str()])?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum LocalPrefError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}
