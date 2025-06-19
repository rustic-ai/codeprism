# API Documentation

This document provides detailed API documentation for all CodeCodePrism components.

## Table of Contents

- [Core Library (codeprism`)](#core-librarycodeprism)
- [JavaScript/TypeScript Parser (`codeprism-lang-js`)](#javascripttypescript-parser-codeprism-lang-js)
- [MCP Server (`codeprism-mcp`)](#mcp-server-codeprism-mcp)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Core Library (codeprism`)

The core library provides the fundamental types and engine for code analysis.

### Universal AST Types

#### `NodeId`

Unique identifier for AST nodes using Blake3 hashing.

```rust
pub struct NodeId([u8; 16]);

impl NodeId {
    /// Create a new NodeId from components
    pub fn new(
        repo_id: &str,
        file_path: &Path,
        span: &Span,
        kind: &NodeKind
    ) -> Self

    /// Get the ID as a hex string
    pub fn to_hex(&self) -> String
}
```

**Example:**
```rust
use codeprism::ast::{NodeId, NodeKind, Span};
use std::path::Path;

let span = Span::new(0, 10, 1, 1, 1, 11);
let id = NodeId::new("my-repo", Path::new("app.js"), &span, &NodeKind::Function);
println!("Node ID: {}", id.to_hex());
```

#### `NodeKind`

Enumeration of all supported node types in the Universal AST.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Module,      // A module or file
    Class,       // A class definition
    Function,    // A function definition
    Method,      // A method definition
    Parameter,   // A function/method parameter
    Variable,    // A variable declaration
    Call,        // A function/method call
    Import,      // An import statement
    Literal,     // A literal value
    Route,       // An HTTP route definition
    SqlQuery,    // A SQL query
    Event,       // An event emission
    Unknown,     // Unknown node type
}
```

#### `EdgeKind`

Enumeration of relationship types between nodes.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Calls,       // Function/method call
    Reads,       // Variable/field read
    Writes,      // Variable/field write
    Imports,     // Module import
    Emits,       // Event emission
    RoutesTo,    // HTTP route mapping
    Raises,      // Exception raising
    Extends,     // Type inheritance
    Implements,  // Interface implementation
}
```

#### `Span`

Source code location information.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: usize,    // Starting byte offset
    pub end_byte: usize,      // Ending byte offset (exclusive)
    pub start_line: usize,    // Starting line (1-indexed)
    pub end_line: usize,      // Ending line (1-indexed)
    pub start_column: usize,  // Starting column (1-indexed)
    pub end_column: usize,    // Ending column (1-indexed)
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
    ) -> Self

    /// Create a span from tree-sitter node
    pub fn from_node(node: &tree_sitter::Node) -> Self
}
```

#### `Node`

A node in the Universal AST.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,                    // Unique identifier
    pub kind: NodeKind,                // Node type
    pub name: String,                  // Node name (e.g., function name)
    pub lang: Language,                // Programming language
    pub file: PathBuf,                 // Source file path
    pub span: Span,                    // Source location
    pub signature: Option<String>,     // Optional type signature
    pub metadata: serde_json::Value,   // Additional metadata
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
    ) -> Self

    /// Create a node builder
    pub fn builder() -> NodeBuilder
}
```

#### `Edge`

A relationship between two nodes.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeId,  // Source node ID
    pub target: NodeId,  // Target node ID
    pub kind: EdgeKind,  // Edge type
}

impl Edge {
    /// Create a new edge
    pub fn new(source: NodeId, target: NodeId, kind: EdgeKind) -> Self
}
```

### Parser Engine

#### `LanguageParser` Trait

Trait that all language parsers must implement.

```rust
pub trait LanguageParser: Send + Sync {
    /// Parse a file and return the result
    fn parse_file(
        &self,
        context: ParseContext,
    ) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];

    /// Get the language name
    fn language_name(&self) -> &str;
}
```

#### `LanguageRegistry`

Thread-safe registry for language parsers.

```rust
pub struct LanguageRegistry {
    // Internal implementation
}

impl LanguageRegistry {
    /// Create a new language registry
    pub fn new() -> Self

