# peon-ping

Warcraft III Peon voice lines for Claude Code hooks. A compiled Rust binary that plays sounds, sets tab titles, and sends notifications when Claude Code needs attention.

## Architecture

**Functional Core / Imperative Shell** — two-crate workspace:

- `crates/core` (`peon_core`) — Pure library. Event routing, sound selection, annoyed detection, agent detection, tab title formatting. No I/O, fully deterministic.
- `crates/peon` (`peon`) — Binary shell. CLI (clap), stdin hook handler, platform integration (audio, notifications, focus detection).

See [.claude/context/architecture.md](.claude/context/architecture.md) for the full event pipeline and crate internals.

## Key Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run all tests (unit + integration)
cargo xtask lint               # All quality checks (fmt, check, clippy, test, file-length)
cargo xtask lint --fix         # Auto-fix formatting
```

## Conventions

- `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` — no `.unwrap()` outside tests
- Pure functions in `peon_core`, I/O only in `peon`
- YAGNI — solve the current problem, don't anticipate future ones
- Sound packs are self-contained: `packs/<name>/manifest.json` + `sounds/`

## Key Paths (Runtime)

| Path | Purpose |
|------|---------|
| `~/.claude/hooks/peon-ping/config.json` | User configuration |
| `~/.claude/hooks/peon-ping/.state.json` | Runtime state (last played, agent sessions, timestamps) |
| `~/.claude/hooks/peon-ping/.paused` | Presence = sounds muted |
| `~/.claude/hooks/peon-ping/packs/` | Installed sound packs |
| `CLAUDE_PEON_DIR` env var | Overrides base dir (used in tests) |

## CLI

`peon` with no args = hook mode (reads JSON from stdin). Subcommands:

```
peon pause | resume | toggle | status | packs | pack [name]
```

## Project Structure

```
Cargo.toml              # Workspace root
crates/core/            # peon_core — pure library
crates/peon/            # peon — binary
packs/                  # Sound packs (manifest.json + sounds/)
scripts/install.sh      # Installer (curl|bash or local clone)
scripts/uninstall.sh    # Uninstaller
.github/                # CI/CD workflows, dependabot
completions.bash        # Tab completion for bash/zsh
config.json             # Default configuration
skills/                 # Claude Code slash command (/peon-ping-toggle)
xtask/                  # Dev tooling (lint, hooks)
```

## Context Files

- [Architecture & Event Pipeline](.claude/context/architecture.md)
- [Sound Packs](.claude/context/sound-packs.md)
- [CI/CD](.claude/context/ci.md)
