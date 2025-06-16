//! MCP Resources implementation
//! 
//! Resources allow servers to share data that provides context to language models,
//! such as files, database schemas, or application-specific information.
//! Each resource is uniquely identified by a URI.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::GCoreMcpServer;

/// Resource capabilities as defined by MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    /// Whether the client can subscribe to be notified of changes to individual resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether the server will emit notifications when the list of available resources changes
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource (URI)
    pub uri: String,
    /// Optional human-readable name for display purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource
    #[serde(rename = "mimeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource content (for reading resources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// The resource URI
    pub uri: String,
    /// MIME type of the content
    #[serde(rename = "mimeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content (for text resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content (base64 encoded for binary resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Parameters for listing resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Result of listing resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// List of available resources
    pub resources: Vec<Resource>,
    /// Optional cursor for pagination
    #[serde(rename = "nextCursor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters for reading a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// URI of the resource to read
    pub uri: String,
}

/// Result of reading a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// List of resource content (can contain multiple items)
    pub contents: Vec<ResourceContent>,
}

/// Resource manager for MCP server
pub struct ResourceManager {
    server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>) -> Self {
        Self { server }
    }

    /// List available resources
    pub async fn list_resources(&self, _params: ListResourcesParams) -> Result<ListResourcesResult> {
        let server = self.server.read().await;
        
        let mut resources = Vec::new();
        
        // Add repository-level resources
        if let Some(repo_path) = server.repository_path() {
            // Repository root resource
            resources.push(Resource {
                uri: format!("gcore://repository/"),
                name: Some("Repository Root".to_string()),
                description: Some("Root directory of the indexed repository".to_string()),
                mime_type: Some("application/vnd.gcore.directory".to_string()),
            });

            // Repository stats resource
            resources.push(Resource {
                uri: format!("gcore://repository/stats"),
                name: Some("Repository Statistics".to_string()),
                description: Some("Statistical information about the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Repository configuration resource
            resources.push(Resource {
                uri: format!("gcore://repository/config"),
                name: Some("Repository Configuration".to_string()),
                description: Some("Configuration and metadata for the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // File tree resource
            resources.push(Resource {
                uri: format!("gcore://repository/tree"),
                name: Some("File Tree".to_string()),
                description: Some("Complete file tree structure of the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Graph resource
            resources.push(Resource {
                uri: format!("gcore://graph/repository"),
                name: Some("Repository Graph".to_string()),
                description: Some("Graph structure and statistics for the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Symbol resources by type
            resources.push(Resource {
                uri: format!("gcore://symbols/functions"),
                name: Some("Functions".to_string()),
                description: Some("All function symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: format!("gcore://symbols/classes"),
                name: Some("Classes".to_string()),
                description: Some("All class symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: format!("gcore://symbols/variables"),
                name: Some("Variables".to_string()),
                description: Some("All variable symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: format!("gcore://symbols/modules"),
                name: Some("Modules".to_string()),
                description: Some("All module symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Add individual file resources from the repository
            if let Ok(scan_result) = server.scanner().discover_files(repo_path) {
                for file_path in scan_result.iter().take(100) { // Limit to first 100 files
                    if let Ok(relative_path) = file_path.strip_prefix(repo_path) {
                        let uri = format!("gcore://repository/file/{}", relative_path.display());
                        let name = file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        let mime_type = detect_mime_type(file_path);
                        
                        resources.push(Resource {
                            uri,
                            name: Some(name),
                            description: Some(format!("Source file: {}", relative_path.display())),
                            mime_type: Some(mime_type),
                        });
                    }
                }
            }
        }

        Ok(ListResourcesResult {
            resources,
            next_cursor: None, // Simple implementation without pagination
        })
    }

    /// Read a specific resource
    pub async fn read_resource(&self, params: ReadResourceParams) -> Result<ReadResourceResult> {
        let server = self.server.read().await;
        
        let content = if params.uri.starts_with("gcore://repository/") 
            || params.uri.starts_with("gcore://graph/") 
            || params.uri.starts_with("gcore://symbols/") {
            self.handle_repository_resource(&server, &params.uri).await?
        } else {
            return Err(anyhow::anyhow!("Unsupported resource URI: {}", params.uri));
        };

        Ok(ReadResourceResult {
            contents: vec![content],
        })
    }

    /// Handle repository-specific resources
    async fn handle_repository_resource(
        &self,
        server: &GCoreMcpServer,
        uri: &str,
    ) -> Result<ResourceContent> {
        let repo_path = server.repository_path()
            .ok_or_else(|| anyhow::anyhow!("No repository initialized"))?;

        match uri {
            "gcore://repository/" => {
                // Repository root information
                let info = serde_json::json!({
                    "path": repo_path.display().to_string(),
                    "name": repo_path.file_name().and_then(|n| n.to_str()).unwrap_or("repository"),
                    "type": "repository_root"
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&info)?),
                    blob: None,
                })
            }
            
            "gcore://repository/stats" => {
                // Repository statistics
                let stats = server.repository_manager().get_total_stats();
                let stats_json = serde_json::json!({
                    "total_repositories": stats.get("repositories").unwrap_or(&0),
                    "total_files": stats.get("files").unwrap_or(&0),
                    "total_nodes": stats.get("nodes").unwrap_or(&0),
                    "total_edges": stats.get("edges").unwrap_or(&0)
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&stats_json)?),
                    blob: None,
                })
            }

            "gcore://repository/config" => {
                // Repository configuration
                let config = serde_json::json!({
                    "path": repo_path.display().to_string(),
                    "scanner_config": {
                        "supported_extensions": ["js", "ts", "py", "java"],
                        "ignore_patterns": [".git", "node_modules", "__pycache__"]
                    }
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&config)?),
                    blob: None,
                })
            }

            "gcore://repository/tree" => {
                // File tree structure
                let files = server.scanner().discover_files(repo_path)?;
                let tree = files.iter()
                    .filter_map(|path| path.strip_prefix(repo_path).ok())
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();

                let tree_json = serde_json::json!({
                    "files": tree,
                    "total_count": tree.len()
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&tree_json)?),
                    blob: None,
                })
            }

            "gcore://graph/repository" => {
                // Repository graph structure
                let graph_stats = server.graph_store().get_stats();
                let graph_json = serde_json::json!({
                    "nodes": graph_stats.total_nodes,
                    "edges": graph_stats.total_edges,
                    "files": graph_stats.total_files,
                    "nodes_by_kind": graph_stats.nodes_by_kind,
                    "last_updated": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&graph_json)?),
                    blob: None,
                })
            }

            "gcore://symbols/functions" => {
                // All function symbols
                let functions = server.graph_store().get_nodes_by_kind(gcore::NodeKind::Function);
                let functions_json = serde_json::json!(
                    functions.iter().map(|node| {
                        serde_json::json!({
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "file": node.file.display().to_string(),
                            "span": {
                                "start_line": node.span.start_line,
                                "end_line": node.span.end_line,
                                "start_column": node.span.start_column,
                                "end_column": node.span.end_column
                            },
                            "signature": node.signature,
                            "language": format!("{:?}", node.lang)
                        })
                    }).collect::<Vec<_>>()
                );

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&functions_json)?),
                    blob: None,
                })
            }

            "gcore://symbols/classes" => {
                // All class symbols
                let classes = server.graph_store().get_nodes_by_kind(gcore::NodeKind::Class);
                let classes_json = serde_json::json!(
                    classes.iter().map(|node| {
                        serde_json::json!({
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "file": node.file.display().to_string(),
                            "span": {
                                "start_line": node.span.start_line,
                                "end_line": node.span.end_line,
                                "start_column": node.span.start_column,
                                "end_column": node.span.end_column
                            },
                            "signature": node.signature,
                            "language": format!("{:?}", node.lang)
                        })
                    }).collect::<Vec<_>>()
                );

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&classes_json)?),
                    blob: None,
                })
            }

            "gcore://symbols/variables" => {
                // All variable symbols
                let variables = server.graph_store().get_nodes_by_kind(gcore::NodeKind::Variable);
                let variables_json = serde_json::json!(
                    variables.iter().map(|node| {
                        serde_json::json!({
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "file": node.file.display().to_string(),
                            "span": {
                                "start_line": node.span.start_line,
                                "end_line": node.span.end_line,
                                "start_column": node.span.start_column,
                                "end_column": node.span.end_column
                            },
                            "signature": node.signature,
                            "language": format!("{:?}", node.lang)
                        })
                    }).collect::<Vec<_>>()
                );

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&variables_json)?),
                    blob: None,
                })
            }

            "gcore://symbols/modules" => {
                // All module symbols
                let modules = server.graph_store().get_nodes_by_kind(gcore::NodeKind::Module);
                let modules_json = serde_json::json!(
                    modules.iter().map(|node| {
                        serde_json::json!({
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "file": node.file.display().to_string(),
                            "span": {
                                "start_line": node.span.start_line,
                                "end_line": node.span.end_line,
                                "start_column": node.span.start_column,
                                "end_column": node.span.end_column
                            },
                            "signature": node.signature,
                            "language": format!("{:?}", node.lang)
                        })
                    }).collect::<Vec<_>>()
                );

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&modules_json)?),
                    blob: None,
                })
            }

            uri if uri.starts_with("gcore://repository/file/") => {
                // Individual file content
                let file_path = uri.strip_prefix("gcore://repository/file/").unwrap();
                let full_path = repo_path.join(file_path);

                if !full_path.exists() {
                    return Err(anyhow::anyhow!("File not found: {}", file_path));
                }

                let content = std::fs::read_to_string(&full_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some(detect_mime_type(&full_path)),
                    text: Some(content),
                    blob: None,
                })
            }

            _ => Err(anyhow::anyhow!("Unknown resource URI: {}", uri)),
        }
    }
}

