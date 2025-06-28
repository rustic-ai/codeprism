# API Documentation

This document provides detailed API documentation for all CodeCodePrism components.

## Table of Contents

- [Core Library (`codeprism`)](#core-librarycodeprism)
- [JavaScript/TypeScript Parser (`codeprism-lang-js`)](#javascripttypescript-parser-codeprism-lang-js)
- [Parser Development Tools (`codeprism-dev-tools`)](#parser-development-tools-codeprism-dev-tools)
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

## Parser Development Tools (`codeprism-dev-tools`)

The parser development tools crate provides essential debugging and development utilities for CodePrism parser development.

### Main Types

#### `DevTools`

Main facade providing access to all development utilities.

```rust
pub struct DevTools {
    // Internal components
}

impl DevTools {
    /// Create a new DevTools instance with default configuration
    pub fn new() -> Self

    /// Create a DevTools instance with custom configuration
    pub fn with_config(config: DevToolsConfig) -> Self

    /// Get access to the AST visualizer
    pub fn visualizer(&self) -> &AstVisualizer

    /// Get access to the parser validator
    pub fn validator(&self) -> &ParserValidator

    /// Get access to the performance profiler
    pub fn profiler(&self) -> &PerformanceProfiler

    /// Get access to the GraphViz exporter
    pub fn exporter(&self) -> &GraphVizExporter

    /// Start an interactive development REPL
    pub async fn start_repl(&self, language: Option<&str>) -> Result<()>

    /// Perform a comprehensive analysis of a parse result
    pub fn analyze_parse_result(
        &self,
        parse_result: &codeprism_core::ParseResult,
        source: &str,
    ) -> Result<AnalysisReport>

    /// Compare two parse results and generate a diff report
    pub fn compare_parse_results(
        &self,
        old_result: &codeprism_core::ParseResult,
        new_result: &codeprism_core::ParseResult,
        source: &str,
    ) -> Result<DiffReport>
}
```

**Example:**
```rust
use codeprism_dev_tools::DevTools;

// Create dev tools with default configuration
let dev_tools = DevTools::new();

// Analyze a parse result
let report = dev_tools.analyze_parse_result(&parse_result, &source_code)?;
println!("{}", report.format_report());

// Start interactive REPL for Python development
dev_tools.start_repl(Some("python")).await?;
```

#### `AstVisualizer`

Pretty-print syntax trees with configurable formatting.

```rust
pub struct AstVisualizer {
    // Internal configuration
}

impl AstVisualizer {
    /// Create a new visualizer with default configuration
    pub fn new() -> Self

    /// Create a visualizer with custom configuration
    pub fn with_config(config: VisualizationConfig) -> Self

    /// Visualize a tree-sitter tree
    pub fn visualize_tree(&self, tree: &Tree, source: &str) -> Result<String>

    /// Visualize in a specific format
    pub fn visualize_with_format(
        &self,
        tree: &Tree,
        source: &str,
        format: VisualizationFormat,
    ) -> Result<String>

    /// Generate statistics about the AST
    pub fn generate_statistics(&self, tree: &Tree, source: &str) -> Result<AstStatistics>

    /// Compare two ASTs and highlight differences
    pub fn compare_asts(&self, old_node: &Node, new_node: &Node, source: &str) -> Result<String>
}
```

**Supported Formats:**
```rust
#[derive(Debug, Clone, Copy)]
pub enum VisualizationFormat {
    Tree,        // Traditional tree format with Unicode box characters
    List,        // Flat list format
    Json,        // JSON representation
    SExpression, // S-expression format (Lisp-like)
    Compact,     // Condensed format for large trees
}
```

#### `ParserValidator`

Comprehensive validation tools for parser output.

```rust
pub struct ParserValidator {
    // Internal configuration
}

impl ParserValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self

    /// Perform comprehensive validation of a parse result
    pub fn validate_complete(
        &self,
        parse_result: &ParseResult,
        source: &str,
    ) -> Result<ValidationReport>

    /// Validate only span overlaps
    pub fn validate_spans(&self, nodes: &[Node]) -> Result<Vec<ValidationError>>

    /// Validate edge consistency
    pub fn validate_edges(&self, nodes: &[Node], edges: &[Edge]) -> Result<Vec<ValidationError>>
}
```

**Validation Report:**
```rust
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub statistics: ValidationStatistics,
    pub is_valid: bool,
}

impl ValidationReport {
    /// Check if the validation passed (no errors)
    pub fn is_valid(&self) -> bool

    /// Get all validation errors
    pub fn errors(&self) -> &[ValidationError]

    /// Get all validation warnings
    pub fn warnings(&self) -> &[ValidationWarning]

    /// Generate a summary report
    pub fn summary(&self) -> String
}
```

#### `GraphVizExporter`

Export ASTs to GraphViz DOT format for visual analysis.

```rust
pub struct GraphVizExporter {
    // Internal configuration
}

impl GraphVizExporter {
    /// Create a new exporter with default configuration
    pub fn new() -> Self

    /// Create an exporter with custom configuration
    pub fn with_config(config: GraphVizConfig) -> Self

    /// Export nodes and edges to GraphViz DOT format
    pub fn export_nodes_and_edges(&self, nodes: &[Node], edges: &[Edge]) -> Result<String>

    /// Export with custom options
    pub fn export_with_options(
        &self,
        nodes: &[Node],
        edges: &[Edge],
        options: &GraphVizOptions,
    ) -> Result<String>

    /// Export a tree-sitter tree to DOT format
    pub fn export_tree(&self, tree: &Tree, source: &str) -> Result<String>
}
```

**GraphViz Options:**
```rust
pub struct GraphVizOptions {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub highlight_nodes: Vec<String>,
    pub highlight_edges: Vec<String>,
    pub filter_node_types: Option<Vec<NodeKind>>,
    pub filter_edge_types: Option<Vec<EdgeKind>>,
    pub cluster_by_file: bool,
    pub show_spans: bool,
}
```

#### `PerformanceProfiler`

Real-time parsing performance metrics with bottleneck detection.

```rust
pub struct PerformanceProfiler {
    // Internal metrics storage
}

impl PerformanceProfiler {
    /// Create a new profiler with default configuration
    pub fn new() -> Self

    /// Create a profiler with custom configuration
    pub fn with_config(config: ProfilingConfig) -> Self

    /// Start a new profiling session
    pub fn start_session(&mut self)

    /// Record a performance metric
    pub fn record_metric(
        &mut self,
        metric_type: MetricType,
        value: f64,
        unit: &str,
        context: Option<String>,
    )

    /// Generate performance summary
    pub fn generate_summary(&self) -> PerformanceSummary

    /// Analyze performance bottlenecks
    pub fn analyze_bottlenecks(&self) -> Vec<PerformanceBottleneck>

    /// Get performance recommendations
    pub fn get_recommendations(&self) -> Vec<String>
}
```

**Metric Types:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    ParseTime,     // Time to parse a file
    MemoryUsage,   // Memory consumption
    NodeCreation,  // Number of nodes created
    EdgeCreation,  // Number of edges created
    FileSize,      // Size of file being parsed
}
```

#### `AstDiff`

Compare parse results between parser versions with detailed change analysis.

```rust
pub struct AstDiff {
    // Internal configuration
}

impl AstDiff {
    /// Create a new AST diff analyzer
    pub fn new() -> Self

    /// Create with custom configuration
    pub fn with_config(config: DiffConfig) -> Self

    /// Compare two parse results
    pub fn compare(
        &self,
        old_result: &ParseResult,
        new_result: &ParseResult,
        source: &str,
    ) -> Result<DiffReport>

    /// Compare only nodes
    pub fn compare_nodes(&self, old_nodes: &[Node], new_nodes: &[Node]) -> Result<Vec<DiffType>>

    /// Compare only edges
    pub fn compare_edges(&self, old_edges: &[Edge], new_edges: &[Edge]) -> Result<Vec<DiffType>>
}
```

**Diff Report:**
```rust
pub struct DiffReport {
    pub differences: Vec<DiffType>,
    pub statistics: DiffStatistics,
    pub similarity_score: f64,
    pub is_significant_change: bool,
    pub summary: String,
}

impl DiffReport {
    /// Format the report for display
    pub fn format_report(&self) -> String

    /// Get only the significant differences
    pub fn significant_differences(&self) -> Vec<&DiffType>
}
```

#### `DevRepl`

Interactive command-line interface for parser development and testing.

```rust
pub struct DevRepl {
    // Internal state
}

impl DevRepl {
    /// Create a new REPL for the specified language
    pub fn new(language: Option<&str>) -> Result<Self>

    /// Set the AST visualizer
    pub fn set_visualizer(&mut self, visualizer: AstVisualizer)

    /// Set the parser validator
    pub fn set_validator(&mut self, validator: ParserValidator)

    /// Set the performance profiler
    pub fn set_profiler(&mut self, profiler: PerformanceProfiler)

    /// Set the GraphViz exporter
    pub fn set_exporter(&mut self, exporter: GraphVizExporter)

    /// Run the interactive REPL
    pub async fn run(&mut self) -> Result<()>

    /// Execute a single command
    pub async fn execute_command(&mut self, command: &str) -> ReplResult
}
```

**REPL Commands:**
```rust
#[derive(Debug, Clone)]
pub enum ReplCommand {
    Parse { source: String },
    Load { file_path: String },
    Show { what: ShowTarget },
    Set { option: String, value: String },
    Export { format: ExportFormat, output: Option<String> },
    Compare { old_source: String, new_source: String },
    Profile { command: String },
    Help,
    Clear,
    History,
    Exit,
}
```

### Configuration Types

#### `DevToolsConfig`

Main configuration for all development tools.

```rust
#[derive(Debug, Clone, Default)]
pub struct DevToolsConfig {
    pub visualization: VisualizationConfig,
    pub validation: ValidationConfig,
    pub profiling: ProfilingConfig,
    pub graphviz: GraphVizConfig,
}
```

#### `VisualizationConfig`

Configuration for AST visualization.

```rust
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub show_positions: bool,
    pub show_byte_ranges: bool,
    pub show_text_content: bool,
    pub max_text_length: usize,
    pub max_depth: usize,
    pub color_scheme: ColorScheme,
}
```

#### `ValidationConfig`

Configuration for parser validation.

```rust
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub check_span_overlaps: bool,
    pub check_edge_consistency: bool,
    pub check_unreachable_nodes: bool,
    pub check_text_coverage: bool,
    pub check_duplicate_nodes: bool,
    pub min_span_size: usize,
    pub max_parsing_time_ms: u64,
    pub check_syntax_tree_structure: bool,
}
```

### Example Usage

#### Basic AST Visualization

```rust
use codeprism_dev_tools::{AstVisualizer, VisualizationFormat};

