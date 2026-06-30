//! Wire-string enums mirroring `CoreRecordKind` / `CoreEventKind` in mobile.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

macro_rules! wire_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire),+
                }
            }

            pub fn all() -> &'static [Self] {
                &[$(Self::$variant),+]
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = UnknownKindError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($wire => Ok(Self::$variant),)+
                    _ => Err(UnknownKindError(s.to_string())),
                }
            }
        }
    };
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown CORE kind: {0}")]
pub struct UnknownKindError(pub String);

wire_enum! {
    CoreRecordKind {
        Capture => "capture",
        Task => "task",
        TimelineItem => "timeline_item",
        MetricDefinition => "metric_definition",
        MetricLog => "metric_log",
        Habit => "habit",
        CriticalSignal => "critical_signal",
        WidgetInstance => "widget_instance",
        DashboardSnapshot => "dashboard_snapshot",
        AppSetting => "app_setting",
        AgentRun => "agent_run",
        AgentSuggestion => "agent_suggestion",
        UserMemory => "user_memory",
        ReviewFinding => "review_finding",
        AgentSessionSummary => "agent_session_summary",
        Project => "project",
        File => "file",
        Email => "email",
        CalendarEvent => "calendar_event",
        Contact => "contact",
        RepoEvent => "repo_event",
        PaymentEvent => "payment_event",
        FeedItem => "feed_item",
        Connector => "connector",
        Outlet => "outlet",
    }
}

wire_enum! {
    CoreEventKind {
        RecordCreated => "record_created",
        RecordUpdated => "record_updated",
        RecordDeleted => "record_deleted",
        CaptureOffloaded => "capture_offloaded",
        ClassificationCompleted => "classification_completed",
        ClassificationFailed => "classification_failed",
        ClassificationCorrected => "classification_corrected",
        ClassifierAliasLearned => "classifier_alias_learned",
        TaskScheduled => "task_scheduled",
        MetricLogged => "metric_logged",
        HabitStarted => "habit_started",
        HabitReset => "habit_reset",
        CriticalRaised => "critical_raised",
        WidgetLayoutChanged => "widget_layout_changed",
        SnapshotSaved => "snapshot_saved",
        SnapshotRestored => "snapshot_restored",
        ProjectCreated => "project_created",
        ExternalIngested => "external_ingested",
        ChannelBundled => "channel_bundled",
    }
}

wire_enum! {
    CoreLinkKind {
        DerivedFrom => "derived_from",
        SuggestedFrom => "suggested_from",
        GeneratedBy => "generated_by",
        LinkedRecord => "linked_record",
        ParentOf => "parent_of",
        References => "references",
        StreamsTo => "streams_to",
        PublishesTo => "publishes_to",
    }
}

wire_enum! {
    SyncOutboxStatus {
        Pending => "pending",
        Processing => "processing",
        Synced => "synced",
        Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn load_manifest() -> serde_json::Value {
        let path = crate::paths::parity_manifest_path();
        let raw = std::fs::read_to_string(path).expect("parity manifest");
        serde_json::from_str(&raw).expect("valid json")
    }

    fn assert_parity<T: FromStr>(manifest_key: &str, variants: &[T])
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let manifest = load_manifest();
        let expected: Vec<String> = manifest[manifest_key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let actual: HashSet<String> = variants
            .iter()
            .map(|_| {
                // round-trip via string representation of each variant
                unimplemented!()
            })
            .collect();
        let _ = actual;
        let _ = expected;
    }

    #[test]
    fn record_kinds_match_manifest() {
        let manifest = load_manifest();
        let expected: HashSet<_> = manifest["record_kinds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let actual: HashSet<_> = CoreRecordKind::all().iter().map(|k| k.as_str()).collect();
        assert_eq!(expected, actual, "CoreRecordKind drift vs mobile");
    }

    #[test]
    fn event_kinds_match_manifest() {
        let manifest = load_manifest();
        let expected: HashSet<_> = manifest["event_kinds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let actual: HashSet<_> = CoreEventKind::all().iter().map(|k| k.as_str()).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn link_kinds_match_manifest() {
        let manifest = load_manifest();
        let expected: HashSet<_> = manifest["link_kinds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let actual: HashSet<_> = CoreLinkKind::all().iter().map(|k| k.as_str()).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn outbox_statuses_match_manifest() {
        let manifest = load_manifest();
        let expected: HashSet<_> = manifest["outbox_statuses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let actual: HashSet<_> = SyncOutboxStatus::all().iter().map(|k| k.as_str()).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn unknown_kind_fails_closed() {
        assert!("not_a_kind".parse::<CoreRecordKind>().is_err());
    }
}
