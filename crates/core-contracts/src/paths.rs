//! Vendored contract file paths (`vendor/core` submodule).

use std::path::PathBuf;

/// Workspace-relative path to the `vendor/core` submodule root.
pub fn vendor_core_root() -> PathBuf {
    PathBuf::from(env!("VENDOR_CORE_ROOT"))
}

pub fn sync_strategy_path() -> PathBuf {
    vendor_core_root().join("docs/agent/SYNC_STRATEGY.md")
}

pub fn data_model_path() -> PathBuf {
    vendor_core_root().join("docs/agent/CORE_DATA_MODEL.md")
}

pub fn widget_seed_path() -> PathBuf {
    vendor_core_root().join("docs/agent/WIDGET_AGENT_METADATA_SEED.json")
}

pub fn supabase_migrations_dir() -> PathBuf {
    vendor_core_root().join("supabase/migrations")
}

pub fn supabase_env_example_path() -> PathBuf {
    vendor_core_root().join("supabase.local.json.example")
}

pub fn parity_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/parity_manifest.json")
}

pub fn contracts_present() -> bool {
    [
        sync_strategy_path(),
        data_model_path(),
        widget_seed_path(),
        supabase_env_example_path(),
        parity_manifest_path(),
    ]
    .iter()
    .all(|p| p.is_file())
        && supabase_migrations_dir().is_dir()
}

pub fn assert_contracts_present() -> Result<(), ContractError> {
    if contracts_present() {
        Ok(())
    } else {
        Err(ContractError::MissingArtifacts {
            root: vendor_core_root(),
        })
    }
}

pub fn load_widget_seed() -> Result<serde_json::Value, ContractError> {
    let path = widget_seed_path();
    let raw = std::fs::read_to_string(&path).map_err(|source| ContractError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|source| ContractError::InvalidWidgetSeed { source })
}

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("vendor/core contracts missing at {root}; run `git submodule update --init`")]
    MissingArtifacts { root: PathBuf },
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid widget seed JSON: {source}")]
    InvalidWidgetSeed {
        #[source]
        source: serde_json::Error,
    },
}
