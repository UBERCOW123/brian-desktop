use core_contracts::vendor_core_root;
use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub schema_version: i32,
    pub contracts_present: bool,
}

#[tauri::command]
pub fn health_check(_state: State<'_, AppState>) -> HealthResponse {
    HealthResponse {
        ok: true,
        schema_version: core_db::SCHEMA_VERSION,
        contracts_present: core_contracts::contracts_present(),
    }
}

#[derive(Serialize)]
pub struct ContractsInfo {
    pub vendor_root: String,
    pub contracts_version_file: String,
    pub sync_strategy: String,
    pub data_model: String,
    pub widget_seed: String,
    pub db_path: String,
}

#[tauri::command]
pub fn contracts_info(state: State<'_, AppState>) -> ContractsInfo {
    ContractsInfo {
        vendor_root: vendor_core_root().display().to_string(),
        contracts_version_file: include_str!("../../CONTRACTS_VERSION").trim().to_string(),
        sync_strategy: core_contracts::sync_strategy_path().display().to_string(),
        data_model: core_contracts::data_model_path().display().to_string(),
        widget_seed: core_contracts::widget_seed_path().display().to_string(),
        db_path: state.db_path.display().to_string(),
    }
}
