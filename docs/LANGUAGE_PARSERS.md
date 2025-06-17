# Language Parser Implementation Guide

This guide explains how to implement language parsers for Prism, using the JavaScript/TypeScript parser as a reference implementation.

## Table of Contents

- [Overview](#overview)
- [Parser Architecture](#parser-architecture)
- [Implementation Steps](#implementation-steps)
- [Tree-Sitter Integration](#tree-sitter-integration)
- [AST Mapping](#ast-mapping)
- [Testing Strategy](#testing-strategy)
- [Performance Optimization](#performance-optimization)
- [Best Practices](#best-practices)

## Overview

Language parsers in Prism convert language-specific syntax trees (CST) from Tree-Sitter into the Universal AST representation. Each parser is implemented as a separate crate following a consistent pattern.

### Key Responsibilities

1. **Parse Source Code**: Use Tree-Sitter to generate CST
2. **Extract Nodes**: Convert CST nodes to Universal AST nodes
3. **Extract Edges**: Identify relationships between nodes
4. **Handle Incremental Updates**: Support efficient re-parsing
5. **Provide Integration**: Adapt to the core parser engine

### Supported Languages

| Language | Status | Crate | Tree-Sitter Grammar |
|----------|--------|-------|-------------------|
| JavaScript/TypeScript | ✅ Complete | `gcore-lang-js` | `tree-sitter-javascript`, `tree-sitter-typescript` |
| Python | ✅ Complete | `gcore-lang-python` | `tree-sitter-python` |
| Rust | 🚧 Next Priority | `gcore-lang-rust` | `tree-sitter-rust` |
| Java | 🚧 Planned | `gcore-lang-java` | `tree-sitter-java` |
| Go | 📋 Future | `gcore-lang-go` | `tree-sitter-go` |

## Parser Architecture

### Crate Structure

```
gcore-lang-{language}/
├── Cargo.toml              # Dependencies and metadata
├── build.rs                # Build script for grammar compilation
├── src/
│   ├── lib.rs             # Public API and exports
│   ├── parser.rs          # Main parser implementation
│   ├── ast_mapper.rs      # CST to U-AST conversion
│   ├── adapter.rs         # Integration with gcore
│   ├── types.rs           # Language-specific types
│   └── error.rs           # Error handling
├── tests/
│   ├── fixtures/          # Test source files
│   │   ├── simple.{ext}   # Basic language features
│   │   ├── complex.{ext}  # Advanced features
│   │   └── edge_cases.{ext} # Error conditions
│   └── integration_test.rs # Integration tests
└── benches/               # Performance benchmarks
    └── parse_benchmark.rs
```

### Component Responsibilities

#### `parser.rs` - Main Parser

```rust
pub struct LanguageParser {
    parser: Parser,  // Tree-sitter parser
}

impl LanguageParser {
    pub fn new() -> Self
    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult>
    pub fn detect_language(path: &Path) -> Language
}
```

#### `ast_mapper.rs` - AST Conversion

```rust
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
    pub fn new(...) -> Self
    pub fn extract(self, tree: &Tree) -> Result<(Vec<Node>, Vec<Edge>)>
    fn visit_node(&mut self, cursor: &TreeCursor) -> Result<()>
}
```

#### `adapter.rs` - Integration Layer

```rust
pub struct LanguageParserAdapter {
    parser: Mutex<LanguageParser>,
}

impl LanguageParser for LanguageParserAdapter {
    fn parse_file(&self, context: ParseContext) -> Result<ParseResult>
    fn supported_extensions(&self) -> &[&str]
    fn language_name(&self) -> &str
}
```

## Implementation Steps

### Step 1: Create Crate Structure

```bash
# Create new crate
mkdir crates/gcore-lang-{language}
cd crates/gcore-lang-{language}

# Initialize Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "gcore-lang-{language}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "{Language} language support for gcore"

[dependencies]
# Core dependencies
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
serde_json.workspace = true

# Tree-sitter
tree-sitter.workspace = true
tree-sitter-{language} = "x.y.z"  # Check latest version

# Import gcore types without circular dependency
blake3.workspace = true
hex.workspace = true

[dev-dependencies]
insta.workspace = true
tempfile.workspace = true
tokio = { workspace = true, features = ["test-util"] }

[build-dependencies]
cc = "1.0"
EOF
```

### Step 2: Implement Error Types

```rust
// src/error.rs
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error in {file}: {message}")]
    Parse {
        file: PathBuf,
        message: String,
    },

    #[error("Failed to set language: {0}")]
    Language(String),

    #[error("Failed to extract node at {file}:{line}:{column}: {message}")]
    NodeExtraction {
        file: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse {
            file: file.into(),
            message: message.into(),
        }
    }

    pub fn node_extraction(
        file: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::NodeExtraction {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }
}
```

### Step 3: Define Language-Specific Types

```rust
// src/types.rs
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Mirror gcore types to avoid circular dependencies
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId([u8; 16]);

impl NodeId {
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

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

// Language-specific enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    {LanguageName},
    // Add variants for language dialects if needed
}

// Re-export common types
pub use gcore::ast::{NodeKind, EdgeKind, Span, Node, Edge};
```

### Step 4: Implement Main Parser

```rust
// src/parser.rs
use crate::ast_mapper::AstMapper;
use crate::error::{Error, Result};
use crate::types::{Edge, Language, Node};
use std::path::PathBuf;
use tree_sitter::{Parser, Tree};

#[derive(Debug, Clone)]
pub struct ParseContext {
    pub repo_id: String,
    pub file_path: PathBuf,
    pub old_tree: Option<Tree>,
    pub content: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub tree: Tree,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub struct LanguageParser {
    parser: Parser,
}

impl LanguageParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_{language}::language())
            .expect("Failed to load {language} grammar");

        Self { parser }
    }

    pub fn detect_language(path: &PathBuf) -> Language {
        match path.extension().and_then(|s| s.to_str()) {
            Some("{ext1}") | Some("{ext2}") => Language::{LanguageName},
            _ => Language::{LanguageName}, // Default
        }
    }

    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
        let language = Self::detect_language(&context.file_path);
        
        // Parse the file
        let tree = self.parser
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

impl Default for LanguageParser {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 5: Implement AST Mapper

```rust
// src/ast_mapper.rs
use crate::error::Result;
use crate::types::{Edge, EdgeKind, Language, Node, NodeKind, Span};
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
    node_map: HashMap<usize, crate::types::NodeId>,
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
        
        // Create module node for the file
        let module_node = self.create_module_node(&cursor)?;
        self.nodes.push(module_node);
        
        // Walk the tree and extract nodes
        self.walk_tree(&mut cursor)?;
        
        Ok((self.nodes, self.edges))
    }

    fn walk_tree(&mut self, cursor: &mut TreeCursor) -> Result<()> {
        self.visit_node(cursor)?;
        
        if cursor.goto_first_child() {
            loop {
                self.walk_tree(cursor)?;
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        
        Ok(())
    }

    fn visit_node(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let kind = node.kind();
        
        match kind {
            // Map language-specific node types to Universal AST
            "function_declaration" | "function_definition" => {
                self.handle_function(cursor)?;
            }
            "class_declaration" | "class_definition" => {
                self.handle_class(cursor)?;
            }
            "variable_declaration" | "assignment" => {
                self.handle_variable(cursor)?;
            }
            "call_expression" | "function_call" => {
                self.handle_call(cursor)?;
            }
            "import_statement" | "import_declaration" => {
                self.handle_import(cursor)?;
            }
            _ => {
                // Skip unknown node types
            }
        }
        
        Ok(())
    }

    // Implement handler methods for each node type
    fn handle_function(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);
        
        // Extract function name
        let name = self.extract_function_name(&node)?;
        
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

    // Add more handler methods...
}
```

### Step 6: Create Integration Adapter

```rust
// src/adapter.rs
use crate::parser::{LanguageParser, ParseContext as LangParseContext};
use crate::types as lang_types;

pub struct LanguageParserAdapter {
    parser: std::sync::Mutex<LanguageParser>,
}

impl LanguageParserAdapter {
    pub fn new() -> Self {
        Self {
            parser: std::sync::Mutex::new(LanguageParser::new()),
        }
    }
}

impl Default for LanguageParserAdapter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_file(
    parser: &LanguageParserAdapter,
    repo_id: &str,
    file_path: std::path::PathBuf,
    content: String,
    old_tree: Option<tree_sitter::Tree>,
) -> Result<(tree_sitter::Tree, Vec<lang_types::Node>, Vec<lang_types::Edge>), crate::error::Error> {
    let context = LangParseContext {
        repo_id: repo_id.to_string(),
        file_path,
        old_tree,
        content,
    };
    
    let mut parser = parser.parser.lock().unwrap();
    let result = parser.parse(&context)?;
    
    Ok((result.tree, result.nodes, result.edges))
}

pub fn create_parser() -> LanguageParserAdapter {
    LanguageParserAdapter::new()
}
```

### Step 7: Set Up Public API

```rust
// src/lib.rs
//! {Language} language support for gcore

mod adapter;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{LanguageParserAdapter, parse_file, create_parser};
pub use error::{Error, Result};
pub use parser::{LanguageParser, ParseContext, ParseResult};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
```

## Tree-Sitter Integration

### Understanding Tree-Sitter Grammars

Tree-Sitter generates concrete syntax trees (CST) that include all syntactic details. You need to understand the grammar structure for your target language.

#### Exploring Grammar Structure

```rust
#[cfg(test)]
fn explore_grammar() {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_{language}::language()).unwrap();
    
    let source_code = r#"
        // Your test code here
    "#;
    
    let tree = parser.parse(source_code, None).unwrap();
    print_tree(&tree, source_code);
}

fn print_tree(tree: &Tree, source: &str) {
    let mut cursor = tree.walk();
    print_node(&mut cursor, source, 0);
}

fn print_node(cursor: &mut TreeCursor, source: &str, depth: usize) {
    let node = cursor.node();
    let indent = "  ".repeat(depth);
    
    println!("{}{}[{}..{}] {:?}", 
        indent, 
        node.kind(), 
        node.start_byte(), 
        node.end_byte(),
        node.utf8_text(source.as_bytes()).unwrap_or("<error>")
    );
    
    if cursor.goto_first_child() {
        loop {
            print_node(cursor, source, depth + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
```

### Common Tree-Sitter Patterns

#### Field Access

```rust
// Get named fields from nodes
if let Some(name_node) = node.child_by_field_name("name") {
    let name = name_node.utf8_text(source.as_bytes())?;
}

if let Some(body_node) = node.child_by_field_name("body") {
    // Process function body
}
```

#### Child Iteration

```rust
// Iterate through all children
for i in 0..node.child_count() {
    if let Some(child) = node.child(i) {
        match child.kind() {
            "parameter" => handle_parameter(&child),
            "statement" => handle_statement(&child),
            _ => {}
        }
    }
}
```

#### Named vs Anonymous Nodes

```rust
// Only process named nodes (skip punctuation)
if node.is_named() {
    process_node(&node);
}
```

## AST Mapping

### Node Type Mapping

Create a mapping from language-specific node types to Universal AST types:

```rust
fn map_node_kind(ts_kind: &str) -> Option<NodeKind> {
    match ts_kind {
        // Functions
        "function_declaration" | "function_definition" | "def" => Some(NodeKind::Function),
        "method_declaration" | "method_definition" => Some(NodeKind::Method),
        
        // Classes
        "class_declaration" | "class_definition" => Some(NodeKind::Class),
        
        // Variables
        "variable_declaration" | "assignment" | "let_declaration" => Some(NodeKind::Variable),
        
        // Calls
        "call_expression" | "function_call" | "method_call" => Some(NodeKind::Call),
        
        // Imports
        "import_statement" | "import_declaration" | "from_import" => Some(NodeKind::Import),
        
        // Modules
        "module" | "program" | "source_file" => Some(NodeKind::Module),
        
        _ => None,
    }
}
```

### Edge Extraction

Identify relationships between nodes:

```rust
fn extract_edges(&mut self, node: &tree_sitter::Node) -> Result<()> {
    match node.kind() {
        "call_expression" => {
            // Function call: caller CALLS callee
            if let Some(caller_id) = self.find_containing_function(node) {
                if let Some(callee_id) = self.extract_call_target(node) {
                    self.edges.push(Edge::new(caller_id, callee_id, EdgeKind::Calls));
                }
            }
        }
        
        "assignment" => {
            // Variable assignment: function WRITES variable
            if let Some(writer_id) = self.find_containing_function(node) {
                if let Some(var_id) = self.extract_assignment_target(node) {
                    self.edges.push(Edge::new(writer_id, var_id, EdgeKind::Writes));
                }
            }
        }
        
        "import_statement" => {
            // Module import: module IMPORTS dependency
            if let Some(module_id) = self.find_module_node() {
                if let Some(dep_id) = self.extract_import_target(node) {
                    self.edges.push(Edge::new(module_id, dep_id, EdgeKind::Imports));
                }
            }
        }
        
        _ => {}
    }
    
    Ok(())
}
```

### Handling Language-Specific Features

#### Python Example

```rust
// Python-specific node handling
match node.kind() {
    "function_definition" => {
        // Handle def statements
        let name = self.extract_function_name(node)?;
        let decorators = self.extract_decorators(node);
        // ...
    }
    
    "class_definition" => {
        // Handle class statements with inheritance
        let name = self.extract_class_name(node)?;
        let bases = self.extract_base_classes(node);
        // ...
    }
    
    "import_statement" => {
        // Handle: import module
        let module = self.extract_import_module(node)?;
        // ...
    }
    
    "import_from_statement" => {
        // Handle: from module import name
        let module = self.extract_from_module(node)?;
        let names = self.extract_import_names(node)?;
        // ...
    }
    
    _ => {}
}
```

#### Java Example

```rust
// Java-specific node handling
match node.kind() {
    "method_declaration" => {
        let name = self.extract_method_name(node)?;
        let modifiers = self.extract_modifiers(node);
        let return_type = self.extract_return_type(node);
        // ...
    }
    
    "class_declaration" => {
        let name = self.extract_class_name(node)?;
        let extends = self.extract_extends_clause(node);
        let implements = self.extract_implements_clause(node);
        // ...
    }
    
    "interface_declaration" => {
        let name = self.extract_interface_name(node)?;
        let extends = self.extract_interface_extends(node);
        // ...
    }
    
    _ => {}
}
```

## Testing Strategy

### Test Structure

```rust
// tests/integration_test.rs
use gcore_lang_{language}::{LanguageParser, ParseContext};
use std::fs;
use std::path::PathBuf;

fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_parse_simple_function() {
    let mut parser = LanguageParser::new();
    let file_path = get_fixture_path("simple.{ext}");
    let content = fs::read_to_string(&file_path).unwrap();
    
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: file_path.clone(),
        old_tree: None,
        content,
    };
    
    let result = parser.parse(&context).unwrap();
    
    // Verify expected nodes
    assert!(result.nodes.iter().any(|n| n.name == "expected_function"));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Function)));
    
    // Verify edges
    assert!(!result.edges.is_empty());
}
```

### Test Fixtures

Create comprehensive test files covering:

1. **Basic Features**: Functions, classes, variables
2. **Advanced Features**: Generics, async/await, decorators
3. **Edge Cases**: Syntax errors, incomplete code, Unicode
4. **Real-World Examples**: Framework code, libraries

```javascript
// tests/fixtures/comprehensive.js
// Basic function
function greet(name) {
    console.log(`Hello, ${name}!`);
}

// Class with methods
class User {
    constructor(name, email) {
        this.name = name;
        this.email = email;
    }
    
    async save() {
        await database.save(this);
    }
    
    static findById(id) {
        return database.findById(id);
    }
}

// Arrow functions
const multiply = (a, b) => a * b;

// Imports
import { Component } from 'react';
import * as utils from './utils';

// Exports
export { User };
export default greet;
```

### Snapshot Testing

Use `insta` for snapshot testing:

```rust
#[test]
fn test_parse_snapshot() {
    let mut parser = LanguageParser::new();
    let content = include_str!("fixtures/example.{ext}");
    
    let context = ParseContext {
        repo_id: "test".to_string(),
        file_path: PathBuf::from("example.{ext}"),
        old_tree: None,
        content: content.to_string(),
    };
    
    let result = parser.parse(&context).unwrap();
    
    // Snapshot the extracted nodes
    insta::assert_debug_snapshot!(result.nodes);
    insta::assert_debug_snapshot!(result.edges);
}
```

## Performance Optimization

### Parsing Performance

1. **Minimize Allocations**: Use string slices where possible
2. **Efficient Tree Traversal**: Avoid redundant walks
3. **Lazy Evaluation**: Only extract needed information
4. **Caching**: Cache expensive computations

```rust
// Efficient string handling
fn get_node_text<'a>(&self, node: &tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

// Avoid unnecessary allocations
fn extract_name(&self, node: &tree_sitter::Node) -> Option<&str> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
}
```

### Memory Management

```rust
// Pre-allocate collections with estimated capacity
let mut nodes = Vec::with_capacity(estimated_node_count);
let mut edges = Vec::with_capacity(estimated_edge_count);

// Use efficient data structures
let mut node_map = HashMap::with_capacity(estimated_node_count);
```

### Benchmarking

```rust
// benches/parse_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcore_lang_{language}::{LanguageParser, ParseContext};

fn bench_parse_large_file(c: &mut Criterion) {
    let content = include_str!("../tests/fixtures/large.{ext}");
    let mut parser = LanguageParser::new();
    
    c.bench_function("parse_large_{language}", |b| {
        b.iter(|| {
            let context = ParseContext {
                repo_id: "bench".to_string(),
                file_path: PathBuf::from("large.{ext}"),
                old_tree: None,
                content: black_box(content.to_string()),
            };
            parser.parse(&context).unwrap()
        })
    });
}

criterion_group!(benches, bench_parse_large_file);
criterion_main!(benches);
```

## Best Practices

### Error Handling

1. **Graceful Degradation**: Continue parsing when possible
2. **Detailed Error Messages**: Include location and context
3. **Error Recovery**: Handle malformed syntax gracefully

```rust
fn handle_malformed_node(&mut self, node: &tree_sitter::Node) -> Result<()> {
    if node.has_error() {
        tracing::warn!(
            "Malformed node at {}:{}: {}",
            node.start_position().row + 1,
            node.start_position().column + 1,
            node.kind()
        );
        // Continue processing other nodes
        return Ok(());
    }
    
    // Normal processing
    self.process_node(node)
}
```

### Logging and Debugging

```rust
use tracing::{debug, info, warn, instrument};

#[instrument(skip(self, content))]
pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
    info!("Parsing {}", context.file_path.display());
    debug!("Content length: {} bytes", context.content.len());
    
    let start = std::time::Instant::now();
    let result = self.parse_internal(context)?;
    let duration = start.elapsed();
    
    info!(
        "Parsed {} nodes, {} edges in {:?}",
        result.nodes.len(),
        result.edges.len(),
        duration
    );
    
    Ok(result)
}
```

### Documentation

```rust
/// Parse a {language} source file into Universal AST.
///
/// This function performs incremental parsing when an old tree is provided,
/// which can significantly improve performance for small edits.
///
/// # Arguments
///
/// * `context` - Parse context containing file information and content
///
/// # Returns
///
/// Returns a `ParseResult` containing the syntax tree, extracted nodes, and edges.
///
/// # Errors
///
/// Returns `Error::Parse` if the source code contains syntax errors that prevent
/// parsing, or `Error::NodeExtraction` if AST extraction fails.
///
/// # Examples
///
/// ```rust
/// use gcore_lang_{language}::{LanguageParser, ParseContext};
/// 
/// let mut parser = LanguageParser::new();
/// let context = ParseContext {
///     repo_id: "my-repo".to_string(),
///     file_path: PathBuf::from("example.{ext}"),
///     old_tree: None,
///     content: "function hello() {}".to_string(),
/// };
/// 
/// let result = parser.parse(&context)?;
/// assert!(!result.nodes.is_empty());
/// ```
pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
    // Implementation...
}
```

### Testing Guidelines

1. **Comprehensive Coverage**: Test all language features
2. **Edge Cases**: Malformed code, Unicode, large files
3. **Performance Tests**: Ensure parsing speed targets
4. **Integration Tests**: End-to-end functionality

### Version Compatibility

1. **Grammar Versions**: Pin Tree-Sitter grammar versions
2. **Language Versions**: Support multiple language versions
3. **Backward Compatibility**: Maintain API stability

This guide provides a comprehensive foundation for implementing language parsers in Prism. Each parser should follow these patterns while adapting to the specific characteristics of the target language. 