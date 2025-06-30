//! Performance monitoring and baseline management for the CodePrism Test Harness
//!
//! This module provides comprehensive performance tracking, baseline management,
//! and regression detection capabilities for test execution monitoring.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;
use tracing::{debug, info, warn};

/// Comprehensive performance metrics for a test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Test identifier
    pub test_id: String,
    /// Tool name being tested
    pub tool_name: String,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Peak memory usage in MB
    pub peak_memory_mb: f64,
    /// CPU usage percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Response payload size in bytes
    pub response_size_bytes: u64,
    /// Symbols processed per second (for analysis tools)
    pub throughput_symbols_per_sec: Option<f64>,
    /// Number of files processed
    pub files_processed: Option<u32>,
    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
    /// Additional custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(test_id: String, tool_name: String) -> Self {
        Self {
            test_id,
            tool_name,
            execution_time_ms: 0,
            peak_memory_mb: 0.0,
            cpu_usage_percent: 0.0,
            response_size_bytes: 0,
            throughput_symbols_per_sec: None,
            files_processed: None,
            timestamp: Utc::now(),
            custom_metrics: HashMap::new(),
        }
    }

    /// Calculate performance score (0.0 to 1.0, higher is better)
    pub fn calculate_performance_score(&self) -> f64 {
        // Simple scoring algorithm based on execution time and memory usage
        let time_score = (5000.0 / (self.execution_time_ms as f64 + 1000.0)).min(1.0);
        let memory_score = (256.0 / (self.peak_memory_mb + 64.0)).min(1.0);
        let cpu_score = (100.0 - self.cpu_usage_percent) / 100.0;

        (time_score * 0.5) + (memory_score * 0.3) + (cpu_score * 0.2)
    }
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Tool name this baseline applies to
    pub tool_name: String,
    /// Baseline execution time in milliseconds
    pub baseline_execution_time_ms: u64,
    /// Baseline memory usage in MB
    pub baseline_memory_mb: f64,
    /// Baseline CPU usage percentage
    pub baseline_cpu_percent: f64,
    /// Expected throughput range
    pub expected_throughput_range: Option<(f64, f64)>,
    /// When this baseline was established
    pub established_at: DateTime<Utc>,
    /// Version or git commit when baseline was set
    pub version: String,
    /// Number of samples used to establish baseline
    pub sample_count: u32,
    /// Standard deviation for metrics
    pub std_deviation: f64,
}

impl PerformanceBaseline {
    /// Create baseline from multiple metrics samples
    pub fn from_samples(
        tool_name: String,
        samples: &[PerformanceMetrics],
        version: String,
    ) -> Self {
        let sample_count = samples.len() as u32;
        if sample_count == 0 {
            return Self {
                tool_name,
                baseline_execution_time_ms: 5000, // Default 5 seconds
                baseline_memory_mb: 256.0,
                baseline_cpu_percent: 50.0,
                expected_throughput_range: None,
                established_at: Utc::now(),
                version,
                sample_count: 0,
                std_deviation: 0.0,
            };
        }

        let avg_time =
            samples.iter().map(|s| s.execution_time_ms).sum::<u64>() / sample_count as u64;
        let avg_memory =
            samples.iter().map(|s| s.peak_memory_mb).sum::<f64>() / sample_count as f64;
        let avg_cpu =
            samples.iter().map(|s| s.cpu_usage_percent).sum::<f64>() / sample_count as f64;

        // Calculate standard deviation for execution time
        let variance = samples
            .iter()
            .map(|s| {
                let diff = s.execution_time_ms as f64 - avg_time as f64;
                diff * diff
            })
            .sum::<f64>()
            / sample_count as f64;
        let std_deviation = variance.sqrt();

        // Calculate throughput range if available
        let throughput_samples: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.throughput_symbols_per_sec)
            .collect();
        let expected_throughput_range = if !throughput_samples.is_empty() {
            let min_throughput = throughput_samples
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
            let max_throughput = throughput_samples
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            Some((min_throughput * 0.8, max_throughput * 1.2)) // 20% tolerance
        } else {
            None
        };

