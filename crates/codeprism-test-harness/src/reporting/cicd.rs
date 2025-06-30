//! CI/CD integration for test results
//!
//! This module provides integration with CI/CD systems, including GitHub Actions
//! annotations, exit code management, and automated artifact generation.

use super::{FailureDetail, Report, ReportFormatter};
use anyhow::Result;
use std::env;

/// CI/CD integration manager
pub struct CiCdIntegration {
    github_annotator: GitHubActionsAnnotator,
    exit_code_manager: ExitCodeManager,
}

impl CiCdIntegration {
    pub fn new() -> Self {
        Self {
            github_annotator: GitHubActionsAnnotator::new(),
            exit_code_manager: ExitCodeManager::new(),
        }
    }

    /// Process report for CI/CD integration
    pub async fn process_report(&self, report: &Report) -> Result<i32> {
        // Generate GitHub Actions annotations if running in GitHub Actions
        if self.is_github_actions() {
            self.github_annotator
                .annotate_failures(&report.failure_analysis.failure_details)?;
            self.github_annotator.annotate_summary(&report.summary)?;
        }

        // Generate performance regression alerts
        if !report.performance_analysis.regression_alerts.is_empty() {
            self.github_annotator
                .annotate_performance_regressions(&report.performance_analysis.regression_alerts)?;
        }

        // Set GitHub Actions outputs
        self.set_github_outputs(report)?;

        // Return appropriate exit code
        Ok(self.exit_code_manager.determine_exit_code(report))
    }

    /// Check if running in GitHub Actions environment
    fn is_github_actions(&self) -> bool {
        env::var("GITHUB_ACTIONS").is_ok()
    }

    /// Set GitHub Actions output variables
    fn set_github_outputs(&self, report: &Report) -> Result<()> {
        if !self.is_github_actions() {
            return Ok(());
        }

        let outputs = vec![
            ("total_tests", report.summary.total_tests.to_string()),
            ("passed_tests", report.summary.passed_tests.to_string()),
            (
                "failed_tests",
                (report.summary.total_tests - report.summary.passed_tests).to_string(),
            ),
            (
                "success_rate",
                format!("{:.1}", report.summary.success_rate_percent),
            ),
            (
                "total_time_ms",
                report.summary.total_execution_time_ms.to_string(),
            ),
            (
                "has_failures",
                (report.failure_analysis.total_failures > 0).to_string(),
            ),
            (
                "has_performance_regressions",
                (!report.performance_analysis.regression_alerts.is_empty()).to_string(),
            ),
        ];

        for (key, value) in outputs {
            println!("::set-output name={}::{}", key, value);
        }

        Ok(())
    }
}

/// GitHub Actions annotation generator
pub struct GitHubActionsAnnotator;

impl GitHubActionsAnnotator {
    pub fn new() -> Self {
        Self
    }

    /// Annotate test failures with GitHub Actions annotations
    pub fn annotate_failures(&self, failures: &[FailureDetail]) -> Result<()> {
        for failure in failures {
            let annotation_type = match failure.category {
                super::FailureCategory::ValidationError => "error",
                super::FailureCategory::PerformanceRegression => "warning",
                super::FailureCategory::ConfigurationError => "error",
                _ => "error",
            };

            println!(
                "::{} title=Test Failure: {}::Tool '{}' failed: {}",
                annotation_type, failure.test_id, failure.tool_name, failure.message
            );
        }
        Ok(())
    }

    /// Annotate test summary
    pub fn annotate_summary(&self, summary: &super::TestSummary) -> Result<()> {
        if summary.passed_tests == summary.total_tests {
            println!(
                "::notice title=All Tests Passed::✅ {}/{} tests passed ({:.1}% success rate)",
                summary.passed_tests, summary.total_tests, summary.success_rate_percent
            );
        } else {
            println!(
                "::error title=Test Failures::❌ {}/{} tests failed ({:.1}% success rate)",
                summary.total_tests - summary.passed_tests,
                summary.total_tests,
                100.0 - summary.success_rate_percent
            );
        }
        Ok(())
    }

