use std::path::PathBuf;
use std::sync::Mutex;

use core_db::{Connection, DbError};
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub db_path: PathBuf,
    #[allow(dead_code)] // used by sync/commands in plan Phase 0
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, DbError> {
        let data_dir = app
            .path()
            .app_data_dir()
            .expect("app data dir");
        std::fs::create_dir_all(&data_dir).ok();

        let db_path = data_dir.join(core_db::DEFAULT_DB_FILENAME);
        let db = core_db::open(&db_path)?;

        Ok(Self {
            db_path,
            db: Mutex::new(db),
        })
    }
}
