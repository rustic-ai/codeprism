//! Error recovery and retry logic for the Mandrel MCP Test Harness
//!
//! This module provides comprehensive error recovery mechanisms including retry logic
//! with exponential backoff, circuit breaker patterns, and configurable recovery strategies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn, Span};

use crate::error_handling::{ErrorClassifier, RetryConfig, TestHarnessError};

/// Error recovery configuration for different error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    /// Retry configuration for connection errors
    pub connection_retry: RetryConfig,
    /// Retry configuration for network errors
    pub network_retry: RetryConfig,
    /// Retry configuration for performance errors
    pub performance_retry: RetryConfig,
    /// Retry configuration for I/O errors
    pub io_retry: RetryConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Global retry settings
    pub global_settings: GlobalRetrySettings,
}

/// Circuit breaker configuration for preventing cascade failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Time window for counting failures (in seconds)
    pub failure_window_seconds: u64,
    /// Time to wait before attempting to close the circuit (in seconds)
    pub recovery_timeout_seconds: u64,
    /// Percentage of successful calls needed to close the circuit
    pub success_threshold_percentage: f64,
    /// Number of test calls to make when half-open
    pub test_call_count: u32,
}

/// Global retry settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRetrySettings {
    /// Global maximum retry attempts across all operations
    pub global_max_retries: u32,
    /// Global timeout for all retry operations (in seconds)
    pub global_timeout_seconds: u64,
    /// Whether to enable jitter in retry delays
    pub enable_jitter: bool,
    /// Maximum jitter percentage (0.0 to 1.0)
    pub max_jitter_factor: f64,
    /// Whether to respect circuit breaker state
    pub respect_circuit_breaker: bool,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, allowing all operations
    Closed,
    /// Circuit is open, rejecting all operations
    Open { opened_at: DateTime<Utc> },
    /// Circuit is half-open, allowing limited test operations
    HalfOpen { test_calls_made: u32 },
}

/// Circuit breaker for preventing cascade failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    window_start: DateTime<Utc>,
}

/// Retry executor with comprehensive retry logic
#[derive(Debug, Clone)]
pub struct RetryExecutor {
    config: ErrorRecoveryConfig,
    error_classifier: ErrorClassifier,
    circuit_breaker: Option<CircuitBreaker>,
}

/// Retry attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    /// Attempt number (1-based)
    pub attempt_number: u32,
    /// Timestamp of this attempt
    pub timestamp: DateTime<Utc>,
    /// Error that occurred (if any)
    pub error: Option<TestHarnessError>,
    /// Duration of this attempt
    pub duration: Duration,
    /// Whether this attempt succeeded
    pub succeeded: bool,
    /// Delay before this attempt
    pub delay_before_attempt: Duration,
}

/// Retry session tracking multiple attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySession {
    /// Unique session identifier
    pub session_id: uuid::Uuid,
    /// Operation being retried
    pub operation_name: String,
    /// All retry attempts in this session
    pub attempts: Vec<RetryAttempt>,
    /// Total time spent on retries
    pub total_duration: Duration,
    /// Final result of the retry session
    pub final_result: Option<RetryResult>,
    /// Configuration used for this session
    pub config_used: RetryConfig,
}

/// Result of a retry session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryResult {
    /// Operation succeeded after retries
    Success { attempts_made: u32 },
    /// Operation failed permanently after all retries
    PermanentFailure {
        attempts_made: u32,
        final_error: Box<TestHarnessError>,
    },
    /// Operation was aborted due to circuit breaker
    CircuitBreakerOpen,
    /// Operation was aborted due to global timeout
    GlobalTimeout,
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
            network_retry: RetryConfig {
                max_attempts: 5,
                initial_delay_ms: 50,
                max_delay_ms: 2000,
                backoff_multiplier: 1.5,
                jitter_factor: 0.2,
            },
            performance_retry: RetryConfig {
                max_attempts: 2,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 3.0,
                jitter_factor: 0.05,
            },
            io_retry: RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 200,
                max_delay_ms: 3000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.15,
            },
            circuit_breaker: CircuitBreakerConfig::default(),
            global_settings: GlobalRetrySettings::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window_seconds: 60,
            recovery_timeout_seconds: 30,
            success_threshold_percentage: 80.0,
            test_call_count: 3,
        }
    }
}

impl Default for GlobalRetrySettings {
    fn default() -> Self {
        Self {
            global_max_retries: 10,
            global_timeout_seconds: 300, // 5 minutes
            enable_jitter: true,
            max_jitter_factor: 0.25,
            respect_circuit_breaker: true,
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            window_start: Utc::now(),
        }
    }

