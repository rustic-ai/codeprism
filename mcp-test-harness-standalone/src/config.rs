//! Configuration management for the standalone MCP test harness
//!
//! Provides universal configuration system that works with any MCP server implementation.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main test configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Global test configuration
    pub global: GlobalConfig,

    /// MCP server configuration
    pub server: ServerConfig,

    /// Performance monitoring settings
    pub performance: Option<PerformanceConfig>,

    /// Test suites to execute
    pub test_suites: Vec<TestSuite>,

    /// Performance baselines (optional)
    pub baselines: Option<HashMap<String, PerformanceBaseline>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Maximum number of concurrent tests
    pub max_global_concurrency: usize,

    /// Global timeout in seconds
    pub timeout_seconds: u64,

    /// Whether to fail fast on first error
    pub fail_fast: bool,

    /// Default project path for testing
    pub default_project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server type (stdio, http)
    pub transport: String,

    /// Command to start the server (for stdio)
    pub command: Option<String>,

    /// Arguments for the server command
    pub args: Option<Vec<String>>,

    /// Working directory for the server
    pub working_dir: Option<String>,

    /// Environment variables for the server
    pub env: Option<HashMap<String, String>>,

    /// Server URL (for HTTP transport)
    pub url: Option<String>,

    /// Connection timeout in seconds
    pub connection_timeout: Option<u64>,

    /// Server startup delay in seconds
    pub startup_delay: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,

    /// Path to store baseline data
    pub baseline_storage_path: Option<String>,

    /// Regression detection settings
    pub regression_detection: Option<RegressionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Warning threshold percentage
    pub warning_threshold_percent: f64,

    /// Error threshold percentage
    pub error_threshold_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Name of the test suite
    pub name: String,

    /// Description of what this suite tests
    pub description: Option<String>,

    /// Individual test cases
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique identifier for the test
    pub id: String,

    /// MCP tool/method name to test
    pub tool_name: String,

    /// Description of the test
    pub description: Option<String>,

    /// Whether this test is enabled
    pub enabled: bool,

    /// Input parameters for the test
    pub input_params: Option<serde_json::Value>,

    /// Expected response validation
    pub expected: Option<ExpectedResponse>,

    /// Performance requirements
    pub performance_requirements: Option<PerformanceRequirements>,

    /// Custom validation scripts
    pub custom_scripts: Option<Vec<CustomScript>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResponse {
    /// Validation patterns to check
    pub patterns: Option<Vec<ValidationPattern>>,

    /// Whether empty results are acceptable
    pub allow_empty_results: Option<bool>,

    /// Whether test failure is acceptable
    pub allow_failure: Option<bool>,

    /// Performance requirements for this test
    pub performance_requirements: Option<PerformanceRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPattern {
    /// JSON path key to validate
    pub key: String,

    /// Validation configuration
    pub validation: ValidationConfig,

    /// Whether this validation is required
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationConfig {
    Exists,
    Equals { value: serde_json::Value },
    Range { min: f64, max: f64 },
    Array,
    ArrayMinLength { min_length: usize },
    ArrayMaxLength { max_length: usize },
    Boolean { value: Option<bool> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,

    /// Maximum memory usage in MB
    pub max_memory_usage_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScript {
    /// Script name/identifier
    pub name: String,

    /// Programming language
    pub language: String,

    /// Script content
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,

    /// Peak memory usage in MB
    pub peak_memory_mb: f64,

    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
}

/// Template information for common server types
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub server_type: String,
}

impl TestConfig {
    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        // Try YAML first, then JSON
        if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content).with_context(|| "Failed to parse JSON configuration")
        } else {
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML configuration")
        }
    }

    /// Validate configuration file
    pub fn validate<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Self::load(path)?;

        // Basic validation
        if config.test_suites.is_empty() {
            return Err(anyhow!(
                "Configuration must contain at least one test suite"
            ));
        }

        for suite in &config.test_suites {
            if suite.test_cases.is_empty() {
                return Err(anyhow!(
                    "Test suite '{}' must contain at least one test case",
                    suite.name
                ));
            }

            for test_case in &suite.test_cases {
                if test_case.tool_name.is_empty() {
                    return Err(anyhow!(
                        "Test case '{}' must specify a tool_name",
                        test_case.id
                    ));
                }
            }
        }

        Ok(())
    }

    /// Create a default configuration
    pub fn default_config() -> Self {
        Self {
            global: GlobalConfig {
                max_global_concurrency: 2,
                timeout_seconds: 30,
                fail_fast: false,
                default_project_path: None,
            },
            server: ServerConfig {
                transport: "stdio".to_string(),
                command: None,
                args: None,
                working_dir: None,
                env: None,
                url: None,
                connection_timeout: Some(10),
                startup_delay: Some(2),
            },
            performance: Some(PerformanceConfig {
                enable_monitoring: true,
                baseline_storage_path: Some("baselines/".to_string()),
                regression_detection: Some(RegressionConfig {
                    warning_threshold_percent: 25.0,
                    error_threshold_percent: 50.0,
                }),
            }),
            test_suites: vec![TestSuite {
                name: "Basic MCP Compliance".to_string(),
                description: Some("Basic Model Context Protocol compliance tests".to_string()),
                test_cases: vec![TestCase {
                    id: "initialize".to_string(),
                    tool_name: "initialize".to_string(),
                    description: Some("Test MCP initialization".to_string()),
                    enabled: true,
                    input_params: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {
                            "name": "mcp-test-harness",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    })),
                    expected: Some(ExpectedResponse {
                        patterns: Some(vec![
                            ValidationPattern {
                                key: "protocolVersion".to_string(),
                                validation: ValidationConfig::Exists,
                                required: true,
                            },
                            ValidationPattern {
                                key: "capabilities".to_string(),
                                validation: ValidationConfig::Exists,
                                required: true,
                            },
                        ]),
                        allow_empty_results: Some(false),
                        allow_failure: Some(false),
                        performance_requirements: Some(PerformanceRequirements {
                            max_execution_time_ms: Some(5000),
                            max_memory_usage_mb: Some(64),
                        }),
                    }),
                    performance_requirements: None,
                    custom_scripts: None,
                }],
            }],
            baselines: None,
        }
    }
}

