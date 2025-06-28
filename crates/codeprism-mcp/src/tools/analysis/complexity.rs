//! Complexity analysis tools

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;
use anyhow::Result;
use codeprism_analysis::complexity::ComplexityAnalyzer;
use serde_json::Value;
use std::path::Path;

/// Analyze complexity for a specific file
fn analyze_file_complexity(file_path: &str, metrics: &[String], threshold_warnings: bool) -> Value {
    let path = Path::new(file_path);

    if !path.exists() {
        return serde_json::json!({
            "target": file_path,
            "error": "File not found",
            "file_exists": false
        });
    }

    let analyzer = ComplexityAnalyzer::new();

    match analyzer.analyze_file_complexity(path, metrics, threshold_warnings) {
        Ok(mut result) => {
            // Add target information for consistency
            result["target"] = serde_json::Value::String(file_path.to_string());
            result["file_exists"] = serde_json::Value::Bool(true);
            result
        }
        Err(e) => {
            serde_json::json!({
                "target": file_path,
                "error": format!("Failed to analyze file: {}", e),
                "file_exists": true
            })
        }
    }
}

/// Analyze complexity for a specific symbol/node
fn analyze_symbol_complexity(
    node: &codeprism_core::Node,
    metrics: &[String],
    threshold_warnings: bool,
) -> Value {
    // For symbol-specific analysis, we need to extract content from the file at the symbol's location
    let file_path = &node.file;

    if let Ok(content) = std::fs::read_to_string(file_path) {
        let lines: Vec<&str> = content.lines().collect();

        // Extract symbol content based on span
        let start_line = node.span.start_line.saturating_sub(1); // Convert to 0-based
        let end_line = (node.span.end_line.saturating_sub(1)).min(lines.len().saturating_sub(1));

        if start_line <= end_line && start_line < lines.len() {
            let symbol_content = lines[start_line..=end_line].join("\n");
            let symbol_lines = end_line - start_line + 1;

            let analyzer = ComplexityAnalyzer::new();
            let complexity_metrics = analyzer.calculate_all_metrics(&symbol_content, symbol_lines);

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
                    "lines_of_code": symbol_lines
                },
                "metrics": {}
            });

            // Add requested metrics
            for metric in metrics {
                match metric.as_str() {
                    "all" => {
                        result["metrics"]["cyclomatic_complexity"] =
                            complexity_metrics.cyclomatic.into();
                        result["metrics"]["cognitive_complexity"] =
                            complexity_metrics.cognitive.into();
                        result["metrics"]["halstead"] = serde_json::json!({
                            "volume": complexity_metrics.halstead_volume,
                            "difficulty": complexity_metrics.halstead_difficulty,
                            "effort": complexity_metrics.halstead_effort
                        });
                        result["metrics"]["maintainability_index"] =
                            complexity_metrics.maintainability_index.into();
                    }
                    "cyclomatic" => {
                        result["metrics"]["cyclomatic_complexity"] =
                            complexity_metrics.cyclomatic.into();
                    }
                    "cognitive" => {
                        result["metrics"]["cognitive_complexity"] =
                            complexity_metrics.cognitive.into();
                    }
                    "halstead" => {
                        result["metrics"]["halstead"] = serde_json::json!({
                            "volume": complexity_metrics.halstead_volume,
                            "difficulty": complexity_metrics.halstead_difficulty,
                            "effort": complexity_metrics.halstead_effort
                        });
                    }
                    "maintainability_index" | "maintainability" => {
                        result["metrics"]["maintainability_index"] =
                            complexity_metrics.maintainability_index.into();
                    }
                    _ => {
                        result["metrics"][metric] = serde_json::json!({
                            "status": "unknown_metric",
                            "note": format!("Unknown metric '{}'. Available metrics: cyclomatic, cognitive, halstead, maintainability_index, all", metric)
                        });
                    }
                }
            }

            // Add threshold warnings
            if threshold_warnings {
                let mut warnings = Vec::new();

                if complexity_metrics.cyclomatic > 10 {
                    warnings.push(format!(
                        "High cyclomatic complexity: {} (recommended: < 10)",
                        complexity_metrics.cyclomatic
                    ));
                }

                if complexity_metrics.cognitive > 15 {
                    warnings.push(format!(
                        "High cognitive complexity: {} (recommended: < 15)",
                        complexity_metrics.cognitive
                    ));
                }

                if complexity_metrics.maintainability_index < 50.0 {
                    warnings.push(format!(
                        "Low maintainability index: {:.1} (recommended: > 50.0)",
                        complexity_metrics.maintainability_index
                    ));
                }

                if !warnings.is_empty() {
                    result["warnings"] = serde_json::json!(warnings);
                }
            }

            result
        } else {
            serde_json::json!({
                "target": node.name,
                "error": "Invalid line range for symbol",
                "symbol_analysis": {
                    "id": node.id.to_hex(),
                    "name": node.name,
                    "kind": format!("{:?}", node.kind),
                    "file": node.file.display().to_string(),
                    "span": {
                        "start_line": node.span.start_line,
                        "end_line": node.span.end_line
                    }
                }
            })
        }
    } else {
        serde_json::json!({
            "target": node.name,
            "error": "Failed to read symbol's source file",
            "symbol_analysis": {
                "id": node.id.to_hex(),
                "name": node.name,
                "file": node.file.display().to_string()
            }
        })
    }
}

