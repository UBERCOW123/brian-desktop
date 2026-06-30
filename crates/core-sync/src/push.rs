//! Push local changelog to Supabase under user JWT + RLS.

use crate::SyncError;
use core_auth::SupabaseClient;
use core_contracts::{CoreEvent, CoreLink, CoreRecord, SyncOutboxItem};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PushResult {
    pub accepted: bool,
    pub error: Option<String>,
}

pub struct SupabasePushClient {
    supabase: SupabaseClient,
    user_id: String,
    workspace_id: String,
    device_id: String,
}

impl SupabasePushClient {
    pub fn new(
        supabase: SupabaseClient,
        user_id: String,
        workspace_id: String,
        device_id: String,
    ) -> Self {
        Self {
            supabase,
            user_id,
            workspace_id,
            device_id,
        }
    }

    pub async fn push_event(
        &self,
        event: &CoreEvent,
        record: &CoreRecord,
        links: &[CoreLink],
        outbox_item: &SyncOutboxItem,
    ) -> Result<PushResult, SyncError> {
        if !is_uuid(&record.id.to_string()) || !is_uuid(&event.id.to_string()) {
            return Ok(PushResult {
                accepted: false,
                error: Some("Invalid record or event id".into()),
            });
        }

        if self.is_stale_revision(record).await? {
            return Ok(PushResult {
                accepted: false,
                error: Some("stale_revision".into()),
            });
        }

        self.supabase
            .upsert_json(
                "core_records",
                record_row(record, &self.user_id, &self.workspace_id, &self.device_id),
            )
            .await?;

        let event_row = event_row(event, &self.user_id, &self.workspace_id, &self.device_id);
        if let Err(e) = self.supabase.post_json("core_events", event_row, None).await {
            if !is_unique_violation(&e) {
                return Err(e.into());
            }
        }

        self.push_links(links).await?;

        let outbox_ack = serde_json::json!({
            "user_id": self.user_id,
            "workspace_id": self.workspace_id,
            "event_id": event.id.to_string(),
            "status": "synced",
            "attempts": outbox_item.attempts,
            "queued_at": outbox_item.queued_at.to_rfc3339(),
            "last_attempt_at": chrono::Utc::now().to_rfc3339(),
        });
        if let Err(e) = self
            .supabase
            .post_json("client_outbox", outbox_ack, None)
            .await
        {
            if !is_unique_violation(&e) {
                return Err(e.into());
            }
        }

        Ok(PushResult {
            accepted: true,
            error: None,
        })
    }

    async fn is_stale_revision(&self, record: &CoreRecord) -> Result<bool, SyncError> {
        let query = format!("?select=revision&id=eq.{}", record.id);
        let rows = self.supabase.get_json("core_records", &query).await?;
        let remote = rows.as_array().and_then(|a| a.first());
        if let Some(row) = remote {
            let rev = row.get("revision").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            return Ok(rev > record.revision);
        }
        Ok(false)
    }

    async fn push_links(&self, links: &[CoreLink]) -> Result<(), SyncError> {
        for link in links {
            if !is_uuid(&link.id.to_string())
                || !is_uuid(&link.source_record_id.to_string())
                || !is_uuid(&link.target_record_id.to_string())
            {
                continue;
            }
            let row = serde_json::json!({
                "id": link.id.to_string(),
                "user_id": self.user_id,
                "workspace_id": self.workspace_id,
                "source_record_id": link.source_record_id.to_string(),
                "target_record_id": link.target_record_id.to_string(),
                "kind": link.kind.to_string(),
                "metadata_json": parse_json_object(&link.metadata_json),
                "created_at": link.created_at.to_rfc3339(),
            });
            if let Err(e) = self.supabase.upsert_json("core_links", row).await {
                if !is_unique_violation(&e) {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }
}

fn record_row(
    record: &CoreRecord,
    user_id: &str,
    workspace_id: &str,
    device_id: &str,
) -> Value {
    serde_json::json!({
        "id": record.id.to_string(),
        "user_id": user_id,
        "workspace_id": workspace_id,
        "kind": record.kind.to_string(),
        "title": record.title,
        "summary": record.summary,
        "status": record.status,
        "payload_json": parse_json_object(&record.payload_json),
        "source_record_id": record.source_record_id.map(|id| id.to_string()),
        "sort_at": record.sort_at.to_rfc3339(),
        "created_at": record.created_at.to_rfc3339(),
        "updated_at": record.updated_at.to_rfc3339(),
        "deleted_at": record.deleted_at.map(|d| d.to_rfc3339()),
        "revision": record.revision,
        "schema_version": record.schema_version,
        "origin_device_id": device_id,
    })
}

fn event_row(
    event: &CoreEvent,
    user_id: &str,
    workspace_id: &str,
    device_id: &str,
) -> Value {
    serde_json::json!({
        "id": event.id.to_string(),
        "user_id": user_id,
        "workspace_id": workspace_id,
        "record_id": event.record_id.to_string(),
        "kind": event.kind.to_string(),
        "payload_json": parse_json_object(&event.payload_json),
        "occurred_at": event.occurred_at.to_rfc3339(),
        "actor_type": event.actor_type,
        "actor_id": event.actor_id.clone().unwrap_or_else(|| user_id.to_string()),
        "origin_device_id": device_id,
        "base_revision": event.base_revision,
        "created_at": event.created_at.to_rfc3339(),
    })
}

fn parse_json_object(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or(Value::Object(Default::default()))
}

fn is_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok()
}

fn is_unique_violation(err: &core_auth::ClientError) -> bool {
    err.to_string().contains("409") || err.to_string().contains("23505")
}