    /// Register a language parser
    pub fn register(&self, parser: Arc<dyn LanguageParser>)

    /// Get a parser for a file extension
    pub fn get_parser(&self, extension: &str) -> Option<Arc<dyn LanguageParser>>

    /// List all registered languages
    pub fn list_languages(&self) -> Vec<String>
}
```

#### `ParserEngine`

Main parsing engine that coordinates language parsers.

```rust
pub struct ParserEngine {
    // Internal implementation
}

impl ParserEngine {
    /// Create a new parser engine
    pub fn new(registry: Arc<LanguageRegistry>) -> Self

    /// Parse a file using the appropriate language parser
    pub fn parse_file(&self, context: ParseContext) -> Result<ParseResult>

    /// Parse multiple files concurrently
    pub async fn parse_files(&self, contexts: Vec<ParseContext>) -> Vec<Result<ParseResult>>
}
```

#### `ParseContext`

Context information for parsing operations.

```rust
#[derive(Debug, Clone)]
pub struct ParseContext {
    pub repo_id: String,              // Repository identifier
    pub file_path: PathBuf,           // File path being parsed
    pub content: String,              // File content
    pub old_tree: Option<Tree>,       // Previous tree for incremental parsing
}

impl ParseContext {
    /// Create a new parse context
    pub fn new(repo_id: String, file_path: PathBuf, content: String) -> Self

    /// Create with old tree for incremental parsing
    pub fn with_old_tree(
        repo_id: String,
        file_path: PathBuf,
        content: String,
        old_tree: Tree,
    ) -> Self
}
```

#### `ParseResult`

Result of a parsing operation.

```rust
#[derive(Debug)]
pub struct ParseResult {
    pub tree: Tree,        // The parsed syntax tree
    pub nodes: Vec<Node>,  // Extracted nodes
    pub edges: Vec<Edge>,  // Extracted edges
}
```

### File Watcher

#### `FileWatcher`

Monitors file system changes with debouncing.

```rust
pub struct FileWatcher {
    // Internal implementation
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self>

    /// Watch a directory for changes
    pub fn watch_dir(&mut self, path: &Path, root: PathBuf) -> Result<()>

    /// Stop watching a directory
    pub fn unwatch_dir(&mut self, path: &Path) -> Result<()>

    /// Get the next change event (async)
    pub async fn next_change(&mut self) -> Option<ChangeEvent>

    /// Set debounce duration
    pub fn set_debounce_duration(&mut self, duration: Duration)
}
```

#### `ChangeEvent`

File system change event.

```rust
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub path: PathBuf,      // Path that changed
    pub kind: ChangeKind,   // Type of change
    pub timestamp: SystemTime,  // When the change occurred
}
```

#### `ChangeKind`

Types of file system changes.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,   // File was created
    Modified,  // File was modified
    Deleted,   // File was deleted
    Renamed,   // File was renamed
}
```

### Graph Patches

#### `AstPatch`

Represents incremental changes to the AST graph.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPatch {
    pub id: String,                    // Unique patch ID
    pub repo_id: String,               // Repository ID
    pub file_path: PathBuf,            // File that changed
    pub timestamp: SystemTime,         // When the patch was created
    pub added_nodes: Vec<Node>,        // Nodes to add
    pub removed_nodes: Vec<NodeId>,    // Nodes to remove
    pub added_edges: Vec<Edge>,        // Edges to add
    pub removed_edges: Vec<Edge>,      // Edges to remove
}

impl AstPatch {
    /// Create a new patch
    pub fn new(repo_id: String, file_path: PathBuf) -> Self

    /// Create a patch builder
    pub fn builder(repo_id: String, file_path: PathBuf) -> AstPatchBuilder

    /// Add a node to the patch
    pub fn add_node(&mut self, node: Node)

    /// Remove a node from the patch
    pub fn remove_node(&mut self, node_id: NodeId)

    /// Add an edge to the patch
    pub fn add_edge(&mut self, edge: Edge)

    /// Remove an edge from the patch
    pub fn remove_edge(&mut self, edge: Edge)

