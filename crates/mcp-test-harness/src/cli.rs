//! CLI utilities for the MCP test harness

// This module can contain shared CLI utilities
// Currently, the main CLI logic is in main.rs

/// CLI utility functions
pub mod utils {
    /// Format test results for console output
    pub fn format_test_summary(passed: usize, _failed: usize, total: usize) -> String {
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        format!("{}/{} tests passed ({:.1}%)", passed, total, pass_rate)
    }

    /// Get appropriate exit code based on test results
    pub fn get_exit_code(failed_tests: usize) -> i32 {
        if failed_tests > 0 {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn test_format_test_summary() {
        assert_eq!(format_test_summary(8, 2, 10), "8/10 tests passed (80.0%)");
        assert_eq!(format_test_summary(0, 0, 0), "0/0 tests passed (0.0%)");
    }

    #[test]
    fn test_get_exit_code() {
        assert_eq!(get_exit_code(0), 0);
        assert_eq!(get_exit_code(1), 1);
        assert_eq!(get_exit_code(10), 1);
    }
}
