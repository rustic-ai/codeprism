//! Configuration management for the CodePrism MCP Server

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure for the CodePrism MCP Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Tool configuration
    pub tools: ToolsConfig,

    /// Analysis configuration
    pub analysis: AnalysisConfig,
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name for MCP identification
    pub name: String,

    /// Server version
    pub version: String,

    /// Maximum number of concurrent tool executions
    pub max_concurrent_tools: usize,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Enable core tools (file operations, content search, etc.)
    pub enable_core: bool,

    /// Enable search tools (semantic search, dependency analysis, etc.)
    pub enable_search: bool,

    /// Enable analysis tools (complexity, patterns, etc.)
    pub enable_analysis: bool,

    /// Enable workflow tools (validation, generation, etc.)
    pub enable_workflow: bool,
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Maximum file size to analyze (in bytes)
    pub max_file_size_bytes: usize,

    /// Maximum number of files to analyze in a single request
    pub max_files_per_request: usize,

    /// Enable caching of analysis results
    pub enable_caching: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                name: crate::SERVER_NAME.to_string(),
                version: crate::VERSION.to_string(),
                max_concurrent_tools: 10,
                request_timeout_secs: 30,
            },
            tools: ToolsConfig {
                enable_core: true,
                enable_search: true,
                enable_analysis: true,
                enable_workflow: true,
            },
            analysis: AnalysisConfig {
                max_file_size_bytes: 10 * 1024 * 1024, // 10MB
                max_files_per_request: 100,
                enable_caching: true,
                cache_ttl_secs: 3600, // 1 hour
            },
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let extension = path_ref.extension().and_then(|s| s.to_str());
        let content = tokio::fs::read_to_string(path_ref).await?;

        let config = match extension {
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            _ => {
                // Try to detect format by trying each parser
                toml::from_str(&content)
                    .or_else(|_| serde_yaml::from_str(&content))
                    .or_else(|_| serde_json::from_str(&content))?
            }
        };
        Ok(config)
    }

    /// Save configuration to a file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::to_string_pretty(self)?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)?,
            Some("json") => serde_json::to_string_pretty(self)?,
            _ => toml::to_string_pretty(self)?, // Default to TOML
        };
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.server.name.is_empty() {
            return Err(crate::Error::server_init("Server name cannot be empty"));
        }

        if self.server.max_concurrent_tools == 0 {
            return Err(crate::Error::server_init(
                "Max concurrent tools must be greater than 0",
            ));
        }

        if self.analysis.max_file_size_bytes == 0 {
            return Err(crate::Error::server_init(
                "Max file size must be greater than 0",
            ));
        }

        if self.analysis.max_files_per_request == 0 {
            return Err(crate::Error::server_init(
                "Max files per request must be greater than 0",
            ));
        }

        Ok(())
    }
}
