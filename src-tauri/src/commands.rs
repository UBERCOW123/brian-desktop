use core_auth::{
    fetch_default_workspace_id, platform_label, DeviceRegistrationService, OAuthFlow,
    SupabaseConfig, WorkspaceContext,
};
use core_db::{CommandService, CoreRepository, LocalPrefKey, LocalPrefs, Projections};
use core_sync::CoreSyncService;
use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub schema_version: i32,
    pub contracts_present: bool,
    pub supabase_configured: bool,
    pub signed_in: bool,
}

#[tauri::command]
pub fn health_check(state: State<'_, AppState>) -> HealthResponse {
    let signed_in = state
        .supabase
        .lock()
        .map(|c| c.is_signed_in())
        .unwrap_or(false);
    HealthResponse {
        ok: true,
        schema_version: core_db::SCHEMA_VERSION,
        contracts_present: core_contracts::contracts_present(),
        supabase_configured: SupabaseConfig::from_env()
            .map(|c| c.is_configured())
            .unwrap_or(false),
        signed_in,
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
        vendor_root: core_contracts::vendor_core_root().display().to_string(),
        contracts_version_file: include_str!("../../CONTRACTS_VERSION").trim().to_string(),
        sync_strategy: core_contracts::sync_strategy_path().display().to_string(),
        data_model: core_contracts::data_model_path().display().to_string(),
        widget_seed: core_contracts::widget_seed_path().display().to_string(),
        db_path: state.db_path.display().to_string(),
    }
}

#[derive(Serialize)]
pub struct AuthStartResponse {
    pub authorize_url: String,
}

#[tauri::command]
pub fn auth_start_apple(state: State<'_, AppState>) -> Result<AuthStartResponse, String> {
    let config = SupabaseConfig::from_env().map_err(|e| e.to_string())?;
    let start = OAuthFlow::start_apple(&config);
    let url = start.authorize_url.clone();
    *state.pending_oauth.lock().map_err(|e| e.to_string())? = Some(start);
    Ok(AuthStartResponse {
        authorize_url: url,
    })
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub signed_in: bool,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub workspace_id: Option<String>,
}

#[tauri::command]
pub async fn auth_complete_oauth(
    callback_url: String,
    state: State<'_, AppState>,
) -> Result<SessionInfo, String> {
    let code = OAuthFlow::parse_callback_code(&callback_url).map_err(|e| e.to_string())?;
    let verifier = state
        .pending_oauth
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("No pending OAuth flow")?
        .code_verifier;

    let mut client = state.supabase.lock().map_err(|e| e.to_string())?.clone();
    client
        .exchange_pkce_code(&code, &verifier)
        .await
        .map_err(|e| e.to_string())?;
    *state.supabase.lock().map_err(|e| e.to_string())? = client.clone();

    let workspace_id = fetch_default_workspace_id(&client)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut workspace = state.workspace.lock().map_err(|e| e.to_string())?;
        workspace.set_workspace_id(workspace_id.clone());
    }

    if let Some(ref wid) = workspace_id {
        ensure_device_registered(&state, wid).await?;
    }

    let client = state.supabase.lock().map_err(|e| e.to_string())?;
    let workspace = state.workspace.lock().map_err(|e| e.to_string())?;
    session_info_inner(&client, &workspace)
}

async fn ensure_device_registered(state: &AppState, workspace_id: &str) -> Result<(), String> {
    let cached = state.with_db(|conn| {
        DeviceRegistrationService::read_cached_device_id(conn).map_err(|e| e.to_string())
    })?;

    let client = state.supabase.lock().map_err(|e| e.to_string())?.clone();
    let device_id = DeviceRegistrationService::ensure_registered(
        &client,
        cached,
        workspace_id,
        platform_label(),
        "Brian Desktop",
    )
    .await
    .map_err(|e| e.to_string())?;

    if let Some(id) = device_id {
        state.with_db(|conn| {
            DeviceRegistrationService::write_cached_device_id(conn, &id)
                .map_err(|e| e.to_string())
        })?;
    }

    Ok(())
}