/// Detect MIME type based on file extension
fn detect_mime_type(path: &Path) -> String {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js") => "application/javascript".to_string(),
        Some("ts") => "application/typescript".to_string(),
        Some("py") => "text/x-python".to_string(),
        Some("java") => "text/x-java-source".to_string(),
        Some("json") => "application/json".to_string(),
        Some("md") => "text/markdown".to_string(),
        Some("txt") => "text/plain".to_string(),
        Some("html") => "text/html".to_string(),
        Some("css") => "text/css".to_string(),
        Some("xml") => "application/xml".to_string(),
        Some("yaml") | Some("yml") => "application/yaml".to_string(),
        Some("toml") => "application/toml".to_string(),
        _ => "text/plain".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_resource_capabilities() {
        let capabilities = ResourceCapabilities {
            subscribe: Some(true),
            list_changed: Some(true),
        };

        assert_eq!(capabilities.subscribe, Some(true));
        assert_eq!(capabilities.list_changed, Some(true));
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "gcore://repository/test.py".to_string(),
            name: Some("test.py".to_string()),
            description: Some("A Python test file".to_string()),
            mime_type: Some("text/x-python".to_string()),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.uri, deserialized.uri);
        assert_eq!(resource.name, deserialized.name);
        assert_eq!(resource.description, deserialized.description);
        assert_eq!(resource.mime_type, deserialized.mime_type);
    }

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(detect_mime_type(Path::new("test.js")), "application/javascript");
        assert_eq!(detect_mime_type(Path::new("test.py")), "text/x-python");
        assert_eq!(detect_mime_type(Path::new("test.java")), "text/x-java-source");
        assert_eq!(detect_mime_type(Path::new("test.unknown")), "text/plain");
    }

    async fn create_test_server() -> crate::GCoreMcpServer {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        
        // Create test files for comprehensive resource testing
        fs::write(repo_path.join("main.py"), r#"
class Application:
    """Main application class."""
    
    def __init__(self, name: str):
        self.name = name
        self.users = []
    
    def add_user(self, user: 'User') -> None:
        """Add a user to the application."""
        self.users.append(user)
    
    def run(self) -> None:
        """Run the application."""
        print(f"Running {self.name}")

class User:
    """User class representing a system user."""
    
    def __init__(self, username: str, email: str):
        self.username = username
        self.email = email
    
    def get_display_name(self) -> str:
        """Get the display name for the user."""
        return f"{self.username} ({self.email})"

def create_app() -> Application:
    """Create and configure the application."""
    app = Application("MyApp")
    return app

if __name__ == "__main__":
    app = create_app()
    user = User("alice", "alice@example.com")
    app.add_user(user)
    app.run()
"#).unwrap();

        fs::write(repo_path.join("utils.py"), r#"
"""Utility functions for the application."""

import os
import json
from typing import Dict, Any, List, Optional

def load_config(config_path: str) -> Dict[str, Any]:
    """Load configuration from a JSON file."""
    if not os.path.exists(config_path):
        return {}
    
    with open(config_path, 'r') as f:
        return json.load(f)

def validate_email(email: str) -> bool:
    """Simple email validation."""
    return '@' in email and '.' in email

def format_user_list(users: List['User']) -> str:
    """Format a list of users for display."""
    if not users:
        return "No users"
    
    return ', '.join(user.get_display_name() for user in users)

class ConfigManager:
    """Manages application configuration."""
    
    def __init__(self, config_path: str):
        self.config_path = config_path
        self.config = load_config(config_path)
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        return self.config.get(key, default)
    
    def set(self, key: str, value: Any) -> None:
        """Set a configuration value."""
        self.config[key] = value
"#).unwrap();

        fs::write(repo_path.join("constants.py"), r#"
"""Application constants."""

# Database configuration
DATABASE_URL = "sqlite:///app.db"
MAX_CONNECTIONS = 10

# User limits
MAX_USERNAME_LENGTH = 50
MAX_EMAIL_LENGTH = 100

# Application settings
APP_NAME = "MyApplication"
VERSION = "1.0.0"
DEBUG = False

# Feature flags
ENABLE_LOGGING = True
ENABLE_METRICS = False
ENABLE_CACHE = True
"#).unwrap();

        let mut server = crate::GCoreMcpServer::new().expect("Failed to create server");
        server.initialize_with_repository(repo_path).await
            .expect("Failed to initialize repository");
        
        // Keep temp_dir alive
        std::mem::forget(temp_dir);
        
        server
    }

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        // Resource manager should be created successfully
        assert!(true); // Just testing creation doesn't panic
    }

    #[tokio::test]
    async fn test_list_resources_with_repository() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ListResourcesParams { cursor: None };
        let result = resource_manager.list_resources(params).await;
        assert!(result.is_ok());
        
        let resources_result = result.unwrap();
        assert!(!resources_result.resources.is_empty());
        assert!(resources_result.next_cursor.is_none());
        
        // Verify we have the expected resource types
        let resource_uris: Vec<String> = resources_result.resources.iter().map(|r| r.uri.clone()).collect();
        
        // Should have repository resources
        assert!(resource_uris.iter().any(|uri| uri == "gcore://repository/"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://repository/stats"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://repository/config"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://repository/tree"));
        
        // Should have graph resources
        assert!(resource_uris.iter().any(|uri| uri == "gcore://graph/repository"));
        
        // Should have symbol resources
        assert!(resource_uris.iter().any(|uri| uri == "gcore://symbols/functions"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://symbols/classes"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://symbols/variables"));
        assert!(resource_uris.iter().any(|uri| uri == "gcore://symbols/modules"));
        
        // Should have file resources
        assert!(resource_uris.iter().any(|uri| uri.contains("main.py")));
        assert!(resource_uris.iter().any(|uri| uri.contains("utils.py")));
        assert!(resource_uris.iter().any(|uri| uri.contains("constants.py")));
    }

    #[tokio::test]
    async fn test_read_repository_root_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        assert_eq!(read_result.contents.len(), 1);
        
        let content = &read_result.contents[0];
        assert_eq!(content.uri, "gcore://repository/");
        assert_eq!(content.mime_type, Some("application/json".to_string()));
        assert!(content.text.is_some());
        
        let info: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(info["path"].is_string());
        assert_eq!(info["type"].as_str().unwrap(), "repository_root");
    }

    #[tokio::test]
    async fn test_read_repository_stats_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/stats".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        assert_eq!(read_result.contents.len(), 1);
        
        let content = &read_result.contents[0];
        assert_eq!(content.uri, "gcore://repository/stats");
        assert_eq!(content.mime_type, Some("application/json".to_string()));
        assert!(content.text.is_some());
        
        let stats: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(stats["total_files"].is_number());
        assert!(stats["total_nodes"].is_number());
        assert!(stats["total_edges"].is_number());
    }

    #[tokio::test]
    async fn test_read_repository_config_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/config".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        let config: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(config["path"].is_string());
        assert!(config["scanner_config"].is_object());
        assert!(config["scanner_config"]["supported_extensions"].is_array());
    }

    #[tokio::test]
    async fn test_read_file_tree_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/tree".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        let tree: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(tree["files"].is_array());
        assert!(tree["total_count"].is_number());
        
        let files = tree["files"].as_array().unwrap();
        assert!(files.iter().any(|f| f.as_str().unwrap().contains("main.py")));
        assert!(files.iter().any(|f| f.as_str().unwrap().contains("utils.py")));
    }

    #[tokio::test]
    async fn test_read_graph_repository_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://graph/repository".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        let graph: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(graph["nodes"].is_number());
        assert!(graph["edges"].is_number());
        assert!(graph["files"].is_number());
        assert!(graph["nodes_by_kind"].is_object());
        assert!(graph["last_updated"].is_number());
    }

    #[tokio::test]
    async fn test_read_symbols_functions_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://symbols/functions".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        let functions: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(functions.is_array());
        
        // Check structure of function entries
        if let Some(first_function) = functions.as_array().unwrap().first() {
            assert!(first_function["id"].is_string());
            assert!(first_function["name"].is_string());
            assert!(first_function["file"].is_string());
            assert!(first_function["span"].is_object());
            assert!(first_function["language"].is_string());
        }
    }

    #[tokio::test]
    async fn test_read_symbols_classes_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://symbols/classes".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        let classes: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(classes.is_array());
    }

    #[tokio::test]
    async fn test_read_file_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/file/main.py".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());
        
        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        
        assert_eq!(content.uri, "gcore://repository/file/main.py");
        assert_eq!(content.mime_type, Some("text/x-python".to_string()));
        assert!(content.text.is_some());
        
        let file_content = content.text.as_ref().unwrap();
        assert!(file_content.contains("class Application"));
        assert!(file_content.contains("class User"));
        assert!(file_content.contains("def create_app"));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/file/nonexistent.py".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("File not found"));
    }

    #[tokio::test]
    async fn test_read_unsupported_resource_uri() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "invalid://unsupported/resource".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unsupported resource URI"));
    }

    #[tokio::test]
    async fn test_read_unknown_repository_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);
        
        let params = ReadResourceParams {
            uri: "gcore://repository/unknown_resource".to_string(),
        };
        
        let result = resource_manager.read_resource(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unknown resource URI"));
    }

    #[test]
    fn test_resource_content_serialization() {
        let content = ResourceContent {
            uri: "gcore://test".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some("{}".to_string()),
            blob: None,
        };
        
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: ResourceContent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(content.uri, deserialized.uri);
        assert_eq!(content.mime_type, deserialized.mime_type);
        assert_eq!(content.text, deserialized.text);
        assert_eq!(content.blob, deserialized.blob);
    }

    #[test]
    fn test_list_resources_params_serialization() {
        let params = ListResourcesParams {
            cursor: Some("test_cursor".to_string()),
        };
        
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ListResourcesParams = serde_json::from_str(&json).unwrap();
        
        assert_eq!(params.cursor, deserialized.cursor);
    }

    #[test]
    fn test_read_resource_params_serialization() {
        let params = ReadResourceParams {
            uri: "gcore://test".to_string(),
        };
        
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ReadResourceParams = serde_json::from_str(&json).unwrap();
        
        assert_eq!(params.uri, deserialized.uri);
    }

    #[test]
    fn test_additional_mime_types() {
        assert_eq!(detect_mime_type(Path::new("config.json")), "application/json");
        assert_eq!(detect_mime_type(Path::new("README.md")), "text/markdown");
        assert_eq!(detect_mime_type(Path::new("data.xml")), "application/xml");
        assert_eq!(detect_mime_type(Path::new("config.yaml")), "application/yaml");
        assert_eq!(detect_mime_type(Path::new("config.yml")), "application/yaml");
        assert_eq!(detect_mime_type(Path::new("Cargo.toml")), "application/toml");
        assert_eq!(detect_mime_type(Path::new("index.html")), "text/html");
        assert_eq!(detect_mime_type(Path::new("styles.css")), "text/css");
        assert_eq!(detect_mime_type(Path::new("notes.txt")), "text/plain");
    }
} 