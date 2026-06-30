//! Outbox drain + pull orchestration — port of mobile `CoreSyncService`.
//!
//! SQLite access runs on the blocking thread pool so async network I/O stays `Send`.

use crate::pull::{apply_pull_batch, PullResult, SupabasePullClient};
use crate::push::{PushResult, SupabasePushClient};
use crate::SyncError;
use chrono::{DateTime, Utc};
use core_auth::SupabaseClient;
use core_contracts::{CoreEvent, CoreLink, CoreRecord, SyncOutboxItem, SyncOutboxStatus};
use core_db::{Connection, CoreRepository, LocalPrefKey, LocalPrefs};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct DrainReport {
    pub processed: u32,
    pub synced: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug)]
enum DrainWorkItem {
    LocalOnly(SyncOutboxItem),
    Remote {
        outbox: SyncOutboxItem,
        event: CoreEvent,
        record: CoreRecord,
        links: Vec<CoreLink>,
    },
}

/// Owned sync engine — safe to drive from Tauri async commands.
pub struct CoreSyncService {
    db_path: PathBuf,
    supabase: SupabaseClient,
    workspace_id: Option<String>,
    device_id: Option<String>,
}

impl CoreSyncService {
    pub fn new(
        db_path: PathBuf,
        supabase: SupabaseClient,
        workspace_id: Option<&str>,
        device_id: Option<&str>,
    ) -> Self {
        Self {
            db_path,
            supabase,
            workspace_id: workspace_id.map(String::from),
            device_id: device_id.map(String::from),
        }
    }

    pub fn is_remote_push_configured(&self) -> bool {
        matches!(
            (
                self.supabase.user_id(),
                self.workspace_id.as_deref(),
                self.device_id.as_deref()
            ),
            (Some(_), Some(_), Some(_))
        )
    }

    pub fn is_remote_pull_configured(&self) -> bool {
        self.workspace_id.is_some()
    }

    pub async fn recover_stale_processing_outbox(
        &self,
        stale_after_minutes: i64,
    ) -> Result<usize, SyncError> {
        run_db(self.db_path.clone(), move |conn| {
            Ok(CoreRepository::new(conn).recover_stale_processing_outbox(stale_after_minutes)?)
        })
        .await
    }

    pub async fn drain_outbox(&self, limit: u32) -> Result<DrainReport, SyncError> {
        let push = self.push_client()?;
        self.recover_stale_processing_outbox(5).await?;

        let pending = run_db(self.db_path.clone(), move |conn| {
            let repo = CoreRepository::new(conn);
            Ok(repo.get_outbox_items(
                Some(SyncOutboxStatus::Pending),
                Some(limit as i32),
            )?)
        })
        .await?;

        let mut report = DrainReport::default();

        for item in pending {
            report.processed += 1;

            let work = match self.prepare_drain_item(item).await? {
                Some(w) => w,
                None => {
                    report.failed += 1;
                    continue;
                }
            };

            match work {
                DrainWorkItem::LocalOnly(outbox) => {
                    let outbox_id = outbox.id;
                    run_db(self.db_path.clone(), move |conn| {
                        CoreRepository::new(conn).mark_outbox_synced(outbox_id)?;
                        Ok(())
                    })
                    .await?;
                    report.synced += 1;
                }
                DrainWorkItem::Remote {
                    outbox,
                    event,
                    record,
                    links,
                } => {
                    let outbox_id = outbox.id;
                    let PushResult { accepted, error } = push
                        .push_event(&event, &record, &links, &outbox)
                        .await?;

                    if accepted {
                        run_db(self.db_path.clone(), move |conn| {
                            CoreRepository::new(conn).mark_outbox_synced(outbox_id)?;
                            Ok(())
                        })
                        .await?;
                        report.synced += 1;
                    } else {
                        let msg = error.unwrap_or_else(|| "push rejected".into());
                        let msg_for_db = msg.clone();
                        run_db(self.db_path.clone(), move |conn| {
                            CoreRepository::new(conn).mark_outbox_failed(outbox_id, &msg_for_db)?;
                            Ok(())
                        })
                        .await?;
                        report.failed += 1;
                        report.errors.push(msg);
                    }
                }
            }
        }

        Ok(report)
    }

    pub async fn pull_server_records(&self) -> Result<PullResult, SyncError> {
        let workspace_id = self
            .workspace_id
            .clone()
            .ok_or(SyncError::PullNotConfigured)?;
        let pull = SupabasePullClient::new(self.supabase.clone(), workspace_id);

        let since = run_db(self.db_path.clone(), |conn| {
            let prefs = LocalPrefs::new(conn);
            let raw = prefs.get(LocalPrefKey::SyncLastPullAt)?;
            Ok(raw.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }))
        })
        .await?;

        let batch = pull.fetch_since(since).await;

        let result = run_db(self.db_path.clone(), move |conn| {
            let repo = CoreRepository::new(conn);
            Ok(apply_pull_batch(&repo, batch))
        })
        .await?;

        if let Some(latest) = result.latest_updated_at {
            run_db(self.db_path.clone(), move |conn| {
                LocalPrefs::new(conn).set(LocalPrefKey::SyncLastPullAt, &latest.to_rfc3339())?;
                Ok(())
            })
            .await?;
        }

        Ok(result)
    }

    fn push_client(&self) -> Result<SupabasePushClient, SyncError> {
        match (
            self.supabase.user_id(),
            self.workspace_id.as_deref(),
            self.device_id.as_deref(),
        ) {
            (Some(uid), Some(wid), Some(did)) => Ok(SupabasePushClient::new(
                self.supabase.clone(),
                uid.to_string(),
                wid.to_string(),
                did.to_string(),
            )),
            _ => Err(SyncError::PushNotConfigured),
        }
    }

    async fn prepare_drain_item(
        &self,
        item: SyncOutboxItem,
    ) -> Result<Option<DrainWorkItem>, SyncError> {
        run_db(self.db_path.clone(), move |conn| {
            let repo = CoreRepository::new(conn);
            repo.mark_outbox_processing(item.id)?;

            let event = match repo.get_event_by_id(item.event_id)? {
                Some(e) => e,
                None => {
                    let msg = format!("Missing event {}", item.event_id);
                    repo.mark_outbox_failed(item.id, &msg)?;
                    return Ok(None);
                }
            };

            let local_only = event
                .payload()
                .ok()
                .and_then(|p| {
                    p.get("outboxPolicy")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "local_only")
                })
                .unwrap_or(false);

            if local_only {
                return Ok(Some(DrainWorkItem::LocalOnly(item)));
            }

            let record = match repo.get_record(event.record_id)? {
                Some(r) => r,
                None => {
                    let msg = format!("Missing record {}", event.record_id);
                    repo.mark_outbox_failed(item.id, &msg)?;
                    return Ok(None);
                }
            };

            let links = repo.links_for_record(record.id)?;

            Ok(Some(DrainWorkItem::Remote {
                outbox: item,
                event,
                record,
                links,
            }))
        })
        .await
    }
}

async fn run_db<T, F>(db_path: PathBuf, f: F) -> Result<T, SyncError>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, SyncError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let conn = core_db::open(&db_path)?;
        f(&conn)
    })
    .await
    .map_err(|e| SyncError::Other {
        message: e.to_string(),
    })?
}