/// Generate template configuration for different server types
pub fn generate_template(server_type: &str) -> Result<String> {
    let template = match server_type {
        "filesystem" => generate_filesystem_template(),
        "database" => generate_database_template(),
        "api" => generate_api_template(),
        "custom" => TestConfig::default_config(),
        _ => return Err(anyhow!("Unknown server type: {}", server_type)),
    };

    serde_yaml::to_string(&template).with_context(|| "Failed to serialize template to YAML")
}

fn generate_filesystem_template() -> TestConfig {
    let mut config = TestConfig::default_config();
    config.server.command = Some("npx".to_string());
    config.server.args = Some(vec![
        "@modelcontextprotocol/server-filesystem".to_string(),
        "/tmp".to_string(),
    ]);

    // Add filesystem-specific tests
    config.test_suites.push(TestSuite {
        name: "Filesystem Operations".to_string(),
        description: Some("Test filesystem MCP server operations".to_string()),
        test_cases: vec![TestCase {
            id: "list_files".to_string(),
            tool_name: "list_resources".to_string(),
            description: Some("List available file resources".to_string()),
            enabled: true,
            input_params: None,
            expected: Some(ExpectedResponse {
                patterns: Some(vec![ValidationPattern {
                    key: "resources".to_string(),
                    validation: ValidationConfig::Array,
                    required: true,
                }]),
                allow_empty_results: Some(true),
                allow_failure: Some(false),
                performance_requirements: None,
            }),
            performance_requirements: Some(PerformanceRequirements {
                max_execution_time_ms: Some(3000),
                max_memory_usage_mb: Some(32),
            }),
            custom_scripts: None,
        }],
    });

    config
}

