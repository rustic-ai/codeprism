# Prism Critical Action Plan - Addressing Implementation Gaps

## Executive Summary

Following the comprehensive review of the Prism MCP Server description document against current implementation, **critical gaps have been identified** that prevent the system from delivering its promised functionality. This document outlines the specific actions required to bridge these gaps and achieve compliance with the specification.

## Critical Status Assessment

### ❌ Current State Issues
1. **No Repository Operations**: Cannot scan or index repositories
2. **MCP Non-Compliance**: Current server won't work with MCP clients
3. **Missing Real-Time Updates**: File monitoring not integrated
4. **Non-Functional CLI**: Basic stubs only, missing repository commands

### ✅ Strengths to Build On
1. **Solid Foundation**: Core AST types and parser engine complete
2. **Language Support**: JavaScript/TypeScript and Python parsers working
3. **File Watching**: FileWatcher component implemented
4. **Basic Graph Storage**: In-memory graph operations functional

## Phase 2.5: Repository Scanner Implementation (CRITICAL PRIORITY)

### 2.5.1 Create Repository Scanner Module

**File**: `crates/prism/src/scanner/mod.rs`

```rust
//! Repository scanner for bulk directory processing

use crate::parser::{ParserEngine, ParseContext};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use walkdir::WalkDir;
use ignore::Walk;

pub struct RepositoryScanner {
    parser_engine: Arc<ParserEngine>,
    ignore_patterns: ignore::gitignore::Gitignore,
    supported_extensions: HashSet<String>,
    parallel_limit: usize,
}

#[derive(Debug)]
pub struct ScanProgress {
    pub files_discovered: usize,
    pub files_processed: usize,
    pub files_failed: usize,
    pub current_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ScanResult {
    pub total_files: usize,
    pub parsed_files: usize,
    pub failed_files: Vec<(PathBuf, String)>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub languages_detected: HashMap<Language, usize>,
}

impl RepositoryScanner {
    pub fn new(parser_engine: Arc<ParserEngine>) -> Self;
    
    pub fn with_ignore_patterns(mut self, patterns: &[&str]) -> Self;
    
    pub fn with_parallel_limit(mut self, limit: usize) -> Self;
    
    pub async fn scan_repository(
        &self,
        repo_path: &Path,
        progress_callback: impl Fn(ScanProgress) + Send + 'static,
    ) -> Result<ScanResult>;
    
    pub fn discover_files(&self, repo_path: &Path) -> Result<Vec<PathBuf>>;
    
    pub async fn parse_files_parallel(
        &self,
        files: Vec<PathBuf>,
        repo_id: String,
        progress_callback: impl Fn(ScanProgress) + Send + 'static,
    ) -> Result<ScanResult>;
}
```

### 2.5.2 Create Bulk Indexing Engine

**File**: `crates/prism/src/indexer/mod.rs`

```rust
//! Bulk indexing engine for repository processing

use crate::scanner::{RepositoryScanner, ScanResult, ScanProgress};
use crate::ast::{Node, Edge};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct BulkIndexer {
    scanner: RepositoryScanner,
    batch_size: usize,
    max_memory_mb: usize,
}

#[derive(Debug)]
pub struct IndexProgress {
    pub scan_progress: ScanProgress,
    pub nodes_indexed: usize,
    pub edges_indexed: usize,
    pub memory_usage_mb: usize,
}

#[derive(Debug)]
pub struct IndexResult {
    pub scan_result: ScanResult,
    pub index_time_ms: u64,
    pub peak_memory_mb: usize,
}

impl BulkIndexer {
    pub fn new(scanner: RepositoryScanner) -> Self;
    
    pub fn with_batch_size(mut self, size: usize) -> Self;
    
    pub fn with_memory_limit(mut self, limit_mb: usize) -> Self;
    
    pub async fn index_repository(
        &self,
        repo_path: &Path,
        graph_store: Arc<dyn GraphStore>,
        progress_callback: impl Fn(IndexProgress) + Send + 'static,
    ) -> Result<IndexResult>;
    
    pub async fn batch_update_graph(
        &self,
        nodes: Vec<Node>,
        edges: Vec<Edge>,
        graph_store: Arc<dyn GraphStore>,
    ) -> Result<()>;
}
```

