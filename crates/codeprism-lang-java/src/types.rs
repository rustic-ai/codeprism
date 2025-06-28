//! Types for Java parser
//!
//! These types mirror the ones in codeprism_core::ast but are defined here to avoid
//! circular dependencies. The parser returns these types which are then
//! converted to codeprism types by the caller.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", &self.to_hex()[..8])
    }
}

/// Types of nodes in the Universal AST for Java
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

    // Java-specific node types
    /// An interface definition
    Interface,
    /// An enum definition
    Enum,
    /// A package declaration
    Package,
    /// An annotation definition
    Annotation,
    /// A constructor definition
    Constructor,
    /// A field in a class
    Field,
    /// A static initialization block
    StaticBlock,
    /// An instance initialization block
    InstanceBlock,
    /// A try-catch-finally block
    TryBlock,
    /// A catch clause
    CatchClause,
    /// A finally clause
    FinallyClause,
    /// A throw statement
    ThrowStatement,
    /// A lambda expression
    Lambda,
    /// A method reference
    MethodReference,
    /// A generic type parameter
    TypeParameter,
    /// A wildcard type
    WildcardType,
    /// An array creation expression
    ArrayCreation,
    /// A synchronized block
    SynchronizedBlock,
    /// An assert statement
    AssertStatement,

    /// Unknown node type
    Unknown,
}

/// Types of edges between nodes for Java
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

    // Java-specific edge types
    /// Interface implementation (for a class)
    ImplementsInterface,
    /// Package import relationship
    ImportsPackage,
    /// Annotation application
    Annotates,
    /// Generic type parameter binding
    TypeParameterBinds,
    /// Exception throwing
    Throws,
    /// Exception catching
    Catches,
    /// Method overriding
    Overrides,
    /// Field access
    Accesses,
    /// Type casting
    Casts,
    /// Instance creation
    Instantiates,
    /// Static member access
    StaticAccess,
    /// Synchronized on object
    Synchronizes,
    /// Lambda captures variable
    Captures,
    /// Containment relationship
    Contains,
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

    /// Create a span from tree-sitter node
    pub fn from_node(node: &tree_sitter::Node) -> Self {
        let start_pos = node.start_position();
        let end_pos = node.end_position();

        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_line: start_pos.row + 1, // tree-sitter uses 0-indexed
            end_line: end_pos.row + 1,
            start_column: start_pos.column + 1,
            end_column: end_pos.column + 1,
        }
    }
}

/// Programming language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Java
    Java,
    /// For compatibility with other parsers
    Python,
    Rust,
}

/// A node in the Universal AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Node type
    pub kind: NodeKind,
    /// Node name (e.g., class name, method name)
    pub name: String,
    /// Programming language
    pub lang: Language,
    /// Source file path
    pub file: PathBuf,
    /// Source location
    pub span: Span,
    /// Optional type signature
    pub signature: Option<String>,
    /// Additional metadata (Java-specific info like visibility, modifiers, etc.)
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

    /// Set metadata for the node
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set signature for the node
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
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
} 