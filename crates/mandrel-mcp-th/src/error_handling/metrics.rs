//! Error metrics collection and analysis for the Mandrel MCP Test Harness
//!
//! This module provides comprehensive error metrics collection, analysis, and reporting
//! capabilities to track error patterns, recovery rates, and system reliability.

use crate::error_handling::{ErrorCategory, ErrorContext, ErrorSeverity, TestHarnessError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive error metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total number of errors recorded
    pub total_errors: u64,
    /// Errors grouped by category
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    /// Errors grouped by severity level
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    /// Errors grouped by test name
    pub errors_by_test: HashMap<String, u64>,
    /// Errors grouped by server name
    pub errors_by_server: HashMap<String, u64>,
    /// Retry attempt statistics
    pub retry_stats: RetryStatistics,
    /// Error recovery success rate (percentage)
    pub recovery_success_rate: f64,
    /// Average time to resolve errors (in seconds)
    pub average_resolution_time_seconds: f64,
    /// Error frequency over time
    pub error_frequency: ErrorFrequency,
    /// Performance impact statistics
    pub performance_impact: PerformanceImpact,
    /// Time range for these metrics
    pub time_range: TimeRange,
}

/// Retry attempt statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStatistics {
    /// Total number of retry attempts
    pub total_retries: u64,
    /// Number of successful recoveries after retry
    pub successful_recoveries: u64,
    /// Number of permanent failures after all retries
    pub permanent_failures: u64,
    /// Average number of retries per error
    pub average_retries_per_error: f64,
    /// Distribution of retry counts
    pub retry_distribution: HashMap<u32, u64>,
    /// Average retry delay in milliseconds
    pub average_retry_delay_ms: f64,
}

/// Error frequency analysis over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFrequency {
    /// Errors per hour
    pub errors_per_hour: f64,
    /// Peak error rate (errors per minute)
    pub peak_error_rate: f64,
    /// Time of peak error rate
    pub peak_error_time: Option<DateTime<Utc>>,
    /// Error trends (increasing, decreasing, stable)
    pub trend: ErrorTrend,
    /// Hourly error distribution
    pub hourly_distribution: Vec<u64>,
}

/// Error trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorTrend {
    Increasing { rate_per_hour: f64 },
    Decreasing { rate_per_hour: f64 },
    Stable { variance: f64 },
    InsufficientData,
}

/// Performance impact of errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    /// Average operation duration increase due to errors (in milliseconds)
    pub average_duration_increase_ms: f64,
    /// Memory overhead from error handling (in MB)
    pub memory_overhead_mb: f64,
    /// CPU time spent on error handling (in milliseconds)
    pub cpu_time_error_handling_ms: u64,
    /// Network bandwidth consumed by retries (in bytes)
    pub retry_bandwidth_bytes: u64,
    /// Test execution slowdown percentage
    pub test_slowdown_percentage: f64,
}

/// Time range for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: u64,
}

/// Individual error event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Unique identifier for this error event
    pub id: uuid::Uuid,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
    /// The error that occurred
    pub error: TestHarnessError,
    /// Context information
    pub context: ErrorContext,
    /// Whether recovery was attempted
    pub recovery_attempted: bool,
    /// Whether recovery was successful
    pub recovery_successful: bool,
    /// Time taken to resolve the error (if resolved)
    pub resolution_time: Option<Duration>,
    /// Number of retry attempts made
    pub retry_attempts: u32,
    /// Performance impact of this error
    pub performance_cost: Option<PerformanceCost>,
}

/// Performance cost of a single error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCost {
    /// Additional time spent due to this error
    pub additional_duration_ms: u64,
    /// Memory allocated for error handling
    pub memory_allocated_bytes: u64,
    /// CPU cycles consumed for error handling
    pub cpu_cycles: Option<u64>,
    /// Network I/O for retries
    pub network_io_bytes: u64,
}