#[tauri::command]
pub async fn auth_sign_out(state: State<'_, AppState>) -> Result<(), String> {
    let mut client = state.supabase.lock().map_err(|e| e.to_string())?.clone();
    client.sign_out().await.map_err(|e| e.to_string())?;
    *state.supabase.lock().map_err(|e| e.to_string())? = client;
    state.workspace.lock().map_err(|e| e.to_string())?.clear();
    state.with_db(|conn| DeviceRegistrationService::clear_cache(conn).map_err(|e| e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn auth_session_info(state: State<'_, AppState>) -> Result<SessionInfo, String> {
    let client = state.supabase.lock().map_err(|e| e.to_string())?;
    let workspace = state.workspace.lock().map_err(|e| e.to_string())?;
    session_info_inner(&client, &workspace)
}

fn session_info_inner(
    client: &core_auth::SupabaseClient,
    workspace: &WorkspaceContext,
) -> Result<SessionInfo, String> {
    Ok(SessionInfo {
        signed_in: client.is_signed_in(),
        user_id: client.user_id().map(String::from),
        email: client.session().and_then(|s| s.email.clone()),
        workspace_id: workspace.workspace_id().map(String::from),
    })
}

#[tauri::command]
pub fn list_tasks(
    include_completed: bool,
    state: State<'_, AppState>,
) -> Result<Vec<core_db::TaskQueueItem>, String> {
    state.with_db(|conn| {
        let repo = CoreRepository::new(conn);
        Projections::new(repo)
            .task_queue(include_completed)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_timeline(state: State<'_, AppState>) -> Result<Vec<core_db::TimelineEntry>, String> {
    state.with_db(|conn| {
        let repo = CoreRepository::new(conn);
        Projections::new(repo)
            .timeline_entries()
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_widgets(
    state: State<'_, AppState>,
) -> Result<Vec<core_db::WidgetInstanceProjection>, String> {
    state.with_db(|conn| {
        let repo = CoreRepository::new(conn);
        Projections::new(repo)
            .widget_instances()
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn create_task(title: String, state: State<'_, AppState>) -> Result<String, String> {
    state.with_db(|conn| {
        let device_id = LocalPrefs::new(conn)
            .get(LocalPrefKey::CloudDeviceId)
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "local-device".into());
        let svc = CommandService::new(conn, device_id);
        let record = svc.create_task(title, None).map_err(|e| e.to_string())?;
        Ok(record.id.to_string())
    })
}

#[tauri::command]
pub async fn sync_drain(state: State<'_, AppState>) -> Result<core_sync::DrainReport, String> {
    let (db_path, supabase, workspace_id, device_id) = {
        let supabase = state.supabase.lock().map_err(|e| e.to_string())?;
        if !supabase.is_signed_in() {
            return Err("Not signed in".into());
        }
        let workspace = state.workspace.lock().map_err(|e| e.to_string())?;
        let device_id = state.with_db(|conn| {
            LocalPrefs::new(conn)
                .get(LocalPrefKey::CloudDeviceId)
                .map_err(|e| e.to_string())
        })?;
        (
            state.db_path.clone(),
            supabase.clone(),
            workspace.workspace_id().map(String::from),
            device_id,
        )
    };

    let sync = CoreSyncService::new(
        db_path,
        supabase,
        workspace_id.as_deref(),
        device_id.as_deref(),
    );
    sync.drain_outbox(20).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_pull(state: State<'_, AppState>) -> Result<core_sync::PullResult, String> {
    let (db_path, supabase, workspace_id) = {
        let supabase = state.supabase.lock().map_err(|e| e.to_string())?;
        if !supabase.is_signed_in() {
            return Err("Not signed in".into());
        }
        let workspace = state.workspace.lock().map_err(|e| e.to_string())?;
        (
            state.db_path.clone(),
            supabase.clone(),
            workspace.workspace_id().map(String::from),
        )
    };

    let sync = CoreSyncService::new(db_path, supabase, workspace_id.as_deref(), None);
    sync.pull_server_records().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn widget_catalog() -> Result<core_contracts::WidgetCatalog, String> {
    core_contracts::WidgetCatalog::load().map_err(|e| e.to_string())
}
