//! Observability and monitoring for CodePrism
//!
//! This module provides comprehensive monitoring, metrics collection, health checks,
//! and structured logging for production deployments.

use crate::error::{Error, ErrorSeverity, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Metrics collector for error rates and performance monitoring
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<Metrics>>,
}

#[derive(Debug, Clone)]
struct Metrics {
    /// Error counts by type
    error_counts: HashMap<String, u64>,
    /// Error counts by severity
    error_severity_counts: HashMap<ErrorSeverity, u64>,
    /// Operation latencies
    operation_latencies: HashMap<String, Vec<Duration>>,
    /// Success/failure rates
    operation_success_rates: HashMap<String, (u64, u64)>, // (success, total)
    /// Resource usage tracking
    resource_usage: HashMap<String, u64>,
    /// Start time for uptime calculation
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            error_counts: HashMap::new(),
            error_severity_counts: HashMap::new(),
            operation_latencies: HashMap::new(),
            operation_success_rates: HashMap::new(),
            resource_usage: HashMap::new(),
            start_time: Instant::now(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
    }

    /// Record an error occurrence
    pub fn record_error(&self, error: &Error, operation: Option<&str>) {
        let mut metrics = self.metrics.lock().unwrap();

        // Count error by type
        let error_type = format!("{:?}", std::mem::discriminant(error));
        *metrics.error_counts.entry(error_type.clone()).or_insert(0) += 1;

        // Count error by severity
        *metrics
            .error_severity_counts
            .entry(error.severity())
            .or_insert(0) += 1;

        // Update operation failure rate
        if let Some(op) = operation {
            let (_success, total) = metrics
                .operation_success_rates
                .entry(op.to_string())
                .or_insert((0, 0));
            *total += 1;
        }

        // Log structured error
        error!(
            error = %error,
            error_type = error_type,
            severity = ?error.severity(),
            operation = operation,
            error_code = error.error_code(),
            "Error recorded"
        );
    }

    /// Record a successful operation
    pub fn record_success(&self, operation: &str, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();

        // Record latency
        metrics
            .operation_latencies
            .entry(operation.to_string())
            .or_default()
            .push(duration);

        // Update success rate
        let (success, total) = metrics
            .operation_success_rates
            .entry(operation.to_string())
            .or_insert((0, 0));
        *success += 1;
        *total += 1;

        debug!(
            operation = operation,
            duration_ms = duration.as_millis(),
            "Operation completed successfully"
        );
    }

    /// Record resource usage
    pub fn record_resource_usage(&self, resource: &str, usage: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.resource_usage.insert(resource.to_string(), usage);
    }

    /// Get error rate for an operation
    pub fn get_error_rate(&self, operation: &str) -> f64 {
        let metrics = self.metrics.lock().unwrap();
        if let Some((success, total)) = metrics.operation_success_rates.get(operation) {
            if *total == 0 {
                0.0
            } else {
                1.0 - (*success as f64 / *total as f64)
            }
        } else {
            0.0
        }
    }

    /// Get average latency for an operation
    pub fn get_average_latency(&self, operation: &str) -> Option<Duration> {
        let metrics = self.metrics.lock().unwrap();
        if let Some(latencies) = metrics.operation_latencies.get(operation) {
            if latencies.is_empty() {
                None
            } else {
                let total_ms: u64 = latencies.iter().map(|d| d.as_millis() as u64).sum();
                Some(Duration::from_millis(total_ms / latencies.len() as u64))
            }
        } else {
            None
        }
    }

    /// Get system uptime
    pub fn uptime(&self) -> Duration {
        let metrics = self.metrics.lock().unwrap();
        Instant::now().duration_since(metrics.start_time)
    }

    /// Get all metrics as a JSON-serializable structure
    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let metrics = self.metrics.lock().unwrap();

