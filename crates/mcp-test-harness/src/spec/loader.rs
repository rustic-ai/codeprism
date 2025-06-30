//! Specification loader for MCP test harness
//!
//! This module provides comprehensive functionality for loading and validating
//! MCP server specifications from YAML/JSON files against a JSON schema.

use crate::spec::schema::{ServerSpec, ValidationError};
use anyhow::{Context, Result};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Specification loader with schema validation capabilities
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use mcp_test_harness_lib::spec::SpecLoader;
///
/// # tokio_test::block_on(async {
/// let loader = SpecLoader::new()?;
/// let spec = loader.load_spec("server.yaml").await?;
/// assert!(!spec.name.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
///
/// With custom schema path:
/// ```no_run
/// use mcp_test_harness_lib::spec::SpecLoader;
///
/// # tokio_test::block_on(async {
/// let loader = SpecLoader::with_schema_path("custom-schema.json").await?;
/// let spec = loader.load_spec("server.yaml").await?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub struct SpecLoader {
    /// Compiled JSON schema for validation
    schema: JSONSchema,
    /// Base directory for resolving relative paths
    base_dir: PathBuf,
}

impl SpecLoader {
    /// Create a new specification loader with the default schema
    ///
    /// This loads the embedded JSON schema for server specifications.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_test_harness_lib::spec::SpecLoader;
    ///
    /// let loader = SpecLoader::new().expect("Failed to create spec loader");
    /// ```
    pub fn new() -> Result<Self, ValidationError> {
        info!("Initializing SpecLoader with embedded schema");

        // Load the embedded schema
        let schema_content = include_str!("../../schemas/server-spec.json");
        let schema_value: Value = serde_json::from_str(schema_content)
            .context("Failed to parse embedded schema")
            .map_err(|e| ValidationError::InvalidFormat(e.to_string()))?;

        // Compile the JSON schema
        let schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
            .map_err(|e| ValidationError::SchemaValidation(e.to_string()))?;

        debug!("Successfully compiled JSON schema for validation");

        Ok(Self {
            schema,
            base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        })
    }

    /// Create a specification loader with a custom schema file
    ///
    /// # Arguments
    /// * `schema_path` - Path to the JSON schema file to use for validation
    ///
    /// # Errors
    /// Returns `ValidationError` if the schema file cannot be loaded or compiled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use mcp_test_harness_lib::spec::SpecLoader;
    ///
    /// let loader = SpecLoader::with_schema_path("my-schema.json").await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn with_schema_path<P: AsRef<Path>>(schema_path: P) -> Result<Self, ValidationError> {
        let schema_path = schema_path.as_ref();
        info!("Loading custom JSON schema from: {}", schema_path.display());

        if !schema_path.exists() {
            return Err(ValidationError::FileNotFound(schema_path.to_path_buf()));
        }

        let schema_content = tokio::fs::read_to_string(schema_path)
            .await
            .map_err(ValidationError::Io)?;

        let schema_value: Value = serde_json::from_str(&schema_content)
            .context("Failed to parse custom schema")
            .map_err(|e| ValidationError::InvalidFormat(e.to_string()))?;

        let schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
            .map_err(|e| ValidationError::SchemaValidation(e.to_string()))?;

        debug!("Successfully compiled custom JSON schema");

