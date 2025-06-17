//! Adapter to integrate JavaScript/TypeScript parser with prism

use crate::parser::{JavaScriptParser, ParseContext as JsParseContext};
use crate::types as js_types;

/// Adapter that implements prism's LanguageParser trait
pub struct JavaScriptLanguageParser {
    parser: std::sync::Mutex<JavaScriptParser>,
}

impl JavaScriptLanguageParser {
    /// Create a new JavaScript language parser adapter
    pub fn new() -> Self {
        Self {
            parser: std::sync::Mutex::new(JavaScriptParser::new()),
        }
    }
}

impl Default for JavaScriptLanguageParser {
    fn default() -> Self {
        Self::new()
    }
}

// Since we can't import prism types directly, we'll need to define a conversion
// trait that the caller can implement
pub trait ParseResultConverter {
    type Node;
    type Edge;
    type ParseResult;

    fn convert_node(node: js_types::Node) -> Self::Node;
    fn convert_edge(edge: js_types::Edge) -> Self::Edge;
    fn create_parse_result(
        tree: tree_sitter::Tree,
        nodes: Vec<Self::Node>,
        edges: Vec<Self::Edge>,
    ) -> Self::ParseResult;
}

/// Parse a file and return the result in our internal types
pub fn parse_file(
    parser: &JavaScriptLanguageParser,
    repo_id: &str,
    file_path: std::path::PathBuf,
    content: String,
    old_tree: Option<tree_sitter::Tree>,
) -> Result<(tree_sitter::Tree, Vec<js_types::Node>, Vec<js_types::Edge>), crate::error::Error> {
    let context = JsParseContext {
        repo_id: repo_id.to_string(),
        file_path,
        old_tree,
        content,
    };

    let mut parser = parser.parser.lock().unwrap();
    let result = parser.parse(&context)?;

    Ok((result.tree, result.nodes, result.edges))
}
