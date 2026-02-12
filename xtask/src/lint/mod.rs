use crate::prelude::*;
use error::Result;
use std::io::BufRead;

pub mod error;
mod hooks;

/// Code quality checks and git hooks management
#[derive(Debug, clap::Parser)]
#[command(
    long_about = "Run code quality checks including formatting, compilation, linting, and tests.

This command runs the following checks in order:

1. cargo fmt - Code formatting (auto-fix with --fix)
2. cargo check - Compilation check
3. cargo clippy - Linting with all warnings treated as errors
4. cargo test - Run all tests including doctests
5. File length check - Ensures no .rs file exceeds 1000 lines

When used with --install-hooks, this command also manages git pre-commit hooks that
run these same checks automatically before each commit."
)]
pub struct LintCommand {
    /// Auto-fix issues when possible (applies to fmt and clippy)
    #[arg(long)]
    pub fix: bool,

    /// Check all files instead of just staged files (for hooks)
    #[arg(long)]
    pub force: bool,

    /// Only check staged files (used by git hooks)
    #[arg(long, hide = true)]
    pub staged_only: bool,

    /// Install git pre-commit hooks
    #[arg(long, conflicts_with_all = &["uninstall_hooks", "hooks_status", "test_hooks"])]
    pub install_hooks: bool,

    /// Uninstall git pre-commit hooks
    #[arg(long, conflicts_with_all = &["install_hooks", "hooks_status", "test_hooks"])]
    pub uninstall_hooks: bool,

    /// Show git hooks installation status
    #[arg(long, conflicts_with_all = &["install_hooks", "uninstall_hooks", "test_hooks"])]
    pub hooks_status: bool,

    /// Test installed git hooks
    #[arg(long, conflicts_with_all = &["install_hooks", "uninstall_hooks", "hooks_status"])]
    pub test_hooks: bool,
}

pub async fn run(command: LintCommand, global: crate::Global) -> Result<()> {
    if command.install_hooks {
        return hooks::install_hooks(&global).await;
    }

    if command.uninstall_hooks {
        return hooks::uninstall_hooks(&global).await;
    }

    if command.hooks_status {
        return hooks::show_status().await;
    }

    if command.test_hooks {
        return hooks::test_hooks().await;
    }

    run_lint_checks(&command, &global).await
}

async fn run_lint_checks(command: &LintCommand, global: &crate::Global) -> Result<()> {
    use error::require_command;

    require_command("cargo", "Required for Rust development: https://rustup.rs/")?;

    if !global.is_silent() {
        aprintln!("{}", p_b("Running code quality checks..."));
        aprintln!();
    }

    let mut all_passed = true;

    if !run_cargo_fmt(command, global).await? {
        all_passed = false;
    }

    if !run_cargo_check(global).await? {
        all_passed = false;
    }

    if !run_cargo_clippy(global).await? {
        all_passed = false;
    }

    if !run_cargo_test(global).await? {
        all_passed = false;
    }

    if !run_file_length_check(global).await? {
        all_passed = false;
    }

    aprintln!();
    if all_passed {
        aprintln!("{} {}", p_g("✅"), p_g("All checks passed!"));
        Ok(())
    } else {
        aprintln!("{} {}", p_r("❌"), p_r("Some checks failed"));
        aprintln!();
        if !global.is_silent() {
            aprintln!("{}", p_b("Quick fixes:"));
            aprintln!("  • {} - Format code", p_c("cargo xtask lint --fix"));
            aprintln!("  • {} - Auto-fix clippy issues", p_c("cargo clippy --fix"));
            aprintln!("  • {} - Check compilation", p_c("cargo check"));
        }
        Err(error::LintError::ChecksFailed)?
    }
}

async fn run_cargo_fmt(command: &LintCommand, global: &crate::Global) -> Result<bool> {
    if !global.is_silent() {
        aprintln!("{} {}", p_b("🔧"), p_b("Running cargo fmt..."));
    }

    let check_output = tokio::process::Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .await?;

    if check_output.status.success() {
        if !global.is_silent() {
            aprintln!("{} {}", p_g("✅"), "Code formatting is correct");
        }
        return Ok(true);
    }

    if command.fix || command.staged_only {
        if global.is_verbose() {
            aprintln!(
                "{} {}",
                p_y("⚠️"),
                "Code formatting issues found. Auto-fixing..."
            );
        }

        let fmt_status = tokio::process::Command::new("cargo")
            .arg("fmt")
            .status()
            .await?;

        if fmt_status.success() {
            if command.staged_only {
                restage_rust_files(global).await?;
                if !global.is_silent() {
                    aprintln!("{} {}", p_g("✅"), "Code formatted and re-staged");
                }
            } else if !global.is_silent() {
                aprintln!("{} {}", p_g("✅"), "Code formatted");
            }
            Ok(true)
        } else {
            aprintln!("{} {}", p_r("❌"), "cargo fmt failed");
            Ok(false)
        }
    } else {
        aprintln!(
            "{} {}",
            p_r("❌"),
            "Code formatting check failed. Run with --fix to auto-format"
        );
        Ok(false)
    }
}

