//! Adapter to integrate Python parser with prism

use crate::parser::{ParseContext as PyParseContext, PythonParser};
use crate::types as py_types;

/// Adapter that implements prism's LanguageParser trait
pub struct PythonLanguageParser {
    parser: std::sync::Mutex<PythonParser>,
}

impl PythonLanguageParser {
    /// Create a new Python language parser adapter
    pub fn new() -> Self {
        Self {
            parser: std::sync::Mutex::new(PythonParser::new()),
        }
    }
}

impl Default for PythonLanguageParser {
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

    fn convert_node(node: py_types::Node) -> Self::Node;
    fn convert_edge(edge: py_types::Edge) -> Self::Edge;
    fn create_parse_result(
        tree: tree_sitter::Tree,
        nodes: Vec<Self::Node>,
        edges: Vec<Self::Edge>,
    ) -> Self::ParseResult;
}

/// Parse a file and return the result in our internal types
pub fn parse_file(
    parser: &PythonLanguageParser,
    repo_id: &str,
    file_path: std::path::PathBuf,
    content: String,
    old_tree: Option<tree_sitter::Tree>,
) -> Result<(tree_sitter::Tree, Vec<py_types::Node>, Vec<py_types::Edge>), crate::error::Error> {
    let context = PyParseContext {
        repo_id: repo_id.to_string(),
        file_path,
        old_tree,
        content,
    };

    let mut parser = parser.parser.lock().unwrap();
    let result = parser.parse(&context)?;

    Ok((result.tree, result.nodes, result.edges))
}
