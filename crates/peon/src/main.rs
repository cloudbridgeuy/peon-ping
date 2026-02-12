mod cli;
mod hook;
mod paths;
mod platform;
mod state_io;

use clap::Parser;
use std::process::ExitCode;

use cli::{Cli, Commands};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Some(command) => run_command(command),
        None => {
            // No subcommand — act as hook handler (read from stdin)
            hook::handle_hook()?;
            Ok(())
        }
    }
}

fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Pause => {
            let path = paths::paused_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, "")?;
            println!("peon-ping: sounds paused");
        }
        Commands::Resume => {
            let path = paths::paused_path();
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("peon-ping: sounds resumed");
        }
        Commands::Toggle => {
            let path = paths::paused_path();
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("peon-ping: sounds resumed");
            } else {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, "")?;
                println!("peon-ping: sounds paused");
            }
        }
        Commands::Status => {
            let paused = state_io::is_paused(&paths::paused_path());
            let config = state_io::load_config(&paths::config_path());
            if paused {
                println!("peon-ping: paused");
            } else {
                println!("peon-ping: active");
            }
            println!("pack: {} ", config.active_pack);
        }
        Commands::Packs => {
            let config = state_io::load_config(&paths::config_path());
            let packs = state_io::list_packs(&paths::packs_dir());
            if packs.is_empty() {
                println!("No packs found in {}", paths::packs_dir().display());
            } else {
                for (name, manifest) in &packs {
                    let display = if manifest.display_name.is_empty() {
                        name.as_str()
                    } else {
                        &manifest.display_name
                    };
                    let marker = if *name == config.active_pack {
                        " *"
                    } else {
                        ""
                    };
                    println!("  {name:24} {display}{marker}");
                }
            }
        }
        Commands::Pack { name } => {
            let config_path = paths::config_path();
            let mut config_map = state_io::load_config_map(&config_path);
            let packs = state_io::list_packs(&paths::packs_dir());
            let pack_names: Vec<&str> = packs.iter().map(|(n, _)| n.as_str()).collect();

            let new_pack = match name {
                Some(ref requested) => {
                    if !pack_names.contains(&requested.as_str()) {
                        eprintln!("Error: pack \"{}\" not found.", requested);
                        eprintln!("Available packs: {}", pack_names.join(", "));
                        return Err("pack not found".into());
                    }
                    requested.clone()
                }
                None => {
                    // Cycle to next pack
                    if pack_names.is_empty() {
                        return Err("no packs found".into());
                    }
                    let current = config_map
                        .get("active_pack")
                        .and_then(|v| v.as_str())
                        .unwrap_or("peon");
                    let idx = pack_names.iter().position(|&n| n == current).unwrap_or(0);
                    let next_idx = (idx + 1) % pack_names.len();
                    pack_names[next_idx].to_string()
                }
            };

            config_map.insert(
                "active_pack".into(),
                serde_json::Value::String(new_pack.clone()),
            );
            state_io::save_config_map(&config_path, &config_map)?;

            let display = packs
                .iter()
                .find(|(n, _)| n == &new_pack)
                .map(|(_, m)| {
                    if m.display_name.is_empty() {
                        m.name.as_str()
                    } else {
                        m.display_name.as_str()
                    }
                })
                .unwrap_or(&new_pack);
            println!("peon-ping: switched to {new_pack} ({display})");
        }
    }
    Ok(())
}
