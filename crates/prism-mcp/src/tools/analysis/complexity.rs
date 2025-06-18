//! Complexity analysis tools

use anyhow::Result;
use serde_json::Value;
use crate::tools_legacy::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::PrismMcpServer;

/// Analyze complexity for a specific file
fn analyze_file_complexity(file_path: &str, metrics: &[String], threshold_warnings: bool) -> Value {
    let path = std::path::Path::new(file_path);
    
    if !path.exists() {
        return serde_json::json!({
            "target": file_path,
            "error": "File not found",
            "file_exists": false
        });
    }
    
    // Read file and calculate basic metrics
    if let Ok(content) = std::fs::read_to_string(path) {
        let line_count = content.lines().count();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        
        // Basic complexity estimation based on file content
        let estimated_complexity = calculate_basic_complexity(&content);
        
        let mut result = serde_json::json!({
            "target": file_path,
            "file_analysis": {
                "file_exists": true,
                "line_count": line_count,
                "character_count": char_count,
                "word_count": word_count,
                "estimated_complexity": estimated_complexity
            },
            "metrics": {}
        });
        
        // Add requested metrics
        for metric in metrics {
            match metric.as_str() {
                "all" | "basic" => {
                    result["metrics"]["basic"] = serde_json::json!({
                        "lines_of_code": line_count,
                        "complexity_score": estimated_complexity,
                        "maintainability_index": calculate_maintainability_index(line_count, estimated_complexity)
                    });
                }
                "cyclomatic" => {
                    result["metrics"]["cyclomatic"] = serde_json::json!({
                        "estimated_complexity": estimated_complexity,
                        "note": "Estimated based on control flow indicators"
                    });
                }
                _ => {
                    result["metrics"][metric] = serde_json::json!({
                        "status": "not_implemented",
                        "note": format!("Metric '{}' calculation not yet implemented", metric)
                    });
                }
            }
        }
        
        if threshold_warnings && estimated_complexity > 10 {
            result["warnings"] = serde_json::json!([
                format!("High complexity detected: {} (recommended: < 10)", estimated_complexity)
            ]);
        }
        
        result
    } else {
        serde_json::json!({
            "target": file_path,
            "error": "Failed to read file",
            "file_exists": true
        })
    }
}

/// Analyze complexity for a specific symbol/node
fn analyze_symbol_complexity(node: &prism_core::Node, metrics: &[String], threshold_warnings: bool) -> Value {
    let symbol_complexity = match node.kind {
        prism_core::NodeKind::Function | prism_core::NodeKind::Method => {
            // Functions typically have higher complexity
            5 + (node.name.len() / 10) // Simple heuristic
        }
        prism_core::NodeKind::Class => {
            // Classes have moderate complexity
            3 + (node.name.len() / 20)
        }
        _ => {
            // Other symbols have low complexity
            1
        }
    };
    
    let mut result = serde_json::json!({
        "target": node.name,
        "symbol_analysis": {
            "id": node.id.to_hex(),
            "name": node.name,
            "kind": format!("{:?}", node.kind),
            "file": node.file.display().to_string(),
            "span": {
                "start_line": node.span.start_line,
                "end_line": node.span.end_line
            },
            "symbol_complexity": symbol_complexity
        },
        "metrics": {}
    });
    
    // Add requested metrics
    for metric in metrics {
        match metric.as_str() {
            "all" | "basic" => {
                result["metrics"]["basic"] = serde_json::json!({
                    "symbol_complexity": symbol_complexity,
                    "maintainability_index": calculate_maintainability_index(1, symbol_complexity)
                });
            }
            "cyclomatic" => {
                result["metrics"]["cyclomatic"] = serde_json::json!({
                    "estimated_complexity": symbol_complexity,
                    "note": "Estimated based on symbol type and name"
                });
            }
            _ => {
                result["metrics"][metric] = serde_json::json!({
                    "status": "not_implemented",
                    "note": format!("Metric '{}' calculation not yet implemented for symbols", metric)
                });
            }
        }
    }
    
    if threshold_warnings && symbol_complexity > 5 {
        result["warnings"] = serde_json::json!([
            format!("High symbol complexity: {} (recommended: < 5)", symbol_complexity)
        ]);
    }
    
    result
}

/// Calculate basic complexity estimation from code content
fn calculate_basic_complexity(content: &str) -> usize {
    let mut complexity = 1; // Base complexity
    
    // Count control flow statements
    for line in content.lines() {
        let line = line.trim();
        if line.contains("if ") || line.contains("elif ") {
            complexity += 1;
        }
        if line.contains("for ") || line.contains("while ") {
            complexity += 1;
        }
        if line.contains("try:") || line.contains("except") {
            complexity += 1;
        }
        if line.contains("match ") || line.contains("case ") {
            complexity += 1;
        }
    }
    
    complexity
}

/// Calculate maintainability index (simplified)
fn calculate_maintainability_index(lines: usize, complexity: usize) -> f64 {
    // Simplified maintainability index calculation
    let halstead_volume = (lines as f64).log2() * 10.0; // Rough approximation
    let mi = 171.0 - 5.2 * (halstead_volume).log2() - 0.23 * (complexity as f64) - 16.2 * (lines as f64).log2();
    mi.max(0.0).min(100.0)
}

/// List complexity analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_complexity".to_string(),
            title: Some("Analyze Code Complexity".to_string()),
            description: "Calculate complexity metrics for code elements including cyclomatic, cognitive, and maintainability metrics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "File path or symbol ID to analyze"
                    },
                    "metrics": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["cyclomatic", "cognitive", "halstead", "maintainability_index", "all"]
                        },
                        "description": "Types of complexity metrics to calculate",
                        "default": ["all"]
                    },
                    "threshold_warnings": {
                        "type": "boolean",
                        "description": "Include warnings for metrics exceeding thresholds",
                        "default": true
                    }
                },
                "required": ["target"]
            }),
        }
    ]
}

/// Route complexity analysis tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "analyze_complexity" => analyze_complexity(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!("Unknown complexity analysis tool: {}", params.name)),
    }
}

/// Analyze code complexity
async fn analyze_complexity(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    
    // Support both "target" and "path" parameter names for backward compatibility
    let target = args.get("target")
        .or_else(|| args.get("path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter (or path)"))?;
    
    let metrics = args.get("metrics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let threshold_warnings = args.get("threshold_warnings")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Try to resolve as symbol identifier or file path
    let result = if target.contains('/') || target.contains('.') {
        // Handle as file path
        analyze_file_complexity(target, &metrics, threshold_warnings)
    } else {
        // Try to resolve as symbol name using search, then analyze
        if let Ok(symbol_results) = server.graph_query().search_symbols(target, None, Some(1)) {
            if let Some(symbol_result) = symbol_results.first() {
                analyze_symbol_complexity(&symbol_result.node, &metrics, threshold_warnings)
            } else {
                serde_json::json!({
                    "target": target,
                    "error": "Symbol not found",
                    "suggestion": "Try using a file path or check if the symbol name is correct"
                })
            }
        } else {
            serde_json::json!({
                "target": target,
                "error": "Failed to search for symbol",
                "suggestion": "Try using a file path instead"
            })
        }
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(result.get("error").is_some()),
    })
} 