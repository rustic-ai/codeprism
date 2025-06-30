//! JSON Schema validation for MCP test harness

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Validation error for specifications
#[derive(Debug, Error)]
pub enum ValidationError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// I/O error reading file
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// JSON schema validation error
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    /// Invalid format or content
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Inconsistent capabilities (e.g., claiming to support tools but defining none)
    #[error("Inconsistent capabilities: {0}")]
    InconsistentCapabilities(String),

    /// Duplicate names (tools, resources, prompts)
    #[error("Duplicate names: {0}")]
    DuplicateNames(String),

    /// Invalid test case configuration
    #[error("Invalid test case: {0}")]
    InvalidTestCase(String),

    /// General validation error
    #[error("Validation error: {0}")]
    General(String),
}

impl From<anyhow::Error> for ValidationError {
    fn from(err: anyhow::Error) -> Self {
        ValidationError::General(err.to_string())
    }
}

/// JSON Schema validator
pub struct SchemaValidator {
    // FUTURE: Add schema compilation and validation for enhanced spec validation
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a JSON value against a schema
    pub fn validate(
        &self,
        _value: &serde_json::Value,
        _schema: &serde_json::Value,
    ) -> Result<bool> {
        // FUTURE: Implement JSON schema validation using jsonschema crate for runtime validation
        Ok(true)
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Server specification schema definitions
///
/// This module defines the Rust types that correspond to the JSON schema
/// Comprehensive server specification format compatible with JSON Schema validation
/// for MCP server specifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerSpec {
    /// Human-readable name of the MCP server
    pub name: String,

    /// Semantic version of the server
    pub version: String,

    /// Optional description of the server functionality
    pub description: Option<String>,

    /// MCP capabilities supported by this server
    pub capabilities: ServerCapabilities,

    /// Server startup and connection configuration
    pub server: ServerConfig,

    /// Tool specifications (if server supports tools)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,

    /// Resource specifications (if server supports resources)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceSpec>>,

    /// Prompt specifications (if server supports prompts)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Vec<PromptSpec>>,

    /// Test execution configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_config: Option<TestConfig>,

    /// Additional metadata for the server
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Server capability configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerCapabilities {
    /// Whether the server supports tools
    pub tools: bool,
    /// Whether the server supports resources
    pub resources: bool,
    /// Whether the server supports prompts
    pub prompts: bool,
    /// Whether the server supports sampling
    pub sampling: bool,
    /// Whether the server supports logging
    pub logging: bool,
    /// Experimental features as key-value pairs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

/// Server startup and connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    /// Command to start the MCP server
    pub command: String,

    /// Arguments to pass to the server command
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables to set for the server
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Working directory for the server process
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Transport mechanism for MCP communication
    pub transport: String,

    /// Timeout for server startup in seconds
    #[serde(default = "default_startup_timeout")]
    pub startup_timeout_seconds: u32,

    /// Timeout for server shutdown in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_seconds: u32,
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSpec {
    /// Tool name as defined by the MCP server
    pub name: String,

    /// Human-readable description of the tool
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to JSON schema file for input parameters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<String>,

    /// Path to JSON schema file for output/result
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,

    /// Test cases for this tool
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSpec {
    /// URI template for the resource (RFC 6570)
    pub uri_template: String,

    /// Human-readable name for the resource
    pub name: String,

    /// MIME type of the resource content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Test cases for this resource
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Prompt specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptSpec {
    /// Prompt name as defined by the MCP server
    pub name: String,

    /// Human-readable description of the prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Arguments the prompt accepts
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,

    /// Test cases for this prompt
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Prompt argument specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,

    /// Argument description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this argument is required
    pub required: bool,
}

/// Test case specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    /// Test case name
    pub name: String,

    /// Description of what this test case validates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input parameters for the test
    pub input: serde_json::Value,

    /// Expected output specification
    pub expected: ExpectedOutput,

    /// Performance requirements for this test
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceRequirements>,

    /// Whether to skip this test case
    #[serde(default)]
    pub skip: bool,

    /// Tags for test categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Expected output specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectedOutput {
    /// Whether an error response is expected
    #[serde(default)]
    pub error: bool,

    /// Expected JSON-RPC error code (if error is true)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,

    /// Pattern that error message should contain
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message_contains: Option<String>,

    /// Path to JSON schema file for validating the result
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_file: Option<String>,

    /// Inline JSON schema for validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,

    /// Specific field validations
    #[serde(default)]
    pub fields: Vec<FieldValidation>,

    /// Whether extra fields are allowed in response
    #[serde(default = "default_allow_extra_fields")]
    pub allow_extra_fields: bool,
}

/// Field validation specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValidation {
    /// JSONPath expression for the field to validate
    pub path: String,

    /// Expected value (exact match)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Expected field type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,

    /// Whether this field is required
    #[serde(default = "default_field_required")]
    pub required: bool,

    /// Regular expression pattern for string validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Minimum value (for numbers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value (for numbers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Performance requirements for test cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceRequirements {
    /// Maximum response time in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_ms: Option<u32>,

    /// Maximum memory usage in megabytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<f64>,

    /// Minimum operations per second (for throughput tests)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_ops_per_sec: Option<f64>,
}

/// Test execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestConfig {
    /// Test timeout in seconds
    #[serde(default = "default_test_timeout")]
    pub timeout_seconds: u32,

    /// Maximum number of concurrent tests
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,

    /// Whether to stop on first failure
    #[serde(default)]
    pub fail_fast: bool,

    /// Retry configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration for failed tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u32,

    /// Whether to use exponential backoff
    #[serde(default = "default_exponential_backoff")]
    pub exponential_backoff: bool,
}

// Default value functions for serde
fn default_startup_timeout() -> u32 {
    30
}
fn default_shutdown_timeout() -> u32 {
    10
}
fn default_allow_extra_fields() -> bool {
    true
}
fn default_field_required() -> bool {
    true
}
fn default_test_timeout() -> u32 {
    30
}
fn default_max_concurrency() -> u32 {
    4
}
fn default_max_retries() -> u32 {
    2
}
fn default_retry_delay() -> u32 {
    1000
}
fn default_exponential_backoff() -> bool {
    true
}

impl ServerSpec {
    /// Create a minimal specification for protocol testing only
    pub fn minimal_protocol_spec(command: String, args: Vec<String>) -> Self {
        Self {
            name: "Unknown Server".to_string(),
            version: "unknown".to_string(),
            description: Some("Minimal specification for protocol testing".to_string()),
            capabilities: ServerCapabilities::default(),
            server: ServerConfig {
                command,
                args,
                env: HashMap::new(),
                working_dir: None,
                transport: "stdio".to_string(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },
            tools: None,
            resources: None,
            prompts: None,
            test_config: None,
            metadata: Some(HashMap::new()),
        }
    }

    /// Validate the server specification for logical consistency
    ///
    /// This performs additional validation beyond JSON schema validation,
    /// checking for logical consistency and business rules.
    ///
    /// # Errors
    /// Returns `ValidationError` if the specification has logical inconsistencies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mcp_test_harness_lib::spec::ServerSpec;
    /// # let spec: ServerSpec = ServerSpec::default(); // Example usage
    /// spec.validate().expect("Specification should be valid");
    /// ```
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate version format
        if !self.is_valid_semver(&self.version) {
            return Err(ValidationError::InvalidFormat(format!(
                "Invalid semantic version: {}",
                self.version
            )));
        }

        // Validate transport type
        match self.server.transport.as_str() {
            "stdio" | "http" | "websocket" => {}
            _ => {
                return Err(ValidationError::InvalidFormat(format!(
                    "Invalid transport type: {}",
                    self.server.transport
                )))
            }
        }

        // Validate capabilities consistency
        if self.capabilities.tools && self.tools.is_none() {
            return Err(ValidationError::InconsistentCapabilities(
                "Server claims to support tools but no tools are defined".to_string(),
            ));
        }

        if self.capabilities.resources && self.resources.is_none() {
            return Err(ValidationError::InconsistentCapabilities(
                "Server claims to support resources but no resources are defined".to_string(),
            ));
        }

        if self.capabilities.prompts && self.prompts.is_none() {
            return Err(ValidationError::InconsistentCapabilities(
                "Server claims to support prompts but no prompts are defined".to_string(),
            ));
        }

        // Validate tool names are unique
        if let Some(tools) = &self.tools {
            let mut names = std::collections::HashSet::new();
            for tool in tools {
                if !names.insert(&tool.name) {
                    return Err(ValidationError::DuplicateNames(format!(
                        "Duplicate tool name: {}",
                        tool.name
                    )));
                }

                // Validate each tool's test cases
                for test_case in &tool.tests {
                    self.validate_test_case(test_case)?;
                }
            }
        }

        // Validate resource URI templates are unique
        if let Some(resources) = &self.resources {
            let mut uri_templates = std::collections::HashSet::new();
            for resource in resources {
                if !uri_templates.insert(&resource.uri_template) {
                    return Err(ValidationError::DuplicateNames(format!(
                        "Duplicate resource URI template: {}",
                        resource.uri_template
                    )));
                }

                // Validate each resource's test cases
                for test_case in &resource.tests {
                    self.validate_test_case(test_case)?;
                }
            }
        }

        // Validate prompt names are unique
        if let Some(prompts) = &self.prompts {
            let mut names = std::collections::HashSet::new();
            for prompt in prompts {
                if !names.insert(&prompt.name) {
                    return Err(ValidationError::DuplicateNames(format!(
                        "Duplicate prompt name: {}",
                        prompt.name
                    )));
                }

                // Validate each prompt's test cases
                for test_case in &prompt.tests {
                    self.validate_test_case(test_case)?;
                }
            }
        }

        Ok(())
    }

    /// Validate an individual test case
    fn validate_test_case(&self, test_case: &TestCase) -> Result<(), ValidationError> {
        // Validate that error expectations are consistent
        if test_case.expected.error {
            if test_case.expected.schema.is_some() || !test_case.expected.fields.is_empty() {
                return Err(ValidationError::InvalidFormat(
                    format!("Test case '{}': Cannot specify success validation (schema/fields) when expecting an error", 
                           test_case.name)
                ));
            }
        } else if test_case.expected.error_code.is_some()
            || test_case.expected.error_message_contains.is_some()
        {
            return Err(ValidationError::InvalidFormat(format!(
                "Test case '{}': Cannot specify error validation when expecting success",
                test_case.name
            )));
        }

        // Validate field validation paths
        for field_validation in &test_case.expected.fields {
            if field_validation.path.is_empty() {
                return Err(ValidationError::InvalidFormat(format!(
                    "Test case '{}': Field validation path cannot be empty",
                    test_case.name
                )));
            }

            // Basic JSONPath validation - should start with $ or have proper syntax
            if !field_validation.path.starts_with('$') && !field_validation.path.starts_with('[') {
                return Err(ValidationError::InvalidFormat(format!(
                    "Test case '{}': Invalid JSONPath expression: {}",
                    test_case.name, field_validation.path
                )));
            }
        }

        // Validate performance requirements
        if let Some(perf) = &test_case.performance {
            if let Some(max_duration) = perf.max_duration_ms {
                if max_duration == 0 {
                    return Err(ValidationError::InvalidFormat(format!(
                        "Test case '{}': max_duration_ms must be greater than 0",
                        test_case.name
                    )));
                }
            }

            if let Some(max_memory) = perf.max_memory_mb {
                if max_memory <= 0.0 {
                    return Err(ValidationError::InvalidFormat(format!(
                        "Test case '{}': max_memory_mb must be greater than 0",
                        test_case.name
                    )));
                }
            }

            if let Some(min_ops) = perf.min_ops_per_sec {
                if min_ops <= 0.0 {
                    return Err(ValidationError::InvalidFormat(format!(
                        "Test case '{}': min_ops_per_sec must be greater than 0",
                        test_case.name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if a version string is a valid semantic version
    fn is_valid_semver(&self, version: &str) -> bool {
        let version_regex = regex::Regex::new(r"^\d+\.\d+\.\d+(-[\w.-]+)?(\+[\w.-]+)?$").unwrap();
        version_regex.is_match(version)
    }

    /// Get all test cases from all tools, resources, and prompts
    ///
    /// # Returns
    /// An iterator over all test cases in the specification.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mcp_test_harness_lib::spec::ServerSpec;
    /// # let spec: ServerSpec = ServerSpec::default(); // Example usage
    /// let total_tests = spec.all_test_cases().count();
    /// println!("Total test cases: {}", total_tests);
    /// ```
    pub fn all_test_cases(&self) -> impl Iterator<Item = &TestCase> {
        let tool_tests = self
            .tools
            .as_ref()
            .map(|tools| {
                tools
                    .iter()
                    .flat_map(|tool| tool.tests.iter())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let resource_tests = self
            .resources
            .as_ref()
            .map(|resources| {
                resources
                    .iter()
                    .flat_map(|resource| resource.tests.iter())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let prompt_tests = self
            .prompts
            .as_ref()
            .map(|prompts| {
                prompts
                    .iter()
                    .flat_map(|prompt| prompt.tests.iter())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        tool_tests
            .into_iter()
            .chain(resource_tests)
            .chain(prompt_tests)
    }

    /// Get test cases filtered by tags
    ///
    /// # Arguments
    /// * `tags` - Tags to filter by (test must have at least one of these tags)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mcp_test_harness_lib::spec::ServerSpec;
    /// # let spec: ServerSpec = ServerSpec::default(); // Example usage
    /// let performance_tests: Vec<_> = spec.test_cases_with_tags(&["performance", "benchmark"]).collect();
    /// ```
    pub fn test_cases_with_tags<'a>(
        &'a self,
        tags: &'a [&str],
    ) -> impl Iterator<Item = &'a TestCase> + 'a {
        self.all_test_cases().filter(move |test_case| {
            tags.iter()
                .any(|tag| test_case.tags.contains(&tag.to_string()))
        })
    }

    /// Get test cases that are not skipped
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mcp_test_harness_lib::spec::ServerSpec;
    /// # let spec: ServerSpec = ServerSpec::default(); // Example usage
    /// let runnable_tests: Vec<_> = spec.active_test_cases().collect();
    /// ```
    pub fn active_test_cases(&self) -> impl Iterator<Item = &TestCase> {
        self.all_test_cases().filter(|test_case| !test_case.skip)
    }
}

impl Default for ServerSpec {
    fn default() -> Self {
        Self {
            name: "Default Test Server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Default server specification for testing".to_string()),
            capabilities: ServerCapabilities::default(),
            server: ServerConfig {
                command: "test".to_string(),
                args: vec![],
                env: HashMap::new(),
                working_dir: None,
                transport: "stdio".to_string(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },
            tools: None,
            resources: None,
            prompts: None,
            test_config: None,
            metadata: None,
        }
    }
}
