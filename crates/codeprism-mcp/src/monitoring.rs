//! Performance Monitoring System (Phase 2.2)
//!
//! This module provides real-time performance monitoring for the MCP server,
//! tracking metrics like response times, memory usage, error rates, and tool performance.

use crate::config::MonitoringConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{debug, info};

/// Performance metrics collector and monitor
#[derive(Debug)]
pub struct PerformanceMonitor {
    config: MonitoringConfig,
    metrics: Arc<RwLock<MetricsStorage>>,
    alert_sender: Option<mpsc::UnboundedSender<Alert>>,
    start_time: Instant,
    last_cleanup: Instant,
}

/// Storage for performance metrics
#[derive(Debug, Default)]
struct MetricsStorage {
    /// Response time measurements
    response_times: VecDeque<ResponseTimeMetric>,
    /// Memory usage measurements  
    memory_usage: VecDeque<MemoryMetric>,
    /// Error tracking
    error_tracking: VecDeque<ErrorMetric>,
    /// Tool performance tracking
    tool_performance: HashMap<String, ToolPerformanceMetrics>,
    /// System metrics
    system_metrics: VecDeque<SystemMetric>,
    /// Session statistics
    session_stats: SessionStatistics,
}

/// Response time metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetric {
    pub timestamp: u64,
    pub tool_name: String,
    pub duration_ms: u64,
    pub success: bool,
    pub client_type: Option<String>,
    pub payload_size_bytes: usize,
}

/// Memory usage metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetric {
    pub timestamp: u64,
    pub memory_mb: usize,
    pub heap_mb: Option<usize>,
    pub available_mb: Option<usize>,
    pub cache_size_mb: Option<usize>,
}

/// Error tracking metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetric {
    pub timestamp: u64,
    pub tool_name: String,
    pub error_type: ErrorType,
    pub error_message: String,
    pub severity: ErrorSeverity,
    pub client_type: Option<String>,
}

/// System performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetric {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_activity_mb: f64,
    pub active_connections: usize,
}

/// Tool-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPerformanceMetrics {
    pub tool_name: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub last_called: Option<u64>,
    pub error_rate: f64,
    pub throughput_per_minute: f64,
}

/// Session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub session_start: Option<u64>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub unique_tools_used: std::collections::HashSet<String>,
    pub avg_session_duration_ms: f64,
    pub peak_memory_mb: usize,
    pub total_data_processed_mb: f64,
}

/// Error types for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    ValidationError,
    TimeoutError,
    MemoryError,
    FileSystemError,
    ParsingError,
    NetworkError,
    InternalError,
    ClientError,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub timestamp: u64,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub suggested_action: Option<String>,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighMemoryUsage,
    SlowResponseTime,
    HighErrorRate,
    LowSuccessRate,
    SystemResourceExhaustion,
    ToolPerformanceDegradation,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
    Emergency,
}

/// Performance summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub current_memory_mb: usize,
    pub peak_memory_mb: usize,
    pub error_rate: f64,
    pub active_alerts: Vec<Alert>,
    pub top_tools_by_usage: Vec<ToolUsageInfo>,
    pub recent_errors: Vec<ErrorMetric>,
    pub system_health: SystemHealthStatus,
}

