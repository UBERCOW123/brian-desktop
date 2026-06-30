//! Authenticated Supabase HTTP client (REST + Auth).

use crate::config::SupabaseConfig;
use crate::oauth::OAuthError;
use crate::session::{Session, SessionStore};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct SupabaseClient {
    config: SupabaseConfig,
    http: Arc<reqwest::Client>,
    session: Option<Session>,
}

impl SupabaseClient {
    pub fn new(config: SupabaseConfig) -> Self {
        Self {
            config,
            http: Arc::new(reqwest::Client::new()),
            session: SessionStore::load().ok().flatten(),
        }
    }

    pub fn config(&self) -> &SupabaseConfig {
        &self.config
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn user_id(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.user_id.as_str())
    }

    pub fn is_signed_in(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| !s.is_expired())
    }

    pub fn reload_session(&mut self) -> Result<(), ClientError> {
        self.session = SessionStore::load()?;
        Ok(())
    }

    pub async fn exchange_pkce_code(
        &mut self,
        code: &str,
        code_verifier: &str,
    ) -> Result<Session, ClientError> {
        let body = serde_json::json!({
            "auth_code": code,
            "code_verifier": code_verifier,
        });
        let url = self.config.auth_url("/token?grant_type=pkce");
        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let token: TokenResponse = resp.json().await?;
        let session = Session {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at: chrono::Utc::now().timestamp() + token.expires_in,
            user_id: token.user.id,
            email: token.user.email,
        };
        SessionStore::save(&session)?;
        self.session = Some(session.clone());
        Ok(session)
    }

    pub async fn sign_out(&mut self) -> Result<(), ClientError> {
        SessionStore::clear()?;
        self.session = None;
        Ok(())
    }

    pub fn rest_headers(&self) -> Result<HeaderMap, ClientError> {
        let session = self.session.as_ref().ok_or(ClientError::NotSignedIn)?;
        let mut headers = HeaderMap::new();
        headers.insert("apikey", HeaderValue::from_str(&self.config.anon_key)?);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", session.access_token))?,
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    pub async fn get_json(
        &self,
        table: &str,
        query: &str,
    ) -> Result<serde_json::Value, ClientError> {
        let headers = self.rest_headers()?;
        let url = format!("{}{query}", self.config.rest_url(table));
        let resp = self.http.get(url).headers(headers).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn post_json(
        &self,
        table: &str,
        body: serde_json::Value,
        prefer: Option<&str>,
    ) -> Result<(), ClientError> {
        let mut headers = self.rest_headers()?;
        if let Some(p) = prefer {
            headers.insert("Prefer", HeaderValue::from_str(p)?);
        }
        let url = self.config.rest_url(table);
        self.http
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn upsert_json(
        &self,
        table: &str,
        body: serde_json::Value,
    ) -> Result<(), ClientError> {
        self.post_json(table, body, Some("resolution=merge-duplicates")).await
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: UserResponse,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    id: String,
    email: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("not signed in")]
    NotSignedIn,
    #[error("oauth: {0}")]
    OAuth(#[from] OAuthError),
    #[error("session: {0}")]
    Session(#[from] crate::session::SessionError),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("header: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("{0}")]
    Other(String),
}