        let mut operation_metrics = HashMap::new();
        for (operation, (success, total)) in &metrics.operation_success_rates {
            let error_rate = if *total == 0 {
                0.0
            } else {
                1.0 - (*success as f64 / *total as f64)
            };
            let avg_latency = metrics
                .operation_latencies
                .get(operation)
                .and_then(|latencies| {
                    if latencies.is_empty() {
                        None
                    } else {
                        let total_ms: u64 = latencies.iter().map(|d| d.as_millis() as u64).sum();
                        Some(Duration::from_millis(total_ms / latencies.len() as u64))
                    }
                });

            operation_metrics.insert(
                operation.clone(),
                OperationMetrics {
                    success_count: *success,
                    total_count: *total,
                    error_rate,
                    average_latency_ms: avg_latency.map(|d| d.as_millis() as u64),
                },
            );
        }

        MetricsSnapshot {
            uptime_seconds: Instant::now().duration_since(metrics.start_time).as_secs(),
            error_counts: metrics.error_counts.clone(),
            error_severity_distribution: metrics
                .error_severity_counts
                .iter()
                .map(|(k, v)| (format!("{k:?}"), *v))
                .collect(),
            operation_metrics,
            resource_usage: metrics.resource_usage.clone(),
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Error counts by error type
    pub error_counts: HashMap<String, u64>,
    /// Distribution of errors by severity level
    pub error_severity_distribution: HashMap<String, u64>,
    /// Metrics for each tracked operation
    pub operation_metrics: HashMap<String, OperationMetrics>,
    /// Resource usage statistics
    pub resource_usage: HashMap<String, u64>,
}

/// Metrics for a specific operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct OperationMetrics {
    /// Number of successful executions
    pub success_count: u64,
    /// Total number of executions
    pub total_count: u64,
    /// Error rate as a decimal (0.0 to 1.0)
    pub error_rate: f64,
    /// Average latency in milliseconds
    pub average_latency_ms: Option<u64>,
}