let visualizer = AstVisualizer::new();

// Parse some code
let source = "function hello() { return 'world'; }";
let parse_result = parser.parse_source(&source)?;

// Visualize the AST
let tree_output = visualizer.visualize_tree(&parse_result.tree, &source)?;
println!("{}", tree_output);

// Try different formats
let json_output = visualizer.visualize_with_format(
    &parse_result.tree,
    &source,
    VisualizationFormat::Json,
)?;
```

#### Parser Validation

```rust
use codeprism_dev_tools::ParserValidator;

let validator = ParserValidator::new();

// Validate a parse result
let report = validator.validate_complete(&parse_result, &source)?;

if !report.is_valid() {
    println!("Validation failed:");
    for error in report.errors() {
        println!("  - {}", error);
    }
}

println!("Validation summary:\n{}", report.summary());
```

#### Performance Profiling

```rust
use codeprism_dev_tools::{PerformanceProfiler, MetricType};

let mut profiler = PerformanceProfiler::new();

// Start profiling session
profiler.start_session();

// Record metrics during parsing
let start = std::time::Instant::now();
let parse_result = parser.parse_source(&source)?;
let duration = start.elapsed();

profiler.record_metric(
    MetricType::ParseTime,
    duration.as_millis() as f64,
    "ms",
    Some("test_file.js".to_string()),
);

// Analyze results
let summary = profiler.generate_summary();
let bottlenecks = profiler.analyze_bottlenecks();
let recommendations = profiler.get_recommendations();
```

#### Interactive Development

```rust
use codeprism_dev_tools::DevRepl;

// Start an interactive REPL for JavaScript
let mut repl = DevRepl::new(Some("javascript"))?;
repl.run().await?;

// Or execute specific commands
let result = repl.execute_command("parse function test() {}").await?;
println!("{}", result.output);
```

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