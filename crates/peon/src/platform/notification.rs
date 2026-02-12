use peon_core::types::NotifyColor;
use std::process::Command;

/// Send a macOS notification via osascript. Non-blocking.
pub fn send_notification(
    message: &str,
    title: &str,
    _color: &NotifyColor,
) -> Result<(), std::io::Error> {
    let script = format!(
        r#"display notification "{}" with title "{}""#,
        message.replace('"', r#"\""#),
        title.replace('"', r#"\""#),
    );
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}
