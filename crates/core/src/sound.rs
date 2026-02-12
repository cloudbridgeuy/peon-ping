use crate::types::Sound;
use rand::Rng;

/// Pick a random sound from the list, avoiding the last-played file when possible.
///
/// Returns `None` if the sound list is empty.
pub fn pick_sound<'a>(
    sounds: &'a [Sound],
    last_played: Option<&str>,
    rng: &mut impl Rng,
) -> Option<&'a Sound> {
    if sounds.is_empty() {
        return None;
    }
    if sounds.len() == 1 {
        return Some(&sounds[0]);
    }

    let candidates: Vec<&Sound> = match last_played {
        Some(last) => {
            let filtered: Vec<_> = sounds.iter().filter(|s| s.file != last).collect();
            if filtered.is_empty() {
                sounds.iter().collect()
            } else {
                filtered
            }
        }
        None => sounds.iter().collect(),
    };

    let idx = rng.gen_range(0..candidates.len());
    Some(candidates[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    fn make_sounds(files: &[&str]) -> Vec<Sound> {
        files
            .iter()
            .map(|f| Sound {
                file: f.to_string(),
                line: String::new(),
            })
            .collect()
    }

    #[test]
    fn empty_list_returns_none() {
        let mut rng = StepRng::new(0, 1);
        assert!(pick_sound(&[], None, &mut rng).is_none());
    }

    #[test]
    fn single_sound_always_returned() {
        let sounds = make_sounds(&["only.wav"]);
        let mut rng = StepRng::new(0, 1);
        let picked = pick_sound(&sounds, Some("only.wav"), &mut rng);
        assert_eq!(picked.map(|s| s.file.as_str()), Some("only.wav"));
    }

    #[test]
    fn avoids_last_played() {
        let sounds = make_sounds(&["a.wav", "b.wav"]);
        let mut rng = StepRng::new(0, 1);
        // With last_played = "a.wav", should pick "b.wav"
        let picked = pick_sound(&sounds, Some("a.wav"), &mut rng);
        assert_eq!(picked.map(|s| s.file.as_str()), Some("b.wav"));
    }

    #[test]
    fn no_last_played_picks_from_all() {
        let sounds = make_sounds(&["a.wav", "b.wav", "c.wav"]);
        let mut rng = StepRng::new(0, 1);
        let picked = pick_sound(&sounds, None, &mut rng);
        assert!(picked.is_some());
    }
}