        Self {
            tool_name,
            baseline_execution_time_ms: avg_time,
            baseline_memory_mb: avg_memory,
            baseline_cpu_percent: avg_cpu,
            expected_throughput_range,
            established_at: Utc::now(),
            version,
            sample_count,
            std_deviation,
        }
    }
}

/// Performance regression alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    /// Type of regression detected
    pub regression_type: RegressionType,
    /// Severity of the regression
    pub severity: AlertSeverity,
    /// Description of the regression
    pub message: String,
    /// Current value that triggered the alert
    pub current_value: f64,
    /// Expected baseline value
    pub baseline_value: f64,
    /// Percentage difference from baseline
    pub percentage_difference: f64,
    /// Threshold that was exceeded
    pub threshold_exceeded: f64,
}

/// Types of performance regressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    ExecutionTime,
    MemoryUsage,
    CpuUsage,
    Throughput,
    ResponseSize,
    Custom(String),
}

/// Severity levels for performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,     // Within normal variance
    Warning,  // Noticeable degradation
    Error,    // Significant regression
    Critical, // Severe performance loss
}

/// Configuration for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enabled: bool,
    /// Directory to store performance baselines
    pub baseline_dir: PathBuf,
    /// Regression thresholds as percentage increase
    pub regression_thresholds: RegressionThresholds,
    /// Number of historical runs to keep
    pub history_retention_count: usize,
}

/// Regression detection thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionThresholds {
    /// Warning threshold for execution time (% increase)
    pub execution_time_warning_percent: f64,
    /// Error threshold for execution time (% increase)
    pub execution_time_error_percent: f64,
    /// Warning threshold for memory usage (% increase)
    pub memory_warning_percent: f64,
    /// Error threshold for memory usage (% increase)
    pub memory_error_percent: f64,
    /// Threshold for throughput decrease (% decrease)
    pub throughput_degradation_percent: f64,
}

impl Default for RegressionThresholds {
    fn default() -> Self {
        Self {
            execution_time_warning_percent: 20.0,
            execution_time_error_percent: 50.0,
            memory_warning_percent: 30.0,
            memory_error_percent: 100.0,
            throughput_degradation_percent: 25.0,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            baseline_dir: PathBuf::from("performance-baselines"),
            regression_thresholds: RegressionThresholds::default(),
            history_retention_count: 100,
        }
    }
}

/// Performance baseline manager
pub struct BaselineManager {
    config: PerformanceConfig,
    baselines: HashMap<String, PerformanceBaseline>,
    history: Vec<PerformanceMetrics>,
}