fn generate_database_template() -> TestConfig {
    let mut config = TestConfig::default_config();
    config.server.command = Some("node".to_string());
    config.server.args = Some(vec!["database-mcp-server.js".to_string()]);
    config.server.env = Some({
        let mut env = HashMap::new();
        env.insert(
            "DATABASE_URL".to_string(),
            "sqlite:///tmp/test.db".to_string(),
        );
        env
    });

    // Add database-specific tests
    config.test_suites.push(TestSuite {
        name: "Database Operations".to_string(),
        description: Some("Test database MCP server operations".to_string()),
        test_cases: vec![TestCase {
            id: "list_tools".to_string(),
            tool_name: "list_tools".to_string(),
            description: Some("List available database tools".to_string()),
            enabled: true,
            input_params: None,
            expected: Some(ExpectedResponse {
                patterns: Some(vec![ValidationPattern {
                    key: "tools".to_string(),
                    validation: ValidationConfig::Array,
                    required: true,
                }]),
                allow_empty_results: Some(false),
                allow_failure: Some(false),
                performance_requirements: None,
            }),
            performance_requirements: Some(PerformanceRequirements {
                max_execution_time_ms: Some(2000),
                max_memory_usage_mb: Some(64),
            }),
            custom_scripts: None,
        }],
    });

    config
}

fn generate_api_template() -> TestConfig {
    let mut config = TestConfig::default_config();
    config.server.transport = "http".to_string();
    config.server.url = Some("http://localhost:3000".to_string());

    // Add API-specific tests
    config.test_suites.push(TestSuite {
        name: "API Integration".to_string(),
        description: Some("Test API wrapper MCP server".to_string()),
        test_cases: vec![TestCase {
            id: "health_check".to_string(),
            tool_name: "ping".to_string(),
            description: Some("Basic health check".to_string()),
            enabled: true,
            input_params: None,
            expected: Some(ExpectedResponse {
                patterns: Some(vec![ValidationPattern {
                    key: "status".to_string(),
                    validation: ValidationConfig::Equals {
                        value: serde_json::Value::String("ok".to_string()),
                    },
                    required: true,
                }]),
                allow_empty_results: Some(false),
                allow_failure: Some(false),
                performance_requirements: None,
            }),
            performance_requirements: Some(PerformanceRequirements {
                max_execution_time_ms: Some(1000),
                max_memory_usage_mb: Some(16),
            }),
            custom_scripts: None,
        }],
    });

    config
}

/// List available templates
pub fn list_available_templates(server_type: Option<&str>) -> Result<Vec<TemplateInfo>> {
    let mut templates = vec![
        TemplateInfo {
            name: "filesystem".to_string(),
            description: "Template for filesystem MCP servers".to_string(),
            server_type: "filesystem".to_string(),
        },
        TemplateInfo {
            name: "database".to_string(),
            description: "Template for database MCP servers".to_string(),
            server_type: "database".to_string(),
        },
        TemplateInfo {
            name: "api".to_string(),
            description: "Template for API wrapper MCP servers".to_string(),
            server_type: "api".to_string(),
        },
        TemplateInfo {
            name: "custom".to_string(),
            description: "Basic custom MCP server template".to_string(),
            server_type: "custom".to_string(),
        },
    ];

    if let Some(filter_type) = server_type {
        templates.retain(|t| t.server_type == filter_type);
    }

    Ok(templates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_serialization() {
        let config = TestConfig::default_config();
        let yaml = serde_yaml::to_string(&config).unwrap();
        println!("Default config YAML:\n{}", yaml);

        // Should be able to deserialize back
        let _: TestConfig = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    fn test_template_generation() {
        let filesystem_template = generate_template("filesystem").unwrap();
        assert!(!filesystem_template.is_empty());

        let database_template = generate_template("database").unwrap();
        assert!(!database_template.is_empty());

        let api_template = generate_template("api").unwrap();
        assert!(!api_template.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = TestConfig::default_config();
        let mut temp_file = NamedTempFile::new().unwrap();

        let yaml = serde_yaml::to_string(&config).unwrap();
        std::fs::write(&temp_file, yaml).unwrap();

        assert!(TestConfig::validate(temp_file.path()).is_ok());
    }
}
