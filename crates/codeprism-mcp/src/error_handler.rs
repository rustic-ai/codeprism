//! Comprehensive error handling for the MCP server
//!
//! This module provides centralized error handling, recovery mechanisms,
//! and integration with observability systems for production reliability.

use codeprism_core::{
    Error as CoreError, ErrorContext, ErrorSeverity, RecoveryStrategy,
    MetricsCollector, HealthMonitor, PerformanceMonitor, ResilienceManager,
    RetryConfig, CircuitState, HealthCheckResult,
    resilience::CircuitBreakerConfig,
};
use crate::protocol::{JsonRpcError, JsonRpcResponse};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Enhanced error type for MCP operations
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Core codeprism error
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
    
    /// JSON-RPC protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Tool execution error
    #[error("Tool execution error: {tool_name}: {message}")]
    ToolExecution {
        tool_name: String,
        message: String,
        context: Option<ErrorContext>,
    },
    
    /// Resource operation error
    #[error("Resource error: {resource_type}: {message}")]
    Resource {
        resource_type: String,
        message: String,
    },
    
    /// Prompt generation error
    #[error("Prompt error: {prompt_name}: {message}")]
    Prompt {
        prompt_name: String,
        message: String,
    },
    
    /// Cancellation error
    #[error("Operation cancelled: {operation}")]
    Cancelled {
        operation: String,
        reason: Option<String>,
    },
    
    /// Timeout error
    #[error("Operation timed out: {operation} (timeout: {timeout_ms}ms)")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },
    
    /// Rate limiting error
    #[error("Rate limit exceeded for operation: {operation}")]
    RateLimit {
        operation: String,
        retry_after_ms: u64,
    },
}

