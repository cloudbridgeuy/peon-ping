#!/bin/bash
# peon-ping tab completion for bash and zsh

_peon_completions() {
  local cur prev opts packs_dir
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"

  # Top-level subcommands
  opts="pause resume toggle status packs pack sounds play pull upgrade help"

  # Subcommand-specific completions
  case "$prev" in
    pack|sounds)
      # Complete pack names by scanning manifest files
      packs_dir="${PEON_PACKS:-${CLAUDE_PEON_DIR:-$HOME/.claude/hooks/peon-ping}/packs}"
      if [ -d "$packs_dir" ]; then
        local names
        names=$(find "$packs_dir" -maxdepth 2 -name manifest.json -exec dirname {} \; 2>/dev/null | xargs -I{} basename {} | sort)
        COMPREPLY=( $(compgen -W "$names" -- "$cur") )
      fi
      return 0
      ;;
    play)
      # Complete category names
      COMPREPLY=( $(compgen -W "greeting acknowledge complete error permission resource_limit annoyed" -- "$cur") )
      return 0
      ;;
    pull)
      # Complete with --all flag or pack names from GitHub (fall back to local)
      if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "--all" -- "$cur") )
      else
        packs_dir="${PEON_PACKS:-${CLAUDE_PEON_DIR:-$HOME/.claude/hooks/peon-ping}/packs}"
        if [ -d "$packs_dir" ]; then
          local names
          names=$(find "$packs_dir" -maxdepth 2 -name manifest.json -exec dirname {} \; 2>/dev/null | xargs -I{} basename {} | sort)
          COMPREPLY=( $(compgen -W "$names" -- "$cur") )
        fi
      fi
      return 0
      ;;
    --pack)
      # Complete pack names for --pack flag (used by play)
      packs_dir="${PEON_PACKS:-${CLAUDE_PEON_DIR:-$HOME/.claude/hooks/peon-ping}/packs}"
      if [ -d "$packs_dir" ]; then
        local names
        names=$(find "$packs_dir" -maxdepth 2 -name manifest.json -exec dirname {} \; 2>/dev/null | xargs -I{} basename {} | sort)
        COMPREPLY=( $(compgen -W "$names" -- "$cur") )
      fi
      return 0
      ;;
  esac

  # Flag completions
  if [[ "$cur" == -* ]]; then
    COMPREPLY=( $(compgen -W "--packs-dir --help" -- "$cur") )
    return 0
  fi

  COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
  return 0
}

complete -F _peon_completions peon

# zsh compatibility: if running under zsh, enable bashcompinit
if [ -n "$ZSH_VERSION" ]; then
  autoload -Uz bashcompinit 2>/dev/null && bashcompinit
  complete -F _peon_completions peon
fi
