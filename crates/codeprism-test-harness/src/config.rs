//! Configuration management for the CodePrism Test Harness
//!
//! This module handles loading, parsing, and validating configuration files
//! for test suites, server settings, and global options.

use crate::types::TestSuite;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main configuration for the test harness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Global settings for test execution
    pub global: GlobalSettings,
    /// MCP server configuration
    pub server: ServerConfig,
    /// Test suites to execute
    pub test_suites: Vec<TestSuite>,
    /// Reporting configuration
    pub reporting: ReportingConfig,
    /// Environment configuration
    pub environment: EnvironmentConfig,
}

/// Global settings that apply to all test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Maximum number of concurrent tests across all suites
    pub max_global_concurrency: usize,
    /// Global timeout for all tests in seconds
    pub global_timeout_seconds: u64,
    /// Default project path for tests (can be overridden per test)
    pub default_project_path: Option<String>,
    /// Whether to stop execution on first failure
    pub fail_fast: bool,
    /// Retry configuration for failed tests
    pub retry: RetryConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Configuration for retry logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries for failed tests
    pub max_retries: usize,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff for retry delays
    pub exponential_backoff: bool,
    /// List of error patterns that should trigger retries
    pub retry_on_patterns: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Whether to log to console
    pub console: bool,
    /// Optional log file path
    pub file: Option<String>,
    /// Whether to include timestamps in logs
    pub timestamps: bool,
    /// Whether to use structured JSON logging
    pub json_format: bool,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to start the MCP server
    pub start_command: String,
    /// Arguments to pass to the server command
    pub args: Vec<String>,
    /// Environment variables for the server
    pub env: HashMap<String, String>,
    /// Working directory for the server
    pub working_dir: Option<String>,
    /// Port the server listens on (if applicable)
    pub port: Option<u16>,
    /// Host the server binds to
    pub host: Option<String>,
    /// Timeout for server startup in seconds
    pub startup_timeout_seconds: u64,
    /// Timeout for server shutdown in seconds
    pub shutdown_timeout_seconds: u64,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Health check configuration for the MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether to perform health checks
    pub enabled: bool,
    /// Endpoint to check for health (if HTTP-based)
    pub endpoint: Option<String>,
    /// Interval between health checks in seconds
    pub interval_seconds: u64,
    /// Number of consecutive failures before considering server unhealthy
    pub failure_threshold: usize,
    /// Timeout for individual health checks in seconds
    pub timeout_seconds: u64,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Output directory for reports
    pub output_dir: String,
    /// Report formats to generate
    pub formats: Vec<ReportFormat>,
    /// Whether to open HTML reports in browser automatically
    pub open_html: bool,
    /// Whether to include detailed debugging information
    pub include_debug_info: bool,
    /// Performance charts configuration
    pub charts: ChartConfig,
    /// Whether to generate trend reports comparing with previous runs
    pub trend_analysis: bool,
}

/// Report format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Html,
    Json,
    Xml,
    Junit,
    Markdown,
}

/// Chart generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Whether to generate performance charts
    pub enabled: bool,
    /// Chart types to generate
    pub types: Vec<ChartType>,
    /// Chart size configuration
    pub size: ChartSize,
}

/// Types of charts to generate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    ResponseTime,
    MemoryUsage,
    SuccessRate,
    TestCoverage,
}

/// Chart size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSize {
    pub width: u32,
    pub height: u32,
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentConfig {
    /// Environment variables to set for all tests
    pub variables: HashMap<String, String>,
    /// Paths to add to PATH environment variable
    pub path_additions: Vec<String>,
    /// Working directory for test execution
    pub working_dir: Option<String>,
    /// Resource limits for test execution
    pub limits: ResourceLimits,
}

/// Resource limits for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU time in seconds
    pub max_cpu_seconds: Option<u64>,
    /// Maximum number of open files
    pub max_open_files: Option<u64>,
    /// Maximum process execution time in seconds
    pub max_process_time_seconds: Option<u64>,
}

