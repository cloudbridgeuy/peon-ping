#!/bin/bash
# peon-ping installer
# Works both via `curl | bash` (downloads from GitHub Releases) and local clone
# Re-running updates core files; sounds are version-controlled in the repo
set -euo pipefail

INSTALL_DIR="$HOME/.claude/hooks/peon-ping"
SETTINGS="$HOME/.claude/settings.json"
REPO="stuctf/peon-ping"
REPO_BASE="https://raw.githubusercontent.com/$REPO/main"
BIN_NAME="peon"

# --- macOS guard ---
if [ "$(uname -s)" != "Darwin" ]; then
  echo "Error: peon-ping requires macOS"
  exit 1
fi

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    *) echo "unknown" ;;
  esac
}
ARCH=$(detect_arch)

# --- Detect update vs fresh install ---
UPDATING=false
if command -v peon &>/dev/null; then
  UPDATING=true
fi

if [ "$UPDATING" = true ]; then
  echo "=== peon-ping updater ==="
  echo ""
  echo "Existing install found. Updating..."
else
  echo "=== peon-ping installer ==="
  echo ""
fi

# --- Prerequisites ---
if ! command -v afplay &>/dev/null; then
  echo "Error: afplay is required (should be built into macOS)"
  exit 1
fi

if [ ! -d "$HOME/.claude" ]; then
  echo "Error: ~/.claude/ not found. Is Claude Code installed?"
  exit 1
fi

# --- Detect if running from local clone ---
SCRIPT_DIR=""
if [ -n "${BASH_SOURCE[0]:-}" ] && [ "${BASH_SOURCE[0]}" != "bash" ]; then
  CANDIDATE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)"
  if [ -f "$CANDIDATE/Cargo.toml" ]; then
    SCRIPT_DIR="$CANDIDATE"
  fi
fi

# --- Install binary ---
BIN_DIR="/usr/local/bin"
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/target/release/$BIN_NAME" ]; then
  # Local clone with pre-built binary
  echo "Installing binary from local build..."
  cp "$SCRIPT_DIR/target/release/$BIN_NAME" "$BIN_DIR/$BIN_NAME"
  chmod +x "$BIN_DIR/$BIN_NAME"
elif [ -z "$SCRIPT_DIR" ]; then
  # Download pre-built binary from GitHub Releases
  echo "Downloading peon binary..."
  TARGET=""
  case "$ARCH" in
    aarch64) TARGET="aarch64-apple-darwin" ;;
    x86_64)  TARGET="x86_64-apple-darwin" ;;
    *)
      echo "Error: unsupported architecture: $ARCH"
      exit 1
      ;;
  esac

  # Get latest release URL
  RELEASE_URL="https://github.com/$REPO/releases/latest/download/peon-$TARGET"
  TMP_BIN=$(mktemp)
  if curl -fsSL "$RELEASE_URL" -o "$TMP_BIN"; then
    chmod +x "$TMP_BIN"
    mv "$TMP_BIN" "$BIN_DIR/$BIN_NAME"
    echo "Binary installed to $BIN_DIR/$BIN_NAME"
  else
    rm -f "$TMP_BIN"
    echo "Error: failed to download binary from $RELEASE_URL"
    echo "You can build from source: cargo build --release"
    exit 1
  fi
else
  echo "No pre-built binary found. Build first: cargo build --release"
  exit 1
fi

# --- Install sound packs ---
mkdir -p "$INSTALL_DIR/packs"
if [ -n "$SCRIPT_DIR" ]; then
  # Local clone — copy files directly (including sounds)
  cp -r "$SCRIPT_DIR/packs/"* "$INSTALL_DIR/packs/"
  cp "$SCRIPT_DIR/completions.bash" "$INSTALL_DIR/"
  [ -f "$SCRIPT_DIR/scripts/uninstall.sh" ] && cp "$SCRIPT_DIR/scripts/uninstall.sh" "$INSTALL_DIR/"
  if [ "$UPDATING" = false ]; then
    cp "$SCRIPT_DIR/config.json" "$INSTALL_DIR/"
  fi
else
  # curl|bash — use peon to pull packs from GitHub
  echo "Downloading sound packs..."
  "$BIN_DIR/$BIN_NAME" pull --all --packs-dir "$INSTALL_DIR/packs"
  curl -fsSL "$REPO_BASE/completions.bash" -o "$INSTALL_DIR/completions.bash"
  curl -fsSL "$REPO_BASE/scripts/uninstall.sh" -o "$INSTALL_DIR/uninstall.sh"
  if [ "$UPDATING" = false ]; then
    curl -fsSL "$REPO_BASE/config.json" -o "$INSTALL_DIR/config.json"
  fi
fi

# --- Install skill (slash command) ---
SKILL_DIR="$HOME/.claude/skills/peon-ping-toggle"
mkdir -p "$SKILL_DIR"
if [ -n "$SCRIPT_DIR" ] && [ -d "$SCRIPT_DIR/skills/peon-ping-toggle" ]; then
  cp "$SCRIPT_DIR/skills/peon-ping-toggle/SKILL.md" "$SKILL_DIR/"
elif [ -z "$SCRIPT_DIR" ]; then
  curl -fsSL "$REPO_BASE/skills/peon-ping-toggle/SKILL.md" -o "$SKILL_DIR/SKILL.md"
fi

# --- Add tab completion ---
COMPLETION_LINE='[ -f ~/.claude/hooks/peon-ping/completions.bash ] && source ~/.claude/hooks/peon-ping/completions.bash'
for rcfile in "$HOME/.zshrc" "$HOME/.bashrc"; do
  if [ -f "$rcfile" ] && ! grep -qF 'peon-ping/completions.bash' "$rcfile"; then
    echo "" >> "$rcfile"
    echo "$COMPLETION_LINE" >> "$rcfile"
    echo "Added tab completion to $(basename "$rcfile")"
  fi
