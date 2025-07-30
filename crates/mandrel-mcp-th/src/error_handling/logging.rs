//! Structured logging framework for MOTH test harness
//!
//! This module implements comprehensive logging with tracing, multiple output formats,
//! and configurable log levels as specified in the design document.

use crate::error_handling::errors::{ErrorContext, TestHarnessError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;

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
    pub filter_patterns: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            outputs: vec![LogOutput::Stdout],
            enable_colors: true,
            include_timestamps: true,
            include_thread_ids: false,
            enable_span_events: true,
            filter_patterns: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
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
    File {
        path: PathBuf,
        rotation: Option<FileRotation>,
    },
    Network {
        endpoint: String,
        format: LogFormat,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRotation {
    pub max_size_mb: u64,
    pub max_files: u32,
    pub compress: bool,
}

/// Central logging system that coordinates all logging activities
pub struct LoggingSystem {
    #[allow(dead_code)] // Reserved for future configuration updates
    config: LoggingConfig,
    error_collector: Arc<Mutex<ErrorCollector>>,
}

impl LoggingSystem {
    /// Initialize the logging system with the given configuration
    #[allow(clippy::result_large_err)] // Error enum variants are from existing error system
    pub fn initialize(config: LoggingConfig) -> Result<Self, TestHarnessError> {
        let error_collector = Arc::new(Mutex::new(ErrorCollector::new()));

        // Create a simple subscriber with a format layer and filter
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(config.enable_colors)
                    .with_target(true),
            )
            .with(tracing_subscriber::filter::LevelFilter::from_level(
                config.level.to_tracing_level(),
            ));

        // Set the global default subscriber
        tracing::subscriber::set_global_default(subscriber).map_err(|e| {
            TestHarnessError::Configuration(
                crate::error_handling::errors::ConfigurationError::InvalidConfig {
                    field: "logging".to_string(),
                    message: format!("Failed to set global subscriber: {e}"),
                    provided_value: None,
                },
            )
        })?;

        info!(
            level = ?config.level,
            format = ?config.format,
            outputs = config.outputs.len(),
            "Logging system initialized"
        );

        Ok(Self {
            config,
            error_collector,
        })
    }

    /// Shutdown the logging system and return error summary
    pub async fn shutdown(self) -> Result<ErrorSummary, TestHarnessError> {
        info!("Shutting down logging system");

        let collector = self.error_collector.lock().map_err(|e| {
            TestHarnessError::Reporting(
                crate::error_handling::errors::ReportingError::GenerationFailed {
                    format: "error_summary".to_string(),
                    message: format!("Failed to acquire error collector lock: {e}"),
                    output_path: None,
                },
            )
        })?;

        Ok(collector.generate_summary())
    }

    /// Get current error statistics
    #[allow(clippy::result_large_err)] // Error enum variants are from existing error system
    pub fn get_error_statistics(&self) -> Result<ErrorStatistics, TestHarnessError> {
        let collector = self.error_collector.lock().map_err(|e| {
            TestHarnessError::Reporting(
                crate::error_handling::errors::ReportingError::GenerationFailed {
                    format: "error_statistics".to_string(),
                    message: format!("Failed to acquire error collector lock: {e}"),
                    output_path: None,
                },
            )
        })?;

        Ok(collector.get_statistics())
    }
}

/// Error collector for gathering error events and statistics
#[derive(Debug)]
pub struct ErrorCollector {
    events: Vec<ErrorEvent>,
    statistics: ErrorStatistics,
    start_time: std::time::Instant,
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            statistics: ErrorStatistics::default(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_error(&mut self, error: &TestHarnessError, context: ErrorContext) {
        let event = ErrorEvent {
            timestamp: chrono::Utc::now(),
            error: error.clone(),
            context,
            recovery_attempted: false,
            recovery_successful: false,
        };

        self.events.push(event);
        self.update_statistics(error);
    }

    fn update_statistics(&mut self, error: &TestHarnessError) {
        self.statistics.total_errors += 1;

        let category = self.categorize_error(error);
        *self
            .statistics
            .errors_by_category
            .entry(category)
            .or_insert(0) += 1;
    }

    fn categorize_error(&self, error: &TestHarnessError) -> String {
        match error {
            TestHarnessError::Client(_) => "client".to_string(),
            TestHarnessError::Execution(_) => "execution".to_string(),
            TestHarnessError::Validation(_) => "validation".to_string(),
            TestHarnessError::Configuration(_) => "configuration".to_string(),
            TestHarnessError::Io(_) => "io".to_string(),
            TestHarnessError::Reporting(_) => "reporting".to_string(),
            TestHarnessError::Network(_) => "network".to_string(),
            TestHarnessError::Performance(_) => "performance".to_string(),
            TestHarnessError::Security(_) => "security".to_string(),
        }
    }

    pub fn generate_summary(&self) -> ErrorSummary {
        let duration = self.start_time.elapsed();

        ErrorSummary {
            total_errors: self.statistics.total_errors,
            errors_by_category: self.statistics.errors_by_category.clone(),
            execution_duration: duration,
            error_rate: self.statistics.total_errors as f64 / duration.as_secs_f64(),
            most_common_error: self.find_most_common_error(),
        }
    }

    pub fn get_statistics(&self) -> ErrorStatistics {
        self.statistics.clone()
    }

    fn find_most_common_error(&self) -> Option<String> {
        self.statistics
            .errors_by_category
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category.clone())
    }
}

/// Error event for detailed error tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: TestHarnessError,
    pub context: ErrorContext,
    pub recovery_attempted: bool,
    pub recovery_successful: bool,
}

