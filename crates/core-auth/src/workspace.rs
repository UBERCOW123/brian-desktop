//! Resolve default personal workspace for signed-in user.

use crate::client::{ClientError, SupabaseClient};
use serde::Deserialize;

pub struct WorkspaceContext {
    workspace_id: Option<String>,
}

impl WorkspaceContext {
    pub fn new() -> Self {
        Self {
            workspace_id: None,
        }
    }

    pub fn workspace_id(&self) -> Option<&str> {
        self.workspace_id.as_deref()
    }

    pub fn clear(&mut self) {
        self.workspace_id = None;
    }

    pub fn set_workspace_id(&mut self, id: Option<String>) {
        self.workspace_id = id;
    }

    pub async fn load_default(&mut self, client: &SupabaseClient) -> Result<Option<String>, ClientError> {
        let id = fetch_default_workspace_id(client).await?;
        self.workspace_id = id.clone();
        Ok(id)
    }
}

pub async fn fetch_default_workspace_id(
    client: &SupabaseClient,
) -> Result<Option<String>, ClientError> {
    let user_id = client.user_id().ok_or(ClientError::NotSignedIn)?;
    let query = format!("?select=id&owner_id=eq.{user_id}&order=created_at.asc&limit=1");
    let rows = client.get_json("workspaces", &query).await?;
    Ok(rows
        .as_array()
        .and_then(|a| a.first())
        .and_then(|r| r.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from))
}

impl Default for WorkspaceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct WorkspaceRow {
    id: String,
}
