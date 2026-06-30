//! CORE record, event, link, and outbox row models.

use crate::kinds::{CoreEventKind, CoreLinkKind, CoreRecordKind, SyncOutboxStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreRecord {
    pub id: Uuid,
    pub kind: CoreRecordKind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub status: String,
    pub payload_json: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_record_id: Option<Uuid>,
    pub sort_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    pub revision: i32,
    pub schema_version: i32,
    pub origin_device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

impl CoreRecord {
    pub fn new(kind: CoreRecordKind, title: impl Into<String>) -> Self {
        let now = now_utc();
        Self {
            id: Uuid::new_v4(),
            kind,
            title: title.into(),
            summary: None,
            status: "active".into(),
            payload_json: "{}".into(),
            source_record_id: None,
            sort_at: now,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            revision: 1,
            schema_version: 1,
            origin_device_id: "local-device".into(),
            external_id: None,
        }
    }

    pub fn payload(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::from_str(&self.payload_json)
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload_json = payload.to_string();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEvent {
    pub id: Uuid,
    pub record_id: Uuid,
    pub kind: CoreEventKind,
    pub payload_json: String,
    pub occurred_at: DateTime<Utc>,
    pub actor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    pub origin_device_id: String,
    pub base_revision: i32,
    pub created_at: DateTime<Utc>,
}

impl CoreEvent {
    pub fn new(record_id: Uuid, kind: CoreEventKind) -> Self {
        let now = now_utc();
        Self {
            id: Uuid::new_v4(),
            record_id,
            kind,
            payload_json: "{}".into(),
            occurred_at: now,
            actor_type: "system".into(),
            actor_id: None,
            origin_device_id: "local-device".into(),
            base_revision: 0,
            created_at: now,
        }
    }

    pub fn payload(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::from_str(&self.payload_json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLink {
    pub id: Uuid,
    pub source_record_id: Uuid,
    pub target_record_id: Uuid,
    pub kind: CoreLinkKind,
    pub metadata_json: String,
    pub created_at: DateTime<Utc>,
}

impl CoreLink {
    pub fn new(
        source_record_id: Uuid,
        target_record_id: Uuid,
        kind: CoreLinkKind,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_record_id,
            target_record_id,
            kind,
            metadata_json: "{}".into(),
            created_at: now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOutboxItem {
    pub id: Uuid,
    pub event_id: Uuid,
    pub status: SyncOutboxStatus,
    pub attempts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub queued_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_started_at: Option<DateTime<Utc>>,
}

impl SyncOutboxItem {
    pub fn new(event_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_id,
            status: SyncOutboxStatus::Pending,
            attempts: 0,
            last_error: None,
            queued_at: now_utc(),
            last_attempt_at: None,
            processing_started_at: None,
        }
    }
}

pub fn parse_outbox_status(raw: &str) -> Result<SyncOutboxStatus, String> {
    SyncOutboxStatus::from_str(raw).map_err(|e| e.to_string())
}

pub fn parse_record_kind(raw: &str) -> Result<CoreRecordKind, String> {
    CoreRecordKind::from_str(raw).map_err(|e| e.to_string())
}