### 2.5.3 Create Repository Manager

**File**: `crates/prism/src/repository/mod.rs`

```rust
//! Repository management and configuration

use crate::indexer::{BulkIndexer, IndexResult};
use crate::scanner::RepositoryScanner;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub path: PathBuf,
    pub ignore_patterns: Vec<String>,
    pub languages: Vec<String>,
    pub parallel_limit: usize,
    pub batch_size: usize,
    pub auto_watch: bool,
}

pub struct RepositoryManager {
    config: RepositoryConfig,
    indexer: BulkIndexer,
    graph_store: Arc<dyn GraphStore>,
}

#[derive(Debug)]
pub struct RepositoryStatus {
    pub config: RepositoryConfig,
    pub last_indexed: Option<std::time::SystemTime>,
    pub total_files: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub health: RepositoryHealth,
}

#[derive(Debug)]
pub enum RepositoryHealth {
    Healthy,
    NeedsReindex,
    Error(String),
}

impl RepositoryManager {
    pub fn new(config: RepositoryConfig, graph_store: Arc<dyn GraphStore>) -> Self;
    
    pub async fn initial_index(&self) -> Result<IndexResult>;
    
    pub async fn get_status(&self) -> RepositoryStatus;
    
    pub async fn reindex(&self) -> Result<IndexResult>;
    
    pub fn start_watching(&self) -> Result<()>;
    
    pub fn stop_watching(&self) -> Result<()>;
}
```

## Phase 3: MCP Protocol Compliance (HIGH PRIORITY)

### 3.1 Implement JSON-RPC 2.0 for MCP

**File**: `crates/prism-mcp/src/protocol/mod.rs`

```rust
//! MCP protocol implementation

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>,
    pub prompts: Option<PromptCapabilities>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub subscribe: bool,
    pub list_changed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: McpCapabilities,
}

// JSON-RPC 2.0 types
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}
```

### 3.2 Implement MCP Resources

**File**: `crates/prism-mcp/src/resources/mod.rs`

```rust
//! MCP Resources implementation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: Option<String>,
    pub blob: Option<Vec<u8>>,
    pub annotations: Option<Value>,
}

pub struct ResourceHandler {
    repository_manager: Arc<RepositoryManager>,
}

impl ResourceHandler {
    // resources/list - List repository files and graph data
    pub async fn list_resources(&self) -> Result<Vec<McpResource>>;
    
    // resources/read - Read file content with code analysis
    pub async fn read_resource(&self, uri: &str) -> Result<McpResourceContent>;
    
    // Helper methods
    pub fn list_file_resources(&self) -> Result<Vec<McpResource>>;
    pub fn list_graph_resources(&self) -> Result<Vec<McpResource>>;
    pub fn list_symbol_resources(&self) -> Result<Vec<McpResource>>;
}
```

### 3.3 Implement MCP Tools

**File**: `crates/prism-mcp/src/tools/mod.rs`

```rust
//! MCP Tools implementation

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value, // JSON Schema
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpToolContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub data: Option<Value>,
}

pub struct ToolHandler {
    repository_manager: Arc<RepositoryManager>,
    graph_store: Arc<dyn GraphStore>,
}

impl ToolHandler {
    // tools/list - List available tools
    pub fn list_tools(&self) -> Vec<McpTool>;
    
    // tools/call - Execute tool
    pub async fn call_tool(&self, call: McpToolCall) -> Result<McpToolResult>;
    
    // Specific tool implementations
    pub async fn tool_trace_path(&self, args: Value) -> Result<McpToolResult>;
    pub async fn tool_explain_symbol(&self, args: Value) -> Result<McpToolResult>;
    pub async fn tool_find_dependencies(&self, args: Value) -> Result<McpToolResult>;
    pub async fn tool_repo_stats(&self, args: Value) -> Result<McpToolResult>;
    pub async fn tool_find_references(&self, args: Value) -> Result<McpToolResult>;
    pub async fn tool_search_symbols(&self, args: Value) -> Result<McpToolResult>;
}
```

