//! Rust parser implementation

use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

/// Parse context for Rust files
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

/// Rust parser
pub struct RustParser {
    /// Tree-sitter parser for Rust
    parser: Parser,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Failed to load Rust grammar");

        Self { parser }
    }

    /// Get the language for a file based on its extension
    pub fn detect_language(path: &Path) -> Language {
        // All Rust files are Rust language
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            _ => Language::Rust, // Default to Rust
        }
    }

    /// Parse a Rust file
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

impl Default for RustParser {
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
            RustParser::detect_language(Path::new("test.rs")),
            Language::Rust
        );
    }

    #[test]
    fn test_parse_simple_rust() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "fn hello() -> &'static str {\n    \"world\"\n}".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

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
    fn test_parse_struct() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "struct Point { x: i32, y: i32 }".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

        // Should have module and struct nodes
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Module)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Struct)));
    }

    #[test]
    fn test_parse_trait_and_impl() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content:
                "trait Display { fn fmt(&self); }\nimpl Display for String { fn fmt(&self) {} }"
                    .to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

        // Should have trait and impl nodes
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Trait)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Impl)));
    }

    #[test]
    fn test_parse_use_statements() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "use std::collections::HashMap;\nuse serde::{Serialize, Deserialize};"
                .to_string(),
        };

        let result = parser.parse(&context).unwrap();

        let use_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Use))
            .collect();

        // Should have at least one use node
        assert!(!use_nodes.is_empty());
    }

    #[test]
    fn test_parse_enum() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "enum Color { Red, Green, Blue }".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have enum node
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Enum)));
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = RustParser::new();

        // First parse
        let context1 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "fn foo() -> i32 {\n    1\n}".to_string(),
        };
        let result1 = parser.parse(&context1).unwrap();

        // Second parse with small edit
        let context2 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: Some(result1.tree),
            content: "fn foo() -> i32 {\n    2\n}".to_string(),
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
}
