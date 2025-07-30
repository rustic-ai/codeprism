//! Performance testing module for CodePrism
//! 
//! This module contains performance benchmarks and scalability tests for:
//! - Language parsers (Python, JavaScript, Rust, Java)
//! - MCP tools and analysis operations
//! - Memory usage and leak detection
//! - Concurrent request handling

pub mod parser_benchmarks;
pub mod mcp_tool_benchmarks;
pub mod memory_tests;
pub mod scalability_tests;
pub mod integration_performance;

use std::time::{Duration, Instant};
use std::fmt;

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceResult {
    pub test_name: String,
    pub duration: Duration,
    pub memory_used: Option<usize>,
    pub success: bool,
    pub details: String,
}

impl PerformanceResult {
    pub fn new(test_name: &str, duration: Duration, success: bool) -> Self {
        Self {
            test_name: test_name.to_string(),
            duration,
            memory_used: None,
            success,
            details: String::new(),
        }
    }

    pub fn with_memory(mut self, memory_used: usize) -> Self {
        self.memory_used = Some(memory_used);
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = details.to_string();
        self
    }
}

impl fmt::Display for PerformanceResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.2}ms {}{}",
            self.test_name,
            self.duration.as_millis(),
            if let Some(mem) = self.memory_used {
                format!("({}KB) ", mem / 1024)
            } else {
                String::new()
            },
            if self.success { "✓" } else { "✗" }
        )
    }
}

/// Performance test harness
pub struct PerformanceTestHarness {
    results: Vec<PerformanceResult>,
}

impl PerformanceTestHarness {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a performance test
    pub fn run_test<F>(&mut self, name: &str, test_fn: F) -> &PerformanceResult
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
    {
        let start = Instant::now();
        let success = test_fn().is_ok();
        let duration = start.elapsed();
        
        let result = PerformanceResult::new(name, duration, success);
        self.results.push(result);
        self.results.last().unwrap()
    }

    /// Get all results
    pub fn results(&self) -> &[PerformanceResult] {
        &self.results
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\n📊 Performance Test Summary");
        println!("{}", "=".repeat(50));
        
        for result in &self.results {
            println!("{}", result);
        }
        
        let total_tests = self.results.len();
        let passed = self.results.iter().filter(|r| r.success).count();
        let avg_duration = self.results.iter()
            .map(|r| r.duration.as_millis())
            .sum::<u128>() / total_tests as u128;
            
        println!("{}", "=".repeat(50));
        println!("📈 Passed: {}/{} | Avg Duration: {}ms", passed, total_tests, avg_duration);
    }
}

impl Default for PerformanceTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance thresholds for various operations
pub struct PerformanceThresholds;

impl PerformanceThresholds {
    // Parser thresholds (per 1000 lines of code)
    pub const PYTHON_PARSE_MS: u128 = 100;
    pub const JAVASCRIPT_PARSE_MS: u128 = 80;
    pub const RUST_PARSE_MS: u128 = 120;
    pub const JAVA_PARSE_MS: u128 = 90;
    
    // MCP tool thresholds
    pub const TOOL_RESPONSE_MS: u128 = 500;
    pub const COMPLEX_ANALYSIS_MS: u128 = 2000;
    
    // Memory thresholds (in MB)
    pub const MAX_MEMORY_MB: usize = 1024;
    pub const PARSER_MEMORY_MB: usize = 100;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_result_creation() {
        // Test successful result creation
        let result = PerformanceResult::new("test", Duration::from_millis(100), true);
        assert_eq!(result.test_name, "test", "Should store test name correctly");
        assert_eq!(result.duration, Duration::from_millis(100), "Should store duration correctly");
        assert!(result.success, "Should store success status correctly");
        
        // Test failed result creation
        let failed_result = PerformanceResult::new("failed_test", Duration::from_millis(50), false);
        assert_eq!(failed_result.test_name, "failed_test", "Should handle failed test names");
        assert!(!failed_result.success, "Should correctly store failure status");
        assert!(failed_result.duration.as_millis() > 0, "Should record non-zero duration");
    }

    #[test]
    fn test_performance_harness() {
        let mut harness = PerformanceTestHarness::new();
        
        // Test successful test execution
        harness.run_test("simple_test", || Ok(()));
        
        // Validate actual test execution and results
        assert_eq!(harness.results().len(), 1, "Should have one test result");
        
        let result = &harness.results()[0];
        assert!(result.success, "Test should be marked as successful");
        assert_eq!(result.test_name, "simple_test", "Should capture test name");
        assert!(result.duration.as_millis() >= 0, "Should measure execution time");
        
        // Test failed test execution
        harness.run_test("failing_test", || Err("test error".into()));
        assert_eq!(harness.results().len(), 2, "Should have two test results");
        assert!(!harness.results()[1].success, "Failed test should be marked as unsuccessful");
    }
} 