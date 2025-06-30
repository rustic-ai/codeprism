//! Report formatters for different output formats

use super::Report;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Supported report output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportFormat {
    /// Rich HTML report with charts and interactive elements
    Html,
    /// Machine-readable JSON format
    Json,
    /// JUnit XML format for CI/CD integration
    JunitXml,
    /// Markdown format for GitHub comments
    Markdown,
}

/// Trait for formatting reports in different output formats
#[async_trait]
pub trait ReportFormatter: Send + Sync {
    /// Format a report into the target format
    async fn format(&self, report: &Report) -> Result<String>;

    /// Get the file extension for this format
    fn file_extension(&self) -> &'static str;

    /// Get the MIME type for this format
    fn mime_type(&self) -> &'static str;
}

/// HTML report formatter with rich visual elements
pub struct HtmlFormatter;

impl HtmlFormatter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportFormatter for HtmlFormatter {
    async fn format(&self, report: &Report) -> Result<String> {
        let mut html = String::new();

        // HTML header
        html.push_str(&format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CodePrism Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; margin: -30px -30px 20px -30px; border-radius: 8px 8px 0 0; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }}
        .metric {{ background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }}
        .metric-value {{ font-size: 2.5em; font-weight: bold; }}
        .metric-label {{ color: #666; margin-top: 10px; }}
        .success {{ color: #28a745; }}
        .error {{ color: #dc3545; }}
        .warning {{ color: #ffc107; }}
        .section {{ margin: 30px 0; }}
        .test-result {{ padding: 15px; margin: 10px 0; border-radius: 8px; }}
        .test-pass {{ background: #d4edda; border-left: 4px solid #28a745; }}
        .test-fail {{ background: #f8d7da; border-left: 4px solid #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>CodePrism Test Report</h1>
            <p>Generated: {}</p>
        </div>
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary section
        html.push_str(&format!(
            r#"        <div class="section">
            <h2>Test Summary</h2>
            <div class="summary">
                <div class="metric">
                    <div class="metric-value">{}</div>
                    <div class="metric-label">Total Tests</div>
                </div>
                <div class="metric">
                    <div class="metric-value {}">{}</div>
                    <div class="metric-label">Passed Tests</div>
                </div>
                <div class="metric">
                    <div class="metric-value {}">{:.1}%</div>
                    <div class="metric-label">Success Rate</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{}ms</div>
                    <div class="metric-label">Total Time</div>
                </div>
            </div>
        </div>
"#,
            report.summary.total_tests,
            if report.summary.passed_tests == report.summary.total_tests {
                "success"
            } else {
                "error"
            },
            report.summary.passed_tests,
            if report.summary.success_rate_percent >= 90.0 {
                "success"
            } else if report.summary.success_rate_percent >= 70.0 {
                "warning"
            } else {
                "error"
            },
            report.summary.success_rate_percent,
            report.summary.total_execution_time_ms
        ));

        // Test details
        html.push_str(
            r#"        <div class="section">
            <h2>Test Results</h2>
"#,
        );

        for suite in &report.test_suites {
            html.push_str(&format!(
                r#"            <h3>{}</h3>"#,
                suite.test_suite.name
            ));

            for test in &suite.test_results {
                let class = if test.success {
                    "test-pass"
                } else {
                    "test-fail"
                };
                let status = if test.success { "✅" } else { "❌" };

                html.push_str(&format!(
                    r#"            <div class="test-result {}">
                {} {} - {}ms {}
            </div>
"#,
                    class,
                    status,
                    test.test_case.id,
                    test.duration.as_millis(),
                    if !test.success {
                        format!(
                            "<br><small>Error: {}</small>",
                            test.error_message.as_deref().unwrap_or("Unknown error")
                        )
                    } else {
                        String::new()
                    }
                ));
            }
        }

        html.push_str(
            r#"        </div>
    </div>
</body>
</html>"#,
        );

        Ok(html)
    }

    fn file_extension(&self) -> &'static str {
        "html"
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }
}

/// JSON report formatter for programmatic analysis
pub struct JsonFormatter {
    pretty_print: bool,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self { pretty_print: true }
    }

    pub fn with_pretty_print(pretty_print: bool) -> Self {
        Self { pretty_print }
    }
}

#[async_trait]
impl ReportFormatter for JsonFormatter {
    async fn format(&self, report: &Report) -> Result<String> {
        if self.pretty_print {
            Ok(serde_json::to_string_pretty(report)?)
        } else {
            Ok(serde_json::to_string(report)?)
        }
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }
}

/// JUnit XML formatter for CI/CD integration
pub struct JunitXmlFormatter;

impl JunitXmlFormatter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportFormatter for JunitXmlFormatter {
    async fn format(&self, report: &Report) -> Result<String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        // Root testsuites element
        xml.push_str(&format!(
            r#"<testsuites name="CodePrism Tests" tests="{}" failures="{}" time="{:.3}">"#,
            report.summary.total_tests,
            report.summary.total_tests - report.summary.passed_tests,
            report.summary.total_execution_time_ms as f64 / 1000.0
        ));
        xml.push('\n');

        // Individual test suites
        for suite in &report.test_suites {
            xml.push_str(&format!(
                r#"  <testsuite name="{}" tests="{}" failures="{}" time="{:.3}">"#,
                suite.test_suite.name,
                suite.stats.total_tests,
                suite.stats.failed_tests,
                suite.stats.total_duration.as_secs_f64()
            ));
            xml.push('\n');

            // Individual test cases
            for test_result in &suite.test_results {
                xml.push_str(&format!(
                    r#"    <testcase name="{}" classname="{}" time="{:.3}""#,
                    test_result.test_case.id,
                    test_result.test_case.tool_name,
                    test_result.duration.as_secs_f64()
                ));

                if test_result.success {
                    xml.push_str(" />");
                } else {
                    xml.push('>');
                    xml.push('\n');
                    xml.push_str(&format!(
                        r#"      <failure message="{}">{}</failure>"#,
                        test_result
                            .error_message
                            .as_deref()
                            .unwrap_or("Test failed"),
                        test_result
                            .error_message
                            .as_deref()
                            .unwrap_or("Unknown error")
                    ));
                    xml.push('\n');
                    xml.push_str("    </testcase>");
                }
                xml.push('\n');
            }

            xml.push_str("  </testsuite>");
            xml.push('\n');
        }

        xml.push_str("</testsuites>");
        Ok(xml)
    }

    fn file_extension(&self) -> &'static str {
        "xml"
    }

    fn mime_type(&self) -> &'static str {
        "application/xml"
    }
}

/// Markdown formatter for GitHub PR comments
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportFormatter for MarkdownFormatter {
    async fn format(&self, report: &Report) -> Result<String> {
        let mut md = String::new();

        md.push_str("# 🧪 CodePrism Test Report\n\n");

        // Summary table
        md.push_str("## 📊 Summary\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!(
            "| Total Tests | {} |\n",
            report.summary.total_tests
        ));
        md.push_str(&format!(
            "| Passed Tests | {} |\n",
            report.summary.passed_tests
        ));
        md.push_str(&format!(
            "| Success Rate | {:.1}% |\n",
            report.summary.success_rate_percent
        ));
        md.push_str(&format!(
            "| Total Time | {}ms |\n",
            report.summary.total_execution_time_ms
        ));
        md.push('\n');

        // Status emoji
        let status_emoji = if report.summary.success_rate_percent >= 90.0 {
            "✅"
        } else if report.summary.success_rate_percent >= 70.0 {
            "⚠️"
        } else {
            "❌"
        };

        md.push_str(&format!(
            "**Overall Status: {} {}**\n\n",
            status_emoji,
            if report.summary.passed_tests == report.summary.total_tests {
                "All tests passed!"
            } else {
                "Some tests failed"
            }
        ));

        // Failures section
        if report.failure_analysis.total_failures > 0 {
            md.push_str("## ❌ Failures\n\n");
            for detail in &report.failure_analysis.failure_details {
                md.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    detail.test_id, detail.tool_name, detail.message
                ));
            }
            md.push('\n');
        }

        // Performance section
        md.push_str("## ⚡ Performance\n\n");
        md.push_str(&format!(
            "- Average execution time: {:.1}ms\n",
            report
                .performance_analysis
                .summary
                .average_execution_time_ms
        ));
        md.push_str(&format!(
            "- 95th percentile: {}ms\n",
            report.performance_analysis.summary.p95_execution_time_ms
        ));
        md.push('\n');

        Ok(md)
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for JunitXmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self::new()
    }
}
