//! PKCE OAuth flow for Sign in with Apple (browser on Windows).

use crate::config::SupabaseConfig;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OAuthStart {
    pub authorize_url: String,
    pub code_verifier: String,
}

pub struct OAuthFlow;

impl OAuthFlow {
    pub fn start_apple(config: &SupabaseConfig) -> OAuthStart {
        let code_verifier = generate_verifier();
        let challenge = challenge_s256(&code_verifier);
        let redirect = urlencoding::encode(&config.redirect_uri);
        let authorize_url = format!(
            "{}?provider=apple&redirect_to={redirect}&code_challenge={challenge}&code_challenge_method=s256",
            config.auth_url("/authorize"),
        );
        OAuthStart {
            authorize_url,
            code_verifier,
        }
    }

    pub fn parse_callback_code(callback_url: &str) -> Result<String, OAuthError> {
        let parsed = url::Url::parse(callback_url).map_err(|_| OAuthError::InvalidCallback)?;
        let mut code = None;
        for (k, v) in parsed.query_pairs() {
            if k == "code" {
                code = Some(v.into_owned());
            }
        }
        code.ok_or(OAuthError::MissingCode)
    }
}

pub fn generate_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn challenge_s256(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("invalid OAuth callback URL")]
    InvalidCallback,
    #[error("authorization code missing from callback")]
    MissingCode,
}

// Re-export for token exchange body
pub type TokenExchangeBody = HashMap<&'static str, String>;