### 3.4 Implement MCP Prompts

**File**: `crates/prism-mcp/src/prompts/mod.rs`

```rust
//! MCP Prompts implementation

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPromptCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPromptResult {
    pub description: String,
    pub messages: Vec<McpPromptMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPromptMessage {
    pub role: String,
    pub content: McpPromptContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpPromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

pub struct PromptHandler {
    repository_manager: Arc<RepositoryManager>,
    graph_store: Arc<dyn GraphStore>,
}

impl PromptHandler {
    // prompts/list - List available prompts
    pub fn list_prompts(&self) -> Vec<McpPrompt>;
    
    // prompts/get - Generate prompt
    pub async fn get_prompt(&self, call: McpPromptCall) -> Result<McpPromptResult>;
    
    // Specific prompt implementations
    pub async fn prompt_repo_overview(&self, args: Value) -> Result<McpPromptResult>;
    pub async fn prompt_debug_issue(&self, args: Value) -> Result<McpPromptResult>;
    pub async fn prompt_refactor_guidance(&self, args: Value) -> Result<McpPromptResult>;
}
```

## Phase 2.6: File Monitoring Integration (MEDIUM PRIORITY)

### 2.6.1 Create Parsing Pipeline

**File**: `crates/prism/src/pipeline/mod.rs`

```rust
//! File monitoring and parsing pipeline

use crate::watcher::{FileWatcher, ChangeEvent};
use crate::parser::{ParserEngine, ParseContext};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ParsingPipeline {
    watcher: Arc<FileWatcher>,
    parser_engine: Arc<ParserEngine>,
    graph_store: Arc<dyn GraphStore>,
    debounce_ms: u64,
}

#[derive(Debug)]
pub struct PipelineEvent {
    pub event_type: PipelineEventType,
    pub file_path: PathBuf,
    pub nodes_updated: usize,
    pub edges_updated: usize,
}

#[derive(Debug)]
pub enum PipelineEventType {
    FileAdded,
    FileModified,
    FileDeleted,
    ParseError(String),
}

impl ParsingPipeline {
    pub fn new(
        watcher: Arc<FileWatcher>,
        parser_engine: Arc<ParserEngine>,
        graph_store: Arc<dyn GraphStore>,
    ) -> Self;
    
    pub async fn start(&self) -> Result<mpsc::Receiver<PipelineEvent>>;
    
    pub async fn stop(&self) -> Result<()>;
    
    async fn handle_file_change(&self, event: ChangeEvent) -> Result<PipelineEvent>;
    
    async fn update_graph_for_file(&self, file_path: &Path, repo_id: &str) -> Result<(usize, usize)>;
    
    async fn remove_file_from_graph(&self, file_path: &Path) -> Result<()>;
}
```

## Phase 4: CLI/Daemon Implementation (LOWER PRIORITY)

### 4.1 Repository-Aware CLI

**File**: `crates/prism-cli/src/commands/mod.rs`

```rust
//! CLI commands for repository operations

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prism")]
#[command(about = "Prism graph-first code intelligence")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a repository
    Index {
        /// Repository path
        path: PathBuf,
        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
        /// Show progress
        #[arg(long)]
        progress: bool,
    },
    /// Start MCP server for repository
    Serve {
        /// Repository path
        path: PathBuf,
        /// Server port
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Enable file watching
        #[arg(long)]
        watch: bool,
    },
    /// Watch repository for changes
    Watch {
        /// Repository path
        path: PathBuf,
    },
    /// Show repository statistics
    Stats {
        /// Repository path
        path: PathBuf,
        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
    },
}

pub async fn execute_command(cli: Cli) -> Result<()>;
```

### 4.2 Repository-Aware Daemon