async fn run_cargo_check(global: &crate::Global) -> Result<bool> {
    if !global.is_silent() {
        aprintln!("{} {}", p_b("🔧"), p_b("Running cargo check..."));
    }

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["check", "--all-targets"]);

    if !global.is_verbose() {
        cmd.arg("--quiet");
    }

    let status = cmd.status().await?;

    if status.success() {
        if !global.is_silent() {
            aprintln!("{} {}", p_g("✅"), "Cargo check passed");
        }
        Ok(true)
    } else {
        aprintln!("{} {}", p_r("❌"), "Cargo check failed");
        aprintln!("{}", p_r("Please fix compilation errors before proceeding"));
        Ok(false)
    }
}

async fn run_cargo_clippy(global: &crate::Global) -> Result<bool> {
    if !global.is_silent() {
        aprintln!("{} {}", p_b("🔧"), p_b("Running cargo clippy..."));
    }

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["clippy", "--all-targets"]);

    if !global.is_verbose() {
        cmd.arg("--quiet");
    }

    cmd.args(["--", "-D", "warnings"]);

    let status = cmd.status().await?;

    if status.success() {
        if !global.is_silent() {
            aprintln!("{} {}", p_g("✅"), "Clippy checks passed");
        }
        Ok(true)
    } else {
        aprintln!("{} {}", p_r("❌"), "Clippy checks failed");
        aprintln!("{}", p_r("Please fix clippy warnings before proceeding"));
        if !global.is_silent() {
            aprintln!(
                "{} Run {} to auto-fix some issues",
                p_b("Tip:"),
                p_c("cargo clippy --fix")
            );
        }
        Ok(false)
    }
}

async fn run_cargo_test(global: &crate::Global) -> Result<bool> {
    if !global.is_silent() {
        aprintln!("{} {}", p_b("🔧"), p_b("Running cargo test..."));
    }

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["test", "--all-targets"]);

    if !global.is_verbose() {
        cmd.arg("--quiet");
    }

    let status = cmd.status().await?;

    if status.success() {
        if !global.is_silent() {
            aprintln!("{} {}", p_g("✅"), "All tests passed");
        }
        Ok(true)
    } else {
        aprintln!("{} {}", p_r("❌"), "Tests failed");
        aprintln!("{}", p_r("Please fix failing tests before proceeding"));
        Ok(false)
    }
}

async fn restage_rust_files(global: &crate::Global) -> Result<()> {
    let output = tokio::process::Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(());
    }

    let files = String::from_utf8_lossy(&output.stdout);
    let rust_files: Vec<&str> = files.lines().filter(|line| line.ends_with(".rs")).collect();

    if !rust_files.is_empty() {
        let mut cmd = tokio::process::Command::new("git");
        cmd.arg("add");
        cmd.args(&rust_files);
        cmd.status().await?;

        if global.is_verbose() {
            aprintln!("{} Re-staged {} Rust files", p_b("Info:"), rust_files.len());
        }
    }

    Ok(())
}

const MAX_FILE_LINES: usize = 1000;

type FileLineCount = (String, usize);
type FileError = (String, std::io::Error);

fn find_file_length_violations(
    file_paths: &[&str],
    max_lines: usize,
) -> (Vec<FileLineCount>, Vec<FileError>) {
    let mut violations = Vec::new();
    let mut errors = Vec::new();

    for &path in file_paths {
        match std::fs::File::open(path) {
            Ok(file) => {
                let line_count = std::io::BufReader::new(file).lines().count();
                if line_count > max_lines {
                    violations.push((path.to_string(), line_count));
                }
            }
            Err(e) => {
                errors.push((path.to_string(), e));
            }
        }
    }

    (violations, errors)
}

async fn run_file_length_check(global: &crate::Global) -> Result<bool> {
    if !global.is_silent() {
        aprintln!("{} {}", p_b("🔧"), p_b("Checking file lengths..."));
    }

    let output = tokio::process::Command::new("git")
        .args(["ls-files", "*.rs", "**/*.rs"])
        .output()
        .await?;

    if !output.status.success() {
        aprintln!("{} Failed to list Rust files", p_r("❌"));
        return Ok(false);
    }

    let files_output = String::from_utf8_lossy(&output.stdout);
    let file_paths: Vec<&str> = files_output.lines().filter(|l| !l.is_empty()).collect();

    let (violations, errors) = find_file_length_violations(&file_paths, MAX_FILE_LINES);

    if global.is_verbose() {
        for (path, err) in &errors {
            aprintln!("{} Skipping {}: {}", p_y("⚠️"), path, err);
        }
    }

    if violations.is_empty() {
        if !global.is_silent() {
            aprintln!(
                "{} {}",
                p_g("✅"),
                format!("All files under {} lines", MAX_FILE_LINES)
            );
        }
        Ok(true)
    } else {
        aprintln!(
            "{} {} file(s) exceed {} lines:",
            p_r("❌"),
            violations.len(),
            MAX_FILE_LINES
        );
        for (path, count) in &violations {
            aprintln!("  {} ({} lines)", p_r(path), count);
        }
        if !global.is_silent() {
            aprintln!();
            aprintln!(
                "{} Consider splitting large files into modules",
                p_b("Tip:")
            );
        }
        Ok(false)
    }
}