    /// Check if the circuit breaker allows the operation
    pub fn can_execute(&mut self) -> bool {
        self.update_state();

        match &self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { .. } => false,
            CircuitBreakerState::HalfOpen { test_calls_made } => {
                *test_calls_made < self.config.test_call_count
            }
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match &mut self.state {
            CircuitBreakerState::Closed => {
                self.success_count += 1;
                self.failure_count = 0; // Reset failure count on success
            }
            CircuitBreakerState::HalfOpen { test_calls_made } => {
                *test_calls_made += 1;
                self.success_count += 1;

                // Check if we should close the circuit
                let success_rate = self.success_count as f64 / *test_calls_made as f64 * 100.0;
                if success_rate >= self.config.success_threshold_percentage
                    && *test_calls_made >= self.config.test_call_count
                {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    info!("Circuit breaker closed - sufficient success rate achieved");
                }
            }
            CircuitBreakerState::Open { .. } => {
                // Shouldn't happen, but handle gracefully
                warn!("Recorded success while circuit breaker is open");
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.last_failure_time = Some(Utc::now());

        match &mut self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open {
                        opened_at: Utc::now(),
                    };
                    warn!("Circuit breaker opened due to failure threshold");
                }
            }
            CircuitBreakerState::HalfOpen { test_calls_made } => {
                *test_calls_made += 1;
                self.failure_count += 1;

                // Failed during half-open, go back to open
                self.state = CircuitBreakerState::Open {
                    opened_at: Utc::now(),
                };
                warn!("Circuit breaker reopened due to failure during half-open state");
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, just increment counter
                self.failure_count += 1;
            }
        }
    }

    /// Update circuit breaker state based on time and conditions
    fn update_state(&mut self) {
        let now = Utc::now();

        // Reset failure count if outside the failure window
        if let Some(last_failure) = self.last_failure_time {
            let window_duration = Duration::from_secs(self.config.failure_window_seconds);
            if now
                .signed_duration_since(last_failure)
                .to_std()
                .unwrap_or(Duration::ZERO)
                > window_duration
            {
                self.failure_count = 0;
            }
        }

        // Check if we should transition from Open to HalfOpen
        if let CircuitBreakerState::Open { opened_at } = self.state {
            let recovery_duration = Duration::from_secs(self.config.recovery_timeout_seconds);
            if now
                .signed_duration_since(opened_at)
                .to_std()
                .unwrap_or(Duration::ZERO)
                >= recovery_duration
            {
                self.state = CircuitBreakerState::HalfOpen { test_calls_made: 0 };
                self.success_count = 0;
                info!("Circuit breaker transitioned to half-open for testing");
            }
        }
    }

    /// Get current circuit breaker state
    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }

    /// Get failure count
    pub fn get_failure_count(&self) -> u32 {
        self.failure_count
    }
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        let circuit_breaker = if config.global_settings.respect_circuit_breaker {
            Some(CircuitBreaker::new(config.circuit_breaker.clone()))
        } else {
            None
        };

        Self {
            config,
            error_classifier: ErrorClassifier::new(),
            circuit_breaker,
        }
    }

    /// Execute an operation with retry logic
    #[instrument(skip(self, operation))]
    pub async fn execute_with_retry<T, F, Fut, E>(
        &mut self,
        operation: F,
        operation_name: &str,
    ) -> Result<T, TestHarnessError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<TestHarnessError> + Clone,
    {
        let session_id = uuid::Uuid::new_v4();
        let span = Span::current();
        span.record("operation", operation_name);
        span.record("session_id", session_id.to_string());

        let mut session = RetrySession {
            session_id,
            operation_name: operation_name.to_string(),
            attempts: Vec::new(),
            total_duration: Duration::ZERO,
            final_result: None,
            config_used: self.get_retry_config_for_operation(operation_name),
        };

        let session_start = Instant::now();
        let global_timeout =
            Duration::from_secs(self.config.global_settings.global_timeout_seconds);

        for attempt in 1..=session.config_used.max_attempts {
            // Check global timeout
            if session_start.elapsed() >= global_timeout {
                session.final_result = Some(RetryResult::GlobalTimeout);
                warn!(
                    "Operation {} timed out globally after {} attempts",
                    operation_name,
                    attempt - 1
                );
                return Err(TestHarnessError::Performance(
                    crate::error_handling::PerformanceError::OperationTimeout {
                        operation: operation_name.to_string(),
                        limit_ms: global_timeout.as_millis() as u64,
                        actual_ms: session_start.elapsed().as_millis() as u64,
                        resource_contention: false,
                    },
                ));
            }

            // Check circuit breaker
            if let Some(ref mut circuit_breaker) = self.circuit_breaker {
                if !circuit_breaker.can_execute() {
                    session.final_result = Some(RetryResult::CircuitBreakerOpen);
                    warn!("Operation {} blocked by circuit breaker", operation_name);
                    return Err(TestHarnessError::Network(
                        crate::error_handling::NetworkError::ConnectionRefused {
                            endpoint: operation_name.to_string(),
                            reason: "Circuit breaker is open".to_string(),
                            retry_after_seconds: Some(
                                self.config.circuit_breaker.recovery_timeout_seconds,
                            ),
                        },
                    ));
                }
            }

            // Calculate delay for this attempt (except first)
            let delay = if attempt > 1 {
                self.calculate_delay(attempt - 1, &session.config_used)
            } else {
                Duration::ZERO
            };

            // Apply delay if needed
            if delay > Duration::ZERO {
                debug!("Waiting {}ms before attempt {}", delay.as_millis(), attempt);
                sleep(delay).await;
            }

            // Execute the operation
            let attempt_start = Instant::now();
            let result = operation().await;
            let attempt_duration = attempt_start.elapsed();

            let mut retry_attempt = RetryAttempt {
                attempt_number: attempt,
                timestamp: Utc::now(),
                error: None,
                duration: attempt_duration,
                succeeded: false,
                delay_before_attempt: delay,
            };

            match result {
                Ok(success_result) => {
                    retry_attempt.succeeded = true;
                    session.attempts.push(retry_attempt);
                    session.total_duration = session_start.elapsed();
                    session.final_result = Some(RetryResult::Success {
                        attempts_made: attempt,
                    });

                    // Record success in circuit breaker
                    if let Some(ref mut circuit_breaker) = self.circuit_breaker {
                        circuit_breaker.record_success();
                    }

                    if attempt > 1 {
                        info!(
                            "Operation {} succeeded after {} attempts",
                            operation_name, attempt
                        );
                    }
                    return Ok(success_result);
                }
                Err(error) => {
                    let test_error: TestHarnessError = error.into();
                    retry_attempt.error = Some(test_error.clone());
                    session.attempts.push(retry_attempt);

                    // Record failure in circuit breaker
                    if let Some(ref mut circuit_breaker) = self.circuit_breaker {
                        circuit_breaker.record_failure();
                    }

                    // Check if error is retryable
                    if !self.error_classifier.is_retryable(&test_error) {
                        session.total_duration = session_start.elapsed();
                        session.final_result = Some(RetryResult::PermanentFailure {
                            attempts_made: attempt,
                            final_error: Box::new(test_error.clone()),
                        });
                        error!(
                            "Operation {} failed with non-retryable error: {}",
                            operation_name, test_error
                        );
                        return Err(test_error);
                    }

                    // Check if this is the last attempt
                    if attempt == session.config_used.max_attempts {
                        session.total_duration = session_start.elapsed();
                        session.final_result = Some(RetryResult::PermanentFailure {
                            attempts_made: attempt,
                            final_error: Box::new(test_error.clone()),
                        });
                        error!(
                            "Operation {} failed permanently after {} attempts: {}",
                            operation_name, attempt, test_error
                        );
                        return Err(test_error);
                    }

                    warn!(
                        "Operation {} failed on attempt {} (retryable): {}",
                        operation_name, attempt, test_error
                    );
                }
            }
        }

        unreachable!("Retry loop should have returned")
    }

    /// Get retry configuration for a specific operation
    fn get_retry_config_for_operation(&self, operation_name: &str) -> RetryConfig {
        // Simple heuristic based on operation name
        // In a real implementation, this could be more sophisticated
        if operation_name.contains("connect") || operation_name.contains("connection") {
            self.config.connection_retry.clone()
        } else if operation_name.contains("network") || operation_name.contains("request") {
            self.config.network_retry.clone()
        } else if operation_name.contains("performance") || operation_name.contains("timeout") {
            self.config.performance_retry.clone()
        } else if operation_name.contains("io") || operation_name.contains("file") {
            self.config.io_retry.clone()
        } else {
            // Default to connection retry config
            self.config.connection_retry.clone()
        }
    }

    /// Calculate delay for retry attempt with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32, config: &RetryConfig) -> Duration {
        let base_delay = config.initial_delay_ms as f64;
        let backoff_delay = base_delay * config.backoff_multiplier.powi(attempt as i32 - 1);
        let capped_delay = backoff_delay.min(config.max_delay_ms as f64);

        // Apply jitter if enabled
        let final_delay = if self.config.global_settings.enable_jitter {
            let jitter_factor = config
                .jitter_factor
                .min(self.config.global_settings.max_jitter_factor);
            let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * jitter_factor;
            capped_delay * jitter
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay.max(0.0) as u64)
    }

    /// Get circuit breaker state if enabled
    pub fn get_circuit_breaker_state(&self) -> Option<&CircuitBreakerState> {
        self.circuit_breaker.as_ref().map(|cb| cb.get_state())
    }

    /// Manually reset circuit breaker
    pub fn reset_circuit_breaker(&mut self) {
        if let Some(ref mut circuit_breaker) = self.circuit_breaker {
            circuit_breaker.state = CircuitBreakerState::Closed;
            circuit_breaker.failure_count = 0;
            circuit_breaker.success_count = 0;
            info!("Circuit breaker manually reset");
        }
    }
}

