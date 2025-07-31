//! Python parser implementation

use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

/// Parse context for Python files
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Repository ID
    pub repo_id: String,
    /// File path being parsed
    pub file_path: PathBuf,
    /// Previous tree for incremental parsing
    pub old_tree: Option<Tree>,
    /// File content
    pub content: String,
}

/// Parse result containing nodes and edges
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed tree
    pub tree: Tree,
    /// Extracted nodes
    pub nodes: Vec<Node>,
    /// Extracted edges
    pub edges: Vec<Edge>,
}

/// Python parser
pub struct PythonParser {
    /// Tree-sitter parser for Python
    parser: Parser,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");

        Self { parser }
    }

    /// Get the language for a file based on its extension
    pub fn detect_language(path: &Path) -> Language {
        // All Python files are Python language
        match path.extension().and_then(|s| s.to_str()) {
            Some("py") | Some("pyw") => Language::Python,
            _ => Language::Python, // Default to Python
        }
    }

    /// Parse a Python file
    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
        let language = Self::detect_language(&context.file_path);

        // Parse the file
        let tree = self
            .parser
            .parse(&context.content, context.old_tree.as_ref())
            .ok_or_else(|| Error::parse(&context.file_path, "Failed to parse file"))?;

        // Extract nodes and edges
        let mapper = AstMapper::new(
            &context.repo_id,
            context.file_path.clone(),
            language,
            &context.content,
        );

        let (nodes, edges) = mapper.extract(&tree)?;

        Ok(ParseResult { tree, nodes, edges })
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            PythonParser::detect_language(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            PythonParser::detect_language(Path::new("test.pyw")),
            Language::Python
        );
    }

    #[test]
    fn test_parse_simple_python() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "def hello():\n    return 'world'".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty(), "Should not be empty");

        // Should have at least a module node and a function node
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Module)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Function)));
    }

    #[test]
    fn test_parse_class() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "class MyClass:\n    def method(self):\n        pass".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty(), "Should not be empty");

        // Should have module, class, and method nodes
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Module)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Class)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Method)));
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = PythonParser::new();

        // First parse
        let context1 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "def foo():\n    return 1".to_string(),
        };
        let result1 = parser.parse(&context1).unwrap();

        // Second parse with small edit
        let context2 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: Some(result1.tree),
            content: "def foo():\n    return 2".to_string(),
        };
        let result2 = parser.parse(&context2).unwrap();

        // Both should have the same structure
        assert_eq!(result1.nodes.len(), result2.nodes.len());

        // Function should still be found
        let func1 = result1
            .nodes
            .iter()
            .find(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .unwrap();
        let func2 = result2
            .nodes
            .iter()
            .find(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .unwrap();

        assert_eq!(func1.name, "foo");
        assert_eq!(func2.name, "foo");
    }

    #[test]
    fn test_parse_multiple_functions() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "def foo():\n    pass\n\ndef bar():\n    pass".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        println!("Parsed nodes:");
        for node in &result.nodes {
            println!("  {:?} - {}", node.kind, node.name);
        }

        // Should have a module and two functions
        assert!(result.nodes.len() >= 3);

        let func_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .collect();

        assert_eq!(func_nodes.len(), 2, "Should have 2 items");
        assert!(func_nodes.iter().any(|n| n.name == "foo"));
        assert!(func_nodes.iter().any(|n| n.name == "bar"));
    }

    #[test]
    fn test_parse_imports() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "import os\nfrom sys import path\nimport json as j".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        let import_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Import))
            .collect();

        // Should have at least one import node
        assert!(!import_nodes.is_empty(), "Should not be empty");
    }
}
