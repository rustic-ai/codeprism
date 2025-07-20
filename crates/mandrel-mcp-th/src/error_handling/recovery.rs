//! Error recovery and retry system for MOTH test harness
//!
//! This module implements retry mechanisms with exponential backoff, error classification,
//! and circuit breaker patterns as specified in the design document.

use crate::error_handling::errors::TestHarnessError;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;
use tracing::{error, info, instrument, warn, Span};

/// Retry configuration per error category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Error recovery configuration for different error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    pub connection_retry: RetryConfig,
    pub request_retry: RetryConfig,
    pub validation_retry: RetryConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            connection_retry: RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            },
            request_retry: RetryConfig {
                max_attempts: 5,
                initial_delay_ms: 50,
                max_delay_ms: 2000,
                backoff_multiplier: 1.5,
                jitter_factor: 0.2,
            },
            validation_retry: RetryConfig {
                max_attempts: 1,
                initial_delay_ms: 0,
                max_delay_ms: 0,
                backoff_multiplier: 1.0,
                jitter_factor: 0.0,
            },
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

/// Circuit breaker configuration for preventing cascading failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u64,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 60000,
            half_open_max_calls: 2,
        }
    }
}

/// Error classifier for determining retry and recovery strategies
#[derive(Debug, Clone, Default)]
pub struct ErrorClassifier {
    custom_rules: HashMap<String, bool>,
}

impl ErrorClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an error is retryable based on its type and context
    pub fn is_retryable(&self, error: &TestHarnessError) -> bool {
        match error {
            TestHarnessError::Client(client_error) => match client_error {
                crate::error_handling::errors::McpClientError::ConnectionFailed { .. } => true,
                crate::error_handling::errors::McpClientError::RequestTimeout { .. } => true,
                crate::error_handling::errors::McpClientError::ServerError { code, .. } => {
                    // Retry on server errors that indicate temporary issues
                    matches!(code, 500..=599)
                }
                crate::error_handling::errors::McpClientError::ProtocolViolation { .. } => false,
                crate::error_handling::errors::McpClientError::TransportError {
                    recoverable,
                    ..
                } => *recoverable,
                crate::error_handling::errors::McpClientError::AuthenticationError {
                    retry_allowed,
                    ..
                } => *retry_allowed,
            },
            TestHarnessError::Network(_) => true,
            TestHarnessError::Io(_) => true,
            TestHarnessError::Performance(_) => true,
            TestHarnessError::Execution(_) => false,
            TestHarnessError::Configuration(_) => false,
            TestHarnessError::Validation(_) => false,
            TestHarnessError::Reporting(_) => false,
            TestHarnessError::Security(_) => false,
        }
    }

    /// Get the error category for classification
    pub fn categorize(&self, error: &TestHarnessError) -> ErrorCategory {
        match error {
            TestHarnessError::Client(_) => ErrorCategory::Connection,
            TestHarnessError::Network(_) => ErrorCategory::Network,
            TestHarnessError::Execution(_) => ErrorCategory::Execution,
            TestHarnessError::Configuration(_) => ErrorCategory::Configuration,
            TestHarnessError::Io(_) => ErrorCategory::Io,
            TestHarnessError::Validation(_) => ErrorCategory::Validation,
            TestHarnessError::Reporting(_) => ErrorCategory::Reporting,
            TestHarnessError::Performance(_) => ErrorCategory::Performance,
            TestHarnessError::Security(_) => ErrorCategory::Security,
        }
    }

    /// Add a custom rule for error classification
    pub fn add_custom_rule(&mut self, pattern: String, is_retryable: bool) {
        self.custom_rules.insert(pattern, is_retryable);
    }

    /// Get retry configuration for a specific error
    pub fn get_retry_config(
        &self,
        error: &TestHarnessError,
        config: &ErrorRecoveryConfig,
    ) -> Option<RetryConfig> {
        if !self.is_retryable(error) {
            return None;
        }

        match self.categorize(error) {
            ErrorCategory::Connection => Some(config.connection_retry.clone()),
            ErrorCategory::Network => Some(config.request_retry.clone()),
            ErrorCategory::Io => Some(config.request_retry.clone()),
            ErrorCategory::Performance => Some(config.request_retry.clone()),
            _ => None,
        }
    }
}

