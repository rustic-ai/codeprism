//! YAML specification parser and validation module
//!
//! This module provides comprehensive YAML parsing and validation for MCP test specifications.
//! It supports the complete specification format including server configuration, test cases,
//! validation rules, and metadata.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Complete test specification parsed from YAML
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestSpecification {
    /// Human-readable name of the MCP server
    pub name: String,
    /// Semantic version of the server
    pub version: String,
    /// Optional description
    pub description: Option<String>,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server configuration
    pub server: ServerConfig,
    /// Tool specifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    /// Resource specifications  
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceSpec>>,
    /// Prompt specifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Vec<PromptSpec>>,
    /// Test configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_config: Option<TestConfig>,
    /// Additional metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Server capability configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
    pub sampling: bool,
    pub logging: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

/// Server startup and connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    pub transport: String,
    #[serde(default = "default_startup_timeout")]
    pub startup_timeout_seconds: u32,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_seconds: u32,
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSpec {
    pub uri_template: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Prompt specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

/// Prompt argument specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
}

/// Test case specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input: serde_json::Value,
    pub expected: ExpectedOutput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceRequirements>,
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Expected output specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectedOutput {
    #[serde(default)]
    pub error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message_contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub fields: Vec<FieldValidation>,
    #[serde(default = "default_allow_extra_fields")]
    pub allow_extra_fields: bool,
}

/// Field validation specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValidation {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    #[serde(default = "default_field_required")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Types of field validation that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldValidationType {
    /// Check if field exists
    Exists,
    /// Check exact value equality
    Equals,
    /// Check field type (string, number, boolean, array, object)
    Type,
    /// Check pattern match (regex)
    Pattern,
    /// Check value range (for numbers)
    Range,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceRequirements {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_ops_per_sec: Option<f64>,
}

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestConfig {
    #[serde(default = "default_test_timeout")]
    pub timeout_seconds: u32,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    #[serde(default)]
    pub fail_fast: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u32,
    #[serde(default = "default_exponential_backoff")]
    pub exponential_backoff: bool,
}

/// Validation error for specifications
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yml::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Inconsistent capabilities: {0}")]
    InconsistentCapabilities(String),
    #[error("Duplicate names: {0}")]
    DuplicateNames(String),
    #[error("Invalid test case: {0}")]
    InvalidTestCase(String),
    #[error("Validation error: {0}")]
    General(String),
}

/// Comprehensive YAML specification loader with validation
pub struct SpecificationLoader {
    /// Base directory for resolving relative paths
    base_dir: Option<PathBuf>,
}

impl SpecificationLoader {
    /// Create a new specification loader
    pub fn new() -> Result<Self> {
        Ok(Self { base_dir: None })
    }

