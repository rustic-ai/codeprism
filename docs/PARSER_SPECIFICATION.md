# CodePrism Parser Specification

**Version:** 1.0  
**Author:** CodePrism Development Team  
**Date:** 2025-06-25  
**Status:** Final

## Table of Contents

- [Overview](#overview)
- [Core Parser Interface](#core-parser-interface)
- [Universal AST Specification](#universal-ast-specification)
- [Integration Patterns](#integration-patterns)
- [Implementation Guidelines](#implementation-guidelines)
- [Performance Requirements](#performance-requirements)
- [Testing Requirements](#testing-requirements)
- [Code Examples](#code-examples)
- [Template Repository Structure](#template-repository-structure)

## Overview

This document defines the standardized parser interface and implementation requirements for CodePrism language parsers. All language parsers must conform to this specification to ensure consistent integration with the CodePrism system.

### Purpose

- **Standardization**: Establish consistent parser interfaces across all languages
- **Quality Assurance**: Define performance, testing, and reliability requirements  
- **Scalability**: Enable efficient parallel processing and incremental updates
- **Maintainability**: Provide clear implementation guidelines and patterns

### Scope

This specification covers:
- Core parser structure and contracts
- Universal AST node and edge type specifications
- MCP server integration patterns
- Performance benchmarks and optimization guidelines
- Comprehensive testing patterns and requirements

## Core Parser Interface

All language parsers are expected to be self-contained crates that expose a parser struct and a set of shared data types for the Universal AST. The primary interaction point is an adapter that prepares the parser for use within the CodePrism system.

### Language-Specific Parser

Each language crate should define its own parser struct. This struct is responsible for holding the tree-sitter parser state and handling the core parsing logic.

```rust
// Example from codeprism-lang-python/src/parser.rs

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

    /// Get the language for a file based on its extension
    pub fn detect_language(path: &Path) -> Language {
        match path.extension().and_then(|s| s.to_str()) {
            Some("py") | Some("pyw") => Language::Python,
            _ => Language::Python, // Default to Python
        }
    }
}
```

**Implementation Requirements:**
- **State Management**: The struct should manage the `tree-sitter::Parser` instance.
- **Error Handling**: Graceful degradation for malformed code.
- **Incremental Support**: Utilize tree-sitter's incremental parsing capabilities by accepting an old tree in the `ParseContext`.

### ParseContext Structure

The `ParseContext` provides all necessary information for parsing:

```rust
/// Parser context for incremental parsing
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Repository ID for node identification
    pub repo_id: String,
    /// File path being parsed
    pub file_path: PathBuf,
    /// Previous tree for incremental parsing (optional)
    pub old_tree: Option<Tree>,
    /// File content as UTF-8 string
    pub content: String,
}
```

**Usage Guidelines:**
- **Repository ID**: Must be consistent across all files in a repository
- **File Path**: Should be relative to repository root when possible
- **Old Tree**: Always provide when available for performance optimization
- **Content Validation**: Ensure UTF-8 encoding before parsing
- **Direct Construction**: Create instances directly using struct literal syntax:

```rust
// Example usage - direct struct construction
let context = ParseContext {
    repo_id: "test_repo".to_string(),
    file_path: PathBuf::from("test.py"),
    old_tree: None,
    content: "def hello():\n    return 'world'".to_string(),
};

// For incremental parsing with old tree
let context_with_tree = ParseContext {
    repo_id: "test_repo".to_string(),
    file_path: PathBuf::from("test.py"),
    old_tree: Some(previous_tree),
    content: "def hello():\n    return 'updated'".to_string(),
};
```

### ParseResult Format

The `ParseResult` contains all extracted information:

```rust
/// Result of parsing a file
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed tree-sitter syntax tree
    pub tree: Tree,
    /// Extracted Universal AST nodes
    pub nodes: Vec<Node>,
    /// Extracted relationships between nodes
    pub edges: Vec<Edge>,
}
```

**Expectations:**
- **Tree Preservation**: Original tree-sitter tree for incremental updates
- **Complete Extraction**: All significant language constructs represented
- **Consistent Naming**: Follow language-specific naming conventions
- **Relationship Accuracy**: Edges must represent actual code relationships

## Universal AST Specification

### NodeKind Enum

The Universal AST supports the following node types:

```rust
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
    /// A method definition (function within a class)
    Method,
    /// A function/method parameter
    Parameter,
    /// A variable declaration
    Variable,
    /// A function/method call
    Call,
    /// An import statement
    Import,
    /// A literal value (string, number, boolean)
    Literal,
    /// An HTTP route definition
    Route,
    /// A SQL query
    SqlQuery,
    /// An event emission
    Event,
    /// Unknown node type (fallback)
    Unknown,
}
```

**Node Type Guidelines:**

#### Module
- **Purpose**: Represents a file, namespace, or module
- **Examples**: Python files, JavaScript modules, Java packages
- **Naming**: Use filename without extension or module name
- **Metadata**: Include package/namespace information

#### Class  
- **Purpose**: Object-oriented class definitions
- **Examples**: `class MyClass`, `struct MyStruct`, `interface MyInterface`
- **Naming**: Use the class identifier
- **Metadata**: Include inheritance, generics, modifiers

#### Function
- **Purpose**: Standalone function definitions
- **Examples**: `def my_func()`, `function myFunc()`, `fn my_func()`
- **Naming**: Use the function identifier
- **Metadata**: Include parameters, return type, decorators

#### Method
- **Purpose**: Functions defined within classes
- **Examples**: Class methods, instance methods, static methods
- **Naming**: Use the method identifier
- **Metadata**: Include visibility, static/instance, overrides

#### Parameter
- **Purpose**: Function/method parameters
- **Examples**: `(name: str)`, `(int value)`, `(name)`
- **Naming**: Use parameter name
- **Metadata**: Include type information, default values

#### Variable
- **Purpose**: Variable declarations and definitions
- **Examples**: `let x = 5`, `int count;`, `name = "value"`
- **Naming**: Use variable identifier
- **Metadata**: Include type, scope, mutability

#### Call
- **Purpose**: Function/method invocations
- **Examples**: `my_func()`, `obj.method()`, `func(args)`
- **Naming**: Use called function/method name
- **Metadata**: Include arguments, receiver type

#### Import
- **Purpose**: Module import statements
- **Examples**: `import os`, `from x import y`, `require('module')`
- **Naming**: Use imported module/symbol name
- **Metadata**: Include source module, import type

#### Literal
- **Purpose**: Constant literal values
- **Examples**: `"string"`, `42`, `true`, `null`
- **Naming**: Use literal value (truncated if long)
- **Metadata**: Include literal type, actual value

### EdgeKind Enum

Relationships between nodes are represented by edges:

```rust
/// Types of edges between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeKind {
    /// Function/method call relationship
    Calls,
    /// Variable/field read access
    Reads,
    /// Variable/field write access  
    Writes,
    /// Module import relationship
    Imports,
    /// Event emission
    Emits,
    /// HTTP route mapping
    RoutesTo,
    /// Exception raising
    Raises,
    /// Type inheritance (extends)
    Extends,
    /// Interface implementation
    Implements,
}
```

**Edge Type Guidelines:**

#### CALLS
- **Source**: Call node
- **Target**: Function/Method node
- **Purpose**: Represents function/method invocations
- **Examples**: `foo() -> foo`, `obj.method() -> method`

#### READS  
- **Source**: Expression/Call node
- **Target**: Variable node
- **Purpose**: Variable access without modification
- **Examples**: `print(x) -> x`, `return value -> value`

#### WRITES
- **Source**: Assignment/Declaration node  
- **Target**: Variable node
- **Purpose**: Variable assignment or modification
- **Examples**: `x = 5 -> x`, `self.field = value -> field`

#### IMPORTS
- **Source**: Import node
- **Target**: Module node  
- **Purpose**: Module dependency relationships
- **Examples**: `import math -> math`, `from os import path -> path`

#### EXTENDS
- **Source**: Class node
- **Target**: Base class node
- **Purpose**: Class inheritance relationships
- **Examples**: `class Child(Parent) -> Parent`

#### IMPLEMENTS  
- **Source**: Class node
- **Target**: Interface node
- **Purpose**: Interface implementation relationships
- **Examples**: `class MyClass implements MyInterface -> MyInterface`

### Span and Location Tracking

All nodes must include precise source location information:

```rust
/// Source code location
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Starting byte offset (0-indexed)
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
```

**Location Requirements:**
- **Byte Accuracy**: Must match tree-sitter node boundaries exactly
- **Line/Column Calculation**: Must account for multi-byte UTF-8 characters
- **Consistency**: Spans must be non-overlapping for sibling nodes
- **Completeness**: All significant nodes must have accurate spans

### Node Structure

The Universal AST node structure:

```rust
/// A node in the Universal AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier (generated from content)
    pub id: NodeId,
    /// Node type from NodeKind enum
    pub kind: NodeKind,
    /// Human-readable node name
    pub name: String,
    /// Programming language
    pub lang: Language,
    /// Source file path
    pub file: PathBuf,
    /// Source location
    pub span: Span,
    /// Optional type signature
    pub signature: Option<String>,
    /// Additional language-specific metadata
    pub metadata: serde_json::Value,
}
```

**Node Creation Guidelines:**
- **Unique IDs**: Generated deterministically from repo_id, file, span, and kind
- **Meaningful Names**: Use actual identifiers from source code
- **Type Signatures**: Include when available (function signatures, variable types)
- **Rich Metadata**: Store language-specific information for advanced analysis

## Integration Patterns

### MCP Server Integration

Language parsers are not integrated directly. Instead, each parser crate provides an **Adapter** that prepares it for consumption by the MCP server. The calling environment (the MCP server) is then responsible for converting the parser's output into its internal types.

This is achieved through an adapter struct in the parser crate and a `ParseResultConverter` trait that the caller implements.

#### Parser Crate Adapter

The parser crate should expose an adapter struct and a top-level `parse_file` function.

```rust
// Example from codeprism-lang-python/src/adapter.rs

/// Adapter that prepares the Python parser for use in CodePrism
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

/// Parse a file and return the result in the parser's internal types
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
```

#### Caller-Side Conversion

The consumer of the parser (e.g., the MCP server) must implement a `ParseResultConverter` trait to transform the parser's output into the application's native U-AST types.

```rust
// Example from codeprism-lang-python/src/adapter.rs

// This trait is defined in the parser crate and implemented by the caller.
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
```

**Integration Requirements:**
- **Decoupling**: The parser crate must not depend on core application crates (like `codeprism-core`).
- **Thread Safety**: The adapter must handle concurrent parsing requests, typically using a `Mutex`.
- **Type Conversion**: The caller is responsible for converting the parser-specific `Node` and `Edge` types into its own types.
- **Error Propagation**: Convert parser errors to a format understood by the calling application.

### Error Handling Patterns

Parsers must implement comprehensive error handling. It's recommended to use a dedicated error enum with `thiserror`.

```rust
/// Parser-specific error types (from actual Python parser implementation)
#[derive(Error, Debug)]
pub enum Error {
    /// Parse error
    #[error("Failed to parse {file}: {message}")]
    ParseError { file: PathBuf, message: String },

    /// Tree-sitter error
    #[error("Tree-sitter error in {file}: {message}")]
    TreeSitterError { file: PathBuf, message: String },

    /// AST mapping error
    #[error("AST mapping error in {file}: {message}")]
    AstMappingError { file: PathBuf, message: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error
    #[error("Python parser error: {0}")]
    Generic(String),
}

impl Error {
    /// Create a parse error
    pub fn parse(file: &std::path::Path, message: &str) -> Self {
        Self::ParseError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }

    /// Create a tree-sitter error
    pub fn tree_sitter(file: &std::path::Path, message: &str) -> Self {
        Self::TreeSitterError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }

    /// Create an AST mapping error
    pub fn ast_mapping(file: &std::path::Path, message: &str) -> Self {
        Self::AstMappingError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }

    /// Create a generic error
    pub fn generic(message: &str) -> Self {
        Self::Generic(message.to_string())
    }
}

/// Result type for Python parser
pub type Result<T> = std::result::Result<T, Error>;
```

**Error Handling Guidelines:**
- **Graceful Degradation**: Continue processing when encountering errors
- **Detailed Context**: Include file location and error description
- **Error Recovery**: Handle malformed syntax without crashing
- **Logging Integration**: Use structured logging for debugging

### Performance Requirements

Parsers must meet specific performance benchmarks:

| Metric | Requirement | Target |
|--------|-------------|---------|
| **Parse Speed** | < 10µs per line of code | < 5µs per line |
| **Memory Usage** | < 500 bytes per node | < 200 bytes per node |
| **Incremental Update** | < 100ms for typical edit | < 10ms for typical edit |
| **Throughput** | > 1MB/s source code | > 5MB/s source code |
| **Error Rate** | < 0.1% parse failures | < 0.01% parse failures |

**Performance Optimization Techniques:**

```rust
/// Efficient AST extraction patterns
impl AstMapper {
    /// Pre-allocate collections based on estimated size
    fn with_capacity_estimate(estimated_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(estimated_nodes),
            edges: Vec::with_capacity(estimated_nodes * 2),
            node_map: HashMap::with_capacity(estimated_nodes),
            // ...
        }
    }
    
    /// Use string slices to avoid unnecessary allocations
    fn extract_node_name<'a>(&self, node: &tree_sitter::Node<'a>, source: &'a str) -> &'a str {
        &source[node.start_byte()..node.end_byte()]
    }
    
    /// Batch edge creation for efficiency
    fn create_edges_batch(&mut self, relationships: Vec<(NodeId, NodeId, EdgeKind)>) {
        self.edges.extend(
            relationships.into_iter().map(|(src, tgt, kind)| Edge::new(src, tgt, kind))
        );
    }
}
```

## Implementation Guidelines

### Tree-Sitter Integration

All parsers must use tree-sitter for syntax parsing:

```rust
/// Standard tree-sitter integration pattern (from actual implementation)
pub struct PythonParser {
    parser: tree_sitter::Parser,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");
        
        Self { parser }
    }
    
    fn parse_with_tree_sitter(&mut self, context: &ParseContext) -> Result<tree_sitter::Tree> {
        self.parser
            .parse(&context.content, context.old_tree.as_ref())
            .ok_or_else(|| Error::parse(&context.file_path, "Failed to parse file"))
    }
}
```

**Tree-Sitter Requirements:**
- **Grammar Compatibility**: Use stable tree-sitter grammar versions
- **Error Handling**: Handle tree-sitter parsing failures gracefully
- **Incremental Support**: Always use old_tree when available
- **Memory Management**: Properly manage tree-sitter memory allocation

### Memory Usage Guidelines

Efficient memory management is critical for large repositories:

**Best Practices:**
1. **Minimize String Allocations**: Use string slices where possible
2. **Pre-allocate Collections**: Estimate collection sizes to avoid resizing
3. **Efficient Data Structures**: Use appropriate hash maps and vectors
4. **Memory Pooling**: Reuse objects for repeated parsing operations

```rust
/// Memory-efficient implementation patterns
impl AstMapper {
    /// Use Cow for potentially borrowed strings
    fn extract_name(&self, node: &tree_sitter::Node) -> Cow<str> {
        if let Ok(text) = node.utf8_text(self.source.as_bytes()) {
            Cow::Borrowed(text)
        } else {
            Cow::Owned(format!("invalid_utf8_{}", node.id()))
        }
    }
    
    /// Efficient span calculation
    fn create_span(&self, node: &tree_sitter::Node) -> Span {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        Span {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_line: start_pos.row + 1,
            end_line: end_pos.row + 1,
            start_column: start_pos.column + 1,
            end_column: end_pos.column + 1,
        }
    }
}
```

### Testing Requirements

Comprehensive testing is mandatory for all parsers:

#### Unit Tests
- **Node Extraction**: Test all supported language constructs
- **Edge Creation**: Verify relationship accuracy
- **Error Handling**: Test malformed code scenarios
- **Performance**: Benchmark parsing speed and memory usage

#### Integration Tests
- **Real Files**: Test on actual project files
- **Incremental Updates**: Verify incremental parsing correctness
- **Large Files**: Test performance on large source files
- **Edge Cases**: Handle unusual syntax and error conditions

#### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_extraction() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "def hello():\n    return 'world'".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

        // Should have at least a module node and a function node
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Module)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Function)));
    }
    
    #[test]
    fn test_class_parsing() {
        let mut parser = PythonParser::new();
        let context = ParseContext {
            repo_id: "test_repo".to_string(),
            file_path: PathBuf::from("test.py"),
            old_tree: None,
            content: "class MyClass:\n    def method(self):\n        pass".to_string(),
        };

        let result = parser.parse(&context).unwrap();
        assert!(!result.nodes.is_empty());

        // Should have module, class, and method nodes
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Module)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Class)));
        assert!(result
            .nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Method)));
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
    }
    
    #[test]
    fn test_import_parsing() {
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
            .filter(|n| matches!(n.kind, NodeKind::Import))
            .collect();

        // Should have at least one import node
        assert!(!import_nodes.is_empty());
    }
}
```

#### Performance Tests

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_parse_large_file(c: &mut Criterion) {
        let content = include_str!("../tests/fixtures/large.py");
        let mut parser = PythonParser::new();
        
        c.bench_function("parse_large_python", |b| {
            b.iter(|| {
                let context = ParseContext {
                    repo_id: "bench".to_string(),
                    file_path: PathBuf::from("large.py"),
                    old_tree: None,
                    content: black_box(content.to_string()),
                };
                parser.parse(&context).unwrap()
            })
        });
    }
    
    criterion_group!(benches, bench_parse_large_file);
    criterion_main!(benches);
}
```

