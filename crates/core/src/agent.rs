use std::collections::HashSet;

/// Returns `true` if the session should be treated as an agent (suppressed).
///
/// A session is considered an agent if:
/// - Its `permission_mode` is `"delegate"`, OR
/// - It was previously seen with delegate mode (recorded in `agent_sessions`).
pub fn is_agent_session(
    agent_sessions: &HashSet<String>,
    session_id: &str,
    permission_mode: &str,
) -> bool {
    permission_mode == "delegate" || agent_sessions.contains(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegate_mode_is_agent() {
        let sessions = HashSet::new();
        assert!(is_agent_session(&sessions, "s1", "delegate"));
    }

    #[test]
    fn default_mode_not_agent() {
        let sessions = HashSet::new();
        assert!(!is_agent_session(&sessions, "s1", "default"));
    }

    #[test]
    fn previously_seen_delegate_is_agent() {
        let mut sessions = HashSet::new();
        sessions.insert("s1".to_string());
        assert!(is_agent_session(&sessions, "s1", "default"));
    }

    #[test]
    fn accept_edits_not_agent() {
        let sessions = HashSet::new();
        assert!(!is_agent_session(&sessions, "s1", "acceptEdits"));
    }
}
