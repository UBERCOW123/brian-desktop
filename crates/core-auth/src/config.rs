//! Supabase client configuration (anon key only — never service_role).

use std::env;

pub const DESKTOP_REDIRECT_URI: &str = "com.celix.core.desktop://login-callback";

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub redirect_uri: String,
}

impl SupabaseConfig {
    pub fn empty() -> Self {
        Self {
            url: String::new(),
            anon_key: String::new(),
            redirect_uri: DESKTOP_REDIRECT_URI.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("SUPABASE_URL").map_err(|_| ConfigError::MissingUrl)?;
        let anon_key = env::var("SUPABASE_ANON_KEY").map_err(|_| ConfigError::MissingAnonKey)?;
        let redirect_uri = env::var("OAUTH_REDIRECT_URI")
            .unwrap_or_else(|_| DESKTOP_REDIRECT_URI.to_string());
        if url.is_empty() || anon_key.is_empty() {
            return Err(ConfigError::Empty);
        }
        Ok(Self {
            url: url.trim_end_matches('/').to_string(),
            anon_key,
            redirect_uri,
        })
    }

    pub fn is_configured(&self) -> bool {
        !self.url.is_empty() && !self.anon_key.is_empty()
    }

    pub fn auth_url(&self, path: &str) -> String {
        format!("{}/auth/v1{path}", self.url)
    }

    pub fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{table}", self.url)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("SUPABASE_URL not set")]
    MissingUrl,
    #[error("SUPABASE_ANON_KEY not set")]
    MissingAnonKey,
    #[error("supabase config is empty")]
    Empty,
}
