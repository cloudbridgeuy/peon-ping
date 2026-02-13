mod cli;
mod github;
mod hook;
mod paths;
mod platform;
mod state_io;
mod upgrade;

use clap::Parser;
use rand::Rng;
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
        Some(command) => run_command(command, cli.packs_dir),
        None => {
            hook::handle_hook()?;
            Ok(())
        }
    }
}

fn run_command(
    command: Commands,
    packs_dir_override: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
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
            let packs = state_io::list_packs(&paths::packs_dir(packs_dir_override.as_deref()));
            if packs.is_empty() {
                println!(
                    "No packs found in {}",
                    paths::packs_dir(packs_dir_override.as_deref()).display()
                );
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
        Commands::Upgrade(app) => {
            upgrade::run(app)?;
        }
        Commands::Pack { name } => {
            let config_path = paths::config_path();
            let mut config_map = state_io::load_config_map(&config_path);
            let packs = state_io::list_packs(&paths::packs_dir(packs_dir_override.as_deref()));
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
        Commands::Sounds { name } => {
            let packs_dir = paths::packs_dir(packs_dir_override.as_deref());
            let config = state_io::load_config(&paths::config_path());

            let pack_name = name.unwrap_or(config.active_pack);
            let pack_path = packs_dir.join(&pack_name);

            let manifest = state_io::load_manifest(&pack_path).map_err(|_| {
                let available = state_io::list_packs(&packs_dir);
                let names: Vec<&str> = available.iter().map(|(n, _)| n.as_str()).collect();
                format!(
                    "pack \"{}\" not found. Available: {}",
                    pack_name,
                    if names.is_empty() {
                        "(none)".to_string()
                    } else {
                        names.join(", ")
                    }
                )
            })?;

            print!("{}", peon_core::format_pack_sounds(&manifest));
        }
        Commands::Play { category, pack } => {
            let packs_dir = paths::packs_dir(packs_dir_override.as_deref());
            let config = state_io::load_config(&paths::config_path());

            let pack_name = pack.unwrap_or(config.active_pack);
            let pack_path = packs_dir.join(&pack_name);

            let manifest = state_io::load_manifest(&pack_path).map_err(|_| {
                let available = state_io::list_packs(&packs_dir);
                let names: Vec<&str> = available.iter().map(|(n, _)| n.as_str()).collect();
                format!(
                    "pack \"{}\" not found. Available: {}",
                    pack_name,
                    if names.is_empty() {
                        "(none)".to_string()
                    } else {
                        names.join(", ")
                    }
                )
            })?;

            if manifest.categories.is_empty() {
                return Err(format!("pack \"{}\" has no categories", pack_name).into());
            }

            let mut rng = rand::thread_rng();

            let cat = match category {
                Some(ref cat_name) => {
                    manifest.categories.get(cat_name.as_str()).ok_or_else(|| {
                        let available: Vec<&str> =
                            manifest.categories.keys().map(|k| k.as_str()).collect();
                        format!(
                            "category \"{}\" not found in pack \"{}\". Available: {}",
                            cat_name,
                            pack_name,
                            available.join(", ")
                        )
                    })?
                }
                None => {
                    let keys: Vec<&String> = manifest.categories.keys().collect();
                    let idx = rng.gen_range(0..keys.len());
                    &manifest.categories[keys[idx]]
                }
            };

            match peon_core::pick_sound(&cat.sounds, None, &mut rng) {
                Some(sound) => {
                    println!("Playing: \"{}\" ({})", sound.line, sound.file);
                    let sound_path = pack_path.join("sounds").join(&sound.file);
                    if sound_path.exists() {
                        platform::audio::play_sound(&sound_path, config.volume)?;
                    } else {
                        return Err(
                            format!("sound file not found: {}", sound_path.display()).into()
                        );
                    }
                }
                None => {
                    return Err("no sounds in category".into());
                }
            }
        }
        Commands::Pull { name, all } => {
            let packs_dir = paths::packs_dir(packs_dir_override.as_deref());
            std::fs::create_dir_all(&packs_dir)?;

            if all {
                println!("Pulling all packs from GitHub...");
                let remote_packs = github::list_remote_packs()?;
                if remote_packs.is_empty() {
                    return Err("no packs found on GitHub".into());
                }
                let mut installed = 0;
                for pack_name in &remote_packs {
                    match github::pull_pack(pack_name, &packs_dir) {
                        Ok(result) => {
                            println!("  {} ({} files)", result.name, result.files);
                            installed += 1;
                        }
                        Err(e) => {
                            eprintln!("  {} — failed: {}", pack_name, e);
                        }
                    }
                }
                println!("Installed {} packs.", installed);
            } else {
                let pack_name = name.ok_or("pack name required (or use --all)")?;
                println!("Pulling pack '{pack_name}' from GitHub...");
                let result = github::pull_pack(&pack_name, &packs_dir)?;
                println!("Installed: {} ({} files)", result.name, result.files);
            }
        }
    }
    Ok(())
}