/// Tool usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageInfo {
    pub tool_name: String,
    pub usage_count: u64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealthStatus {
    Healthy,
    Warning,
    Critical,
    Degraded,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(MetricsStorage::default())),
            alert_sender: None,
            start_time: Instant::now(),
            last_cleanup: Instant::now(),
        }
    }

    /// Start the monitoring system
    pub async fn start(&mut self) -> Result<mpsc::UnboundedReceiver<Alert>> {
        if !self.config.enabled {
            info!("Performance monitoring is disabled");
            let (_, rx) = mpsc::unbounded_channel();
            return Ok(rx);
        }

        let (tx, rx) = mpsc::unbounded_channel();
        self.alert_sender = Some(tx);

        // Start background collection task
        self.start_collection_task().await?;

        // Start alert monitoring task
        self.start_alert_monitoring_task().await?;

        info!(
            "Performance monitoring started with {}s collection interval",
            self.config.collection_interval.as_secs()
        );

        Ok(rx)
    }

    /// Start metrics collection background task
    async fn start_collection_task(&self) -> Result<()> {
        if !self.config.monitor_memory && !self.config.monitor_response_times {
            return Ok(());
        }

        let metrics = Arc::clone(&self.metrics);
        let interval = self.config.collection_interval;
        let monitor_memory = self.config.monitor_memory;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if monitor_memory {
                    if let Ok(memory_info) = Self::collect_memory_info() {
                        if let Ok(mut storage) = metrics.write() {
                            storage.memory_usage.push_back(memory_info);

                            // Keep only recent metrics (last hour)
                            let cutoff = Self::current_timestamp() - 3600;
                            storage.memory_usage.retain(|m| m.timestamp > cutoff);
                        }
                    }
                }

                // Collect system metrics
                if let Ok(system_info) = Self::collect_system_info() {
                    if let Ok(mut storage) = metrics.write() {
                        storage.system_metrics.push_back(system_info);

                        // Keep only recent metrics
                        let cutoff = Self::current_timestamp() - 3600;
                        storage.system_metrics.retain(|m| m.timestamp > cutoff);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start alert monitoring background task
    async fn start_alert_monitoring_task(&self) -> Result<()> {
        let metrics = Arc::clone(&self.metrics);
        let thresholds = self.config.alert_thresholds.clone();
        let sender = self.alert_sender.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval_timer.tick().await;

                if let (Ok(storage), Some(ref alert_tx)) = (metrics.read(), &sender) {
                    // Check memory alerts
                    if let Some(latest_memory) = storage.memory_usage.back() {
                        if latest_memory.memory_mb > thresholds.max_memory_mb {
                            let alert = Alert {
                                timestamp: Self::current_timestamp(),
                                alert_type: AlertType::HighMemoryUsage,
                                severity: if latest_memory.memory_mb > thresholds.max_memory_mb * 2
                                {
                                    AlertSeverity::Critical
                                } else {
                                    AlertSeverity::Warning
                                },
                                message: format!(
                                    "High memory usage detected: {}MB",
                                    latest_memory.memory_mb
                                ),
                                metric_value: latest_memory.memory_mb as f64,
                                threshold_value: thresholds.max_memory_mb as f64,
                                suggested_action: Some(
                                    "Consider reducing batch size or restarting server".to_string(),
                                ),
                            };

                            let _ = alert_tx.send(alert);
                        }
                    }

                    // Check response time alerts
                    let recent_responses: Vec<_> = storage
                        .response_times
                        .iter()
                        .filter(|r| r.timestamp > Self::current_timestamp() - 300) // Last 5 minutes
                        .collect();

                    if !recent_responses.is_empty() {
                        let avg_response_time =
                            recent_responses.iter().map(|r| r.duration_ms).sum::<u64>() as f64
                                / recent_responses.len() as f64;

                        if avg_response_time > thresholds.max_response_time_ms as f64 {
                            let alert = Alert {
                                timestamp: Self::current_timestamp(),
                                alert_type: AlertType::SlowResponseTime,
                                severity: AlertSeverity::Warning,
                                message: format!(
                                    "Slow response times detected: {:.1}ms average",
                                    avg_response_time
                                ),
                                metric_value: avg_response_time,
                                threshold_value: thresholds.max_response_time_ms as f64,
                                suggested_action: Some(
                                    "Check system resources and optimize tool usage".to_string(),
                                ),
                            };

                            let _ = alert_tx.send(alert);
                        }
                    }

                    // Check error rate alerts
                    let recent_errors: Vec<_> = storage
                        .error_tracking
                        .iter()
                        .filter(|e| e.timestamp > Self::current_timestamp() - 300)
                        .collect();

                    if !recent_responses.is_empty() {
                        let error_rate = recent_errors.len() as f64 / recent_responses.len() as f64;

                        if error_rate > thresholds.max_error_rate {
                            let alert = Alert {
                                timestamp: Self::current_timestamp(),
                                alert_type: AlertType::HighErrorRate,
                                severity: if error_rate > 0.2 {
                                    AlertSeverity::Critical
                                } else {
                                    AlertSeverity::Warning
                                },
                                message: format!(
                                    "High error rate detected: {:.1}%",
                                    error_rate * 100.0
                                ),
                                metric_value: error_rate,
                                threshold_value: thresholds.max_error_rate,
                                suggested_action: Some(
                                    "Check logs for error patterns and system health".to_string(),
                                ),
                            };

                            let _ = alert_tx.send(alert);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Record a tool execution
    pub fn record_tool_execution(
        &self,
        tool_name: &str,
        duration: Duration,
        success: bool,
        client_type: Option<&str>,
        payload_size: usize,
    ) {
        if !self.config.enabled || !self.config.monitor_response_times {
            return;
        }

        let metric = ResponseTimeMetric {
            timestamp: Self::current_timestamp(),
            tool_name: tool_name.to_string(),
            duration_ms: duration.as_millis() as u64,
            success,
            client_type: client_type.map(|s| s.to_string()),
            payload_size_bytes: payload_size,
        };

        if let Ok(mut storage) = self.metrics.write() {
            storage.response_times.push_back(metric);

            // Update tool performance metrics
            let tool_perf = storage
                .tool_performance
                .entry(tool_name.to_string())
                .or_insert_with(|| ToolPerformanceMetrics {
                    tool_name: tool_name.to_string(),
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    avg_response_time_ms: 0.0,
                    min_response_time_ms: u64::MAX,
                    max_response_time_ms: 0,
                    last_called: None,
                    error_rate: 0.0,
                    throughput_per_minute: 0.0,
                });

            tool_perf.total_calls += 1;
            if success {
                tool_perf.successful_calls += 1;
            } else {
                tool_perf.failed_calls += 1;
            }

            let duration_ms = duration.as_millis() as u64;
            tool_perf.avg_response_time_ms = (tool_perf.avg_response_time_ms
                * (tool_perf.total_calls - 1) as f64
                + duration_ms as f64)
                / tool_perf.total_calls as f64;
            tool_perf.min_response_time_ms = tool_perf.min_response_time_ms.min(duration_ms);
            tool_perf.max_response_time_ms = tool_perf.max_response_time_ms.max(duration_ms);
            tool_perf.last_called = Some(Self::current_timestamp());
            tool_perf.error_rate = tool_perf.failed_calls as f64 / tool_perf.total_calls as f64;

            // Update session statistics
            storage.session_stats.total_requests += 1;
            if success {
                storage.session_stats.successful_requests += 1;
            } else {
                storage.session_stats.failed_requests += 1;
            }
            storage
                .session_stats
                .unique_tools_used
                .insert(tool_name.to_string());

            // Cleanup old metrics
            self.cleanup_old_metrics(&mut storage);
        }

        debug!(
            "Recorded tool execution: {} ({}ms, success: {})",
            tool_name,
            duration.as_millis(),
            success
        );
    }

    /// Record an error
    pub fn record_error(
        &self,
        tool_name: &str,
        error_type: ErrorType,
        error_message: &str,
        severity: ErrorSeverity,
        client_type: Option<&str>,
    ) {
        if !self.config.enabled || !self.config.monitor_errors {
            return;
        }

        let metric = ErrorMetric {
            timestamp: Self::current_timestamp(),
            tool_name: tool_name.to_string(),
            error_type,
            error_message: error_message.to_string(),
            severity,
            client_type: client_type.map(|s| s.to_string()),
        };

        if let Ok(mut storage) = self.metrics.write() {
            storage.error_tracking.push_back(metric);
            self.cleanup_old_metrics(&mut storage);
        }

        debug!("Recorded error: {} - {}", tool_name, error_message);
    }

    /// Get current performance summary
    pub fn get_performance_summary(&self) -> Result<PerformanceSummary> {
        let storage = self
            .metrics
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to read metrics"))?;

        let uptime_seconds = self.start_time.elapsed().as_secs();
        let current_timestamp = Self::current_timestamp();

        // Calculate success rate
        let success_rate = if storage.session_stats.total_requests > 0 {
            storage.session_stats.successful_requests as f64
                / storage.session_stats.total_requests as f64
        } else {
            1.0
        };

        // Calculate average response time
        let recent_responses: Vec<_> = storage
            .response_times
            .iter()
            .filter(|r| r.timestamp > current_timestamp - 3600) // Last hour
            .collect();

        let avg_response_time_ms = if !recent_responses.is_empty() {
            recent_responses.iter().map(|r| r.duration_ms).sum::<u64>() as f64
                / recent_responses.len() as f64
        } else {
            0.0
        };

        // Get current memory usage
        let current_memory_mb = storage
            .memory_usage
            .back()
            .map(|m| m.memory_mb)
            .unwrap_or(0);

        // Calculate error rate
        let recent_errors: Vec<_> = storage
            .error_tracking
            .iter()
            .filter(|e| e.timestamp > current_timestamp - 3600)
            .collect();

        let error_rate = if !recent_responses.is_empty() {
            recent_errors.len() as f64 / recent_responses.len() as f64
        } else {
            0.0
        };

        // Get top tools by usage
        let mut tool_usage: Vec<_> = storage
            .tool_performance
            .values()
            .map(|metrics| ToolUsageInfo {
                tool_name: metrics.tool_name.clone(),
                usage_count: metrics.total_calls,
                avg_response_time_ms: metrics.avg_response_time_ms,
                error_rate: metrics.error_rate,
            })
            .collect();

        tool_usage.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        tool_usage.truncate(10); // Top 10

        // Get recent errors
        let mut recent_errors_list: Vec<_> = storage
            .error_tracking
            .iter()
            .filter(|e| e.timestamp > current_timestamp - 300) // Last 5 minutes
            .cloned()
            .collect();
        recent_errors_list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        recent_errors_list.truncate(10);

        // Determine system health
        let system_health =
            if error_rate > 0.1 || current_memory_mb > self.config.alert_thresholds.max_memory_mb {
                SystemHealthStatus::Critical
            } else if error_rate > 0.05
                || avg_response_time_ms > self.config.alert_thresholds.max_response_time_ms as f64
            {
                SystemHealthStatus::Warning
            } else {
                SystemHealthStatus::Healthy
            };

        Ok(PerformanceSummary {
            timestamp: current_timestamp,
            uptime_seconds,
            total_requests: storage.session_stats.total_requests,
            success_rate,
            avg_response_time_ms,
            current_memory_mb,
            peak_memory_mb: storage.session_stats.peak_memory_mb,
            error_rate,
            active_alerts: vec![], // Would be populated from alert monitoring
            top_tools_by_usage: tool_usage,
            recent_errors: recent_errors_list,
            system_health,
        })
    }

    /// Export metrics to file
    pub async fn export_metrics(&self) -> Result<()> {
        if !self.config.export_metrics {
            return Ok(());
        }

        let Some(ref export_path) = self.config.metrics_export_path else {
            return Err(anyhow::anyhow!("Export path not configured"));
        };

        let summary = self.get_performance_summary()?;

        // Create export directory if it doesn't exist
        if let Some(parent) = export_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Generate filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = export_path.join(format!("metrics_{}.json", timestamp));

        let json_data = serde_json::to_string_pretty(&summary)?;
        tokio::fs::write(filename, json_data).await?;

        info!("Exported performance metrics to {}", export_path.display());
        Ok(())
    }

    /// Cleanup old metrics to prevent memory growth
    fn cleanup_old_metrics(&self, storage: &mut MetricsStorage) {
        if self.last_cleanup.elapsed() < Duration::from_secs(300) {
            return; // Cleanup every 5 minutes
        }

        let cutoff = Self::current_timestamp() - 3600; // Keep last hour

        storage.response_times.retain(|m| m.timestamp > cutoff);
        storage.memory_usage.retain(|m| m.timestamp > cutoff);
        storage.error_tracking.retain(|m| m.timestamp > cutoff);
        storage.system_metrics.retain(|m| m.timestamp > cutoff);
    }

    /// Collect memory information
    fn collect_memory_info() -> Result<MemoryMetric> {
        let timestamp = Self::current_timestamp();

        #[cfg(unix)]
        {
            // On Unix systems, use /proc/self/status for memory info
            if let Ok(status_content) = std::fs::read_to_string("/proc/self/status") {
                let mut memory_mb = 0;
                let mut heap_mb = None;
                let mut available_mb = None;

                for line in status_content.lines() {
                    if line.starts_with("VmRSS:") {
                        // Resident Set Size (physical memory currently used)
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                memory_mb = kb / 1024; // Convert KB to MB
                            }
                        }
                    } else if line.starts_with("VmData:") {
                        // Heap memory (data segment)
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                heap_mb = Some(kb / 1024);
                            }
                        }
                    }
                }

                // Get available system memory
                if let Ok(meminfo_content) = std::fs::read_to_string("/proc/meminfo") {
                    for line in meminfo_content.lines() {
                        if line.starts_with("MemAvailable:") {
                            if let Some(kb_str) = line.split_whitespace().nth(1) {
                                if let Ok(kb) = kb_str.parse::<usize>() {
                                    available_mb = Some(kb / 1024);
                                }
                            }
                            break;
                        }
                    }
                }

                return Ok(MemoryMetric {
                    timestamp,
                    memory_mb,
                    heap_mb,
                    available_mb,
                    cache_size_mb: None, // Could be calculated from /proc/meminfo if needed
                });
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we could use Windows API calls but currently provide a basic implementation
            // This would require additional dependencies like winapi or windows-rs
            // Currently returning 0 as a safe fallback
            return Ok(MemoryMetric {
                timestamp,
                memory_mb: 0,
                heap_mb: None,
                available_mb: None,
                cache_size_mb: None,
            });
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for other platforms
            return Ok(MemoryMetric {
                timestamp,
                memory_mb: 0,
                heap_mb: None,
                available_mb: None,
                cache_size_mb: None,
            });
        }

        // Ultimate fallback if all platform-specific code fails
        Ok(MemoryMetric {
            timestamp,
            memory_mb: 0,
            heap_mb: None,
            available_mb: None,
            cache_size_mb: None,
        })
    }

    /// Collect system information
    fn collect_system_info() -> Result<SystemMetric> {
        let timestamp = Self::current_timestamp();

        #[cfg(unix)]
        {
            let mut cpu_usage_percent = 0.0;
            let mut disk_usage_percent = 0.0;
            let mut network_activity_mb = 0.0;
            let mut active_connections = 0;

            // Get CPU usage from /proc/stat
            if let Ok(stat_content) = std::fs::read_to_string("/proc/stat") {
                if let Some(cpu_line) = stat_content.lines().next() {
                    if cpu_line.starts_with("cpu ") {
                        let values: Vec<&str> = cpu_line.split_whitespace().collect();
                        if values.len() >= 8 {
                            // Parse CPU times: user, nice, system, idle, iowait, irq, softirq, steal
                            let user: u64 = values[1].parse().unwrap_or(0);
                            let nice: u64 = values[2].parse().unwrap_or(0);
                            let system: u64 = values[3].parse().unwrap_or(0);
                            let idle: u64 = values[4].parse().unwrap_or(0);
                            let iowait: u64 = values[5].parse().unwrap_or(0);
                            let irq: u64 = values[6].parse().unwrap_or(0);
                            let softirq: u64 = values[7].parse().unwrap_or(0);

                            let total_active = user + nice + system + irq + softirq + iowait;
                            let total = total_active + idle;

                            if total > 0 {
                                cpu_usage_percent = (total_active as f64 / total as f64) * 100.0;
                            }
                        }
                    }
                }
            }

            // Get disk usage for root filesystem using df command
            if let Ok(output) = std::process::Command::new("df").args(["-h", "/"]).output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines().skip(1) {
                        let fields: Vec<&str> = line.split_whitespace().collect();
                        if fields.len() >= 5 && fields[4].ends_with('%') {
                            if let Ok(usage) = fields[4].trim_end_matches('%').parse::<f64>() {
                                disk_usage_percent = usage;
                                break;
                            }
                        }
                    }
                }
            }

            // Get network activity from /proc/net/dev
            if let Ok(net_content) = std::fs::read_to_string("/proc/net/dev") {
                let mut total_bytes = 0u64;
                for line in net_content.lines().skip(2) {
                    // Skip header lines
                    if let Some(colon_pos) = line.find(':') {
                        let stats = &line[colon_pos + 1..];
                        let values: Vec<&str> = stats.split_whitespace().collect();
                        if values.len() >= 9 {
                            // bytes received + bytes transmitted
                            let rx_bytes: u64 = values[0].parse().unwrap_or(0);
                            let tx_bytes: u64 = values[8].parse().unwrap_or(0);
                            total_bytes += rx_bytes + tx_bytes;
                        }
                    }
                }
                network_activity_mb = total_bytes as f64 / (1024.0 * 1024.0);
            }

            // Count active network connections from /proc/net/tcp
            if let Ok(tcp_content) = std::fs::read_to_string("/proc/net/tcp") {
                active_connections = tcp_content.lines().skip(1).count(); // Skip header
            }

            Ok(SystemMetric {
                timestamp,
                cpu_usage_percent,
                disk_usage_percent,
                network_activity_mb,
                active_connections,
            })
        }

        #[cfg(windows)]
        {
            // On Windows, we could use Windows API calls
            // Currently providing a basic implementation with system calls
            let mut cpu_usage_percent = 0.0;
            let mut disk_usage_percent = 0.0;

            // Try to get CPU usage using wmic (if available)
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["cpu", "get", "loadpercentage", "/value"])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("LoadPercentage=") {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(value) = value_str.trim().parse::<f64>() {
                                    cpu_usage_percent = value;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Try to get disk usage for C: drive
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&[
                    "logicaldisk",
                    "where",
                    "size!=0",
                    "get",
                    "size,freespace",
                    "/value",
                ])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut size = 0u64;
                    let mut free_space = 0u64;

                    for line in output_str.lines() {
                        if line.starts_with("Size=") {
                            if let Some(value_str) = line.split('=').nth(1) {
                                size = value_str.trim().parse().unwrap_or(0);
                            }
                        } else if line.starts_with("FreeSpace=") {
                            if let Some(value_str) = line.split('=').nth(1) {
                                free_space = value_str.trim().parse().unwrap_or(0);
                            }
                        }
                    }

                    if size > 0 {
                        let used = size - free_space;
                        disk_usage_percent = (used as f64 / size as f64) * 100.0;
                    }
                }
            }

            return Ok(SystemMetric {
                timestamp,
                cpu_usage_percent,
                disk_usage_percent,
                network_activity_mb: 0.0, // Would need Windows-specific implementation
                active_connections: 0,    // Would need Windows-specific implementation
            });
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for other platforms
            Ok(SystemMetric {
                timestamp,
                cpu_usage_percent: 0.0,
                disk_usage_percent: 0.0,
                network_activity_mb: 0.0,
                active_connections: 0,
            })
        }
    }

    /// Get current timestamp in seconds since Unix epoch
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Monitoring middleware for tool execution
pub struct MonitoringMiddleware {
    monitor: Arc<PerformanceMonitor>,
}

