#![cfg_attr(not(test), deny(clippy::unwrap_used))]

use std::io::Read;
use std::path::Path;

const GITHUB_API_BASE: &str = "https://api.github.com/repos/cloudbridgeuy/peon-ping/contents/packs";

#[derive(thiserror::Error, Debug)]
pub enum GithubError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] Box<ureq::Error>),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(serde::Deserialize)]
struct GithubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    download_url: Option<String>,
}

pub struct PullResult {
    pub name: String,
    pub files: usize,
}

/// List all pack names available on GitHub.
pub fn list_remote_packs() -> Result<Vec<String>, GithubError> {
    let contents = fetch_contents(GITHUB_API_BASE)?;
    let packs = contents
        .into_iter()
        .filter(|c| c.content_type == "dir")
        .map(|c| c.name)
        .collect();
    Ok(packs)
}

/// Download a single pack from GitHub to the local packs directory.
/// Creates `<dest_dir>/<pack_name>/manifest.json` and `<dest_dir>/<pack_name>/sounds/*`.
pub fn pull_pack(pack_name: &str, dest_dir: &Path) -> Result<PullResult, GithubError> {
    let pack_url = format!("{GITHUB_API_BASE}/{pack_name}");
    let contents = fetch_contents(&pack_url)?;

    let pack_dir = dest_dir.join(pack_name);
    std::fs::create_dir_all(&pack_dir)?;

    let mut file_count = 0;

    for item in &contents {
        if item.content_type == "file" {
            // Top-level files (manifest.json)
            if let Some(url) = &item.download_url {
                let bytes = download_raw(url)?;
                std::fs::write(pack_dir.join(&item.name), &bytes)?;
                file_count += 1;
            }
        } else if item.content_type == "dir" && item.name == "sounds" {
            // Recurse into sounds/ directory
            let sounds_url = format!("{pack_url}/sounds");
            let sound_contents = fetch_contents(&sounds_url)?;
            let sounds_dir = pack_dir.join("sounds");
            std::fs::create_dir_all(&sounds_dir)?;

            for sound_item in &sound_contents {
                if sound_item.content_type == "file" {
                    if let Some(url) = &sound_item.download_url {
                        let bytes = download_raw(url)?;
                        std::fs::write(sounds_dir.join(&sound_item.name), &bytes)?;
                        file_count += 1;
                    }
                }
            }
        }
    }

    Ok(PullResult {
        name: pack_name.to_string(),
        files: file_count,
    })
}

fn github_get(url: &str) -> Result<ureq::Response, GithubError> {
    let mut request = ureq::get(url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "peon-ping");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    request.call().map_err(|e| GithubError::Http(Box::new(e)))
}

fn fetch_contents(url: &str) -> Result<Vec<GithubContent>, GithubError> {
    let response = github_get(url)?;
    let contents: Vec<GithubContent> = response.into_json()?;
    Ok(contents)
}

fn download_raw(url: &str) -> Result<Vec<u8>, GithubError> {
    let response = github_get(url)?;
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}