        Ok(Self {
            schema,
            base_dir: schema_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        })
    }

    /// Set the base directory for resolving relative paths in specifications
    ///
    /// # Arguments
    /// * `base_dir` - Base directory path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_test_harness_lib::spec::SpecLoader;
    /// use std::path::Path;
    ///
    /// let mut loader = SpecLoader::new()?;
    /// loader.set_base_dir(Path::new("/project/specs"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_base_dir<P: AsRef<Path>>(&mut self, base_dir: P) {
        self.base_dir = base_dir.as_ref().to_path_buf();
        debug!("Set base directory to: {}", self.base_dir.display());
    }

    /// Load a specification from a file with comprehensive validation
    ///
    /// This method:
    /// 1. Loads the file content
    /// 2. Parses YAML/JSON based on file extension
    /// 3. Validates against the JSON schema
    /// 4. Performs additional business logic validation
    /// 5. Resolves relative paths
    ///
    /// # Arguments
    /// * `path` - Path to the specification file
    ///
    /// # Returns
    /// A validated `ServerSpec` instance ready for use.
    ///
    /// # Errors
    /// Returns detailed `ValidationError` with context about what failed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use mcp_test_harness_lib::spec::SpecLoader;
    ///
    /// let loader = SpecLoader::new()?;
    /// let spec = loader.load_spec("examples/simple-server.yaml").await?;
    ///
    /// println!("Loaded server: {} v{}", spec.name, spec.version);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn load_spec<P: AsRef<Path>>(&self, path: P) -> Result<ServerSpec, ValidationError> {
        let path = path.as_ref();
        info!("Loading server specification from: {}", path.display());

        // Check if file exists
        if !path.exists() {
            warn!("Specification file not found: {}", path.display());
            return Err(ValidationError::FileNotFound(path.to_path_buf()));
        }

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(ValidationError::Io)
            .with_context(|| format!("Failed to read spec file: {}", path.display()))?;

        debug!("Read {} bytes from specification file", content.len());

        // Parse based on file extension
        let spec_value = self.parse_content(&content, path)?;

        // Validate against JSON schema
        self.validate_against_schema(&spec_value)
            .with_context(|| format!("Schema validation failed for: {}", path.display()))?;

        debug!("Specification passed JSON schema validation");

        // Convert to ServerSpec
        let mut spec: ServerSpec = serde_json::from_value(spec_value).map_err(|e| {
            ValidationError::InvalidFormat(format!("Failed to deserialize spec: {}", e))
        })?;

        // Perform additional validation
        spec.validate()?;

        // Resolve relative paths
        self.resolve_relative_paths(&mut spec, path)?;

        info!(
            "Successfully loaded specification: {} v{}",
            spec.name, spec.version
        );
        debug!(
            "Specification capabilities: tools={}, resources={}, prompts={}",
            spec.capabilities.tools, spec.capabilities.resources, spec.capabilities.prompts
        );

        Ok(spec)
    }

    /// Validate a specification value against the JSON schema
    ///
    /// # Arguments
    /// * `spec_value` - The parsed specification as a JSON value
    ///
    /// # Errors
    /// Returns `ValidationError::SchemaValidation` with detailed error information
    /// if the specification doesn't match the schema.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_test_harness_lib::spec::SpecLoader;
    /// use serde_json::json;
    ///
    /// let loader = SpecLoader::new()?;
    /// let spec_data = json!({
    ///     "name": "Test Server",
    ///     "version": "1.0.0",
    ///     "capabilities": {
    ///         "tools": true,
    ///         "resources": false,
    ///         "prompts": false,
    ///         "sampling": false,
    ///         "logging": false
    ///     },
    ///     "server": {
    ///         "command": "test-server",
    ///         "transport": "stdio"
    ///     }
    /// });
    ///
    /// loader.validate_against_schema(&spec_data)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate_against_schema(&self, spec_value: &Value) -> Result<(), ValidationError> {
        let validation_result = self.schema.validate(spec_value);

        if let Err(errors) = validation_result {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Path '{}': {}", error.instance_path, error))
                .collect();

            let full_error = format!("Schema validation failed:\n{}", error_messages.join("\n"));

            warn!("JSON schema validation failed: {}", full_error);
            return Err(ValidationError::SchemaValidation(full_error));
        }

        debug!("JSON schema validation passed");
        Ok(())
    }

    /// Parse file content based on the file extension
    fn parse_content(&self, content: &str, path: &Path) -> Result<Value, ValidationError> {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        debug!("Parsing file with extension: {}", extension);

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                let yaml_value: serde_yaml::Value =
                    serde_yaml::from_str(content).map_err(ValidationError::Yaml)?;
                // Convert yaml Value to json Value
                serde_json::to_value(yaml_value).map_err(ValidationError::Json)
            }
            "json" => serde_json::from_str(content).map_err(ValidationError::Json),
            _ => {
                // Try YAML first, then JSON
                debug!("Unknown extension, trying YAML first");
                if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(content) {
                    serde_json::to_value(yaml_value).map_err(ValidationError::Json)
                } else {
                    debug!("YAML parsing failed, trying JSON");
                    serde_json::from_str(content).map_err(|_| {
                        ValidationError::InvalidFormat(
                            "Content is neither valid YAML nor JSON".to_string(),
                        )
                    })
                }
            }
        }
        .map_err(|e| {
            warn!("Failed to parse content from {}: {}", path.display(), e);
            e
        })
    }

    /// Resolve relative paths in the specification relative to the spec file location
    fn resolve_relative_paths(
        &self,
        spec: &mut ServerSpec,
        spec_path: &Path,
    ) -> Result<(), ValidationError> {
        let spec_dir = spec_path.parent().unwrap_or(Path::new("."));
        debug!(
            "Resolving relative paths relative to: {}",
            spec_dir.display()
        );

        // Resolve working directory if relative
        if let Some(working_dir) = &spec.server.working_dir {
            if Path::new(working_dir).is_relative() {
                let resolved = spec_dir.join(working_dir);
                spec.server.working_dir = Some(resolved.to_string_lossy().to_string());
                debug!(
                    "Resolved working_dir to: {}",
                    spec.server.working_dir.as_ref().unwrap()
                );
            }
        }

        // Resolve schema file paths in tools
        if let Some(tools) = &mut spec.tools {
            for tool in tools {
                if let Some(input_schema) = &tool.input_schema {
                    if Path::new(input_schema).is_relative() {
                        let resolved = spec_dir.join(input_schema);
                        tool.input_schema = Some(resolved.to_string_lossy().to_string());
                    }
                }
                if let Some(output_schema) = &tool.output_schema {
                    if Path::new(output_schema).is_relative() {
                        let resolved = spec_dir.join(output_schema);
                        tool.output_schema = Some(resolved.to_string_lossy().to_string());
                    }
                }
            }
        }

        debug!("Successfully resolved all relative paths");
        Ok(())
    }

    /// Get information about the loaded schema
    pub fn schema_info(&self) -> String {
        format!(
            "JSON Schema (Draft 7) with base directory: {}",
            self.base_dir.display()
        )
    }
}

