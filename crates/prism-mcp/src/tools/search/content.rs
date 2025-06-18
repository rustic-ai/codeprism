//! Content search and file discovery tools

use anyhow::Result;
use serde_json::Value;
use crate::tools_legacy::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::PrismMcpServer;

/// List content search tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "search_content".to_string(),
            title: Some("Search Content".to_string()),
            description: "Search across all content including documentation, comments, and configuration files".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "content_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["documentation", "comments", "configuration", "code"]
                        },
                        "description": "Types of content to search in"
                    },
                    "file_patterns": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "File patterns to include (regex)"
                    },
                    "exclude_patterns": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "File patterns to exclude (regex)"
                    },
                    "max_results": {
                        "type": "number",
                        "description": "Maximum number of results",
                        "default": 50
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search",
                        "default": false
                    },
                    "use_regex": {
                        "type": "boolean",
                        "description": "Use regex pattern matching",
                        "default": false
                    },
                    "include_context": {
                        "type": "boolean",
                        "description": "Include context around matches",
                        "default": true
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "find_files".to_string(),
            title: Some("Find Files".to_string()),
            description: "Find files by name or path pattern".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "File pattern to search for (supports regex)"
                    }
                },
                "required": ["pattern"]
            }),
        },
        Tool {
            name: "content_stats".to_string(),
            title: Some("Content Statistics".to_string()),
            description: "Get statistics about indexed content".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    ]
}

/// Route content search tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "search_content" => search_content(server, params.arguments.as_ref()).await,
        "find_files" => find_files(server, params.arguments.as_ref()).await,
        "content_stats" => content_stats(server).await,
        _ => Err(anyhow::anyhow!("Unknown content search tool: {}", params.name)),
    }
}

/// Search content across all files
async fn search_content(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

    let content_types = args.get("content_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let file_patterns = args.get("file_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let exclude_patterns = args.get("exclude_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(50);

    let case_sensitive = args.get("case_sensitive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let use_regex = args.get("use_regex")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let include_context = args.get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Check if content is indexed
    let stats = server.content_search().get_stats();
    if stats.total_files == 0 {
        let result = serde_json::json!({
            "query": query,
            "results": [],
            "total_results": 0,
            "status": "no_content_indexed",
            "message": "Content search is not yet indexed. This feature requires repository content to be indexed first.",
            "suggestion": "Repository indexing may still be in progress. Try again in a few moments."
        });

        return Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        });
    }

    match server.content_search().simple_search(query, Some(max_results)) {
        Ok(search_results) => {
            let result = serde_json::json!({
                "query": query,
                "content_types": content_types,
                "file_patterns": file_patterns,
                "exclude_patterns": exclude_patterns,
                "max_results": max_results,
                "case_sensitive": case_sensitive,
                "use_regex": use_regex,
                "include_context": include_context,
                "total_results": search_results.len(),
                "results": search_results.iter().map(|result| {
                    serde_json::json!({
                        "file": result.chunk.file_path.display().to_string(),
                        "content_type": format!("{:?}", result.chunk.content_type),
                        "score": result.score,
                        "matches": result.matches.iter().map(|m| {
                            serde_json::json!({
                                "text": m.text,
                                "line": m.line_number,
                                "column": m.column_number,
                                "context_before": m.context_before,
                                "context_after": m.context_after
                            })
                        }).collect::<Vec<_>>(),
                        "chunk_content_preview": if result.chunk.content.len() > 200 {
                            format!("{}...", &result.chunk.content[..200])
                        } else {
                            result.chunk.content.clone()
                        }
                    })
                }).collect::<Vec<_>>()
            });

            Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(false),
            })
        }
        Err(e) => Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: format!("Content search error: {}", e),
            }],
            is_error: Some(true),
        }),
    }
}

/// Find files by pattern
async fn find_files(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    
    let pattern = args.get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;

    // Check if content is indexed
    let stats = server.content_search().get_stats();
    if stats.total_files == 0 {
        // Fall back to scanning the repository directly
        if let Some(repo_path) = server.repository_path() {
            match server.scanner().discover_files(repo_path) {
                Ok(all_files) => {
                    let pattern_regex = match regex::Regex::new(pattern) {
                        Ok(regex) => regex,
                        Err(_) => {
                            // Fall back to glob-style matching
                            let glob_pattern = pattern.replace("*", ".*").replace("?", ".");
                            match regex::Regex::new(&glob_pattern) {
                                Ok(regex) => regex,
                                Err(e) => {
                                    return Ok(CallToolResult {
                                        content: vec![ToolContent::Text {
                                            text: format!("Invalid pattern '{}': {}", pattern, e),
                                        }],
                                        is_error: Some(true),
                                    });
                                }
                            }
                        }
                    };

                    let matching_files: Vec<_> = all_files.iter()
                        .filter(|path| pattern_regex.is_match(&path.to_string_lossy()))
                        .collect();

                    let result = serde_json::json!({
                        "pattern": pattern,
                        "total_files": matching_files.len(),
                        "source": "repository_scan",
                        "files": matching_files.iter().map(|path| {
                            serde_json::json!({
                                "path": path.display().to_string(),
                                "name": path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(""),
                                "extension": path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .unwrap_or("")
                            })
                        }).collect::<Vec<_>>()
                    });

                    return Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: serde_json::to_string_pretty(&result)?,
                        }],
                        is_error: Some(false),
                    });
                }
                Err(e) => {
                    return Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Failed to scan repository for files: {}", e),
                        }],
                        is_error: Some(true),
                    });
                }
            }
        } else {
            let result = serde_json::json!({
                "pattern": pattern,
                "total_files": 0,
                "source": "no_repository",
                "files": [],
                "message": "No repository is currently loaded"
            });

            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(false),
            });
        }
    }

    match server.content_search().find_files(pattern) {
        Ok(files) => {
            let result = serde_json::json!({
                "pattern": pattern,
                "total_files": files.len(),
                "source": "content_index",
                "files": files.iter().map(|path| {
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "name": path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(""),
                        "extension": path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                    })
                }).collect::<Vec<_>>()
            });

            Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(false),
            })
        }
        Err(e) => Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: format!("File search error: {}", e),
            }],
            is_error: Some(true),
        }),
    }
}

/// Get content statistics
async fn content_stats(server: &PrismMcpServer) -> Result<CallToolResult> {
    let stats = server.content_search().get_stats();
    
    let result = if stats.total_files == 0 {
        serde_json::json!({
            "total_files": 0,
            "total_chunks": 0,
            "total_tokens": 0,
            "content_by_type": {},
            "size_distribution": {},
            "status": "no_content_indexed",
            "message": "Content indexing has not been performed yet. Only code symbol analysis is available.",
            "suggestion": "Content indexing for documentation, configuration files, and comments may still be in progress."
        })
    } else {
        serde_json::json!({
            "total_files": stats.total_files,
            "total_chunks": stats.total_chunks,
            "total_tokens": stats.total_tokens,
            "content_by_type": stats.content_by_type,
            "size_distribution": stats.size_distribution,
            "computed_at": stats.computed_at.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "status": "indexed"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
} 