/// Error categories for classification and recovery strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// MCP client connection and communication errors
    Connection,
    /// MCP protocol violations and format errors
    Protocol,
    /// Test execution and assertion errors
    Execution,
    /// Configuration and setup errors
    Configuration,
    /// File system and I/O errors
    Io,
    /// Validation and schema errors
    Validation,
    /// Report generation and output errors
    Reporting,
    /// Network and transport errors
    Network,
    /// Timeout and performance errors
    Performance,
    /// Authentication and authorization errors
    Security,
    /// Unknown or unclassified errors
    Unknown,
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
    error_classifier: ErrorClassifier,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            error_classifier: ErrorClassifier::new(),
        }
    }

    pub fn with_classifier(config: RetryConfig, classifier: ErrorClassifier) -> Self {
        Self {
            config,
            error_classifier: classifier,
        }
    }

    /// Execute an operation with retry logic and exponential backoff
    #[instrument(skip(self, operation))]
    pub async fn execute_with_retry<T, F, Fut, E>(
        &self,
        operation: F,
        operation_name: &str,
    ) -> Result<T, TestHarnessError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<TestHarnessError> + Clone,
    {
        let span = Span::current();
        span.record("operation", operation_name);

        for attempt in 1..=self.config.max_attempts {
            span.record("attempt", attempt);

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(attempt, "Operation succeeded after retry");
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let test_error: TestHarnessError = error.into();

                    if !self.error_classifier.is_retryable(&test_error)
                        || attempt == self.config.max_attempts
                    {
                        error!(?test_error, attempt, "Operation failed permanently");
                        return Err(test_error);
                    }

                    let delay = self.calculate_delay(attempt);
                    warn!(
                        ?test_error,
                        attempt,
                        delay_ms = delay.as_millis(),
                        "Operation failed, retrying"
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }

        unreachable!("Retry loop should have returned")
    }

    /// Calculate delay with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_delay_ms as f64;
        let multiplier = self.config.backoff_multiplier;
        let jitter_factor = self.config.jitter_factor;

        // Calculate exponential backoff
        let exponential_delay = base_delay * multiplier.powi(attempt as i32 - 1);

        // Apply maximum delay limit
        let capped_delay = exponential_delay.min(self.config.max_delay_ms as f64);

        // Add jitter to prevent thundering herd
        let jitter = if jitter_factor > 0.0 {
            let mut rng = rand::thread_rng();
            let jitter_range = capped_delay * jitter_factor;
            rng.gen_range(-jitter_range..=jitter_range)
        } else {
            0.0
        };

        let final_delay = (capped_delay + jitter).max(0.0) as u64;

        Duration::from_millis(final_delay)
    }

    /// Get retry statistics for monitoring
    pub fn get_config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<std::time::Instant>,
    half_open_calls: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_calls: 0,
        }
    }

    /// Execute an operation through the circuit breaker
    pub async fn execute<T, F, Fut>(&mut self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, TestHarnessError>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed().as_millis() > self.config.timeout_ms.into() {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.half_open_calls = 0;
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                }
            }
            CircuitBreakerState::HalfOpen => {
                if self.half_open_calls >= self.config.half_open_max_calls {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
                self.half_open_calls += 1;
            }
            CircuitBreakerState::Closed => {
                // Allow execution
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    fn on_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::Open => {
                // Should not happen
            }
        }
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // Already open
            }
        }
    }

    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }

    pub fn get_failure_count(&self) -> u32 {
        self.failure_count
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(TestHarnessError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_handling::errors::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_error_classifier_retryable_errors() {
        let classifier = ErrorClassifier::new();

        let connection_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 0,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });

        assert!(classifier.is_retryable(&connection_error));

        let protocol_error = TestHarnessError::Client(McpClientError::ProtocolViolation {
            method: "tools/list".to_string(),
            message: "Invalid JSON-RPC format".to_string(),
            request_id: Some("123".to_string()),
            invalid_payload: None,
        });

        assert!(!classifier.is_retryable(&protocol_error));
    }

    #[test]
    fn test_error_classifier_categorization() {
        let classifier = ErrorClassifier::new();

        let client_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "Failed".to_string(),
            retry_count: 0,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });

        assert_eq!(
            classifier.categorize(&client_error),
            ErrorCategory::Connection
        );

        let validation_error = TestHarnessError::Validation(ValidationError::SchemaValidation {
            path: "$.test".to_string(),
            message: "Invalid".to_string(),
            expected_schema: None,
            actual_value: None,
        });

        assert_eq!(
            classifier.categorize(&validation_error),
            ErrorCategory::Validation
        );
    }

    #[test]
    fn test_error_classifier_custom_rules() {
        let mut classifier = ErrorClassifier::new();
        classifier.add_custom_rule("timeout".to_string(), true);
        classifier.add_custom_rule("auth".to_string(), false);

        assert!(classifier.custom_rules.contains_key("timeout"));
        assert!(classifier.custom_rules.contains_key("auth"));
        assert_eq!(classifier.custom_rules.get("timeout"), Some(&true));
        assert_eq!(classifier.custom_rules.get("auth"), Some(&false));
    }

    #[test]
    fn test_retry_executor_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
        };

        let executor = RetryExecutor::new(config);

        let delay1 = executor.calculate_delay(1);
        let delay2 = executor.calculate_delay(2);
        let delay3 = executor.calculate_delay(3);

        assert_eq!(delay1.as_millis(), 100);
        assert_eq!(delay2.as_millis(), 200);
        assert_eq!(delay3.as_millis(), 400);
    }

    #[test]
    fn test_retry_executor_max_delay_cap() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 300,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let executor = RetryExecutor::new(config);

        let delay4 = executor.calculate_delay(4);
        let delay5 = executor.calculate_delay(5);

        // Should be capped at max_delay_ms
        assert_eq!(delay4.as_millis(), 300);
        assert_eq!(delay5.as_millis(), 300);
    }

    #[tokio::test]
    async fn test_retry_executor_success_on_first_attempt() {
        let config = RetryConfig::default();
        let executor = RetryExecutor::new(config);

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        let operation = move || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<i32, TestHarnessError>(42)
            }
        };

        let result = executor
            .execute_with_retry(operation, "test_operation")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_executor_eventual_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1, // Short delay for testing
            max_delay_ms: 10,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };
        let executor = RetryExecutor::new(config);

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        let operation = move || {
            let count = call_count_clone.clone();
            async move {
                let current_count = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if current_count < 3 {
                    Err(TestHarnessError::Network(NetworkError::ConnectionTimeout {
                        endpoint: "test".to_string(),
                        timeout_ms: 1000,
                    }))
                } else {
                    Ok::<i32, TestHarnessError>(42)
                }
            }
        };

        let result = executor
            .execute_with_retry(operation, "test_operation")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        assert_eq!(breaker.get_state(), &CircuitBreakerState::Closed);
        assert_eq!(breaker.get_failure_count(), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout_ms: 1000,
            half_open_max_calls: 1,
        };
        let mut breaker = CircuitBreaker::new(config);

        // First failure
        let result1 = breaker
            .execute(|| async {
                Err::<(), TestHarnessError>(TestHarnessError::Network(
                    NetworkError::ConnectionTimeout {
                        endpoint: "test".to_string(),
                        timeout_ms: 1000,
                    },
                ))
            })
            .await;
        assert!(result1.is_err());
        assert_eq!(breaker.get_state(), &CircuitBreakerState::Closed);

        // Second failure - should open circuit
        let result2 = breaker
            .execute(|| async {
                Err::<(), TestHarnessError>(TestHarnessError::Network(
                    NetworkError::ConnectionTimeout {
                        endpoint: "test".to_string(),
                        timeout_ms: 1000,
                    },
                ))
            })
            .await;
        assert!(result2.is_err());
        assert_eq!(breaker.get_state(), &CircuitBreakerState::Open);

        // Third call should be rejected immediately
        let result3 = breaker
            .execute(|| async { Ok::<(), TestHarnessError>(()) })
            .await;
        assert!(result3.is_err());
        assert!(matches!(
            result3.unwrap_err(),
            CircuitBreakerError::CircuitOpen
        ));
    }

    #[test]
    fn test_error_recovery_config_default() {
        let config = ErrorRecoveryConfig::default();
        assert_eq!(config.connection_retry.max_attempts, 3);
        assert_eq!(config.request_retry.max_attempts, 5);
        assert_eq!(config.validation_retry.max_attempts, 1);
    }

    #[test]
    fn test_error_classifier_get_retry_config() {
        let classifier = ErrorClassifier::new();
        let recovery_config = ErrorRecoveryConfig::default();

        let connection_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "failed".to_string(),
            retry_count: 0,
            last_attempt: chrono::Utc::now(),
            underlying_error: None,
        });

        let retry_config = classifier.get_retry_config(&connection_error, &recovery_config);
        assert!(retry_config.is_some());
        assert_eq!(retry_config.unwrap().max_attempts, 3);

        let validation_error = TestHarnessError::Validation(ValidationError::SchemaValidation {
            path: "$.test".to_string(),
            message: "Invalid".to_string(),
            expected_schema: None,
            actual_value: None,
        });

        let retry_config = classifier.get_retry_config(&validation_error, &recovery_config);
        assert!(retry_config.is_none());
    }
}
