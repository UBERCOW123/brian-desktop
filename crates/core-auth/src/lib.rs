//! Supabase authentication, session persistence, workspace + device registration.

mod client;
mod config;
mod device;
mod oauth;
mod session;
mod workspace;

pub use client::{ClientError, SupabaseClient};
pub use config::SupabaseConfig;
pub use device::{platform_label, DeviceRegistrationService};
pub use oauth::{OAuthFlow, OAuthStart};
pub use session::{Session, SessionStore};
pub use workspace::{fetch_default_workspace_id, WorkspaceContext};