impl TestConfig {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        Self::from_yaml(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))
    }

    /// Load configuration from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let config: TestConfig =
            serde_yaml::from_str(yaml).context("Failed to parse YAML configuration")?;

        config
            .validate()
            .context("Configuration validation failed")?;

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate global settings
        if self.global.max_global_concurrency == 0 {
            return Err(anyhow::anyhow!(
                "max_global_concurrency must be greater than 0"
            ));
        }

        if self.global.global_timeout_seconds == 0 {
            return Err(anyhow::anyhow!(
                "global_timeout_seconds must be greater than 0"
            ));
        }

        // Validate server configuration
        if self.server.start_command.is_empty() {
            return Err(anyhow::anyhow!("server start_command cannot be empty"));
        }

        // Validate test suites
        if self.test_suites.is_empty() {
            return Err(anyhow::anyhow!("At least one test suite must be defined"));
        }

        for (i, suite) in self.test_suites.iter().enumerate() {
            if suite.name.is_empty() {
                return Err(anyhow::anyhow!("Test suite {} has empty name", i));
            }

            if suite.test_cases.is_empty() {
                return Err(anyhow::anyhow!(
                    "Test suite '{}' has no test cases",
                    suite.name
                ));
            }

            // Validate each test case in the suite
            for (j, test_case) in suite.test_cases.iter().enumerate() {
                if test_case.id.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Test case {} in suite '{}' has empty id",
                        j,
                        suite.name
                    ));
                }

                if test_case.tool_name.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Test case '{}' in suite '{}' has empty tool_name",
                        test_case.id,
                        suite.name
                    ));
                }
            }
        }

        // Validate reporting configuration
        if self.reporting.output_dir.is_empty() {
            return Err(anyhow::anyhow!("reporting.output_dir cannot be empty"));
        }

        if self.reporting.formats.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one report format must be specified"
            ));
        }

        Ok(())
    }

    /// Get default configuration for testing
    pub fn default_for_tests() -> Self {
        Self {
            global: GlobalSettings::default(),
            server: ServerConfig::default(),
            test_suites: vec![],
            reporting: ReportingConfig::default(),
            environment: EnvironmentConfig::default(),
        }
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(&mut self, other: TestConfig) {
        // Merge global settings
        self.global = other.global;

        // Merge server configuration
        self.server = other.server;

        // Replace test suites (don't merge, replace entirely)
        self.test_suites = other.test_suites;

        // Merge reporting configuration
        self.reporting = other.reporting;

        // Merge environment configuration
        self.environment
            .variables
            .extend(other.environment.variables);
        self.environment
            .path_additions
            .extend(other.environment.path_additions);
        if other.environment.working_dir.is_some() {
            self.environment.working_dir = other.environment.working_dir;
        }
        self.environment.limits = other.environment.limits;
    }
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            max_global_concurrency: 4,
            global_timeout_seconds: 300, // 5 minutes
            default_project_path: None,
            fail_fast: false,
            retry: RetryConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            retry_delay_ms: 1000,
            exponential_backoff: true,
            retry_on_patterns: vec![
                "connection refused".to_string(),
                "timeout".to_string(),
                "temporary failure".to_string(),
            ],
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console: true,
            file: None,
            timestamps: true,
            json_format: false,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            start_command: "cargo run --bin codeprism-mcp".to_string(),
            args: vec!["stdio".to_string()],
            env: HashMap::new(),
            working_dir: None,
            port: None,
            host: None,
            startup_timeout_seconds: 30,
            shutdown_timeout_seconds: 10,
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            interval_seconds: 10,
            failure_threshold: 3,
            timeout_seconds: 5,
        }
    }
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            output_dir: "test-reports".to_string(),
            formats: vec![ReportFormat::Html, ReportFormat::Json],
            open_html: false,
            include_debug_info: true,
            charts: ChartConfig::default(),
            trend_analysis: false,
        }
    }
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            types: vec![
                ChartType::ResponseTime,
                ChartType::MemoryUsage,
                ChartType::SuccessRate,
            ],
            size: ChartSize {
                width: 800,
                height: 400,
            },
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(1024),  // 1GB
            max_cpu_seconds: Some(300), // 5 minutes
            max_open_files: Some(1024),
            max_process_time_seconds: Some(300), // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let mut config = TestConfig::default_for_tests();

        // Add a minimal test suite to make config valid
        config.test_suites.push(TestSuite {
            name: "test_suite".to_string(),
            description: "Test suite".to_string(),
            test_cases: vec![crate::types::TestCase::new(
                "test1".to_string(),
                "repository_stats".to_string(),
                serde_json::json!({}),
            )],
            parallel_execution: false,
            max_concurrency: None,
            setup: None,
            cleanup: None,
        });

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = TestConfig::default_for_tests();

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: TestConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(
            config.global.max_global_concurrency,
            deserialized.global.max_global_concurrency
        );
    }

    #[test]
    fn test_config_validation_failures() {
        let mut config = TestConfig::default_for_tests();

        // Empty test suites should fail validation
        assert!(config.validate().is_err());

        // Empty server command should fail
        config.server.start_command = String::new();
        assert!(config.validate().is_err());
    }
}