done

# --- Verify sounds are installed ---
echo ""
for pack_dir in "$INSTALL_DIR/packs"/*/; do
  [ -d "$pack_dir" ] || continue
  pack=$(basename "$pack_dir")
  sound_count=$({ ls "$pack_dir/sounds/"*.wav "$pack_dir/sounds/"*.mp3 "$pack_dir/sounds/"*.ogg 2>/dev/null || true; } | wc -l | tr -d ' ')
  if [ "$sound_count" -eq 0 ]; then
    echo "[$pack] Warning: No sound files found!"
  else
    echo "[$pack] $sound_count sound files installed."
  fi
done

# --- Update settings.json ---
echo ""
echo "Updating Claude Code hooks in settings.json..."

# Use the peon binary as the hook command
HOOK_CMD="$BIN_DIR/$BIN_NAME"

# Build settings update without python3 dependency
if command -v python3 &>/dev/null; then
  python3 -c "
import json, os, sys

settings_path = os.path.expanduser('$SETTINGS')
hook_cmd = '$HOOK_CMD'

if os.path.exists(settings_path):
    with open(settings_path) as f:
        settings = json.load(f)
else:
    settings = {}

hooks = settings.setdefault('hooks', {})

peon_hook = {
    'type': 'command',
    'command': hook_cmd,
    'timeout': 10
}

peon_entry = {
    'matcher': '',
    'hooks': [peon_hook]
}

events = ['SessionStart', 'UserPromptSubmit', 'Stop', 'Notification', 'PermissionRequest']

for event in events:
    event_hooks = hooks.get(event, [])
    event_hooks = [
        h for h in event_hooks
        if not any(
            'peon' == os.path.basename(hk.get('command', ''))
            for hk in h.get('hooks', [])
        )
    ]
    event_hooks.append(peon_entry)
    hooks[event] = event_hooks

settings['hooks'] = hooks

with open(settings_path, 'w') as f:
    json.dump(settings, f, indent=2)
    f.write('\n')

print('Hooks registered for: ' + ', '.join(events))
"
elif command -v jq &>/dev/null; then
  # jq fallback for settings update
  EVENTS="SessionStart UserPromptSubmit Stop Notification PermissionRequest"
  TMP_SETTINGS=$(mktemp)
  if [ -f "$SETTINGS" ]; then
    cp "$SETTINGS" "$TMP_SETTINGS"
  else
    echo '{}' > "$TMP_SETTINGS"
  fi
  for event in $EVENTS; do
    UPDATED=$(jq --arg event "$event" --arg cmd "$HOOK_CMD" '
      .hooks //= {} |
      .hooks[$event] = [
        (.hooks[$event] // [] | map(
          select(.hooks | all(.command | endswith("/peon") | not))
        )[]] +
        [{"matcher": "", "hooks": [{"type": "command", "command": $cmd, "timeout": 10}]}]
      ]
    ' "$TMP_SETTINGS")
    echo "$UPDATED" > "$TMP_SETTINGS"
  done
  cp "$TMP_SETTINGS" "$SETTINGS"
  rm -f "$TMP_SETTINGS"
  echo "Hooks registered for: $EVENTS"
else
  echo "Warning: neither python3 nor jq found. Please manually update $SETTINGS"
fi

# --- Initialize state (fresh install only) ---
if [ "$UPDATING" = false ]; then
  echo '{}' > "$INSTALL_DIR/.state.json"
fi

# --- Test sound ---
echo ""
echo "Testing sound..."
# Read active pack from config without python3
ACTIVE_PACK="peon"
if [ -f "$INSTALL_DIR/config.json" ]; then
  if command -v jq &>/dev/null; then
    ACTIVE_PACK=$(jq -r '.active_pack // "peon"' "$INSTALL_DIR/config.json" 2>/dev/null || echo "peon")
  else
    ACTIVE_PACK=$(grep -o '"active_pack"[[:space:]]*:[[:space:]]*"[^"]*"' "$INSTALL_DIR/config.json" 2>/dev/null | sed 's/.*"active_pack"[[:space:]]*:[[:space:]]*"//;s/"$//' || echo "peon")
  fi
fi
PACK_DIR="$INSTALL_DIR/packs/$ACTIVE_PACK"
TEST_SOUND=$({ ls "$PACK_DIR/sounds/"*.wav "$PACK_DIR/sounds/"*.mp3 "$PACK_DIR/sounds/"*.ogg 2>/dev/null || true; } | head -1)
if [ -n "$TEST_SOUND" ]; then
  afplay -v 0.3 "$TEST_SOUND"
  echo "Sound working!"
else
  echo "Warning: No sound files found. Sounds may not play."
fi

echo ""
if [ "$UPDATING" = true ]; then
  echo "=== Update complete! ==="
  echo ""
  echo "Updated: peon binary, sound packs"
  echo "Preserved: config.json, state"
else
  echo "=== Installation complete! ==="
  echo ""
  echo "Config: $INSTALL_DIR/config.json"
  echo "  - Adjust volume, toggle categories, switch packs"
  echo ""
  echo "Uninstall: bash $INSTALL_DIR/uninstall.sh"
fi
echo ""
echo "Quick controls:"
echo "  /peon-ping-toggle  — toggle sounds in Claude Code"
echo "  peon toggle        — toggle sounds from any terminal"
echo "  peon status        — check if sounds are paused"
echo "  peon sounds        — show voice lines for current pack"
echo "  peon play          — preview a random sound"
echo ""
echo "Ready to work!"
