//! Incremental pull of server-originated rows into local SQLite.

use crate::SyncError;
use chrono::{DateTime, Utc};
use core_auth::SupabaseClient;
use core_contracts::{
    parse_record_kind, CoreEvent, CoreEventKind, CoreLink, CoreLinkKind, CoreRecord,
};
use core_db::CoreRepository;
use std::str::FromStr;
use uuid::Uuid;

const PAGE_SIZE: usize = 100;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct PullResult {
    pub records_pulled: u32,
    pub links_pulled: u32,
    pub events_pulled: u32,
    pub errors: Vec<String>,
    pub latest_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct PullBatch {
    pub records: Vec<CoreRecord>,
    pub links: Vec<CoreLink>,
    pub events: Vec<CoreEvent>,
    pub latest_updated_at: Option<DateTime<Utc>>,
}

pub struct SupabasePullClient {
    supabase: SupabaseClient,
    workspace_id: String,
}

impl SupabasePullClient {
    pub fn new(supabase: SupabaseClient, workspace_id: String) -> Self {
        Self {
            supabase,
            workspace_id,
        }
    }

    pub async fn fetch_since(&self, since: Option<DateTime<Utc>>) -> PullBatch {
        let mut batch = PullBatch::default();

        match self
            .fetch_table("core_records", "updated_at", since, record_from_remote)
            .await
        {
            Ok((items, latest)) => {
                batch.records = items;
                batch.latest_updated_at = max_dt(batch.latest_updated_at, latest);
            }
            Err(e) => tracing::warn!("pull records failed: {e}"),
        }

        match self
            .fetch_table("core_links", "created_at", since, link_from_remote)
            .await
        {
            Ok((items, latest)) => {
                batch.links = items;
                batch.latest_updated_at = max_dt(batch.latest_updated_at, latest);
            }
            Err(e) => tracing::warn!("pull links failed: {e}"),
        }

        match self
            .fetch_table("core_events", "created_at", since, event_from_remote)
            .await
        {
            Ok((items, latest)) => {
                batch.events = items;
                batch.latest_updated_at = max_dt(batch.latest_updated_at, latest);
            }
            Err(e) => tracing::warn!("pull events failed: {e}"),
        }

        batch
    }

    async fn fetch_table<T, F>(
        &self,
        table: &str,
        cursor_col: &str,
        since: Option<DateTime<Utc>>,
        parse: F,
    ) -> Result<(Vec<T>, Option<DateTime<Utc>>), SyncError>
    where
        F: Fn(serde_json::Value) -> Result<T, SyncError>,
    {
        let mut cursor = since;
        let mut items = Vec::new();
        let mut latest = None;

        loop {
            let mut query = format!(
                "?select=*&workspace_id=eq.{}&order={cursor_col}.asc&limit={PAGE_SIZE}",
                self.workspace_id
            );
            if let Some(c) = cursor {
                query.push_str(&format!("&{cursor_col}=gt.{}", c.to_rfc3339()));
            }
            let rows_val = self.supabase.get_json(table, &query).await?;
            let rows = rows_val.as_array().cloned().unwrap_or_default();
            if rows.is_empty() {
                break;
            }

            for row in &rows {
                items.push(parse(row.clone())?);
                if let Some(ts) = parse_ts(row.get(cursor_col)) {
                    latest = max_dt(latest, Some(ts));
                }
            }

            if let Some(ts) = rows
                .last()
                .and_then(|r| parse_ts(r.get(cursor_col)))
            {
                cursor = Some(ts);
            }
            if rows.len() < PAGE_SIZE {
                break;
            }
        }

        Ok((items, latest))
    }
}

pub fn apply_pull_batch(repo: &CoreRepository<'_>, batch: PullBatch) -> PullResult {
    let mut result = PullResult {
        latest_updated_at: batch.latest_updated_at,
        ..PullResult::default()
    };

    for record in batch.records {
        match upsert_record(repo, &record) {
            Ok(true) => result.records_pulled += 1,
            Ok(false) => {}
            Err(e) => result.errors.push(format!("record {}: {e}", record.id)),
        }
    }

    for link in batch.links {
        match repo.upsert_link(&link) {
            Ok(()) => result.links_pulled += 1,
            Err(e) => result.errors.push(format!("link {}: {e}", link.id)),
        }
    }

    for event in batch.events {
        match repo.insert_event(&event) {
            Ok(()) => result.events_pulled += 1,
            Err(e) => result.errors.push(format!("event {}: {e}", event.id)),
        }
    }

    result
}

fn upsert_record(repo: &CoreRepository<'_>, record: &CoreRecord) -> Result<bool, SyncError> {
    if let Some(local) = repo.get_record(record.id)? {
        if local.revision > record.revision {
            return Ok(false);
        }
    }
    repo.upsert_record(record)?;
    Ok(true)
}

fn record_from_remote(row: serde_json::Value) -> Result<CoreRecord, SyncError> {
    let id = parse_uuid_field(&row, "id")?;
    let kind = parse_record_kind(row.get("kind").and_then(|v| v.as_str()).unwrap_or("email"))
        .map_err(|e| SyncError::Other { message: e })?;
    let payload = row.get("payload_json").cloned().unwrap_or(serde_json::json!({}));
    let payload_json = if payload.is_string() {
        payload.as_str().unwrap_or("{}").to_string()
    } else {
        payload.to_string()
    };
    Ok(CoreRecord {
        id,
        kind,
        title: row
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        summary: row.get("summary").and_then(|v| v.as_str()).map(String::from),
        status: row
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active")
            .to_string(),
        payload_json,
        source_record_id: row
            .get("source_record_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        sort_at: parse_ts(row.get("sort_at")).unwrap_or_else(Utc::now),
        created_at: parse_ts(row.get("created_at")).unwrap_or_else(Utc::now),
        updated_at: parse_ts(row.get("updated_at")).unwrap_or_else(Utc::now),
        deleted_at: row.get("deleted_at").and_then(|v| v.as_str()).and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        }),
        revision: row.get("revision").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
        schema_version: row
            .get("schema_version")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32,
        origin_device_id: row
            .get("origin_device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("remote")
            .to_string(),
        external_id: row.get("external_id").and_then(|v| v.as_str()).map(String::from),
    })
}

fn link_from_remote(row: serde_json::Value) -> Result<CoreLink, SyncError> {
    Ok(CoreLink {
        id: parse_uuid_field(&row, "id")?,
        source_record_id: parse_uuid_field(&row, "source_record_id")?,
        target_record_id: parse_uuid_field(&row, "target_record_id")?,
        kind: CoreLinkKind::from_str(
            row.get("kind").and_then(|v| v.as_str()).unwrap_or("linked_record"),
        )
        .map_err(|e| SyncError::Other {
            message: e.to_string(),
        })?,
        metadata_json: metadata_to_string(row.get("metadata_json")),
        created_at: parse_ts(row.get("created_at")).unwrap_or_else(Utc::now),
    })
}

fn event_from_remote(row: serde_json::Value) -> Result<CoreEvent, SyncError> {
    Ok(CoreEvent {
        id: parse_uuid_field(&row, "id")?,
        record_id: parse_uuid_field(&row, "record_id")?,
        kind: CoreEventKind::from_str(
            row.get("kind").and_then(|v| v.as_str()).unwrap_or("record_updated"),
        )
        .map_err(|e| SyncError::Other {
            message: e.to_string(),
        })?,
        payload_json: metadata_to_string(row.get("payload_json")),
        occurred_at: parse_ts(row.get("occurred_at")).unwrap_or_else(Utc::now),
        actor_type: row
            .get("actor_type")
            .and_then(|v| v.as_str())
            .unwrap_or("system")
            .to_string(),
        actor_id: row.get("actor_id").and_then(|v| v.as_str()).map(String::from),
        origin_device_id: row
            .get("origin_device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("remote")
            .to_string(),
        base_revision: row
            .get("base_revision")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32,
        created_at: parse_ts(row.get("created_at")).unwrap_or_else(Utc::now),
    })
}

fn metadata_to_string(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(v) if v.is_string() => v.as_str().unwrap_or("{}").to_string(),
        Some(v) => v.to_string(),
        None => "{}".into(),
    }
}

fn parse_uuid_field(row: &serde_json::Value, key: &str) -> Result<Uuid, SyncError> {
    let raw = row
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| SyncError::Other {
            message: format!("missing {key}"),
        })?;
    Uuid::parse_str(raw).map_err(|e| SyncError::Other {
        message: e.to_string(),
    })
}

fn parse_ts(value: Option<&serde_json::Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}

fn max_dt(a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if x > y { x } else { y }),
        (None, y) => y,
        (x, None) => x,
    }
}
