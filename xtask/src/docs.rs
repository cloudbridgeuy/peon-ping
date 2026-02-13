use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DocsError>;

#[derive(Error, Debug)]
pub enum DocsError {
    #[error("packs/ directory not found — run from repo root")]
    PacksDirNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error in {path}: {source}")]
    Json {
        path: String,
        source: serde_json::Error,
    },
}

/// Generate SOUNDS.md from pack manifests
#[derive(Debug, clap::Parser)]
#[command(
    long_about = "Generate SOUNDS.md documentation from all sound pack manifests.

Reads every packs/*/manifest.json file and produces a SOUNDS.md at the repo root
listing all packs, categories, and voice lines."
)]
pub struct DocsCommand {}

// Minimal types — only what we need to read manifests
#[derive(Deserialize)]
struct Manifest {
    name: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    categories: HashMap<String, Category>,
}

#[derive(Deserialize)]
struct Category {
    #[serde(default)]
    sounds: Vec<Sound>,
}

#[derive(Deserialize)]
struct Sound {
    #[serde(default)]
    line: String,
}

pub async fn run(_cmd: DocsCommand, global: crate::Global) -> Result<()> {
    if !global.is_silent() {
        aprintln!("{} Generating SOUNDS.md...", p_b("docs:"));
    }

    let packs_dir = Path::new("packs");
    if !packs_dir.exists() {
        return Err(DocsError::PacksDirNotFound);
    }

    let mut packs: Vec<(String, Manifest)> = Vec::new();

    for entry in std::fs::read_dir(packs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                let manifest: Manifest =
                    serde_json::from_str(&content).map_err(|e| DocsError::Json {
                        path: manifest_path.display().to_string(),
                        source: e,
                    })?;
                packs.push((manifest.name.clone(), manifest));
            }
        }
    }

    packs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut md = String::from("# Sound Packs\n\n");
    md.push_str(&format!("{} packs available.\n", packs.len()));

    for (_, manifest) in &packs {
        let display = if manifest.display_name.is_empty() {
            &manifest.name
        } else {
            &manifest.display_name
        };

        md.push_str(&format!("\n## {} (`{}`)\n", display, manifest.name));

        if manifest.categories.is_empty() {
            md.push_str("\nNo categories.\n");
            continue;
        }

        let mut categories: Vec<_> = manifest.categories.iter().collect();
        categories.sort_by_key(|(name, _)| name.to_owned().clone());

        for (cat_name, category) in categories {
            let count = category.sounds.len();
            let label = if count == 1 { "sound" } else { "sounds" };
            md.push_str(&format!("\n### {} ({} {})\n\n", cat_name, count, label));
            for sound in &category.sounds {
                if !sound.line.is_empty() {
                    md.push_str(&format!("- \"{}\"\n", sound.line));
                }
            }
        }
    }

    std::fs::write("SOUNDS.md", &md)?;

    if !global.is_silent() {
        aprintln!(
            "{} Generated SOUNDS.md ({} packs)",
            p_g("done:"),
            packs.len()
        );
    }

    Ok(())
}