/// Error collector for gathering and analyzing error events
#[derive(Debug)]
pub struct ErrorCollector {
    /// All error events recorded
    events: Vec<ErrorEvent>,
    /// Start time for metrics collection
    start_time: Instant,
    /// Maximum number of events to retain
    max_events: usize,
    /// Current metrics snapshot
    current_metrics: ErrorMetrics,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create a new error collector with specified capacity
    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events.min(1000)),
            start_time: Instant::now(),
            max_events,
            current_metrics: ErrorMetrics::new(),
        }
    }

    /// Record an error event
    pub fn record_error(&mut self, error: TestHarnessError, context: ErrorContext) -> uuid::Uuid {
        let event = ErrorEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            error,
            context,
            recovery_attempted: false,
            recovery_successful: false,
            resolution_time: None,
            retry_attempts: 0,
            performance_cost: None,
        };

        let event_id = event.id;
        self.add_event(event);
        event_id
    }

    /// Update an error event with recovery information
    pub fn update_recovery(
        &mut self,
        event_id: uuid::Uuid,
        successful: bool,
        resolution_time: Duration,
    ) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
            event.recovery_attempted = true;
            event.recovery_successful = successful;
            event.resolution_time = Some(resolution_time);
            self.update_metrics();
        }
    }

    /// Update an error event with retry information
    pub fn update_retry(&mut self, event_id: uuid::Uuid, retry_count: u32) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
            event.retry_attempts = retry_count;
            self.update_metrics();
        }
    }

    /// Update an error event with performance cost
    pub fn update_performance_cost(&mut self, event_id: uuid::Uuid, cost: PerformanceCost) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
            event.performance_cost = Some(cost);
            self.update_metrics();
        }
    }

    /// Add an error event to the collection
    fn add_event(&mut self, event: ErrorEvent) {
        // Maintain capacity limit
        if self.events.len() >= self.max_events {
            self.events.remove(0); // Remove oldest event
        }

        self.events.push(event);
        self.update_metrics();
    }

    /// Update current metrics based on collected events
    fn update_metrics(&mut self) {
        self.current_metrics = self.calculate_metrics();
    }

    /// Calculate comprehensive metrics from collected events
    pub fn calculate_metrics(&self) -> ErrorMetrics {
        if self.events.is_empty() {
            return ErrorMetrics::new();
        }

        let total_errors = self.events.len() as u64;
        let mut errors_by_category = HashMap::new();
        let mut errors_by_severity = HashMap::new();
        let mut errors_by_test = HashMap::new();
        let mut errors_by_server = HashMap::new();

        #[allow(unused_variables)]
        let mut total_retries = 0u64;
        let mut successful_recoveries = 0u64;
        let mut total_resolution_time = Duration::ZERO;
        let mut resolved_errors = 0u64;

        for event in &self.events {
            // Category counting
            let category = event.error.category();
            *errors_by_category.entry(category).or_insert(0) += 1;

            // Severity counting
            let severity = event.error.severity();
            *errors_by_severity.entry(severity).or_insert(0) += 1;

            // Test name counting
            if let Some(ref test_ctx) = event.context.test_context {
                *errors_by_test
                    .entry(test_ctx.test_name.clone())
                    .or_insert(0) += 1;
            }

            // Server name counting
            if let Some(ref server_ctx) = event.context.server_context {
                *errors_by_server
                    .entry(server_ctx.server_name.clone())
                    .or_insert(0) += 1;
            }

            // Retry statistics
            total_retries += event.retry_attempts as u64;
            if event.recovery_successful {
                successful_recoveries += 1;
            }

            // Resolution time
            if let Some(resolution_time) = event.resolution_time {
                total_resolution_time += resolution_time;
                resolved_errors += 1;
            }
        }

        let recovery_success_rate = if total_errors > 0 {
            (successful_recoveries as f64 / total_errors as f64) * 100.0
        } else {
            0.0
        };

        let average_resolution_time_seconds = if resolved_errors > 0 {
            total_resolution_time.as_secs_f64() / resolved_errors as f64
        } else {
            0.0
        };

        let retry_stats = self.calculate_retry_statistics();
        let error_frequency = self.calculate_error_frequency();
        let performance_impact = self.calculate_performance_impact();

        ErrorMetrics {
            total_errors,
            errors_by_category,
            errors_by_severity,
            errors_by_test,
            errors_by_server,
            retry_stats,
            recovery_success_rate,
            average_resolution_time_seconds,
            error_frequency,
            performance_impact,
            time_range: self.get_time_range(),
        }
    }

    /// Calculate retry statistics
    fn calculate_retry_statistics(&self) -> RetryStatistics {
        if self.events.is_empty() {
            return RetryStatistics::default();
        }

        let total_retries: u64 = self.events.iter().map(|e| e.retry_attempts as u64).sum();
        let successful_recoveries =
            self.events.iter().filter(|e| e.recovery_successful).count() as u64;
        let permanent_failures = self
            .events
            .iter()
            .filter(|e| e.recovery_attempted && !e.recovery_successful)
            .count() as u64;

        let average_retries_per_error = if !self.events.is_empty() {
            total_retries as f64 / self.events.len() as f64
        } else {
            0.0
        };

        let mut retry_distribution = HashMap::new();
        for event in &self.events {
            *retry_distribution.entry(event.retry_attempts).or_insert(0) += 1;
        }

        RetryStatistics {
            total_retries,
            successful_recoveries,
            permanent_failures,
            average_retries_per_error,
            retry_distribution,
            average_retry_delay_ms: 0.0, // Would need actual retry delay tracking
        }
    }

    /// Calculate error frequency over time
    fn calculate_error_frequency(&self) -> ErrorFrequency {
        if self.events.is_empty() {
            return ErrorFrequency::default();
        }

        let time_range = self.get_time_range();
        let duration_hours = time_range.duration_seconds as f64 / 3600.0;

        let errors_per_hour = if duration_hours > 0.0 {
            self.events.len() as f64 / duration_hours
        } else {
            0.0
        };

        // Calculate hourly distribution (simplified)
        let hourly_distribution = vec![0; 24]; // Would need actual hourly breakdown

        ErrorFrequency {
            errors_per_hour,
            peak_error_rate: 0.0, // Would need time window analysis
            peak_error_time: None,
            trend: ErrorTrend::InsufficientData,
            hourly_distribution,
        }
    }

    /// Calculate performance impact
    fn calculate_performance_impact(&self) -> PerformanceImpact {
        let mut total_duration_increase = 0u64;
        let mut total_memory_overhead = 0u64;
        let mut total_cpu_time = 0u64;
        let mut total_retry_bandwidth = 0u64;

        for event in &self.events {
            if let Some(ref cost) = event.performance_cost {
                total_duration_increase += cost.additional_duration_ms;
                total_memory_overhead += cost.memory_allocated_bytes;
                total_cpu_time += cost.cpu_cycles.unwrap_or(0);
                total_retry_bandwidth += cost.network_io_bytes;
            }
        }

        let event_count = self.events.len() as f64;

        PerformanceImpact {
            average_duration_increase_ms: if event_count > 0.0 {
                total_duration_increase as f64 / event_count
            } else {
                0.0
            },
            memory_overhead_mb: total_memory_overhead as f64 / (1024.0 * 1024.0),
            cpu_time_error_handling_ms: total_cpu_time,
            retry_bandwidth_bytes: total_retry_bandwidth,
            test_slowdown_percentage: 0.0, // Would need baseline measurements
        }
    }

    /// Get the time range for current metrics
    fn get_time_range(&self) -> TimeRange {
        let now = Utc::now();
        let start = if let Some(first_event) = self.events.first() {
            first_event.timestamp
        } else {
            now
        };

        TimeRange {
            start,
            end: now,
            duration_seconds: (now - start).num_seconds().max(0) as u64,
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> &ErrorMetrics {
        &self.current_metrics
    }

    /// Generate error summary report
    pub fn generate_summary(&self) -> ErrorSummary {
        let metrics = &self.current_metrics;

        ErrorSummary {
            total_errors: metrics.total_errors,
            error_rate_per_hour: metrics.error_frequency.errors_per_hour,
            recovery_success_rate: metrics.recovery_success_rate,
            most_common_category: self.get_most_common_category(),
            most_problematic_test: self.get_most_problematic_test(),
            most_problematic_server: self.get_most_problematic_server(),
            recommendations: self.generate_recommendations(),
            time_range: metrics.time_range.clone(),
        }
    }

    /// Get the most common error category
    fn get_most_common_category(&self) -> Option<ErrorCategory> {
        self.current_metrics
            .errors_by_category
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category.clone())
    }

    /// Get the test with the most errors
    fn get_most_problematic_test(&self) -> Option<String> {
        self.current_metrics
            .errors_by_test
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(test_name, _)| test_name.clone())
    }

    /// Get the server with the most errors
    fn get_most_problematic_server(&self) -> Option<String> {
        self.current_metrics
            .errors_by_server
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(server_name, _)| server_name.clone())
    }

    /// Generate actionable recommendations based on error patterns
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let metrics = &self.current_metrics;

        // High error rate recommendation
        if metrics.error_frequency.errors_per_hour > 10.0 {
            recommendations.push(
                "High error rate detected. Consider reviewing test configurations and server stability.".to_string()
            );
        }

        // Low recovery rate recommendation
        if metrics.recovery_success_rate < 50.0 && metrics.total_errors > 5 {
            recommendations.push(
                "Low recovery success rate. Review retry configurations and error handling logic."
                    .to_string(),
            );
        }

        // Specific category recommendations
        if let Some(category) = self.get_most_common_category() {
            match category {
                ErrorCategory::Connection => {
                    recommendations.push(
                        "Many connection errors detected. Check network stability and server availability.".to_string()
                    );
                }
                ErrorCategory::Validation => {
                    recommendations.push(
                        "Many validation errors detected. Review test data and server response formats.".to_string()
                    );
                }
                _ => {}
            }
        }

        recommendations
    }

    /// Clear all collected events
    pub fn clear(&mut self) {
        self.events.clear();
        self.start_time = Instant::now();
        self.current_metrics = ErrorMetrics::new();
    }

    /// Get the number of events collected
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Error summary report for high-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub total_errors: u64,
    pub error_rate_per_hour: f64,
    pub recovery_success_rate: f64,
    pub most_common_category: Option<ErrorCategory>,
    pub most_problematic_test: Option<String>,
    pub most_problematic_server: Option<String>,
    pub recommendations: Vec<String>,
    pub time_range: TimeRange,
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            errors_by_severity: HashMap::new(),
            errors_by_test: HashMap::new(),
            errors_by_server: HashMap::new(),
            retry_stats: RetryStatistics::default(),
            recovery_success_rate: 0.0,
            average_resolution_time_seconds: 0.0,
            error_frequency: ErrorFrequency::default(),
            performance_impact: PerformanceImpact::default(),
            time_range: TimeRange {
                start: Utc::now(),
                end: Utc::now(),
                duration_seconds: 0,
            },
        }
    }
}