impl McpError {
    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Core(core_error) => core_error.severity(),
            Self::Protocol(_) => ErrorSeverity::Error,
            Self::ToolExecution { .. } => ErrorSeverity::Warning,
            Self::Resource { .. } => ErrorSeverity::Error,
            Self::Prompt { .. } => ErrorSeverity::Warning,
            Self::Cancelled { .. } => ErrorSeverity::Info,
            Self::Timeout { .. } => ErrorSeverity::Warning,
            Self::RateLimit { .. } => ErrorSeverity::Warning,
        }
    }

    /// Get the recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Core(core_error) => core_error.recovery_strategy(),
            Self::Protocol(_) => RecoveryStrategy::UserIntervention,
            Self::ToolExecution { .. } => RecoveryStrategy::Fallback,
            Self::Resource { .. } => RecoveryStrategy::Retry,
            Self::Prompt { .. } => RecoveryStrategy::Fallback,
            Self::Cancelled { .. } => RecoveryStrategy::UserIntervention,
            Self::Timeout { .. } => RecoveryStrategy::Retry,
            Self::RateLimit { .. } => RecoveryStrategy::Retry,
        }
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Retry)
    }

    /// Get JSON-RPC error code
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            Self::Core(core_error) => core_error.error_code(),
            Self::Protocol(_) => JsonRpcError::INVALID_REQUEST,
            Self::ToolExecution { .. } => -32100, // Custom tool error code
            Self::Resource { .. } => -32101, // Custom resource error code
            Self::Prompt { .. } => -32102, // Custom prompt error code
            Self::Cancelled { .. } => -32015, // Request cancelled
            Self::Timeout { .. } => -32012, // Request timeout
            Self::RateLimit { .. } => -32016, // Rate limit exceeded
        }
    }

    /// Convert to JSON-RPC error
    pub fn to_json_rpc_error(&self) -> JsonRpcError {
        JsonRpcError::new(
            self.json_rpc_code(),
            self.to_string(),
            Some(serde_json::json!({
                "severity": format!("{:?}", self.severity()),
                "recovery_strategy": format!("{:?}", self.recovery_strategy()),
                "error_type": std::mem::discriminant(self),
            }))
        )
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// Comprehensive error handler for the MCP server
pub struct McpErrorHandler {
    metrics_collector: MetricsCollector,
    health_monitor: HealthMonitor,
    performance_monitor: PerformanceMonitor,
    resilience_manager: ResilienceManager,
    circuit_states: Arc<RwLock<std::collections::HashMap<String, CircuitState>>>,
}

impl McpErrorHandler {
    /// Create a new MCP error handler
    pub fn new() -> Self {
        let metrics_collector = MetricsCollector::new();
        let health_monitor = HealthMonitor::new(metrics_collector.clone());
        let performance_monitor = PerformanceMonitor::new(metrics_collector.clone());
        
        let retry_config = RetryConfig::new(3, std::time::Duration::from_millis(100))
            .with_max_delay(std::time::Duration::from_secs(5))
            .with_backoff_multiplier(2.0)
            .with_jitter(true);
            
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            time_window: std::time::Duration::from_secs(60),
        };
        
        let resilience_manager = ResilienceManager::new(retry_config, circuit_config);

        Self {
            metrics_collector,
            health_monitor,
            performance_monitor,
            resilience_manager,
            circuit_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Handle an error with comprehensive logging and metrics
    pub async fn handle_error(&self, error: &McpError, operation: Option<&str>) {
        // Record error in metrics
        let core_error = match error {
            McpError::Core(e) => e.clone(),
            _ => CoreError::other(error.to_string()),
        };
        self.metrics_collector.record_error(&core_error, operation);

        // Update circuit breaker state if needed
        if matches!(error.severity(), ErrorSeverity::Error | ErrorSeverity::Critical) {
            if let Some(op) = operation {
                let mut states = self.circuit_states.write().await;
                let current_state = self.resilience_manager.circuit_state();
                states.insert(op.to_string(), current_state.clone());
                self.health_monitor.update_circuit_state(op, current_state);
            }
        }

        // Log error with appropriate level
        match error.severity() {
            ErrorSeverity::Info => info!(
                error = %error,
                operation = operation,
                severity = ?error.severity(),
                "Informational error"
            ),
            ErrorSeverity::Warning => warn!(
                error = %error,
                operation = operation,
                severity = ?error.severity(),
                recovery_strategy = ?error.recovery_strategy(),
                "Warning: recoverable error"
            ),
            ErrorSeverity::Error => error!(
                error = %error,
                operation = operation,
                severity = ?error.severity(),
                recovery_strategy = ?error.recovery_strategy(),
                "Error: significant issue encountered"
            ),
            ErrorSeverity::Critical => {
                error!(
                    error = %error,
                    operation = operation,
                    severity = ?error.severity(),
                    recovery_strategy = ?error.recovery_strategy(),
                    "CRITICAL: system stability at risk"
                );
                
                // Trigger alert/notification system here if available
                self.trigger_critical_alert(error, operation).await;
            }
        }
    }

    /// Execute an operation with comprehensive error handling and recovery
    pub async fn execute_with_recovery<F, Fut, T>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> McpResult<T>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = McpResult<T>>,
    {
        let result = self.performance_monitor.time_operation(operation_name, || async {
            self.resilience_manager.execute(|| {
                let op = operation.clone();
                async move {
                    match op().await {
                        Ok(value) => Ok(value),
                        Err(mcp_error) => {
                            let core_error = match &mcp_error {
                                McpError::Core(e) => e.clone(),
                                _ => CoreError::other(mcp_error.to_string()),
                            };
                            Err(core_error)
                        }
                    }
                }
            }).await.map_err(McpError::Core)
        }).await;

        match &result {
            Ok(_) => {
                debug!(operation = operation_name, "Operation completed successfully");
            }
            Err(error) => {
                self.handle_error(error, Some(operation_name)).await;
            }
        }

        result
    }

    /// Execute operation with graceful degradation
    pub async fn execute_with_fallback<F, Fut, T, FB, FutB>(
        &self,
        operation_name: &str,
        operation: F,
        fallback: FB,
    ) -> T
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = McpResult<T>>,
        FB: Fn() -> FutB,
        FutB: std::future::Future<Output = T>,
    {
        match self.execute_with_recovery(operation_name, operation).await {
            Ok(result) => result,
            Err(error) => {
                warn!(
                    operation = operation_name,
                    error = %error,
                    "Operation failed, using fallback"
                );
                fallback().await
            }
        }
    }

    /// Get health status
    pub fn get_health_status(&self) -> codeprism_core::HealthCheckResult {
        self.health_monitor.health_check()
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> codeprism_core::MetricsSnapshot {
        self.metrics_collector.get_metrics_snapshot()
    }

    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.resilience_manager.is_healthy()
    }

    /// Convert MCP error to JSON-RPC response
    pub fn error_to_response(&self, error: &McpError, request_id: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: None,
            error: Some(error.to_json_rpc_error()),
        }
    }

    /// Handle partial results for large operations
    pub async fn handle_partial_operation<T>(
        &self,
        operation_name: &str,
        total_items: usize,
        processed_items: usize,
        error: &McpError,
    ) -> McpResult<Option<T>> {
        let completion_rate = (processed_items as f64 / total_items as f64) * 100.0;

        match error.recovery_strategy() {
            RecoveryStrategy::Degrade => {
                if completion_rate >= 80.0 {
                    warn!(
                        operation = operation_name,
                        completion_rate = completion_rate,
                        error = %error,
                        "Operation completed with degraded results"
                    );
                    Ok(None) // Return partial success
                } else {
                    error!(
                        operation = operation_name,
                        completion_rate = completion_rate,
                        error = %error,
                        "Operation failed with insufficient completion rate"
                    );
                    Err(error.clone())
                }
            }
            _ => Err(error.clone()),
        }
    }

    /// Trigger critical alert (placeholder for notification system)
    async fn trigger_critical_alert(&self, error: &McpError, operation: Option<&str>) {
        // In a real implementation, this would integrate with:
        // - PagerDuty, Slack, email alerts
        // - Monitoring systems like Prometheus/Grafana
        // - Incident management systems
        
        error!(
            alert_type = "CRITICAL_ERROR",
            error = %error,
            operation = operation,
            timestamp = %chrono::Utc::now(),
            "CRITICAL ALERT: Manual intervention required"
        );
        
        // Example: Send to external monitoring system
        // monitoring_client.send_alert(AlertLevel::Critical, error, operation).await;
    }

    /// Create error context for better tracing
    pub fn create_context(
        &self,
        request_id: Option<String>,
        operation: Option<String>,
    ) -> ErrorContext {
        let mut context = ErrorContext::new();
        
        if let Some(id) = request_id {
            context = context.with_request_id(id);
        }
        
        if let Some(op) = operation {
            context = context.with_operation(op);
        }
        
        // Add system metrics as context
        let health = self.get_health_status();
        context = context.with_metadata(
            "system_health".to_string(),
            serde_json::to_value(health.status).unwrap_or_default()
        );
        
        context
    }
}

impl Default for McpErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macros for error handling
#[macro_export]
macro_rules! mcp_try {
    ($expr:expr, $handler:expr, $operation:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                let mcp_error = McpError::Core(error);
                $handler.handle_error(&mcp_error, Some($operation)).await;
                return Err(mcp_error);
            }
        }
    };
}

