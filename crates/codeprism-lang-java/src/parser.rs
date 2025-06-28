//! Java parser implementation

use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

/// Parse context for Java files
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

/// Java parser
pub struct JavaParser {
    /// Tree-sitter parser for Java
    parser: Parser,
}

impl JavaParser {
    /// Create a new Java parser
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_java::LANGUAGE.into())
            .expect("Failed to load Java grammar");

        Self { parser }
    }

    /// Get the language for a file based on its extension
    pub fn detect_language(path: &Path) -> Language {
        // All Java files are Java language
        match path.extension().and_then(|s| s.to_str()) {
            Some("java") => Language::Java,
            _ => Language::Java, // Default to Java
        }
    }

    /// Parse a Java file
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

impl Default for JavaParser {
    fn default() -> Self {
        Self::new()
    }
} 