# CI/CD

## Workflows

### `ci.yml` — Lint & Test

Runs on every push to `main` and on PRs targeting `main`.

Single job on `ubuntu-latest`: `cargo xtask lint` (fmt, check, clippy, test, file-length).

### `release.yml` — Build & Release

Triggered by `v*` tags (creates GitHub release) and `workflow_dispatch` (build-only).

**Build matrix** — 2 macOS targets on `macos-latest`:

| Target | Asset name |
|--------|-----------|
| `aarch64-apple-darwin` | `peon-aarch64-apple-darwin` |
| `x86_64-apple-darwin` | `peon-x86_64-apple-darwin` |

Steps: checkout → rust toolchain + target → cargo build --release → strip → upload artifact.

**Release job** (tags only): downloads artifacts, generates changelog from git log, creates GitHub release via `softprops/action-gh-release@v2`.

Asset names must match `scripts/install.sh` line ~87: `peon-$TARGET` where TARGET is `{arch}-apple-darwin`.

### `dependabot.yml` (workflow) — Auto-Merge

Runs on `pull_request_target` from `dependabot[bot]`. Checks out the PR, runs `cargo xtask lint`, then auto-merges with `gh pr merge --auto --merge`.

## Dependabot

`.github/dependabot.yml` configures weekly dependency updates:

- **Cargo** — Mondays 09:00, limit 10 PRs, labels: `dependencies`, `rust`
- **GitHub Actions** — Mondays 09:00, limit 5 PRs, labels: `dependencies`, `github-actions`

Both assign and request review from `guzmonne`.

## Release Process

Automated via `cargo xtask release`:

```bash
cargo xtask release 2.1.0              # Full release
cargo xtask release 2.1.0 --no-monitor # Skip workflow polling
cargo xtask release 2.1.0 --auto-upgrade # Auto-run peon upgrade after
cargo xtask release --cleanup v2.1.0   # Clean up a failed release
```

### What it does

1. **Pre-flight checks**: gh CLI auth, on `main`, clean working dir, CI passed, valid semver, version is a bump
2. **Version bump**: Updates `version` in root `Cargo.toml` `[workspace.package]` (both crates inherit via `version.workspace = true`)
3. **Git operations**: `git add Cargo.toml Cargo.lock` → commit (`chore: bump version to X`) → annotated tag (`vX`) → push
4. **Workflow monitoring**: Polls `gh run list` every 30s, max 30min timeout
5. **Retry**: Up to 3 attempts — cleans up tag + rolls back version bump between retries
6. **Post-release**: Optionally runs `peon upgrade` to update the local binary

### Self-update

Users can self-update via `peon upgrade` (or `peon upgrade --force`). Downloads the matching binary from the latest GitHub release and replaces the current executable with rollback on failure. Uses `ureq` (blocking HTTP) against the GitHub releases API.

**GitHub repo**: `cloudbridgeuy/peon-ping`
**API endpoint**: `https://api.github.com/repos/cloudbridgeuy/peon-ping/releases/latest`
**Asset pattern**: `peon-{arch}-apple-darwin` (must match `release.yml` matrix)
