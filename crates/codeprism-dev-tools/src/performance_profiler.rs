//! Performance profiling utilities for parser development

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance profiler for tracking parsing metrics
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    config: ProfilingConfig,
    metrics: Vec<PerformanceMetric>,
}

/// Configuration for performance profiling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub track_memory_usage: bool,
    pub track_parse_time: bool,
    pub track_node_creation: bool,
    pub track_edge_creation: bool,
    pub sample_interval_ms: u64,
    pub max_samples: usize,
    pub enable_detailed_timing: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            track_memory_usage: true,
            track_parse_time: true,
            track_node_creation: true,
            track_edge_creation: true,
            sample_interval_ms: 100,
            max_samples: 1000,
            enable_detailed_timing: false,
        }
    }
}

/// Type of performance metric
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MetricType {
    ParseTime,
    MemoryUsage,
    NodeCreation,
    EdgeCreation,
    FileSize,
    TreeDepth,
    NodeCount,
    EdgeCount,
    ValidationTime,
    VisualizationTime,
}

/// Individual performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub timestamp: Duration,
    pub context: Option<String>,
}

/// Comprehensive profiling report
#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub metrics: Vec<PerformanceMetric>,
    pub summary: PerformanceSummary,
    pub analysis: PerformanceAnalysis,
    pub recommendations: Vec<String>,
}

/// Summary statistics for all metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceSummary {
    pub total_parse_time_ms: f64,
    pub average_parse_time_ms: f64,
    pub peak_memory_usage_mb: f64,
    pub total_nodes_created: u64,
    pub total_edges_created: u64,
    pub parse_throughput_kb_per_sec: f64,
    pub nodes_per_second: f64,
    pub validation_overhead_percent: f64,
}

/// Performance analysis results
#[derive(Debug, Clone, Default)]
pub struct PerformanceAnalysis {
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub trends: Vec<PerformanceTrend>,
    pub comparisons: Vec<PerformanceComparison>,
    pub efficiency_score: f64,
}

/// Identified performance bottleneck
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub area: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact_percent: f64,
    pub suggestion: String,
}

/// Severity of performance bottleneck
#[derive(Debug, Clone, Copy)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance trend over time
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    pub metric_type: MetricType,
    pub trend_direction: TrendDirection,
    pub rate_of_change: f64,
    pub confidence: f64,
}

/// Direction of performance trend
#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

