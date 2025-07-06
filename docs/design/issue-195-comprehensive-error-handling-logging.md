# Issue #195: Comprehensive Error Handling and Logging System Design

## Problem Statement

The Mandrel MCP Test Harness currently lacks a comprehensive error handling and logging system, making it difficult to diagnose test failures, debug MCP server interactions, and provide meaningful feedback to users. We need a robust error handling framework with structured logging that provides clear diagnostics and debugging capabilities throughout the entire test execution pipeline.

## Current State Analysis

**Existing Error Handling:**
- Basic error types scattered across modules
- Limited error context and debugging information
- Inconsistent error reporting patterns
- No structured logging or tracing
- Missing error recovery mechanisms

**Pain Points:**
- Difficult to debug failed MCP server connections
- Poor error messages for validation failures
- No visibility into test execution flow
- Missing error categorization and aggregation
- Limited debugging capabilities for complex test scenarios

## Proposed Solution

Implement a comprehensive error handling and logging system with four core components:

### 1. **Hierarchical Error System**
- Comprehensive error type hierarchy using `thiserror`
- Error categorization by component and failure type
- Rich error context with debugging information
- Error chaining and propagation through async operations

### 2. **Structured Logging Framework**
- `tracing`-based structured logging with spans and events
- Multiple output formats (JSON, plain text, pretty)
- Configurable log levels and filtering
- Performance-aware logging with minimal overhead

### 3. **Error Recovery and Retry System**
- Configurable retry mechanisms with exponential backoff
- Error recovery strategies per error category
- Circuit breaker patterns for unreliable connections
- Graceful degradation capabilities

### 4. **Diagnostic and Debugging System**
- Debug modes with verbose output and tracing
- Error metrics collection and analysis
- Error aggregation and summary reporting
- Integration with existing reporting system

## Architecture Design

### Core Error Types Hierarchy

```rust
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Root error type for all test harness operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestHarnessError {
    #[error("MCP client error: {0}")]
    Client(#[from] McpClientError),
    
    #[error("Test execution error: {0}")]
    Execution(#[from] TestExecutionError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    
    #[error("I/O error: {0}")]
    Io(#[from] IoError),
    
    #[error("Reporting error: {0}")]
    Reporting(#[from] ReportingError),
}

/// MCP client-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum McpClientError {
    #[error("Connection failed: {message} (server: {server_name})")]
    ConnectionFailed {
        server_name: String,
        message: String,
        retry_count: u32,
        last_attempt: chrono::DateTime<chrono::Utc>,
    },
    
    #[error("Protocol violation: {message} (method: {method})")]
    ProtocolViolation {
        method: String,
        message: String,
        request_id: Option<String>,
    },
    
    #[error("Request timeout: {method} took {duration_ms}ms (limit: {timeout_ms}ms)")]
    RequestTimeout {
        method: String,
        duration_ms: u64,
        timeout_ms: u64,
        request_id: Option<String>,
    },
    
    #[error("Server error: {error} (code: {code})")]
    ServerError {
        code: i32,
        error: String,
        data: Option<serde_json::Value>,
        method: Option<String>,
    },
}

/// Test execution errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestExecutionError {
    #[error("Test setup failed: {message} (test: {test_name})")]
    SetupFailed {
        test_name: String,
        message: String,
        phase: String,
    },
    
    #[error("Test assertion failed: {message} (test: {test_name}, step: {step})")]
    AssertionFailed {
        test_name: String,
        step: u32,
        message: String,
        expected: Option<serde_json::Value>,
        actual: Option<serde_json::Value>,
    },
    
    #[error("Test timeout: {test_name} exceeded {timeout_seconds}s")]
    TestTimeout {
        test_name: String,
        timeout_seconds: u64,
        elapsed_seconds: u64,
    },
}
```

### Structured Logging Framework

```rust
use tracing::{info, warn, error, debug, instrument, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Logging configuration and setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub outputs: Vec<LogOutput>,
    pub enable_colors: bool,
    pub include_timestamps: bool,
    pub include_thread_ids: bool,
    pub enable_span_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File { path: PathBuf, rotation: Option<FileRotation> },
    Network { endpoint: String, format: LogFormat },
}

/// Central logging system
pub struct LoggingSystem {
    config: LoggingConfig,
    error_collector: Arc<Mutex<ErrorCollector>>,
}

impl LoggingSystem {
    pub fn initialize(config: LoggingConfig) -> Result<Self, TestHarnessError> {
        // Initialize tracing subscriber with configured layers
        let subscriber = tracing_subscriber::registry()
            .with(self.create_fmt_layer(&config)?)
            .with(self.create_error_collector_layer()?)
            .with(self.create_metrics_layer()?);
            
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(Self { config, error_collector: Arc::new(Mutex::new(ErrorCollector::new())) })
    }
    
    pub async fn shutdown(&self) -> Result<ErrorSummary, TestHarnessError> {
        let collector = self.error_collector.lock().await;
        Ok(collector.generate_summary())
    }
}
```

### Error Recovery and Retry System

```rust
/// Retry configuration per error category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    pub connection_retry: RetryConfig,
    pub request_retry: RetryConfig,
    pub validation_retry: RetryConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
    error_classifier: ErrorClassifier,
}

impl RetryExecutor {
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
                    
                    if !self.error_classifier.is_retryable(&test_error) || attempt == self.config.max_attempts {
                        error!(?test_error, attempt, "Operation failed permanently");
                        return Err(test_error);
                    }
                    
                    let delay = self.calculate_delay(attempt);
                    warn!(?test_error, attempt, delay_ms = delay.as_millis(), "Operation failed, retrying");
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        unreachable!("Retry loop should have returned")
    }
}
```

