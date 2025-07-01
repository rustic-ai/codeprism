//! Enhanced test reporting and output generation with professional styling

use anyhow::{Context, Result};
use chrono::Utc;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Report format enumeration
#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    /// HTML report with interactive features and professional styling
    Html,
    /// JSON machine-readable report
    Json,
    /// XML report
    Xml,
    /// JUnit XML format for CI/CD integration
    Junit,
    /// Markdown report for documentation
    Markdown,
}

impl fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportFormat::Html => write!(f, "html"),
            ReportFormat::Json => write!(f, "json"),
            ReportFormat::Xml => write!(f, "xml"),
            ReportFormat::Junit => write!(f, "junit"),
            ReportFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Enhanced report generator with professional styling and templates
pub struct ReportGenerator {
    handlebars: Handlebars<'static>,
    enable_charts: bool,
    report_id: String,
}

/// Template data for report generation
#[derive(Debug, Serialize)]
struct ReportData {
    // Core data
    spec: crate::spec::schema::ServerSpec,
    stats: crate::types::TestStats,
    results: Vec<TestResultData>,

    // Enhanced fields
    generation_time: String,
    formatted_duration: String,
    pass_rate: f64,
    failure_rate: f64,
    average_duration: String,
    failed_tests: Vec<TestResultData>,
    timeline_labels: Vec<String>,
    timeline_durations: Vec<u128>,
    report_id: String,
    harness_version: String,

    // Template features
    enable_charts: bool,
    styles: String,
}

/// Enhanced test result data for templates
#[derive(Debug, Clone, Serialize)]
struct TestResultData {
    test_name: String,
    description: Option<String>,
    passed: bool,
    duration_ms: u128,
    error: Option<String>,
    tags: Vec<String>,
    start_time: String,
}

impl ReportGenerator {
    /// Create a new enhanced report generator
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register HTML template
        let html_template = include_str!("templates/report.hbs");
        handlebars
            .register_template_string("html_report", html_template)
            .context("Failed to register HTML template")?;

        // Register built-in helpers
        handlebars.register_helper("percentage", Box::new(percentage_helper));
        handlebars.register_helper("format_duration", Box::new(duration_helper));

