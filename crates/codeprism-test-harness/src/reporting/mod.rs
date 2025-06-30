//! Comprehensive test result reporting and error diagnostics
//!
//! This module provides detailed error reporting, multiple output formats,
//! and integration with CI/CD systems for the CodePrism Test Harness.

mod analysis;
mod cicd;
mod diagnostics;
mod formatters;

pub use analysis::{PerformanceAnalyzer, TestCoverageAnalyzer};
pub use cicd::{CiCdIntegration, ExitCodeManager, GitHubActionsAnnotator};
pub use diagnostics::{DiffHighlight, FailureContext, FailureDiagnostics};
pub use formatters::{
    HtmlFormatter, JsonFormatter, JunitXmlFormatter, MarkdownFormatter, ReportFormat,
    ReportFormatter,
};

use crate::performance::PerformanceResult;
use crate::types::TestSuiteResult;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Comprehensive test report containing all analysis and formatting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Summary statistics
    pub summary: TestSummary,
    /// Detailed test suite results
    pub test_suites: Vec<TestSuiteResult>,
    /// Failure analysis and diagnostics
    pub failure_analysis: FailureAnalysis,
    /// Performance analysis results
    pub performance_analysis: PerformanceAnalysis,
    /// Coverage analysis
    pub coverage_analysis: CoverageAnalysis,
}

/// Report metadata and generation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Test harness version
    pub harness_version: String,
    /// Environment information
    pub environment: EnvironmentInfo,
    /// Report format configuration
    pub format_config: ReportFormatConfig,
}

/// High-level test execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total number of test suites executed
    pub total_suites: usize,
    /// Number of test suites that passed
    pub passed_suites: usize,
    /// Total number of individual tests
    pub total_tests: usize,
    /// Number of individual tests that passed
    pub passed_tests: usize,
    /// Total execution time for all tests
    pub total_execution_time_ms: u64,
    /// Average execution time per test
    pub average_execution_time_ms: u64,
    /// Overall success rate percentage
    pub success_rate_percent: f64,
}

/// Detailed failure analysis with context and diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    /// Total number of failures
    pub total_failures: usize,
    /// Failures categorized by type
    pub failure_categories: HashMap<String, usize>,
    /// Detailed failure information
    pub failure_details: Vec<FailureDetail>,
    /// Common failure patterns identified
    pub common_patterns: Vec<FailurePattern>,
}

/// Individual failure detail with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetail {
    /// Test case identifier
    pub test_id: String,
    /// Tool that was being tested
    pub tool_name: String,
    /// Failure category
    pub category: FailureCategory,
    /// Human-readable error message
    pub message: String,
    /// Expected vs actual comparison
    pub comparison: Option<ExpectedVsActual>,
    /// Stack trace or execution path
    pub stack_trace: Option<String>,
    /// Full request that was sent
    pub request_context: serde_json::Value,
    /// Response that was received
    pub response_context: Option<serde_json::Value>,
    /// Performance metrics at time of failure
    pub performance_context: Option<PerformanceResult>,
}

/// Expected vs actual value comparison with diff highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedVsActual {
    /// Expected value or pattern
    pub expected: serde_json::Value,
    /// Actual value received
    pub actual: serde_json::Value,
    /// Diff highlighting for visual comparison
    pub diff: DiffResult,
    /// JSON path where the difference occurred
    pub diff_path: String,
}

/// Performance analysis across all tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Performance summary statistics
    pub summary: PerformanceSummary,
    /// Performance trends over time
    pub trends: Vec<PerformanceTrend>,
    /// Regression alerts detected
    pub regression_alerts: Vec<RegressionAlert>,
    /// Performance baseline comparisons
    pub baseline_comparisons: Vec<BaselineComparison>,
}

/// Test coverage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    /// Tool coverage statistics
    pub tool_coverage: ToolCoverage,
    /// Feature coverage analysis
    pub feature_coverage: FeatureCoverage,
    /// Coverage trends over time
    pub coverage_trends: Vec<CoverageTrend>,
}

/// Environment information for debugging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// Rust version
    pub rust_version: String,
    /// Test harness version
    pub test_harness_version: String,
    /// Environment variables relevant to testing
    pub env_vars: HashMap<String, String>,
}

/// Configuration for report formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFormatConfig {
    /// Include debug information
    pub include_debug_info: bool,
    /// Include performance charts
    pub include_performance_charts: bool,
    /// Include full request/response context
    pub include_full_context: bool,
    /// Maximum number of failure details to include
    pub max_failure_details: usize,
}

impl Default for ReportFormatConfig {
    fn default() -> Self {
        Self {
            include_debug_info: true,
            include_performance_charts: true,
            include_full_context: false,
            max_failure_details: 50,
        }
    }
}

/// Main report generator that coordinates all analysis and formatting
pub struct ReportGenerator {
    failure_diagnostics: FailureDiagnostics,
    performance_analyzer: PerformanceAnalyzer,
    coverage_analyzer: TestCoverageAnalyzer,
    formatters: HashMap<ReportFormat, Box<dyn ReportFormatter>>,
}