**File**: `crates/prism-daemon/src/service/mod.rs`

```rust
//! Prism daemon service

use crate::config::DaemonConfig;
use crate::repository::RepositoryManager;
use crate::mcp::McpServer;
use std::sync::Arc;

pub struct PrismDaemon {
    config: DaemonConfig,
    repository_manager: Arc<RepositoryManager>,
    mcp_server: McpServer,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl PrismDaemon {
    pub fn new(config: DaemonConfig) -> Result<Self>;
    
    pub async fn start(&mut self) -> Result<()>;
    
    pub async fn stop(&mut self) -> Result<()>;
    
    pub async fn reload_config(&mut self) -> Result<()>;
    
    pub fn health_check(&self) -> DaemonHealth;
    
    async fn setup_signal_handlers(&self) -> Result<()>;
    
    async fn start_mcp_server(&self) -> Result<()>;
    
    async fn start_file_monitoring(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct DaemonHealth {
    pub status: HealthStatus,
    pub repository_status: RepositoryStatus,
    pub mcp_server_status: ServerStatus,
    pub memory_usage_mb: usize,
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Error(String),
}
```

## Implementation Schedule

### Phase 2.5: Repository Operations Foundation
- [ ] Repository Scanner implementation
- [ ] Bulk Indexing Engine implementation  
- [ ] Repository Manager implementation
- [ ] Integration tests for repository operations
- [ ] Performance benchmarks for large repositories

### Phase 3: MCP Protocol Compliance
- [ ] JSON-RPC 2.0 implementation
- [ ] MCP Resources implementation
- [ ] MCP Tools implementation
- [ ] MCP Prompts implementation
- [ ] MCP protocol compliance tests

### Phase 2.6: Real-Time Integration
- [ ] Parsing Pipeline implementation
- [ ] FileWatcher integration
- [ ] Real-time graph updates
- [ ] Event handling and conflict resolution
- [ ] End-to-end monitoring tests

### Phase 4: User Interface
- [ ] Repository-aware CLI commands
- [ ] Repository-aware daemon service
- [ ] Configuration management
- [ ] Documentation updates
- [ ] User acceptance testing

## Success Criteria

### Functional Requirements (MUST ACHIEVE)
- ✅ `prism index /path/to/repository` - Scan and index any repository
- ✅ `prism serve /path/to/repository` - Start MCP server with repository
- ✅ MCP client compatibility (Claude Desktop, Cursor, etc.)
- ✅ Real-time file monitoring and graph updates
- ✅ All MCP tools and resources working correctly

### Performance Requirements
- Repository scanning: < 1000 files/second
- File change response: < 100ms to graph update
- Memory usage: < 2GB for 10M nodes
- Query response: < 1s for complex graph queries

### Quality Requirements
- Test coverage: > 85% for new components
- Zero memory leaks in long-running daemon
- Graceful error handling for all operations
- Comprehensive integration tests

## Risk Mitigation

### High-Risk Areas
1. **MCP Protocol Complexity** - Start with minimal compliant implementation
2. **Large Repository Performance** - Implement incremental processing
3. **Real-Time Update Reliability** - Add comprehensive error handling
4. **Integration Complexity** - Build and test incrementally

### Fallback Plans
1. **Performance Issues** - Implement file filtering and batching
2. **Memory Usage** - Add streaming processing and caching limits
3. **MCP Client Incompatibility** - Create compatibility layer
4. **File Monitoring Failures** - Add periodic re-scan fallback

## Conclusion

This action plan addresses the critical gaps identified in the implementation review. By following this phase-based approach and maintaining focus on the core functionality described in the specification document, Prism can achieve full compliance and deliver its promised capabilities.

**Key Success Factors:**
1. **Prioritize Repository Operations** - Enable "point-to-folder" functionality first
2. **Ensure MCP Compliance** - Critical for client integration
3. **Test Incrementally** - Validate each component before integration
4. **Monitor Performance** - Ensure scalability requirements are met

**Phase-based implementation approach ensures systematic delivery of core functionality.** 