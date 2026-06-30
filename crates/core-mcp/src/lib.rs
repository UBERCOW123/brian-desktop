//! MCP tool registry stubs — port from `vendor/core/lib/core/mcp/`.
//!
//! Desktop adds an OS capability tier (filesystem, shell, clipboard) behind preview/confirm.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolTier {
    /// In-app CORE mutations (tasks, widgets, captures).
    Core,
    /// Desktop-only OS automation — always requires explicit user confirm.
    Os,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub tier: ToolTier,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("preview required before execution")]
    PreviewRequired,
}

/// Placeholder registry — populated from mobile `core_mcp_registry.dart` in Phase 3.
pub fn core_tool_stubs() -> Vec<ToolDescriptor> {
    vec![ToolDescriptor {
        name: "list_tasks".into(),
        description: "List active tasks from local CORE projections".into(),
        tier: ToolTier::Core,
        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
    }]
}
