//! Session persistence in OS keychain.

use serde::{Deserialize, Serialize};

const SERVICE: &str = "brian-desktop";
const ACCOUNT: &str = "supabase-session";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user_id: String,
    pub email: Option<String>,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at <= now
    }
}

pub struct SessionStore;

impl SessionStore {
    pub fn load() -> Result<Option<Session>, SessionError> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(SessionError::Keyring)?;
        match entry.get_password() {
            Ok(raw) => {
                let session: Session = serde_json::from_str(&raw).map_err(SessionError::Json)?;
                Ok(Some(session))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SessionError::Keyring(e)),
        }
    }

    pub fn save(session: &Session) -> Result<(), SessionError> {
        let raw = serde_json::to_string(session).map_err(SessionError::Json)?;
        let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(SessionError::Keyring)?;
        entry.set_password(&raw).map_err(SessionError::Keyring)?;
        Ok(())
    }

    pub fn clear() -> Result<(), SessionError> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(SessionError::Keyring)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SessionError::Keyring(e)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("keyring: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
