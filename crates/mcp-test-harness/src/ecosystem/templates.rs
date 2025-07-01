//! Test Suite Templates and Pattern System
//!
//! This module provides a template system for generating test suites
//! for common MCP server patterns and implementations.

use crate::spec::schema::{ServerSpec, TestCase};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Types of test templates available
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemplateType {
    /// Basic MCP server compliance template
    BasicCompliance,
    /// File system server template
    FileSystem,
    /// Database integration template
    Database,
    /// API wrapper server template
    ApiWrapper,
    /// Development tool server template
    DevelopmentTool,
    /// Custom template from file
    Custom(String),
}

/// Test template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTemplate {
    /// Template name and version
    pub name: String,
    pub version: String,
    pub description: String,

    /// Template type
    pub template_type: TemplateType,

    /// Base server specification template
    pub base_spec: ServerSpec,

    /// Template variables that can be customized
    pub variables: HashMap<String, String>,

    /// Additional test cases specific to this template
    pub additional_tests: Vec<TestCase>,

    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Template manager for creating and managing test suite templates
#[derive(Debug)]
pub struct TemplateManager {
    /// Available templates
    templates: HashMap<TemplateType, TestTemplate>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Load built-in templates
        manager.load_builtin_templates();

        manager
    }

    /// Load built-in templates
    fn load_builtin_templates(&mut self) {
        // Basic Compliance Template
        let basic_compliance = self.create_basic_compliance_template();
        self.templates
            .insert(TemplateType::BasicCompliance, basic_compliance);

        // File System Template
        let filesystem = self.create_filesystem_template();
        self.templates.insert(TemplateType::FileSystem, filesystem);

        // Database Template
        let database = self.create_database_template();
        self.templates.insert(TemplateType::Database, database);

        // API Wrapper Template
        let api_wrapper = self.create_api_wrapper_template();
        self.templates.insert(TemplateType::ApiWrapper, api_wrapper);

        // Development Tool Template
        let dev_tool = self.create_development_tool_template();
        self.templates
            .insert(TemplateType::DevelopmentTool, dev_tool);
    }

    /// Create basic MCP compliance template
    fn create_basic_compliance_template(&self) -> TestTemplate {
        TestTemplate {
            name: "Basic MCP Compliance".to_string(),
            version: "1.0.0".to_string(),
            description: "Template for testing basic MCP protocol compliance".to_string(),
            template_type: TemplateType::BasicCompliance,
            base_spec: ServerSpec {
                name: "${SERVER_NAME}".to_string(),
                version: "${SERVER_VERSION}".to_string(),
                description: Some("MCP server testing with basic compliance template".to_string()),
                capabilities: crate::spec::schema::ServerCapabilities {
                    tools: true,
                    resources: false,
                    prompts: false,
                    sampling: false,
                    logging: false,
                    experimental: None,
                },
                server: crate::spec::schema::ServerConfig {
                    command: "${SERVER_COMMAND}".to_string(),
                    args: vec!["${SERVER_ARGS}".to_string()],
                    env: HashMap::new(),
                    working_dir: None,
                    transport: "stdio".to_string(),
                    startup_timeout_seconds: 30,
                    shutdown_timeout_seconds: 10,
                },
                tools: None, // Will be discovered
                resources: None,
                prompts: None,
                test_config: Some(crate::spec::schema::TestConfig {
                    timeout_seconds: 30,
                    max_concurrency: 4,
                    fail_fast: false,
                    retry: Some(crate::spec::schema::RetryConfig {
                        max_retries: 2,
                        retry_delay_ms: 1000,
                        exponential_backoff: true,
                    }),
                }),
                metadata: None,
            },
            variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_NAME".to_string(), "Test Server".to_string());
                vars.insert("SERVER_VERSION".to_string(), "1.0.0".to_string());
                vars.insert("SERVER_COMMAND".to_string(), "node".to_string());
                vars.insert("SERVER_ARGS".to_string(), "server.js".to_string());
                vars
            },
            additional_tests: vec![TestCase {
                name: "protocol_initialization".to_string(),
                description: Some("Test MCP protocol initialization handshake".to_string()),
                input: serde_json::json!({}),
                expected: crate::spec::schema::ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["protocol".to_string(), "compliance".to_string()],
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::json!("compliance"));
                meta.insert("difficulty".to_string(), serde_json::json!("basic"));
                meta
            },
        }
    }

    /// Create filesystem server template
    fn create_filesystem_template(&self) -> TestTemplate {
        TestTemplate {
            name: "File System Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Template for testing file system MCP servers".to_string(),
            template_type: TemplateType::FileSystem,
            base_spec: ServerSpec {
                name: "${SERVER_NAME}".to_string(),
                version: "${SERVER_VERSION}".to_string(),
                description: Some("File system MCP server testing template".to_string()),
                capabilities: crate::spec::schema::ServerCapabilities {
                    tools: true,
                    resources: true,
                    prompts: false,
                    sampling: false,
                    logging: true,
                    experimental: None,
                },
                server: crate::spec::schema::ServerConfig {
                    command: "${SERVER_COMMAND}".to_string(),
                    args: vec!["${ALLOWED_PATH}".to_string()],
                    env: HashMap::new(),
                    working_dir: None,
                    transport: "stdio".to_string(),
                    startup_timeout_seconds: 30,
                    shutdown_timeout_seconds: 10,
                },
                tools: None,     // Will be discovered
                resources: None, // Will be discovered
                prompts: None,
                test_config: Some(crate::spec::schema::TestConfig {
                    timeout_seconds: 30,
                    max_concurrency: 4,
                    fail_fast: false,
                    retry: Some(crate::spec::schema::RetryConfig {
                        max_retries: 2,
                        retry_delay_ms: 1000,
                        exponential_backoff: true,
                    }),
                }),
                metadata: Some({
                    let mut meta = HashMap::new();
                    meta.insert(
                        "security_features".to_string(),
                        serde_json::json!(["path_restriction", "file_validation"]),
                    );
                    meta
                }),
            },
            variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_NAME".to_string(), "Filesystem Server".to_string());
                vars.insert("SERVER_VERSION".to_string(), "1.0.0".to_string());
                vars.insert("SERVER_COMMAND".to_string(), "npx".to_string());
                vars.insert("ALLOWED_PATH".to_string(), "/tmp/test".to_string());
                vars
            },
            additional_tests: vec![TestCase {
                name: "file_security_test".to_string(),
                description: Some("Test file access security restrictions".to_string()),
                input: serde_json::json!({"path": "/etc/passwd"}),
                expected: crate::spec::schema::ExpectedOutput {
                    error: true,
                    error_code: Some(-32602),
                    error_message_contains: Some("access denied".to_string()),
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["security".to_string(), "filesystem".to_string()],
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::json!("filesystem"));
                meta.insert("security_focus".to_string(), serde_json::json!(true));
                meta
            },
        }
    }

    /// Create database server template
    fn create_database_template(&self) -> TestTemplate {
        TestTemplate {
            name: "Database Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Template for testing database MCP servers".to_string(),
            template_type: TemplateType::Database,
            base_spec: ServerSpec {
                name: "${SERVER_NAME}".to_string(),
                version: "${SERVER_VERSION}".to_string(),
                description: Some("Database MCP server testing template".to_string()),
                capabilities: crate::spec::schema::ServerCapabilities {
                    tools: true,
                    resources: true,
                    prompts: false,
                    sampling: false,
                    logging: true,
                    experimental: None,
                },
                server: crate::spec::schema::ServerConfig {
                    command: "${SERVER_COMMAND}".to_string(),
                    args: vec!["${DATABASE_URL}".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("DATABASE_URL".to_string(), "${DATABASE_URL}".to_string());
                        env
                    },
                    working_dir: None,
                    transport: "stdio".to_string(),
                    startup_timeout_seconds: 45,
                    shutdown_timeout_seconds: 15,
                },
                tools: None,     // Will be discovered
                resources: None, // Will be discovered
                prompts: None,
                test_config: Some(crate::spec::schema::TestConfig {
                    timeout_seconds: 60,
                    max_concurrency: 2,
                    fail_fast: false,
                    retry: Some(crate::spec::schema::RetryConfig {
                        max_retries: 3,
                        retry_delay_ms: 2000,
                        exponential_backoff: true,
                    }),
                }),
                metadata: Some({
                    let mut meta = HashMap::new();
                    meta.insert(
                        "database_types".to_string(),
                        serde_json::json!(["sqlite", "postgresql", "mysql"]),
                    );
                    meta
                }),
            },
            variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_NAME".to_string(), "Database Server".to_string());
                vars.insert("SERVER_VERSION".to_string(), "1.0.0".to_string());
                vars.insert("SERVER_COMMAND".to_string(), "python".to_string());
                vars.insert("DATABASE_URL".to_string(), "sqlite:///test.db".to_string());
                vars
            },
            additional_tests: vec![TestCase {
                name: "sql_injection_protection".to_string(),
                description: Some("Test SQL injection protection".to_string()),
                input: serde_json::json!({"query": "SELECT * FROM users WHERE id = '1; DROP TABLE users; --'"}),
                expected: crate::spec::schema::ExpectedOutput {
                    error: true,
                    error_code: Some(-32602),
                    error_message_contains: Some("invalid query".to_string()),
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec![
                    "security".to_string(),
                    "database".to_string(),
                    "sql".to_string(),
                ],
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::json!("database"));
                meta.insert("security_focus".to_string(), serde_json::json!(true));
                meta
            },
        }
    }

    /// Create API wrapper template
    fn create_api_wrapper_template(&self) -> TestTemplate {
        TestTemplate {
            name: "API Wrapper Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Template for testing API wrapper MCP servers".to_string(),
            template_type: TemplateType::ApiWrapper,
            base_spec: ServerSpec {
                name: "${SERVER_NAME}".to_string(),
                version: "${SERVER_VERSION}".to_string(),
                description: Some("API wrapper MCP server testing template".to_string()),
                capabilities: crate::spec::schema::ServerCapabilities {
                    tools: true,
                    resources: false,
                    prompts: true,
                    sampling: false,
                    logging: true,
                    experimental: None,
                },
                server: crate::spec::schema::ServerConfig {
                    command: "${SERVER_COMMAND}".to_string(),
                    args: vec!["${API_MODULE}".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("API_KEY".to_string(), "${API_KEY}".to_string());
                        env.insert("API_BASE_URL".to_string(), "${API_BASE_URL}".to_string());
                        env
                    },
                    working_dir: None,
                    transport: "stdio".to_string(),
                    startup_timeout_seconds: 30,
                    shutdown_timeout_seconds: 10,
                },
                tools: None, // Will be discovered
                resources: None,
                prompts: None, // Will be discovered
                test_config: Some(crate::spec::schema::TestConfig {
                    timeout_seconds: 45,
                    max_concurrency: 3,
                    fail_fast: false,
                    retry: Some(crate::spec::schema::RetryConfig {
                        max_retries: 3,
                        retry_delay_ms: 1500,
                        exponential_backoff: true,
                    }),
                }),
                metadata: Some({
                    let mut meta = HashMap::new();
                    meta.insert(
                        "rate_limits".to_string(),
                        serde_json::json!({"requests_per_minute": 100}),
                    );
                    meta
                }),
            },
            variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_NAME".to_string(), "API Wrapper".to_string());
                vars.insert("SERVER_VERSION".to_string(), "1.0.0".to_string());
                vars.insert("SERVER_COMMAND".to_string(), "python".to_string());
                vars.insert("API_MODULE".to_string(), "api_wrapper_server".to_string());
                vars.insert("API_KEY".to_string(), "test_key".to_string());
                vars.insert(
                    "API_BASE_URL".to_string(),
                    "https://api.example.com".to_string(),
                );
                vars
            },
            additional_tests: vec![TestCase {
                name: "api_rate_limit_test".to_string(),
                description: Some("Test API rate limiting behavior".to_string()),
                input: serde_json::json!({"requests": 10, "interval": "1s"}),
                expected: crate::spec::schema::ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: Some(crate::spec::schema::PerformanceRequirements {
                    max_duration_ms: Some(2000),
                    max_memory_mb: Some(50.0),
                    min_ops_per_sec: Some(5.0),
                }),
                skip: false,
                tags: vec![
                    "performance".to_string(),
                    "api".to_string(),
                    "rate_limit".to_string(),
                ],
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::json!("api"));
                meta.insert("external_dependencies".to_string(), serde_json::json!(true));
                meta
            },
        }
    }

    /// Create development tool template
    fn create_development_tool_template(&self) -> TestTemplate {
        TestTemplate {
            name: "Development Tool Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Template for testing development tool MCP servers".to_string(),
            template_type: TemplateType::DevelopmentTool,
            base_spec: ServerSpec {
                name: "${SERVER_NAME}".to_string(),
                version: "${SERVER_VERSION}".to_string(),
                description: Some("Development tool MCP server testing template".to_string()),
                capabilities: crate::spec::schema::ServerCapabilities {
                    tools: true,
                    resources: true,
                    prompts: true,
                    sampling: false,
                    logging: true,
                    experimental: Some({
                        let mut exp = HashMap::new();
                        exp.insert("code_analysis".to_string(), serde_json::json!(true));
                        exp
                    }),
                },
                server: crate::spec::schema::ServerConfig {
                    command: "${SERVER_COMMAND}".to_string(),
                    args: vec!["${WORKSPACE_PATH}".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert(
                            "WORKSPACE_PATH".to_string(),
                            "${WORKSPACE_PATH}".to_string(),
                        );
                        env
                    },
                    working_dir: Some("${WORKSPACE_PATH}".to_string()),
                    transport: "stdio".to_string(),
                    startup_timeout_seconds: 60,
                    shutdown_timeout_seconds: 30,
                },
                tools: None,     // Will be discovered
                resources: None, // Will be discovered
                prompts: None,   // Will be discovered
                test_config: Some(crate::spec::schema::TestConfig {
                    timeout_seconds: 120,
                    max_concurrency: 2,
                    fail_fast: false,
                    retry: Some(crate::spec::schema::RetryConfig {
                        max_retries: 1,
                        retry_delay_ms: 3000,
                        exponential_backoff: false,
                    }),
                }),
                metadata: Some({
                    let mut meta = HashMap::new();
                    meta.insert(
                        "development_features".to_string(),
                        serde_json::json!(["code_analysis", "refactoring", "testing"]),
                    );
                    meta
                }),
            },
            variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_NAME".to_string(), "Development Tool".to_string());
                vars.insert("SERVER_VERSION".to_string(), "1.0.0".to_string());
                vars.insert(
                    "SERVER_COMMAND".to_string(),
                    "codeprism-mcp-server".to_string(),
                );
                vars.insert("WORKSPACE_PATH".to_string(), "./workspace".to_string());
                vars
            },
            additional_tests: vec![TestCase {
                name: "code_analysis_performance".to_string(),
                description: Some("Test code analysis performance on large codebase".to_string()),
                input: serde_json::json!({"path": "src/", "analysis_type": "complexity"}),
                expected: crate::spec::schema::ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: Some(crate::spec::schema::PerformanceRequirements {
                    max_duration_ms: Some(30000), // 30 seconds
                    max_memory_mb: Some(200.0),
                    min_ops_per_sec: Some(1.0),
                }),
                skip: false,
                tags: vec![
                    "performance".to_string(),
                    "development".to_string(),
                    "analysis".to_string(),
                ],
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::json!("development"));
                meta.insert("performance_critical".to_string(), serde_json::json!(true));
                meta
            },
        }
    }

    /// Get available template types
    pub fn available_templates(&self) -> Vec<&TemplateType> {
        self.templates.keys().collect()
    }

    /// Get a template by type
    pub fn get_template(&self, template_type: &TemplateType) -> Option<&TestTemplate> {
        self.templates.get(template_type)
    }

    /// Generate a server specification from a template
    pub fn generate_spec(
        &self,
        template_type: &TemplateType,
        variables: &HashMap<String, String>,
    ) -> Result<ServerSpec> {
        let template = self
            .templates
            .get(template_type)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {:?}", template_type))?;

        // Serialize template to JSON and substitute variables
        let mut spec_json = serde_json::to_string(&template.base_spec)?;

        // Substitute template variables
        for (key, value) in variables {
            let template_var = format!("${{{}}}", key);
            spec_json = spec_json.replace(&template_var, value);
        }

        // Deserialize back to ServerSpec
        let spec: ServerSpec = serde_json::from_str(&spec_json)?;

        Ok(spec)
    }

    /// Load custom template from file
    pub async fn load_template<P: AsRef<Path>>(&mut self, path: P) -> Result<TemplateType> {
        let content = tokio::fs::read_to_string(&path).await?;
        let template: TestTemplate = serde_yaml::from_str(&content)?;

        let template_type = TemplateType::Custom(path.as_ref().to_string_lossy().to_string());
        self.templates.insert(template_type.clone(), template);

        Ok(template_type)
    }

    /// Save template to file
    pub async fn save_template<P: AsRef<Path>>(
        &self,
        template_type: &TemplateType,
        path: P,
    ) -> Result<()> {
        let template = self
            .templates
            .get(template_type)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {:?}", template_type))?;

        let yaml_content = serde_yaml::to_string(template)?;
        tokio::fs::write(path, yaml_content).await?;

        Ok(())
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