    /// Create a loader with a base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        Ok(Self {
            base_dir: Some(base_dir.as_ref().to_path_buf()),
        })
    }

    /// Load a specification from a YAML file
    pub async fn load_from_file(&self, path: &Path) -> Result<TestSpecification> {
        // Resolve path relative to base directory if needed
        let resolved_path = if path.is_relative() {
            if let Some(base) = &self.base_dir {
                base.join(path)
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        // Check if file exists
        if !resolved_path.exists() {
            return Err(crate::error::Error::spec(format!(
                "File not found: {}",
                resolved_path.display()
            )));
        }

        // Read file content
        let content = tokio::fs::read_to_string(&resolved_path)
            .await
            .map_err(|e| crate::error::Error::spec(format!("Failed to read file: {}", e)))?;

        // Parse YAML content
        self.parse_yaml(&content)
    }

    /// Load multiple specifications from a directory
    pub async fn load_from_directory(&self, path: &Path) -> Result<Vec<TestSpecification>> {
        if !path.is_dir() {
            return Err(crate::error::Error::spec(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        let mut specs = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| crate::error::Error::spec(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| {
            crate::error::Error::spec(format!("Failed to read directory entry: {}", e))
        })? {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                    if matches!(extension.to_lowercase().as_str(), "yaml" | "yml" | "json") {
                        match self.load_from_file(&file_path).await {
                            Ok(spec) => specs.push(spec),
                            Err(_) => {
                                // Skip files that can't be parsed
                                continue;
                            }
                        }
                    }
                }
            }
        }

        Ok(specs)
    }

    /// Validate a specification against schema
    pub fn validate_specification(&self, spec: &TestSpecification) -> Result<()> {
        // Basic validation: check for inconsistent capabilities
        if spec.capabilities.tools && spec.tools.is_none() {
            return Err(crate::error::Error::spec(
                "Server claims to support tools but defines no tools".to_string(),
            ));
        }

        if spec.capabilities.resources && spec.resources.is_none() {
            return Err(crate::error::Error::spec(
                "Server claims to support resources but defines no resources".to_string(),
            ));
        }

        if spec.capabilities.prompts && spec.prompts.is_none() {
            return Err(crate::error::Error::spec(
                "Server claims to support prompts but defines no prompts".to_string(),
            ));
        }

        Ok(())
    }

    /// Parse YAML content into a specification
    pub fn parse_yaml(&self, content: &str) -> Result<TestSpecification> {
        // Try YAML first
        match serde_yml::from_str::<TestSpecification>(content) {
            Ok(spec) => Ok(spec),
            Err(yaml_err) => {
                // Try JSON as fallback
                match serde_json::from_str::<TestSpecification>(content) {
                    Ok(spec) => Ok(spec),
                    Err(_) => {
                        // Return the YAML error since it's more likely
                        Err(crate::error::Error::Yaml(yaml_err))
                    }
                }
            }
        }
    }
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================

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
    60
}
fn default_max_concurrency() -> u32 {
    4
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u32 {
    1000
}
fn default_exponential_backoff() -> bool {
    true
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for ExpectedOutput {
    fn default() -> Self {
        Self {
            error: false,
            error_code: None,
            error_message_contains: None,
            schema_file: None,
            schema: None,
            fields: Vec::new(),
            allow_extra_fields: true,
        }
    }
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            input: serde_json::Value::Null,
            expected: ExpectedOutput::default(),
            performance: None,
            skip: false,
            tags: Vec::new(),
        }
    }
}

// ============================================================================
// COMPREHENSIVE FAILING TESTS (TDD RED PHASE)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ========================================================================
    // PHASE 1: Basic YAML Parsing Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[tokio::test]
    async fn test_load_minimal_yaml_specification() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Create minimal valid YAML
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
name: "Test Server"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
"#
        )
        .unwrap();

        let spec = loader.load_from_file(temp_file.path()).await.unwrap();
        assert_eq!(spec.name, "Test Server");
        assert_eq!(spec.version, "1.0.0");
        assert!(!spec.capabilities.tools);
        assert_eq!(spec.server.command, "test-server");
        assert_eq!(spec.server.transport, "stdio");
    }

    #[tokio::test]
    async fn test_load_comprehensive_yaml_specification() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Create comprehensive YAML with tools, resources, prompts
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
name: "Comprehensive Test Server"
version: "2.1.0"
description: "A server for comprehensive testing"
capabilities:
  tools: true
  resources: true
  prompts: true
  sampling: false
  logging: true
  experimental:
    custom_feature: true
server:
  command: "python"
  args: ["server.py", "--port", "8080"]
  env:
    DEBUG: "true"
    LOG_LEVEL: "info"
  transport: "stdio"
  startup_timeout_seconds: 45
  shutdown_timeout_seconds: 15
tools:
  - name: "calculate"
    description: "Perform mathematical calculations"
    input_schema: "schemas/calculate-input.json"
    output_schema: "schemas/calculate-output.json"
    tests:
      - name: "basic_addition"
        description: "Test simple addition"
        input:
          operation: "add"
          operands: [2, 3]
        expected:
          error: false
          fields:
            - path: "$.result"
              value: 5
              required: true
        tags: ["math", "basic"]
