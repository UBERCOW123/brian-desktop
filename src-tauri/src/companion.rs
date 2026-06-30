use core_db::{CommandService, LocalPrefKey, LocalPrefs};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::State;

use crate::app_state::AppState;

const BROWSER_STATE_KEY: &str = "desktop_browser_state";
const IDE_STATE_KEY: &str = "desktop_ide_state";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub url: String,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub history_index: i32,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            url: "https://example.com".into(),
            history: vec!["https://example.com".into()],
            history_index: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdeState {
    pub root_path: Option<String>,
    pub open_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdeFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

fn command_service_for<'a>(
    conn: &'a core_db::Connection,
) -> Result<CommandService<'a>, String> {
    let device_id = LocalPrefs::new(conn)
        .get(LocalPrefKey::CloudDeviceId)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "local-device".into());
    Ok(CommandService::new(conn, device_id))
}

fn normalize_http_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL is required".into());
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let parsed = url::Url::parse(&with_scheme).map_err(|e| e.to_string())?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("Only http and https URLs are allowed".into());
    }
    Ok(parsed.to_string())
}

pub fn validate_path_under_root(root: &Path, target: &Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Invalid root folder: {e}"))?;
    let target = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| format!("Invalid path: {e}"))?
    } else {
        let parent = target
            .parent()
            .ok_or_else(|| "Invalid path".to_string())?
            .canonicalize()
            .map_err(|e| format!("Invalid path: {e}"))?;
        let joined = parent.join(
            target
                .file_name()
                .ok_or_else(|| "Invalid path".to_string())?,
        );
        joined
    };
    if !target.starts_with(&root) {
        return Err("Path is outside the granted folder".into());
    }
    Ok(target)
}

fn load_ide_root(state: &AppState) -> Result<Option<PathBuf>, String> {
    state.with_db(|conn| {
        let value = command_service_for(conn)?
            .get_app_setting_json(IDE_STATE_KEY)
            .map_err(|e| e.to_string())?;
        Ok(value
            .and_then(|v| serde_json::from_value::<IdeState>(v).ok())
            .and_then(|s| s.root_path.map(PathBuf::from)))
    })
}

#[tauri::command]
pub fn get_browser_state(state: State<'_, AppState>) -> Result<BrowserState, String> {
    state.with_db(|conn| {
        let value = command_service_for(conn)?
            .get_app_setting_json(BROWSER_STATE_KEY)
            .map_err(|e| e.to_string())?;
        Ok(value
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default())
    })
}

#[tauri::command]
pub fn set_browser_state(payload: BrowserState, state: State<'_, AppState>) -> Result<(), String> {
    let url = normalize_http_url(&payload.url)?;
    let mut next = payload;
    next.url = url;
    state.with_db(|conn| {
        command_service_for(conn)?
            .set_app_setting_json(
                BROWSER_STATE_KEY,
                serde_json::to_value(&next).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<Vec<core_db::NotebookNote>, String> {
    state.with_db(|conn| {
        let repo = core_db::CoreRepository::new(conn);
        core_db::Projections::new(repo)
            .notebook_notes()
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn create_note(title: String, body: String, state: State<'_, AppState>) -> Result<String, String> {
    state.with_db(|conn| {
        let record = command_service_for(conn)?
            .create_note(title, body)
            .map_err(|e| e.to_string())?;
        Ok(record.id.to_string())
    })
}

#[tauri::command]
pub fn update_note(
    id: String,
    title: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let note_id = uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.with_db(|conn| {
        command_service_for(conn)?
            .update_note(note_id, title, body)
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn get_ide_state(state: State<'_, AppState>) -> Result<IdeState, String> {
    state.with_db(|conn| {
        let value = command_service_for(conn)?
            .get_app_setting_json(IDE_STATE_KEY)
            .map_err(|e| e.to_string())?;
        Ok(value
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default())
    })
}

#[tauri::command]
pub fn set_ide_state(payload: IdeState, state: State<'_, AppState>) -> Result<(), String> {
    state.with_db(|conn| {
        command_service_for(conn)?
            .set_app_setting_json(
                IDE_STATE_KEY,
                serde_json::to_value(&payload).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn pick_ide_folder(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let picked = rfd::FileDialog::new().pick_folder();
    let Some(path) = picked else {
        return Ok(None);
    };
    let root = path.to_string_lossy().to_string();
    state.with_db(|conn| {
        let svc = command_service_for(conn)?;
        let mut ide_state: IdeState = svc
            .get_app_setting_json(IDE_STATE_KEY)
            .map_err(|e| e.to_string())?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        ide_state.root_path = Some(root.clone());
        ide_state.open_file = None;
        svc.set_app_setting_json(
            IDE_STATE_KEY,
            serde_json::to_value(&ide_state).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    })?;
    Ok(Some(root))
}

#[tauri::command]
pub fn list_ide_files(state: State<'_, AppState>) -> Result<Vec<IdeFileEntry>, String> {
    let root = load_ide_root(&state)?.ok_or("No folder selected")?;
    let mut entries = Vec::new();
    collect_ide_entries(&root, &root, 0, 4, &mut entries)?;
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        }
    });
    Ok(entries)
}

fn collect_ide_entries(
    root: &Path,
    dir: &Path,
    depth: i32,
    max_depth: i32,
    out: &mut Vec<IdeFileEntry>,
) -> Result<(), String> {
    if depth > max_depth {
        return Ok(());
    }
    let read_dir = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let rel = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        out.push(IdeFileEntry {
            name,
            path: rel,
            is_dir: meta.is_dir(),
        });
        if meta.is_dir() {
            collect_ide_entries(root, &path, depth + 1, max_depth, out)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn read_ide_file(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let root = load_ide_root(&state)?.ok_or("No folder selected")?;
    let target = validate_path_under_root(&root, &root.join(&path))?;
    if target.is_dir() {
        return Err("Cannot read a directory".into());
    }
    fs::read_to_string(&target).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_ide_file(path: String, contents: String, state: State<'_, AppState>) -> Result<(), String> {
    let root = load_ide_root(&state)?.ok_or("No folder selected")?;
    let target = root.join(&path);
    if target
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Invalid path".into());
    }
    let validated = validate_path_under_root(&root, &target)?;
    if let Some(parent) = validated.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&validated, contents).map_err(|e| e.to_string())
}