/// Performance comparison between different conditions
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    pub description: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub change_percent: f64,
    pub is_improvement: bool,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            config: ProfilingConfig::default(),
            metrics: Vec::new(),
        }
    }

    /// Create a profiler with custom configuration
    pub fn with_config(config: ProfilingConfig) -> Self {
        Self {
            config,
            metrics: Vec::new(),
        }
    }

    /// Start a new profiling session
    pub fn start_session(&mut self) {
        self.metrics.clear();
        self.record_metric(
            MetricType::ParseTime,
            0.0,
            "ms",
            Some("Session started".to_string()),
        );
    }

    /// Record a performance metric
    pub fn record_metric(
        &mut self,
        metric_type: MetricType,
        value: f64,
        unit: &str,
        context: Option<String>,
    ) {
        if self.metrics.len() >= self.config.max_samples {
            // Remove oldest metric to maintain size limit
            self.metrics.remove(0);
        }

        let metric = PerformanceMetric {
            metric_type,
            value,
            unit: unit.to_string(),
            timestamp: self.get_session_time(),
            context,
        };

        self.metrics.push(metric);
    }

    /// Time a parsing operation
    pub fn time_parse_operation<F, R>(&mut self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        let start_time = Instant::now();
        let result = operation()?;
        let elapsed = start_time.elapsed();

        self.record_metric(
            MetricType::ParseTime,
            elapsed.as_millis() as f64,
            "ms",
            Some("Parse operation".to_string()),
        );

        Ok(result)
    }

    /// Profile memory usage (simplified - would need real memory tracking in production)
    pub fn profile_memory_usage(&mut self) {
        // Simplified memory profiling
        // In a real implementation, this would use system APIs to get actual memory usage
        let estimated_memory_mb = self.estimate_memory_usage();

        self.record_metric(
            MetricType::MemoryUsage,
            estimated_memory_mb,
            "MB",
            Some("Memory snapshot".to_string()),
        );
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> f64 {
        // This is a simplified estimation
        // In production, you'd use proper memory profiling tools
        let metrics_size = self.metrics.len() * std::mem::size_of::<PerformanceMetric>();
        metrics_size as f64 / (1024.0 * 1024.0) // Convert to MB
    }

    /// Record node creation metrics
    pub fn record_node_creation(&mut self, count: u64) {
        if self.config.track_node_creation {
            self.record_metric(
                MetricType::NodeCreation,
                count as f64,
                "nodes",
                Some("Nodes created".to_string()),
            );
        }
    }

    /// Record edge creation metrics
    pub fn record_edge_creation(&mut self, count: u64) {
        if self.config.track_edge_creation {
            self.record_metric(
                MetricType::EdgeCreation,
                count as f64,
                "edges",
                Some("Edges created".to_string()),
            );
        }
    }

    /// Record file size being parsed
    pub fn record_file_size(&mut self, size_bytes: u64) {
        self.record_metric(
            MetricType::FileSize,
            size_bytes as f64 / 1024.0, // Convert to KB
            "KB",
            Some("File size".to_string()),
        );
    }

    /// Generate a comprehensive profiling report
    pub fn generate_report(&self) -> ProfilingReport {
        let summary = self.calculate_summary();
        let analysis = self.analyze_performance();
        let recommendations = self.generate_recommendations(&analysis);

        ProfilingReport {
            metrics: self.metrics.clone(),
            summary,
            analysis,
            recommendations,
        }
    }

    /// Calculate summary statistics
    fn calculate_summary(&self) -> PerformanceSummary {
        let mut summary = PerformanceSummary::default();

        let parse_times: Vec<f64> = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::ParseTime))
            .map(|m| m.value)
            .collect();

        if !parse_times.is_empty() {
            summary.total_parse_time_ms = parse_times.iter().sum();
            summary.average_parse_time_ms = summary.total_parse_time_ms / parse_times.len() as f64;
        }

        // Peak memory usage
        summary.peak_memory_usage_mb = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::MemoryUsage))
            .map(|m| m.value)
            .fold(0.0, f64::max);

        // Total nodes and edges
        summary.total_nodes_created = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::NodeCreation))
            .map(|m| m.value as u64)
            .sum();

        summary.total_edges_created = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::EdgeCreation))
            .map(|m| m.value as u64)
            .sum();

        // Calculate throughput
        let total_file_size_kb: f64 = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::FileSize))
            .map(|m| m.value)
            .sum();

        if summary.total_parse_time_ms > 0.0 {
            summary.parse_throughput_kb_per_sec =
                total_file_size_kb / (summary.total_parse_time_ms / 1000.0);
            summary.nodes_per_second =
                summary.total_nodes_created as f64 / (summary.total_parse_time_ms / 1000.0);
        }

        summary
    }

    /// Analyze performance for bottlenecks and trends
    fn analyze_performance(&self) -> PerformanceAnalysis {
        PerformanceAnalysis {
            bottlenecks: self.identify_bottlenecks(),
            trends: self.analyze_trends(),
            efficiency_score: self.calculate_efficiency_score(),
            ..Default::default()
        }
    }

    /// Identify performance bottlenecks
    fn identify_bottlenecks(&self) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // Check for slow parsing
        let avg_parse_time = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::ParseTime))
            .map(|m| m.value)
            .sum::<f64>()
            / self.metrics.len().max(1) as f64;

        if avg_parse_time > 1000.0 {
            // More than 1 second
            bottlenecks.push(PerformanceBottleneck {
                area: "Parse Time".to_string(),
                severity: BottleneckSeverity::High,
                description: format!("Average parse time is {avg_parse_time:.1}ms"),
                impact_percent: 80.0,
                suggestion: "Consider optimizing parser grammar or using incremental parsing"
                    .to_string(),
            });
        }

        // Check for high memory usage
        let peak_memory = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::MemoryUsage))
            .map(|m| m.value)
            .fold(0.0, f64::max);

        if peak_memory > 100.0 {
            // More than 100 MB
            bottlenecks.push(PerformanceBottleneck {
                area: "Memory Usage".to_string(),
                severity: BottleneckSeverity::Medium,
                description: format!("Peak memory usage is {peak_memory:.1}MB"),
                impact_percent: 40.0,
                suggestion: "Consider streaming parsing or memory pooling".to_string(),
            });
        }

        bottlenecks
    }

    /// Analyze performance trends
    fn analyze_trends(&self) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();

        // Analyze parse time trend
        let parse_times: Vec<f64> = self
            .metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::ParseTime))
            .map(|m| m.value)
            .collect();

        if parse_times.len() >= 3 {
            let trend = self.calculate_trend(&parse_times);
            trends.push(PerformanceTrend {
                metric_type: MetricType::ParseTime,
                trend_direction: trend.0,
                rate_of_change: trend.1,
                confidence: 0.8,
            });
        }

        trends
    }

    /// Calculate trend direction and rate for a series of values
    fn calculate_trend(&self, values: &[f64]) -> (TrendDirection, f64) {
        if values.len() < 2 {
            return (TrendDirection::Stable, 0.0);
        }

        let first_half_avg =
            values[..values.len() / 2].iter().sum::<f64>() / (values.len() / 2) as f64;
        let second_half_avg = values[values.len() / 2..].iter().sum::<f64>()
            / (values.len() - values.len() / 2) as f64;

        let change_percent = ((second_half_avg - first_half_avg) / first_half_avg) * 100.0;

        let direction = if change_percent > 5.0 {
            TrendDirection::Degrading // Higher values are usually worse for performance
        } else if change_percent < -5.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        (direction, change_percent.abs())
    }

    /// Calculate overall efficiency score (0-100)
    fn calculate_efficiency_score(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Deduct points for bottlenecks
        let bottlenecks = self.identify_bottlenecks();
        for bottleneck in bottlenecks {
            let deduction = match bottleneck.severity {
                BottleneckSeverity::Critical => 30.0,
                BottleneckSeverity::High => 20.0,
                BottleneckSeverity::Medium => 10.0,
                BottleneckSeverity::Low => 5.0,
            };
            score -= deduction;
        }

        score.max(0.0)
    }

    /// Generate performance recommendations
    fn generate_recommendations(&self, analysis: &PerformanceAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if analysis.efficiency_score < 70.0 {
            recommendations.push(
                "Overall performance needs improvement. Consider profiling specific operations."
                    .to_string(),
            );
        }

        for bottleneck in &analysis.bottlenecks {
            recommendations.push(format!("{}: {}", bottleneck.area, bottleneck.suggestion));
        }

        for trend in &analysis.trends {
            if matches!(trend.trend_direction, TrendDirection::Degrading) {
                recommendations.push(format!(
                    "Performance degradation detected in {:?}. Monitor and optimize.",
                    trend.metric_type
                ));
            }
        }

        if recommendations.is_empty() {
            recommendations
                .push("Performance looks good! Continue monitoring for regressions.".to_string());
        }

        recommendations
    }

    /// Get session time since profiling started
    fn get_session_time(&self) -> Duration {
        // Simplified - in reality, you'd track session start time
        Duration::from_millis(self.metrics.len() as u64 * 10)
    }

    /// Export metrics to CSV format
    pub fn export_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("timestamp_ms,metric_type,value,unit,context\n");

        for metric in &self.metrics {
            csv.push_str(&format!(
                "{},{:?},{},{},{}\n",
                metric.timestamp.as_millis(),
                metric.metric_type,
                metric.value,
                metric.unit,
                metric.context.as_deref().unwrap_or("")
            ));
        }

        csv
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilingReport {
    /// Format a summary of the profiling report
    pub fn format_summary(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Performance Profile Summary ===\n");
        output.push_str(&format!(
            "Total Parse Time: {:.1}ms\n",
            self.summary.total_parse_time_ms
        ));
        output.push_str(&format!(
            "Average Parse Time: {:.1}ms\n",
            self.summary.average_parse_time_ms
        ));
        output.push_str(&format!(
            "Peak Memory Usage: {:.1}MB\n",
            self.summary.peak_memory_usage_mb
        ));
        output.push_str(&format!(
            "Nodes Created: {}\n",
            self.summary.total_nodes_created
        ));
        output.push_str(&format!(
            "Edges Created: {}\n",
            self.summary.total_edges_created
        ));
        output.push_str(&format!(
            "Parse Throughput: {:.1} KB/s\n",
            self.summary.parse_throughput_kb_per_sec
        ));
        output.push_str(&format!(
            "Efficiency Score: {:.1}/100\n",
            self.analysis.efficiency_score
        ));

        if !self.analysis.bottlenecks.is_empty() {
            output.push_str("\n## Performance Issues:\n");
            for bottleneck in &self.analysis.bottlenecks {
                output.push_str(&format!(
                    "- {}: {}\n",
                    bottleneck.area, bottleneck.description
                ));
            }
        }

        if !self.recommendations.is_empty() {
            output.push_str("\n## Recommendations:\n");
            for rec in &self.recommendations {
                output.push_str(&format!("- {rec}\n"));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        assert!(profiler.config.track_parse_time);
        assert!(profiler.config.track_memory_usage);
        assert_eq!(profiler.metrics.len(), 0);
    }

    #[test]
    fn test_record_metric() {
        let mut profiler = PerformanceProfiler::new();
        profiler.record_metric(MetricType::ParseTime, 100.0, "ms", None);

        assert_eq!(profiler.metrics.len(), 1);
        assert_eq!(profiler.metrics[0].value, 100.0);
        assert_eq!(profiler.metrics[0].unit, "ms");
    }

    #[test]
    fn test_generate_report() {
        let mut profiler = PerformanceProfiler::new();
        profiler.record_metric(MetricType::ParseTime, 50.0, "ms", None);
        profiler.record_metric(MetricType::MemoryUsage, 10.0, "MB", None);

        let report = profiler.generate_report();
        assert_eq!(report.summary.total_parse_time_ms, 50.0);
        assert_eq!(report.summary.peak_memory_usage_mb, 10.0);
    }

    #[test]
    fn test_efficiency_score_calculation() {
        let profiler = PerformanceProfiler::new();
        let score = profiler.calculate_efficiency_score();
        assert_eq!(score, 100.0); // No metrics, perfect score
    }
}