impl BaselineManager {
    /// Create new baseline manager
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            baselines: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Load baselines from disk
    pub async fn load_baselines(&mut self) -> Result<()> {
        if !self.config.baseline_dir.exists() {
            fs::create_dir_all(&self.config.baseline_dir).await?;
            return Ok(());
        }

        let mut dir_entries = fs::read_dir(&self.config.baseline_dir).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                if let Some(tool_name) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_baseline_file(&path).await {
                        Ok(baseline) => {
                            debug!("Loaded baseline for tool: {}", tool_name);
                            self.baselines.insert(tool_name.to_string(), baseline);
                        }
                        Err(e) => {
                            warn!("Failed to load baseline from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Loaded {} performance baselines", self.baselines.len());
        Ok(())
    }

    /// Load a single baseline file
    async fn load_baseline_file(&self, path: &Path) -> Result<PerformanceBaseline> {
        let content = fs::read_to_string(path).await?;
        let baseline: PerformanceBaseline = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    /// Save baseline for a tool
    pub async fn save_baseline(&mut self, baseline: PerformanceBaseline) -> Result<()> {
        let tool_name = baseline.tool_name.clone();
        let baseline_path = self.config.baseline_dir.join(format!("{}.json", tool_name));

        let content = serde_json::to_string_pretty(&baseline)?;
        fs::write(&baseline_path, content).await?;

        self.baselines.insert(tool_name.clone(), baseline);
        info!("Saved baseline for tool: {}", tool_name);
        Ok(())
    }

    /// Get baseline for a tool
    pub fn get_baseline(&self, tool_name: &str) -> Option<&PerformanceBaseline> {
        self.baselines.get(tool_name)
    }

    /// Add metrics to history and maintain retention limit
    pub fn add_to_history(&mut self, metrics: PerformanceMetrics) {
        self.history.push(metrics);

        // Maintain history retention limit
        if self.history.len() > self.config.history_retention_count {
            self.history.remove(0);
        }
    }

    /// Get performance history for a tool
    pub fn get_tool_history(&self, tool_name: &str) -> Vec<&PerformanceMetrics> {
        self.history
            .iter()
            .filter(|m| m.tool_name == tool_name)
            .collect()
    }
}

/// Regression detector for performance analysis
pub struct RegressionDetector {
    config: PerformanceConfig,
}

impl RegressionDetector {
    /// Create new regression detector
    pub fn new(config: PerformanceConfig) -> Self {
        Self { config }
    }

    /// Detect regressions by comparing metrics against baseline
    pub fn detect_regressions(
        &self,
        metrics: &PerformanceMetrics,
        baseline: &PerformanceBaseline,
    ) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();

        // Check execution time regression
        let time_increase = calculate_percentage_increase(
            metrics.execution_time_ms as f64,
            baseline.baseline_execution_time_ms as f64,
        );

        if time_increase
            >= self
                .config
                .regression_thresholds
                .execution_time_error_percent
        {
            alerts.push(RegressionAlert {
                regression_type: RegressionType::ExecutionTime,
                severity: AlertSeverity::Error,
                message: format!("Execution time increased by {:.1}%", time_increase),
                current_value: metrics.execution_time_ms as f64,
                baseline_value: baseline.baseline_execution_time_ms as f64,
                percentage_difference: time_increase,
                threshold_exceeded: self
                    .config
                    .regression_thresholds
                    .execution_time_error_percent,
            });
        } else if time_increase
            >= self
                .config
                .regression_thresholds
                .execution_time_warning_percent
        {
            alerts.push(RegressionAlert {
                regression_type: RegressionType::ExecutionTime,
                severity: AlertSeverity::Warning,
                message: format!("Execution time increased by {:.1}%", time_increase),
                current_value: metrics.execution_time_ms as f64,
                baseline_value: baseline.baseline_execution_time_ms as f64,
                percentage_difference: time_increase,
                threshold_exceeded: self
                    .config
                    .regression_thresholds
                    .execution_time_warning_percent,
            });
        }

        // Check memory usage regression
        let memory_increase =
            calculate_percentage_increase(metrics.peak_memory_mb, baseline.baseline_memory_mb);

        if memory_increase >= self.config.regression_thresholds.memory_error_percent {
            alerts.push(RegressionAlert {
                regression_type: RegressionType::MemoryUsage,
                severity: AlertSeverity::Error,
                message: format!("Memory usage increased by {:.1}%", memory_increase),
                current_value: metrics.peak_memory_mb,
                baseline_value: baseline.baseline_memory_mb,
                percentage_difference: memory_increase,
                threshold_exceeded: self.config.regression_thresholds.memory_error_percent,
            });
        } else if memory_increase >= self.config.regression_thresholds.memory_warning_percent {
            alerts.push(RegressionAlert {
                regression_type: RegressionType::MemoryUsage,
                severity: AlertSeverity::Warning,
                message: format!("Memory usage increased by {:.1}%", memory_increase),
                current_value: metrics.peak_memory_mb,
                baseline_value: baseline.baseline_memory_mb,
                percentage_difference: memory_increase,
                threshold_exceeded: self.config.regression_thresholds.memory_warning_percent,
            });
        }

        // Check throughput degradation
        if let (Some(current_throughput), Some((min_expected, _max_expected))) = (
            metrics.throughput_symbols_per_sec,
            baseline.expected_throughput_range,
        ) {
            if current_throughput < min_expected {
                let degradation = ((min_expected - current_throughput) / min_expected) * 100.0;
                if degradation
                    >= self
                        .config
                        .regression_thresholds
                        .throughput_degradation_percent
                {
                    alerts.push(RegressionAlert {
                        regression_type: RegressionType::Throughput,
                        severity: AlertSeverity::Warning,
                        message: format!("Throughput decreased by {:.1}%", degradation),
                        current_value: current_throughput,
                        baseline_value: min_expected,
                        percentage_difference: degradation,
                        threshold_exceeded: self
                            .config
                            .regression_thresholds
                            .throughput_degradation_percent,
                    });
                }
            }
        }

        alerts
    }
}

/// Calculate percentage increase (positive values indicate increase)
fn calculate_percentage_increase(current: f64, baseline: f64) -> f64 {
    if baseline == 0.0 {
        return 0.0;
    }
    ((current - baseline) / baseline) * 100.0
}

/// Main performance monitoring system
pub struct PerformanceMonitor {
    baseline_manager: BaselineManager,
    regression_detector: RegressionDetector,
    config: PerformanceConfig,
    current_metrics: Option<PerformanceMetrics>,
    start_time: Option<Instant>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        let baseline_manager = BaselineManager::new(config.clone());
        let regression_detector = RegressionDetector::new(config.clone());

        Self {
            baseline_manager,
            regression_detector,
            config,
            current_metrics: None,
            start_time: None,
        }
    }

    /// Initialize performance monitor (load baselines)
    pub async fn initialize(&mut self) -> Result<()> {
        if self.config.enabled {
            self.baseline_manager.load_baselines().await?;
            info!("Performance monitoring initialized");
        }
        Ok(())
    }

    /// Start monitoring a test
    pub fn start_test_monitoring(&mut self, test_id: String, tool_name: String) {
        if !self.config.enabled {
            return;
        }

        self.current_metrics = Some(PerformanceMetrics::new(test_id, tool_name));
        self.start_time = Some(Instant::now());
        debug!(
            "Started performance monitoring for test: {}",
            self.current_metrics.as_ref().unwrap().test_id
        );
    }

    /// Update current metrics during test execution
    pub fn update_metrics<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut PerformanceMetrics),
    {
        if let Some(ref mut metrics) = self.current_metrics {
            updater(metrics);
        }
    }

