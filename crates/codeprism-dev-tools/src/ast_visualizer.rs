//! AST Visualization utilities for parser development
//!
//! This module provides tools for visualizing Abstract Syntax Trees (ASTs) in various formats
//! to help with parser development and debugging.

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tree_sitter::{Node, Tree};

/// AST visualizer for pretty-printing syntax trees
#[derive(Debug, Clone)]
pub struct AstVisualizer {
    config: VisualizationConfig,
}

/// Configuration for AST visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Maximum depth to visualize (prevents infinite recursion)
    pub max_depth: usize,
    /// Whether to show node positions (line, column)
    pub show_positions: bool,
    /// Whether to show node byte ranges
    pub show_byte_ranges: bool,
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Whether to show node text content
    pub show_text_content: bool,
    /// Maximum length of text content to display
    pub max_text_length: usize,
    /// Whether to show only named nodes
    pub named_nodes_only: bool,
    /// Custom node type colors
    pub node_color_names: HashMap<String, String>,
    /// Indentation string for tree structure
    pub indent_string: String,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        let mut node_color_names = HashMap::new();

        // Set up default colors for common node types
        node_color_names.insert("function_definition".to_string(), "blue".to_string());
        node_color_names.insert("class_definition".to_string(), "green".to_string());
        node_color_names.insert("function_call".to_string(), "cyan".to_string());
        node_color_names.insert("variable".to_string(), "yellow".to_string());
        node_color_names.insert("string".to_string(), "red".to_string());
        node_color_names.insert("number".to_string(), "magenta".to_string());
        node_color_names.insert("comment".to_string(), "brightblack".to_string());
        node_color_names.insert("keyword".to_string(), "brightblue".to_string());
        node_color_names.insert("operator".to_string(), "brightyellow".to_string());
        node_color_names.insert("identifier".to_string(), "white".to_string());

        Self {
            max_depth: 20,
            show_positions: true,
            show_byte_ranges: false,
            use_colors: true,
            show_text_content: true,
            max_text_length: 50,
            named_nodes_only: false,
            node_color_names,
            indent_string: "  ".to_string(),
        }
    }
}

/// Format options for AST visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationFormat {
    /// Pretty-printed tree structure
    Tree,
    /// Flat list of nodes
    List,
    /// JSON representation
    Json,
    /// S-expression format
    SExpression,
    /// Compact one-line format
    Compact,
}

impl AstVisualizer {
    /// Create a new AST visualizer with default configuration
    pub fn new() -> Self {
        Self {
            config: VisualizationConfig::default(),
        }
    }

