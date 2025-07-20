//! Error metrics collection and analysis for MOTH test harness
//!
//! This module implements comprehensive error metrics collection, analysis, and reporting
//! as specified in the design document.

use crate::error_handling::errors::{ErrorContext, TestHarnessError};
use crate::error_handling::logging::ErrorEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive error metrics for monitoring and analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_test: HashMap<String, u64>,
    pub errors_by_server: HashMap<String, u64>,
    pub retry_counts: HashMap<String, u64>,
    pub recovery_success_rate: f64,
    pub average_error_resolution_time: Duration,
    pub error_rate_per_hour: f64,
    pub peak_error_time: Option<chrono::DateTime<chrono::Utc>>,
    pub most_frequent_error: Option<String>,
}

/// Error trend analysis over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrends {
    pub hourly_error_counts: Vec<HourlyErrorCount>,
    pub daily_error_counts: Vec<DailyErrorCount>,
    pub error_patterns: Vec<ErrorPattern>,
    pub trending_errors: Vec<TrendingError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyErrorCount {
    pub hour: chrono::DateTime<chrono::Utc>,
    pub count: u64,
    pub categories: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyErrorCount {
    pub date: chrono::NaiveDate,
    pub count: u64,
    pub categories: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_id: String,
    pub description: String,
    pub frequency: u64,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub affected_tests: Vec<String>,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingError {
    pub error_type: String,
    pub current_count: u64,
    pub previous_count: u64,
    pub trend_direction: TrendDirection,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Error analysis engine for comprehensive error insights
pub struct ErrorAnalyzer {
    metrics: ErrorMetrics,
    events: Vec<ErrorEvent>,
    analysis_window: Duration,
    start_time: Instant,
}

impl ErrorAnalyzer {
    /// Create a new error analyzer
    pub fn new(analysis_window: Duration) -> Self {
        Self {
            metrics: ErrorMetrics::default(),
            events: Vec::new(),
            analysis_window,
            start_time: Instant::now(),
        }
    }

    /// Record an error event for analysis
    pub fn record_error(&mut self, error: &TestHarnessError, context: ErrorContext) {
        let event = ErrorEvent {
            timestamp: chrono::Utc::now(),
            error: error.clone(),
            context: context.clone(),
            recovery_attempted: false,
            recovery_successful: false,
        };

        self.events.push(event);
        self.update_metrics(error, &context);
    }

    /// Record a recovery attempt
    pub fn record_recovery_attempt(&mut self, _error_id: Option<String>, successful: bool) {
        if let Some(event) = self.events.last_mut() {
            event.recovery_attempted = true;
            event.recovery_successful = successful;
        }

        self.update_recovery_metrics(successful);
    }

    /// Update metrics based on a new error
    fn update_metrics(&mut self, error: &TestHarnessError, context: &ErrorContext) {
        self.metrics.total_errors += 1;

        // Update category metrics
        let category = self.categorize_error(error);
        *self.metrics.errors_by_category.entry(category).or_insert(0) += 1;

        // Update test metrics
        if let Some(test_name) = &context.test_name {
            *self
                .metrics
                .errors_by_test
                .entry(test_name.clone())
                .or_insert(0) += 1;
        }

        // Update server metrics
        if let Some(server_name) = &context.server_name {
            *self
                .metrics
                .errors_by_server
                .entry(server_name.clone())
                .or_insert(0) += 1;
        }

        // Update error rate
        let elapsed = self.start_time.elapsed();
        self.metrics.error_rate_per_hour =
            (self.metrics.total_errors as f64) / elapsed.as_secs_f64() * 3600.0;

        // Update most frequent error
        self.metrics.most_frequent_error = self.find_most_frequent_error();
    }

    /// Update recovery success rate
    fn update_recovery_metrics(&mut self, _successful: bool) {
        let recovery_attempts = self.events.iter().filter(|e| e.recovery_attempted).count();

        let successful_recoveries = self
            .events
            .iter()
            .filter(|e| e.recovery_attempted && e.recovery_successful)
            .count();

        if recovery_attempts > 0 {
            self.metrics.recovery_success_rate =
                successful_recoveries as f64 / recovery_attempts as f64;
        }
    }

    /// Categorize an error for metrics
    fn categorize_error(&self, error: &TestHarnessError) -> String {
        match error {
            TestHarnessError::Client(_) => "client".to_string(),
            TestHarnessError::Execution(_) => "execution".to_string(),
            TestHarnessError::Validation(_) => "validation".to_string(),
            TestHarnessError::Configuration(_) => "configuration".to_string(),
            TestHarnessError::Io(_) => "io".to_string(),
            TestHarnessError::Reporting(_) => "reporting".to_string(),
            TestHarnessError::Network(_) => "network".to_string(),
            TestHarnessError::Performance(_) => "performance".to_string(),
            TestHarnessError::Security(_) => "security".to_string(),
        }
    }

    /// Find the most frequent error category
    fn find_most_frequent_error(&self) -> Option<String> {
        self.metrics
            .errors_by_category
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category.clone())
    }

    /// Get current error metrics
    pub fn get_metrics(&self) -> &ErrorMetrics {
        &self.metrics
    }

    /// Generate error trends analysis
    pub fn analyze_trends(&self) -> ErrorTrends {
        let hourly_counts = self.calculate_hourly_error_counts();
        let daily_counts = self.calculate_daily_error_counts();
        let patterns = self.identify_error_patterns();
        let trending = self.calculate_trending_errors();

        ErrorTrends {
            hourly_error_counts: hourly_counts,
            daily_error_counts: daily_counts,
            error_patterns: patterns,
            trending_errors: trending,
        }
    }

    /// Calculate hourly error counts
    fn calculate_hourly_error_counts(&self) -> Vec<HourlyErrorCount> {
        let mut hourly_map: HashMap<i64, (u64, HashMap<String, u64>)> = HashMap::new();

        for event in &self.events {
            let hour_timestamp = event.timestamp.timestamp() / 3600 * 3600;
            let category = self.categorize_error(&event.error);

            let (count, categories) = hourly_map
                .entry(hour_timestamp)
                .or_insert((0, HashMap::new()));
            *count += 1;
            *categories.entry(category).or_insert(0) += 1;
        }

        let mut hourly_counts: Vec<_> = hourly_map
            .into_iter()
            .map(|(timestamp, (count, categories))| HourlyErrorCount {
                hour: chrono::DateTime::from_timestamp(timestamp, 0)
                    .unwrap_or_else(chrono::Utc::now),
                count,
                categories,
            })
            .collect();

        hourly_counts.sort_by_key(|h| h.hour);
        hourly_counts
    }

    /// Calculate daily error counts
    fn calculate_daily_error_counts(&self) -> Vec<DailyErrorCount> {
        let mut daily_map: HashMap<chrono::NaiveDate, (u64, HashMap<String, u64>)> = HashMap::new();

        for event in &self.events {
            let date = event.timestamp.date_naive();
            let category = self.categorize_error(&event.error);

            let (count, categories) = daily_map.entry(date).or_insert((0, HashMap::new()));
            *count += 1;
            *categories.entry(category).or_insert(0) += 1;
        }

        let mut daily_counts: Vec<_> = daily_map
            .into_iter()
            .map(|(date, (count, categories))| DailyErrorCount {
                date,
                count,
                categories,
            })
            .collect();

        daily_counts.sort_by_key(|d| d.date);
        daily_counts
    }

    /// Identify error patterns in the data
    fn identify_error_patterns(&self) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();
        let mut pattern_map: HashMap<String, Vec<&ErrorEvent>> = HashMap::new();

        // Group errors by similar characteristics
        for event in &self.events {
            let pattern_key = self.generate_pattern_key(&event.error);
            pattern_map.entry(pattern_key).or_default().push(event);
        }

        // Create patterns for groups with sufficient frequency
        for (_pattern_key, events) in pattern_map {
            if events.len() >= 2 {
                // Minimum frequency threshold
                let first_event = events.first().unwrap();
                let last_event = events.last().unwrap();

                let affected_tests: Vec<String> = events
                    .iter()
                    .filter_map(|e| e.context.test_name.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                patterns.push(ErrorPattern {
                    pattern_id: format!("pattern_{}", uuid::Uuid::new_v4()),
                    description: self.generate_pattern_description(&first_event.error),
                    frequency: events.len() as u64,
                    first_seen: first_event.timestamp,
                    last_seen: last_event.timestamp,
                    affected_tests,
                    suggested_fix: self.suggest_fix(&first_event.error),
                });
            }
        }

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns
    }

    /// Generate a pattern key for grouping similar errors
    fn generate_pattern_key(&self, error: &TestHarnessError) -> String {
        match error {
            TestHarnessError::Client(client_error) => match client_error {
                crate::error_handling::errors::McpClientError::ConnectionFailed { .. } => {
                    "client_connection_failed".to_string()
                }
                crate::error_handling::errors::McpClientError::ProtocolViolation { .. } => {
                    "client_protocol_violation".to_string()
                }
                crate::error_handling::errors::McpClientError::RequestTimeout { .. } => {
                    "client_request_timeout".to_string()
                }
                crate::error_handling::errors::McpClientError::ServerError { .. } => {
                    "client_server_error".to_string()
                }
                crate::error_handling::errors::McpClientError::TransportError { .. } => {
                    "client_transport_error".to_string()
                }
                crate::error_handling::errors::McpClientError::AuthenticationError { .. } => {
                    "client_authentication_error".to_string()
                }
            },
            TestHarnessError::Network(network_error) => match network_error {
                crate::error_handling::errors::NetworkError::ConnectionTimeout { .. } => {
                    "network_connection_timeout".to_string()
                }
                crate::error_handling::errors::NetworkError::DnsResolutionFailed { .. } => {
                    "network_dns_resolution_failed".to_string()
                }
            },
            TestHarnessError::Validation(validation_error) => match validation_error {
                crate::error_handling::errors::ValidationError::SchemaValidation { .. } => {
                    "validation_schema_validation".to_string()
                }
                crate::error_handling::errors::ValidationError::JsonPathValidation { .. } => {
                    "validation_jsonpath_validation".to_string()
                }
                crate::error_handling::errors::ValidationError::ResponseFormat { .. } => {
                    "validation_response_format".to_string()
                }
            },
            TestHarnessError::Execution(_) => "execution".to_string(),
            TestHarnessError::Configuration(_) => "configuration".to_string(),
            TestHarnessError::Io(_) => "io".to_string(),
            TestHarnessError::Reporting(_) => "reporting".to_string(),
            TestHarnessError::Performance(_) => "performance".to_string(),
            TestHarnessError::Security(_) => "security".to_string(),
        }
    }

    /// Generate a human-readable pattern description
    fn generate_pattern_description(&self, error: &TestHarnessError) -> String {
        match error {
            TestHarnessError::Client(_) => {
                "MCP client connection or communication issues".to_string()
            }
            TestHarnessError::Network(_) => "Network connectivity or timeout problems".to_string(),
            TestHarnessError::Validation(_) => {
                "Response validation or schema compliance failures".to_string()
            }
            TestHarnessError::Execution(_) => {
                "Test execution failures or assertion errors".to_string()
            }
            TestHarnessError::Configuration(_) => "Configuration or setup issues".to_string(),
            TestHarnessError::Io(_) => "File system or I/O operation failures".to_string(),
            TestHarnessError::Reporting(_) => "Report generation or output issues".to_string(),
            TestHarnessError::Performance(_) => {
                "Performance threshold violations or timeouts".to_string()
            }
            TestHarnessError::Security(_) => {
                "Security policy violations or access issues".to_string()
            }
        }
    }

    /// Suggest a fix for common error patterns
    fn suggest_fix(&self, error: &TestHarnessError) -> Option<String> {
        match error {
            TestHarnessError::Client(_) => {
                Some("Check MCP server connectivity and configuration".to_string())
            }
            TestHarnessError::Network(_) => {
                Some("Verify network connectivity and increase timeout values".to_string())
            }
            TestHarnessError::Validation(_) => {
                Some("Review response schema and validation rules".to_string())
            }
            TestHarnessError::Execution(_) => {
                Some("Check test logic and expected outcomes".to_string())
            }
            TestHarnessError::Configuration(_) => {
                Some("Verify configuration file format and required fields".to_string())
            }
            TestHarnessError::Performance(_) => {
                Some("Optimize operations or increase performance thresholds".to_string())
            }
            _ => None,
        }
    }

    /// Calculate trending errors
    fn calculate_trending_errors(&self) -> Vec<TrendingError> {
        // This is a simplified implementation
        // In practice, you'd want more sophisticated trend analysis
        let mut trending = Vec::new();

        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let two_hours_ago = now - chrono::Duration::hours(2);

        let mut current_counts: HashMap<String, u64> = HashMap::new();
        let mut previous_counts: HashMap<String, u64> = HashMap::new();

        // Count errors in the last hour
        for event in &self.events {
            if event.timestamp >= one_hour_ago {
                let category = self.categorize_error(&event.error);
                *current_counts.entry(category).or_insert(0) += 1;
            } else if event.timestamp >= two_hours_ago {
                let category = self.categorize_error(&event.error);
                *previous_counts.entry(category).or_insert(0) += 1;
            }
        }

        // Calculate trends
        for (category, current) in current_counts {
            let previous = previous_counts.get(&category).unwrap_or(&0);
            let trend_direction = if current > *previous {
                TrendDirection::Increasing
            } else if current < *previous {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            };

            let confidence = if *previous > 0 {
                1.0 - ((*previous as f64 - current as f64).abs() / *previous as f64)
            } else if current > 0 {
                1.0
            } else {
                0.0
            };

            trending.push(TrendingError {
                error_type: category,
                current_count: current,
                previous_count: *previous,
                trend_direction,
                confidence,
            });
        }

        trending.sort_by(|a, b| {
            b.current_count.cmp(&a.current_count).then_with(|| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        trending
    }

    /// Generate a comprehensive error report
    pub fn generate_report(&self) -> ErrorAnalysisReport {
        let trends = self.analyze_trends();

        ErrorAnalysisReport {
            summary: self.metrics.clone(),
            trends,
            recommendations: self.generate_recommendations(),
            analysis_period: self.start_time.elapsed(),
            report_generated_at: chrono::Utc::now(),
        }
    }

    /// Generate actionable recommendations based on error analysis
    fn generate_recommendations(&self) -> Vec<ErrorRecommendation> {
        let mut recommendations = Vec::new();

        // Only generate recommendations if there are errors
        if self.metrics.total_errors == 0 {
            return recommendations;
        }

        // High error rate recommendation
        if self.metrics.error_rate_per_hour > 10.0 {
            recommendations.push(ErrorRecommendation {
                priority: RecommendationPriority::High,
                category: "error_rate".to_string(),
                title: "High Error Rate Detected".to_string(),
                description: format!(
                    "Error rate of {:.1} errors/hour exceeds recommended threshold",
                    self.metrics.error_rate_per_hour
                ),
                action: "Investigate root causes and implement error prevention measures"
                    .to_string(),
                estimated_impact: "Reduce error rate by 50-70%".to_string(),
            });
        }

        // Low recovery rate recommendation
        if self.metrics.recovery_success_rate < 0.8 {
            recommendations.push(ErrorRecommendation {
                priority: RecommendationPriority::Medium,
                category: "recovery".to_string(),
                title: "Low Error Recovery Rate".to_string(),
                description: format!(
                    "Recovery success rate of {:.1}% is below optimal threshold",
                    self.metrics.recovery_success_rate * 100.0
                ),
                action: "Review and improve retry logic and error handling strategies".to_string(),
                estimated_impact: "Improve system reliability by 20-30%".to_string(),
            });
        }

        // Test-specific recommendations
        if let Some(problematic_test) = self.find_most_problematic_test() {
            recommendations.push(ErrorRecommendation {
                priority: RecommendationPriority::Medium,
                category: "test_quality".to_string(),
                title: format!("Test '{}' Has High Error Rate", problematic_test.0),
                description: format!(
                    "Test '{}' accounts for {} errors ({}% of total)",
                    problematic_test.0,
                    problematic_test.1,
                    (problematic_test.1 as f64 / self.metrics.total_errors as f64 * 100.0) as u32
                ),
                action: "Review test implementation and expected behaviors".to_string(),
                estimated_impact: "Reduce test-specific failures by 40-60%".to_string(),
            });
        }

        recommendations
    }

    /// Find the test with the most errors
    fn find_most_problematic_test(&self) -> Option<(String, u64)> {
        self.metrics
            .errors_by_test
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(test, count)| (test.clone(), *count))
    }
}

/// Comprehensive error analysis report
#[derive(Debug, Clone, Serialize)]
pub struct ErrorAnalysisReport {
    pub summary: ErrorMetrics,
    pub trends: ErrorTrends,
    pub recommendations: Vec<ErrorRecommendation>,
    pub analysis_period: Duration,
    pub report_generated_at: chrono::DateTime<chrono::Utc>,
}

/// Error improvement recommendation
#[derive(Debug, Clone, Serialize)]
pub struct ErrorRecommendation {
    pub priority: RecommendationPriority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub action: String,
    pub estimated_impact: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum RecommendationPriority {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_handling::errors::*;

    #[test]
    fn test_error_analyzer_creation() {
        let analyzer = ErrorAnalyzer::new(Duration::from_secs(3600)); // 1 hour
        assert_eq!(analyzer.metrics.total_errors, 0);
        assert!(analyzer.events.is_empty());
    }

    #[test]
    fn test_error_recording() {
        let mut analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        let error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection failed".to_string(),
            retry_count: 1,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });

        let context = ErrorContext::new("test_operation")
            .with_test("sample_test")
            .with_server("test_server");

        analyzer.record_error(&error, context);

        assert_eq!(analyzer.metrics.total_errors, 1);
        assert_eq!(analyzer.events.len(), 1);
        assert_eq!(analyzer.metrics.errors_by_category.get("client"), Some(&1));
        assert_eq!(analyzer.metrics.errors_by_test.get("sample_test"), Some(&1));
        assert_eq!(
            analyzer.metrics.errors_by_server.get("test_server"),
            Some(&1)
        );
    }

    #[test]
    fn test_recovery_metrics() {
        let mut analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        let error = TestHarnessError::Network(NetworkError::ConnectionTimeout {
            endpoint: "test-endpoint".to_string(),
            timeout_ms: 5000,
        });

        let context = ErrorContext::new("test_operation");

        analyzer.record_error(&error, context);
        analyzer.record_recovery_attempt(None, true);

        assert_eq!(analyzer.metrics.recovery_success_rate, 1.0);
    }

    #[test]
    fn test_error_categorization() {
        let analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        let client_error = TestHarnessError::Client(McpClientError::RequestTimeout {
            method: "test_method".to_string(),
            duration_ms: 5000,
            timeout_ms: 3000,
            request_id: None,
        });

        assert_eq!(analyzer.categorize_error(&client_error), "client");

        let validation_error = TestHarnessError::Validation(ValidationError::SchemaValidation {
            path: "$.test".to_string(),
            message: "Invalid schema".to_string(),
            expected_schema: None,
            actual_value: None,
        });

        assert_eq!(analyzer.categorize_error(&validation_error), "validation");
    }

    #[test]
    fn test_most_frequent_error() {
        let mut analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        // Add multiple network errors
        for _ in 0..3 {
            let error = TestHarnessError::Network(NetworkError::ConnectionTimeout {
                endpoint: "test".to_string(),
                timeout_ms: 1000,
            });
            analyzer.record_error(&error, ErrorContext::new("test"));
        }

        // Add one client error
        let error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "failed".to_string(),
            retry_count: 0,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });
        analyzer.record_error(&error, ErrorContext::new("test"));

        assert_eq!(
            analyzer.metrics.most_frequent_error,
            Some("network".to_string())
        );
    }

    #[test]
    fn test_pattern_generation() {
        let analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        let error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "failed".to_string(),
            retry_count: 0,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });

        let pattern_key = analyzer.generate_pattern_key(&error);
        assert!(pattern_key.starts_with("client_"));

        let description = analyzer.generate_pattern_description(&error);
        assert!(description.contains("MCP client"));
    }

    #[test]
    fn test_trending_calculation() {
        let mut analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        // Add some errors to create trends
        let error = TestHarnessError::Network(NetworkError::ConnectionTimeout {
            endpoint: "test".to_string(),
            timeout_ms: 1000,
        });

        analyzer.record_error(&error, ErrorContext::new("test"));

        let trending = analyzer.calculate_trending_errors();
        assert!(!trending.is_empty());
    }

    #[test]
    fn test_recommendation_generation() {
        let mut analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));

        // Create high error rate scenario
        for i in 0..20 {
            let error = TestHarnessError::Network(NetworkError::ConnectionTimeout {
                endpoint: format!("test-{}", i),
                timeout_ms: 1000,
            });
            analyzer.record_error(&error, ErrorContext::new("test"));
        }

        // Simulate high error rate
        analyzer.start_time = Instant::now() - Duration::from_secs(60); // 1 minute ago
        analyzer.update_metrics(
            &TestHarnessError::Network(NetworkError::ConnectionTimeout {
                endpoint: "dummy".to_string(),
                timeout_ms: 1000,
            }),
            &ErrorContext::new("dummy"),
        );

        let recommendations = analyzer.generate_recommendations();
        assert!(!recommendations.is_empty());

        // Should have high priority recommendation for error rate
        assert!(recommendations
            .iter()
            .any(|r| matches!(r.priority, RecommendationPriority::High)));
    }

    #[test]
    fn test_report_generation() {
        let analyzer = ErrorAnalyzer::new(Duration::from_secs(3600));
        let report = analyzer.generate_report();

        assert_eq!(report.summary.total_errors, 0);
        assert!(report.trends.hourly_error_counts.is_empty());
        // With zero errors, there should be no recommendations
        assert!(report.recommendations.is_empty());
        assert!(report.analysis_period >= Duration::from_nanos(0));
    }
}
