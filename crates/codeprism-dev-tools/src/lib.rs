//! CodePrism Development Tools
//!
//! This crate provides essential debugging and development utilities for CodePrism parser development.
//! It includes AST visualization tools, parser validation utilities, and an interactive development REPL.
//!
//! # Features
//!
//! - **AST Visualization**: Pretty-print syntax trees and export GraphViz diagrams
//! - **Parser Validation**: Comprehensive validation of nodes, edges, and spans
//! - **Development REPL**: Interactive parser development environment
//! - **Performance Profiling**: Real-time parsing performance metrics
//! - **Diff Comparison**: Compare AST changes between parser versions
//!
//! # Example
//!
//! ```no_run
//! use codeprism_dev_tools::{AstVisualizer, ParserValidator};
//!
//! // Create dev tools
//! let visualizer = AstVisualizer::new();
//! let validator = ParserValidator::new();
//!
//! // Usage would be with actual ParseResult and source code:
//! // let tree_output = visualizer.visualize_tree(&parse_result.tree, &source_code)?;
//! // let report = validator.validate_complete(&parse_result, &source_code)?;
//! ```

use anyhow::Result;
use std::path::Path;

pub mod ast_visualizer;
pub mod dev_repl;
pub mod diff_comparison;
pub mod graphviz_export;
pub mod parser_validator;
pub mod performance_profiler;

// Re-export main types for convenience
pub use ast_visualizer::{AstVisualizer, VisualizationFormat};
pub use dev_repl::{DevRepl, ReplCommand, ReplResult};
pub use diff_comparison::{AstDiff, DiffReport, DiffType};
pub use graphviz_export::{EdgeStyle, GraphVizExporter, GraphVizOptions, NodeStyle};
pub use parser_validator::{ParserValidator, ValidationError, ValidationReport};
pub use performance_profiler::{MetricType, PerformanceProfiler, ProfilingReport};

/// Main development tools facade providing access to all utilities
pub struct DevTools {
    visualizer: AstVisualizer,
    validator: ParserValidator,
    profiler: PerformanceProfiler,
    exporter: GraphVizExporter,
}

impl DevTools {
    /// Create a new DevTools instance with default configuration
    pub fn new() -> Self {
        Self {
            visualizer: AstVisualizer::new(),
            validator: ParserValidator::new(),
            profiler: PerformanceProfiler::new(),
            exporter: GraphVizExporter::new(),
        }
    }

    /// Create a DevTools instance with custom configuration
    pub fn with_config(config: DevToolsConfig) -> Self {
        Self {
            visualizer: AstVisualizer::with_config(config.visualization),
            validator: ParserValidator::with_config(config.validation),
            profiler: PerformanceProfiler::with_config(config.profiling),
            exporter: GraphVizExporter::with_config(config.graphviz),
        }
    }

    /// Get access to the AST visualizer
    pub fn visualizer(&self) -> &AstVisualizer {
        &self.visualizer
    }

    /// Get access to the parser validator
    pub fn validator(&self) -> &ParserValidator {
        &self.validator
    }

    /// Get access to the performance profiler
    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }

    /// Get access to the GraphViz exporter
    pub fn exporter(&self) -> &GraphVizExporter {
        &self.exporter
    }

    /// Start an interactive development REPL
    pub async fn start_repl(&self, language: Option<&str>) -> Result<()> {
        let mut repl = DevRepl::new(language)?;
        repl.set_visualizer(self.visualizer.clone());
        repl.set_validator(self.validator.clone());
        repl.set_profiler(self.profiler.clone());
        repl.set_exporter(self.exporter.clone());
        repl.run().await
    }

    /// Perform a comprehensive analysis of a parse result
    pub fn analyze_parse_result(
        &self,
        parse_result: &codeprism_core::ParseResult,
        source: &str,
    ) -> Result<AnalysisReport> {
        let mut report = AnalysisReport::new();

        // Validate the parse result
        let validation = self.validator.validate_complete(parse_result, source)?;
        report.validation = Some(validation);

        // Generate AST visualization
        let visualization = self.visualizer.visualize_tree(&parse_result.tree, source)?;
        report.visualization = Some(visualization);

        // Export GraphViz if requested
        if !parse_result.nodes.is_empty() {
            let graphviz = self
                .exporter
                .export_nodes_and_edges(&parse_result.nodes, &parse_result.edges)?;
            report.graphviz = Some(graphviz);
        }

        Ok(report)
    }

    /// Compare two parse results and generate a diff report
    pub fn compare_parse_results(
        &self,
        old_result: &codeprism_core::ParseResult,
        new_result: &codeprism_core::ParseResult,
        source: &str,
    ) -> Result<DiffReport> {
        let diff = AstDiff::new();
        diff.compare(old_result, new_result, source)
    }
}

