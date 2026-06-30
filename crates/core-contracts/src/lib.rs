//! CORE contracts — kinds, models, validation, and vendored doc paths.
//!
//! Parity with mobile `core_models.dart` is enforced via `contracts/fixtures/parity_manifest.json`.

mod contract;
mod kinds;
mod models;
mod paths;
mod widget_catalog;

pub use contract::{ContractError as RecordContractError, RecordContract};
pub use kinds::{CoreEventKind, CoreLinkKind, CoreRecordKind, SyncOutboxStatus};
pub use models::{parse_outbox_status, parse_record_kind, CoreEvent, CoreLink, CoreRecord, SyncOutboxItem};
pub use paths::{
    assert_contracts_present, contracts_present, data_model_path, load_widget_seed,
    parity_manifest_path, supabase_env_example_path, supabase_migrations_dir, sync_strategy_path,
    vendor_core_root, widget_seed_path, ContractError as VendorContractError,
};
pub use widget_catalog::{WidgetCatalog, WidgetCatalogEntry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_contracts_exist() {
        assert_contracts_present().expect("init submodules: git submodule update --init");
    }

    #[test]
    fn widget_seed_parses() {
        let seed = load_widget_seed().expect("widget seed");
        assert!(seed.is_object() || seed.is_array());
    }
}
