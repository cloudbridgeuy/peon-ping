use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Strip leading "v" from a version tag.
/// Returns the version string without the prefix.
pub fn parse_version_tag(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

/// Compare two semver strings. Returns Ok(true) if `current` >= `latest`.
pub fn is_version_up_to_date(current: &str, latest: &str) -> Result<bool, String> {
    let current_parts = parse_semver(current)?;
    let latest_parts = parse_semver(latest)?;
    Ok(current_parts >= latest_parts)
}

fn parse_semver(version: &str) -> Result<(u64, u64, u64), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("invalid semver: {version}"));
    }
    let major = parts[0]
        .parse::<u64>()
        .map_err(|_| format!("invalid major version: {}", parts[0]))?;
    let minor = parts[1]
        .parse::<u64>()
        .map_err(|_| format!("invalid minor version: {}", parts[1]))?;
    let patch = parts[2]
        .parse::<u64>()
        .map_err(|_| format!("invalid patch version: {}", parts[2]))?;
    Ok((major, minor, patch))
}

/// Get the expected asset name for a given OS and architecture.
pub fn get_asset_name(os: &str, arch: &str) -> Result<String, String> {
    let target_arch = match arch {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        _ => return Err(format!("unsupported architecture: {arch}")),
    };

    match os {
        "macos" => Ok(format!("peon-{target_arch}-apple-darwin")),
        _ => Err(format!("unsupported OS: {os}")),
    }
}

/// Find a matching asset in a release by name.
pub fn find_matching_asset<'a>(
    release: &'a GitHubRelease,
    asset_name: &str,
) -> Option<&'a GitHubAsset> {
    release.assets.iter().find(|a| a.name == asset_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_tag() {
        assert_eq!(parse_version_tag("v1.2.0"), "1.2.0");
        assert_eq!(parse_version_tag("1.2.0"), "1.2.0");
        assert_eq!(parse_version_tag("v0.0.1"), "0.0.1");
    }

    #[test]
    fn test_is_version_up_to_date() {
        assert!(is_version_up_to_date("2.0.0", "2.0.0").unwrap());
        assert!(is_version_up_to_date("2.1.0", "2.0.0").unwrap());
        assert!(!is_version_up_to_date("2.0.0", "2.0.1").unwrap());
        assert!(!is_version_up_to_date("1.9.9", "2.0.0").unwrap());
        assert!(is_version_up_to_date("3.0.0", "2.9.9").unwrap());
    }

    #[test]
    fn test_is_version_up_to_date_invalid() {
        assert!(is_version_up_to_date("abc", "1.0.0").is_err());
        assert!(is_version_up_to_date("1.0", "1.0.0").is_err());
    }

    #[test]
    fn test_get_asset_name() {
        assert_eq!(
            get_asset_name("macos", "aarch64").unwrap(),
            "peon-aarch64-apple-darwin"
        );
        assert_eq!(
            get_asset_name("macos", "x86_64").unwrap(),
            "peon-x86_64-apple-darwin"
        );
        assert!(get_asset_name("linux", "aarch64").is_err());
        assert!(get_asset_name("macos", "arm").is_err());
    }

    #[test]
    fn test_find_matching_asset() {
        let release = GitHubRelease {
            tag_name: "v2.0.0".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "peon-aarch64-apple-darwin".to_string(),
                    browser_download_url: "https://example.com/aarch64".to_string(),
                },
                GitHubAsset {
                    name: "peon-x86_64-apple-darwin".to_string(),
                    browser_download_url: "https://example.com/x86_64".to_string(),
                },
            ],
        };

        let asset = find_matching_asset(&release, "peon-aarch64-apple-darwin");
        assert!(asset.is_some());
        assert_eq!(
            asset.unwrap().browser_download_url,
            "https://example.com/aarch64"
        );

        assert!(find_matching_asset(&release, "peon-linux-amd64").is_none());
    }

    #[test]
    fn test_find_matching_asset_empty() {
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            assets: vec![],
        };
        assert!(find_matching_asset(&release, "peon-aarch64-apple-darwin").is_none());
    }
}
