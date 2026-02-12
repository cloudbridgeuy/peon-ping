use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_active_pack")]
    pub active_pack: String,
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub categories: CategoryToggles,
    #[serde(default = "default_annoyed_threshold")]
    pub annoyed_threshold: u32,
    #[serde(default = "default_annoyed_window")]
    pub annoyed_window_seconds: f64,
    #[serde(default)]
    pub pack_rotation: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active_pack: default_active_pack(),
            volume: default_volume(),
            enabled: true,
            categories: CategoryToggles::default(),
            annoyed_threshold: default_annoyed_threshold(),
            annoyed_window_seconds: default_annoyed_window(),
            pack_rotation: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryToggles {
    #[serde(default = "default_true")]
    pub greeting: bool,
    #[serde(default = "default_true")]
    pub acknowledge: bool,
    #[serde(default = "default_true")]
    pub complete: bool,
    #[serde(default = "default_true")]
    pub error: bool,
    #[serde(default = "default_true")]
    pub permission: bool,
    #[serde(default = "default_true")]
    pub resource_limit: bool,
    #[serde(default = "default_true")]
    pub annoyed: bool,
}

impl Default for CategoryToggles {
    fn default() -> Self {
        Self {
            greeting: true,
            acknowledge: true,
            complete: true,
            error: true,
            permission: true,
            resource_limit: true,
            annoyed: true,
        }
    }
}

impl CategoryToggles {
    pub fn is_enabled(&self, category: &str) -> bool {
        match category {
            "greeting" => self.greeting,
            "acknowledge" => self.acknowledge,
            "complete" => self.complete,
            "error" => self.error,
            "permission" => self.permission,
            "resource_limit" => self.resource_limit,
            "annoyed" => self.annoyed,
            _ => true,
        }
    }
}

fn default_active_pack() -> String {
    "peon".to_string()
}

fn default_volume() -> f64 {
    0.5
}

fn default_true() -> bool {
    true
}

fn default_annoyed_threshold() -> u32 {
    3
}

fn default_annoyed_window() -> f64 {
    10.0
}

/// Represents config with unknown/extra fields preserved for round-tripping.
/// Used by shell when updating config to avoid losing unknown keys.
pub type ConfigMap = HashMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_config() {
        let json = r#"{
            "active_pack": "sc_kerrigan",
            "volume": 0.8,
            "enabled": true,
            "categories": {
                "greeting": true,
                "acknowledge": false,
                "complete": true,
                "error": true,
                "permission": true,
                "resource_limit": true,
                "annoyed": false
            },
            "annoyed_threshold": 5,
            "annoyed_window_seconds": 15,
            "pack_rotation": ["peon", "sc_kerrigan"]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.active_pack, "sc_kerrigan");
        assert_eq!(config.volume, 0.8);
        assert!(config.enabled);
        assert!(!config.categories.acknowledge);
        assert!(!config.categories.annoyed);
        assert_eq!(config.annoyed_threshold, 5);
        assert_eq!(config.annoyed_window_seconds, 15.0);
        assert_eq!(config.pack_rotation, vec!["peon", "sc_kerrigan"]);
    }

    #[test]
    fn deserialize_empty_config_uses_defaults() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(config.active_pack, "peon");
        assert_eq!(config.volume, 0.5);
        assert!(config.enabled);
        assert!(config.categories.greeting);
        assert!(config.categories.annoyed);
        assert_eq!(config.annoyed_threshold, 3);
        assert_eq!(config.annoyed_window_seconds, 10.0);
        assert!(config.pack_rotation.is_empty());
    }

    #[test]
    fn category_toggles_is_enabled() {
        let toggles = CategoryToggles {
            greeting: true,
            acknowledge: false,
            ..Default::default()
        };
        assert!(toggles.is_enabled("greeting"));
        assert!(!toggles.is_enabled("acknowledge"));
        assert!(toggles.is_enabled("unknown_category"));
    }
}
