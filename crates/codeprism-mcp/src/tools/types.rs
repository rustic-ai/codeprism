//! Core MCP tool types and shared structures.
//!
//! This module defines the fundamental types for MCP tool implementation,
//! including tool definitions, parameters, results, and common data structures.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool capabilities as defined by MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// Whether the server will emit notifications when the list of available tools changes
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique identifier for the tool
    pub name: String,
    /// Optional human-readable title for display purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description of the tool's functionality
    pub description: String,
    /// JSON Schema defining expected input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content returned by the tool
    pub content: Vec<ToolContent>,
    /// Whether the tool execution resulted in an error
    #[serde(rename = "isError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
}

/// Parameters for listing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Result of listing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// List of available tools
    pub tools: Vec<Tool>,
    /// Optional cursor for pagination
    #[serde(rename = "nextCursor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Helper trait for creating tool content
impl ToolContent {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

/// Helper trait for creating tool results
impl CallToolResult {
    /// Create a successful result with text content
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(text)],
            is_error: Some(false),
        }
    }

    /// Create an error result with error message
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(message)],
            is_error: Some(true),
        }
    }

    /// Create a result with multiple content items
    pub fn with_content(content: Vec<ToolContent>) -> Self {
        Self {
            content,
            is_error: Some(false),
        }
    }
}