        Ok(Self {
            handlebars,
            enable_charts: false, // Charts disabled due to system dependencies
            report_id: Uuid::new_v4().to_string(),
        })
    }

    /// Create a report generator with custom configuration
    pub fn with_config(enable_charts: bool) -> Result<Self> {
        let mut generator = Self::new()?;
        generator.enable_charts = enable_charts;
        Ok(generator)
    }

    /// Generate a comprehensive report in the specified format
    pub async fn generate(
        &self,
        report: &crate::testing::TestReport,
        format: ReportFormat,
        output_path: &Path,
    ) -> Result<()> {
        match format {
            ReportFormat::Html => self.generate_html_report(report, output_path).await,
            ReportFormat::Json => self.generate_json_report(report, output_path).await,
            ReportFormat::Junit => self.generate_junit_report(report, output_path).await,
            ReportFormat::Markdown => self.generate_markdown_report(report, output_path).await,
            ReportFormat::Xml => self.generate_xml_report(report, output_path).await,
        }
    }

    /// Generate professional HTML report with charts and styling
    async fn generate_html_report(
        &self,
        report: &crate::testing::TestReport,
        output_path: &Path,
    ) -> Result<()> {
        let template_data = self.prepare_template_data(report)?;

        let html_content = self
            .handlebars
            .render("html_report", &template_data)
            .context("Failed to render HTML template")?;

        fs::write(output_path, html_content)
            .with_context(|| format!("Failed to write HTML report to {}", output_path.display()))?;

        tracing::info!("Generated HTML report: {}", output_path.display());
        Ok(())
    }

    /// Generate machine-readable JSON report
    async fn generate_json_report(
        &self,
        report: &crate::testing::TestReport,
        output_path: &Path,
    ) -> Result<()> {
        let template_data = self.prepare_template_data(report)?;

        let json_content = serde_json::to_string_pretty(&template_data)
            .context("Failed to serialize JSON report")?;

        fs::write(output_path, json_content)
            .with_context(|| format!("Failed to write JSON report to {}", output_path.display()))?;

        tracing::info!("Generated JSON report: {}", output_path.display());
        Ok(())
    }

    /// Generate JUnit XML report for CI/CD integration
    async fn generate_junit_report(
        &self,
        report: &crate::testing::TestReport,
        output_path: &Path,
    ) -> Result<()> {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        xml.push_str(&format!(
            r#"<testsuite name="{}" tests="{}" failures="{}" time="{:.3}">"#,
            report.spec.name,
            report.stats.total_tests,
            report.stats.failed_tests,
            report.stats.total_duration_ms as f64 / 1000.0
        ));
        xml.push('\n');

        for result in &report.results {
            xml.push_str(&format!(
                r#"  <testcase name="{}" classname="{}" time="{:.3}""#,
                result.test_name,
                report.spec.name,
                result.duration_ms() as f64 / 1000.0
            ));

            if result.passed {
                xml.push_str(" />\n");
            } else {
                xml.push_str(">\n");
                xml.push_str(&format!(
                    r#"    <failure message="{}">{}</failure>"#,
                    result.error.as_ref().unwrap_or(&"Test failed".to_string()),
                    result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                ));
                xml.push_str("\n  </testcase>\n");
            }
        }

        xml.push_str("</testsuite>\n");

        fs::write(output_path, xml).with_context(|| {
            format!("Failed to write JUnit report to {}", output_path.display())
        })?;

        tracing::info!("Generated JUnit report: {}", output_path.display());
        Ok(())
    }

    /// Generate Markdown report for documentation
    async fn generate_markdown_report(
        &self,
        report: &crate::testing::TestReport,
        output_path: &Path,
    ) -> Result<()> {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# MCP Test Report: {}\n\n", report.spec.name));
        md.push_str(&format!("**Version**: {}\n", report.spec.version));
        md.push_str(&format!(
            "**Generated**: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!(
            "**Transport**: {}\n\n",
            report.spec.server.transport
        ));

        // Statistics
        md.push_str("## Test Statistics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total Tests | {} |\n", report.stats.total_tests));
        md.push_str(&format!("| Passed | {} |\n", report.stats.passed_tests));
        md.push_str(&format!("| Failed | {} |\n", report.stats.failed_tests));
        md.push_str(&format!(
            "| Pass Rate | {:.1}% |\n",
            report.stats.pass_rate()
        ));
        md.push_str(&format!(
            "| Total Duration | {:.2}s |\n",
            report.stats.total_duration_ms as f64 / 1000.0
        ));
        md.push_str(&format!(
            "| Average Duration | {:.1}ms |\n",
            report.stats.average_duration_ms
        ));
        md.push('\n');

        // Failed tests (if any)
        let failed_tests: Vec<_> = report.results.iter().filter(|r| !r.passed).collect();
        if !failed_tests.is_empty() {
            md.push_str("## Failed Tests\n\n");
            for test in failed_tests {
                md.push_str(&format!("### ❌ {}\n", test.test_name));
                if let Some(description) = &test.description {
                    md.push_str(&format!("**Description**: {}\n", description));
                }
                md.push_str(&format!("**Duration**: {}ms\n", test.duration_ms()));
                if let Some(error) = &test.error {
                    md.push_str(&format!("**Error**:\n```\n{}\n```\n", error));
                }
                md.push('\n');
            }
        }

        // All test results
        md.push_str("## All Test Results\n\n");
        md.push_str("| Status | Test Name | Duration | Tags |\n");
        md.push_str("|--------|-----------|----------|------|\n");

        for result in &report.results {
            let status = if result.passed {
                "✅ Passed"
            } else {
                "❌ Failed"
            };
            let tags = result.tags.join(", ");
            md.push_str(&format!(
                "| {} | {} | {}ms | {} |\n",
                status,
                result.test_name,
                result.duration_ms(),
                tags
            ));
        }

        fs::write(output_path, md).with_context(|| {
            format!(
                "Failed to write Markdown report to {}",
                output_path.display()
            )
        })?;

        tracing::info!("Generated Markdown report: {}", output_path.display());
        Ok(())
    }

    /// Generate XML report
    async fn generate_xml_report(
        &self,
        report: &crate::testing::TestReport,
        output_path: &Path,
    ) -> Result<()> {
        let template_data = self.prepare_template_data(report)?;

        // Convert to XML format
        let xml_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<test-report>
    <server name="{}" version="{}" transport="{}"/>
    <statistics total="{}" passed="{}" failed="{}" pass-rate="{:.1}"/>
    <tests>
{}
    </tests>
</test-report>"#,
            report.spec.name,
            report.spec.version,
            report.spec.server.transport,
            report.stats.total_tests,
            report.stats.passed_tests,
            report.stats.failed_tests,
            report.stats.pass_rate(),
            template_data
                .results
                .iter()
                .map(|r| {
                    let error_attr = if r.passed {
                        String::new()
                    } else {
                        format!(
                            r#" error="{}""#,
                            r.error.as_ref().unwrap_or(&"Unknown".to_string())
                        )
                    };
                    format!(
                        r#"        <test name="{}" passed="{}" duration="{}"{}/>"#,
                        r.test_name, r.passed, r.duration_ms, error_attr
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        fs::write(output_path, xml_content)
            .with_context(|| format!("Failed to write XML report to {}", output_path.display()))?;

        tracing::info!("Generated XML report: {}", output_path.display());
        Ok(())
    }

    /// Prepare template data from test report
    fn prepare_template_data(&self, report: &crate::testing::TestReport) -> Result<ReportData> {
        let results: Vec<TestResultData> = report
            .results
            .iter()
            .map(|r| TestResultData {
                test_name: r.test_name.clone(),
                description: r.description.clone(),
                passed: r.passed,
                duration_ms: r.duration_ms(),
                error: r.error.clone(),
                tags: r.tags.clone(),
                start_time: r.start_time.format("%H:%M:%S").to_string(),
            })
            .collect();

        let failed_tests: Vec<TestResultData> =
            results.iter().filter(|r| !r.passed).cloned().collect();

        let timeline_labels: Vec<String> = results
            .iter()
            .enumerate()
            .map(|(i, _)| format!("T{}", i + 1))
            .collect();

        let timeline_durations: Vec<u128> = results.iter().map(|r| r.duration_ms).collect();

        let pass_rate = if report.stats.total_tests > 0 {
            (report.stats.passed_tests as f64 / report.stats.total_tests as f64) * 100.0
        } else {
            0.0
        };

        let failure_rate = 100.0 - pass_rate;

        let styles_content = include_str!("assets/styles.css");

        Ok(ReportData {
            spec: report.spec.clone(),
            stats: report.stats.clone(),
            results,
            generation_time: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            formatted_duration: format!("{:.2}s", report.stats.total_duration_ms as f64 / 1000.0),
            pass_rate,
            failure_rate,
            average_duration: format!("{:.1}", report.stats.average_duration_ms),
            failed_tests,
            timeline_labels,
            timeline_durations,
            report_id: self.report_id.clone(),
            harness_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_charts: self.enable_charts,
            styles: styles_content.to_string(),
        })
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ReportGenerator")
    }
}

/// Handlebars helper for calculating percentages
fn percentage_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let numerator = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let denominator = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(1.0);

    let percentage = if denominator > 0.0 {
        (numerator / denominator) * 100.0
    } else {
        0.0
    };

    out.write(&format!("{:.1}", percentage))?;
    Ok(())
}

/// Handlebars helper for formatting durations
fn duration_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let duration_ms = h.param(0).and_then(|v| v.value().as_u64()).unwrap_or(0);

    let formatted = if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        format!("{:.2}s", duration_ms as f64 / 1000.0)
    };

    out.write(&formatted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format_display() {
        assert_eq!(ReportFormat::Html.to_string(), "html");
        assert_eq!(ReportFormat::Json.to_string(), "json");
        assert_eq!(ReportFormat::Junit.to_string(), "junit");
        assert_eq!(ReportFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn test_report_generator_creation() {
        let generator = ReportGenerator::new().unwrap();
        assert!(!generator.report_id.is_empty());
    }

    #[test]
    fn test_report_generator_with_config() {
        let generator = ReportGenerator::with_config(true).unwrap();
        assert!(generator.enable_charts);
    }
}
