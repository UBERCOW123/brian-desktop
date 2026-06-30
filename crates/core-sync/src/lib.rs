//! Sync engine — outbox drain + incremental pull (port of mobile `CoreSyncService`).

mod error;
mod pull;
mod push;
mod service;

pub use error::SyncError;

pub use pull::{PullResult, SupabasePullClient};
pub use push::{PushResult, SupabasePushClient};
pub use service::{CoreSyncService, DrainReport};
