//! Analysis MCP tools for code complexity and pattern analysis

use super::McpTool;
use serde_json::json;

/// Analysis tools implementation
pub struct AnalysisTools;

impl AnalysisTools {
    /// Create a new instance of analysis tools
    pub fn new() -> Self {
        Self
    }
}

impl McpTool for AnalysisTools {
    fn name(&self) -> &str {
        "analysis"
    }

    fn description(&self) -> &str {
        "Analysis tools for code complexity and pattern analysis"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "complexity_analysis",
                        "pattern_detection",
                        "code_metrics",
                        "duplicate_detection"
                    ]
                },
                "target": {
                    "type": "string",
                    "description": "File or directory to analyze"
                },
                "metrics": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Specific metrics to calculate"
                }
            },
            "required": ["operation", "target"]
        })
    }
}

impl Default for AnalysisTools {
    fn default() -> Self {
        Self::new()
    }
}
