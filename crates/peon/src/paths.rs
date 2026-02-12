use std::path::PathBuf;

/// Resolve the peon-ping data directory.
///
/// Uses `CLAUDE_PEON_DIR` env var if set (for testing), otherwise `~/.claude/hooks/peon-ping`.
pub fn peon_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_PEON_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("hooks")
        .join("peon-ping")
}

pub fn config_path() -> PathBuf {
    peon_dir().join("config.json")
}

pub fn state_path() -> PathBuf {
    peon_dir().join(".state.json")
}

pub fn paused_path() -> PathBuf {
    peon_dir().join(".paused")
}

pub fn packs_dir() -> PathBuf {
    peon_dir().join("packs")
}
