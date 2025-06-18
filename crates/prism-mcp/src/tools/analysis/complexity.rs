//! Complexity analysis tools

use anyhow::Result;
use serde_json::Value;
use crate::tools_legacy::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::PrismMcpServer;

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

/// Analyze code complexity (simplified implementation for Phase 1)
async fn analyze_complexity(_server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    
    let target = args.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;
    
    let _metrics = args.get("metrics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let _threshold_warnings = args.get("threshold_warnings")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Simplified complexity analysis for Phase 1
    let result = serde_json::json!({
        "target": target,
        "complexity_analysis": {
            "status": "phase1_implementation",
            "message": "Full complexity analysis implementation will be completed in Phase 1 continuation",
            "basic_metrics": {
                "file_exists": std::path::Path::new(target).exists(),
                "analysis_type": if target.contains('/') || target.contains('.') { "file" } else { "symbol" }
            }
        },
        "note": "This tool is being modularized as part of Phase 1 enhancement. Full implementation coming soon."
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
} 