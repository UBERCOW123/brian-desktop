//! Record contract validation — port of `CoreRecordContract` in mobile.

use crate::kinds::CoreRecordKind;
use crate::models::CoreRecord;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("record contract violation: {0}")]
    Validation(String),
    #[error("multiple contract violations: {0:?}")]
    ValidationList(Vec<String>),
}

pub struct RecordContract;

impl RecordContract {
    pub fn validate(record: &CoreRecord) -> Result<(), ContractError> {
        let issues = Self::validation_issues(record);
        match issues.len() {
            0 => Ok(()),
            1 => Err(ContractError::Validation(issues[0].clone())),
            _ => Err(ContractError::ValidationList(issues)),
        }
    }

    pub fn validation_issues(record: &CoreRecord) -> Vec<String> {
        let mut issues = Vec::new();
        if !Self::allows_status(record.kind, &record.status) {
            issues.push(format!(
                "Invalid status \"{}\" for {}",
                record.status,
                record.kind
            ));
        }
        if record.schema_version < 1 {
            issues.push("schemaVersion must be >= 1".into());
        }
        if record.revision < 1 {
            issues.push("revision must be >= 1".into());
        }

        let payload = match serde_json::from_str::<Value>(&record.payload_json) {
            Ok(Value::Object(_)) => serde_json::from_str(&record.payload_json).unwrap(),
            Ok(_) => {
                issues.push("payload_json must decode to a JSON object".into());
                return issues;
            }
            Err(_) => {
                issues.push("payload_json must be valid JSON".into());
                return issues;
            }
        };

        if let Value::Object(ref map) = payload {
            if let Some(required) = Self::expected_payload_keys(record.kind) {
                for key in required {
                    if !map.contains_key(*key) {
                        issues.push(format!(
                            "Missing required payload key \"{key}\" for {}",
                            record.kind
                        ));
                    }
                }
            }
        }
        issues
    }

    pub fn allows_status(kind: CoreRecordKind, status: &str) -> bool {
        Self::allowed_status_slice(kind).contains(&status)
    }

