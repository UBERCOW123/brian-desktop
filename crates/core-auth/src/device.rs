//! Register install in Supabase `devices` table (mirrors mobile).

use crate::client::{ClientError, SupabaseClient};
use core_db::{Connection, LocalPrefError, LocalPrefKey, LocalPrefs};
use serde::Deserialize;

pub struct DeviceRegistrationService;

impl DeviceRegistrationService {
    /// Read cached cloud device id from local prefs (sync).
    pub fn read_cached_device_id(conn: &Connection) -> Result<Option<String>, ClientError> {
        LocalPrefs::new(conn)
            .get(LocalPrefKey::CloudDeviceId)
            .map_err(pref_err)
    }

    /// Persist cloud device id after successful remote registration (sync).
    pub fn write_cached_device_id(conn: &Connection, device_id: &str) -> Result<(), ClientError> {
        LocalPrefs::new(conn)
            .set(LocalPrefKey::CloudDeviceId, device_id)
            .map_err(pref_err)
    }

    /// Register or refresh device without holding SQLite across network I/O.
    pub async fn ensure_registered(
        client: &SupabaseClient,
        cached_device_id: Option<String>,
        workspace_id: &str,
        platform: &str,
        label: &str,
    ) -> Result<Option<String>, ClientError> {
        if let Some(stored) = cached_device_id.filter(|s| !s.is_empty()) {
            let _ = touch_last_seen(client, &stored).await;
            return Ok(Some(stored));
        }

        register_new_device(client, workspace_id, platform, label).await
    }

    pub fn clear_cache(conn: &Connection) -> Result<(), ClientError> {
        LocalPrefs::new(conn)
            .remove(LocalPrefKey::CloudDeviceId)
            .map_err(pref_err)?;
        Ok(())
    }
}

async fn register_new_device(
    client: &SupabaseClient,
    workspace_id: &str,
    platform: &str,
    label: &str,
) -> Result<Option<String>, ClientError> {
    let user_id = client.user_id().ok_or(ClientError::NotSignedIn)?.to_string();
    let body = serde_json::json!({
        "user_id": user_id,
        "workspace_id": workspace_id,
        "label": label,
        "platform": platform,
        "last_seen_at": chrono::Utc::now().to_rfc3339(),
    });
    let headers = client.rest_headers()?;
    let url = format!("{}?select=id", client.config().rest_url("devices"));
    let resp = reqwest::Client::new()
        .post(url)
        .headers(headers)
        .header("Prefer", "return=representation")
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    let rows: Vec<DeviceRow> = resp.json().await?;
    Ok(rows.into_iter().next().map(|r| r.id))
}

async fn touch_last_seen(client: &SupabaseClient, device_id: &str) -> Result<(), ClientError> {
    let headers = client.rest_headers()?;
    let url = format!("{}?id=eq.{device_id}", client.config().rest_url("devices"));
    let body = serde_json::json!({ "last_seen_at": chrono::Utc::now().to_rfc3339() });
    let _ = reqwest::Client::new()
        .patch(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    Ok(())
}

fn pref_err(e: LocalPrefError) -> ClientError {
    ClientError::Other(e.to_string())
}

#[derive(Debug, Deserialize)]
struct DeviceRow {
    id: String,
}

pub fn platform_label() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}