    /// Merge another patch into this one
    pub fn merge(&mut self, other: AstPatch) -> Result<()>

    /// Validate the patch for consistency
    pub fn validate(&self) -> Result<()>
}
```

## JavaScript/TypeScript Parser (`codeprism-lang-js`)

Language-specific parser for JavaScript and TypeScript files.

### `JavaScriptParser`

Main parser implementation for JavaScript/TypeScript.

```rust
pub struct JavaScriptParser {
    // Internal implementation
}

impl JavaScriptParser {
    /// Create a new JavaScript/TypeScript parser
    pub fn new() -> Self

    /// Detect language from file path
    pub fn detect_language(path: &PathBuf) -> Language

    /// Parse a JavaScript or TypeScript file
    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult>
}
```

### `ParseContext` (JS-specific)

Parse context for JavaScript/TypeScript files.

```rust
#[derive(Debug, Clone)]
pub struct ParseContext {
    pub repo_id: String,              // Repository ID
    pub file_path: PathBuf,           // File path being parsed
    pub old_tree: Option<Tree>,       // Previous tree for incremental parsing
    pub content: String,              // File content
}
```

### `Language`

JavaScript/TypeScript language enumeration.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    JavaScript,  // JavaScript
    TypeScript,  // TypeScript
}
```

### Integration Functions

#### `parse_file`

Convenience function for parsing files.

```rust
pub fn parse_file(
    parser: &JavaScriptLanguageParser,
    repo_id: &str,
    file_path: PathBuf,
    content: String,
    old_tree: Option<tree_sitter::Tree>,
) -> Result<(tree_sitter::Tree, Vec<Node>, Vec<Edge>), Error>
```

#### `create_parser`

Factory function for creating language parser adapters.

```rust
pub fn create_parser() -> JavaScriptLanguageParser
```

### Supported Features

The JavaScript/TypeScript parser supports:

- **Functions**: Function declarations, arrow functions, methods
- **Classes**: Class declarations, constructors, methods
- **Variables**: const, let, var declarations
- **Imports/Exports**: ES6 modules, CommonJS requires
- **Calls**: Function calls, method calls
- **TypeScript**: Basic type annotations, interfaces

## MCP Server (`codeprism-mcp`)

Model Context Protocol server for code intelligence.

### Starting the Server

The MCP server is designed to be launched by MCP clients:

```bash
codeprism-mcp <REPOSITORY_PATH>
```

### MCP Client Configuration

#### Claude Desktop Configuration

```json
{
  "mcpServers": {
    codeprism": {
      "command": "codeprism-mcp",
      "args": ["/path/to/repository"]
    }
  }
}
```

#### Cursor Configuration

```json
{
  "mcp": {
    "servers": [{
      "name": codeprism",
      "command": ["codeprism-mcp", "."]
    }]
  }
}
```

### MCP Resources

The server exposes repository data through MCP resources:

- codeprism://repository/file/{path}` - Access file content
- codeprism://graph/repository` - Repository graph structure  
- codeprism://symbols/{type}` - Symbol listings (functions, classes, etc.)

### MCP Tools

Available analysis tools:

#### `repo_stats`
Get repository statistics and metrics.

#### `find_references`
Find all references to a symbol across the codebase.

#### `search_symbols`
Search for symbols by name or regex pattern.

#### `trace_path`
Trace execution paths between symbols.

#### `explain_symbol`
Get detailed information about a symbol including context.

#### `find_dependencies`
Analyze dependencies for files or symbols.

### MCP Prompts

Predefined analysis prompts:

#### `repository_overview`
Generate comprehensive repository overview.

#### `code_analysis`  
Analyze code quality, patterns, and improvements.

#### `debug_assistance`
Help debug issues with contextual information.

#### `refactoring_guidance`
Provide guidance for code refactoring.

## Error Handling

All CodeCodePrism components use structured error handling with `thiserror`.

### Core Errors

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

