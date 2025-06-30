//! Utility functions for the MCP test harness

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

/// Format duration in a human-readable way
pub fn format_duration(duration_ms: u128) -> String {
    let seconds = duration_ms as f64 / 1000.0;
    if seconds < 1.0 {
        format!("{}ms", duration_ms)
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let minutes = seconds / 60.0;
        format!("{:.1}m", minutes)
    }
}

/// Sanitize a string for use as a filename
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(65000), "1.1m");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test file.txt"), "test_file_txt");
        assert_eq!(sanitize_filename("normal-name_123"), "normal-name_123");
    }
}