resources:
  - name: "file_content"
    uri_template: "file:///{{path}}"
    mime_type: "text/plain"
    tests:
      - name: "read_text_file"
        input:
          path: "test.txt"
        expected:
          error: false
          fields:
            - path: "$.content"
              field_type: "string"
              required: true
prompts:
  - name: "code_review"
    description: "Review code for quality and issues"
    arguments:
      - name: "code"
        description: "The code to review"
        required: true
      - name: "language"
        description: "Programming language"
        required: false
    tests:
      - name: "review_python_code"
        input:
          code: "def hello(): print('Hello')"
          language: "python"
        expected:
          error: false
          fields:
            - path: "$.messages"
              field_type: "array"
              required: true
test_config:
  timeout_seconds: 120
  max_concurrency: 8
  fail_fast: true
  retry:
    max_retries: 5
    retry_delay_ms: 2000
    exponential_backoff: true
metadata:
  author: "Test Team"
  license: "MIT"
  version_info:
    build: "12345"
    commit: "abc123"
"#
        )
        .unwrap();

        let spec = loader.load_from_file(temp_file.path()).await.unwrap();

        // Validate basic fields
        assert_eq!(spec.name, "Comprehensive Test Server");
        assert_eq!(spec.version, "2.1.0");
        assert_eq!(
            spec.description,
            Some("A server for comprehensive testing".to_string())
        );

        // Validate capabilities
        assert!(spec.capabilities.tools);
        assert!(spec.capabilities.resources);
        assert!(spec.capabilities.prompts);
        assert!(!spec.capabilities.sampling);
        assert!(spec.capabilities.logging);

        // Validate server config
        assert_eq!(spec.server.command, "python");
        assert_eq!(spec.server.args, vec!["server.py", "--port", "8080"]);
        assert_eq!(spec.server.env.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(spec.server.startup_timeout_seconds, 45);

        // Validate tools
        let tools = spec.tools.as_ref().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "calculate");
        assert_eq!(tools[0].tests.len(), 1);
        assert_eq!(tools[0].tests[0].tags, vec!["math", "basic"]);

        // Validate resources
        let resources = spec.resources.as_ref().unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].name, "file_content");
        assert_eq!(resources[0].uri_template, "file:///{path}");

        // Validate prompts
        let prompts = spec.prompts.as_ref().unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "code_review");
        assert_eq!(prompts[0].arguments.len(), 2);
        assert!(prompts[0].arguments[0].required);
        assert!(!prompts[0].arguments[1].required);

        // Validate test config
        let test_config = spec.test_config.as_ref().unwrap();
        assert_eq!(test_config.timeout_seconds, 120);
        assert_eq!(test_config.max_concurrency, 8);
        assert!(test_config.fail_fast);

        let retry = test_config.retry.as_ref().unwrap();
        assert_eq!(retry.max_retries, 5);
        assert_eq!(retry.retry_delay_ms, 2000);
        assert!(retry.exponential_backoff);

        // Validate metadata
        let metadata = spec.metadata.as_ref().unwrap();
        assert_eq!(
            metadata.get("author").unwrap(),
            &serde_json::Value::String("Test Team".to_string())
        );
        assert_eq!(
            metadata.get("license").unwrap(),
            &serde_json::Value::String("MIT".to_string())
        );
    }

    // ========================================================================
    // PHASE 2: Error Handling Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");
        let result = loader.load_from_file(Path::new("nonexistent.yaml")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_invalid_yaml() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");
        let invalid_yaml = "invalid: yaml: [unclosed";
        let result = loader.parse_yaml(invalid_yaml);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_missing_required_fields() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Missing 'name' field
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
"#
        )
        .unwrap();

        let result = loader.load_from_file(temp_file.path()).await;
        assert!(result.is_err());
    }

    // ========================================================================
    // PHASE 3: Validation Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[test]
    fn test_validate_specification_success() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        let valid_spec = TestSpecification {
            name: "Valid Server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            capabilities: ServerCapabilities {
                tools: true,
                ..Default::default()
            },
            server: ServerConfig {
                command: "test-server".to_string(),
                args: vec![],
                env: HashMap::new(),
                working_dir: None,
                transport: "stdio".to_string(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },
            tools: Some(vec![ToolSpec {
                name: "test_tool".to_string(),
                description: None,
                input_schema: None,
                output_schema: None,
                tests: vec![],
            }]),
            resources: None,
            prompts: None,
            test_config: None,
            metadata: None,
        };

        let result = loader.validate_specification(&valid_spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_specification_inconsistent_capabilities() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Claims to support tools but defines no tools
        let invalid_spec = TestSpecification {
            name: "Invalid Server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            capabilities: ServerCapabilities {
                tools: true, // Claims support
                ..Default::default()
            },
            server: ServerConfig {
                command: "test-server".to_string(),
                args: vec![],
                env: HashMap::new(),
                working_dir: None,
                transport: "stdio".to_string(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },
            tools: None, // But provides no tools
            resources: None,
            prompts: None,
            test_config: None,
            metadata: None,
        };

        let result = loader.validate_specification(&invalid_spec);
        assert!(result.is_err());
    }

    // ========================================================================
    // PHASE 4: Directory Loading Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[tokio::test]
    async fn test_load_from_directory() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Create temporary directory with multiple YAML files
        let temp_dir = tempfile::tempdir().unwrap();

        // Create first spec file
        let mut file1 = std::fs::File::create(temp_dir.path().join("server1.yaml")).unwrap();
        write!(
            file1,
            r#"
name: "Server 1"
version: "1.0.0"
capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "server1"
  transport: "stdio"
"#
        )
        .unwrap();

        // Create second spec file
        let mut file2 = std::fs::File::create(temp_dir.path().join("server2.yaml")).unwrap();
        write!(
            file2,
            r#"
name: "Server 2"
version: "2.0.0"
capabilities:
  tools: false
  resources: true
  prompts: false
  sampling: false
  logging: false
server:
  command: "server2"
  transport: "stdio"
"#
        )
        .unwrap();

        let specs = loader.load_from_directory(temp_dir.path()).await.unwrap();
        assert_eq!(specs.len(), 2);

        // Verify both specs were loaded
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Server 1"));
        assert!(names.contains(&"Server 2"));
    }

    // ========================================================================
    // PHASE 5: JSON Support Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[tokio::test]
    async fn test_load_json_specification() {
        let loader = SpecificationLoader::new().expect("Failed to create loader");

        // Create JSON specification
        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(
            temp_file,
            r#"{{
  "name": "JSON Test Server",
  "version": "1.0.0",
  "capabilities": {{
    "tools": false,
    "resources": false,
    "prompts": false,
    "sampling": false,
    "logging": false
  }},
  "server": {{
    "command": "json-server",
    "transport": "stdio"
  }}
}}"#
        )
        .unwrap();

        let spec = loader.load_from_file(temp_file.path()).await.unwrap();
        assert_eq!(spec.name, "JSON Test Server");
        assert_eq!(spec.server.command, "json-server");
    }

    // ========================================================================
    // PHASE 6: Base Directory Resolution Tests (Should FAIL until GREEN phase)
    // ========================================================================

    #[tokio::test]
    async fn test_base_directory_resolution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let loader =
            SpecificationLoader::with_base_dir(temp_dir.path()).expect("Failed to create loader");

        // Create spec file with relative path
        let mut spec_file = std::fs::File::create(temp_dir.path().join("server.yaml")).unwrap();
        write!(
            spec_file,
            r#"
name: "Base Dir Test"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
"#
        )
        .unwrap();

        // Load using relative path
        let spec = loader
            .load_from_file(Path::new("server.yaml"))
            .await
            .unwrap();
        assert_eq!(spec.name, "Base Dir Test");
    }
}
