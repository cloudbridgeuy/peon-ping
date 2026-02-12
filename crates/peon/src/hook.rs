use peon_core::types::{Action, HookEvent};
use peon_core::{check_annoyed, is_agent_session, pick_sound, resolve_pack, route_event};
use rand::thread_rng;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;
use crate::platform;
use crate::state_io;

#[derive(thiserror::Error, Debug)]
pub enum HookError {
    #[error("Failed to read stdin: {0}")]
    Stdin(#[from] std::io::Error),
    #[error("Failed to parse hook event: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("State I/O error: {0}")]
    StateIo(#[from] crate::state_io::StateIoError),
}

/// Main hook handler: reads JSON from stdin, routes event, executes actions.
pub fn handle_hook() -> Result<(), HookError> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        return Ok(());
    }

    let event: HookEvent = serde_json::from_str(&input)?;
    let config = state_io::load_config(&paths::config_path());
    let mut state = state_io::load_state(&paths::state_path());
    let paused = state_io::is_paused(&paths::paused_path());

    // Agent detection — suppress sounds for delegate sessions
    if is_agent_session(
        &state.agent_sessions,
        event.session_id(),
        event.permission_mode(),
    ) {
        // Record this session as agent if newly detected
        if event.permission_mode() == "delegate"
            && !state.agent_sessions.contains(event.session_id())
        {
            state.agent_sessions.insert(event.session_id().to_string());
            let _ = state_io::save_state(&paths::state_path(), &state);
        }
        return Ok(());
    }

    // If delegate mode, record and exit
    if event.permission_mode() == "delegate" {
        state.agent_sessions.insert(event.session_id().to_string());
        let _ = state_io::save_state(&paths::state_path(), &state);
        return Ok(());
    }

    let mut state_dirty = false;
    let mut rng = thread_rng();

    // Annoyed detection for UserPromptSubmit
    let annoyed = if matches!(event, HookEvent::UserPromptSubmit { .. })
        && config.categories.is_enabled("annoyed")
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let session_id = event.session_id().to_string();
        let timestamps = state.prompt_timestamps.entry(session_id).or_default();

        // Filter to only recent timestamps and add current
        let window = config.annoyed_window_seconds;
        timestamps.retain(|&t| now - t < window);
        timestamps.push(now);
        state_dirty = true;

        check_annoyed(timestamps, config.annoyed_threshold, window, now)
    } else {
        false
    };

    // Route event to actions
    let mut actions = route_event(&event, &config, &state);

    // If annoyed, add a PlaySound for annoyed category
    if annoyed && config.categories.is_enabled("annoyed") {
        actions.push(Action::PlaySound {
            category: "annoyed".into(),
        });
    }

    // Resolve active pack
    let packs = state_io::list_packs(&paths::packs_dir());
    let available_pack_names: Vec<String> = packs.iter().map(|(name, _)| name.clone()).collect();
    let active_pack = resolve_pack(
        &config,
        &state.session_packs,
        event.session_id(),
        &available_pack_names,
        &mut rng,
    );

    // If pack rotation assigned a new pack, record it
    if !config.pack_rotation.is_empty() {
        let session_id = event.session_id().to_string();
        if state.session_packs.get(&session_id) != Some(&active_pack) {
            state.session_packs.insert(session_id, active_pack.clone());
            state_dirty = true;
        }
    }

    // Load manifest for the active pack
    let manifest = packs
        .iter()
        .find(|(name, _)| name == &active_pack)
        .map(|(_, m)| m);

    // Execute actions
    for action in &actions {
        match action {
            Action::SetTabTitle { title } => {
                let escape = peon_core::tab_title::tab_title_escape(title);
                print!("{escape}");
            }
            Action::PlaySound { category } => {
                if paused {
                    continue;
                }
                if let Some(manifest) = manifest {
                    if let Some(cat) = manifest.categories.get(category.as_str()) {
                        let last = state.last_played.get(category.as_str()).map(|s| s.as_str());
                        if let Some(sound) = pick_sound(&cat.sounds, last, &mut rng) {
                            state
                                .last_played
                                .insert(category.clone(), sound.file.clone());
                            state_dirty = true;

                            let sound_path = paths::packs_dir()
                                .join(&active_pack)
                                .join("sounds")
                                .join(&sound.file);
                            if sound_path.exists() {
                                let _ = platform::audio::play_sound(&sound_path, config.volume);
                            }
                        }
                    }
                }
            }
            Action::Notify {
                message,
                title,
                color,
            } => {
                if paused {
                    continue;
                }
                if !platform::focus::terminal_is_focused() {
                    let _ = platform::notification::send_notification(message, title, color);
                }
            }
            Action::Skip => {}
        }
    }

    // Show pause notice on SessionStart
    if matches!(event, HookEvent::SessionStart { .. }) && paused {
        eprintln!("peon-ping: sounds paused — run 'peon resume' or '/peon-ping-toggle' to unpause");
    }

    // Save state if modified
    if state_dirty {
        let _ = state_io::save_state(&paths::state_path(), &state);
    }

    Ok(())
}