impl Default for DevTools {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for development tools
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DevToolsConfig {
    pub visualization: ast_visualizer::VisualizationConfig,
    pub validation: parser_validator::ValidationConfig,
    pub profiling: performance_profiler::ProfilingConfig,
    pub graphviz: graphviz_export::GraphVizConfig,
}


/// Comprehensive analysis report combining all dev tools outputs
#[derive(Debug)]
pub struct AnalysisReport {
    pub validation: Option<ValidationReport>,
    pub visualization: Option<String>,
    pub graphviz: Option<String>,
    pub profiling: Option<ProfilingReport>,
    pub diff: Option<DiffReport>,
}

impl AnalysisReport {
    pub fn new() -> Self {
        Self {
            validation: None,
            visualization: None,
            graphviz: None,
            profiling: None,
            diff: None,
        }
    }

    /// Check if the analysis indicates any issues
    pub fn has_issues(&self) -> bool {
        if let Some(validation) = &self.validation {
            if !validation.is_valid() {
                return true;
            }
        }
        false
    }

    /// Get a summary of all issues found
    pub fn issues_summary(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if let Some(validation) = &self.validation {
            if !validation.is_valid() {
                issues.extend(validation.errors().iter().map(|e| e.to_string()));
            }
        }

        issues
    }

    /// Generate a formatted report suitable for display
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("=== CodePrism Parser Analysis Report ===\n\n");

        if let Some(validation) = &self.validation {
            output.push_str("## Validation Results\n");
            if validation.is_valid() {
                output.push_str("✅ All validation checks passed\n");
            } else {
                output.push_str("❌ Validation errors found:\n");
                for error in validation.errors() {
                    output.push_str(&format!("  - {}\n", error));
                }
            }
            output.push('\n');
        }

        if let Some(visualization) = &self.visualization {
            output.push_str("## AST Visualization\n");
            output.push_str(visualization);
            output.push_str("\n\n");
        }

        if let Some(profiling) = &self.profiling {
            output.push_str("## Performance Metrics\n");
            output.push_str(&profiling.format_summary());
            output.push('\n');
        }

        if let Some(diff) = &self.diff {
            output.push_str("## AST Differences\n");
            output.push_str(&diff.format_report());
            output.push('\n');
        }

        output
    }
}

impl Default for AnalysisReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for development tools
#[derive(thiserror::Error, Debug)]
pub enum DevToolsError {
    #[error("Visualization error: {0}")]
    Visualization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("REPL error: {0}")]
    Repl(String),

    #[error("GraphViz export error: {0}")]
    GraphViz(String),

    #[error("Performance profiling error: {0}")]
    Profiling(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] codeprism_core::Error),
}

/// Utility functions for development tools
pub mod utils {
    use super::*;

    /// Load a source file and return its contents
    pub fn load_source_file<P: AsRef<Path>>(path: P) -> Result<String> {
        std::fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to load source file: {}", e))
    }

    /// Detect the language from a file extension
    pub fn detect_language_from_extension<P: AsRef<Path>>(path: P) -> Option<&'static str> {
        match path.as_ref().extension()?.to_str()? {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "mjs" => Some("javascript"),
            "ts" => Some("typescript"),
            "java" => Some("java"),
            _ => None,
        }
    }

    /// Format file size in human-readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format duration in human-readable format
    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_ms = duration.as_millis();

        if total_ms < 1000 {
            format!("{}ms", total_ms)
        } else if total_ms < 60_000 {
            format!("{:.2}s", duration.as_secs_f64())
        } else {
            let minutes = total_ms / 60_000;
            let seconds = (total_ms % 60_000) as f64 / 1000.0;
            format!("{}m {:.1}s", minutes, seconds)
        }
    }
}
