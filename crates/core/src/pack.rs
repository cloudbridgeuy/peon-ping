use rand::Rng;
use std::collections::HashMap;

use crate::types::Config;

/// Resolve which pack to use for this session.
///
/// If `pack_rotation` is configured and non-empty, pins a random pack per session.
/// Otherwise returns the `active_pack` from config.
pub fn resolve_pack<'a>(
    config: &'a Config,
    session_packs: &'a HashMap<String, String>,
    session_id: &str,
    available_packs: &[String],
    rng: &mut impl Rng,
) -> String {
    if config.pack_rotation.is_empty() {
        return config.active_pack.clone();
    }

    // If session already has a pinned pack and it's still in the rotation list, use it
    if let Some(pinned) = session_packs.get(session_id) {
        if config.pack_rotation.contains(pinned) {
            return pinned.clone();
        }
    }

    // Filter rotation list to only include packs that actually exist
    let valid_rotation: Vec<&String> = config
        .pack_rotation
        .iter()
        .filter(|p| available_packs.contains(p))
        .collect();

    if valid_rotation.is_empty() {
        return config.active_pack.clone();
    }

    let idx = rng.gen_range(0..valid_rotation.len());
    valid_rotation[idx].clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn no_rotation_uses_active_pack() {
        let config = default_config();
        let session_packs = HashMap::new();
        let available = vec!["peon".to_string()];
        let mut rng = StepRng::new(0, 1);
        assert_eq!(
            resolve_pack(&config, &session_packs, "s1", &available, &mut rng),
            "peon"
        );
    }

    #[test]
    fn rotation_pins_per_session() {
        let mut config = default_config();
        config.pack_rotation = vec!["peon".into(), "sc_kerrigan".into()];
        let mut session_packs = HashMap::new();
        let available = vec!["peon".to_string(), "sc_kerrigan".to_string()];

        let mut rng = StepRng::new(0, 1);
        let pack = resolve_pack(&config, &session_packs, "s1", &available, &mut rng);
        // Pin it
        session_packs.insert("s1".into(), pack.clone());

        // Second call should return same pack
        let pack2 = resolve_pack(&config, &session_packs, "s1", &available, &mut rng);
        assert_eq!(pack, pack2);
    }

    #[test]
    fn rotation_different_sessions_can_differ() {
        let mut config = default_config();
        config.pack_rotation = vec!["a".into(), "b".into()];
        let session_packs = HashMap::new();
        let available = vec!["a".to_string(), "b".to_string()];

        // Different RNG seeds may produce different results
        let mut rng1 = StepRng::new(0, 1);
        let mut rng2 = StepRng::new(1, 1);
        let _pack1 = resolve_pack(&config, &session_packs, "s1", &available, &mut rng1);
        let _pack2 = resolve_pack(&config, &session_packs, "s2", &available, &mut rng2);
        // Just verify they don't panic — actual values depend on RNG
    }

    #[test]
    fn rotation_with_invalid_packs_falls_back() {
        let mut config = default_config();
        config.pack_rotation = vec!["nonexistent".into()];
        let session_packs = HashMap::new();
        let available = vec!["peon".to_string()];
        let mut rng = StepRng::new(0, 1);
        assert_eq!(
            resolve_pack(&config, &session_packs, "s1", &available, &mut rng),
            "peon"
        );
    }
}
