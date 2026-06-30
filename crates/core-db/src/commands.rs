//! Local write path — record + event + outbox (port of `CoreCommandService` subset).

use crate::layout::{can_install, clamp_layout, next_available_row, COLUMN_COUNT};
use crate::projections::WidgetInstanceProjection;
use crate::repository::CoreRepository;
use chrono::Utc;
use core_contracts::{
    CoreEvent, CoreEventKind, CoreRecord, CoreRecordKind, RecordContract, SyncOutboxItem,
    WidgetCatalogEntry,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

pub struct CommandService<'a> {
    repo: CoreRepository<'a>,
    origin_device_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WidgetLayoutInput {
    pub id: Uuid,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
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
        self.write_record_change(&record, CoreEventKind::RecordCreated, json!({}))?;
        Ok(record)
    }

    pub fn install_widget(
        &self,
        entry: &WidgetCatalogEntry,
        existing: &[WidgetInstanceProjection],
    ) -> Result<CoreRecord, CommandError> {
        if !can_install(&entry.widget_type, entry.allow_multiple, existing) {
            return Err(CommandError::WidgetAlreadyInstalled(entry.widget_type.clone()));
        }

        let (width, height) = (
            entry.default_width().clamp(1, COLUMN_COUNT),
            entry.default_height().max(1),
        );
        let pos_y = next_available_row(existing);
        self.create_widget_record(
            None,
            &entry.widget_type,
            0,
            pos_y,
            width,
            height,
            "{}",
            true,
        )
    }

    pub fn update_widget_layout(&self, layout: WidgetLayoutInput) -> Result<CoreRecord, CommandError> {
        let existing = self
            .repo
            .get_record(layout.id)?
            .ok_or(CommandError::WidgetNotFound(layout.id))?;
        if existing.kind != CoreRecordKind::WidgetInstance || existing.status != "active" {
            return Err(CommandError::WidgetNotFound(layout.id));
        }

        let (pos_x, width) = clamp_layout(layout.pos_x, layout.width);
        let height = layout.height.max(1);
        let pos_y = layout.pos_y.max(0);

        let payload = existing.payload().unwrap_or_default();
        let widget_type = payload
            .get("widgetType")
            .and_then(|v| v.as_str())
            .unwrap_or(&existing.title)
            .to_string();
        let config_json = payload
            .get("configJson")
            .and_then(|v| v.as_str())
            .unwrap_or("{}")
            .to_string();

        self.create_widget_record(
            Some(existing),
            &widget_type,
            pos_x,
            pos_y,
            width,
            height,
            &config_json,
            false,
        )
    }

    pub fn update_widget_layouts(
        &self,
        layouts: &[WidgetLayoutInput],
    ) -> Result<Vec<CoreRecord>, CommandError> {
        let mut updated = Vec::with_capacity(layouts.len());
        for layout in layouts {
            updated.push(self.update_widget_layout(layout.clone())?);
        }
        Ok(updated)
    }

    pub fn delete_widget(&self, id: Uuid) -> Result<(), CommandError> {
        let existing = self
            .repo
            .get_record(id)?
            .ok_or(CommandError::WidgetNotFound(id))?;
        if existing.kind != CoreRecordKind::WidgetInstance {
            return Err(CommandError::WidgetNotFound(id));
        }

        let mut deleted = existing.clone();
        deleted.status = "deleted".into();
        deleted.deleted_at = Some(Utc::now());
        deleted.updated_at = Utc::now();
        deleted.revision += 1;
        deleted.origin_device_id = self.origin_device_id.clone();

        self.write_record_change(
            &deleted,
            CoreEventKind::RecordDeleted,
            json!({ "kind": deleted.kind.to_string() }),
        )?;
        Ok(())
    }

    fn create_widget_record(
        &self,
        existing: Option<CoreRecord>,
        widget_type: &str,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
        config_json: &str,
        is_new: bool,
    ) -> Result<CoreRecord, CommandError> {
        let now = Utc::now();
        let created_at = existing
            .as_ref()
            .map(|r| r.created_at)
            .unwrap_or(now);
        let sort_at = created_at;
        let revision = existing.as_ref().map(|r| r.revision + 1).unwrap_or(1);
        let id = existing.as_ref().map(|r| r.id).unwrap_or_else(Uuid::new_v4);

        let mut record = CoreRecord {
            id,
            kind: CoreRecordKind::WidgetInstance,
            title: widget_type.to_string(),
            summary: Some("Dashboard widget instance".into()),
            status: "active".into(),
            payload_json: json!({
                "widgetType": widget_type,
                "posX": pos_x,
                "posY": pos_y,
                "width": width,
                "height": height,
                "configJson": config_json,
            })
            .to_string(),
            source_record_id: None,
            sort_at,
            created_at,
            updated_at: now,
            deleted_at: None,
            revision,
            schema_version: 1,
            origin_device_id: self.origin_device_id.clone(),
            external_id: None,
        };
        record = RecordContract::with_merged_payload(record);

        let event_kind = if is_new {
            CoreEventKind::RecordCreated
        } else {
            CoreEventKind::WidgetLayoutChanged
        };
        let event_payload = json!({
            "widgetType": widget_type,
            "posX": pos_x,
            "posY": pos_y,
            "width": width,
            "height": height,
        });
        self.write_record_change(&record, event_kind, event_payload)?;
        Ok(record)
    }

    fn write_record_change(
        &self,
        record: &CoreRecord,
        event_kind: CoreEventKind,
        event_payload: serde_json::Value,
    ) -> Result<(), CommandError> {
        self.repo.upsert_record(record)?;
        let mut event = CoreEvent::new(record.id, event_kind);
        event.origin_device_id = self.origin_device_id.clone();
        event.base_revision = record.revision;
        event.payload_json = event_payload.to_string();
        self.repo.insert_event(&event)?;
        self.repo.insert_outbox_item(&SyncOutboxItem::new(event.id))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("repository: {0}")]
    Repository(#[from] crate::RepositoryError),
    #[error("widget already installed: {0}")]
    WidgetAlreadyInstalled(String),
    #[error("widget not found: {0}")]
    WidgetNotFound(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use core_contracts::{SyncOutboxStatus, WidgetCatalog};

    fn clock_entry() -> WidgetCatalogEntry {
        WidgetCatalog::load()
            .unwrap()
            .find("clock_widget")
            .unwrap()
            .clone()
    }

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

    #[test]
    fn install_widget_writes_widget_instance() {
        let conn = open_in_memory().unwrap();
        let svc = CommandService::new(&conn, "test-device");
        let record = svc.install_widget(&clock_entry(), &[]).unwrap();
        assert_eq!(record.kind, CoreRecordKind::WidgetInstance);
        let payload = record.payload().unwrap();
        assert_eq!(payload["widgetType"], "clock_widget");
    }
}
