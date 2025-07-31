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
    fn test_parse_struct() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "struct Point { x: i32, y: i32 }".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty(), "Should not be empty");

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
        assert!(!result.nodes.is_empty(), "Should not be empty");

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
        assert!(!use_nodes.is_empty(), "Should not be empty");
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

    #[test]
    fn test_ownership_patterns() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "fn process_data(data: Vec<String>, buffer: &mut [u8], reference: &str) -> &str { reference }".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have function and parameter nodes
        let func_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .collect();
        assert_eq!(func_nodes.len(), 1, "Should have 1 items");

        let param_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Parameter))
            .collect();

        // Should have parameters with ownership information
        assert!(param_nodes.len() >= 3);

        // Check that at least one parameter has ownership metadata
        let has_ownership_metadata = param_nodes.iter().any(|node| {
            node.metadata
                .as_object()
                .is_some_and(|metadata| metadata.contains_key("ownership"))
        });
        assert!(has_ownership_metadata);
    }

    #[test]
    fn test_lifetime_annotations() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have lifetime nodes
        let lifetime_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Lifetime))
            .collect();

        // Should have at least one lifetime node
        assert!(!lifetime_nodes.is_empty(), "Should not be empty");

        // Check for 'a lifetime
        let has_a_lifetime = lifetime_nodes.iter().any(|node| node.name.contains("'a"));
        assert!(has_a_lifetime);
    }

    #[test]
    fn test_trait_bounds_and_impl() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "trait Clone { fn clone(&self) -> Self; }\nstruct Point { x: i32, y: i32 }\nimpl Clone for Point { fn clone(&self) -> Self { *self } }".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have trait and impl nodes
        let trait_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Trait))
            .collect();
        assert_eq!(trait_nodes.len(), 1, "Should have 1 items");

        let impl_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Impl))
            .collect();
        assert_eq!(impl_nodes.len(), 1, "Should have 1 items");

        // Check impl metadata
        let impl_node = &impl_nodes[0];
        assert!(impl_node.metadata.as_object().is_some_and(|metadata| {
            metadata.get("impl_type") == Some(&serde_json::Value::String("trait_impl".to_string()))
        }));
    }

    #[test]
    fn test_derive_attributes() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "#[derive(Debug, Clone, PartialEq)]\nstruct Point { x: i32, y: i32 }"
                .to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have attribute nodes
        let attr_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Attribute))
            .collect();

        assert!(!attr_nodes.is_empty(), "Should not be empty");

        // Check for derive attribute with traits
        let has_derive_attr = attr_nodes.iter().any(|node| {
            node.name.contains("derive")
                && node.metadata.as_object().is_some_and(|metadata| {
                    metadata.get("attribute_type")
                        == Some(&serde_json::Value::String("derive".to_string()))
                })
        });
        assert!(has_derive_attr);
    }

    #[test]
    fn test_macro_invocations() {
        let mut parser = RustParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.rs"),
            old_tree: None,
            content: "fn main() { println!(\"Hello, world!\"); vec![1, 2, 3]; }".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        // Should have call nodes for macro invocations
        let call_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Call))
            .collect();

        // Should have at least println! and vec! macro calls
        assert!(call_nodes.len() >= 2);

        // Check for macro call names
        let has_println = call_nodes.iter().any(|node| node.name.contains("println!"));
        let has_vec = call_nodes.iter().any(|node| node.name.contains("vec!"));

        assert!(has_println);
        assert!(has_vec);
    }
}