    /// Create an AST visualizer with custom configuration
    pub fn with_config(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Visualize a tree-sitter Tree
    pub fn visualize_tree(&self, tree: &Tree, source: &str) -> Result<String> {
        let root_node = tree.root_node();
        self.visualize_node(&root_node, source, VisualizationFormat::Tree)
    }

    /// Visualize a specific node with the given format
    pub fn visualize_node(
        &self,
        node: &Node,
        source: &str,
        format: VisualizationFormat,
    ) -> Result<String> {
        match format {
            VisualizationFormat::Tree => self.visualize_tree_format(node, source),
            VisualizationFormat::List => self.visualize_list_format(node, source),
            VisualizationFormat::Json => self.visualize_json_format(node, source),
            VisualizationFormat::SExpression => self.visualize_sexp_format(node, source),
            VisualizationFormat::Compact => self.visualize_compact_format(node, source),
        }
    }

    /// Visualize in tree format (default pretty-print)
    fn visualize_tree_format(&self, node: &Node, source: &str) -> Result<String> {
        let mut output = String::new();
        self.visualize_node_recursive(node, source, 0, "", true, &mut output);
        Ok(output)
    }

    /// Recursive helper for tree visualization
    fn visualize_node_recursive(
        &self,
        node: &Node,
        source: &str,
        depth: usize,
        prefix: &str,
        is_last: bool,
        output: &mut String,
    ) {
        if depth > self.config.max_depth {
            output.push_str(&format!("{}{}...\n", prefix, "─── ".dimmed()));
            return;
        }

        // Skip unnamed nodes if configured
        if self.config.named_nodes_only && !node.is_named() {
            return;
        }

        // Create the current line prefix
        let connector = if is_last { "└── " } else { "├── " };
        let node_prefix = format!("{}{}", prefix, connector);

        // Format the node type
        let node_type = self.format_node_type(node.kind());

        // Add position information if enabled
        let position_info = if self.config.show_positions {
            let start = node.start_position();
            let end = node.end_position();
            format!(
                " @{}:{}-{}:{}",
                start.row + 1,
                start.column + 1,
                end.row + 1,
                end.column + 1
            )
        } else {
            String::new()
        };

        // Add byte range information if enabled
        let byte_range_info = if self.config.show_byte_ranges {
            format!(" [{}..{}]", node.start_byte(), node.end_byte())
        } else {
            String::new()
        };

        // Add text content if enabled and node is small enough
        let text_content = if self.config.show_text_content && node.child_count() == 0 {
            let text = node
                .utf8_text(source.as_bytes())
                .unwrap_or("<invalid utf8>");
            if text.len() <= self.config.max_text_length {
                format!(" \"{}\"", text.replace('\n', "\\n").replace('\r', "\\r"))
            } else {
                format!(
                    " \"{}...\"",
                    &text[..self.config.max_text_length.min(text.len())]
                )
            }
        } else {
            String::new()
        };

        // Write the formatted node
        output.push_str(&format!(
            "{}{}{}{}{}\n",
            node_prefix, node_type, position_info, byte_range_info, text_content
        ));

        // Process children
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i) {
                let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                let is_last_child = i == child_count - 1;
                self.visualize_node_recursive(
                    &child,
                    source,
                    depth + 1,
                    &child_prefix,
                    is_last_child,
                    output,
                );
            }
        }
    }

    /// Format node type with colors if enabled
    fn format_node_type(&self, node_type: &str) -> String {
        if !self.config.use_colors {
            return node_type.to_string();
        }

        if let Some(color_name) = self.config.node_color_names.get(node_type) {
            match color_name.as_str() {
                "blue" => node_type.blue().to_string(),
                "green" => node_type.green().to_string(),
                "cyan" => node_type.cyan().to_string(),
                "red" => node_type.red().to_string(),
                "yellow" => node_type.yellow().to_string(),
                "magenta" => node_type.magenta().to_string(),
                _ => node_type.normal().to_string(),
            }
        } else {
            // Default color for unknown node types
            node_type.normal().to_string()
        }
    }

    /// Visualize in list format
    fn visualize_list_format(&self, node: &Node, _source: &str) -> Result<String> {
        let mut output = String::new();
        let mut cursor = node.walk();
        let mut depth = 0;

        loop {
            let current_node = cursor.node();

            // Skip unnamed nodes if configured
            if !self.config.named_nodes_only || current_node.is_named() {
                let indent = self.config.indent_string.repeat(depth);
                let node_type = self.format_node_type(current_node.kind());

                let position_info = if self.config.show_positions {
                    let start = current_node.start_position();
                    format!(" @{}:{}", start.row + 1, start.column + 1)
                } else {
                    String::new()
                };

                output.push_str(&format!("{}{}{}\n", indent, node_type, position_info));
            }

            if cursor.goto_first_child() {
                depth += 1;
            } else if cursor.goto_next_sibling() {
                // Stay at same depth
            } else {
                // Go back up until we find a sibling or reach the root
                loop {
                    if !cursor.goto_parent() {
                        return Ok(output); // Reached root
                    }
                    depth -= 1;
                    if cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            if depth > self.config.max_depth {
                break;
            }
        }

        Ok(output)
    }

    /// Visualize in JSON format
    fn visualize_json_format(&self, node: &Node, source: &str) -> Result<String> {
        let json_node = self.node_to_json(node, source, 0)?;
        Ok(serde_json::to_string_pretty(&json_node)?)
    }

    /// Convert a node to JSON representation
    fn node_to_json(&self, node: &Node, source: &str, depth: usize) -> Result<serde_json::Value> {
        if depth > self.config.max_depth {
            return Ok(serde_json::json!({
                "type": "...",
                "truncated": true
            }));
        }

        let mut json_node = serde_json::Map::new();
        json_node.insert(
            "type".to_string(),
            serde_json::Value::String(node.kind().to_string()),
        );
        json_node.insert(
            "named".to_string(),
            serde_json::Value::Bool(node.is_named()),
        );

        if self.config.show_positions {
            let start = node.start_position();
            let end = node.end_position();
            json_node.insert(
                "start".to_string(),
                serde_json::json!({
                    "row": start.row,
                    "column": start.column
                }),
            );
            json_node.insert(
                "end".to_string(),
                serde_json::json!({
                    "row": end.row,
                    "column": end.column
                }),
            );
        }

        if self.config.show_byte_ranges {
            json_node.insert(
                "start_byte".to_string(),
                serde_json::Value::Number(node.start_byte().into()),
            );
            json_node.insert(
                "end_byte".to_string(),
                serde_json::Value::Number(node.end_byte().into()),
            );
        }

        if self.config.show_text_content && node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let display_text = if text.len() <= self.config.max_text_length {
                    text.to_string()
                } else {
                    format!(
                        "{}...",
                        &text[..self.config.max_text_length.min(text.len())]
                    )
                };
                json_node.insert("text".to_string(), serde_json::Value::String(display_text));
            }
        }

        let mut children = Vec::new();
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if !self.config.named_nodes_only || child.is_named() {
                    children.push(self.node_to_json(&child, source, depth + 1)?);
                }
            }
        }

        if !children.is_empty() {
            json_node.insert("children".to_string(), serde_json::Value::Array(children));
        }

        Ok(serde_json::Value::Object(json_node))
    }

    /// Visualize in S-expression format
    fn visualize_sexp_format(&self, node: &Node, source: &str) -> Result<String> {
        let mut output = String::new();
        self.node_to_sexp(node, source, 0, &mut output)?;
        Ok(output)
    }

    /// Convert node to S-expression format
    fn node_to_sexp(
        &self,
        node: &Node,
        source: &str,
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        if depth > self.config.max_depth {
            output.push_str("...");
            return Ok(());
        }

        if self.config.named_nodes_only && !node.is_named() {
            return Ok(());
        }

        output.push('(');
        output.push_str(node.kind());

        // Add text for leaf nodes
        if node.child_count() == 0 && self.config.show_text_content {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let display_text = if text.len() <= self.config.max_text_length {
                    text.to_string()
                } else {
                    format!(
                        "{}...",
                        &text[..self.config.max_text_length.min(text.len())]
                    )
                };
                output.push_str(&format!(" \"{}\"", display_text.replace('"', "\\\"")));
            }
        }

        // Process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if !self.config.named_nodes_only || child.is_named() {
                    output.push(' ');
                    self.node_to_sexp(&child, source, depth + 1, output)?;
                }
            }
        }

        output.push(')');
        Ok(())
    }

    /// Visualize in compact format
    fn visualize_compact_format(&self, node: &Node, source: &str) -> Result<String> {
        let mut output = String::new();
        self.node_to_compact(node, source, 0, &mut output)?;
        Ok(output.trim().to_string())
    }

    /// Convert node to compact format
    fn node_to_compact(
        &self,
        node: &Node,
        source: &str,
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        if depth > self.config.max_depth {
            output.push_str("...");
            return Ok(());
        }

        if self.config.named_nodes_only && !node.is_named() {
            return Ok(());
        }

        output.push_str(node.kind());

        if node.child_count() == 0 && self.config.show_text_content {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let display_text = if text.len() <= self.config.max_text_length {
                    text.to_string()
                } else {
                    format!(
                        "{}...",
                        &text[..self.config.max_text_length.min(text.len())]
                    )
                };
                output.push_str(&format!(":{}", display_text.replace(' ', "_")));
            }
        }

        if node.child_count() > 0 {
            output.push('[');
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if !self.config.named_nodes_only || child.is_named() {
                        if i > 0 {
                            output.push(',');
                        }
                        self.node_to_compact(&child, source, depth + 1, output)?;
                    }
                }
            }
            output.push(']');
        }

        Ok(())
    }

    /// Get statistics about the AST
    pub fn get_ast_statistics(&self, node: &Node) -> AstStatistics {
        let mut stats = AstStatistics::default();
        self.collect_statistics(node, &mut stats, 0);
        stats
    }

    /// Recursively collect AST statistics
    #[allow(clippy::only_used_in_recursion)]
    fn collect_statistics(&self, node: &Node, stats: &mut AstStatistics, depth: usize) {
        stats.total_nodes += 1;
        stats.max_depth = stats.max_depth.max(depth);

        if node.is_named() {
            stats.named_nodes += 1;
        } else {
            stats.unnamed_nodes += 1;
        }

        *stats
            .node_type_counts
            .entry(node.kind().to_string())
            .or_insert(0) += 1;

        if node.child_count() == 0 {
            stats.leaf_nodes += 1;
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_statistics(&child, stats, depth + 1);
            }
        }
    }

    /// Compare two ASTs and highlight differences
    pub fn compare_asts(&self, old_node: &Node, new_node: &Node, _source: &str) -> Result<String> {
        let mut output = String::new();
        output.push_str("=== AST Comparison ===\n\n");

        let old_stats = self.get_ast_statistics(old_node);
        let new_stats = self.get_ast_statistics(new_node);

        output.push_str("## Statistics Comparison\n");
        output.push_str(&format!(
            "Total nodes: {} -> {} ({}{})\n",
            old_stats.total_nodes,
            new_stats.total_nodes,
            if new_stats.total_nodes >= old_stats.total_nodes {
                "+"
            } else {
                ""
            },
            new_stats.total_nodes as i32 - old_stats.total_nodes as i32
        ));

        output.push_str(&format!(
            "Max depth: {} -> {} ({}{})\n",
            old_stats.max_depth,
            new_stats.max_depth,
            if new_stats.max_depth >= old_stats.max_depth {
                "+"
            } else {
                ""
            },
            new_stats.max_depth as i32 - old_stats.max_depth as i32
        ));

        output.push_str("\n## Structural Differences\n");
        if old_node.kind() != new_node.kind() {
            output.push_str(&format!(
                "Root node type changed: {} -> {}\n",
                old_node.kind(),
                new_node.kind()
            ));
        }

        Ok(output)
    }
}