## Code Examples

### Complete Parser Implementation

Here's a complete example parser implementation:

```rust
// src/lib.rs (actual implementation)
mod adapter;
mod analysis;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, ParseResultConverter, PythonLanguageParser};
pub use analysis::PythonAnalyzer;
pub use error::{Error, Result};
pub use parser::{ParseContext, ParseResult, PythonParser};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> PythonLanguageParser {
    PythonLanguageParser::new()
}
```

```rust
// src/parser.rs (actual implementation)
use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");
        
        Self { parser }
    }
    
    pub fn detect_language(path: &Path) -> Language {
        match path.extension().and_then(|s| s.to_str()) {
            Some("py") | Some("pyw") => Language::Python,
            _ => Language::Python, // Default to Python
        }
    }
    
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
```

```rust
// src/ast_mapper.rs
use crate::error::Result;
use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Tree, TreeCursor};

pub struct AstMapper {
    repo_id: String,
    file_path: PathBuf,
    language: Language,
    source: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_map: HashMap<usize, NodeId>,
}

impl AstMapper {
    pub fn new(repo_id: &str, file_path: PathBuf, language: Language, source: &str) -> Self {
        Self {
            repo_id: repo_id.to_string(),
            file_path,
            language,
            source: source.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
        }
    }
    
    pub fn extract(mut self, tree: &Tree) -> Result<(Vec<Node>, Vec<Edge>)> {
        let mut cursor = tree.walk();
        self.visit_node(&cursor)?;
        
        Ok((self.nodes, self.edges))
    }
    
    fn visit_node(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        
        match node.kind() {
            "function_declaration" => self.handle_function(cursor)?,
            "class_declaration" => self.handle_class(cursor)?,
            "call_expression" => self.handle_call(cursor)?,
            // Add more node type handlers...
            _ => {
                // Process child nodes
                if cursor.goto_first_child() {
                    loop {
                        self.visit_node(cursor)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_function(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = self.create_span(&node);
        
        // Extract function name
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            name_node.utf8_text(self.source.as_bytes())
                .unwrap_or("unnamed_function")
                .to_string()
        } else {
            "anonymous_function".to_string()
        };
        
        let func_node = Node::new(
            &self.repo_id,
            NodeKind::Function,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );
        
        self.node_map.insert(node.id(), func_node.id);
        self.nodes.push(func_node);
        
        Ok(())
    }
    
    // Add more handler methods for different node types...
}
```

