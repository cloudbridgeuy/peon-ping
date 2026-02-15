# Sound Packs

## Structure

Each pack is a directory under `packs/` containing:

```
packs/<name>/
  manifest.json       # Pack metadata + sound-to-category mapping
  sounds/             # Audio files (WAV, MP3, OGG)
```

## Manifest Format

```json
{
  "name": "pack_name",
  "display_name": "Human-Readable Name",
  "categories": {
    "greeting": {
      "sounds": [
        { "file": "Hello.wav", "line": "Ready to work?" }
      ]
    }
  }
}
```

## Categories

| Category | Trigger | Required |
|----------|---------|----------|
| `greeting` | SessionStart | No |
| `acknowledge` | (reserved) | No |
| `complete` | Stop | No |
| `error` | (reserved) | No |
| `permission` | Notification(permission_prompt), PermissionRequest | No |
| `resource_limit` | (reserved) | No |
| `annoyed` | Rapid prompts (3+ in 10s) | No |

Not every category is required. Only include what you have sounds for.

## Pack Rotation

Config supports `pack_rotation` array. When non-empty, each Claude Code session randomly picks one pack from the list and keeps it for the entire session (pinned in state via `session_packs`).

```json
{ "pack_rotation": ["peon", "sc_kerrigan", "peasant"] }
```

Empty array (`[]`) falls back to `active_pack`.

## Discovering & Previewing Packs

```bash
peon packs                          # list installed packs
peon sounds peasant                 # show all categories and voice lines
peon play greeting --pack peon      # hear a random greeting
peon pull sc_kerrigan               # download a pack from GitHub
peon pull --all                     # download all available packs
```

See [SOUNDS.md](/SOUNDS.md) for a full catalog of every pack. Regenerate with `cargo xtask docs`.

## Adding a Pack

1. Create `packs/<name>/manifest.json` and `packs/<name>/sounds/`
2. Run `cargo xtask docs` to regenerate SOUNDS.md
3. Bump version (patch for new packs)
4. See `CONTRIBUTING.md` for full guide
