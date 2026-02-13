use peon_core::upgrade::{
    find_matching_asset, get_asset_name, is_version_up_to_date, parse_version_tag, GitHubRelease,
};
use std::io::Read;
use std::path::PathBuf;

const GITHUB_API_URL: &str = "https://api.github.com/repos/tonyyont/peon-ping/releases/latest";

/// Self-update peon from GitHub releases
#[derive(Debug, clap::Parser)]
pub struct App {
    /// Force upgrade even if already up to date
    #[arg(long)]
    pub force: bool,
}

pub fn run(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("peon-ping: current version {current_version}");

    println!("peon-ping: checking for updates...");
    let release = fetch_latest_release()?;
    let latest_version = parse_version_tag(&release.tag_name);

    if !app.force {
        match is_version_up_to_date(current_version, latest_version) {
            Ok(true) => {
                println!("peon-ping: already up to date ({current_version})");
                return Ok(());
            }
            Ok(false) => {
                println!("peon-ping: new version available: {latest_version}");
            }
            Err(e) => {
                eprintln!("peon-ping: warning: version comparison failed: {e}");
                // Continue with upgrade attempt
            }
        }
    } else {
        println!("peon-ping: force upgrade to {latest_version}");
    }

    let asset_name = get_asset_name(std::env::consts::OS, std::env::consts::ARCH)
        .map_err(|e| format!("peon-ping: {e}"))?;

    let asset = find_matching_asset(&release, &asset_name)
        .ok_or_else(|| format!("peon-ping: no asset found for {asset_name}"))?;

    let current_exe = std::env::current_exe()?;
    let download_path = current_exe.with_extension("download");
    let backup_path = current_exe.with_extension("backup");

    println!("peon-ping: downloading {asset_name}...");
    download_file(&asset.browser_download_url, &download_path)?;

    // Backup current binary
    if current_exe.exists() {
        std::fs::copy(&current_exe, &backup_path)?;
    }

    // Replace binary
    match replace_binary(&download_path, &current_exe) {
        Ok(()) => {
            // Clean up backup and download
            let _ = std::fs::remove_file(&backup_path);
            let _ = std::fs::remove_file(&download_path);
            println!("peon-ping: upgraded to {latest_version} (was {current_version})");
            Ok(())
        }
        Err(e) => {
            // Rollback from backup
            eprintln!("peon-ping: upgrade failed: {e}");
            if backup_path.exists() {
                eprintln!("peon-ping: rolling back...");
                if let Err(rb_err) = std::fs::rename(&backup_path, &current_exe) {
                    eprintln!("peon-ping: rollback failed: {rb_err}");
                } else {
                    eprintln!("peon-ping: rolled back successfully");
                }
            }
            let _ = std::fs::remove_file(&download_path);
            Err(e)
        }
    }
}

fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let response = ureq::get(GITHUB_API_URL)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "peon-ping-upgrade")
        .call()?;

    let release: GitHubRelease = response.into_json()?;
    Ok(release)
}

fn download_file(url: &str, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let response = ureq::get(url)
        .set("User-Agent", "peon-ping-upgrade")
        .call()?;

    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;

    std::fs::write(dest, &bytes)?;
    Ok(())
}

fn replace_binary(source: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::rename(source, dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}
