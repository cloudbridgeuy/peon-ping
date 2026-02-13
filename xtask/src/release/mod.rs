use crate::prelude::*;
use error::{ReleaseError, Result};

pub mod error;

const GITHUB_REPO: &str = "tonyyont/peon-ping";
const WORKFLOW_FILE: &str = "release.yml";
const WORKFLOW_CHECK_INTERVAL: u64 = 30;
const WORKFLOW_TIMEOUT: u64 = 1800;
const MAX_RETRIES: u32 = 3;

/// Release automation for peon-ping
#[derive(Debug, clap::Parser)]
#[command(
    long_about = "Automate the full release flow: validate, bump version, commit, tag, push, monitor CI."
)]
pub struct ReleaseCommand {
    /// Version to release (e.g. "2.1.0")
    pub version: Option<String>,

    /// Clean up a failed release by deleting a tag
    #[arg(long)]
    pub cleanup: Option<String>,

    /// Run `peon upgrade` after release completes
    #[arg(long)]
    pub auto_upgrade: bool,

    /// Skip workflow monitoring
    #[arg(long)]
    pub no_monitor: bool,
}

pub async fn run(command: ReleaseCommand, global: crate::Global) -> Result<()> {
    if let Some(tag) = &command.cleanup {
        return cleanup_command(tag, &global).await;
    }

    let version = match &command.version {
        Some(v) => v.clone(),
        None => {
            aprintln!("{}", p_r("Error: version argument is required"));
            aprintln!("Usage: cargo xtask release 2.1.0");
            return Err(ReleaseError::Aborted);
        }
    };

    // Pre-flight checks
    aprintln!("{}", p_b("Running pre-flight checks..."));
    aprintln!();

    check_gh_cli()?;
    check_main_branch(&global).await?;
    check_clean_working_dir(&global).await?;
    check_ci_status(&global).await?;
    validate_version(&version)?;

    let current = get_current_version().await?;
    if !version_is_bump(&current, &version)? {
        return Err(ReleaseError::VersionNotBumped {
            current,
            new: version,
        });
    }

    aprintln!();
    aprintln!("{} All pre-flight checks passed", p_g("✅"));
    aprintln!();
    aprintln!("  {} → {}", p_y(&current), p_g(&version));
    aprintln!();

    if !confirm("Proceed with release?").await? {
        return Err(ReleaseError::Aborted);
    }

    // Bump version
    aprintln!();
    aprintln!("{} {}", p_b("📦"), p_b("Updating version..."));
    update_version(&version).await?;

    // Create tag and push — retry on workflow failure
    let mut attempts = 0;
    loop {
        attempts += 1;

        aprintln!("{} {}", p_b("🏷️"), p_b("Creating tag and pushing..."));
        create_and_push_tag(&version).await?;

        if command.no_monitor {
            aprintln!("{} Skipping workflow monitoring (--no-monitor)", p_y("⚠️"));
            break;
        }

        aprintln!("{} {}", p_b("👀"), p_b("Monitoring release workflow..."));
        match wait_for_workflow(&version, &global).await {
            Ok(()) => break,
            Err(e) => {
                aprintln!("{} Workflow failed: {e}", p_r("❌"));
                if attempts >= MAX_RETRIES {
                    return Err(ReleaseError::WorkflowFailed(MAX_RETRIES));
                }
                aprintln!(
                    "{} Attempt {attempts}/{MAX_RETRIES} failed. Cleaning up for retry...",
                    p_y("⚠️")
                );
                cleanup_tag(&format!("v{version}"), &global).await?;
                rollback_version().await?;

                if !confirm("Retry release?").await? {
                    return Err(ReleaseError::Aborted);
                }

                // Re-bump version for next attempt
                update_version(&version).await?;
            }
        }
    }

    aprintln!();
    aprintln!(
        "{} {} released successfully!",
        p_g("🎉"),
        p_g(&format!("v{version}"))
    );

    if command.auto_upgrade {
        aprintln!();
        aprintln!("{} {}", p_b("⬆️"), p_b("Running peon upgrade..."));
        run_peon_upgrade().await?;
    } else {
        aprintln!();
        if confirm("Run `peon upgrade` to update local binary?").await? {
            run_peon_upgrade().await?;
        }
    }

    Ok(())
}

