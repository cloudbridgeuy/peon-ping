/// Build a tab title string. The shell layer wraps this in an ANSI escape sequence.
///
/// The `marker` is prepended (e.g. "● " for notifications).
pub fn build_tab_title(project: &str, status: &str, marker: &str) -> String {
    format!("{marker}{project}: {status}")
}

/// Generate the ANSI escape sequence to set the terminal tab title.
pub fn tab_title_escape(title: &str) -> String {
    format!("\x1b]0;{title}\x07")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_title() {
        assert_eq!(
            build_tab_title("my-project", "ready", ""),
            "my-project: ready"
        );
    }

    #[test]
    fn title_with_marker() {
        assert_eq!(
            build_tab_title("my-project", "done", "\u{25cf} "),
            "\u{25cf} my-project: done"
        );
    }

    #[test]
    fn escape_sequence() {
        let title = build_tab_title("proj", "ready", "");
        let escaped = tab_title_escape(&title);
        assert_eq!(escaped, "\x1b]0;proj: ready\x07");
    }
}
