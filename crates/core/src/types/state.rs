use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    #[serde(default)]
    pub last_played: HashMap<String, String>,
    #[serde(default)]
    pub agent_sessions: HashSet<String>,
    #[serde(default, deserialize_with = "deserialize_prompt_timestamps")]
    pub prompt_timestamps: HashMap<String, Vec<f64>>,
    #[serde(default)]
    pub session_packs: HashMap<String, String>,
}

/// The bash/python version could write prompt_timestamps as either a list (legacy) or
/// a map. We handle both by defaulting to empty map if deserialization fails for the
/// legacy list format. In practice the Rust version always writes the map format.
fn deserialize_prompt_timestamps<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<f64>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                let timestamps: Vec<f64> = serde_json::from_value(v).map_err(D::Error::custom)?;
                result.insert(k, timestamps);
            }
            Ok(result)
        }
        serde_json::Value::Array(_) => {
            // Legacy format: was a flat list, discard and start fresh
            Ok(HashMap::new())
        }
        serde_json::Value::Null => Ok(HashMap::new()),
        _ => Err(D::Error::custom(
            "expected object or array for prompt_timestamps",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_state() {
        let state: State = serde_json::from_str("{}").unwrap();
        assert!(state.last_played.is_empty());
        assert!(state.agent_sessions.is_empty());
        assert!(state.prompt_timestamps.is_empty());
        assert!(state.session_packs.is_empty());
    }

    #[test]
    fn deserialize_full_state() {
        let json = r#"{
            "last_played": {"greeting": "PeonReady1.wav", "complete": "PeonYes1.wav"},
            "agent_sessions": ["agent-123", "agent-456"],
            "prompt_timestamps": {
                "session-1": [1708000000.123, 1708000001.456],
                "session-2": [1708000010.0]
            },
            "session_packs": {
                "session-1": "sc_kerrigan"
            }
        }"#;

        let state: State = serde_json::from_str(json).unwrap();
        assert_eq!(state.last_played.get("greeting").unwrap(), "PeonReady1.wav");
        assert!(state.agent_sessions.contains("agent-123"));
        assert_eq!(state.prompt_timestamps["session-1"].len(), 2);
        assert_eq!(state.session_packs["session-1"], "sc_kerrigan");
    }

    #[test]
    fn deserialize_legacy_list_timestamps() {
        let json = r#"{
            "prompt_timestamps": [1.0, 2.0, 3.0]
        }"#;
        let state: State = serde_json::from_str(json).unwrap();
        assert!(state.prompt_timestamps.is_empty());
    }

    #[test]
    fn round_trip_state() {
        let mut state = State::default();
        state
            .last_played
            .insert("greeting".into(), "test.wav".into());
        state.agent_sessions.insert("sess-1".into());

        let json = serde_json::to_string(&state).unwrap();
        let restored: State = serde_json::from_str(&json).unwrap();
        assert_eq!(state.last_played, restored.last_played);
    }
}