### Error Metrics and Analysis

```rust
/// Error metrics collection and analysis
#[derive(Debug, Default)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_test: HashMap<String, u64>,
    pub retry_counts: HashMap<String, u64>,
    pub recovery_success_rate: f64,
    pub average_error_resolution_time: Duration,
}

#[derive(Debug)]
pub struct ErrorCollector {
    errors: Vec<ErrorEvent>,
    metrics: ErrorMetrics,
    start_time: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: TestHarnessError,
    pub context: ErrorContext,
    pub recovery_attempted: bool,
    pub recovery_successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub test_name: Option<String>,
    pub server_name: Option<String>,
    pub operation: String,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub user_data: HashMap<String, serde_json::Value>,
}
```

## Implementation Plan

### **TDD Phase 1: Core Error Types and Structure**
1. Implement hierarchical error types with `thiserror`
2. Add error context and debugging information
3. Create error categorization system
4. Add serialization support for error reporting
5. Implement error chaining and propagation

**Deliverables:**
- Complete error type hierarchy
- Error context system
- Error categorization framework
- Unit tests for all error types

### **TDD Phase 2: Structured Logging Framework**
1. Implement `tracing`-based logging system
2. Add configurable output formats and destinations
3. Create logging configuration system
4. Implement log filtering and level management
5. Add performance-aware logging patterns

**Deliverables:**
- Logging system initialization
- Multiple output format support
- Configuration management
- Integration tests for logging

### **TDD Phase 3: Error Recovery and Retry System**
1. Implement retry executor with exponential backoff
2. Add error classification for retry decisions
3. Create circuit breaker patterns
4. Implement graceful degradation
5. Add retry metrics and monitoring

**Deliverables:**
- Retry execution framework
- Error classification system
- Circuit breaker implementation
- Recovery strategy patterns

### **TDD Phase 4: Diagnostic and Debugging System**
1. Implement error metrics collection
2. Add debugging modes and verbose output
3. Create error aggregation and summary reporting
4. Integrate with existing reporting system
5. Add error analysis and insights

**Deliverables:**
- Error metrics collection
- Debug mode implementation
- Error summary reporting
- Integration with existing systems

### **TDD Phase 5: Integration and Testing**
1. Integrate error handling across all modules
2. Add comprehensive error handling tests
3. Create error scenario testing
4. Implement error recovery testing
5. Add performance testing for logging overhead

**Deliverables:**
- Complete integration across codebase
- Comprehensive test suite
- Performance benchmarks
- Documentation and examples

## Performance Considerations

**Logging Overhead:**
- Use `tracing::enabled!` macros for conditional logging
- Implement lazy evaluation for expensive log operations
- Buffer log outputs for better I/O performance
- Configure appropriate log levels for production use

**Memory Management:**
- Limit error history retention with configurable limits
- Use Arc<str> for shared error messages
- Implement error event pooling for high-frequency scenarios
- Configure automatic log rotation and cleanup

**Async Performance:**
- Minimize blocking operations in error handling paths
- Use async-friendly logging patterns throughout
- Implement non-blocking error metric collection
- Optimize error propagation through async call chains

## Security Considerations

**Sensitive Information:**
- Implement configurable PII redaction in logs
- Provide secure error message templates
- Add audit trails for error access patterns
- Ensure error context doesn't leak credentials

**Error Information Leakage:**
- Control error detail levels based on context
- Implement safe error serialization patterns
- Add configurable error message sanitization
- Provide production-safe error reporting modes

## Integration Points

### **Existing Systems Integration:**
1. **MCP Client**: Enhanced error reporting with connection diagnostics
2. **Test Execution**: Rich test failure context and debugging information
3. **Validation System**: Detailed validation error reporting with suggestions
4. **Reporting System**: Integration of error summaries and metrics in reports
5. **CLI Interface**: User-friendly error messages and debugging options

### **External Dependencies:**
- `thiserror` for error type definitions
- `tracing` and `tracing-subscriber` for structured logging
- `chrono` for timestamp handling
- `serde` for error serialization
- `tokio` for async error propagation

## Success Criteria

**Functional Requirements:**
- [ ] All components use consistent error types and patterns
- [ ] Structured logging captures full execution context
- [ ] Error recovery works for all retryable error categories
- [ ] Debug modes provide actionable diagnostic information
- [ ] Error metrics enable performance and reliability monitoring

**Quality Requirements:**
- [ ] <1ms logging overhead for normal operations
- [ ] 95%+ error recovery success rate for retryable errors
- [ ] Complete test coverage for error handling paths
- [ ] Zero information leakage in production error messages
- [ ] User-friendly error messages with suggested remediation

**Integration Requirements:**
- [ ] All existing modules migrate to new error system
- [ ] Backward compatibility with existing error handling
- [ ] CLI provides helpful error diagnostics and suggestions
- [ ] Reports include comprehensive error analysis and trends
- [ ] Performance monitoring shows error impact on test execution

## Future Enhancements

**Advanced Features:**
- Error prediction based on historical patterns
- Automated error resolution suggestions
- Integration with external monitoring systems
- Machine learning-based error classification
- Real-time error alerting and notification systems

**Monitoring Integration:**
- Prometheus metrics export
- OpenTelemetry tracing integration
- Custom dashboard support
- Alert rule templates
- SLA monitoring and reporting 