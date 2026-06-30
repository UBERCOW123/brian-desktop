//! Local write path — record + event + outbox (port of `CoreCommandService` subset).

use crate::repository::CoreRepository;
use chrono::Utc;
use core_contracts::{
    CoreEvent, CoreEventKind, CoreRecord, CoreRecordKind, RecordContract, SyncOutboxItem,
};
use rusqlite::Connection;
use serde_json::json;
use thiserror::Error;

pub struct CommandService<'a> {
    repo: CoreRepository<'a>,
    origin_device_id: String,
}

impl<'a> CommandService<'a> {
    pub fn new(conn: &'a Connection, origin_device_id: impl Into<String>) -> Self {
        Self {
            repo: CoreRepository::new(conn),
            origin_device_id: origin_device_id.into(),
        }
    }

    pub fn create_task(
        &self,
        title: impl Into<String>,
        due_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<CoreRecord, CommandError> {
        let title = title.into();
        let mut record = CoreRecord::new(CoreRecordKind::Task, title.clone());
        record.origin_device_id = self.origin_device_id.clone();
        record.payload_json = json!({
            "dueAt": due_at.map(|d| d.to_rfc3339()),
            "sourceCaptureId": "",
            "isCompleted": false,
            "displayTitle": title,
        })
        .to_string();
        record = RecordContract::with_merged_payload(record);
        self.write_record_change(&record, CoreEventKind::RecordCreated)?;
        Ok(record)
    }

    fn write_record_change(
        &self,
        record: &CoreRecord,
        event_kind: CoreEventKind,
    ) -> Result<(), CommandError> {
        self.repo.upsert_record(record)?;
        let mut event = CoreEvent::new(record.id, event_kind);
        event.origin_device_id = self.origin_device_id.clone();
        event.base_revision = record.revision;
        self.repo.insert_event(&event)?;
        self.repo.insert_outbox_item(&SyncOutboxItem::new(event.id))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("repository: {0}")]
    Repository(#[from] crate::RepositoryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use core_contracts::SyncOutboxStatus;

    #[test]
    fn create_task_enqueues_outbox() {
        let conn = open_in_memory().unwrap();
        let svc = CommandService::new(&conn, "test-device");
        let task = svc.create_task("Ship desktop", None).unwrap();
        let repo = CoreRepository::new(&conn);
        let pending = repo
            .get_outbox_items(Some(SyncOutboxStatus::Pending), None)
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(repo.get_record(task.id).unwrap().unwrap().title, "Ship desktop");
    }
}
