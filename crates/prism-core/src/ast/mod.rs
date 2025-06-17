//! Universal Abstract Syntax Tree (U-AST) types
//!
//! This module defines language-agnostic AST node and edge types that can
//! represent code structures from any supported programming language.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Unique identifier for AST nodes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId([u8; 16]);

impl NodeId {
    /// Create a new NodeId from components
    pub fn new(repo_id: &str, file_path: &Path, span: &Span, kind: &NodeKind) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(repo_id.as_bytes());
        hasher.update(file_path.to_string_lossy().as_bytes());
        hasher.update(&span.start_byte.to_le_bytes());
        hasher.update(&span.end_byte.to_le_bytes());
        hasher.update(format!("{:?}", kind).as_bytes());

        let hash = hasher.finalize();
        let mut id = [0u8; 16];
        id.copy_from_slice(&hash.as_bytes()[..16]);
        Self(id)
    }

    /// Get the ID as a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a NodeId from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 16 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut id = [0u8; 16];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", &self.to_hex()[..8])
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..8])
    }
}

/// Types of nodes in the Universal AST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A module or file
    Module,
    /// A class definition
    Class,
    /// A function definition
    Function,
    /// A method definition
    Method,
    /// A function/method parameter
    Parameter,
    /// A variable declaration
    Variable,
    /// A function/method call
    Call,
    /// An import statement
    Import,
    /// A literal value
    Literal,
    /// An HTTP route definition
    Route,
    /// A SQL query
    SqlQuery,
    /// An event emission
    Event,
    /// Unknown node type
    Unknown,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Module => write!(f, "Module"),
            NodeKind::Class => write!(f, "Class"),
            NodeKind::Function => write!(f, "Function"),
            NodeKind::Method => write!(f, "Method"),
            NodeKind::Parameter => write!(f, "Parameter"),
            NodeKind::Variable => write!(f, "Variable"),
            NodeKind::Call => write!(f, "Call"),
            NodeKind::Import => write!(f, "Import"),
            NodeKind::Literal => write!(f, "Literal"),
            NodeKind::Route => write!(f, "Route"),
            NodeKind::SqlQuery => write!(f, "SqlQuery"),
            NodeKind::Event => write!(f, "Event"),
            NodeKind::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Types of edges between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeKind {
    /// Function/method call
    Calls,
    /// Variable/field read
    Reads,
    /// Variable/field write
    Writes,
    /// Module import
    Imports,
    /// Event emission
    Emits,
    /// HTTP route mapping
    RoutesTo,
    /// Exception raising
    Raises,
    /// Type inheritance
    Extends,
    /// Interface implementation
    Implements,
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeKind::Calls => write!(f, "CALLS"),
            EdgeKind::Reads => write!(f, "READS"),
            EdgeKind::Writes => write!(f, "WRITES"),
            EdgeKind::Imports => write!(f, "IMPORTS"),
            EdgeKind::Emits => write!(f, "EMITS"),
            EdgeKind::RoutesTo => write!(f, "ROUTES_TO"),
            EdgeKind::Raises => write!(f, "RAISES"),
            EdgeKind::Extends => write!(f, "EXTENDS"),
            EdgeKind::Implements => write!(f, "IMPLEMENTS"),
        }
    }
}

/// Source code location
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Starting byte offset
    pub start_byte: usize,
    /// Ending byte offset (exclusive)
    pub end_byte: usize,
    /// Starting line (1-indexed)
    pub start_line: usize,
    /// Ending line (1-indexed)
    pub end_line: usize,
    /// Starting column (1-indexed)
    pub start_column: usize,
    /// Ending column (1-indexed)
    pub end_column: usize,
}

impl Span {
    /// Create a new span
    pub fn new(
        start_byte: usize,
        end_byte: usize,
        start_line: usize,
        end_line: usize,
        start_column: usize,
        end_column: usize,
    ) -> Self {
        Self {
            start_byte,
            end_byte,
            start_line,
            end_line,
            start_column,
            end_column,
        }
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.end_byte - self.start_byte
    }

    /// Check if the span is empty
    pub fn is_empty(&self) -> bool {
        self.start_byte == self.end_byte
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start_line, self.start_column, self.end_line, self.end_column
        )
    }
}

/// Programming language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// JavaScript
    JavaScript,
    /// TypeScript
    TypeScript,
    /// Python
    Python,
    /// Java
    Java,
    /// Go
    Go,
    /// Rust
    Rust,
    /// C
    C,
    /// C++
    Cpp,
    /// Unknown language
    Unknown,
}

