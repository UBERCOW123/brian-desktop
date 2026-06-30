//! Widget catalog parsed from mobile seed + desktop shell extensions.

use crate::paths::{load_widget_seed, ContractError};
use serde::{Deserialize, Serialize};

const DESKTOP_CATALOG_JSON: &str = include_str!("../desktop_widget_catalog.json");

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
pub struct WidgetSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetCatalogEntry {
    #[serde(rename(serialize = "widgetType", deserialize = "type"))]
    pub widget_type: String,
    pub display_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub surface_kind: Option<String>,
    #[serde(default = "default_allow_multiple")]
    pub allow_multiple: bool,
    #[serde(default)]
    pub default_size: Option<WidgetSize>,
}

fn default_allow_multiple() -> bool {
    true
}

impl WidgetCatalogEntry {
    pub fn default_width(&self) -> i32 {
        self.default_size.as_ref().map(|s| s.width).unwrap_or(2)
    }

    pub fn default_height(&self) -> i32 {
        self.default_size.as_ref().map(|s| s.height).unwrap_or(2)
    }
}

impl WidgetCatalog {
    pub fn load() -> Result<Self, ContractError> {
        let seed = load_widget_seed()?;
        let mut catalog: WidgetCatalog =
            serde_json::from_value(seed).map_err(|source| ContractError::InvalidWidgetSeed { source })?;
        let desktop: WidgetCatalog = serde_json::from_str(DESKTOP_CATALOG_JSON)
            .map_err(|source| ContractError::InvalidWidgetSeed { source })?;
        for entry in desktop.widgets {
            if catalog.find(&entry.widget_type).is_none() {
                catalog.widgets.push(entry);
            }
        }
        Ok(catalog)
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

    #[test]
    fn serializes_widget_type_as_widget_type_camel_case() {
        let catalog = WidgetCatalog::load().expect("catalog");
        let entry = catalog.find("clock_widget").expect("clock_widget");
        let json = serde_json::to_value(entry).expect("serialize");
        assert_eq!(json.get("widgetType").and_then(|v| v.as_str()), Some("clock_widget"));
        assert!(json.get("type").is_none());
    }

    #[test]
    fn loads_desktop_shell_widgets() {
        let catalog = WidgetCatalog::load().expect("catalog");
        assert!(catalog.find("desktop_assist").is_some());
        assert!(catalog.find("desktop_timeline").is_some());
        assert!(catalog.find("desktop_sync").is_some());
    }
}