impl ReportGenerator {
    /// Create a new report generator with default configuration
    pub fn new() -> Self {
        let mut formatters: HashMap<ReportFormat, Box<dyn ReportFormatter>> = HashMap::new();
        formatters.insert(ReportFormat::Html, Box::new(HtmlFormatter::new()));
        formatters.insert(ReportFormat::Json, Box::new(JsonFormatter::new()));

        Self {
            failure_diagnostics: FailureDiagnostics::new(),
            performance_analyzer: PerformanceAnalyzer::new(),
            coverage_analyzer: TestCoverageAnalyzer::new(),
            formatters,
        }
    }

    /// Generate a comprehensive report from test results
    pub async fn generate_report(
        &self,
        test_results: &[TestSuiteResult],
        format: ReportFormat,
        output_path: Option<PathBuf>,
    ) -> Result<Report> {
        // Build comprehensive report
        let report = self.build_report(test_results).await?;

        // Format and save report if output path provided
        if let Some(path) = output_path {
            self.save_formatted_report(&report, format, &path).await?;
        }

        Ok(report)
    }

    /// Build comprehensive report from test results
    async fn build_report(&self, test_results: &[TestSuiteResult]) -> Result<Report> {
        let metadata = self.build_metadata().await?;
        let summary = self.build_summary(test_results);
        let failure_analysis = self
            .failure_diagnostics
            .analyze_failures(test_results)
            .await?;
        let performance_analysis = self.performance_analyzer.analyze(test_results).await?;
        let coverage_analysis = self.coverage_analyzer.analyze(test_results).await?;

        Ok(Report {
            metadata,
            summary,
            test_suites: test_results.to_vec(),
            failure_analysis,
            performance_analysis,
            coverage_analysis,
        })
    }

    /// Build report metadata
    async fn build_metadata(&self) -> Result<ReportMetadata> {
        Ok(ReportMetadata {
            generated_at: Utc::now(),
            harness_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: EnvironmentInfo {
                os: std::env::consts::OS.to_string(),
                rust_version: "1.70+".to_string(), // Could be detected dynamically
                test_harness_version: env!("CARGO_PKG_VERSION").to_string(),
                env_vars: std::env::vars()
                    .filter(|(k, _)| k.starts_with("CARGO_") || k.starts_with("RUST_"))
                    .collect(),
            },
            format_config: ReportFormatConfig::default(),
        })
    }

    /// Build test summary statistics
    fn build_summary(&self, test_results: &[TestSuiteResult]) -> TestSummary {
        let total_suites = test_results.len();
        let passed_suites = test_results.iter().filter(|r| r.suite_passed).count();

        let total_tests: usize = test_results.iter().map(|r| r.stats.total_tests).sum();
        let passed_tests: usize = test_results.iter().map(|r| r.stats.passed_tests).sum();

        let total_execution_time_ms: u64 = test_results
            .iter()
            .map(|r| r.stats.total_duration.as_millis() as u64)
            .sum();

        let average_execution_time_ms = if total_tests > 0 {
            total_execution_time_ms / total_tests as u64
        } else {
            0
        };

        let success_rate_percent = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            100.0
        };

        TestSummary {
            total_suites,
            passed_suites,
            total_tests,
            passed_tests,
            total_execution_time_ms,
            average_execution_time_ms,
            success_rate_percent,
        }
    }

    /// Save formatted report to file
    async fn save_formatted_report(
        &self,
        report: &Report,
        format: ReportFormat,
        output_path: &PathBuf,
    ) -> Result<()> {
        if let Some(formatter) = self.formatters.get(&format) {
            let formatted_content = formatter.format(report).await?;
            tokio::fs::write(output_path, formatted_content).await?;
        }
        Ok(())
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Additional type definitions for completeness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureCategory {
    ValidationError,
    TimeoutError,
    ConnectionError,
    PerformanceRegression,
    UnexpectedResponse,
    ConfigurationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub pattern: String,
    pub frequency: usize,
    pub affected_tools: Vec<String>,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub added_lines: Vec<String>,
    pub removed_lines: Vec<String>,
    pub context_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub average_execution_time_ms: f64,
    pub p95_execution_time_ms: u64,
    pub total_memory_usage_mb: f64,
    pub throughput_ops_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric_name: String,
    pub data_points: Vec<(DateTime<Utc>, f64)>,
    pub trend_direction: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub tool_name: String,
    pub current_performance: f64,
    pub baseline_performance: f64,
    pub performance_delta_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCoverage {
    pub total_tools: usize,
    pub tested_tools: usize,
    pub coverage_percentage: f64,
    pub untested_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCoverage {
    pub total_features: usize,
    pub tested_features: usize,
    pub coverage_percentage: f64,
    pub feature_breakdown: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrend {
    pub timestamp: DateTime<Utc>,
    pub tool_coverage_percent: f64,
    pub feature_coverage_percent: f64,
}

// Re-export RegressionAlert from performance module
pub use crate::performance::RegressionAlert;
