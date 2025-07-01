//! Popular MCP Server Configurations and Profiles

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Popular MCP servers registry
#[derive(Debug, Clone)]
pub struct PopularServers {
    servers: HashMap<String, ServerProfile>,
}

/// Server profile containing configuration and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub repository_url: Option<String>,
    pub documentation_url: Option<String>,
    pub server_config: ServerConfig,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub popularity_score: f64,
}

/// Server configuration for popular servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub installation_instructions: Vec<String>,
}

/// Server template for generating test configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTemplate {
    pub server_name: String,
    pub template_content: String,
    pub required_variables: Vec<String>,
    pub optional_variables: HashMap<String, String>,
}

impl PopularServers {
    /// Create a new registry with popular servers
    pub fn new() -> Self {
        let mut registry = Self {
            servers: HashMap::new(),
        };
        registry.load_popular_servers();
        registry
    }

    /// Load popular server profiles
    fn load_popular_servers(&mut self) {
        // CodePrism MCP Server
        let codeprism = ServerProfile {
            name: "CodePrism MCP Server".to_string(),
            description: "Advanced code analysis and development tools MCP server".to_string(),
            version: "1.0.0".to_string(),
            author: "CodePrism Team".to_string(),
            repository_url: Some("https://github.com/rustic-ai/codeprism".to_string()),
            documentation_url: Some("https://codeprism.dev/mcp".to_string()),
            server_config: ServerConfig {
                command: "codeprism-mcp-server".to_string(),
                args: vec!["--workspace".to_string(), ".".to_string()],
                env: {
                    let mut env = HashMap::new();
                    env.insert("RUST_LOG".to_string(), "info".to_string());
                    env
                },
                working_dir: None,
                installation_instructions: vec![
                    "cargo install codeprism-mcp-server".to_string(),
                    "Or download from GitHub releases".to_string(),
                ],
            },
            capabilities: vec![
                "code_analysis".to_string(),
                "refactoring".to_string(),
                "pattern_detection".to_string(),
                "dependency_analysis".to_string(),
            ],
            tags: vec![
                "development".to_string(),
                "code_analysis".to_string(),
                "rust".to_string(),
            ],
            popularity_score: 8.5,
        };
        self.servers.insert("codeprism".to_string(), codeprism);

        // Filesystem MCP Server
        let filesystem = ServerProfile {
            name: "Filesystem MCP Server".to_string(),
            description: "Secure file system operations with path restrictions".to_string(),
            version: "1.1.0".to_string(),
            author: "Anthropic".to_string(),
            repository_url: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            documentation_url: Some(
                "https://modelcontextprotocol.io/servers/filesystem".to_string(),
            ),
            server_config: ServerConfig {
                command: "npx".to_string(),
                args: vec![
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    "/allowed/path".to_string(),
                ],
                env: HashMap::new(),
                working_dir: None,
                installation_instructions: vec![
                    "npm install -g @modelcontextprotocol/server-filesystem".to_string(),
                ],
            },
            capabilities: vec![
                "file_operations".to_string(),
                "directory_listing".to_string(),
                "file_search".to_string(),
                "path_restriction".to_string(),
            ],
            tags: vec![
                "filesystem".to_string(),
                "security".to_string(),
                "node".to_string(),
            ],
            popularity_score: 9.2,
        };
        self.servers.insert("filesystem".to_string(), filesystem);

        // SQLite MCP Server
        let sqlite = ServerProfile {
            name: "SQLite MCP Server".to_string(),
            description: "SQLite database operations and query execution".to_string(),
            version: "1.0.2".to_string(),
            author: "Anthropic".to_string(),
            repository_url: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            documentation_url: Some("https://modelcontextprotocol.io/servers/sqlite".to_string()),
            server_config: ServerConfig {
                command: "npx".to_string(),
                args: vec![
                    "@modelcontextprotocol/server-sqlite".to_string(),
                    "database.db".to_string(),
                ],
                env: HashMap::new(),
                working_dir: None,
                installation_instructions: vec![
                    "npm install -g @modelcontextprotocol/server-sqlite".to_string(),
                ],
            },
            capabilities: vec![
                "sql_execution".to_string(),
                "schema_introspection".to_string(),
                "data_querying".to_string(),
                "transaction_support".to_string(),
            ],
            tags: vec![
                "database".to_string(),
                "sqlite".to_string(),
                "sql".to_string(),
                "node".to_string(),
            ],
            popularity_score: 8.8,
        };
        self.servers.insert("sqlite".to_string(), sqlite);

        // Weather MCP Server (Example API wrapper)
        let weather = ServerProfile {
            name: "Weather MCP Server".to_string(),
            description: "Weather information API wrapper for location-based queries".to_string(),
            version: "1.0.0".to_string(),
            author: "Community".to_string(),
            repository_url: Some("https://github.com/mcp-community/weather-server".to_string()),
            documentation_url: None,
            server_config: ServerConfig {
                command: "python".to_string(),
                args: vec!["-m".to_string(), "weather_mcp_server".to_string()],
                env: {
                    let mut env = HashMap::new();
                    env.insert(
                        "WEATHER_API_KEY".to_string(),
                        "${WEATHER_API_KEY}".to_string(),
                    );
                    env
                },
                working_dir: None,
                installation_instructions: vec![
                    "pip install weather-mcp-server".to_string(),
                    "Set WEATHER_API_KEY environment variable".to_string(),
                ],
            },
            capabilities: vec![
                "weather_queries".to_string(),
                "location_lookup".to_string(),
                "forecast_data".to_string(),
                "historical_data".to_string(),
            ],
            tags: vec![
                "api_wrapper".to_string(),
                "weather".to_string(),
                "python".to_string(),
            ],
            popularity_score: 7.5,
        };
        self.servers.insert("weather".to_string(), weather);
    }

    /// Get all available servers
    pub fn list_servers(&self) -> Vec<&ServerProfile> {
        self.servers.values().collect()
    }

    /// Get server by name
    pub fn get_server(&self, name: &str) -> Option<&ServerProfile> {
        self.servers.get(name)
    }

    /// Search servers by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&ServerProfile> {
        self.servers
            .values()
            .filter(|server| server.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get top servers by popularity
    pub fn top_servers(&self, limit: usize) -> Vec<&ServerProfile> {
        let mut servers: Vec<&ServerProfile> = self.servers.values().collect();
        servers.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap());
        servers.into_iter().take(limit).collect()
    }

    /// Generate test template for server
    pub fn generate_template(&self, server_name: &str) -> Result<ServerTemplate> {
        let server = self
            .servers
            .get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server not found: {}", server_name))?;

        // Generate YAML template
        let template_content = format!(
            r#"# Test configuration for {}
name: "{}"
version: "${{SERVER_VERSION}}"
description: "{}"

capabilities:
  tools: true
  resources: true
  prompts: false
  sampling: false
  logging: true

server:
  command: "{}"
  args: {:?}
  env: {}
  transport: "stdio"
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10

test_config:
  timeout_seconds: 60
  max_concurrency: 4
  fail_fast: false
"#,
            server.name,
            server.name,
            server.description,
            server.server_config.command,
            server.server_config.args,
            serde_yaml::to_string(&server.server_config.env).unwrap_or_default(),
        );

        Ok(ServerTemplate {
            server_name: server_name.to_string(),
            template_content,
            required_variables: vec!["SERVER_VERSION".to_string()],
            optional_variables: {
                let mut vars = HashMap::new();
                vars.insert("SERVER_VERSION".to_string(), server.version.clone());
                vars
            },
        })
    }
}

impl Default for PopularServers {
    fn default() -> Self {
        Self::new()
    }
}
