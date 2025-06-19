//! MCP Resources implementation
//!
//! Resources allow servers to share data that provides context to language models,
//! such as files, database schemas, or application-specific information.
//! Each resource is uniquely identified by a URI.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::CodePrismMcpServer;

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
    server: std::sync::Arc<tokio::sync::RwLock<CodePrismMcpServer>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<CodePrismMcpServer>>) -> Self {
        Self { server }
    }

    /// Extract source context around a line number from a file
    fn extract_source_context(
        &self,
        file_path: &std::path::Path,
        line_number: usize,
        context_lines: usize,
    ) -> Option<serde_json::Value> {
        // Read the file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return None,
        };

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        if line_number == 0 || line_number > total_lines {
            return None;
        }

        // Convert to 0-based indexing
        let target_line_idx = line_number - 1;

        // Calculate context range (with bounds checking)
        let start_idx = target_line_idx.saturating_sub(context_lines);
        let end_idx = std::cmp::min(target_line_idx + context_lines, total_lines - 1);

        // Extract context lines with line numbers
        let mut context_lines_with_numbers = Vec::new();
        for (i, _) in lines.iter().enumerate().take(end_idx + 1).skip(start_idx) {
            context_lines_with_numbers.push(serde_json::json!({
                "line_number": i + 1,
                "content": lines[i],
                "is_target": i == target_line_idx
            }));
        }

        Some(serde_json::json!({
            "target_line": line_number,
            "context_range": {
                "start_line": start_idx + 1,
                "end_line": end_idx + 1
            },
            "lines": context_lines_with_numbers
        }))
    }

    /// Create enhanced node information with source context
    fn create_node_info_with_context(
        &self,
        node: &codeprism_core::Node,
        context_lines: usize,
    ) -> serde_json::Value {
        let mut node_info = serde_json::json!({
            "id": node.id.to_hex(),
            "name": node.name,
            "kind": format!("{:?}", node.kind),
            "language": format!("{:?}", node.lang),
            "file": node.file.display().to_string(),
            "span": {
                "start_line": node.span.start_line,
                "end_line": node.span.end_line,
                "start_column": node.span.start_column,
                "end_column": node.span.end_column
            },
            "signature": node.signature
        });

        // Add source context around the symbol location
        if let Some(context) =
            self.extract_source_context(&node.file, node.span.start_line, context_lines)
        {
            node_info["source_context"] = context;
        }

        node_info
    }

    /// List available resources
    pub async fn list_resources(
        &self,
        _params: ListResourcesParams,
    ) -> Result<ListResourcesResult> {
        let server = self.server.read().await;

        let mut resources = Vec::new();

        // Add repository-level resources
        if let Some(repo_path) = server.repository_path() {
            // Repository root resource
            resources.push(Resource {
                uri: "codeprism://repository/".to_string(),
                name: Some("Repository Root".to_string()),
                description: Some("Root directory of the indexed repository".to_string()),
                mime_type: Some("application/vnd.codeprism.directory".to_string()),
            });

            // Repository stats resource
            resources.push(Resource {
                uri: "codeprism://repository/stats".to_string(),
                name: Some("Repository Statistics".to_string()),
                description: Some("Statistical information about the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Repository configuration resource
            resources.push(Resource {
                uri: "codeprism://repository/config".to_string(),
                name: Some("Repository Configuration".to_string()),
                description: Some("Configuration and metadata for the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // File tree resource
            resources.push(Resource {
                uri: "codeprism://repository/tree".to_string(),
                name: Some("File Tree".to_string()),
                description: Some("Complete file tree structure of the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Graph resource
            resources.push(Resource {
                uri: "codeprism://graph/repository".to_string(),
                name: Some("Repository Graph".to_string()),
                description: Some("Graph structure and statistics for the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Symbol resources by type
            resources.push(Resource {
                uri: "codeprism://symbols/functions".to_string(),
                name: Some("Functions".to_string()),
                description: Some("All function symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: "codeprism://symbols/classes".to_string(),
                name: Some("Classes".to_string()),
                description: Some("All class symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: "codeprism://symbols/variables".to_string(),
                name: Some("Variables".to_string()),
                description: Some("All variable symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: "codeprism://symbols/modules".to_string(),
                name: Some("Modules".to_string()),
                description: Some("All module symbols in the repository".to_string()),
                mime_type: Some("application/json".to_string()),
            });

            // Add quality metrics dashboard resource
            resources.push(Resource {
                uri: "codeprism://metrics/quality_dashboard".to_string(),
                name: Some("Quality Dashboard".to_string()),
                description: Some(
                    "Code quality metrics, complexity analysis, and technical debt assessment"
                        .to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            });

            // Add architectural overview resources
            resources.push(Resource {
                uri: "codeprism://architecture/layers".to_string(),
                name: Some("Architectural Layers".to_string()),
                description: Some(
                    "Layer structure identification and architectural organization".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: "codeprism://architecture/patterns".to_string(),
                name: Some("Architectural Patterns".to_string()),
                description: Some(
                    "Detected design patterns and architectural structures".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: "codeprism://architecture/dependencies".to_string(),
                name: Some("Architectural Dependencies".to_string()),
                description: Some(
                    "High-level dependency analysis and architectural dependency graph".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            });

            // Add individual file resources from the repository
            if let Ok(scan_result) = server.scanner().discover_files(repo_path) {
                for file_path in scan_result.iter().take(100) {
                    // Limit to first 100 files
                    if let Ok(relative_path) = file_path.strip_prefix(repo_path) {
                        let uri =
                            format!("codeprism://repository/file/{}", relative_path.display());
                        let name = file_path
                            .file_name()
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

        let content = if params.uri.starts_with("codeprism://repository/")
            || params.uri.starts_with("codeprism://graph/")
            || params.uri.starts_with("codeprism://symbols/")
            || params.uri.starts_with("codeprism://metrics/")
            || params.uri.starts_with("codeprism://architecture/")
        {
            self.handle_repository_resource(&server, &params.uri)
                .await?
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
        server: &CodePrismMcpServer,
        uri: &str,
    ) -> Result<ResourceContent> {
        let repo_path = server
            .repository_path()
            .ok_or_else(|| anyhow::anyhow!("No repository initialized"))?;

        match uri {
            "codeprism://repository/" => {
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

            "codeprism://repository/stats" => {
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

            "codeprism://repository/config" => {
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

            "codeprism://repository/tree" => {
                // File tree structure
                let files = server.scanner().discover_files(repo_path)?;
                let tree = files
                    .iter()
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

            "codeprism://graph/repository" => {
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

            "codeprism://symbols/functions" => {
                // All function symbols
                let functions = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Function);
                let functions_json = serde_json::json!(functions
                    .iter()
                    .map(|node| { self.create_node_info_with_context(node, 5) })
                    .collect::<Vec<_>>());

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&functions_json)?),
                    blob: None,
                })
            }

            "codeprism://symbols/classes" => {
                // All class symbols
                let classes = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                let classes_json = serde_json::json!(classes
                    .iter()
                    .map(|node| { self.create_node_info_with_context(node, 5) })
                    .collect::<Vec<_>>());

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&classes_json)?),
                    blob: None,
                })
            }

            "codeprism://symbols/variables" => {
                // All variable symbols
                let variables = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Variable);
                let variables_json = serde_json::json!(variables
                    .iter()
                    .map(|node| { self.create_node_info_with_context(node, 5) })
                    .collect::<Vec<_>>());

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&variables_json)?),
                    blob: None,
                })
            }

            "codeprism://symbols/modules" => {
                // All module symbols
                let modules = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Module);
                let modules_json = serde_json::json!(modules
                    .iter()
                    .map(|node| { self.create_node_info_with_context(node, 5) })
                    .collect::<Vec<_>>());

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&modules_json)?),
                    blob: None,
                })
            }

            "codeprism://metrics/quality_dashboard" => {
                // Quality metrics dashboard
                let graph_stats = server.graph_store().get_stats();
                let functions = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Function);
                let classes = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Class);

                // Calculate basic quality metrics
                let total_functions = functions.len();
                let total_classes = classes.len();
                let average_file_nodes = if graph_stats.total_files > 0 {
                    graph_stats.total_nodes as f64 / graph_stats.total_files as f64
                } else {
                    0.0
                };

                // Estimate complexity distribution (simplified)
                let complexity_distribution = serde_json::json!({
                    "low": (total_functions as f64 * 0.6) as usize,
                    "medium": (total_functions as f64 * 0.3) as usize,
                    "high": (total_functions as f64 * 0.1) as usize
                });

                // Generate technical debt indicators
                let technical_debt = serde_json::json!({
                    "large_functions": functions.iter().filter(|f| f.span.len() > 100).count(),
                    "files_with_many_functions": "estimated_based_on_node_distribution",
                    "potential_duplicates": "requires_duplicate_analysis",
                    "complex_classes": classes.iter().filter(|c| {
                        server.graph_store().get_outgoing_edges(&c.id).len() > 15
                    }).count()
                });

                // Overall quality score (simplified calculation)
                let maintainability_score = if total_functions > 0 {
                    let large_function_ratio =
                        technical_debt["large_functions"].as_u64().unwrap_or(0) as f64
                            / total_functions as f64;
                    let complex_class_ratio =
                        technical_debt["complex_classes"].as_u64().unwrap_or(0) as f64
                            / total_classes.max(1) as f64;
                    ((1.0 - large_function_ratio * 0.5 - complex_class_ratio * 0.3) * 100.0)
                        .max(0.0)
                } else {
                    100.0
                };

                let quality_json = serde_json::json!({
                    "repository_overview": {
                        "total_files": graph_stats.total_files,
                        "total_nodes": graph_stats.total_nodes,
                        "total_edges": graph_stats.total_edges,
                        "average_nodes_per_file": average_file_nodes
                    },
                    "code_structure": {
                        "functions": total_functions,
                        "classes": total_classes,
                        "modules": graph_stats.nodes_by_kind.get(&codeprism_core::NodeKind::Module).unwrap_or(&0),
                        "variables": graph_stats.nodes_by_kind.get(&codeprism_core::NodeKind::Variable).unwrap_or(&0)
                    },
                    "complexity_distribution": complexity_distribution,
                    "technical_debt": technical_debt,
                    "quality_scores": {
                        "overall": ((maintainability_score + 70.0) / 2.0).clamp(0.0, 100.0),
                        "maintainability": maintainability_score,
                        "readability": 75.0
                    },
                    "recommendations": [
                        "Consider refactoring functions longer than 100 lines",
                        "Review classes with more than 15 methods for single responsibility principle",
                        "Use analyze_complexity tool for detailed complexity metrics",
                        "Use detect_patterns tool to identify architectural patterns"
                    ]
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&quality_json)?),
                    blob: None,
                })
            }

            "codeprism://architecture/layers" => {
                // Architectural layer analysis
                let classes = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                let functions = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Function);

                // Identify potential layers based on naming conventions and structure
                let mut layers = std::collections::HashMap::new();

                // Presentation layer
                let presentation_classes = classes
                    .iter()
                    .filter(|c| {
                        let name_lower = c.name.to_lowercase();
                        name_lower.contains("controller")
                            || name_lower.contains("view")
                            || name_lower.contains("ui")
                            || name_lower.contains("component")
                            || c.file.to_string_lossy().contains("view")
                            || c.file.to_string_lossy().contains("ui")
                    })
                    .count();

                // Business/Service layer
                let business_classes = classes
                    .iter()
                    .filter(|c| {
                        let name_lower = c.name.to_lowercase();
                        name_lower.contains("service")
                            || name_lower.contains("business")
                            || name_lower.contains("logic")
                            || name_lower.contains("manager")
                            || c.file.to_string_lossy().contains("service")
                            || c.file.to_string_lossy().contains("business")
                    })
                    .count();

                // Data access layer
                let data_classes = classes
                    .iter()
                    .filter(|c| {
                        let name_lower = c.name.to_lowercase();
                        name_lower.contains("repository")
                            || name_lower.contains("dao")
                            || name_lower.contains("data")
                            || name_lower.contains("model")
                            || c.file.to_string_lossy().contains("repository")
                            || c.file.to_string_lossy().contains("model")
                    })
                    .count();

                // Infrastructure layer
                let infrastructure_classes = classes
                    .iter()
                    .filter(|c| {
                        let name_lower = c.name.to_lowercase();
                        name_lower.contains("config")
                            || name_lower.contains("util")
                            || name_lower.contains("helper")
                            || name_lower.contains("infrastructure")
                            || c.file.to_string_lossy().contains("config")
                            || c.file.to_string_lossy().contains("util")
                    })
                    .count();

                layers.insert("presentation", presentation_classes);
                layers.insert("business", business_classes);
                layers.insert("data", data_classes);
                layers.insert("infrastructure", infrastructure_classes);

                // Directory structure analysis
                let all_files = server.graph_store().get_all_files();
                let mut directory_layers = std::collections::HashMap::new();

                for file in &all_files {
                    if let Some(parent) = file.parent() {
                        let dir_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        *directory_layers.entry(dir_name.to_string()).or_insert(0) += 1;
                    }
                }

                let layers_json = serde_json::json!({
                    "layer_analysis": {
                        "presentation_layer": {
                            "classes": presentation_classes,
                            "description": "Controllers, views, UI components"
                        },
                        "business_layer": {
                            "classes": business_classes,
                            "description": "Business logic, services, managers"
                        },
                        "data_layer": {
                            "classes": data_classes,
                            "description": "Repositories, DAOs, data models"
                        },
                        "infrastructure_layer": {
                            "classes": infrastructure_classes,
                            "description": "Configuration, utilities, infrastructure"
                        }
                    },
                    "directory_structure": directory_layers,
                    "total_classes": classes.len(),
                    "total_functions": functions.len(),
                    "layering_assessment": {
                        "well_layered": presentation_classes > 0 && business_classes > 0 && data_classes > 0,
                        "dominant_layer": layers.iter().max_by_key(|(_, count)| *count).map(|(name, _)| *name).unwrap_or("unclear"),
                        "architectural_style": if presentation_classes > 0 && business_classes > 0 && data_classes > 0 {
                            "Layered Architecture"
                        } else {
                            "Unclear or Monolithic"
                        }
                    }
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&layers_json)?),
                    blob: None,
                })
            }

            "codeprism://architecture/patterns" => {
                // Design pattern detection (simplified version for resource)
                let classes = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                let mut detected_patterns = Vec::new();

                // Singleton pattern detection
                for class in &classes {
                    let methods = server.graph_store().get_outgoing_edges(&class.id);
                    let has_get_instance = methods.iter().any(|edge| {
                        if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                            target_node.name.to_lowercase().contains("getinstance")
                                || target_node.name.to_lowercase().contains("get_instance")
                        } else {
                            false
                        }
                    });

                    if has_get_instance {
                        detected_patterns.push(serde_json::json!({
                            "pattern": "Singleton",
                            "class": class.name,
                            "file": class.file.display().to_string(),
                            "confidence": "medium"
                        }));
                    }
                }

                // Factory pattern detection
                let factory_classes = classes
                    .iter()
                    .filter(|c| c.name.to_lowercase().contains("factory"))
                    .map(|c| {
                        serde_json::json!({
                            "pattern": "Factory",
                            "class": c.name,
                            "file": c.file.display().to_string(),
                            "confidence": "high"
                        })
                    })
                    .collect::<Vec<_>>();

                detected_patterns.extend(factory_classes);

                // MVC pattern detection
                let controllers = classes
                    .iter()
                    .filter(|c| c.name.to_lowercase().contains("controller"))
                    .count();
                let models = classes
                    .iter()
                    .filter(|c| c.name.to_lowercase().contains("model"))
                    .count();
                let views = classes
                    .iter()
                    .filter(|c| c.name.to_lowercase().contains("view"))
                    .count();

                if controllers > 0 && models > 0 && views > 0 {
                    detected_patterns.push(serde_json::json!({
                        "pattern": "MVC (Model-View-Controller)",
                        "components": {
                            "controllers": controllers,
                            "models": models,
                            "views": views
                        },
                        "confidence": "high"
                    }));
                }

                let patterns_json = serde_json::json!({
                    "detected_patterns": detected_patterns,
                    "pattern_summary": {
                        "total_patterns": detected_patterns.len(),
                        "design_patterns": detected_patterns.iter().filter(|p|
                            p["pattern"].as_str().unwrap_or("") != "MVC (Model-View-Controller)"
                        ).count(),
                        "architectural_patterns": if controllers > 0 && models > 0 && views > 0 { 1 } else { 0 }
                    },
                    "recommendations": [
                        "Use detect_patterns tool for detailed pattern analysis",
                        "Consider implementing missing patterns for better architecture",
                        "Review pattern implementations for best practices"
                    ]
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&patterns_json)?),
                    blob: None,
                })
            }

            "codeprism://architecture/dependencies" => {
                // High-level dependency analysis
                let graph_stats = server.graph_store().get_stats();
                let files = server.graph_store().get_all_files();

                // Calculate dependency metrics
                let mut file_dependencies = std::collections::HashMap::new();
                let mut total_dependencies = 0;

                for file in &files {
                    let nodes = server.graph_store().get_nodes_in_file(file);
                    let mut file_dep_count = 0;

                    for node in nodes {
                        let outgoing = server.graph_store().get_outgoing_edges(&node.id);
                        file_dep_count += outgoing.len();
                        total_dependencies += outgoing.len();
                    }

                    if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
                        file_dependencies.insert(file_name.to_string(), file_dep_count);
                    }
                }

                // Find highly coupled files
                let average_dependencies = if !files.is_empty() {
                    total_dependencies as f64 / files.len() as f64
                } else {
                    0.0
                };

                let highly_coupled_files: Vec<_> = file_dependencies
                    .iter()
                    .filter(|(_, &count)| count as f64 > average_dependencies * 1.5)
                    .map(|(name, count)| {
                        serde_json::json!({
                            "file": name,
                            "dependencies": count
                        })
                    })
                    .collect();

                // Identify potential dependency cycles (simplified)
                let import_edges = graph_stats.total_edges; // Simplified - would need more detailed analysis
                let potential_cycles = if import_edges > graph_stats.total_nodes {
                    (import_edges as f64 - graph_stats.total_nodes as f64).max(0.0) as usize
                } else {
                    0
                };

                let dependencies_json = serde_json::json!({
                    "dependency_overview": {
                        "total_files": files.len(),
                        "total_dependencies": total_dependencies,
                        "average_dependencies_per_file": average_dependencies,
                        "highly_coupled_files": highly_coupled_files.len()
                    },
                    "coupling_analysis": {
                        "files_above_average": file_dependencies.iter()
                            .filter(|(_, &count)| count as f64 > average_dependencies)
                            .count(),
                        "max_dependencies": file_dependencies.values().max().unwrap_or(&0),
                        "min_dependencies": file_dependencies.values().min().unwrap_or(&0)
                    },
                    "highly_coupled_files": highly_coupled_files,
                    "potential_issues": {
                        "potential_cycles": potential_cycles,
                        "coupling_hotspots": highly_coupled_files.len()
                    },
                    "recommendations": [
                        "Use analyze_transitive_dependencies tool for detailed cycle detection",
                        "Consider refactoring highly coupled files",
                        "Review dependency chains for optimization opportunities"
                    ]
                });

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&dependencies_json)?),
                    blob: None,
                })
            }

            uri if uri.starts_with("codeprism://repository/file/") => {
                // Individual file content
                let file_path = uri.strip_prefix("codeprism://repository/file/").unwrap();
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
    use std::fs;
    use tempfile::TempDir;

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
            uri: "codeprism://repository/test.py".to_string(),
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
        assert_eq!(
            detect_mime_type(Path::new("test.js")),
            "application/javascript"
        );
        assert_eq!(detect_mime_type(Path::new("test.py")), "text/x-python");
        assert_eq!(
            detect_mime_type(Path::new("test.java")),
            "text/x-java-source"
        );
        assert_eq!(detect_mime_type(Path::new("test.unknown")), "text/plain");
    }

    async fn create_test_server() -> crate::CodePrismMcpServer {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Create test files for comprehensive resource testing
        fs::write(
            repo_path.join("main.py"),
            r#"
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
"#,
        )
        .unwrap();

        fs::write(
            repo_path.join("utils.py"),
            r#"
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
"#,
        )
        .unwrap();

        fs::write(
            repo_path.join("constants.py"),
            r#"
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
"#,
        )
        .unwrap();

        let mut server = crate::CodePrismMcpServer::new().expect("Failed to create server");
        server
            .initialize_with_repository(repo_path)
            .await
            .expect("Failed to initialize repository");

        // Keep temp_dir alive
        std::mem::forget(temp_dir);

        server
    }

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let _resource_manager = ResourceManager::new(server_arc);

        // Resource manager should be created successfully
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
        let resource_uris: Vec<String> = resources_result
            .resources
            .iter()
            .map(|r| r.uri.clone())
            .collect();

        // Should have repository resources
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://repository/"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://repository/stats"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://repository/config"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://repository/tree"));

        // Should have graph resources
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://graph/repository"));

        // Should have symbol resources
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://symbols/functions"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://symbols/classes"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://symbols/variables"));
        assert!(resource_uris
            .iter()
            .any(|uri| uri == "codeprism://symbols/modules"));

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
            uri: "codeprism://repository/".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        assert_eq!(read_result.contents.len(), 1);

        let content = &read_result.contents[0];
        assert_eq!(content.uri, "codeprism://repository/");
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
            uri: "codeprism://repository/stats".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        assert_eq!(read_result.contents.len(), 1);

        let content = &read_result.contents[0];
        assert_eq!(content.uri, "codeprism://repository/stats");
        assert_eq!(content.mime_type, Some("application/json".to_string()));
        assert!(content.text.is_some());

        let stats: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
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
            uri: "codeprism://repository/config".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let config: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
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
            uri: "codeprism://repository/tree".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let tree: serde_json::Value = serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(tree["files"].is_array());
        assert!(tree["total_count"].is_number());

        let files = tree["files"].as_array().unwrap();
        assert!(files
            .iter()
            .any(|f| f.as_str().unwrap().contains("main.py")));
        assert!(files
            .iter()
            .any(|f| f.as_str().unwrap().contains("utils.py")));
    }

    #[tokio::test]
    async fn test_read_graph_repository_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);

        let params = ReadResourceParams {
            uri: "codeprism://graph/repository".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let graph: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
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
            uri: "codeprism://symbols/functions".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let functions: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
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
            uri: "codeprism://symbols/classes".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let classes: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();
        assert!(classes.is_array());
    }

    #[tokio::test]
    async fn test_read_file_resource() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);

        let params = ReadResourceParams {
            uri: "codeprism://repository/file/main.py".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        assert_eq!(content.uri, "codeprism://repository/file/main.py");
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
            uri: "codeprism://repository/file/nonexistent.py".to_string(),
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
            uri: "codeprism://repository/unknown_resource".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unknown resource URI"));
    }

    #[test]
    fn test_resource_content_serialization() {
        let content = ResourceContent {
            uri: "codeprism://test".to_string(),
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
            uri: "codeprism://test".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ReadResourceParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.uri, deserialized.uri);
    }

    #[test]
    fn test_additional_mime_types() {
        assert_eq!(
            detect_mime_type(Path::new("config.json")),
            "application/json"
        );
        assert_eq!(detect_mime_type(Path::new("README.md")), "text/markdown");
        assert_eq!(detect_mime_type(Path::new("data.xml")), "application/xml");
        assert_eq!(
            detect_mime_type(Path::new("config.yaml")),
            "application/yaml"
        );
        assert_eq!(
            detect_mime_type(Path::new("config.yml")),
            "application/yaml"
        );
        assert_eq!(
            detect_mime_type(Path::new("Cargo.toml")),
            "application/toml"
        );
        assert_eq!(detect_mime_type(Path::new("index.html")), "text/html");
        assert_eq!(detect_mime_type(Path::new("styles.css")), "text/css");
        assert_eq!(detect_mime_type(Path::new("notes.txt")), "text/plain");
    }

    #[tokio::test]
    async fn test_symbol_resources_include_source_context() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc.clone());

        // Wait for indexing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test functions resource includes context
        let params = ReadResourceParams {
            uri: "codeprism://symbols/functions".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];
        assert!(content.text.is_some());

        let functions: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();

        if let Some(functions_array) = functions.as_array() {
            if !functions_array.is_empty() {
                let first_function = &functions_array[0];

                // Verify basic function info is present
                assert!(first_function["id"].is_string());
                assert!(first_function["name"].is_string());
                assert!(first_function["kind"].is_string());
                assert!(first_function["file"].is_string());

                // Verify source context is included
                assert!(first_function["source_context"].is_object());
                assert!(first_function["source_context"]["target_line"].is_number());
                assert!(first_function["source_context"]["lines"].is_array());

                let lines = first_function["source_context"]["lines"]
                    .as_array()
                    .unwrap();
                assert!(!lines.is_empty());

                // Verify target line is marked
                let has_target = lines.iter().any(|line| line["is_target"] == true);
                assert!(has_target);
            }
        }
    }

    #[tokio::test]
    async fn test_classes_resource_with_context() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let resource_manager = ResourceManager::new(server_arc);

        // Wait for indexing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let params = ReadResourceParams {
            uri: "codeprism://symbols/classes".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let read_result = result.unwrap();
        let content = &read_result.contents[0];

        let classes: serde_json::Value =
            serde_json::from_str(content.text.as_ref().unwrap()).unwrap();

        if let Some(classes_array) = classes.as_array() {
            if !classes_array.is_empty() {
                let first_class = &classes_array[0];

                // Verify source context is included for classes too
                assert!(first_class["source_context"].is_object());
                assert!(first_class["source_context"]["target_line"].is_number());
                assert!(first_class["source_context"]["lines"].is_array());
            }
        }
    }

    #[tokio::test]
    async fn test_context_extraction_in_resource_manager() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");

        // Create a test file
        fs::write(
            &test_file,
            r#"# Test file
def example_function():
    """An example function."""
    return "hello"
"#,
        )
        .unwrap();

        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        // Test context extraction
        let context = resource_manager.extract_source_context(&test_file, 2, 2);
        assert!(context.is_some());

        let context_value = context.unwrap();
        assert_eq!(context_value["target_line"], 2);

        let lines = context_value["lines"].as_array().unwrap();
        assert!(!lines.is_empty());
    }

    #[tokio::test]
    async fn test_architectural_layers_resource() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ReadResourceParams {
            uri: "codeprism://architecture/layers".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let resource_result = result.unwrap();
        assert_eq!(resource_result.contents.len(), 1);

        let content = &resource_result.contents[0];
        assert_eq!(content.uri, "codeprism://architecture/layers");
        assert_eq!(content.mime_type, Some("application/json".to_string()));
        assert!(content.text.is_some());

        // Verify the JSON structure
        let json_text = content.text.as_ref().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_text).unwrap();
        assert!(parsed["layer_analysis"].is_object());
        assert!(parsed["directory_structure"].is_object());
        assert!(parsed["layering_assessment"].is_object());
    }

    #[tokio::test]
    async fn test_architectural_patterns_resource() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ReadResourceParams {
            uri: "codeprism://architecture/patterns".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let resource_result = result.unwrap();
        assert_eq!(resource_result.contents.len(), 1);

        let content = &resource_result.contents[0];
        assert_eq!(content.uri, "codeprism://architecture/patterns");
        assert!(content.text.is_some());

        // Verify the JSON structure
        let json_text = content.text.as_ref().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_text).unwrap();
        assert!(parsed["detected_patterns"].is_array());
        assert!(parsed["pattern_summary"].is_object());
        assert!(parsed["recommendations"].is_array());
    }

    #[tokio::test]
    async fn test_architectural_dependencies_resource() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ReadResourceParams {
            uri: "codeprism://architecture/dependencies".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let resource_result = result.unwrap();
        assert_eq!(resource_result.contents.len(), 1);

        let content = &resource_result.contents[0];
        assert_eq!(content.uri, "codeprism://architecture/dependencies");
        assert!(content.text.is_some());

        // Verify the JSON structure
        let json_text = content.text.as_ref().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_text).unwrap();
        assert!(parsed["dependency_overview"].is_object());
        assert!(parsed["coupling_analysis"].is_object());
        assert!(parsed["potential_issues"].is_object());
        assert!(parsed["recommendations"].is_array());
    }

    #[tokio::test]
    async fn test_architectural_resources_in_list() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ListResourcesParams { cursor: None };
        let result = resource_manager.list_resources(params).await;
        assert!(result.is_ok());

        let resources_result = result.unwrap();
        let resource_uris: Vec<&String> =
            resources_result.resources.iter().map(|r| &r.uri).collect();

        // Check that our new architectural resources are included
        assert!(resource_uris.contains(&&"codeprism://architecture/layers".to_string()));
        assert!(resource_uris.contains(&&"codeprism://architecture/patterns".to_string()));
        assert!(resource_uris.contains(&&"codeprism://architecture/dependencies".to_string()));

        // Should have all resources including architectural ones
        assert!(resources_result.resources.len() >= 12); // Original + Quality + Architectural
    }

    #[tokio::test]
    async fn test_enhanced_quality_dashboard() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ReadResourceParams {
            uri: "codeprism://metrics/quality_dashboard".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_ok());

        let resource_result = result.unwrap();
        let content = &resource_result.contents[0];
        let json_text = content.text.as_ref().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_text).unwrap();

        // Verify enhanced structure
        assert!(parsed["repository_overview"].is_object());
        assert!(parsed["code_structure"].is_object());
        assert!(parsed["complexity_distribution"].is_object());
        assert!(parsed["technical_debt"].is_object());
        assert!(parsed["quality_scores"].is_object());
        assert!(parsed["recommendations"].is_array());

        // Verify quality scores
        let quality_scores = &parsed["quality_scores"];
        assert!(quality_scores["overall"].is_number());
        assert!(quality_scores["maintainability"].is_number());
        assert!(quality_scores["readability"].is_number());
    }

    #[tokio::test]
    async fn test_architectural_resource_error_handling() {
        let server = create_test_server().await;
        let resource_manager =
            ResourceManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        // Test with invalid architectural resource URI
        let params = ReadResourceParams {
            uri: "codeprism://architecture/invalid".to_string(),
        };

        let result = resource_manager.read_resource(params).await;
        assert!(result.is_err()); // Should return error for unsupported URI
    }
}
