//! Map between SQLite rows and `core-contracts` models.

use chrono::{DateTime, Utc};
use core_contracts::{
    parse_outbox_status, parse_record_kind, CoreEvent, CoreEventKind, CoreLink, CoreLinkKind,
    CoreRecord, RecordContract, SyncOutboxItem,
};
use rusqlite::Row;
use std::str::FromStr;
use uuid::Uuid;

pub fn record_from_row(row: &Row<'_>) -> rusqlite::Result<CoreRecord> {
    let kind_raw: String = row.get("kind")?;
    let kind = parse_record_kind(&kind_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
    })?;
    Ok(CoreRecord {
        id: parse_uuid(row.get::<_, String>("id")?)?,
        kind,
        title: row.get::<_, String>("title")?,
        summary: row.get("summary")?,
        status: row.get::<_, String>("status")?,
        payload_json: row.get::<_, String>("payload_json")?,
        source_record_id: optional_uuid(row.get::<_, Option<String>>("source_record_id")?)?,
        sort_at: parse_dt(row.get::<_, String>("sort_at")?)?,
        created_at: parse_dt(row.get::<_, String>("created_at")?)?,
        updated_at: parse_dt(row.get::<_, String>("updated_at")?)?,
        deleted_at: optional_dt(row.get::<_, Option<String>>("deleted_at")?)?,
        revision: row.get("revision")?,
        schema_version: row.get("schema_version")?,
        origin_device_id: row.get::<_, String>("origin_device_id")?,
        external_id: row.get("external_id")?,
    })
}

pub fn record_to_params(record: &CoreRecord) -> rusqlite::Result<Vec<rusqlite::types::Value>> {
    let normalized = RecordContract::with_merged_payload(record.clone());
    RecordContract::validate(&normalized).map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
    })?;
    let external_id = normalized
        .payload()
        .ok()
        .and_then(|p| {
            p.get("externalId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .filter(|s| !s.is_empty())
        .or_else(|| normalized.external_id.clone());

    Ok(vec![
        normalized.id.to_string().into(),
        normalized.kind.to_string().into(),
        normalized.title.clone().into(),
        normalized.summary.clone().into(),
        normalized.status.clone().into(),
        normalized.payload_json.clone().into(),
        normalized
            .source_record_id
            .map(|id| id.to_string())
            .into(),
        normalized.sort_at.to_rfc3339().into(),
        normalized.created_at.to_rfc3339().into(),
        normalized.updated_at.to_rfc3339().into(),
        normalized.deleted_at.map(|d| d.to_rfc3339()).into(),
        normalized.revision.into(),
        normalized.schema_version.into(),
        normalized.origin_device_id.clone().into(),
        external_id.into(),
    ])
}

pub fn event_from_row(row: &Row<'_>) -> rusqlite::Result<CoreEvent> {
    let kind_raw: String = row.get("kind")?;
    let kind = CoreEventKind::from_str(&kind_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
    })?;
    Ok(CoreEvent {
        id: parse_uuid(row.get::<_, String>("id")?)?,
        record_id: parse_uuid(row.get::<_, String>("record_id")?)?,
        kind,
        payload_json: row.get::<_, String>("payload_json")?,
        occurred_at: parse_dt(row.get::<_, String>("occurred_at")?)?,
        actor_type: row.get::<_, String>("actor_type")?,
        actor_id: row.get("actor_id")?,
        origin_device_id: row.get::<_, String>("origin_device_id")?,
        base_revision: row.get("base_revision")?,
        created_at: parse_dt(row.get::<_, String>("created_at")?)?,
    })
}

pub fn link_from_row(row: &Row<'_>) -> rusqlite::Result<CoreLink> {
    let kind_raw: String = row.get("kind")?;
    let kind = CoreLinkKind::from_str(&kind_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
    })?;
    Ok(CoreLink {
        id: parse_uuid(row.get::<_, String>("id")?)?,
        source_record_id: parse_uuid(row.get::<_, String>("source_record_id")?)?,
        target_record_id: parse_uuid(row.get::<_, String>("target_record_id")?)?,
        kind,
        metadata_json: row.get::<_, String>("metadata_json")?,
        created_at: parse_dt(row.get::<_, String>("created_at")?)?,
    })
}

pub fn outbox_from_row(row: &Row<'_>) -> rusqlite::Result<SyncOutboxItem> {
    let status_raw: String = row.get("status")?;
    let status = parse_outbox_status(&status_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
    })?;
    Ok(SyncOutboxItem {
        id: parse_uuid(row.get::<_, String>("id")?)?,
        event_id: parse_uuid(row.get::<_, String>("event_id")?)?,
        status,
        attempts: row.get("attempts")?,
        last_error: row.get("last_error")?,
        queued_at: parse_dt(row.get::<_, String>("queued_at")?)?,
        last_attempt_at: optional_dt(row.get::<_, Option<String>>("last_attempt_at")?)?,
        processing_started_at: optional_dt(
            row.get::<_, Option<String>>("processing_started_at")?,
        )?,
    })
}

fn parse_uuid(raw: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
    })
}

fn optional_uuid(raw: Option<String>) -> rusqlite::Result<Option<Uuid>> {
    raw.map(parse_uuid).transpose()
}

fn parse_dt(raw: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&raw)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.into())
        })
}

fn optional_dt(raw: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    raw.map(parse_dt).transpose()
}