/// Health check status
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum HealthStatus {
    /// All systems functioning normally
    Healthy,
    /// Some issues detected but system is still operational
    Degraded,
    /// Critical issues preventing normal operation
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheckResult {
    /// Overall system health status
    pub status: HealthStatus,
    /// Individual component health checks
    pub checks: HashMap<String, ComponentHealth>,
    /// Human-readable status message
    pub overall_message: String,
    /// When this health check was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health status of a specific system component
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Health status of this component
    pub status: HealthStatus,
    /// Descriptive message about the component's health
    pub message: String,
    /// Additional metrics relevant to this component
    pub metrics: Option<HashMap<String, serde_json::Value>>,
}

/// Health monitor for system components
pub struct HealthMonitor {
    metrics_collector: MetricsCollector,
    circuit_states: Arc<Mutex<HashMap<String, crate::resilience::CircuitState>>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(metrics_collector: MetricsCollector) -> Self {
        Self {
            metrics_collector,
            circuit_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Update circuit breaker state for a component
    pub fn update_circuit_state(&self, component: &str, state: crate::resilience::CircuitState) {
        let mut states = self.circuit_states.lock().unwrap();
        states.insert(component.to_string(), state);
    }

    /// Perform comprehensive health check
    pub fn health_check(&self) -> HealthCheckResult {
        let mut checks = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check error rates
        let error_rate_health = self.check_error_rates();
        if error_rate_health.status != HealthStatus::Healthy {
            overall_status = match overall_status {
                HealthStatus::Healthy => error_rate_health.status.clone(),
                HealthStatus::Degraded => {
                    if error_rate_health.status == HealthStatus::Unhealthy {
                        HealthStatus::Unhealthy
                    } else {
                        HealthStatus::Degraded
                    }
                }
                HealthStatus::Unhealthy => HealthStatus::Unhealthy,
            };
        }
        checks.insert("error_rates".to_string(), error_rate_health);

        // Check circuit breakers
        let circuit_health = self.check_circuit_breakers();
        if circuit_health.status != HealthStatus::Healthy {
            overall_status = match overall_status {
                HealthStatus::Healthy => circuit_health.status.clone(),
                HealthStatus::Degraded => {
                    if circuit_health.status == HealthStatus::Unhealthy {
                        HealthStatus::Unhealthy
                    } else {
                        HealthStatus::Degraded
                    }
                }
                HealthStatus::Unhealthy => HealthStatus::Unhealthy,
            };
        }
        checks.insert("circuit_breakers".to_string(), circuit_health);

        // Check resource usage
        let resource_health = self.check_resource_usage();
        if resource_health.status != HealthStatus::Healthy {
            overall_status = match overall_status {
                HealthStatus::Healthy => resource_health.status.clone(),
                HealthStatus::Degraded => {
                    if resource_health.status == HealthStatus::Unhealthy {
                        HealthStatus::Unhealthy
                    } else {
                        HealthStatus::Degraded
                    }
                }
                HealthStatus::Unhealthy => HealthStatus::Unhealthy,
            };
        }
        checks.insert("resource_usage".to_string(), resource_health);

        let overall_message = match overall_status {
            HealthStatus::Healthy => "All systems operational".to_string(),
            HealthStatus::Degraded => "Some systems experiencing issues".to_string(),
            HealthStatus::Unhealthy => "Critical systems failing".to_string(),
        };

        HealthCheckResult {
            status: overall_status,
            checks,
            overall_message,
            timestamp: chrono::Utc::now(),
        }
    }

    fn check_error_rates(&self) -> ComponentHealth {
        let metrics = self.metrics_collector.get_metrics_snapshot();

        let mut high_error_operations = Vec::new();
        let mut warning_operations = Vec::new();

        for (operation, metrics) in &metrics.operation_metrics {
            if metrics.error_rate > 0.1 {
                // 10% error rate threshold
                high_error_operations.push(operation.clone());
            } else if metrics.error_rate > 0.05 {
                // 5% warning threshold
                warning_operations.push(operation.clone());
            }
        }

        let status = if !high_error_operations.is_empty() {
            HealthStatus::Unhealthy
        } else if !warning_operations.is_empty() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = match status {
            HealthStatus::Healthy => "Error rates within acceptable limits".to_string(),
            HealthStatus::Degraded => {
                format!("Warning: High error rates in operations: {warning_operations:?}")
            }
            HealthStatus::Unhealthy => {
                format!("Critical: Very high error rates in operations: {high_error_operations:?}")
            }
        };

        ComponentHealth {
            status,
            message,
            metrics: Some(
                serde_json::to_value(&metrics.operation_metrics)
                    .and_then(serde_json::from_value)
                    .unwrap_or_default(),
            ),
        }
    }

    fn check_circuit_breakers(&self) -> ComponentHealth {
        let states = self.circuit_states.lock().unwrap();

        let mut open_circuits = Vec::new();
        let mut half_open_circuits = Vec::new();

        for (component, state) in states.iter() {
            match state {
                crate::resilience::CircuitState::Open => open_circuits.push(component.clone()),
                crate::resilience::CircuitState::HalfOpen => {
                    half_open_circuits.push(component.clone())
                }
                crate::resilience::CircuitState::Closed => {}
            }
        }

        let status = if !open_circuits.is_empty() {
            HealthStatus::Unhealthy
        } else if !half_open_circuits.is_empty() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = match status {
            HealthStatus::Healthy => "All circuit breakers closed".to_string(),
            HealthStatus::Degraded => {
                format!("Circuit breakers in recovery: {half_open_circuits:?}")
            }
            HealthStatus::Unhealthy => format!("Open circuit breakers: {open_circuits:?}"),
        };

        let circuit_metrics = states
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(format!("{v:?}"))))
            .collect();

        ComponentHealth {
            status,
            message,
            metrics: Some(circuit_metrics),
        }
    }

    fn check_resource_usage(&self) -> ComponentHealth {
        let metrics = self.metrics_collector.get_metrics_snapshot();

        // Check if resource usage is being tracked
        if metrics.resource_usage.is_empty() {
            return ComponentHealth {
                status: HealthStatus::Healthy,
                message: "Resource usage monitoring not configured".to_string(),
                metrics: None,
            };
        }

        // Simple resource usage check (would be more sophisticated in production)
        let mut high_usage_resources = Vec::new();

        for (resource, usage) in &metrics.resource_usage {
            // Example thresholds (would be configurable)
            let threshold = match resource.as_str() {
                "memory_mb" => 1024, // 1GB threshold
                "cpu_percent" => 80,
                "disk_usage_percent" => 85,
                _ => continue,
            };

            if *usage > threshold {
                high_usage_resources.push(format!("{resource}: {usage}"));
            }
        }

        let status = if high_usage_resources.len() > 1 {
            HealthStatus::Unhealthy
        } else if !high_usage_resources.is_empty() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = match status {
            HealthStatus::Healthy => "Resource usage normal".to_string(),
            HealthStatus::Degraded => format!("High resource usage: {high_usage_resources:?}"),
            HealthStatus::Unhealthy => {
                format!("Critical resource usage: {high_usage_resources:?}")
            }
        };

        ComponentHealth {
            status,
            message,
            metrics: Some(
                serde_json::to_value(&metrics.resource_usage)
                    .and_then(serde_json::from_value)
                    .unwrap_or_default(),
            ),
        }
    }
}

/// Performance monitor for tracking operation latencies
pub struct PerformanceMonitor {
    metrics_collector: MetricsCollector,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(metrics_collector: MetricsCollector) -> Self {
        Self { metrics_collector }
    }

    /// Time an operation and record its performance
    pub async fn time_operation<F, Fut, T>(&self, operation_name: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();

        match &result {
            Ok(_) => {
                self.metrics_collector
                    .record_success(operation_name, duration);
                info!(
                    operation = operation_name,
                    duration_ms = duration.as_millis(),
                    "Operation completed successfully"
                );
            }
            Err(error) => {
                self.metrics_collector
                    .record_error(error, Some(operation_name));
                warn!(
                    operation = operation_name,
                    duration_ms = duration.as_millis(),
                    error = %error,
                    "Operation failed"
                );
            }
        }

        result
    }

    /// Get performance metrics for an operation
    pub fn get_operation_performance(&self, operation: &str) -> Option<OperationPerformance> {
        let error_rate = self.metrics_collector.get_error_rate(operation);
        let avg_latency = self.metrics_collector.get_average_latency(operation)?;

        Some(OperationPerformance {
            operation: operation.to_string(),
            error_rate,
            average_latency: avg_latency,
        })
    }
}

/// Performance metrics for a specific operation
#[derive(Debug, Clone)]
pub struct OperationPerformance {
    /// Name of the operation
    pub operation: String,
    /// Error rate as a decimal (0.0 to 1.0)
    pub error_rate: f64,
    /// Average execution time
    pub average_latency: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record some metrics
        collector.record_success("parse_file", Duration::from_millis(100));
        collector.record_success("parse_file", Duration::from_millis(150));

        let error = Error::storage("test error");
        collector.record_error(&error, Some("parse_file"));

        // Check metrics
        let error_rate = collector.get_error_rate("parse_file");
        assert!((error_rate - 0.333).abs() < 0.01); // Approximately 1/3

        let avg_latency = collector.get_average_latency("parse_file").unwrap();
        assert_eq!(avg_latency, Duration::from_millis(125));
    }

    #[test]
    fn test_health_monitor() {
        let metrics = MetricsCollector::new();
        let monitor = HealthMonitor::new(metrics);

        // Initial health check should be healthy
        let health = monitor.health_check();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.checks.contains_key("error_rates"));
        assert!(health.checks.contains_key("circuit_breakers"));
        assert!(health.checks.contains_key("resource_usage"));
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let metrics = MetricsCollector::new();
        let monitor = PerformanceMonitor::new(metrics);

        // Time a successful operation
        let result = monitor
            .time_operation("test_op", || async {
                sleep(Duration::from_millis(10)).await;
                Ok("success")
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Check performance metrics
        let perf = monitor.get_operation_performance("test_op");
        assert!(perf.is_some());
        let perf = perf.unwrap();
        assert_eq!(perf.error_rate, 0.0);
        assert!(perf.average_latency >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_snapshot() {
        let collector = MetricsCollector::new();

        collector.record_success("op1", Duration::from_millis(100));
        let error = Error::validation("test_field", "test error");
        collector.record_error(&error, Some("op1"));
        collector.record_resource_usage("memory_mb", 512);

        let snapshot = collector.get_metrics_snapshot();

        // Uptime should be a valid positive number - check it's reasonable
        assert!(snapshot.uptime_seconds < 365 * 24 * 3600); // Less than a year
        assert!(snapshot.operation_metrics.contains_key("op1"));
        assert_eq!(snapshot.resource_usage.get("memory_mb"), Some(&512));
        assert!(!snapshot.error_counts.is_empty());
    }
}
