//! Core MCP tools for basic file operations and content search

use super::McpTool;
use serde_json::json;

/// Core tools implementation
pub struct CoreTools;

impl CoreTools {
    /// Create a new instance of core tools
    pub fn new() -> Self {
        Self
    }
}

impl McpTool for CoreTools {
    fn name(&self) -> &str {
        "core"
    }

    fn description(&self) -> &str {
        "Core tools for file operations and content search"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "read_file",
                        "list_files",
                        "search_content",
                        "get_file_info"
                    ]
                },
                "path": {
                    "type": "string",
                    "description": "File or directory path"
                },
                "pattern": {
                    "type": "string",
                    "description": "Search pattern for content search"
                }
            },
            "required": ["operation"]
        })
    }
}

impl Default for CoreTools {
    fn default() -> Self {
        Self::new()
    }
}
