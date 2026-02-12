/// Check if the user is "annoyed" — sending too many prompts in a short window.
///
/// Returns `true` if the number of timestamps within `window_secs` of `now`
/// meets or exceeds `threshold`.
pub fn check_annoyed(timestamps: &[f64], threshold: u32, window_secs: f64, now: f64) -> bool {
    let cutoff = now - window_secs;
    let recent_count = timestamps.iter().filter(|&&t| t > cutoff).count();
    recent_count >= threshold as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_not_annoyed() {
        let timestamps = vec![100.0, 105.0];
        assert!(!check_annoyed(&timestamps, 3, 10.0, 108.0));
    }

    #[test]
    fn at_threshold_is_annoyed() {
        let timestamps = vec![100.0, 103.0, 106.0];
        assert!(check_annoyed(&timestamps, 3, 10.0, 108.0));
    }

    #[test]
    fn expired_timestamps_ignored() {
        let timestamps = vec![1.0, 2.0, 100.0, 103.0];
        assert!(!check_annoyed(&timestamps, 3, 10.0, 108.0));
    }

    #[test]
    fn empty_timestamps() {
        assert!(!check_annoyed(&[], 3, 10.0, 100.0));
    }

    #[test]
    fn threshold_of_one() {
        let timestamps = vec![99.5];
        assert!(check_annoyed(&timestamps, 1, 10.0, 100.0));
    }

    #[test]
    fn exact_boundary_excluded() {
        // Timestamp at exactly cutoff (now - window) should NOT count
        let timestamps = vec![90.0, 95.0, 98.0];
        assert!(check_annoyed(&timestamps, 2, 10.0, 100.0));
        // 90.0 is exactly at cutoff (100 - 10), should not count (> not >=)
        assert!(!check_annoyed(&[90.0], 1, 10.0, 100.0));
    }
}