impl Language {
    /// Get language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "py" | "pyw" => Language::Python,
            "java" => Language::Java,
            "go" => Language::Go,
            "rs" => Language::Rust,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Language::Cpp,
            _ => Language::Unknown,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Python => write!(f, "Python"),
            Language::Java => write!(f, "Java"),
            Language::Go => write!(f, "Go"),
            Language::Rust => write!(f, "Rust"),
            Language::C => write!(f, "C"),
            Language::Cpp => write!(f, "C++"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

/// A node in the Universal AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Node type
    pub kind: NodeKind,
    /// Node name (e.g., function name)
    pub name: String,
    /// Programming language
    pub lang: Language,
    /// Source file path
    pub file: PathBuf,
    /// Source location
    pub span: Span,
    /// Optional type signature
    pub signature: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl Node {
    /// Create a new node
    pub fn new(
        repo_id: &str,
        kind: NodeKind,
        name: String,
        lang: Language,
        file: PathBuf,
        span: Span,
    ) -> Self {
        let id = NodeId::new(repo_id, &file, &span, &kind);
        Self {
            id,
            kind,
            name,
            lang,
            file,
            span,
            signature: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a new node with an `Arc<PathBuf>`
    pub fn with_arc(
        repo_id: &str,
        kind: NodeKind,
        name: String,
        lang: Language,
        file: Arc<PathBuf>,
        span: Span,
    ) -> Self {
        let id = NodeId::new(repo_id, &file, &span, &kind);
        Self {
            id,
            kind,
            name,
            lang,
            file: (*file).clone(),
            span,
            signature: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set the type signature
    pub fn with_signature(mut self, sig: String) -> Self {
        self.signature = Some(sig);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} '{}' at {}:{}",
            self.lang,
            self.kind,
            self.name,
            self.file.display(),
            self.span
        )
    }
}

/// Builder for creating nodes
pub struct NodeBuilder {
    repo_id: String,
    kind: NodeKind,
    name: String,
    lang: Language,
    file: PathBuf,
    span: Span,
    signature: Option<String>,
    metadata: serde_json::Value,
}

impl NodeBuilder {
    /// Create a new node builder
    pub fn new(repo_id: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            repo_id: repo_id.into(),
            kind,
            name: String::new(),
            lang: Language::Unknown,
            file: PathBuf::new(),
            span: Span::new(0, 0, 1, 1, 1, 1),
            signature: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set the node name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the language
    pub fn language(mut self, lang: Language) -> Self {
        self.lang = lang;
        self
    }

    /// Set the file path
    pub fn file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = file.into();
        self
    }

    /// Set the span
    pub fn span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Set the type signature
    pub fn signature(mut self, sig: impl Into<String>) -> Self {
        self.signature = Some(sig.into());
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the node
    pub fn build(self) -> Node {
        let id = NodeId::new(&self.repo_id, &self.file, &self.span, &self.kind);
        Node {
            id,
            kind: self.kind,
            name: self.name,
            lang: self.lang,
            file: self.file,
            span: self.span,
            signature: self.signature,
            metadata: self.metadata,
        }
    }
}

/// An edge between nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID
    pub source: NodeId,
    /// Target node ID
    pub target: NodeId,
    /// Edge type
    pub kind: EdgeKind,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: NodeId, target: NodeId, kind: EdgeKind) -> Self {
        Self {
            source,
            target,
            kind,
        }
    }

    /// Get a unique ID for this edge
    pub fn id(&self) -> String {
        format!("{}>{}>:{:?}", self.source, self.target, self.kind)
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} --{}-> {}", self.source, self.kind, self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generation() {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let id1 = NodeId::new("repo1", Path::new("file.js"), &span, &NodeKind::Function);
        let id2 = NodeId::new("repo1", Path::new("file.js"), &span, &NodeKind::Function);
        assert_eq!(id1, id2);

        let id3 = NodeId::new("repo2", Path::new("file.js"), &span, &NodeKind::Function);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_node_id_edge_cases() {
        // Empty path
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let id1 = NodeId::new("repo", Path::new(""), &span, &NodeKind::Module);
        assert!(!id1.to_hex().is_empty());

        // Path with special characters
        let id2 = NodeId::new(
            "repo",
            Path::new("src/@types/index.d.ts"),
            &span,
            &NodeKind::Module,
        );
        assert!(!id2.to_hex().is_empty());

        // Unicode in path
        let id3 = NodeId::new("repo", Path::new("src/文件.js"), &span, &NodeKind::Module);
        assert!(!id3.to_hex().is_empty());
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }

    #[test]
    fn test_language_detection_edge_cases() {
        // Case insensitive
        assert_eq!(Language::from_extension("JS"), Language::JavaScript);
        assert_eq!(Language::from_extension("Py"), Language::Python);

        // Multiple extensions
        assert_eq!(Language::from_extension("mjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("cjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("tsx"), Language::TypeScript);

        // C++ variations
        assert_eq!(Language::from_extension("cpp"), Language::Cpp);
        assert_eq!(Language::from_extension("cc"), Language::Cpp);
        assert_eq!(Language::from_extension("cxx"), Language::Cpp);
        assert_eq!(Language::from_extension("hpp"), Language::Cpp);

        // Empty and unknown
        assert_eq!(Language::from_extension(""), Language::Unknown);
        assert_eq!(Language::from_extension("xyz"), Language::Unknown);
    }

    #[test]
    fn test_span_utilities() {
        let span = Span::new(10, 20, 2, 3, 5, 15);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let empty_span = Span::new(10, 10, 2, 2, 5, 5);
        assert_eq!(empty_span.len(), 0);
        assert!(empty_span.is_empty());
    }

    #[test]
    fn test_node_serialization() {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let node = Node::new(
            "test_repo",
            NodeKind::Function,
            "test_func".to_string(),
            Language::JavaScript,
            PathBuf::from("test.js"),
            span,
        );

        // Test serialization round-trip
        let serialized = serde_json::to_string(&node).unwrap();
        let deserialized: Node = serde_json::from_str(&serialized).unwrap();

        assert_eq!(node.id, deserialized.id);
        assert_eq!(node.name, deserialized.name);
        assert_eq!(node.file, deserialized.file);
    }

    #[test]
    fn test_node_with_methods() {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let node = Node::new(
            "test_repo",
            NodeKind::Function,
            "test_func".to_string(),
            Language::JavaScript,
            PathBuf::from("test.js"),
            span,
        )
        .with_signature("(a: number, b: number) => number".to_string())
        .with_metadata(serde_json::json!({ "async": true }));

        assert_eq!(
            node.signature,
            Some("(a: number, b: number) => number".to_string())
        );
        assert_eq!(node.metadata["async"], true);
    }

    #[test]
    fn test_node_builder() {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let node = NodeBuilder::new("test_repo", NodeKind::Function)
            .name("myFunction")
            .language(Language::TypeScript)
            .file("src/index.ts")
            .span(span.clone())
            .signature("() => void")
            .metadata(serde_json::json!({ "exported": true }))
            .build();

        assert_eq!(node.name, "myFunction");
        assert_eq!(node.lang, Language::TypeScript);
        assert_eq!(node.file, PathBuf::from("src/index.ts"));
        assert_eq!(node.span, span);
        assert_eq!(node.signature, Some("() => void".to_string()));
        assert_eq!(node.metadata["exported"], true);
    }

    #[test]
    fn test_edge_creation_and_serialization() {
        let span1 = Span::new(0, 10, 1, 1, 1, 11);
        let span2 = Span::new(20, 30, 2, 1, 2, 11);

        let id1 = NodeId::new("repo", Path::new("file.js"), &span1, &NodeKind::Function);
        let id2 = NodeId::new("repo", Path::new("file.js"), &span2, &NodeKind::Function);

        let edge = Edge::new(id1, id2, EdgeKind::Calls);
        assert_eq!(edge.source, id1);
        assert_eq!(edge.target, id2);
        assert_eq!(edge.kind, EdgeKind::Calls);

        // Test serialization
        let serialized = serde_json::to_string(&edge).unwrap();
        let deserialized: Edge = serde_json::from_str(&serialized).unwrap();
        assert_eq!(edge, deserialized);

        // Test edge ID
        let edge_id = edge.id();
        assert!(edge_id.contains(&id1.to_string()));
        assert!(edge_id.contains(&id2.to_string()));
        assert!(edge_id.contains("Calls"));
    }

    #[test]
    fn test_display_traits() {
        // NodeKind display
        assert_eq!(NodeKind::Function.to_string(), "Function");
        assert_eq!(NodeKind::Module.to_string(), "Module");

        // EdgeKind display
        assert_eq!(EdgeKind::Calls.to_string(), "CALLS");
        assert_eq!(EdgeKind::Imports.to_string(), "IMPORTS");

        // Language display
        assert_eq!(Language::JavaScript.to_string(), "JavaScript");
        assert_eq!(Language::Cpp.to_string(), "C++");

        // Span display
        let span = Span::new(0, 10, 1, 5, 2, 15);
        assert_eq!(span.to_string(), "1:2-5:15");

        // Node display
        let node = Node::new(
            "repo",
            NodeKind::Function,
            "myFunc".to_string(),
            Language::JavaScript,
            PathBuf::from("test.js"),
            span.clone(),
        );
        let display = node.to_string();
        assert!(display.contains("JavaScript"));
        assert!(display.contains("Function"));
        assert!(display.contains("myFunc"));
        assert!(display.contains("test.js"));

        // Edge display
        let id1 = NodeId::new("repo", Path::new("file.js"), &span, &NodeKind::Function);
        let id2 = NodeId::new("repo", Path::new("file.js"), &span, &NodeKind::Variable);
        let edge = Edge::new(id1, id2, EdgeKind::Reads);
        let edge_display = edge.to_string();
        assert!(edge_display.contains("READS"));
    }
}
