use serde::Deserialize;

/// Hook events received from Claude Code via stdin JSON.
///
/// Uses serde's internally-tagged enum representation keyed on `hook_event_name`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "hook_event_name")]
pub enum HookEvent {
    SessionStart {
        #[serde(default)]
        cwd: String,
        #[serde(default)]
        session_id: String,
        #[serde(default)]
        permission_mode: String,
    },
    UserPromptSubmit {
        #[serde(default)]
        cwd: String,
        #[serde(default)]
        session_id: String,
        #[serde(default)]
        permission_mode: String,
    },
    Stop {
        #[serde(default)]
        cwd: String,
        #[serde(default)]
        session_id: String,
        #[serde(default)]
        permission_mode: String,
    },
    Notification {
        #[serde(default)]
        cwd: String,
        #[serde(default)]
        session_id: String,
        #[serde(default)]
        permission_mode: String,
        #[serde(default)]
        notification_type: String,
    },
    PermissionRequest {
        #[serde(default)]
        cwd: String,
        #[serde(default)]
        session_id: String,
        #[serde(default)]
        permission_mode: String,
        #[serde(default)]
        tool_name: String,
        #[serde(default)]
        tool_input: serde_json::Value,
    },
}

impl HookEvent {
    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStart { session_id, .. }
            | Self::UserPromptSubmit { session_id, .. }
            | Self::Stop { session_id, .. }
            | Self::Notification { session_id, .. }
            | Self::PermissionRequest { session_id, .. } => session_id,
        }
    }

    pub fn permission_mode(&self) -> &str {
        match self {
            Self::SessionStart {
                permission_mode, ..
            }
            | Self::UserPromptSubmit {
                permission_mode, ..
            }
            | Self::Stop {
                permission_mode, ..
            }
            | Self::Notification {
                permission_mode, ..
            }
            | Self::PermissionRequest {
                permission_mode, ..
            } => permission_mode,
        }
    }

    pub fn cwd(&self) -> &str {
        match self {
            Self::SessionStart { cwd, .. }
            | Self::UserPromptSubmit { cwd, .. }
            | Self::Stop { cwd, .. }
            | Self::Notification { cwd, .. }
            | Self::PermissionRequest { cwd, .. } => cwd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_session_start() {
        let json = r#"{"hook_event_name":"SessionStart","cwd":"/tmp/project","session_id":"abc-123","permission_mode":"default"}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, HookEvent::SessionStart { .. }));
        assert_eq!(event.session_id(), "abc-123");
        assert_eq!(event.cwd(), "/tmp/project");
    }

    #[test]
    fn deserialize_notification() {
        let json = r#"{"hook_event_name":"Notification","cwd":"/tmp","session_id":"s1","permission_mode":"default","notification_type":"permission_prompt"}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        if let HookEvent::Notification {
            notification_type, ..
        } = &event
        {
            assert_eq!(notification_type, "permission_prompt");
        } else {
            panic!("expected Notification variant");
        }
    }

    #[test]
    fn deserialize_permission_request() {
        let json = r#"{"hook_event_name":"PermissionRequest","cwd":"/tmp","session_id":"s1","permission_mode":"default","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        if let HookEvent::PermissionRequest {
            tool_name,
            tool_input,
            ..
        } = &event
        {
            assert_eq!(tool_name, "Bash");
            assert_eq!(tool_input["command"], "ls");
        } else {
            panic!("expected PermissionRequest variant");
        }
    }

    #[test]
    fn deserialize_delegate_mode() {
        let json = r#"{"hook_event_name":"SessionStart","cwd":"/tmp","session_id":"agent-1","permission_mode":"delegate"}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.permission_mode(), "delegate");
    }

    #[test]
    fn deserialize_with_extra_fields() {
        let json = r#"{"hook_event_name":"Stop","cwd":"/tmp","session_id":"s1","permission_mode":"default","extra_field":"ignored"}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, HookEvent::Stop { .. }));
    }
}