## Template Repository Structure

A complete language parser crate should follow this structure:

```
codeprism-lang-mylang/
├── Cargo.toml                 # Dependencies and metadata
├── build.rs                   # Build script for grammar compilation
├── README.md                  # Parser documentation
├── src/
│   ├── lib.rs                # Public API and exports
│   ├── parser.rs             # Main parser implementation
│   ├── ast_mapper.rs         # Tree-sitter to U-AST mapping
│   ├── adapter.rs            # MCP integration adapter
│   ├── types.rs              # Type definitions and re-exports
│   └── error.rs              # Error handling
├── tests/
│   ├── fixtures/             # Test source files
│   │   ├── simple.mylang    # Basic language features
│   │   ├── complex.mylang   # Advanced features
│   │   ├── edge_cases.mylang # Error conditions
│   │   └── large.mylang     # Performance testing
│   ├── integration_test.rs   # Integration tests
│   ├── unit_tests.rs         # Unit tests
│   └── regression_tests.rs   # Regression tests
├── benches/
│   └── parse_benchmark.rs    # Performance benchmarks
└── examples/
    └── parse_example.rs       # Usage examples
```

### Cargo.toml Template

```toml
[package]
name = "codeprism-lang-python"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Python language support for codeprism"

[dependencies]
# Core dependencies
thiserror.workspace = true
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true

# Tree-sitter
tree-sitter.workspace = true
tree-sitter-python.workspace = true

# CodePrism integration
blake3.workspace = true
hex.workspace = true

[dev-dependencies]
# No dev dependencies in actual implementation

[build-dependencies]
cc = "1.0"

[[bench]]
name = "parsing_benchmark"
harness = false
```

---

## Conclusion

This specification provides a comprehensive blueprint for implementing CodePrism language parsers. By following these guidelines, parser implementations will be:

- **Consistent**: Uniform interfaces and behavior across languages
- **Performant**: Meeting strict performance requirements for large repositories
- **Reliable**: Comprehensive testing and error handling
- **Maintainable**: Clear structure and documented patterns
- **Extensible**: Ready for future enhancements and optimizations

All parser implementations must pass the acceptance criteria defined in this specification before integration into the CodePrism system.

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-06-25 | Initial specification release |
| 1.1 | 2025-01-27 | **MAJOR CORRECTIONS** - Updated all code examples to match actual Python parser implementation. Removed non-existent LanguageParser trait, fixed ParseContext usage, corrected error types, and updated integration patterns to reflect actual codebase structure. |

### References

- [Language Parser Implementation Guide](LANGUAGE_PARSERS.md)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [CodePrism Architecture Documentation](ARCHITECTURE.md)
- [Performance Benchmarking Guide](DEVELOPER.md#performance-profiling) 