impl Default for RetryStatistics {
    fn default() -> Self {
        Self {
            total_retries: 0,
            successful_recoveries: 0,
            permanent_failures: 0,
            average_retries_per_error: 0.0,
            retry_distribution: HashMap::new(),
            average_retry_delay_ms: 0.0,
        }
    }
}

impl Default for ErrorFrequency {
    fn default() -> Self {
        Self {
            errors_per_hour: 0.0,
            peak_error_rate: 0.0,
            peak_error_time: None,
            trend: ErrorTrend::InsufficientData,
            hourly_distribution: vec![0; 24],
        }
    }
}

impl Default for PerformanceImpact {
    fn default() -> Self {
        Self {
            average_duration_increase_ms: 0.0,
            memory_overhead_mb: 0.0,
            cpu_time_error_handling_ms: 0,
            retry_bandwidth_bytes: 0,
            test_slowdown_percentage: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_handling::{McpClientError, TestExecutionError};

    #[test]
    fn test_error_collector_creation() {
        let collector = ErrorCollector::new();
        assert_eq!(collector.event_count(), 0);
        assert_eq!(collector.get_metrics().total_errors, 0);
    }

    #[test]
    fn test_error_recording() {
        let mut collector = ErrorCollector::new();

        let error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        let context = ErrorContext::for_server(
            "test-server".to_string(),
            "stdio://test".to_string(),
            "connect".to_string(),
        );

        let event_id = collector.record_error(error, context);

        assert_eq!(collector.event_count(), 1);
        assert_eq!(collector.get_metrics().total_errors, 1);
        assert!(!event_id.is_nil());
    }

    #[test]
    fn test_recovery_tracking() {
        let mut collector = ErrorCollector::new();

        let error = TestHarnessError::Execution(TestExecutionError::TestTimeout {
            test_name: "test_example".to_string(),
            timeout_seconds: 30,
            elapsed_seconds: 35,
            phase: "execution".to_string(),
            partial_results: None,
        });

        let context = ErrorContext::for_test("test_example".to_string(), "execute".to_string());
        let event_id = collector.record_error(error, context);

        // Update with successful recovery
        collector.update_recovery(event_id, true, Duration::from_secs(5));

        let metrics = collector.get_metrics();
        assert_eq!(metrics.recovery_success_rate, 100.0);
    }

    #[test]
    fn test_retry_statistics() {
        let mut collector = ErrorCollector::new();

        let error = TestHarnessError::Client(McpClientError::RequestTimeout {
            method: "tools/list".to_string(),
            duration_ms: 5000,
            timeout_ms: 3000,
            request_id: Some("123".to_string()),
            partial_response: None,
        });

        let context = ErrorContext::new("tools/list".to_string());
        let event_id = collector.record_error(error, context);

        // Update with retry information
        collector.update_retry(event_id, 3);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.retry_stats.total_retries, 3);
        assert_eq!(metrics.retry_stats.average_retries_per_error, 3.0);
    }

    #[test]
    fn test_error_categorization() {
        let mut collector = ErrorCollector::new();

        // Add a connection error
        let connection_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "server1".to_string(),
            message: "Failed".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        let context1 = ErrorContext::new("connect".to_string());
        collector.record_error(connection_error, context1);

        // Add an execution error
        let execution_error = TestHarnessError::Execution(TestExecutionError::AssertionFailed {
            test_name: "test1".to_string(),
            step: 1,
            message: "Assertion failed".to_string(),
            expected: None,
            actual: None,
            assertion_type: "equals".to_string(),
            context: None,
        });

        let context2 = ErrorContext::for_test("test1".to_string(), "assert".to_string());
        collector.record_error(execution_error, context2);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_errors, 2);
        assert_eq!(
            *metrics
                .errors_by_category
                .get(&ErrorCategory::Connection)
                .unwrap(),
            1
        );
        assert_eq!(
            *metrics
                .errors_by_category
                .get(&ErrorCategory::Execution)
                .unwrap(),
            1
        );
        assert_eq!(*metrics.errors_by_test.get("test1").unwrap(), 1);
    }

