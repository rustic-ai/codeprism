//! Resilience and error recovery mechanisms
//!
//! This module provides comprehensive error handling, retry logic, circuit breakers,
//! and graceful degradation patterns for production reliability.

use crate::error::{Error, ErrorSeverity, RecoveryStrategy, Result};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retry attempts
    pub base_delay: Duration,
    /// Maximum delay between retry attempts
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter to add randomness to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            ..Default::default()
        }
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate delay for a given attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = Duration::from_millis(
            (self.base_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64,
        );

        if delay > self.max_delay {
            delay = self.max_delay;
        }

        if self.jitter {
            // Add up to 25% jitter
            let jitter_ms = (delay.as_millis() as f64 * 0.25 * rand::random::<f64>()) as u64;
            delay += Duration::from_millis(jitter_ms);
        }

        delay
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
    pub success_threshold: u32,
    /// Time to wait before trying half-open state
    pub recovery_timeout: Duration,
    /// Time window for counting failures
    pub time_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
            time_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for handling cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    circuit_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_transition_time: Instant,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                circuit_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_transition_time: Instant::now(),
            })),
        }
    }

    /// Create a circuit breaker with default configuration
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if the circuit allows the request
    pub fn can_execute(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        match state.circuit_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if now.duration_since(state.last_transition_time) >= self.config.recovery_timeout {
                    // Transition to half-open
                    state.circuit_state = CircuitState::HalfOpen;
                    state.success_count = 0;
                    state.last_transition_time = now;
                    debug!("Circuit breaker transitioning to half-open state");
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful execution
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        match state.circuit_state {
            CircuitState::Closed => {
                // Reset failure count in closed state
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    // Transition to closed
                    state.circuit_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_transition_time = now;
                    info!("Circuit breaker closed - service recovered");
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                warn!("Recording success in open circuit state");
            }
        }
    }

    /// Record a failed execution
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        state.last_failure_time = Some(now);

        match state.circuit_state {
            CircuitState::Closed => {
                state.failure_count += 1;
                if state.failure_count >= self.config.failure_threshold {
                    // Transition to open
                    state.circuit_state = CircuitState::Open;
                    state.last_transition_time = now;
                    error!("Circuit breaker opened due to {} failures", state.failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Go back to open
                state.circuit_state = CircuitState::Open;
                state.failure_count += 1;
                state.success_count = 0;
                state.last_transition_time = now;
                warn!("Circuit breaker returned to open state");
            }
            CircuitState::Open => {
                state.failure_count += 1;
            }
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().circuit_state.clone()
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.state.lock().unwrap().failure_count
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Create a retry executor with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..self.config.max_attempts {
            debug!("Executing operation, attempt {}/{}", attempt + 1, self.config.max_attempts);

            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());

                    if !error.should_retry() {
                        debug!("Error is not retryable: {}", error);
                        return Err(error);
                    }

                    if attempt + 1 >= self.config.max_attempts {
                        error!("Operation failed after {} attempts: {}", self.config.max_attempts, error);
                        break;
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!("Operation failed (attempt {}), retrying in {:?}: {}", 
                          attempt + 1, delay, error);

                    sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::other("Unknown retry error")))
    }
}

/// Comprehensive resilience manager
pub struct ResilienceManager {
    retry_executor: RetryExecutor,
    circuit_breaker: CircuitBreaker,
}

impl ResilienceManager {
    /// Create a new resilience manager
    pub fn new(retry_config: RetryConfig, circuit_config: CircuitBreakerConfig) -> Self {
        Self {
            retry_executor: RetryExecutor::new(retry_config),
            circuit_breaker: CircuitBreaker::new(circuit_config),
        }
    }

    /// Create a resilience manager with default configurations
    pub fn default() -> Self {
        Self {
            retry_executor: RetryExecutor::default(),
            circuit_breaker: CircuitBreaker::default(),
        }
    }

    /// Execute an operation with full resilience (circuit breaker + retry)
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T>>,
    {
        if !self.circuit_breaker.can_execute() {
            return Err(Error::other("Circuit breaker is open"));
        }

        let result = self.retry_executor.execute(|| {
            let op = operation.clone();
            async move { op().await }
        }).await;

        match &result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(error) => {
                if matches!(error.severity(), ErrorSeverity::Error | ErrorSeverity::Critical) {
                    self.circuit_breaker.record_failure();
                }
            }
        }

        result
    }

    /// Execute with graceful degradation - returns partial results on failure
    pub async fn execute_with_fallback<F, Fut, T, FB, FutB>(
        &self,
        operation: F,
        fallback: FB,
    ) -> T
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T>>,
        FB: Fn() -> FutB,
        FutB: std::future::Future<Output = T>,
    {
        match self.execute(operation).await {
            Ok(result) => result,
            Err(error) => {
                warn!("Operation failed, using fallback: {}", error);
                fallback().await
            }
        }
    }

    /// Get circuit breaker state
    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state()
    }

    /// Check if the circuit is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.circuit_breaker.state(), CircuitState::Closed)
    }
}