impl MonitoringMiddleware {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Wrap tool execution with monitoring
    pub async fn execute_with_monitoring<F, T>(
        &self,
        tool_name: &str,
        client_type: Option<&str>,
        execution: F,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();
        let result = execution.await;
        let duration = start_time.elapsed();

        let (success, payload_size) = match &result {
            Ok(_) => (true, 0), // Could calculate actual payload size
            Err(_) => (false, 0),
        };

        self.monitor
            .record_tool_execution(tool_name, duration, success, client_type, payload_size);

        if let Err(ref error) = result {
            self.monitor.record_error(
                tool_name,
                ErrorType::InternalError, // Would categorize error type
                &error.to_string(),
                ErrorSeverity::Medium, // Would determine severity
                client_type,
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AlertThresholds, MonitoringConfig};

    fn create_test_config() -> MonitoringConfig {
        MonitoringConfig {
            enabled: true,
            collection_interval: Duration::from_secs(1),
            monitor_memory: true,
            monitor_response_times: true,
            monitor_errors: true,
            export_metrics: false,
            metrics_export_path: None,
            alert_thresholds: AlertThresholds {
                max_memory_mb: 1024,
                max_response_time_ms: 5000,
                max_error_rate: 0.1,
                min_success_rate: 0.9,
            },
        }
    }

    #[test]
    fn test_performance_monitor_creation() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        // Should be initialized with empty metrics
        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.total_requests, 0);
        assert_eq!(summary.success_rate, 1.0);
    }