    /// Finish monitoring and return results
    pub async fn finish_test_monitoring(
        &mut self,
        _version: String,
    ) -> Result<Option<PerformanceResult>> {
        if !self.config.enabled || self.current_metrics.is_none() {
            return Ok(None);
        }

        let mut metrics = self.current_metrics.take().unwrap();

        // Calculate final execution time
        if let Some(start_time) = self.start_time.take() {
            metrics.execution_time_ms = start_time.elapsed().as_millis() as u64;
        }

        // Calculate performance score
        let performance_score = metrics.calculate_performance_score();

        // Get baseline for comparison
        let baseline = self.baseline_manager.get_baseline(&metrics.tool_name);

        // Detect regressions
        let regression_alerts = if let Some(baseline) = baseline {
            self.regression_detector
                .detect_regressions(&metrics, baseline)
        } else {
            Vec::new()
        };

        // Add to history
        self.baseline_manager.add_to_history(metrics.clone());

        let result = PerformanceResult {
            metrics,
            regression_alerts,
            baseline_updated: false, // Could implement auto-update logic
            performance_score,
        };

        Ok(Some(result))
    }

    /// Get baseline for a tool
    pub fn get_baseline(&self, tool_name: &str) -> Option<&PerformanceBaseline> {
        self.baseline_manager.get_baseline(tool_name)
    }

    /// Manually set baseline for a tool
    pub async fn set_baseline(&mut self, baseline: PerformanceBaseline) -> Result<()> {
        self.baseline_manager.save_baseline(baseline).await
    }
}

/// Result of performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    /// Collected performance metrics
    pub metrics: PerformanceMetrics,
    /// Any regression alerts detected
    pub regression_alerts: Vec<RegressionAlert>,
    /// Whether baseline was updated
    pub baseline_updated: bool,
    /// Overall performance score (0.0 to 1.0)
    pub performance_score: f64,
}

impl PerformanceResult {
    /// Check if there are any error-level regressions
    pub fn has_error_regressions(&self) -> bool {
        self.regression_alerts.iter().any(|alert| {
            matches!(
                alert.severity,
                AlertSeverity::Error | AlertSeverity::Critical
            )
        })
    }