    /// Annotate performance regressions
    pub fn annotate_performance_regressions(
        &self,
        regressions: &[crate::performance::RegressionAlert],
    ) -> Result<()> {
        for regression in regressions {
            println!(
                "::warning title=Performance Regression::⚠️ Performance regression detected: {}",
                regression.message
            );
        }
        Ok(())
    }
}

/// Exit code management for CI/CD systems
pub struct ExitCodeManager;

impl ExitCodeManager {
    pub fn new() -> Self {
        Self
    }

    /// Determine appropriate exit code based on test results
    pub fn determine_exit_code(&self, report: &Report) -> i32 {
        // Critical failures always result in exit code 1
        if report.failure_analysis.total_failures > 0 {
            return 1;
        }

        // Performance regressions can be configured to fail builds
        if !report.performance_analysis.regression_alerts.is_empty() {
            let critical_regressions = report
                .performance_analysis
                .regression_alerts
                .iter()
                .any(|alert| matches!(alert.severity, crate::performance::AlertSeverity::Critical));

            if critical_regressions {
                return 1;
            }
        }

        // Coverage below threshold can be configured to fail builds
        if report.coverage_analysis.tool_coverage.coverage_percentage < 80.0 {
            // This could be configurable
            eprintln!(
                "Warning: Tool coverage below 80% ({:.1}%)",
                report.coverage_analysis.tool_coverage.coverage_percentage
            );
        }

        0 // Success
    }
}

/// Generate artifacts for CI/CD systems
#[allow(dead_code)]
pub struct ArtifactGenerator;

impl ArtifactGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate test artifacts for CI/CD
    #[allow(dead_code)]
    pub async fn generate_artifacts(
        &self,
        report: &Report,
        output_dir: &std::path::Path,
    ) -> Result<Vec<ArtifactInfo>> {
        let mut artifacts = Vec::new();

        // Generate HTML report
        let html_path = output_dir.join("test-report.html");
        let html_formatter = super::HtmlFormatter::new();
        let html_content = html_formatter.format(report).await?;
        tokio::fs::write(&html_path, html_content).await?;
        artifacts.push(ArtifactInfo {
            name: "HTML Test Report".to_string(),
            path: html_path,
            artifact_type: ArtifactType::Report,
        });

        // Generate JSON report for programmatic access
        let json_path = output_dir.join("test-results.json");
        let json_formatter = super::JsonFormatter::new();
        let json_content = json_formatter.format(report).await?;
        tokio::fs::write(&json_path, json_content).await?;
        artifacts.push(ArtifactInfo {
            name: "JSON Test Results".to_string(),
            path: json_path,
            artifact_type: ArtifactType::Data,
        });

        // Generate JUnit XML for CI/CD integration
        let junit_path = output_dir.join("junit-results.xml");
        let junit_formatter = super::JunitXmlFormatter::new();
        let junit_content = junit_formatter.format(report).await?;
        tokio::fs::write(&junit_path, junit_content).await?;
        artifacts.push(ArtifactInfo {
            name: "JUnit XML Results".to_string(),
            path: junit_path,
            artifact_type: ArtifactType::TestResults,
        });

        Ok(artifacts)
    }
}

/// Information about generated artifacts
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArtifactInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub artifact_type: ArtifactType,
}

/// Types of artifacts that can be generated
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ArtifactType {
    Report,
    Data,
    TestResults,
    Performance,
}

impl Default for CiCdIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GitHubActionsAnnotator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExitCodeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ArtifactGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_success() {
        let _manager = ExitCodeManager::new();
        // Would need to create a mock report with no failures
        // assert_eq!(manager.determine_exit_code(&mock_report), 0);
    }

    #[test]
    fn test_github_actions_detection() {
        let integration = CiCdIntegration::new();
        // This would depend on environment variables
        println!(
            "GitHub Actions detected: {}",
            integration.is_github_actions()
        );
    }
}