impl Default for SpecLoader {
    /// Create a new spec loader with default settings
    ///
    /// # Panics
    /// Panics if the embedded schema cannot be loaded or compiled.
    /// Use `SpecLoader::new()` for error handling.
    fn default() -> Self {
        Self::new().expect("Failed to create default SpecLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spec_loader_creation() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;
        assert!(!loader.schema_info().is_empty());
        Ok(())
    }

    #[test]
    fn test_schema_validation_success() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;

        let valid_spec = json!({
            "name": "Test Server",
            "version": "1.0.0",
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false,
                "sampling": false,
                "logging": false
            },
            "server": {
                "command": "test-server",
                "transport": "stdio"
            }
        });

        loader.validate_against_schema(&valid_spec)?;
        Ok(())
    }

    #[test]
    fn test_schema_validation_failure() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;

        let invalid_spec = json!({
            "name": "Test Server",
            // Missing required "version" field
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false,
                "sampling": false,
                "logging": false
            },
            "server": {
                "command": "test-server",
                "transport": "stdio"
            }
        });

        let result = loader.validate_against_schema(&invalid_spec);
        assert!(result.is_err());

        if let Err(ValidationError::SchemaValidation(msg)) = result {
            assert!(msg.contains("version"));
        } else {
            panic!("Expected SchemaValidation error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_yaml_file() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;

        // Create a temporary YAML file
        let mut temp_file = NamedTempFile::new()?;
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
  experimental: {{}}
server:
  command: "test-server"
  args: []
  env: {{}}
  transport: "stdio"
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10
metadata: {{}}
"#
        )?;

        let spec = loader.load_spec(temp_file.path()).await?;
        assert_eq!(spec.name, "Test Server");
        assert_eq!(spec.version, "1.0.0");
        assert!(!spec.capabilities.tools);
        assert!(!spec.capabilities.resources);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let loader = SpecLoader::new().unwrap();
        let result = loader.load_spec("nonexistent.yaml").await;

        assert!(result.is_err());
        if let Err(ValidationError::FileNotFound(_)) = result {
            // Expected error type
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    #[test]
    fn test_parse_content_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;
        let yaml_content = r#"
name: "Test"
version: "1.0.0"
"#;
        let path = Path::new("test.yaml");
        let result = loader.parse_content(yaml_content, path)?;

        assert_eq!(result["name"], "Test");
        assert_eq!(result["version"], "1.0.0");
        Ok(())
    }

    #[test]
    fn test_parse_content_json() -> Result<(), Box<dyn std::error::Error>> {
        let loader = SpecLoader::new()?;
        let json_content = r#"{"name": "Test", "version": "1.0.0"}"#;
        let path = Path::new("test.json");
        let result = loader.parse_content(json_content, path)?;

        assert_eq!(result["name"], "Test");
        assert_eq!(result["version"], "1.0.0");
        Ok(())
    }

    #[test]
    fn test_base_dir_setting() -> Result<(), Box<dyn std::error::Error>> {
        let mut loader = SpecLoader::new()?;
        let test_dir = Path::new("/tmp/test");

        loader.set_base_dir(test_dir);
        assert!(loader.schema_info().contains("/tmp/test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_example_specifications() {
        let examples = vec![
            "examples/simple-server.yaml",
            "examples/database-server.yaml",
            "examples/filesystem-server.yaml",
            "examples/api-wrapper-server.yaml",
        ];

        for example_file in examples {
            let loader = SpecLoader::new().expect("Failed to create spec loader");

            // Try to load and validate each example
            let result = loader.load_spec(example_file).await;

            match result {
                Ok(spec) => {
                    // Verify basic structure
                    assert!(
                        !spec.name.is_empty(),
                        "Example {} should have a name",
                        example_file
                    );
                    assert!(
                        !spec.version.is_empty(),
                        "Example {} should have a version",
                        example_file
                    );

                    // Verify capabilities consistency
                    let validation_result = spec.validate();
                    assert!(
                        validation_result.is_ok(),
                        "Example {} should pass validation: {:?}",
                        example_file,
                        validation_result.err()
                    );

                    println!("✅ {} validated successfully", example_file);
                }
                Err(e) => {
                    // For missing files, this is expected for some examples
                    if !e.to_string().contains("not found") {
                        panic!("Example {} failed to load: {}", example_file, e);
                    } else {
                        println!(
                            "⚠️  {} not found (expected for some examples)",
                            example_file
                        );
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_database_server_example_details() {
        let loader = SpecLoader::new().expect("Failed to create spec loader");

        if let Ok(spec) = loader.load_spec("examples/database-server.yaml").await {
            // Verify database-specific features
            assert_eq!(spec.name, "Database MCP Server");
            assert!(
                spec.capabilities.tools,
                "Database server should support tools"
            );
            assert!(
                spec.capabilities.resources,
                "Database server should support resources"
            );

            // Verify tools are defined
            assert!(
                spec.tools.is_some(),
                "Database server should have tools defined"
            );
            if let Some(tools) = &spec.tools {
                assert!(
                    !tools.is_empty(),
                    "Database server should have at least one tool"
                );

                // Check for expected database tools
                let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
                assert!(
                    tool_names.contains(&"execute_query"),
                    "Should have execute_query tool"
                );
                assert!(
                    tool_names.contains(&"get_schema"),
                    "Should have get_schema tool"
                );
            }

            println!("✅ Database server example validated with specific checks");
        }
    }

    #[tokio::test]
    async fn test_filesystem_server_example_details() {
        let loader = SpecLoader::new().expect("Failed to create spec loader");

        if let Ok(spec) = loader.load_spec("examples/filesystem-server.yaml").await {
            // Verify filesystem-specific features
            assert_eq!(spec.name, "File System MCP Server");
            assert!(
                spec.capabilities.tools,
                "Filesystem server should support tools"
            );
            assert!(
                spec.capabilities.resources,
                "Filesystem server should support resources"
            );

            // Verify security-related metadata
            if let Some(metadata) = &spec.metadata {
                if let Some(security_features) = metadata.get("security_features") {
                    let features = security_features.as_array().unwrap();
                    assert!(!features.is_empty(), "Should have security features listed");
                }
            }

            println!("✅ Filesystem server example validated with specific checks");
        }
    }

    #[tokio::test]
    async fn test_api_wrapper_example_details() {
        let loader = SpecLoader::new().expect("Failed to create spec loader");

        if let Ok(spec) = loader.load_spec("examples/api-wrapper-server.yaml").await {
            // Verify API wrapper-specific features
            assert_eq!(spec.name, "Weather API Wrapper");
            assert!(spec.capabilities.tools, "API wrapper should support tools");
            assert!(
                spec.capabilities.prompts,
                "API wrapper should support prompts"
            );

            // Verify external dependencies are documented
            if let Some(metadata) = &spec.metadata {
                if let Some(deps) = metadata.get("external_dependencies") {
                    let dependencies = deps.as_array().unwrap();
                    assert!(
                        !dependencies.is_empty(),
                        "Should have external dependencies listed"
                    );
                }
            }

            // Verify test configuration is appropriate for API calls
            if let Some(test_config) = &spec.test_config {
                assert!(
                    test_config.timeout_seconds >= 30,
                    "API operations should have adequate timeout"
                );
                assert!(
                    test_config.max_concurrency <= 2,
                    "API operations should respect rate limits"
                );
            }

            println!("✅ API wrapper example validated with specific checks");
        }
    }
}
