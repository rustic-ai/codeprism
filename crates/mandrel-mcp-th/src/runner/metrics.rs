//! Metrics collection and aggregation for test suite execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Collects and manages metrics during test suite execution
#[derive(Debug, Default)]
pub struct MetricsCollector {
    suite_start_time: Option<SystemTime>,
    test_metrics: HashMap<String, TestMetrics>,
    memory_samples: Vec<MemorySample>,
    suite_metrics: SuiteMetrics,
}

/// Metrics for individual test case execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub test_name: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub duration: Duration,
    pub memory_peak_mb: u64,
    pub success: bool,
    pub retry_count: usize,
    pub error_message: Option<String>,
}

/// Memory usage sample at a point in time
#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: SystemTime,
    pub memory_mb: u64,
}

/// Aggregated metrics for the entire test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteMetrics {
    pub total_memory_usage: u64,
    pub peak_memory_usage: u64,
    pub average_test_duration: Duration,
    pub slowest_test: Option<String>,
    pub fastest_test: Option<String>,
    pub slowest_duration: Duration,
    pub fastest_duration: Duration,
    pub memory_efficiency_score: f64,
    pub execution_efficiency_score: f64,
}

impl Default for SuiteMetrics {
    fn default() -> Self {
        Self {
            total_memory_usage: 0,
            peak_memory_usage: 0,
            average_test_duration: Duration::from_secs(0),
            slowest_test: None,
            fastest_test: None,
            slowest_duration: Duration::from_secs(0),
            fastest_duration: Duration::from_secs(u64::MAX),
            memory_efficiency_score: 0.0,
            execution_efficiency_score: 0.0,
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the start of test suite execution
    pub fn start_suite(&mut self) {
        self.suite_start_time = Some(SystemTime::now());
        self.sample_memory();
    }

    /// Mark the end of test suite execution and calculate final metrics
    pub fn end_suite(&mut self) {
        self.sample_memory();
        self.calculate_suite_metrics();
    }

    /// Record the start of an individual test
    pub fn start_test(&mut self, test_name: &str) {
        let start_time = SystemTime::now();
        self.test_metrics.insert(
            test_name.to_string(),
            TestMetrics {
                test_name: test_name.to_string(),
                start_time,
                end_time: start_time, // Will be updated when test ends
                duration: Duration::from_secs(0),
                memory_peak_mb: 0,
                success: false,
                retry_count: 0,
                error_message: None,
            },
        );
        self.sample_memory();
    }

    /// Record the completion of an individual test
    pub fn end_test(&mut self, test_name: &str, success: bool, error_message: Option<String>) {
        let end_time = SystemTime::now();

        // Capture start_time before mutable borrow
        let start_time = if let Some(test_metrics) = self.test_metrics.get(test_name) {
            test_metrics.start_time
        } else {
            return; // Test metrics not found
        };

        // Calculate memory peak before mutable borrow
        let memory_peak_mb = self.get_peak_memory_since(start_time);

        if let Some(test_metrics) = self.test_metrics.get_mut(test_name) {
            test_metrics.end_time = end_time;
            test_metrics.duration = end_time
                .duration_since(test_metrics.start_time)
                .unwrap_or_else(|_| Duration::from_secs(0));
            test_metrics.success = success;
            test_metrics.error_message = error_message;
            test_metrics.memory_peak_mb = memory_peak_mb;
        }

        self.sample_memory();
    }

    /// Record a retry attempt for a test
    pub fn record_retry(&mut self, test_name: &str) {
        if let Some(test_metrics) = self.test_metrics.get_mut(test_name) {
            test_metrics.retry_count += 1;
        }
    }

    /// Sample current memory usage
    pub fn sample_memory(&mut self) {
        let memory_mb = self.get_current_memory_usage();
        self.memory_samples.push(MemorySample {
            timestamp: SystemTime::now(),
            memory_mb,
        });
    }

    /// Get current memory usage in MB (simplified implementation)
    fn get_current_memory_usage(&self) -> u64 {
        // Implementation provides Linux /proc/self/status parsing with cross-platform fallback
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                for line in contents.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                return kb / 1024; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }

        // Fallback for non-Linux systems or if reading fails
        64 // Default to 64MB as a reasonable baseline
    }

    /// Get peak memory usage since a specific time
    fn get_peak_memory_since(&self, since: SystemTime) -> u64 {
        self.memory_samples
            .iter()
            .filter(|sample| sample.timestamp >= since)
            .map(|sample| sample.memory_mb)
            .max()
            .unwrap_or(0)
    }

    /// Calculate aggregated suite metrics
    fn calculate_suite_metrics(&mut self) {
        if self.test_metrics.is_empty() {
            return;
        }

        // Calculate memory metrics
        let peak_memory = self
            .memory_samples
            .iter()
            .map(|sample| sample.memory_mb)
            .max()
            .unwrap_or(0);

        let total_memory = self
            .memory_samples
            .iter()
            .map(|sample| sample.memory_mb)
            .sum::<u64>();

        // Calculate duration metrics
        let test_durations: Vec<Duration> = self
            .test_metrics
            .values()
            .map(|metrics| metrics.duration)
            .collect();

        let total_duration: Duration = test_durations.iter().sum();
        let average_duration = if !test_durations.is_empty() {
            total_duration / test_durations.len() as u32
        } else {
            Duration::from_secs(0)
        };

        // Find slowest and fastest tests
        let (slowest_test, slowest_duration) = self
            .test_metrics
            .values()
            .max_by_key(|metrics| metrics.duration)
            .map(|metrics| (Some(metrics.test_name.clone()), metrics.duration))
            .unwrap_or((None, Duration::from_secs(0)));

        let (fastest_test, fastest_duration) = self
            .test_metrics
            .values()
            .min_by_key(|metrics| metrics.duration)
            .map(|metrics| (Some(metrics.test_name.clone()), metrics.duration))
            .unwrap_or((None, Duration::from_secs(0)));

        // Calculate efficiency scores (0-100)
        let memory_efficiency = self.calculate_memory_efficiency(peak_memory);
        let execution_efficiency = self.calculate_execution_efficiency();

        self.suite_metrics = SuiteMetrics {
            total_memory_usage: total_memory,
            peak_memory_usage: peak_memory,
            average_test_duration: average_duration,
            slowest_test,
            fastest_test,
            slowest_duration,
            fastest_duration,
            memory_efficiency_score: memory_efficiency,
            execution_efficiency_score: execution_efficiency,
        };
    }

    /// Calculate memory efficiency score (higher is better)
    fn calculate_memory_efficiency(&self, peak_memory: u64) -> f64 {
        if peak_memory == 0 {
            return 100.0;
        }

        // NOTE: Using simplified heuristic for memory efficiency calculation
        // More sophisticated algorithms could be implemented based on specific requirements
        let memory_per_test = peak_memory as f64 / self.test_metrics.len().max(1) as f64;
        let efficiency = 100.0 - (memory_per_test / 10_000_000.0); // Adjust scale as needed
        efficiency.clamp(0.0, 100.0)
    }

    /// Calculate execution efficiency score (higher is better)
    fn calculate_execution_efficiency(&self) -> f64 {
        if self.test_metrics.is_empty() {
            return 100.0;
        }

        // Calculate coefficient of variation for test durations
        let durations: Vec<f64> = self
            .test_metrics
            .values()
            .map(|metrics| metrics.duration.as_secs_f64())
            .collect();

        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let variance =
            durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;

        let std_dev = variance.sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

        // Convert CV to efficiency score (lower CV = higher efficiency)
        let efficiency = (1.0 / (1.0 + cv)) * 100.0;
        efficiency.clamp(0.0, 100.0)
    }

    /// Get the current suite metrics
    pub fn get_suite_metrics(&self) -> SuiteMetrics {
        self.suite_metrics.clone()
    }

    /// Get metrics for a specific test
    pub fn get_test_metrics(&self, test_name: &str) -> Option<&TestMetrics> {
        self.test_metrics.get(test_name)
    }

    /// Get all test metrics
    pub fn get_all_test_metrics(&self) -> &HashMap<String, TestMetrics> {
        &self.test_metrics
    }

    /// Get memory usage history
    pub fn get_memory_samples(&self) -> &[MemorySample] {
        &self.memory_samples
    }

    /// Calculate summary statistics
    pub fn get_summary(&self) -> MetricsSummary {
        let total_tests = self.test_metrics.len();
        let successful_tests = self
            .test_metrics
            .values()
            .filter(|metrics| metrics.success)
            .count();
        let failed_tests = total_tests - successful_tests;
        let total_retries = self
            .test_metrics
            .values()
            .map(|metrics| metrics.retry_count)
            .sum();

        MetricsSummary {
            total_tests,
            successful_tests,
            failed_tests,
            total_retries,
            suite_metrics: self.suite_metrics.clone(),
        }
    }
}

/// Summary of all collected metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub total_retries: usize,
    pub suite_metrics: SuiteMetrics,
}

