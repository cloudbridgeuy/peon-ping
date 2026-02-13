use crate::upgrade;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "peon",
    about = "Warcraft III Peon voice lines for Claude Code hooks"
)]
pub struct Cli {
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
    /// Self-update peon from GitHub releases
    Upgrade(upgrade::App),
}
