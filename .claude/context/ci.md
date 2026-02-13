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

1. Bump version in both `crates/core/Cargo.toml` and `crates/peon/Cargo.toml`
2. Commit and push to `main`
3. Tag: `git tag v<version> && git push --tags`
4. Release workflow builds binaries and creates GitHub release
5. Users install via `curl | bash` which downloads from the latest release