    #[test]
    fn test_error_summary_generation() {
        let mut collector = ErrorCollector::new();

        let error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "problematic-server".to_string(),
            message: "Connection failed".to_string(),
            retry_count: 2,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        let context = ErrorContext::for_server(
            "problematic-server".to_string(),
            "stdio://test".to_string(),
            "connect".to_string(),
        );

        collector.record_error(error, context);

        let summary = collector.generate_summary();
        assert_eq!(summary.total_errors, 1);
        assert_eq!(
            summary.most_common_category,
            Some(ErrorCategory::Connection)
        );
        assert_eq!(
            summary.most_problematic_server,
            Some("problematic-server".to_string())
        );
        assert!(!summary.recommendations.is_empty());
    }

    #[test]
    fn test_performance_cost_tracking() {
        let mut collector = ErrorCollector::new();

        let error = TestHarnessError::Performance(
            crate::error_handling::PerformanceError::OperationTimeout {
                operation: "test_operation".to_string(),
                limit_ms: 1000,
                actual_ms: 2000,
                resource_contention: false,
            },
        );

        let context = ErrorContext::new("timeout_test".to_string());
        let event_id = collector.record_error(error, context);

        // Add performance cost information
        let cost = PerformanceCost {
            additional_duration_ms: 1000,
            memory_allocated_bytes: 1024 * 1024, // 1MB
            cpu_cycles: Some(1000000),
            network_io_bytes: 512,
        };

        collector.update_performance_cost(event_id, cost);

        let metrics = collector.get_metrics();
        assert!(metrics.performance_impact.average_duration_increase_ms > 0.0);
        assert!(metrics.performance_impact.memory_overhead_mb > 0.0);
    }
}