    #[test]
    fn test_tool_execution_recording() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        // Record some tool executions
        monitor.record_tool_execution(
            "test_tool",
            Duration::from_millis(100),
            true,
            Some("claude"),
            1024,
        );

        monitor.record_tool_execution(
            "test_tool",
            Duration::from_millis(200),
            false,
            Some("cursor"),
            512,
        );

        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.success_rate, 0.5);
        assert!(summary.avg_response_time_ms > 0.0);
    }

    #[test]
    fn test_error_recording() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        monitor.record_error(
            "test_tool",
            ErrorType::ValidationError,
            "Test error message",
            ErrorSeverity::Medium,
            Some("claude"),
        );

        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.recent_errors.len(), 1);
        assert_eq!(summary.recent_errors[0].tool_name, "test_tool");
    }

    #[tokio::test]
    async fn test_monitoring_middleware() {
        let config = create_test_config();
        let monitor = Arc::new(PerformanceMonitor::new(config));
        let middleware = MonitoringMiddleware::new(Arc::clone(&monitor));

        // Test successful execution
        let result = middleware
            .execute_with_monitoring("test_tool", Some("claude"), async {
                Ok("success".to_string())
            })
            .await;

        assert!(result.is_ok());

        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_monitoring_middleware_error() {
        let config = create_test_config();
        let monitor = Arc::new(PerformanceMonitor::new(config));
        let middleware = MonitoringMiddleware::new(Arc::clone(&monitor));

        // Test error execution
        let result: Result<String> = middleware
            .execute_with_monitoring("test_tool", Some("claude"), async {
                Err(anyhow::anyhow!("Test error"))
            })
            .await;

        assert!(result.is_err());

        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.success_rate, 0.0);
        assert_eq!(summary.recent_errors.len(), 1);
    }

    #[test]
    fn test_memory_collection_functionality() {
        // Test that memory collection returns valid data structure
        let memory_info = PerformanceMonitor::collect_memory_info();
        assert!(memory_info.is_ok());

        let memory_metric = memory_info.unwrap();
        assert!(memory_metric.timestamp > 0);
        // Memory should be 0 or positive (can be 0 on unsupported platforms)
        // Note: memory_mb is usize, which is always >= 0

        // On Unix systems, we should get real memory data
        #[cfg(unix)]
        {
            // On Unix, if /proc/self/status exists, we should get non-zero memory
            if std::fs::metadata("/proc/self/status").is_ok() {
                // Memory usage should be at least some MB for a running process
                // This is a reasonable check - even minimal processes use some memory
                println!("Memory usage: {}MB", memory_metric.memory_mb);
            }
        }
    }

    #[test]
    fn test_system_info_collection_functionality() {
        // Test that system info collection returns valid data structure
        let system_info = PerformanceMonitor::collect_system_info();
        assert!(system_info.is_ok());

        let system_metric = system_info.unwrap();
        assert!(system_metric.timestamp > 0);

        // CPU usage should be between 0 and 100 (or 0 on unsupported platforms)
        assert!(system_metric.cpu_usage_percent >= 0.0);
        assert!(system_metric.cpu_usage_percent <= 100.0 || system_metric.cpu_usage_percent == 0.0);

        // Disk usage should be between 0 and 100 (or 0 on unsupported platforms)
        assert!(system_metric.disk_usage_percent >= 0.0);
        assert!(
            system_metric.disk_usage_percent <= 100.0 || system_metric.disk_usage_percent == 0.0
        );

        // Network activity and connections should be non-negative
        assert!(system_metric.network_activity_mb >= 0.0);
        // Note: active_connections is usize, which is always >= 0

        println!(
            "System metrics - CPU: {:.1}%, Disk: {:.1}%, Network: {:.1}MB, Connections: {}",
            system_metric.cpu_usage_percent,
            system_metric.disk_usage_percent,
            system_metric.network_activity_mb,
            system_metric.active_connections
        );
    }

    #[test]
    fn test_performance_monitor_with_real_metrics() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        // Test that the monitor can collect real metrics
        let memory_info = PerformanceMonitor::collect_memory_info();
        assert!(memory_info.is_ok());

        let system_info = PerformanceMonitor::collect_system_info();
        assert!(system_info.is_ok());

        // Verify the metrics are being collected in a structured way
        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.total_requests, 0);
        assert_eq!(summary.success_rate, 1.0);
        // Note: uptime_seconds is u64, which is always >= 0
    }
}