    /// Get summary of performance issues
    pub fn get_summary(&self) -> String {
        if self.regression_alerts.is_empty() {
            format!(
                "Performance: {}ms execution, {:.1}MB memory (score: {:.2})",
                self.metrics.execution_time_ms, self.metrics.peak_memory_mb, self.performance_score
            )
        } else {
            let error_count = self
                .regression_alerts
                .iter()
                .filter(|a| matches!(a.severity, AlertSeverity::Error | AlertSeverity::Critical))
                .count();
            let warning_count = self
                .regression_alerts
                .iter()
                .filter(|a| matches!(a.severity, AlertSeverity::Warning))
                .count();

            format!(
                "Performance issues detected: {} errors, {} warnings",
                error_count, warning_count
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new("test_1".to_string(), "repository_stats".to_string());
        assert_eq!(metrics.test_id, "test_1");
        assert_eq!(metrics.tool_name, "repository_stats");
        assert_eq!(metrics.execution_time_ms, 0);
    }

    #[test]
    fn test_performance_score_calculation() {
        let mut metrics = PerformanceMetrics::new("test_1".to_string(), "test_tool".to_string());
        metrics.execution_time_ms = 1000; // 1 second
        metrics.peak_memory_mb = 128.0;
        metrics.cpu_usage_percent = 50.0;

        let score = metrics.calculate_performance_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_baseline_from_samples() {
        let samples = vec![
            {
                let mut m = PerformanceMetrics::new("test_1".to_string(), "test_tool".to_string());
                m.execution_time_ms = 1000;
                m.peak_memory_mb = 100.0;
                m
            },
            {
                let mut m = PerformanceMetrics::new("test_2".to_string(), "test_tool".to_string());
                m.execution_time_ms = 1200;
                m.peak_memory_mb = 120.0;
                m
            },
        ];

        let baseline = PerformanceBaseline::from_samples(
            "test_tool".to_string(),
            &samples,
            "v1.0.0".to_string(),
        );
        assert_eq!(baseline.baseline_execution_time_ms, 1100); // Average of 1000 and 1200
        assert_eq!(baseline.sample_count, 2);
    }

    #[test]
    fn test_regression_detection() {
        let config = PerformanceConfig::default();
        let detector = RegressionDetector::new(config);

        let mut baseline =
            PerformanceBaseline::from_samples("test_tool".to_string(), &[], "v1.0.0".to_string());
        baseline.baseline_execution_time_ms = 1000;
        baseline.baseline_memory_mb = 100.0;

        let mut metrics = PerformanceMetrics::new("test_1".to_string(), "test_tool".to_string());
        metrics.execution_time_ms = 1300; // 30% increase - should trigger warning
        metrics.peak_memory_mb = 100.0;

        let alerts = detector.detect_regressions(&metrics, &baseline);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(
            alerts[0].regression_type,
            RegressionType::ExecutionTime
        ));
        assert!(matches!(alerts[0].severity, AlertSeverity::Warning));
    }

    #[tokio::test]
    async fn test_baseline_manager() {
        let temp_dir = tempdir().unwrap();
        let config = PerformanceConfig {
            baseline_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = BaselineManager::new(config);
        manager.load_baselines().await.unwrap();

        let baseline =
            PerformanceBaseline::from_samples("test_tool".to_string(), &[], "v1.0.0".to_string());
        manager.save_baseline(baseline).await.unwrap();

        assert!(manager.get_baseline("test_tool").is_some());
    }

    #[tokio::test]
    async fn test_performance_monitor_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let config = PerformanceConfig {
            baseline_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut monitor = PerformanceMonitor::new(config);
        monitor.initialize().await.unwrap();

        monitor.start_test_monitoring("test_1".to_string(), "test_tool".to_string());

        monitor.update_metrics(|metrics| {
            metrics.peak_memory_mb = 128.0;
            metrics.cpu_usage_percent = 50.0;
        });

        let result = monitor
            .finish_test_monitoring("v1.0.0".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.metrics.test_id, "test_1");
        assert_eq!(result.metrics.peak_memory_mb, 128.0);
    }

    #[test]
    fn test_percentage_increase_calculation() {
        assert_eq!(calculate_percentage_increase(120.0, 100.0), 20.0);
        assert_eq!(calculate_percentage_increase(80.0, 100.0), -20.0);
        assert_eq!(calculate_percentage_increase(100.0, 100.0), 0.0);
    }
}
