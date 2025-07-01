//! Search and discovery tools.
//!
//! This module provides tools for searching content, finding files,
//! and discovering code elements in the repository.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List search tools
///
/// Returns a list of search-related tools available in the modular architecture.
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "search_symbols".to_string(),
            title: Some("Search Code Symbols".to_string()),
            description: "Search for symbols (functions, classes, variables) with optional filters"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern for symbol names (regex supported)"
                    },
                    "symbol_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["function", "class", "variable", "module", "method"]
                        },
                        "description": "Filter by symbol types"
                    },
                    "inheritance_filters": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Inheritance filters like 'inherits_from:BaseClass', 'metaclass:Meta', 'uses_mixin:Mixin'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return"
                    },
                    "context_lines": {
                        "type": "integer",
                        "description": "Number of source context lines around symbols",
                        "default": 4
                    }
                },
                "required": ["pattern"]
            }),
        },
        Tool {
            name: "search_content".to_string(),
            title: Some("Search File Content".to_string()),
            description: "Search file contents with pattern matching and filtering options"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Text to search for in file contents"
                    },
                    "content_types": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Filter by content types (e.g., 'code', 'documentation')"
                    },
                    "file_patterns": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Include files matching these patterns"
                    },
                    "exclude_patterns": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Exclude files matching these patterns"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 50
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Whether search should be case sensitive",
                        "default": false
                    },
                    "use_regex": {
                        "type": "boolean",
                        "description": "Whether to use regex pattern matching",
                        "default": false
                    },
                    "include_context": {
                        "type": "boolean",
                        "description": "Whether to include context around matches",
                        "default": true
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "find_files".to_string(),
            title: Some("Find Files by Pattern".to_string()),
            description: "Find files matching name or path patterns".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "File pattern to search for (regex or glob-style)"
                    }
                },
                "required": ["pattern"]
            }),
        },
    ]
}

/// Handle search tool calls
///
/// Routes search tool calls to appropriate functions.
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "search_symbols" => search_symbols(server, arguments).await,
        "search_content" => search_content(server, arguments).await,
        "find_files" => find_files(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown search tool: {}", tool_name)),
    }
}

/// Search for symbols with optional filtering
async fn search_symbols(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;

    let symbol_types = args
        .get("symbol_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| match s {
                    "function" => Some(codeprism_core::NodeKind::Function),
                    "class" => Some(codeprism_core::NodeKind::Class),
                    "variable" => Some(codeprism_core::NodeKind::Variable),
                    "module" => Some(codeprism_core::NodeKind::Module),
                    "method" => Some(codeprism_core::NodeKind::Method),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });

    let inheritance_filters = args
        .get("inheritance_filters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(parse_inheritance_filter)
                .collect::<Vec<_>>()
        });

    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let context_lines = args
        .get("context_lines")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(4);

    // Use enhanced search with inheritance filters if provided
    let results = if let Some(ref filters) = inheritance_filters {
        server.graph_query().search_symbols_with_inheritance(
            pattern,
            symbol_types,
            Some(filters.clone()),
            limit,
        )?
    } else {
        server
            .graph_query()
            .search_symbols(pattern, symbol_types, limit)?
    };

    let result = serde_json::json!({
        "pattern": pattern,
        "inheritance_filters_applied": inheritance_filters.is_some(),
        "results": results.iter().map(|symbol| {
            let mut symbol_info = create_node_info_with_context(&symbol.node, context_lines);
            symbol_info["references_count"] = serde_json::json!(symbol.references_count);
            symbol_info["dependencies_count"] = serde_json::json!(symbol.dependencies_count);

            // Add inheritance info for classes when inheritance filters are used
            if matches!(symbol.node.kind, codeprism_core::NodeKind::Class) && inheritance_filters.is_some() {
                if let Ok(inheritance_info) = server.graph_query().get_inheritance_info(&symbol.node.id) {
                    symbol_info["inheritance_summary"] = serde_json::json!({
                        "is_metaclass": inheritance_info.is_metaclass,
                        "base_classes": inheritance_info.base_classes.iter().map(|rel| rel.class_name.clone()).collect::<Vec<_>>(),
                        "mixins": inheritance_info.mixins.iter().map(|rel| rel.class_name.clone()).collect::<Vec<_>>(),
                        "metaclass": inheritance_info.metaclass.as_ref().map(|mc| mc.class_name.clone())
                    });
                }
            }

            symbol_info
        }).collect::<Vec<_>>()
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Search file contents with pattern matching
async fn search_content(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

    let content_types = args
        .get("content_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let file_patterns = args
        .get("file_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let exclude_patterns = args
        .get("exclude_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(50);

    let case_sensitive = args
        .get("case_sensitive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let use_regex = args
        .get("use_regex")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let include_context = args
        .get("include_context")
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

    match server
        .content_search()
        .simple_search(query, Some(max_results))
    {
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

/// Find files by pattern matching
async fn find_files(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let pattern = args
        .get("pattern")
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

                    let matching_files: Vec<_> = all_files
                        .iter()
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

/// Helper functions extracted from legacy implementation
/// Parse inheritance filter string into InheritanceFilter enum
fn parse_inheritance_filter(filter_str: &str) -> Option<codeprism_core::InheritanceFilter> {
    if let Some(colon_pos) = filter_str.find(':') {
        let filter_type = &filter_str[..colon_pos];
        let class_name = &filter_str[colon_pos + 1..];

        match filter_type {
            "inherits_from" => Some(codeprism_core::InheritanceFilter::InheritsFrom(
                class_name.to_string(),
            )),
            "metaclass" => Some(codeprism_core::InheritanceFilter::HasMetaclass(
                class_name.to_string(),
            )),
            "uses_mixin" => Some(codeprism_core::InheritanceFilter::UsesMixin(
                class_name.to_string(),
            )),
            _ => None,
        }
    } else {
        None
    }
}

/// Extract source context around a line number from a file
fn extract_source_context(
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
    if let Some(context) = extract_source_context(&node.file, node.span.start_line, context_lines) {
        node_info["source_context"] = context;
    }

    node_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inheritance_filter_valid() {
        assert!(matches!(
            parse_inheritance_filter("inherits_from:BaseClass"),
            Some(codeprism_core::InheritanceFilter::InheritsFrom(_))
        ));
        assert!(matches!(
            parse_inheritance_filter("metaclass:Meta"),
            Some(codeprism_core::InheritanceFilter::HasMetaclass(_))
        ));
        assert!(matches!(
            parse_inheritance_filter("uses_mixin:Mixin"),
            Some(codeprism_core::InheritanceFilter::UsesMixin(_))
        ));
    }

    #[test]
    fn test_parse_inheritance_filter_invalid() {
        assert!(parse_inheritance_filter("invalid_format").is_none());
        assert!(parse_inheritance_filter("unknown:BaseClass").is_none());
        assert!(parse_inheritance_filter("").is_none());
    }

    #[test]
    fn test_search_tools_list() {
        let tools = list_tools();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"search_symbols"));
        assert!(tool_names.contains(&"search_content"));
        assert!(tool_names.contains(&"find_files"));
    }
}