// NOTE: Simplified random implementation for development
// FUTURE(#214): Add `rand` crate dependency for production-grade randomization
mod rand {
    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        // Placeholder implementation
        T::from(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let circuit_breaker = CircuitBreaker::new(config);

        assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);
        assert_eq!(circuit_breaker.get_failure_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..CircuitBreakerConfig::default()
        };
        let mut circuit_breaker = CircuitBreaker::new(config);

        // Record failures up to threshold
        for i in 1..=3 {
            assert!(circuit_breaker.can_execute());
            circuit_breaker.record_failure();

            if i < 3 {
                assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);
            } else {
                assert!(matches!(
                    circuit_breaker.get_state(),
                    CircuitBreakerState::Open { .. }
                ));
            }
        }

        // Should not allow execution when open
        assert!(!circuit_breaker.can_execute());
    }

    #[test]
    fn test_circuit_breaker_success_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold_percentage: 100.0,
            test_call_count: 2,
            ..CircuitBreakerConfig::default()
        };
        let mut circuit_breaker = CircuitBreaker::new(config);

        // Trigger opening
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();
        assert!(matches!(
            circuit_breaker.get_state(),
            CircuitBreakerState::Open { .. }
        ));

        // Manually transition to half-open for testing
        circuit_breaker.state = CircuitBreakerState::HalfOpen { test_calls_made: 0 };

        // Record successful test calls
        assert!(circuit_breaker.can_execute());
        circuit_breaker.record_success();
        assert!(circuit_breaker.can_execute());
        circuit_breaker.record_success();

        // Should now be closed
        assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_retry_executor_success_on_first_try() {
        let config = ErrorRecoveryConfig::default();
        let mut executor = RetryExecutor::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor
            .execute_with_retry(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok::<u32, TestHarnessError>(42)
                    }
                },
                "test_operation",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_executor_success_after_retries() {
        let config = ErrorRecoveryConfig::default();
        let mut executor = RetryExecutor::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor
            .execute_with_retry(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        let attempt = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        if attempt < 3 {
                            Err(TestHarnessError::Client(
                                crate::error_handling::McpClientError::ConnectionFailed {
                                    server_name: "test".to_string(),
                                    message: "Connection failed".to_string(),
                                    retry_count: attempt,
                                    last_attempt: Utc::now(),
                                    underlying_error: None,
                                },
                            ))
                        } else {
                            Ok(42u32)
                        }
                    }
                },
                "test_connection",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_executor_permanent_failure() {
        let mut config = ErrorRecoveryConfig::default();
        config.connection_retry.max_attempts = 2;
        let mut executor = RetryExecutor::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor
            .execute_with_retry(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        let attempt = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        Err::<u32, _>(TestHarnessError::Client(
                            crate::error_handling::McpClientError::ConnectionFailed {
                                server_name: "test".to_string(),
                                message: "Connection failed".to_string(),
                                retry_count: attempt,
                                last_attempt: Utc::now(),
                                underlying_error: None,
                            },
                        ))
                    }
                },
                "test_connection",
            )
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Attempted max_attempts times
    }

    #[tokio::test]
    async fn test_retry_executor_non_retryable_error() {
        let config = ErrorRecoveryConfig::default();
        let mut executor = RetryExecutor::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor
            .execute_with_retry(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Err::<u32, _>(TestHarnessError::Configuration(
                            crate::error_handling::ConfigurationError::MissingRequired {
                                field: "required_field".to_string(),
                                section: None,
                                default_available: false,
                                documentation_url: None,
                            },
                        ))
                    }
                },
                "test_config",
            )
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should not retry non-retryable errors
    }

    #[test]
    fn test_retry_config_selection() {
        let config = ErrorRecoveryConfig::default();
        let executor = RetryExecutor::new(config);

        let connection_config = executor.get_retry_config_for_operation("connection_test");
        assert_eq!(connection_config.max_attempts, 3);

        let network_config = executor.get_retry_config_for_operation("network_request");
        assert_eq!(network_config.max_attempts, 5);

        let performance_config = executor.get_retry_config_for_operation("performance_timeout");
        assert_eq!(performance_config.max_attempts, 2);
    }
}
