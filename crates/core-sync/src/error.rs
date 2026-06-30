//! Shared sync errors.

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("workspace missing")]
    WorkspaceMissing,
    #[error("auth: {0}")]
    Auth(#[from] core_auth::ClientError),
    #[error("db: {0}")]
    Db(#[from] core_db::RepositoryError),
    #[error("sqlite: {0}")]
    Sqlite(#[from] core_db::DbError),
    #[error("local pref: {0}")]
    LocalPref(#[from] core_db::LocalPrefError),
    #[error("pull not configured")]
    PullNotConfigured,
    #[error("push not configured")]
    PushNotConfigured,
    #[error("sync: {message}")]
    Other { message: String },
}
