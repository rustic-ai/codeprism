//! Search MCP tools for semantic search and dependency analysis

use super::McpTool;
use serde_json::json;

/// Search tools implementation
pub struct SearchTools;

impl SearchTools {
    /// Create a new instance of search tools
    pub fn new() -> Self {
        Self
    }
}

impl McpTool for SearchTools {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search tools for semantic search and dependency analysis"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "semantic_search",
                        "find_references",
                        "find_dependencies",
                        "symbol_search"
                    ]
                },
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "scope": {
                    "type": "string",
                    "description": "Search scope (file, directory, project)"
                }
            },
            "required": ["operation", "query"]
        })
    }
}

impl Default for SearchTools {
    fn default() -> Self {
        Self::new()
    }
}