/// Graceful degradation handler
pub struct DegradationHandler;

impl DegradationHandler {
    /// Handle parser failure with graceful degradation
    pub async fn handle_parse_failure<T>(
        file_path: &std::path::Path,
        error: &Error,
        fallback_fn: impl std::future::Future<Output = Option<T>>,
    ) -> Result<Option<T>> {
        match error.recovery_strategy() {
            RecoveryStrategy::Degrade => {
                warn!("Gracefully degrading parse operation for {}: {}", 
                      file_path.display(), error);
                Ok(fallback_fn.await)
            }
            RecoveryStrategy::Fallback => {
                info!("Using fallback for parse operation for {}: {}", 
                      file_path.display(), error);
                Ok(fallback_fn.await)
            }
            _ => Err(error.clone()),
        }
    }

    /// Handle indexing failure with partial results
    pub fn handle_indexing_failure(
        total_files: usize,
        processed_files: usize,
        error: &Error,
    ) -> Result<()> {
        match error.recovery_strategy() {
            RecoveryStrategy::Degrade => {
                let completion_rate = (processed_files as f64 / total_files as f64) * 100.0;
                if completion_rate >= 80.0 {
                    warn!("Indexing completed with degraded results: {:.1}% processed", completion_rate);
                    Ok(())
                } else {
                    Err(error.clone())
                }
            }
            _ => Err(error.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new(5, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(5))
            .with_backoff_multiplier(2.0)
            .with_jitter(true);

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(5));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig::new(3, Duration::from_millis(100))
            .with_backoff_multiplier(2.0)
            .with_jitter(false);

        let delay1 = config.calculate_delay(0);
        let delay2 = config.calculate_delay(1);
        let delay3 = config.calculate_delay(2);

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(200));
        assert_eq!(delay3, Duration::from_millis(400));
    }

    #[test]
    fn test_circuit_breaker_states() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            recovery_timeout: Duration::from_millis(100),
            time_window: Duration::from_secs(60),
        };

        let circuit = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(circuit.state(), CircuitState::Closed);
        assert!(circuit.can_execute());

        // Record failures to open circuit
        circuit.record_failure();
        assert_eq!(circuit.state(), CircuitState::Closed);
        assert!(circuit.can_execute());

        circuit.record_failure();
        assert_eq!(circuit.state(), CircuitState::Open);
        assert!(!circuit.can_execute());
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            recovery_timeout: Duration::from_millis(50),
            time_window: Duration::from_secs(60),
        };

        let circuit = CircuitBreaker::new(config);

        // Open the circuit
        circuit.record_failure();
        assert_eq!(circuit.state(), CircuitState::Open);
        assert!(!circuit.can_execute());

        // Wait for recovery timeout
        sleep(Duration::from_millis(60)).await;

        // Should transition to half-open
        assert!(circuit.can_execute());
        assert_eq!(circuit.state(), CircuitState::HalfOpen);

        // Success should close the circuit
        circuit.record_success();
        assert_eq!(circuit.state(), CircuitState::Closed);
        assert!(circuit.can_execute());
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let executor = RetryExecutor::new(RetryConfig::new(3, Duration::from_millis(10)));

        let mut attempts = 0;
        let result = executor
            .execute(|| {
                attempts += 1;
                async move {
                    if attempts < 2 {
                        Err(Error::storage("temporary failure"))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_executor_failure() {
        let executor = RetryExecutor::new(RetryConfig::new(2, Duration::from_millis(10)));

        let mut attempts = 0;
        let result = executor
            .execute(|| {
                attempts += 1;
                async move {
                    Err(Error::storage("persistent failure"))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_resilience_manager() {
        let manager = ResilienceManager::new(
            RetryConfig::new(2, Duration::from_millis(10)),
            CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 1,
                recovery_timeout: Duration::from_millis(50),
                time_window: Duration::from_secs(60),
            },
        );

        let mut attempts = 0;
        let result = manager
            .execute(|| {
                attempts += 1;
                async move {
                    if attempts < 2 {
                        Err(Error::storage("temporary failure"))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert!(manager.is_healthy());
    }

    #[tokio::test]
    async fn test_execute_with_fallback() {
        let manager = ResilienceManager::default();

        let result = manager
            .execute_with_fallback(
                || async { Err(Error::storage("operation failed")) },
                || async { "fallback result" },
            )
            .await;

        assert_eq!(result, "fallback result");
    }
} 