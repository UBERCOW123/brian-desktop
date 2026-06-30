//! Read-only projections for Phase 0 dashboard data.

use crate::repository::CoreRepository;
use chrono::{DateTime, Utc};
use core_contracts::CoreRecordKind;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskQueueStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskQueueItem {
    pub record_id: Uuid,
    pub title: String,
    pub display_title: String,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub status: TaskQueueStatus,
    pub is_overdue: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEntryType {
    Task,
    Alert,
    Note,
    Milestone,
    Achievement,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineEntry {
    pub id: Uuid,
    pub entry_type: TimelineEntryType,
    pub title: String,
    pub subtitle: Option<String>,
    pub date: DateTime<Utc>,
    pub is_completed: bool,
    pub is_critical: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WidgetInstanceProjection {
    pub id: Uuid,
    pub widget_type: String,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
    pub config_json: String,
}

pub struct Projections<'a> {
    repo: CoreRepository<'a>,
}

impl<'a> Projections<'a> {
    pub fn new(repo: CoreRepository<'a>) -> Self {
        Self { repo }
    }

    pub fn task_queue(
        &self,
        include_completed: bool,
    ) -> Result<Vec<TaskQueueItem>, crate::RepositoryError> {
        let tasks = self
            .repo
            .get_records_by_kinds(&[CoreRecordKind::Task], false)?;
        let now = Utc::now();
        let mut items: Vec<TaskQueueItem> = tasks
            .into_iter()
            .filter_map(|record| {
                let payload = record.payload().ok()?;
                let is_completed = record.status == "completed"
                    || payload.get("isCompleted").and_then(|v| v.as_bool()) == Some(true);
                if !include_completed && is_completed {
                    return None;
                }
                let due_at = payload
                    .get("dueAt")
                    .and_then(|v| v.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc));
                let display_title = payload
                    .get("displayTitle")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&record.title)
                    .to_string();
                let is_overdue = !is_completed
                    && due_at.is_some_and(|d| d < now);
                Some(TaskQueueItem {
                    record_id: record.id,
                    title: record.title.clone(),
                    display_title,
                    summary: record.summary.clone(),
                    created_at: record.created_at,
                    due_at,
                    status: if is_completed {
                        TaskQueueStatus::Completed
                    } else {
                        TaskQueueStatus::Active
                    },
                    is_overdue,
                })
            })
            .collect();
        items.sort_by(|a, b| {
            match (a.status == TaskQueueStatus::Completed, b.status == TaskQueueStatus::Completed) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => {
                    let a_sort = a.due_at.unwrap_or(a.created_at);
                    let b_sort = b.due_at.unwrap_or(b.created_at);
                    a_sort.cmp(&b_sort)
                }
            }
        });
        Ok(items)
    }

    pub fn timeline_entries(&self) -> Result<Vec<TimelineEntry>, crate::RepositoryError> {
        let records = self.repo.get_records_by_kinds(
            &[
                CoreRecordKind::Task,
                CoreRecordKind::TimelineItem,
                CoreRecordKind::CriticalSignal,
            ],
            false,
        )?;
        let mut entries = Vec::new();
        for record in records {
            let payload = record.payload().unwrap_or_default();
            let entry = match record.kind {
                CoreRecordKind::Task => TimelineEntry {
                    id: record.id,
                    entry_type: TimelineEntryType::Task,
                    title: record.title.clone(),
                    subtitle: record.summary.clone(),
                    date: record.sort_at,
                    is_completed: record.status == "completed",
                    is_critical: payload
                        .get("isCritical")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                },
                CoreRecordKind::CriticalSignal => TimelineEntry {
                    id: record.id,
                    entry_type: TimelineEntryType::Alert,
                    title: record.title.clone(),
                    subtitle: record.summary.clone(),
                    date: record.sort_at,
                    is_completed: record.status == "dismissed",
                    is_critical: true,
                },
                CoreRecordKind::TimelineItem => {
                    let timeline_type = payload
                        .get("timelineType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("note");
                    let entry_type = match timeline_type {
                        "milestone" => TimelineEntryType::Milestone,
                        "achievement" => TimelineEntryType::Achievement,
                        _ => TimelineEntryType::Note,
                    };
                    TimelineEntry {
                        id: record.id,
                        entry_type,
                        title: record.title.clone(),
                        subtitle: record.summary.clone(),
                        date: record.sort_at,
                        is_completed: record.status == "completed",
                        is_critical: payload
                            .get("isCritical")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    }
                }
                _ => continue,
            };
            entries.push(entry);
        }
        entries.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(entries)
    }

    pub fn widget_instances(&self) -> Result<Vec<WidgetInstanceProjection>, crate::RepositoryError> {
        let records = self
            .repo
            .get_records_by_kinds(&[CoreRecordKind::WidgetInstance], false)?;
        Ok(records
            .into_iter()
            .filter(|r| r.status == "active")
            .filter_map(|record| {
                let payload = record.payload().ok()?;
                Some(WidgetInstanceProjection {
                    id: record.id,
                    widget_type: payload
                        .get("widgetType")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&record.title)
                        .to_string(),
                    pos_x: payload.get("posX").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    pos_y: payload.get("posY").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    width: payload.get("width").and_then(|v| v.as_i64()).unwrap_or(2) as i32,
                    height: payload.get("height").and_then(|v| v.as_i64()).unwrap_or(2) as i32,
                    config_json: payload
                        .get("configJson")
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}")
                        .to_string(),
                })
            })
            .collect())
    }
}
