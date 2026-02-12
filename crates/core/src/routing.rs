use crate::types::{Action, Config, HookEvent, NotifyColor, State};

/// Extract the project name from a cwd path.
/// Sanitizes to only allow [a-zA-Z0-9 ._-].
pub fn extract_project_name(cwd: &str) -> String {
    let name = cwd.rsplit('/').next().unwrap_or("claude");
    let name = if name.is_empty() { "claude" } else { name };
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '.' || *c == '_' || *c == '-')
        .collect::<String>()
}

/// Route a hook event to a list of actions.
///
/// This is the main pure routing function. It does NOT handle:
/// - Agent detection (caller should check first)
/// - Sound selection (caller picks sound based on category from PlaySound action)
/// - Annoyed detection (caller checks timestamps and may override category)
///
/// Returns a list of actions to execute.
pub fn route_event(event: &HookEvent, config: &Config, _state: &State) -> Vec<Action> {
    if !config.enabled {
        return vec![Action::Skip];
    }

    let project = extract_project_name(event.cwd());

    match event {
        HookEvent::SessionStart { .. } => {
            let mut actions = vec![];
            actions.push(Action::SetTabTitle {
                title: crate::tab_title::build_tab_title(&project, "ready", ""),
            });
            if config.categories.is_enabled("greeting") {
                actions.push(Action::PlaySound {
                    category: "greeting".into(),
                });
            }
            actions
        }
        HookEvent::UserPromptSubmit { .. } => {
            // Tab title only — sound is only played if annoyed threshold met
            // The caller handles annoyed detection and may add a PlaySound action
            vec![Action::SetTabTitle {
                title: crate::tab_title::build_tab_title(&project, "working", ""),
            }]
        }
        HookEvent::Stop { .. } => {
            let mut actions = vec![];
            actions.push(Action::SetTabTitle {
                title: crate::tab_title::build_tab_title(&project, "done", "\u{25cf} "),
            });
            if config.categories.is_enabled("complete") {
                actions.push(Action::PlaySound {
                    category: "complete".into(),
                });
            }
            actions.push(Action::Notify {
                message: format!("{project}  \u{2014}  Task complete"),
                title: crate::tab_title::build_tab_title(&project, "done", "\u{25cf} "),
                color: NotifyColor::Blue,
            });
            actions
        }
        HookEvent::Notification {
            notification_type, ..
        } => match notification_type.as_str() {
            "permission_prompt" => {
                let mut actions = vec![];
                actions.push(Action::SetTabTitle {
                    title: crate::tab_title::build_tab_title(
                        &project,
                        "needs approval",
                        "\u{25cf} ",
                    ),
                });
                if config.categories.is_enabled("permission") {
                    actions.push(Action::PlaySound {
                        category: "permission".into(),
                    });
                }
                actions.push(Action::Notify {
                    message: format!("{project}  \u{2014}  Permission needed"),
                    title: crate::tab_title::build_tab_title(
                        &project,
                        "needs approval",
                        "\u{25cf} ",
                    ),
                    color: NotifyColor::Red,
                });
                actions
            }
            "idle_prompt" => {
                let mut actions = vec![];
                actions.push(Action::SetTabTitle {
                    title: crate::tab_title::build_tab_title(&project, "done", "\u{25cf} "),
                });
                actions.push(Action::Notify {
                    message: format!("{project}  \u{2014}  Waiting for input"),
                    title: crate::tab_title::build_tab_title(&project, "done", "\u{25cf} "),
                    color: NotifyColor::Yellow,
                });
                actions
            }
            _ => vec![Action::Skip],
        },
        HookEvent::PermissionRequest { .. } => {
            let mut actions = vec![];
            actions.push(Action::SetTabTitle {
                title: crate::tab_title::build_tab_title(&project, "needs approval", "\u{25cf} "),
            });
            if config.categories.is_enabled("permission") {
                actions.push(Action::PlaySound {
                    category: "permission".into(),
                });
            }
            actions.push(Action::Notify {
                message: format!("{project}  \u{2014}  Permission needed"),
                title: crate::tab_title::build_tab_title(&project, "needs approval", "\u{25cf} "),
                color: NotifyColor::Red,
            });
            actions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Config;

    fn default_event(name: &str) -> HookEvent {
        let json = format!(
            r#"{{"hook_event_name":"{}","cwd":"/home/user/my-project","session_id":"s1","permission_mode":"default"}}"#,
            name
        );
        serde_json::from_str(&json).expect("valid event JSON")
    }

    #[test]
    fn extract_project_from_cwd() {
        assert_eq!(extract_project_name("/home/user/my-project"), "my-project");
        assert_eq!(extract_project_name("/tmp"), "tmp");
        assert_eq!(extract_project_name(""), "claude");
        assert_eq!(extract_project_name("/"), "claude");
    }

    #[test]
    fn extract_project_sanitizes() {
        assert_eq!(
            extract_project_name("/home/user/my project!@#"),
            "my project"
        );
    }

    #[test]
    fn session_start_produces_title_and_sound() {
        let event = default_event("SessionStart");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetTabTitle { .. })));
        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { category } if category == "greeting")));
    }

    #[test]
    fn user_prompt_submit_only_title() {
        let event = default_event("UserPromptSubmit");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetTabTitle { .. })));
        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { .. })));
    }

    #[test]
    fn stop_produces_title_sound_notify() {
        let event = default_event("Stop");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetTabTitle { .. })));
        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { category } if category == "complete")));
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Notify {
                color: NotifyColor::Blue,
                ..
            }
        )));
    }

    #[test]
    fn notification_permission_prompt() {
        let json = r#"{"hook_event_name":"Notification","cwd":"/tmp/proj","session_id":"s1","permission_mode":"default","notification_type":"permission_prompt"}"#;
        let event: HookEvent = serde_json::from_str(json).expect("valid");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { category } if category == "permission")));
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Notify {
                color: NotifyColor::Red,
                ..
            }
        )));
    }

    #[test]
    fn notification_idle_prompt_no_sound() {
        let json = r#"{"hook_event_name":"Notification","cwd":"/tmp/proj","session_id":"s1","permission_mode":"default","notification_type":"idle_prompt"}"#;
        let event: HookEvent = serde_json::from_str(json).expect("valid");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { .. })));
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Notify {
                color: NotifyColor::Yellow,
                ..
            }
        )));
    }

    #[test]
    fn unknown_notification_type_skips() {
        let json = r#"{"hook_event_name":"Notification","cwd":"/tmp","session_id":"s1","permission_mode":"default","notification_type":"something_else"}"#;
        let event: HookEvent = serde_json::from_str(json).expect("valid");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);
        assert_eq!(actions, vec![Action::Skip]);
    }

    #[test]
    fn disabled_config_skips() {
        let event = default_event("SessionStart");
        let config = Config {
            enabled: false,
            ..Default::default()
        };
        let state = State::default();
        let actions = route_event(&event, &config, &state);
        assert_eq!(actions, vec![Action::Skip]);
    }

    #[test]
    fn disabled_category_no_sound() {
        let event = default_event("SessionStart");
        let mut config = Config::default();
        config.categories.greeting = false;
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::SetTabTitle { .. })));
        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { .. })));
    }

    #[test]
    fn permission_request_event() {
        let json = r#"{"hook_event_name":"PermissionRequest","cwd":"/tmp/proj","session_id":"s1","permission_mode":"default","tool_name":"Bash","tool_input":{}}"#;
        let event: HookEvent = serde_json::from_str(json).expect("valid");
        let config = Config::default();
        let state = State::default();
        let actions = route_event(&event, &config, &state);

        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::PlaySound { category } if category == "permission")));
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Notify {
                color: NotifyColor::Red,
                ..
            }
        )));
    }
}
