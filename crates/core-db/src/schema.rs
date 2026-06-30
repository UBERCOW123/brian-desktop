//! DDL mirrored from CORE mobile `DatabaseHelper` (v14).
//! Do not edit without a matching change in `vendor/core` first.

/// Matches `DatabaseHelper._dbVersion` in mobile.
pub const SCHEMA_VERSION: i32 = 14;

/// Full fresh-install schema: CORE tables + agent harness + search adjuncts.
pub const CREATE_ALL: &str = r#"
CREATE TABLE IF NOT EXISTS core_records (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  summary TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  payload_json TEXT NOT NULL DEFAULT '{}',
  source_record_id TEXT,
  sort_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  revision INTEGER NOT NULL DEFAULT 1,
  schema_version INTEGER NOT NULL DEFAULT 1,
  origin_device_id TEXT NOT NULL DEFAULT 'local-device',
  external_id TEXT
);

CREATE TABLE IF NOT EXISTS core_events (
  id TEXT PRIMARY KEY,
  record_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL DEFAULT '{}',
  occurred_at TEXT NOT NULL,
  actor_type TEXT NOT NULL DEFAULT 'system',
  actor_id TEXT,
  origin_device_id TEXT NOT NULL DEFAULT 'local-device',
  base_revision INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS core_links (
  id TEXT PRIMARY KEY,
  source_record_id TEXT NOT NULL,
  target_record_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_outbox (
  id TEXT PRIMARY KEY,
  event_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  queued_at TEXT NOT NULL,
  last_attempt_at TEXT,
  processing_started_at TEXT
);

CREATE TABLE IF NOT EXISTS inlet_sync_cursors (
  link_id TEXT PRIMARY KEY,
  etag TEXT,
  last_modified TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_turn_log (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  turn_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  user_query TEXT NOT NULL,
  decision_type TEXT,
  tool_name TEXT,
  canonical_args_json TEXT NOT NULL DEFAULT '{}',
  resource_uris_json TEXT NOT NULL DEFAULT '[]',
  result_status TEXT,
  operation_id TEXT,
  verified_values_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS agent_messages (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  role TEXT NOT NULL,
  text TEXT NOT NULL,
  created_at TEXT NOT NULL,
  linked_operation_id TEXT,
  attachments_json TEXT
);

CREATE TABLE IF NOT EXISTS record_embeddings (
  record_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL,
  dimensions INTEGER NOT NULL,
  embedding_blob BLOB NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_pending_operations (
  session_id TEXT NOT NULL,
  operation_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (session_id, operation_id)
);

CREATE TABLE IF NOT EXISTS agent_session_state (
  session_id TEXT PRIMARY KEY,
  memory_json TEXT NOT NULL,
  summary_watermark_message_id TEXT,
  data_epoch INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS embedding_queue (
  record_id TEXT PRIMARY KEY,
  enqueued_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS record_fts USING fts5(
  record_id UNINDEXED,
  kind UNINDEXED,
  text
);

CREATE INDEX IF NOT EXISTS idx_core_records_kind_sort ON core_records(kind, sort_at);
CREATE INDEX IF NOT EXISTS idx_core_records_status ON core_records(status);
CREATE INDEX IF NOT EXISTS idx_core_records_kind_external_id ON core_records(kind, external_id);
CREATE INDEX IF NOT EXISTS idx_core_events_record ON core_events(record_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_core_links_source ON core_links(source_record_id, kind);
CREATE INDEX IF NOT EXISTS idx_core_links_target ON core_links(target_record_id, kind);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_status ON sync_outbox(status, queued_at);
CREATE INDEX IF NOT EXISTS idx_agent_turn_log_session ON agent_turn_log(session_id, turn_index);
CREATE INDEX IF NOT EXISTS idx_agent_messages_session ON agent_messages(session_id, created_at);
"#;
