use crate::types::Manifest;

/// Format a manifest's categories and voice lines as a human-readable string.
pub fn format_pack_sounds(manifest: &Manifest) -> String {
    let display = if manifest.display_name.is_empty() {
        &manifest.name
    } else {
        &manifest.display_name
    };

    let mut out = format!("{display} ({})\n", manifest.name);

    if manifest.categories.is_empty() {
        out.push_str("\n  No categories.\n");
        return out;
    }

    let mut categories: Vec<_> = manifest.categories.iter().collect();
    categories.sort_by_key(|(name, _)| name.as_str().to_owned());

    for (name, category) in categories {
        let count = category.sounds.len();
        let label = if count == 1 { "sound" } else { "sounds" };
        out.push_str(&format!("\n  {name} ({count} {label})\n"));
        for sound in &category.sounds {
            out.push_str(&format!("    \"{}\"\n", sound.line));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Category, Manifest, Sound};
    use std::collections::HashMap;

    fn test_manifest() -> Manifest {
        let mut categories = HashMap::new();
        categories.insert(
            "greeting".to_string(),
            Category {
                sounds: vec![
                    Sound {
                        file: "PeonReady1.wav".into(),
                        line: "Ready to work?".into(),
                    },
                    Sound {
                        file: "PeonWhat1.wav".into(),
                        line: "Yes?".into(),
                    },
                ],
            },
        );
        categories.insert(
            "annoyed".to_string(),
            Category {
                sounds: vec![Sound {
                    file: "PeonAngry1.wav".into(),
                    line: "Whaaat?".into(),
                }],
            },
        );
        Manifest {
            name: "peon".into(),
            display_name: "Orc Peon".into(),
            categories,
        }
    }

    #[test]
    fn format_pack_sounds_shows_display_name_and_id() {
        let output = format_pack_sounds(&test_manifest());
        assert!(output.starts_with("Orc Peon (peon)\n"));
    }

    #[test]
    fn format_pack_sounds_shows_categories_with_count() {
        let output = format_pack_sounds(&test_manifest());
        assert!(output.contains("greeting (2 sounds)"));
        assert!(output.contains("annoyed (1 sound)"));
    }

    #[test]
    fn format_pack_sounds_shows_voice_lines_quoted() {
        let output = format_pack_sounds(&test_manifest());
        assert!(output.contains("\"Ready to work?\""));
        assert!(output.contains("\"Yes?\""));
        assert!(output.contains("\"Whaaat?\""));
    }

    #[test]
    fn format_pack_sounds_categories_sorted_alphabetically() {
        let output = format_pack_sounds(&test_manifest());
        let annoyed_pos = output.find("annoyed").unwrap();
        let greeting_pos = output.find("greeting").unwrap();
        assert!(annoyed_pos < greeting_pos);
    }

    #[test]
    fn format_pack_sounds_empty_categories() {
        let manifest = Manifest {
            name: "empty".into(),
            display_name: "Empty Pack".into(),
            categories: HashMap::new(),
        };
        let output = format_pack_sounds(&manifest);
        assert_eq!(output, "Empty Pack (empty)\n\n  No categories.\n");
    }

    #[test]
    fn format_pack_sounds_uses_name_when_display_name_empty() {
        let manifest = Manifest {
            name: "minimal".into(),
            display_name: String::new(),
            categories: HashMap::new(),
        };
        let output = format_pack_sounds(&manifest);
        assert!(output.starts_with("minimal (minimal)\n"));
    }
}
