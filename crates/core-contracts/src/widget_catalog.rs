//! Widget catalog parsed from `WIDGET_AGENT_METADATA_SEED.json`.

use crate::paths::{load_widget_seed, ContractError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetCatalog {
    pub schema_version: i32,
    #[serde(default)]
    pub generated_from: Option<String>,
    pub widgets: Vec<WidgetCatalogEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetCatalogEntry {
    #[serde(rename = "type")]
    pub widget_type: String,
    pub display_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub surface_kind: Option<String>,
}

impl WidgetCatalog {
    pub fn load() -> Result<Self, ContractError> {
        let seed = load_widget_seed()?;
        serde_json::from_value(seed).map_err(|source| ContractError::InvalidWidgetSeed { source })
    }

    pub fn widget_types(&self) -> Vec<&str> {
        self.widgets.iter().map(|w| w.widget_type.as_str()).collect()
    }

    pub fn find(&self, widget_type: &str) -> Option<&WidgetCatalogEntry> {
        self.widgets.iter().find(|w| w.widget_type == widget_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_widget_catalog() {
        let catalog = WidgetCatalog::load().expect("catalog");
        assert!(catalog.schema_version >= 1);
        assert!(!catalog.widgets.is_empty());
        assert!(catalog.find("clock_widget").is_some());
    }
}
