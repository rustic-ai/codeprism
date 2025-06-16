//! JavaScript/TypeScript parser implementation

use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

/// Parse context for JavaScript/TypeScript files
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

/// JavaScript/TypeScript parser
pub struct JavaScriptParser {
    /// Tree-sitter parser for JavaScript
    js_parser: Parser,
    /// Tree-sitter parser for TypeScript
    ts_parser: Parser,
}

impl JavaScriptParser {
    /// Create a new JavaScript/TypeScript parser
    pub fn new() -> Self {
        let mut js_parser = Parser::new();
        js_parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load JavaScript grammar");

        let mut ts_parser = Parser::new();
        ts_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Failed to load TypeScript grammar");

        Self {
            js_parser,
            ts_parser,
        }
    }

    /// Get the language for a file based on its extension
    pub fn detect_language(path: &Path) -> Language {
        match path.extension().and_then(|s| s.to_str()) {
            Some("ts") | Some("tsx") => Language::TypeScript,
            _ => Language::JavaScript,
        }
    }

    /// Parse a JavaScript or TypeScript file
    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
        let language = Self::detect_language(&context.file_path);

        // Select the appropriate parser
        let parser = match language {
            Language::JavaScript => &mut self.js_parser,
            Language::TypeScript => &mut self.ts_parser,
        };

        // Parse the file
        let tree = parser
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

impl Default for JavaScriptParser {
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
            JavaScriptParser::detect_language(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            JavaScriptParser::detect_language(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            JavaScriptParser::detect_language(Path::new("test.tsx")),
            Language::TypeScript
        );
        assert_eq!(
            JavaScriptParser::detect_language(Path::new("test.mjs")),
            Language::JavaScript
        );
    }

    #[test]
    fn test_parse_simple_javascript() {
        let mut parser = JavaScriptParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None,
            content: "function hello() { return 'world'; }".to_string(),
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
    fn test_parse_typescript() {
        let mut parser = JavaScriptParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.ts"),
            old_tree: None,
            content: "function hello(name: string): string { return `Hello, ${name}!`; }"
                .to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

        // Should detect TypeScript
        let func_node = result
            .nodes
            .iter()
            .find(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .expect("Should have a function node");

        assert_eq!(func_node.lang, Language::TypeScript);
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = JavaScriptParser::new();

        // First parse
        let context1 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None,
            content: "function foo() { return 1; }".to_string(),
        };
        let result1 = parser.parse(&context1).unwrap();

        // Second parse with small edit - change return value
        // This is what incremental parsing is designed for
        let context2 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: Some(result1.tree),
            content: "function foo() { return 2; }".to_string(),
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

        // For larger changes, don't use incremental parsing
        let context3 = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None, // Don't use old tree for major changes
            content: "function foo() { return 2; }\nfunction bar() { return 3; }".to_string(),
        };
        let result3 = parser.parse(&context3).unwrap();

        // Should find both functions
        let func_count = result3
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .count();
        assert_eq!(func_count, 2);
    }

    #[test]
    fn test_debug_tree_sitter_nodes() {
        let mut parser = JavaScriptParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None,
            content: "function foo() {}\nfunction bar() {}".to_string(),
        };

        let tree = parser.js_parser.parse(&context.content, None).unwrap();
        let mut cursor = tree.walk();

        fn print_tree(cursor: &mut tree_sitter::TreeCursor, depth: usize) {
            let node = cursor.node();
            println!(
                "{}{} [{:?}]",
                "  ".repeat(depth),
                node.kind(),
                node.start_byte()..node.end_byte()
            );

            if cursor.goto_first_child() {
                loop {
                    print_tree(cursor, depth + 1);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }

        print_tree(&mut cursor, 0);
    }

    #[test]
    fn test_parse_multiple_functions() {
        let mut parser = JavaScriptParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None,
            content: "function foo() {}\nfunction bar() {}".to_string(),
        };

        let result = parser.parse(&context).unwrap();

        println!("Parsed nodes:");
        for node in &result.nodes {
            println!("  {:?} - {}", node.kind, node.name);
        }

        // Should have a module and two functions
        assert_eq!(result.nodes.len(), 3);
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, crate::types::NodeKind::Module)));

        let func_nodes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, crate::types::NodeKind::Function))
            .collect();

        assert_eq!(func_nodes.len(), 2);
        assert!(func_nodes.iter().any(|n| n.name == "foo"));
        assert!(func_nodes.iter().any(|n| n.name == "bar"));
    }

    #[test]
    fn test_debug_ast_mapper() {
        let mut parser = JavaScriptParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.js"),
            old_tree: None,
            content: "function foo() {}\nfunction bar() {}".to_string(),
        };

        let tree = parser.js_parser.parse(&context.content, None).unwrap();

        // Create mapper and extract
        let mapper = crate::ast_mapper::AstMapper::new(
            &context.repo_id,
            context.file_path.clone(),
            Language::JavaScript,
            &context.content,
        );

        let (nodes, edges) = mapper.extract(&tree).unwrap();

        println!("Extracted nodes:");
        for node in &nodes {
            println!("  {:?} - {} at {:?}", node.kind, node.name, node.span);
        }

        println!("\nExtracted edges:");
        for edge in &edges {
            println!("  {:?}", edge.kind);
        }
    }
}