    fn allowed_status_slice(kind: CoreRecordKind) -> &'static [&'static str] {
        match kind {
            CoreRecordKind::Capture => &[
                "captured",
                "classifying",
                "offloaded",
                "classified",
                "needs_review",
                "deleted",
            ],
            CoreRecordKind::Task | CoreRecordKind::TimelineItem => {
                &["active", "completed", "deleted"]
            }
            CoreRecordKind::MetricDefinition => &["active", "deleted"],
            CoreRecordKind::MetricLog => &["logged", "deleted"],
            CoreRecordKind::Habit => &["active", "inactive", "reset", "deleted"],
            CoreRecordKind::CriticalSignal => &["active", "dismissed", "deleted"],
            CoreRecordKind::WidgetInstance
            | CoreRecordKind::DashboardSnapshot
            | CoreRecordKind::AppSetting
            | CoreRecordKind::UserMemory
            | CoreRecordKind::AgentSessionSummary
            | CoreRecordKind::File
            | CoreRecordKind::Contact
            | CoreRecordKind::RepoEvent
            | CoreRecordKind::PaymentEvent
            | CoreRecordKind::FeedItem => &["active", "deleted"],
            CoreRecordKind::AgentRun => &["completed", "needs_review", "failed", "deleted"],
            CoreRecordKind::AgentSuggestion => {
                &["pending_review", "materialized", "dismissed", "deleted"]
            }
            CoreRecordKind::ReviewFinding => &["open", "accepted", "dismissed", "deleted"],
            CoreRecordKind::Project => &["active", "archived", "deleted"],
            CoreRecordKind::Email => &["active", "archived", "deleted"],
            CoreRecordKind::CalendarEvent => &["active", "cancelled", "deleted"],
            CoreRecordKind::Connector => &["placeholder", "linked", "paused", "error", "deleted"],
            CoreRecordKind::Outlet => &["placeholder", "linked", "error", "deleted"],
        }
    }

    #[allow(dead_code)]
    pub fn allowed_statuses(kind: CoreRecordKind) -> &'static [&'static str] {
        Self::allowed_status_slice(kind)
    }

    pub fn expected_payload_keys(kind: CoreRecordKind) -> Option<&'static [&'static str]> {
        Some(match kind {
            CoreRecordKind::Capture => &["content", "offloadRequested", "classification"],
            CoreRecordKind::Task => &["dueAt", "sourceCaptureId", "isCompleted"],
            CoreRecordKind::TimelineItem => &["timelineType"],
            CoreRecordKind::MetricDefinition => &[
                "metricType",
                "period",
                "currentValue",
                "targetValue",
                "previousBest",
                "unit",
            ],
            CoreRecordKind::MetricLog => &["metricDefinitionId", "delta", "unit"],
            CoreRecordKind::Habit => &[
                "habitType",
                "label",
                "startDate",
                "previousBestDays",
                "isActive",
            ],
            CoreRecordKind::CriticalSignal => &["source", "urgency", "isDismissed"],
            CoreRecordKind::WidgetInstance => {
                &["widgetType", "posX", "posY", "width", "height", "configJson"]
            }
            CoreRecordKind::DashboardSnapshot => &["widgets"],
            CoreRecordKind::AppSetting => &[],
            CoreRecordKind::AgentRun => &[
                "captureRecordId",
                "classification",
                "suggestionCount",
                "processor",
            ],
            CoreRecordKind::AgentSuggestion => &["captureRecordId", "agentRunId"],
            CoreRecordKind::UserMemory => &[
                "raw",
                "scope",
                "source",
                "supersedesId",
                "parentMemoryId",
                "topicPath",
                "nodeKind",
                "displayLabel",
                "sortOrder",
                "fileName",
                "filePath",
                "uri",
                "mimeHint",
                "linkedRecordId",
            ],
            CoreRecordKind::ReviewFinding => &[
                "findingType",
                "message",
                "evidenceRecordIds",
                "suggestedTool",
                "status",
            ],
            CoreRecordKind::AgentSessionSummary => &["sessionId", "summary"],
            CoreRecordKind::Project => &["description", "color", "externalRefs"],
            CoreRecordKind::File => &[
                "provider",
                "driveFileId",
                "mimeType",
                "uri",
                "modifiedAt",
                "folderPath",
            ],
            CoreRecordKind::Email => &[
                "messageId",
                "threadId",
                "from",
                "to",
                "subject",
                "snippet",
                "receivedAt",
                "provider",
                "uri",
            ],
            CoreRecordKind::CalendarEvent => &[
                "provider",
                "eventId",
                "startAt",
                "endAt",
                "location",
                "attendees",
                "calendarId",
            ],
            CoreRecordKind::Contact => &[
                "provider",
                "contactId",
                "email",
                "phone",
                "displayName",
                "organization",
            ],
            CoreRecordKind::RepoEvent => &[
                "provider",
                "eventId",
                "eventType",
                "repoFullName",
                "title",
                "state",
                "author",
                "branch",
                "conclusion",
                "uri",
                "occurredAt",
            ],
            CoreRecordKind::PaymentEvent => &[
                "provider",
                "eventId",
                "eventType",
                "amount",
                "currency",
                "customerId",
                "uri",
                "occurredAt",
            ],
            CoreRecordKind::FeedItem => &[
                "provider",
                "externalId",
                "feedType",
                "title",
                "summary",
                "metadata",
                "uri",
                "occurredAt",
            ],
            CoreRecordKind::Connector => &["provider", "accountLabel", "status", "lastSyncAt"],
            CoreRecordKind::Outlet => &[
                "provider",
                "status",
                "repoOwner",
                "repoName",
                "defaultBranch",
                "lastPushAt",
                "lastPullAt",
                "vendorAccountKey",
            ],
        })
    }

    pub fn with_merged_payload(mut record: CoreRecord) -> CoreRecord {
        let defaults = Self::default_payload(record.kind);
        let mut payload = record
            .payload()
            .unwrap_or(Value::Object(serde_json::Map::new()));
        if let Value::Object(ref mut map) = payload {
            for (k, v) in defaults {
                map.entry(k).or_insert(v);
            }
        }
        record.payload_json = payload.to_string();
        record
    }

    pub fn default_payload(kind: CoreRecordKind) -> HashMap<String, Value> {
        let now = chrono::Utc::now().to_rfc3339();
        match kind {
            CoreRecordKind::Capture => HashMap::from([
                ("content".into(), Value::String(String::new())),
                ("offloadRequested".into(), Value::Bool(false)),
                ("classification".into(), Value::String("journal".into())),
            ]),
            CoreRecordKind::Task => HashMap::from([
                ("dueAt".into(), Value::Null),
                ("sourceCaptureId".into(), Value::String(String::new())),
                ("isCompleted".into(), Value::Bool(false)),
            ]),
            CoreRecordKind::TimelineItem => {
                HashMap::from([("timelineType".into(), Value::String("note".into()))])
            }
            CoreRecordKind::WidgetInstance => HashMap::from([
                ("widgetType".into(), Value::String(String::new())),
                ("posX".into(), Value::Number(0.into())),
                ("posY".into(), Value::Number(0.into())),
                ("width".into(), Value::Number(2.into())),
                ("height".into(), Value::Number(2.into())),
                ("configJson".into(), Value::String("{}".into())),
            ]),
            CoreRecordKind::Connector => HashMap::from([
                ("provider".into(), Value::String("generic".into())),
                ("accountLabel".into(), Value::Null),
                ("status".into(), Value::String("placeholder".into())),
                ("lastSyncAt".into(), Value::Null),
            ]),
            CoreRecordKind::Habit => HashMap::from([
                ("habitType".into(), Value::String("generic".into())),
                ("label".into(), Value::String(String::new())),
                ("startDate".into(), Value::String(now)),
                ("previousBestDays".into(), Value::Number(0.into())),
                ("isActive".into(), Value::Bool(true)),
            ]),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CoreRecord;

    #[test]
    fn task_requires_payload_keys() {
        let record = CoreRecord::new(CoreRecordKind::Task, "Test");
        assert!(!RecordContract::validation_issues(&record).is_empty());
        let merged = RecordContract::with_merged_payload(record);
        RecordContract::validate(&merged).expect("merged task valid");
    }
}
