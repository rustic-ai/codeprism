//! Failure diagnostics and context analysis

use crate::types::{TestResult, TestSuiteResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Failure diagnostics analyzer
pub struct FailureDiagnostics;

impl FailureDiagnostics {
    pub fn new() -> Self {
        Self
    }

    /// Analyze failures across test results
    pub async fn analyze_failures(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Result<super::FailureAnalysis> {
        let mut failure_details = Vec::new();
        let mut failure_categories = HashMap::new();
        let mut total_failures = 0;

        for suite in test_results {
            for test in &suite.test_results {
                if !test.success {
                    total_failures += 1;

                    let category = self.categorize_failure(test);
                    *failure_categories
                        .entry(format!("{:?}", category))
                        .or_insert(0) += 1;

                    let failure_detail = super::FailureDetail {
                        test_id: test.test_case.id.clone(),
                        tool_name: test.test_case.tool_name.clone(),
                        category,
                        message: test
                            .error_message
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_string()),
                        comparison: self.build_comparison(test),
                        stack_trace: None,
                        request_context: serde_json::json!({
                            "tool": test.test_case.tool_name,
                            "params": test.test_case.input_params
                        }),
                        response_context: test.actual_response.clone(),
                        performance_context: test
                            .debug_info
                            .get("performance")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    };

                    failure_details.push(failure_detail);
                }
            }
        }

        let common_patterns = self.identify_patterns(&failure_details);

        Ok(super::FailureAnalysis {
            total_failures,
            failure_categories,
            failure_details,
            common_patterns,
        })
    }

    /// Categorize failure type based on error message and context
    fn categorize_failure(&self, test: &TestResult) -> super::FailureCategory {
        if let Some(error_msg) = &test.error_message {
            if error_msg.contains("timeout") || error_msg.contains("time") {
                super::FailureCategory::TimeoutError
            } else if error_msg.contains("connection") || error_msg.contains("network") {
                super::FailureCategory::ConnectionError
            } else if error_msg.contains("validation") || error_msg.contains("expected") {
                super::FailureCategory::ValidationError
            } else if error_msg.contains("performance") || error_msg.contains("slow") {
                super::FailureCategory::PerformanceRegression
            } else if error_msg.contains("config") || error_msg.contains("setting") {
                super::FailureCategory::ConfigurationError
            } else {
                super::FailureCategory::UnexpectedResponse
            }
        } else {
            super::FailureCategory::UnexpectedResponse
        }
    }

    /// Build expected vs actual comparison if available
    fn build_comparison(&self, test: &TestResult) -> Option<super::ExpectedVsActual> {
        // Extract comparison from validation results if available
        if let Some(validation_result) = test.validation_results.first() {
            if let Some(actual) = &validation_result.actual_value {
                return Some(super::ExpectedVsActual {
                    expected: serde_json::json!(validation_result.pattern),
                    actual: actual.clone(),
                    diff: super::DiffResult {
                        added_lines: vec![format!("+ {}", actual)],
                        removed_lines: vec![format!("- {:?}", validation_result.pattern)],
                        context_lines: vec!["Context: validation failure".to_string()],
                    },
                    diff_path: validation_result.pattern.key.clone(),
                });
            }
        }
        None
    }

    /// Identify common failure patterns
    fn identify_patterns(&self, failures: &[super::FailureDetail]) -> Vec<super::FailurePattern> {
        let mut pattern_counts = HashMap::new();
        let mut tool_mapping: HashMap<String, Vec<String>> = HashMap::new();

        for failure in failures {
            // Simple pattern detection based on error message keywords
            let keywords = self.extract_keywords(&failure.message);
            for keyword in keywords {
                let count = pattern_counts.entry(keyword.clone()).or_insert(0);
                *count += 1;

                tool_mapping
                    .entry(keyword)
                    .or_default()
                    .push(failure.tool_name.clone());
            }
        }

        pattern_counts
            .into_iter()
            .filter(|(_, count)| *count > 1) // Only patterns that occur multiple times
            .map(|(pattern, frequency)| super::FailurePattern {
                pattern: pattern.clone(),
                frequency,
                affected_tools: tool_mapping.get(&pattern).cloned().unwrap_or_default(),
                suggested_fix: self.suggest_fix(&pattern),
            })
            .collect()
    }

    /// Extract keywords from error messages for pattern detection
    fn extract_keywords(&self, message: &str) -> Vec<String> {
        let keywords = [
            "timeout",
            "connection",
            "validation",
            "network",
            "config",
            "permission",
        ];
        keywords
            .iter()
            .filter(|&&keyword| message.to_lowercase().contains(keyword))
            .map(|&s| s.to_string())
            .collect()
    }

    /// Suggest fixes for common patterns
    fn suggest_fix(&self, pattern: &str) -> Option<String> {
        match pattern {
            "timeout" => Some(
                "Consider increasing timeout values or optimizing tool performance".to_string(),
            ),
            "connection" => Some("Check network connectivity and server availability".to_string()),
            "validation" => {
                Some("Review expected values and ensure test data is correct".to_string())
            }
            "config" => Some("Verify configuration files and environment settings".to_string()),
            _ => None,
        }
    }
}

impl Default for FailureDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContext {
    /// Environment variables at time of failure
    pub environment: HashMap<String, String>,
    /// System resource usage
    pub system_resources: SystemResources,
    /// Network connectivity status
    pub network_status: NetworkStatus,
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// Available memory in MB
    pub available_memory_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk space available in GB
    pub disk_space_gb: f64,
}

/// Network connectivity status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Internet connectivity available
    pub internet_available: bool,
    /// DNS resolution working
    pub dns_working: bool,
    /// Average latency in milliseconds
    pub latency_ms: Option<u64>,
}

/// Diff highlighting for visual comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHighlight {
    /// Lines added in the diff
    pub added: Vec<DiffLine>,
    /// Lines removed in the diff
    pub removed: Vec<DiffLine>,
    /// Context lines for reference
    pub context: Vec<DiffLine>,
}

/// Individual line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line number
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Line type (added, removed, context)
    pub line_type: DiffLineType,
}

/// Type of diff line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}
