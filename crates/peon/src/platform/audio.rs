use std::path::Path;
use std::process::Command;

/// Play a sound file via afplay (macOS). Non-blocking (spawns background process).
pub fn play_sound(file: &Path, volume: f64) -> Result<(), std::io::Error> {
    Command::new("afplay")
        .arg("-v")
        .arg(volume.to_string())
        .arg(file)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}