fn check_gh_cli() -> Result<()> {
    if !command_exists("gh") {
        return Err(ReleaseError::CommandNotFound {
            command: "gh".into(),
            help: "Install GitHub CLI: https://cli.github.com/".into(),
        });
    }

    let status = std::process::Command::new("gh")
        .args(["auth", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| ReleaseError::GitFailed(format!("gh auth status: {e}")))?;

    if !status.success() {
        return Err(ReleaseError::CommandNotFound {
            command: "gh".into(),
            help: "Not authenticated. Run: gh auth login".into(),
        });
    }

    aprintln!("{} gh CLI authenticated", p_g("✅"));
    Ok(())
}

async fn check_main_branch(global: &crate::Global) -> Result<()> {
    let output = run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let branch = output.trim();

    if branch != "main" {
        if global.is_verbose() {
            aprintln!("{} Current branch: {}", p_r("❌"), branch);
        }
        return Err(ReleaseError::NotOnMainBranch);
    }

    aprintln!("{} On main branch", p_g("✅"));
    Ok(())
}

async fn check_clean_working_dir(global: &crate::Global) -> Result<()> {
    let output = run_git(&["status", "--porcelain"]).await?;

    if !output.trim().is_empty() {
        if global.is_verbose() {
            aprintln!("{} Uncommitted changes:", p_r("❌"));
            aprintln!("{output}");
        }
        return Err(ReleaseError::DirtyWorkingDir);
    }

    aprintln!("{} Working directory clean", p_g("✅"));
    Ok(())
}

async fn check_ci_status(_global: &crate::Global) -> Result<()> {
    let output = tokio::process::Command::new("gh")
        .args([
            "api",
            &format!("repos/{GITHUB_REPO}/commits/main/check-runs"),
            "--jq",
            ".check_runs[] | select(.name == \"Lint & Test\") | .conclusion",
        ])
        .output()
        .await
        .map_err(|e| ReleaseError::GitFailed(format!("gh api: {e}")))?;

    let conclusion = String::from_utf8_lossy(&output.stdout);
    let conclusion = conclusion.trim();

    if conclusion != "success" {
        if conclusion.is_empty() {
            aprintln!(
                "{} No CI status found for main — check runs may still be pending",
                p_y("⚠️")
            );
        } else {
            return Err(ReleaseError::CiNotPassed);
        }
    } else {
        aprintln!("{} CI checks passed on main", p_g("✅"));
    }

    Ok(())
}

fn validate_version(version: &str) -> Result<()> {
    let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").expect("valid regex");
    if !re.is_match(version) {
        return Err(ReleaseError::InvalidVersion(version.into()));
    }
    aprintln!("{} Version format valid: {version}", p_g("✅"));
    Ok(())
}

async fn get_current_version() -> Result<String> {
    let contents = tokio::fs::read_to_string("Cargo.toml")
        .await
        .map_err(|e| ReleaseError::GitFailed(format!("read Cargo.toml: {e}")))?;

    parse_workspace_version(&contents)
        .ok_or_else(|| ReleaseError::GitFailed("could not parse version from Cargo.toml".into()))
}

fn parse_workspace_version(contents: &str) -> Option<String> {
    let re = regex::Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)""#).ok()?;
    // Find the first version in [workspace.package] section
    let mut in_workspace_package = false;
    for line in contents.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package {
            if line.trim().starts_with('[') {
                break;
            }
            if let Some(caps) = re.captures(line) {
                return Some(caps[1].to_string());
            }
        }
    }
    None
}

