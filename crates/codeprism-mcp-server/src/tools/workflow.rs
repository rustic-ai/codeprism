//! Workflow MCP tools for validation and code generation

use super::McpTool;
use serde_json::json;

/// Workflow tools implementation
pub struct WorkflowTools;

impl WorkflowTools {
    /// Create a new instance of workflow tools
    pub fn new() -> Self {
        Self
    }
}

impl McpTool for WorkflowTools {
    fn name(&self) -> &str {
        "workflow"
    }

    fn description(&self) -> &str {
        "Workflow tools for validation and code generation"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "validate_code",
                        "generate_docs",
                        "format_code",
                        "refactor_assist"
                    ]
                },
                "source": {
                    "type": "string",
                    "description": "Source code or file path"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language"
                },
                "options": {
                    "type": "object",
                    "description": "Tool-specific options"
                }
            },
            "required": ["operation", "source"]
        })
    }
}

impl Default for WorkflowTools {
    fn default() -> Self {
        Self::new()
    }
}