impl Default for AstVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about an AST
#[derive(Debug, Default)]
pub struct AstStatistics {
    pub total_nodes: usize,
    pub named_nodes: usize,
    pub unnamed_nodes: usize,
    pub leaf_nodes: usize,
    pub max_depth: usize,
    pub node_type_counts: HashMap<String, usize>,
}

impl fmt::Display for AstStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AST Statistics:")?;
        writeln!(f, "  Total nodes: {}", self.total_nodes)?;
        writeln!(f, "  Named nodes: {}", self.named_nodes)?;
        writeln!(f, "  Unnamed nodes: {}", self.unnamed_nodes)?;
        writeln!(f, "  Leaf nodes: {}", self.leaf_nodes)?;
        writeln!(f, "  Maximum depth: {}", self.max_depth)?;
        writeln!(f, "  Node types:")?;

        let mut types: Vec<_> = self.node_type_counts.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

        for (node_type, count) in types.iter().take(10) {
            // Show top 10
            writeln!(f, "    {}: {}", node_type, count)?;
        }

        if types.len() > 10 {
            writeln!(f, "    ... and {} more", types.len() - 10)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    #[allow(dead_code)]
    fn create_test_parser() -> Parser {
        // For testing, we'll use a simple language grammar
        // In real usage, this would use the appropriate language
        Parser::new()
    }

    #[test]
    fn test_ast_visualizer_creation() {
        let visualizer = AstVisualizer::new();
        assert_eq!(visualizer.config.max_depth, 20);
        assert!(visualizer.config.show_positions);
    }

    #[test]
    fn test_custom_config() {
        let config = VisualizationConfig {
            max_depth: 10,
            show_positions: false,
            ..Default::default()
        };

        let visualizer = AstVisualizer::with_config(config);
        assert_eq!(visualizer.config.max_depth, 10);
        assert!(!visualizer.config.show_positions);
    }

    #[test]
    fn test_format_node_type_with_colors() {
        let visualizer = AstVisualizer::new();
        let formatted = visualizer.format_node_type("function_definition");
        // Note: Testing colored output is difficult, so we just ensure it doesn't panic
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_format_node_type_without_colors() {
        let config = VisualizationConfig {
            use_colors: false,
            ..Default::default()
        };
        let visualizer = AstVisualizer::with_config(config);

        let formatted = visualizer.format_node_type("function_definition");
        assert_eq!(formatted, "function_definition");
    }

    #[test]
    fn test_ast_statistics_display() {
        let mut stats = AstStatistics {
            total_nodes: 100,
            named_nodes: 80,
            unnamed_nodes: 20,
            max_depth: 5,
            ..Default::default()
        };
        stats.node_type_counts.insert("function".to_string(), 10);
        stats.node_type_counts.insert("identifier".to_string(), 30);

        let output = format!("{}", stats);
        assert!(output.contains("Total nodes: 100"));
        assert!(output.contains("Named nodes: 80"));
        assert!(output.contains("Maximum depth: 5"));
    }
}
