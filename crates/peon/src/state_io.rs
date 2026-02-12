use peon_core::types::{Config, ConfigMap, Manifest, State};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum StateIoError {
    #[error("Failed to read file at {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to write file at {path}: {source}")]
    WriteFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse JSON at {path}: {source}")]
    ParseJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Load config from disk. Returns default if file doesn't exist or is invalid.
pub fn load_config(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Load config as a raw map for round-trip editing (preserves unknown keys).
pub fn load_config_map(path: &Path) -> ConfigMap {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ConfigMap::new(),
    }
}

/// Save config map to disk.
pub fn save_config_map(path: &Path, map: &ConfigMap) -> Result<(), StateIoError> {
    let content = serde_json::to_string_pretty(map).map_err(|e| StateIoError::ParseJson {
        path: path.display().to_string(),
        source: e,
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StateIoError::WriteFile {
            path: path.display().to_string(),
            source: e,
        })?;
    }
    std::fs::write(path, content).map_err(|e| StateIoError::WriteFile {
        path: path.display().to_string(),
        source: e,
    })
}

/// Load state from disk. Returns default if file doesn't exist or is invalid.
pub fn load_state(path: &Path) -> State {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => State::default(),
    }
}

/// Save state to disk.
pub fn save_state(path: &Path, state: &State) -> Result<(), StateIoError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StateIoError::WriteFile {
            path: path.display().to_string(),
            source: e,
        })?;
    }
    let content = serde_json::to_string(state).map_err(|e| StateIoError::ParseJson {
        path: path.display().to_string(),
        source: e,
    })?;
    std::fs::write(path, content).map_err(|e| StateIoError::WriteFile {
        path: path.display().to_string(),
        source: e,
    })
}

/// Check if the paused file exists.
pub fn is_paused(path: &Path) -> bool {
    path.exists()
}

/// Load a manifest from a pack directory.
pub fn load_manifest(pack_dir: &Path) -> Result<Manifest, StateIoError> {
    let manifest_path = pack_dir.join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| StateIoError::ReadFile {
        path: manifest_path.display().to_string(),
        source: e,
    })?;
    serde_json::from_str(&content).map_err(|e| StateIoError::ParseJson {
        path: manifest_path.display().to_string(),
        source: e,
    })
}

/// List all available pack names by scanning the packs directory for manifest.json files.
pub fn list_packs(packs_dir: &Path) -> Vec<(String, Manifest)> {
    let mut packs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(packs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(manifest) = load_manifest(&path) {
                    packs.push((manifest.name.clone(), manifest));
                }
            }
        }
    }
    packs.sort_by(|a, b| a.0.cmp(&b.0));
    packs
}
