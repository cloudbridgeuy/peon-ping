use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    #[serde(default)]
    pub sounds: Vec<Sound>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sound {
    pub file: String,
    #[serde(default)]
    pub line: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_manifest() {
        let json = r#"{
            "name": "peon",
            "display_name": "Orc Peon",
            "categories": {
                "greeting": {
                    "sounds": [
                        {"file": "PeonReady1.wav", "line": "Ready to work?"},
                        {"file": "PeonWhat1.wav", "line": "Yes?"}
                    ]
                },
                "annoyed": {
                    "sounds": [
                        {"file": "PeonAngry1.wav", "line": "Whaaat?"}
                    ]
                }
            }
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "peon");
        assert_eq!(manifest.display_name, "Orc Peon");
        assert_eq!(manifest.categories["greeting"].sounds.len(), 2);
        assert_eq!(
            manifest.categories["greeting"].sounds[0].file,
            "PeonReady1.wav"
        );
        assert_eq!(manifest.categories["annoyed"].sounds[0].line, "Whaaat?");
    }

    #[test]
    fn deserialize_manifest_missing_categories() {
        let json = r#"{"name": "minimal", "display_name": "Minimal Pack"}"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert!(manifest.categories.is_empty());
    }
}
