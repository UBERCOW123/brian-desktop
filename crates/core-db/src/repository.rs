//! SQLite repository — port of mobile `CoreRepository` core persistence methods.

use crate::row_mapping::{event_from_row, link_from_row, outbox_from_row, record_from_row, record_to_params};
use chrono::Utc;
use core_contracts::{
    CoreEvent, CoreLink, CoreRecord, CoreRecordKind, SyncOutboxItem, SyncOutboxStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("contract: {0}")]
    Contract(#[from] core_contracts::RecordContractError),
}

pub struct CoreRepository<'a> {
    conn: &'a Connection,
}

impl<'a> CoreRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn connection(&self) -> &'a Connection {
        self.conn
    }

    pub fn get_record(&self, id: Uuid) -> Result<Option<CoreRecord>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, title, summary, status, payload_json, source_record_id,
                    sort_at, created_at, updated_at, deleted_at, revision, schema_version,
                    origin_device_id, external_id
             FROM core_records WHERE id = ?1",
        )?;
        let record = stmt
            .query_row(params![id.to_string()], record_from_row)
            .optional()?;
        Ok(record)
    }

    pub fn upsert_record(&self, record: &CoreRecord) -> Result<(), RepositoryError> {
        let values = record_to_params(record)?;
        self.conn.execute(
            "INSERT INTO core_records (
                id, kind, title, summary, status, payload_json, source_record_id,
                sort_at, created_at, updated_at, deleted_at, revision, schema_version,
                origin_device_id, external_id
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
            ON CONFLICT(id) DO UPDATE SET
                kind=excluded.kind, title=excluded.title, summary=excluded.summary,
                status=excluded.status, payload_json=excluded.payload_json,
                source_record_id=excluded.source_record_id, sort_at=excluded.sort_at,
                created_at=excluded.created_at, updated_at=excluded.updated_at,
                deleted_at=excluded.deleted_at, revision=excluded.revision,
                schema_version=excluded.schema_version, origin_device_id=excluded.origin_device_id,
                external_id=excluded.external_id",
            rusqlite::params_from_iter(values),
        )?;
        Ok(())
    }

    pub fn get_records_by_kinds(
        &self,
        kinds: &[CoreRecordKind],
        include_deleted: bool,
    ) -> Result<Vec<CoreRecord>, RepositoryError> {
        if kinds.is_empty() {
            return Ok(vec![]);
        }
        let placeholders = kinds
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let mut sql = format!(
            "SELECT id, kind, title, summary, status, payload_json, source_record_id,
                    sort_at, created_at, updated_at, deleted_at, revision, schema_version,
                    origin_device_id, external_id
             FROM core_records WHERE kind IN ({placeholders})"
        );
        if !include_deleted {
            sql.push_str(" AND deleted_at IS NULL");
        }
        sql.push_str(" ORDER BY sort_at DESC, created_at DESC");

        let kind_strs: Vec<String> = kinds.iter().map(|k| k.to_string()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(kind_strs), record_from_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_setting_record(&self, key: &str) -> Result<Option<CoreRecord>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, title, summary, status, payload_json, source_record_id,
                    sort_at, created_at, updated_at, deleted_at, revision, schema_version,
                    origin_device_id, external_id
             FROM core_records
             WHERE kind = ?1 AND title = ?2 AND deleted_at IS NULL
             LIMIT 1",
        )?;
        let record = stmt
            .query_row(
                params![CoreRecordKind::AppSetting.to_string(), key],
                record_from_row,
            )
            .optional()?;
        Ok(record)
    }

    pub fn insert_event(&self, event: &CoreEvent) -> Result<(), RepositoryError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO core_events (
                id, record_id, kind, payload_json, occurred_at, actor_type, actor_id,
                origin_device_id, base_revision, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                event.id.to_string(),
                event.record_id.to_string(),
                event.kind.to_string(),
                event.payload_json,
                event.occurred_at.to_rfc3339(),
                event.actor_type,
                event.actor_id,
                event.origin_device_id,
                event.base_revision,
                event.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn upsert_link(&self, link: &CoreLink) -> Result<(), RepositoryError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO core_links (
                id, source_record_id, target_record_id, kind, metadata_json, created_at
            ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                link.id.to_string(),
                link.source_record_id.to_string(),
                link.target_record_id.to_string(),
                link.kind.to_string(),
                link.metadata_json,
                link.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn links_for_record(&self, record_id: Uuid) -> Result<Vec<CoreLink>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_record_id, target_record_id, kind, metadata_json, created_at
             FROM core_links WHERE source_record_id = ?1 OR target_record_id = ?1",
        )?;
        let rows = stmt.query_map(params![record_id.to_string()], link_from_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn insert_outbox_item(&self, item: &SyncOutboxItem) -> Result<(), RepositoryError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sync_outbox (
                id, event_id, status, attempts, last_error, queued_at, last_attempt_at,
                processing_started_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                item.id.to_string(),
                item.event_id.to_string(),
                item.status.to_string(),
                item.attempts,
                item.last_error,
                item.queued_at.to_rfc3339(),
                item.last_attempt_at.map(|d| d.to_rfc3339()),
                item.processing_started_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_outbox_items(
        &self,
        status: Option<SyncOutboxStatus>,
        limit: Option<i32>,
    ) -> Result<Vec<SyncOutboxItem>, RepositoryError> {
        let (sql, status_param) = match status {
            Some(s) => (
                "SELECT id, event_id, status, attempts, last_error, queued_at, last_attempt_at,
                        processing_started_at
                 FROM sync_outbox WHERE status = ?1 ORDER BY queued_at ASC",
                Some(s.to_string()),
            ),
            None => (
                "SELECT id, event_id, status, attempts, last_error, queued_at, last_attempt_at,
                        processing_started_at
                 FROM sync_outbox ORDER BY queued_at ASC",
                None,
            ),
        };
        let sql = if let Some(lim) = limit {
            format!("{sql} LIMIT {lim}")
        } else {
            sql.to_string()
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if let Some(s) = status_param {
            stmt.query_map(params![s], outbox_from_row)?
        } else {
            stmt.query_map([], outbox_from_row)?
        };
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn mark_outbox_processing(&self, id: Uuid) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sync_outbox SET status = ?1, attempts = attempts + 1,
             last_attempt_at = ?2, processing_started_at = ?2 WHERE id = ?3",
            params![SyncOutboxStatus::Processing.to_string(), now, id.to_string()],
        )?;
        Ok(())
    }

    pub fn mark_outbox_synced(&self, id: Uuid) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sync_outbox SET status = ?1, last_error = NULL,
             last_attempt_at = ?2, processing_started_at = NULL WHERE id = ?3",
            params![SyncOutboxStatus::Synced.to_string(), now, id.to_string()],
        )?;
        Ok(())
    }

    pub fn mark_outbox_failed(&self, id: Uuid, error: &str) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sync_outbox SET status = ?1, last_error = ?2,
             last_attempt_at = ?3, processing_started_at = NULL WHERE id = ?4",
            params![
                SyncOutboxStatus::Failed.to_string(),
                error,
                now,
                id.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn get_event_by_id(&self, id: Uuid) -> Result<Option<CoreEvent>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_id, kind, payload_json, occurred_at, actor_type, actor_id,
                    origin_device_id, base_revision, created_at
             FROM core_events WHERE id = ?1",
        )?;
        let event = stmt
            .query_row(params![id.to_string()], event_from_row)
            .optional()?;
        Ok(event)
    }

    pub fn recover_stale_processing_outbox(
        &self,
        stale_after_minutes: i64,
    ) -> Result<usize, RepositoryError> {
        let cutoff = Utc::now() - chrono::Duration::minutes(stale_after_minutes);
        let n = self.conn.execute(
            "UPDATE sync_outbox SET status = ?1, processing_started_at = NULL
             WHERE status = ?2 AND COALESCE(processing_started_at, last_attempt_at) < ?3",
            params![
                SyncOutboxStatus::Pending.to_string(),
                SyncOutboxStatus::Processing.to_string(),
                cutoff.to_rfc3339(),
            ],
        )?;
        Ok(n)
    }
}