fn version_is_bump(current: &str, new: &str) -> Result<bool> {
    let parse = |v: &str| -> Result<(u64, u64, u64)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return Err(ReleaseError::InvalidVersion(v.into()));
        }
        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| ReleaseError::InvalidVersion(v.into()))?;
        let minor = parts[1]
            .parse::<u64>()
            .map_err(|_| ReleaseError::InvalidVersion(v.into()))?;
        let patch = parts[2]
            .parse::<u64>()
            .map_err(|_| ReleaseError::InvalidVersion(v.into()))?;
        Ok((major, minor, patch))
    };

    let new_v = parse(new)?;
    let cur_v = parse(current)?;
    Ok(new_v > cur_v)
}

async fn update_version(version: &str) -> Result<()> {
    let contents = tokio::fs::read_to_string("Cargo.toml")
        .await
        .map_err(|e| ReleaseError::GitFailed(format!("read Cargo.toml: {e}")))?;

    let re = regex::Regex::new(r#"(?m)^(\s*version\s*=\s*)"[^"]+""#).expect("valid regex");

    let mut result = String::new();
    let mut in_workspace_package = false;
    let mut replaced = false;

    for line in contents.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
        } else if line.trim().starts_with('[') {
            in_workspace_package = false;
        }

        if in_workspace_package && !replaced && re.is_match(line) {
            let new_line = re.replace(line, format!("${{1}}\"{version}\""));
            result.push_str(&new_line);
            replaced = true;
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    // Preserve original trailing newline behavior
    if !contents.ends_with('\n') {
        result.pop();
    }

    tokio::fs::write("Cargo.toml", &result)
        .await
        .map_err(|e| ReleaseError::GitFailed(format!("write Cargo.toml: {e}")))?;

    // Run cargo check to update Cargo.lock
    let status = tokio::process::Command::new("cargo")
        .args(["check", "--quiet"])
        .status()
        .await?;

    if !status.success() {
        return Err(ReleaseError::GitFailed(
            "cargo check failed after version bump".into(),
        ));
    }

    aprintln!("{} Version updated to {version}", p_g("✅"));
    Ok(())
}

async fn create_and_push_tag(version: &str) -> Result<()> {
    let tag = format!("v{version}");

    // Stage Cargo.toml and Cargo.lock
    run_git(&["add", "Cargo.toml", "Cargo.lock"]).await?;

    // Commit
    let msg = format!("chore: bump version to {version}");
    run_git(&["commit", "-m", &msg]).await?;

    // Create annotated tag
    let tag_msg = format!("Release {version}");
    run_git(&["tag", "-a", &tag, "-m", &tag_msg]).await?;

    // Push commit and tag
    run_git(&["push", "origin", "main"]).await?;
    run_git(&["push", "origin", &tag]).await?;

    aprintln!("{} Pushed {} to origin", p_g("✅"), p_c(&tag));
    Ok(())
}

async fn wait_for_workflow(version: &str, _global: &crate::Global) -> Result<()> {
    let tag = format!("v{version}");
    let start = std::time::Instant::now();

    // Wait a few seconds for the workflow to register
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        if start.elapsed().as_secs() > WORKFLOW_TIMEOUT {
            return Err(ReleaseError::WorkflowTimeout(WORKFLOW_TIMEOUT));
        }

        let output = tokio::process::Command::new("gh")
            .args([
                "run",
                "list",
                "--workflow",
                WORKFLOW_FILE,
                "--branch",
                &tag,
                "--json",
                "status,conclusion",
                "--limit",
                "1",
            ])
            .output()
            .await
            .map_err(|e| ReleaseError::GitFailed(format!("gh run list: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let runs: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).unwrap_or_default();

        if let Some(run) = runs.first() {
            let status = run["status"].as_str().unwrap_or("");
            let conclusion = run["conclusion"].as_str().unwrap_or("");

            match status {
                "completed" => {
                    if conclusion == "success" {
                        aprintln!("{} Release workflow completed successfully", p_g("✅"));
                        return Ok(());
                    } else {
                        return Err(ReleaseError::GitFailed(format!(
                            "workflow concluded with: {conclusion}"
                        )));
                    }
                }
                "in_progress" | "queued" | "waiting" | "requested" | "pending" => {
                    let elapsed = start.elapsed().as_secs();
                    aprintln!("  {} Workflow {status}... ({elapsed}s elapsed)", p_y("⏳"));
                }
                _ => {
                    aprintln!("  {} Unknown workflow status: {status}", p_y("⚠️"));
                }
            }
        } else {
            let elapsed = start.elapsed().as_secs();
            aprintln!(
                "  {} Waiting for workflow to start... ({elapsed}s elapsed)",
                p_y("⏳")
            );
        }

        tokio::time::sleep(std::time::Duration::from_secs(WORKFLOW_CHECK_INTERVAL)).await;
    }
}

async fn cleanup_tag(tag: &str, _global: &crate::Global) -> Result<()> {
    // Delete remote tag (ignore errors if it doesn't exist)
    let _ = tokio::process::Command::new("git")
        .args(["push", "origin", &format!(":{tag}")])
        .output()
        .await;

    // Delete local tag
    let _ = tokio::process::Command::new("git")
        .args(["tag", "-d", tag])
        .output()
        .await;

    aprintln!("{} Cleaned up tag {tag}", p_g("✅"));
    Ok(())
}

async fn rollback_version() -> Result<()> {
    run_git(&["checkout", "HEAD~1", "--", "Cargo.toml", "Cargo.lock"]).await?;
    // Also reset the commit
    let _ = run_git(&["reset", "--soft", "HEAD~1"]).await;
    aprintln!("{} Rolled back version bump", p_g("✅"));
    Ok(())
}

async fn cleanup_command(tag: &str, global: &crate::Global) -> Result<()> {
    let tag = if tag.starts_with('v') {
        tag.to_string()
    } else {
        format!("v{tag}")
    };

    aprintln!("{} Cleaning up tag {}", p_b("🧹"), p_c(&tag));

    cleanup_tag(&tag, global).await?;

    // Check if there's a version bump commit to revert
    let log = run_git(&["log", "-1", "--pretty=%s"]).await?;
    if log.trim().starts_with("chore: bump version to") {
        aprintln!("{} Found version bump commit, reverting...", p_b("↩️"));
        rollback_version().await?;
    }

    aprintln!("{} Cleanup complete", p_g("✅"));
    Ok(())
}

async fn confirm(prompt: &str) -> Result<bool> {
    aprintln!("{} [y/N] ", p_y(prompt));

    let answer = tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        input
    })
    .await
    .map_err(|e| ReleaseError::GitFailed(format!("read stdin: {e}")))?;

    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

async fn run_peon_upgrade() -> Result<()> {
    let status = tokio::process::Command::new("peon")
        .args(["upgrade", "--force"])
        .status()
        .await?;

    if !status.success() {
        aprintln!("{} peon upgrade failed", p_r("❌"));
    }
    Ok(())
}

// Helpers

async fn run_git(args: &[&str]) -> Result<String> {
    let output = tokio::process::Command::new("git")
        .args(args)
        .output()
        .await
        .map_err(|e| ReleaseError::GitFailed(format!("git {}: {e}", args.join(" "))))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::GitFailed(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace_version() {
        let toml = r#"
[workspace]
members = ["crates/*"]

[workspace.package]
version = "2.0.0"
edition = "2021"
"#;
        assert_eq!(parse_workspace_version(toml), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_parse_workspace_version_missing() {
        let toml = r#"
[workspace]
members = ["crates/*"]

[workspace.package]
edition = "2021"
"#;
        assert_eq!(parse_workspace_version(toml), None);
    }

    #[test]
    fn test_version_is_bump() {
        assert!(version_is_bump("2.0.0", "2.1.0").unwrap());
        assert!(version_is_bump("2.0.0", "3.0.0").unwrap());
        assert!(!version_is_bump("2.0.0", "2.0.0").unwrap());
        assert!(!version_is_bump("2.0.0", "1.9.0").unwrap());
    }
}
