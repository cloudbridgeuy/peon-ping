use crate::upgrade;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "peon",
    about = "Warcraft III Peon voice lines for Claude Code hooks"
)]
pub struct Cli {
    /// Override packs directory (also: PEON_PACKS env var)
    #[arg(long, global = true)]
    pub packs_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Mute sounds
    Pause,
    /// Unmute sounds
    Resume,
    /// Toggle mute on/off
    Toggle,
    /// Check if paused or active
    Status,
    /// List available sound packs
    Packs,
    /// Switch to a specific pack (or cycle if no name given)
    Pack {
        /// Pack name to switch to. Omit to cycle to next pack.
        name: Option<String>,
    },
    /// Show categories and voice lines for a pack
    Sounds {
        /// Pack name to show. Omit for active pack.
        name: Option<String>,
    },
    /// Play a random sound from a pack
    Play {
        /// Sound category (e.g., greeting, complete, annoyed). Omit for random.
        category: Option<String>,
        /// Pack to play from. Omit for active pack.
        #[arg(long)]
        pack: Option<String>,
    },
    /// Self-update peon from GitHub releases
    Upgrade(upgrade::App),
}
