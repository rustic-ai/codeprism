//! Failure diagnostics and context analysis

use crate::types::{TestResult, TestSuiteResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Failure diagnostics analyzer with enhanced intelligence
pub struct FailureDiagnostics {
    /// System information collector
    system_collector: SystemInfoCollector,
    /// Pattern analyzer for error correlation
    pattern_analyzer: ErrorPatternAnalyzer,
}

impl FailureDiagnostics {
    pub fn new() -> Self {
        Self {
            system_collector: SystemInfoCollector::new(),
            pattern_analyzer: ErrorPatternAnalyzer::new(),
        }
    }

    /// Analyze failures across test results with enhanced diagnostics
    pub async fn analyze_failures(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Result<super::FailureAnalysis> {
        let mut failure_details = Vec::new();
        let mut failure_categories = HashMap::new();
        let mut total_failures = 0;

        // Collect system context for analysis
        let system_context = self.system_collector.collect_context().await?;

        for suite in test_results {
            for test in &suite.test_results {
                if !test.success {
                    total_failures += 1;

                    let category = self.categorize_failure_enhanced(test, &system_context);
                    *failure_categories
                        .entry(format!("{:?}", category))
                        .or_insert(0) += 1;

                    let failure_detail = super::FailureDetail {
                        test_id: test.test_case.id.clone(),
                        tool_name: test.test_case.tool_name.clone(),
                        category: category.clone(),
                        message: test
                            .error_message
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_string()),
                        comparison: self.build_enhanced_comparison(test),
                        stack_trace: self.extract_stack_trace(test),
                        request_context: serde_json::json!({
                            "tool": test.test_case.tool_name,
                            "params": test.test_case.input_params,
                            "timestamp": SystemTime::now(),
                            "environment": system_context
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

        let common_patterns = self
            .pattern_analyzer
            .identify_intelligent_patterns(&failure_details);

        Ok(super::FailureAnalysis {
            total_failures,
            failure_categories,
            failure_details,
            common_patterns,
        })
    }

    /// Enhanced failure categorization with intelligent analysis
    fn categorize_failure_enhanced(
        &self,
        test: &TestResult,
        system_context: &SystemContext,
    ) -> super::FailureCategory {
        if let Some(error_msg) = &test.error_message {
            let msg_lower = error_msg.to_lowercase();

            // Performance-related failures
            if msg_lower.contains("timeout")
                || msg_lower.contains("slow")
                || msg_lower.contains("performance")
            {
                // Check if system resources are constrained
                if system_context.cpu_usage_percent > 90.0
                    || system_context.memory_usage_percent > 90.0
                {
                    return super::FailureCategory::PerformanceRegression;
                }
                if msg_lower.contains("timeout") {
                    return super::FailureCategory::TimeoutError;
                }
                return super::FailureCategory::PerformanceRegression;
            }

            // Network/Connection issues
            if msg_lower.contains("connection")
                || msg_lower.contains("network")
                || msg_lower.contains("refused")
            {
                return super::FailureCategory::ConnectionError;
            }

            // Validation failures
            if msg_lower.contains("validation")
                || msg_lower.contains("expected")
                || msg_lower.contains("assert")
            {
                return super::FailureCategory::ValidationError;
            }

            // Configuration issues
            if msg_lower.contains("config")
                || msg_lower.contains("setting")
                || msg_lower.contains("parameter")
            {
                return super::FailureCategory::ConfigurationError;
            }

            // If execution time is significantly higher than normal, it's likely performance
            if test.duration.as_millis() > 5000 {
                return super::FailureCategory::PerformanceRegression;
            }
        }

        super::FailureCategory::UnexpectedResponse
    }

    /// Build enhanced comparison with better diff analysis
    fn build_enhanced_comparison(&self, test: &TestResult) -> Option<super::ExpectedVsActual> {
        if let Some(validation_result) = test.validation_results.first() {
            if let Some(actual) = &validation_result.actual_value {
                let diff = self.generate_enhanced_diff(&validation_result.pattern, actual);

                return Some(super::ExpectedVsActual {
                    expected: serde_json::json!(validation_result.pattern),
                    actual: actual.clone(),
                    diff,
                    diff_path: validation_result.pattern.key.clone(),
                });
            }
        }
        None
    }

    /// Generate enhanced diff with context
    fn generate_enhanced_diff(
        &self,
        expected: &crate::types::JsonPathPattern,
        actual: &serde_json::Value,
    ) -> super::DiffResult {
        let expected_str = format!("{:?}", expected);
        let actual_str = format!("{}", actual);

        // Simple diff implementation - in practice, you'd use a proper diff library
        let mut added_lines = Vec::new();
        let mut removed_lines = Vec::new();
        let mut context_lines = Vec::new();

        if expected_str != actual_str {
            removed_lines.push(format!("- Expected: {}", expected_str));
            added_lines.push(format!("+ Actual: {}", actual_str));
            context_lines.push(format!("@ Validation path: {}", expected.key));
        }

        super::DiffResult {
            added_lines,
            removed_lines,
            context_lines,
        }
    }

    /// Extract stack trace information from test results
    fn extract_stack_trace(&self, test: &TestResult) -> Option<String> {
        // Check debug info for stack trace
        if let Some(debug_info) = test.debug_info.get("stack_trace") {
            if let Ok(stack) = serde_json::from_value::<String>(debug_info.clone()) {
                return Some(stack);
            }
        }

        // Generate synthetic stack trace from error context
        if let Some(error_msg) = &test.error_message {
            return Some(format!(
                "Test execution failed:\n  at test_case: {}\n  in tool: {}\n  error: {}",
                test.test_case.id, test.test_case.tool_name, error_msg
            ));
        }

        None
    }
}

impl Default for FailureDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// System information collector for diagnostic context
struct SystemInfoCollector;

impl SystemInfoCollector {
    fn new() -> Self {
        Self
    }

    async fn collect_context(&self) -> Result<SystemContext> {
        // Collect system information (simplified implementation)
        // In a real implementation, you'd use system APIs to get actual values

        Ok(SystemContext {
            cpu_usage_percent: self.get_cpu_usage(),
            memory_usage_percent: self.get_memory_usage(),
            disk_usage_percent: self.get_disk_usage(),
            network_latency_ms: self.measure_network_latency().await,
            system_load_average: self.get_load_average(),
            environment_variables: self.get_relevant_env_vars(),
        })
    }

    fn get_cpu_usage(&self) -> f64 {
        // Simplified - would use actual system monitoring
        40.0
    }

    fn get_memory_usage(&self) -> f64 {
        // Simplified - would use actual system monitoring
        65.0
    }

    fn get_disk_usage(&self) -> f64 {
        // Simplified - would use actual system monitoring
        75.0
    }

    async fn measure_network_latency(&self) -> f64 {
        // Simplified - would perform actual network probe
        25.0
    }

    fn get_load_average(&self) -> f64 {
        // Simplified - would read from /proc/loadavg on Linux
        1.5
    }

    fn get_relevant_env_vars(&self) -> HashMap<String, String> {
        std::env::vars()
            .filter(|(k, _)| {
                k.starts_with("RUST_")
                    || k.starts_with("CARGO_")
                    || k.starts_with("CI")
                    || k == "PATH"
            })
            .collect()
    }
}

/// Enhanced error pattern analyzer
struct ErrorPatternAnalyzer;

impl ErrorPatternAnalyzer {
    fn new() -> Self {
        Self
    }

    /// Identify intelligent patterns with actionable recommendations
    fn identify_intelligent_patterns(
        &self,
        failures: &[super::FailureDetail],
    ) -> Vec<super::FailurePattern> {
        let mut pattern_analysis = HashMap::new();
        let _tool_mapping: HashMap<String, Vec<String>> = HashMap::new();

        // Advanced pattern detection
        for failure in failures {
            // Extract multiple types of patterns
            let patterns = self.extract_intelligent_patterns(failure);

            for pattern in patterns {
                let entry = pattern_analysis
                    .entry(pattern.clone())
                    .or_insert(PatternAnalysis {
                        frequency: 0,
                        tools: Vec::new(),
                        contexts: Vec::new(),
                    });

                entry.frequency += 1;
                entry.tools.push(failure.tool_name.clone());
                entry.contexts.push(failure.message.clone());
            }
        }

        // Generate patterns with intelligent recommendations
        pattern_analysis
            .into_iter()
            .filter(|(_, analysis)| analysis.frequency > 1)
            .map(|(pattern, analysis)| {
                let suggestion = self.generate_intelligent_suggestion(&pattern, &analysis);

                super::FailurePattern {
                    pattern: pattern.clone(),
                    frequency: analysis.frequency,
                    affected_tools: analysis
                        .tools
                        .into_iter()
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect(),
                    suggested_fix: Some(suggestion),
                }
            })
            .collect()
    }

    /// Extract multiple intelligent patterns from a failure
    fn extract_intelligent_patterns(&self, failure: &super::FailureDetail) -> Vec<String> {
        let mut patterns = Vec::new();
        let msg_lower = failure.message.to_lowercase();

        // Performance patterns
        if msg_lower.contains("timeout") {
            patterns.push("timeout_pattern".to_string());
        }
        if msg_lower.contains("slow") || msg_lower.contains("performance") {
            patterns.push("performance_degradation".to_string());
        }

        // Network patterns
        if msg_lower.contains("connection") || msg_lower.contains("network") {
            patterns.push("connectivity_issue".to_string());
        }

        // Configuration patterns
        if msg_lower.contains("config") || msg_lower.contains("parameter") {
            patterns.push("configuration_error".to_string());
        }

        // Validation patterns
        if msg_lower.contains("validation") || msg_lower.contains("expected") {
            patterns.push("validation_mismatch".to_string());
        }

        // Tool-specific patterns
        patterns.push(format!("tool_{}_failure", failure.tool_name));

        // Category-based patterns
        patterns.push(format!("category_{:?}", failure.category));

        patterns
    }

    /// Generate intelligent suggestions based on pattern analysis
    fn generate_intelligent_suggestion(&self, pattern: &str, analysis: &PatternAnalysis) -> String {
        match pattern {
            "timeout_pattern" => {
                format!(
                    "Consider increasing timeout values or optimizing tool performance. Affects {} tools with {} occurrences.",
                    analysis.tools.len(),
                    analysis.frequency
                )
            }
            "performance_degradation" => {
                "Review system resources and optimize performance-critical code paths. Consider parallel execution or caching strategies.".to_string()
            }
            "connectivity_issue" => {
                "Check network connectivity, firewall settings, and server availability. Verify endpoint URLs and network configuration.".to_string()
            }
            "configuration_error" => {
                "Review configuration files and environment settings. Validate all required parameters are provided and correctly formatted.".to_string()
            }
            "validation_mismatch" => {
                "Review expected values and ensure test data matches current implementation. Update test expectations if behavior has changed intentionally.".to_string()
            }
            pattern if pattern.starts_with("tool_") => {
                let tool_name = pattern.strip_prefix("tool_").unwrap_or("unknown").replace("_failure", "");
                format!(
                    "Tool '{}' is experiencing {} failures. Review tool-specific configuration and implementation.",
                    tool_name, analysis.frequency
                )
            }
            pattern if pattern.starts_with("category_") => {
                format!(
                    "Multiple failures in category '{}' suggest a systematic issue. Review common causes and implement comprehensive fix.",
                    pattern.strip_prefix("category_").unwrap_or("unknown")
                )
            }
            _ => format!(
                "Pattern '{}' occurred {} times across {} tools. Investigate common root cause.",
                pattern, analysis.frequency, analysis.tools.len()
            )
        }
    }
}

/// System context information for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_latency_ms: f64,
    pub system_load_average: f64,
    pub environment_variables: HashMap<String, String>,
}

/// Pattern analysis data
struct PatternAnalysis {
    frequency: usize,
    tools: Vec<String>,
    contexts: Vec<String>,
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
