use core_auth::{OAuthStart, SupabaseClient, WorkspaceContext};
use core_db::{open, Connection, DbError};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub db_path: PathBuf,
    pub db: Mutex<Connection>,
    pub supabase: Mutex<SupabaseClient>,
    pub workspace: Mutex<WorkspaceContext>,
    pub pending_oauth: Mutex<Option<OAuthStart>>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, DbError> {
        let data_dir = app
            .path()
            .app_data_dir()
            .expect("app data dir");
        std::fs::create_dir_all(&data_dir).ok();

        let db_path = data_dir.join(core_db::DEFAULT_DB_FILENAME);
        let db = open(&db_path)?;

        let config = core_auth::SupabaseConfig::from_env().unwrap_or_else(|_| core_auth::SupabaseConfig::empty());

        Ok(Self {
            db_path,
            db: Mutex::new(db),
            supabase: Mutex::new(SupabaseClient::new(config)),
            workspace: Mutex::new(WorkspaceContext::new()),
            pending_oauth: Mutex::new(None),
        })
    }

    pub fn with_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let guard = self.db.lock().map_err(|e| e.to_string())?;
        f(&guard)
    }
}
