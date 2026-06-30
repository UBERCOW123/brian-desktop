//! Sync client traits matching `vendor/core/docs/agent/SYNC_STRATEGY.md`.
//!
//! Implementation lands in plan Phase 0 (`sync-engine` todo). This crate defines the
//! contract surface so Tauri commands and tests can compile against stable types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Outbox row status — must match mobile parsing (fail-closed on unknown values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    Pending,
    Processing,
    Synced,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub supabase_url: String,
    pub anon_key: String,
    /// Per-install cursor — not cloned from other devices.
    pub last_pull_at: Option<String>,
    pub workspace_id: Option<String>,
    pub device_id: String,
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("push rejected: stale revision on record {record_id}")]
    StaleRevision { record_id: String },
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("sync: {message}")]
    Other { message: String },
}

/// Push local changelog events from `sync_outbox` to Supabase.
pub trait SyncPushClient: Send + Sync {
    fn drain_outbox(&self) -> Result<PushSummary, SyncError>;
}

/// Pull server-originated rows incrementally by `updated_at` cursor.
pub trait SyncPullClient: Send + Sync {
    fn pull_server_records(&self) -> Result<PullSummary, SyncError>;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PushSummary {
    pub pushed: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PullSummary {
    pub merged: u32,
    pub cursor: Option<String>,
}

/// No-op push client for offline / unsigned sessions (mirrors `DisabledCoreSyncPushClient`).
pub struct DisabledPushClient;

impl SyncPushClient for DisabledPushClient {
    fn drain_outbox(&self) -> Result<PushSummary, SyncError> {
        Ok(PushSummary::default())
    }
}

/// No-op pull client until auth + workspace are wired.
pub struct DisabledPullClient;

impl SyncPullClient for DisabledPullClient {
    fn pull_server_records(&self) -> Result<PullSummary, SyncError> {
        Ok(PullSummary::default())
    }
}