#[macro_export]
macro_rules! mcp_tool_error {
    ($tool_name:expr, $message:expr) => {
        McpError::ToolExecution {
            tool_name: $tool_name.to_string(),
            message: $message.to_string(),
            context: None,
        }
    };
    ($tool_name:expr, $message:expr, $context:expr) => {
        McpError::ToolExecution {
            tool_name: $tool_name.to_string(),
            message: $message.to_string(),
            context: Some($context),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_mcp_error_severity() {
        let error = McpError::Protocol("test error".to_string());
        assert_eq!(error.severity(), ErrorSeverity::Error);
        
        let error = McpError::Cancelled {
            operation: "test_op".to_string(),
            reason: None,
        };
        assert_eq!(error.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_mcp_error_json_rpc_conversion() {
        let error = McpError::ToolExecution {
            tool_name: "test_tool".to_string(),
            message: "test error".to_string(),
            context: None,
        };
        
        let json_rpc_error = error.to_json_rpc_error();
        assert_eq!(json_rpc_error.code, -32100);
        assert!(json_rpc_error.message.contains("test error"));
    }

    #[tokio::test]
    async fn test_error_handler_creation() {
        let handler = McpErrorHandler::new();
        assert!(handler.is_healthy());
    }

    #[tokio::test]
    async fn test_execute_with_recovery_success() {
        let handler = McpErrorHandler::new();
        
        let result = handler.execute_with_recovery("test_op", || async {
            Ok::<i32, McpError>(42)
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_execute_with_recovery_failure() {
        let handler = McpErrorHandler::new();
        
        let result = handler.execute_with_recovery("test_op", || async {
            Err::<i32, McpError>(McpError::Protocol("test error".to_string()))
        }).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_fallback() {
        let handler = McpErrorHandler::new();
        
        let result = handler.execute_with_fallback(
            "test_op",
            || async { Err::<i32, McpError>(McpError::Protocol("test error".to_string())) },
            || async { 100 },
        ).await;
        
        assert_eq!(result, 100);
    }

    #[tokio::test]
    async fn test_error_handling_and_metrics() {
        let handler = McpErrorHandler::new();
        
        let error = McpError::ToolExecution {
            tool_name: "test_tool".to_string(),
            message: "test error".to_string(),
            context: None,
        };
        
        handler.handle_error(&error, Some("test_operation")).await;
        
        let metrics = handler.get_metrics();
        assert!(metrics.uptime_seconds > 0);
    }

    #[test]
    fn test_error_context_creation() {
        let handler = McpErrorHandler::new();
        
        let context = handler.create_context(
            Some("req-123".to_string()),
            Some("test_operation".to_string()),
        );
        
        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert!(!context.metadata.is_empty());
    }

    #[tokio::test]
    async fn test_partial_operation_handling() {
        let handler = McpErrorHandler::new();
        
        let error = McpError::Core(CoreError::indexing("partial failure"));
        
        // Test successful degradation (80% completion)
        let result = handler.handle_partial_operation::<()>(
            "test_op",
            100,
            85,
            &error,
        ).await;
        assert!(result.is_ok());
        
        // Test failure (low completion rate)
        let result = handler.handle_partial_operation::<()>(
            "test_op",
            100,
            50,
            &error,
        ).await;
        assert!(result.is_err());
    }
} 