/// Error statistics for monitoring and reporting
#[derive(Debug, Clone, Default)]
pub struct ErrorStatistics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
}

/// Error summary for reporting
#[derive(Debug, Clone, Serialize)]
pub struct ErrorSummary {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub execution_duration: std::time::Duration,
    pub error_rate: f64,
    pub most_common_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(matches!(config.level, LogLevel::Info));
        assert!(matches!(config.format, LogFormat::Pretty));
        assert_eq!(config.outputs.len(), 1, "Should have 1 items");
        assert!(config.enable_colors);
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::Error.to_tracing_level(), tracing::Level::ERROR);
        assert_eq!(LogLevel::Warn.to_tracing_level(), tracing::Level::WARN);
        assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
        assert_eq!(LogLevel::Debug.to_tracing_level(), tracing::Level::DEBUG);
        assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
    }

    #[test]
    fn test_error_collector_creation() {
        let collector = ErrorCollector::new();
        assert_eq!(collector.events.len(), 0, "Should have 0 items");
        assert_eq!(collector.statistics.total_errors, 0);
    }

    #[test]
    fn test_error_collector_record_error() {
        let mut collector = ErrorCollector::new();
        let error = TestHarnessError::Configuration(
            crate::error_handling::errors::ConfigurationError::MissingConfig {
                field: "test_field".to_string(),
                config_file: Some("test.yaml".to_string()),
            },
        );
        let context = ErrorContext::new("test_operation");

        collector.record_error(&error, context);

        assert_eq!(collector.events.len(), 1, "Should have 1 items");
        assert_eq!(collector.statistics.total_errors, 1);
        assert_eq!(
            collector.statistics.errors_by_category.get("configuration"),
            Some(&1)
        );
    }

    #[test]
    fn test_error_summary_generation() {
        let mut collector = ErrorCollector::new();
        let error = TestHarnessError::Network(
            crate::error_handling::errors::NetworkError::ConnectionTimeout {
                endpoint: "localhost:8080".to_string(),
                timeout_ms: 5000,
            },
        );
        let context = ErrorContext::new("network_test");

        collector.record_error(&error, context);

        let summary = collector.generate_summary();
        assert_eq!(summary.total_errors, 1);
        assert!(summary.errors_by_category.contains_key("network"));
        assert_eq!(summary.most_common_error, Some("network".to_string()));
    }

    #[tokio::test]
    async fn test_logging_system_initialization() {
        let config = LoggingConfig::default();
        let logging_system = LoggingSystem::initialize(config);

        assert!(logging_system.is_ok(), "Operation should succeed");

        // Test that logging works
        info!("Test log message");

        let system = logging_system.unwrap();
        let stats = system.get_error_statistics();
        assert!(stats.is_ok(), "Operation should succeed");
    }

    #[test]
    fn test_error_categorization() {
        let collector = ErrorCollector::new();

        let client_error = TestHarnessError::Client(
            crate::error_handling::errors::McpClientError::ConnectionFailed {
                server_name: "test".to_string(),
                message: "failed".to_string(),
                retry_count: 0,
                last_attempt: chrono::Utc::now(),
                underlying_error: None,
            },
        );

        assert_eq!(collector.categorize_error(&client_error), "client");

        let validation_error = TestHarnessError::Validation(
            crate::error_handling::errors::ValidationError::SchemaValidation {
                path: "$.test".to_string(),
                message: "Invalid".to_string(),
                expected_schema: None,
                actual_value: None,
            },
        );

        assert_eq!(collector.categorize_error(&validation_error), "validation");
    }
}