### JavaScript Parser Errors

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error in {file}: {message}")]
    Parse { file: PathBuf, message: String },

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
}
```

## Examples

### Basic Parsing

```rust
use codeprism_lang_js::{JavaScriptParser, ParseContext};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = JavaScriptParser::new();
    
    let context = ParseContext {
        repo_id: "example".to_string(),
        file_path: PathBuf::from("app.js"),
        old_tree: None,
        content: r#"
            function greet(name) {
                console.log(`Hello, ${name}!`);
            }
            
            greet("World");
        "#.to_string(),
    };
    
    let result = parser.parse(&context)?;
    
    println!("Found {} nodes:", result.nodes.len());
    for node in &result.nodes {
        println!("  {:?}: {}", node.kind, node.name);
    }
    
    println!("Found {} edges:", result.edges.len());
    for edge in &result.edges {
        println!("  {:?}", edge.kind);
    }
    
    Ok(())
}
```

### File Watching

```rust
use codeprism::{FileWatcher, ChangeKind};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = FileWatcher::new()?;
    
    // Watch the src directory
    watcher.watch_dir(
        Path::new("src/"),
        std::env::current_dir()?
    )?;
    
    println!("Watching for file changes...");
    
    while let Some(event) = watcher.next_change().await {
        match event.kind {
            ChangeKind::Modified => {
                println!("File modified: {:?}", event.path);
                // Trigger parsing here
            }
            ChangeKind::Created => {
                println!("File created: {:?}", event.path);
            }
            ChangeKind::Deleted => {
                println!("File deleted: {:?}", event.path);
            }
            ChangeKind::Renamed => {
                println!("File renamed: {:?}", event.path);
            }
        }
    }
    
    Ok(())
}
```

### Parser Engine Usage

```rust
use codeprism::{LanguageRegistry, ParserEngine, ParseContext};
use codeprism_lang_js::JavaScriptLanguageParser;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create language registry
    let registry = Arc::new(LanguageRegistry::new());
    
    // Register JavaScript parser
    registry.register(Arc::new(JavaScriptLanguageParser::new()));
    
    // Create parser engine
    let engine = ParserEngine::new(registry);
    
    // Parse a file
    let context = ParseContext::new(
        "my-repo".to_string(),
        PathBuf::from("app.js"),
        std::fs::read_to_string("app.js")?,
    );
    
    let result = engine.parse_file(context)?;
    
    println!("Parsed {} nodes and {} edges", 
             result.nodes.len(), 
             result.edges.len());
    
    Ok(())
}
```

### Creating Graph Patches

```rust
use codeprism::{AstPatch, Node, NodeKind, Language, Span};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut patch = AstPatch::new(
        "my-repo".to_string(),
        PathBuf::from("app.js")
    );
    
    // Add a new function node
    let span = Span::new(0, 20, 1, 1, 1, 21);
    let node = Node::new(
        "my-repo",
        NodeKind::Function,
        "newFunction".to_string(),
        Language::JavaScript,
        PathBuf::from("app.js"),
        span,
    );
    
    patch.add_node(node);
    
    // Validate the patch
    patch.validate()?;
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&patch)?;
    println!("Patch: {}", json);
    
    Ok(())
}
```

## Performance Considerations

### Memory Usage

- **NodeId**: 16 bytes per node
- **Node**: ~200-500 bytes depending on metadata
- **Edge**: 48 bytes per edge
- **Tree**: Managed by tree-sitter, minimal overhead

### Parsing Performance

- **JavaScript**: ~5-10µs per line of code
- **Incremental**: Sub-millisecond for small edits
- **Memory**: Scales linearly with file size

### Optimization Tips

1. **Use incremental parsing** for small edits
2. **Batch operations** when possible
3. **Cache parse results** for unchanged files
4. **Use appropriate debounce intervals** for file watching
5. **Profile with `cargo flamegraph`** for hot paths

## Thread Safety

All public APIs are thread-safe:

- **LanguageRegistry**: Uses `DashMap` for concurrent access
- **ParserEngine**: Immutable after creation
- **FileWatcher**: Uses async channels
- **Language Parsers**: Wrapped in `Mutex` when needed

## Versioning

CodeCodePrism follows [Semantic Versioning](https://semver.org/):

- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible

Current version: `0.1.0` (pre-release) 