/// List complexity analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_complexity".to_string(),
            title: Some("Analyze Code Complexity".to_string()),
            description: "Calculate comprehensive complexity metrics including cyclomatic, cognitive, Halstead metrics, and maintainability index for code elements".to_string(),
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
                            "enum": ["cyclomatic", "cognitive", "halstead", "maintainability_index", "maintainability", "all"]
                        },
                        "description": "Types of complexity metrics to calculate. 'all' includes all available metrics.",
                        "default": ["all"]
                    },
                    "threshold_warnings": {
                        "type": "boolean",
                        "description": "Include warnings for metrics exceeding recommended thresholds",
                        "default": true
                    }
                },
                "required": ["target"],
                "additionalProperties": false
            }),
        }
    ]
}

/// Route complexity analysis tool calls
pub async fn call_tool(
    server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    match params.name.as_str() {
        "analyze_complexity" => analyze_complexity(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!(
            "Unknown complexity analysis tool: {}",
            params.name
        )),
    }
}

/// Analyze code complexity
async fn analyze_complexity(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Support both "target" and "path" parameter names for backward compatibility
    let target = args
        .get("target")
        .or_else(|| args.get("path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter (or path)"))?;

    let metrics = args
        .get("metrics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let threshold_warnings = args
        .get("threshold_warnings")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Try to resolve as symbol identifier or file path
    let result = if target.contains('/') || target.contains('.') {
        // Handle as file path
        analyze_file_complexity(target, &metrics, threshold_warnings)
    } else {
        // Try to resolve as symbol name using search, then analyze
        match server.graph_query().search_symbols(target, None, Some(1)) {
            Ok(symbol_results) => {
                if let Some(symbol_result) = symbol_results.first() {
                    analyze_symbol_complexity(&symbol_result.node, &metrics, threshold_warnings)
                } else {
                    serde_json::json!({
                        "target": target,
                        "error": "Symbol not found",
                        "suggestion": "Try using a file path or check if the symbol name is correct",
                        "available_metrics": ["cyclomatic", "cognitive", "halstead", "maintainability_index", "all"]
                    })
                }
            }
            Err(e) => {
                serde_json::json!({
                    "target": target,
                    "error": format!("Failed to search for symbol: {}", e),
                    "suggestion": "Try using a file path instead",
                    "available_metrics": ["cyclomatic", "cognitive", "halstead", "maintainability_index", "all"]
                })
            }
        }
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(result.get("error").is_some()),
    })
}