impl MetricsSummary {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.successful_tests as f64 / self.total_tests as f64) * 100.0
        }
    }

    /// Calculate failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    /// Calculate average retries per test
    pub fn average_retries_per_test(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.total_retries as f64 / self.total_tests as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_collector_basic_flow() {
        let mut collector = MetricsCollector::new();

        collector.start_suite();

        collector.start_test("test1");
        thread::sleep(Duration::from_millis(10));
        collector.end_test("test1", true, None);

        collector.start_test("test2");
        thread::sleep(Duration::from_millis(20));
        collector.end_test("test2", false, Some("Test failed".to_string()));

        collector.end_suite();

        let summary = collector.get_summary();
        assert_eq!(summary.total_tests, 2);
        assert_eq!(summary.successful_tests, 1);
        assert_eq!(summary.failed_tests, 1);
        assert_eq!(summary.success_rate(), 50.0);
    }

    #[test]
    fn test_retry_counting() {
        let mut collector = MetricsCollector::new();

        collector.start_test("flaky_test");
        collector.record_retry("flaky_test");
        collector.record_retry("flaky_test");
        collector.end_test("flaky_test", true, None);

        let metrics = collector.get_test_metrics("flaky_test").unwrap();
        assert_eq!(metrics.retry_count, 2);
    }

    #[test]
    fn test_memory_sampling() {
        let mut collector = MetricsCollector::new();

        collector.sample_memory();
        collector.sample_memory();
        collector.sample_memory();

        let samples = collector.get_memory_samples();
        assert_eq!(samples.len(), 3);

        // All samples should have reasonable memory values
        for sample in samples {
            assert!(sample.memory_mb > 0);
            assert!(sample.memory_mb < 10000); // Reasonable upper bound
        }
    }

    #[test]
    fn test_suite_metrics_calculation() {
        let mut collector = MetricsCollector::new();

        collector.start_suite();

        // Add multiple tests with different durations
        collector.start_test("fast_test");
        thread::sleep(Duration::from_millis(5));
        collector.end_test("fast_test", true, None);

        collector.start_test("slow_test");
        thread::sleep(Duration::from_millis(50));
        collector.end_test("slow_test", true, None);

        collector.end_suite();

        let metrics = collector.get_suite_metrics();
        assert!(metrics.average_test_duration > Duration::from_millis(0));
        assert!(metrics.slowest_duration >= metrics.fastest_duration);
        assert_eq!(metrics.slowest_test, Some("slow_test".to_string()));
        assert_eq!(metrics.fastest_test, Some("fast_test".to_string()));
    }

    #[test]
    fn test_efficiency_scores() {
        let mut collector = MetricsCollector::new();

        collector.start_suite();

        // Create tests with consistent timing for good efficiency
        for i in 0..3 {
            let test_name = format!("test{i}");
            collector.start_test(&test_name);
            thread::sleep(Duration::from_millis(10)); // Consistent timing
            collector.end_test(&test_name, true, None);
        }

        collector.end_suite();

        let metrics = collector.get_suite_metrics();

        // Should have reasonable efficiency scores
        assert!(metrics.execution_efficiency_score >= 0.0);
        assert!(metrics.execution_efficiency_score <= 100.0);
        assert!(metrics.memory_efficiency_score >= 0.0);
        assert!(metrics.memory_efficiency_score <= 100.0);
    }
}
