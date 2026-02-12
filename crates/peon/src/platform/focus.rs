use std::process::Command;

/// Check if a known terminal application is the frontmost window (macOS).
///
/// Returns `true` if a terminal app is focused, `false` otherwise.
pub fn terminal_is_focused() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first process whose frontmost is true"#)
        .output();

    match output {
        Ok(out) => {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            matches!(
                name.as_str(),
                "Terminal" | "iTerm2" | "Warp" | "Alacritty" | "kitty" | "WezTerm" | "Ghostty"
            )
        }
        Err(_) => false,
    }
}
