# Architecture

## Two-Crate Workspace

### `peon_core` (pure library)

All logic is deterministic and testable without I/O:

| Module | Purpose |
|--------|---------|
| `routing` | `route_event()` — maps `HookEvent` to `Vec<Action>` |
| `sound` | `pick_sound()` — random selection with no-repeat |
| `agent` | `is_agent_session()` — detects delegate/agent sessions |
| `annoyed` | `check_annoyed()` — rapid prompt easter egg detection |
| `pack` | `resolve_pack()` — pack rotation with session pinning |
| `sounds` | `format_pack_sounds()` — human-readable pack catalog |
| `tab_title` | `build_tab_title()` — formats terminal tab titles |
| `types/` | `HookEvent`, `Action`, `Config`, `State`, `Manifest` |

### `peon` (binary shell)

I/O boundary — reads stdin, calls core, executes platform commands:

| Module | Purpose |
|--------|---------|
| `main` | Entry point, CLI dispatch (clap) |
| `cli` | `Cli` struct and `Commands` enum |
| `hook` | `handle_hook()` — the main event pipeline |
| `state_io` | Load/save config, state, manifests |
| `paths` | Runtime path resolution (`CLAUDE_PEON_DIR`, `PEON_PACKS` overrides) |
| `platform/audio` | `play_sound()` — afplay (mac) / PowerShell (WSL) |
| `platform/notification` | `send_notification()` — AppleScript / WinForms |
| `github` | `list_remote_packs()`, `pull_pack()` — GitHub Contents API |
| `platform/focus` | `terminal_is_focused()` — frontmost app check |

## Event Pipeline

```
stdin JSON
    |
    v
HookEvent (serde, tagged on hook_event_name)
    |
    v
Agent check — if delegate mode, record session + skip
    |
    v
route_event() -> Vec<Action>  (pure, in peon_core)
    |
    v
Annoyed check — for UserPromptSubmit, may add PlaySound("annoyed")
    |
    v
resolve_pack() — pack rotation with session pinning
    |
    v
For each Action:
  PlaySound  -> pick_sound() -> afplay (background)
  SetTabTitle -> ANSI escape to stdout
  Notify     -> osascript notification (if terminal not focused)
  Skip       -> no-op
    |
    v
Save state (if dirty)
```

## Types

**HookEvent** variants (serde tagged on `hook_event_name`):
- `SessionStart` — Claude Code session begins
- `UserPromptSubmit` — user sends a prompt
- `Stop` — Claude finishes a task
- `Notification { notification_type }` — permission_prompt, idle_prompt, etc.
- `PermissionRequest` — IDE tool approval needed

**Action** variants:
- `PlaySound { category }` — play a random sound from category
- `SetTabTitle { title }` — set terminal tab title via ANSI
- `Notify { message, title, color }` — desktop notification
- `Skip` — no-op

## Testing

- **Unit tests** in `peon_core` — test routing, sound picking, agent detection, annoyed logic
- `CLAUDE_PEON_DIR` env var isolates tests from real config
- `PEON_PACKS` env var overrides packs directory (dev: `PEON_PACKS=./packs`)
