//! Adapter to integrate Java parser with codeprism

use crate::parser::{ParseContext as JavaParseContext, JavaParser};
use crate::types as java_types;

/// Adapter that implements codeprism's LanguageParser trait
pub struct JavaLanguageParser {
    parser: std::sync::Mutex<JavaParser>,
}

impl JavaLanguageParser {
    /// Create a new Java language parser adapter
    pub fn new() -> Self {
        Self {
            parser: std::sync::Mutex::new(JavaParser::new()),
        }
    }
}

impl Default for JavaLanguageParser {
    fn default() -> Self {
        Self::new()
    }
}

// Since we can't import codeprism types directly, we'll need to define a conversion
// trait that the caller can implement
pub trait ParseResultConverter {
    type Node;
    type Edge;
    type ParseResult;

    fn convert_node(node: java_types::Node) -> Self::Node;
    fn convert_edge(edge: java_types::Edge) -> Self::Edge;
    fn create_parse_result(
        tree: tree_sitter::Tree,
        nodes: Vec<Self::Node>,
        edges: Vec<Self::Edge>,
    ) -> Self::ParseResult;
}

/// Parse a file and return the result in our internal types
pub fn parse_file(
    parser: &JavaLanguageParser,
    repo_id: &str,
    file_path: std::path::PathBuf,
    content: String,
    old_tree: Option<tree_sitter::Tree>,
) -> Result<(tree_sitter::Tree, Vec<java_types::Node>, Vec<java_types::Edge>), crate::error::Error> {
    let context = JavaParseContext {
        repo_id: repo_id.to_string(),
        file_path,
        old_tree,
        content,
    };

    let mut parser = parser.parser.lock().unwrap();
    let result = parser.parse(&context)?;

    Ok((result.tree, result.nodes, result.edges))
} 