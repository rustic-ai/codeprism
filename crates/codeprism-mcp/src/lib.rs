//! # CodePrism MCP Server
//!
//! A Model Context Protocol (MCP) compliant server that provides access to code repositories
//! through standardized Resources, Tools, and Prompts.
//!
//! This implementation follows the MCP specification for JSON-RPC 2.0 communication
//! over stdio transport, enabling integration with MCP clients like Claude Desktop,
//! Cursor, and other AI applications.

use anyhow::Result;

use codeprism_core::{
    ast::{Edge, Language, Node},
    graph::{GraphQuery, GraphStore},
    indexer::BulkIndexer,
    parser::{LanguageParser, ParseContext, ParseResult, ParserEngine},
    repository::RepositoryManager,
    scanner::RepositoryScanner,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub mod config; // Phase 2.2: Advanced configuration system
pub mod context;
pub mod error_handler;
pub mod monitoring; // Phase 2.2: Performance monitoring system
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;
pub mod tools_legacy;
pub mod transport;
pub mod validation; // Phase 2.2: Configuration validation & health checks

// Re-export main types
pub use error_handler::{McpError, McpErrorHandler, McpResult};
pub use protocol::{
    InitializeParams, InitializeResult, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    ServerCapabilities,
};
pub use server::McpServer;
pub use transport::{StdioTransport, Transport};

// Re-export Phase 2.2 types
pub use config::{
    CachingConfig, ConfigProfileManager, McpConfigProfile, MonitoringConfig, SecurityConfig,
};
pub use monitoring::{MonitoringMiddleware, PerformanceMonitor, PerformanceSummary};
pub use tools::dynamic_enablement::{DynamicToolManager, RepositoryAnalysis};
pub use validation::{StartupReport, SystemValidator, ValidationResult};

/// Python language parser adapter
struct PythonParserAdapter;

impl LanguageParser for PythonParserAdapter {
    fn language(&self) -> Language {
        Language::Python
    }

    fn parse(&self, context: &ParseContext) -> codeprism_core::error::Result<ParseResult> {
        // Use the Python parser from codeprism-lang-python
        let python_parser = codeprism_lang_python::PythonLanguageParser::new();

        match codeprism_lang_python::parse_file(
            &python_parser,
            &context.repo_id,
            context.file_path.clone(),
            context.content.clone(),
            context.old_tree.clone(),
        ) {
            Ok((tree, py_nodes, py_edges)) => {
                // Convert Python parser types to codeprism types
                let nodes: Vec<Node> = py_nodes
                    .into_iter()
                    .map(|py_node| {
                        // Convert NodeKind
                        let codeprism_kind = match py_node.kind {
                            codeprism_lang_python::NodeKind::Function => {
                                codeprism_core::ast::NodeKind::Function
                            }
                            codeprism_lang_python::NodeKind::Class => {
                                codeprism_core::ast::NodeKind::Class
                            }
                            codeprism_lang_python::NodeKind::Variable => {
                                codeprism_core::ast::NodeKind::Variable
                            }
                            codeprism_lang_python::NodeKind::Module => {
                                codeprism_core::ast::NodeKind::Module
                            }
                            codeprism_lang_python::NodeKind::Import => {
                                codeprism_core::ast::NodeKind::Import
                            }
                            codeprism_lang_python::NodeKind::Parameter => {
                                codeprism_core::ast::NodeKind::Parameter
                            }
                            codeprism_lang_python::NodeKind::Method => {
                                codeprism_core::ast::NodeKind::Method
                            }
                            codeprism_lang_python::NodeKind::Call => {
                                codeprism_core::ast::NodeKind::Call
                            }
                            codeprism_lang_python::NodeKind::Literal => {
                                codeprism_core::ast::NodeKind::Literal
                            }
                            codeprism_lang_python::NodeKind::Route => {
                                codeprism_core::ast::NodeKind::Route
                            }
                            codeprism_lang_python::NodeKind::SqlQuery => {
                                codeprism_core::ast::NodeKind::SqlQuery
                            }
                            codeprism_lang_python::NodeKind::Event => {
                                codeprism_core::ast::NodeKind::Event
                            }
                            codeprism_lang_python::NodeKind::Unknown => {
                                codeprism_core::ast::NodeKind::Unknown
                            }
                        };

                        // Convert Span
                        let codeprism_span = codeprism_core::ast::Span::new(
                            py_node.span.start_byte,
                            py_node.span.end_byte,
                            py_node.span.start_line,
                            py_node.span.end_line,
                            py_node.span.start_column,
                            py_node.span.end_column,
                        );

                        Node::new(
                            &context.repo_id,
                            codeprism_kind,
                            py_node.name,
                            Language::Python,
                            context.file_path.clone(),
                            codeprism_span,
                        )
                    })
                    .collect();

                let edges: Vec<Edge> = py_edges
                    .into_iter()
                    .map(|py_edge| {
                        // Convert EdgeKind
                        let codeprism_edge_kind = match py_edge.kind {
                            codeprism_lang_python::EdgeKind::Calls => {
                                codeprism_core::ast::EdgeKind::Calls
                            }
                            codeprism_lang_python::EdgeKind::Reads => {
                                codeprism_core::ast::EdgeKind::Reads
                            }
                            codeprism_lang_python::EdgeKind::Writes => {
                                codeprism_core::ast::EdgeKind::Writes
                            }
                            codeprism_lang_python::EdgeKind::Imports => {
                                codeprism_core::ast::EdgeKind::Imports
                            }
                            codeprism_lang_python::EdgeKind::Emits => {
                                codeprism_core::ast::EdgeKind::Emits
                            }
                            codeprism_lang_python::EdgeKind::RoutesTo => {
                                codeprism_core::ast::EdgeKind::RoutesTo
                            }
                            codeprism_lang_python::EdgeKind::Raises => {
                                codeprism_core::ast::EdgeKind::Raises
                            }
                            codeprism_lang_python::EdgeKind::Extends => {
                                codeprism_core::ast::EdgeKind::Extends
                            }
                            codeprism_lang_python::EdgeKind::Implements => {
                                codeprism_core::ast::EdgeKind::Implements
                            }
                        };

                        // Convert NodeIds by using hex representation
                        let codecodeprism_source =
                            codeprism_core::ast::NodeId::from_hex(&py_edge.source.to_hex())
                                .unwrap();
                        let codeprism_target =
                            codeprism_core::ast::NodeId::from_hex(&py_edge.target.to_hex())
                                .unwrap();

                        Edge::new(codecodeprism_source, codeprism_target, codeprism_edge_kind)
                    })
                    .collect();

                Ok(ParseResult { tree, nodes, edges })
            }
            Err(e) => Err(codeprism_core::error::Error::parse(
                &context.file_path,
                format!("Python parsing failed: {}", e),
            )),
        }
    }
}

/// JavaScript language parser adapter
struct JavaScriptParserAdapter;

impl LanguageParser for JavaScriptParserAdapter {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn parse(&self, context: &ParseContext) -> codeprism_core::error::Result<ParseResult> {
        // Use the JavaScript parser from codeprism-lang-js
        let js_parser = codeprism_lang_js::JavaScriptLanguageParser::new();

        match codeprism_lang_js::parse_file(
            &js_parser,
            &context.repo_id,
            context.file_path.clone(),
            context.content.clone(),
            context.old_tree.clone(),
        ) {
            Ok((tree, js_nodes, js_edges)) => {
                // Convert JavaScript parser types to codeprism types
                let nodes: Vec<Node> = js_nodes
                    .into_iter()
                    .map(|js_node| {
                        // Convert NodeKind
                        let codeprism_kind = match js_node.kind {
                            codeprism_lang_js::NodeKind::Function => {
                                codeprism_core::ast::NodeKind::Function
                            }
                            codeprism_lang_js::NodeKind::Class => {
                                codeprism_core::ast::NodeKind::Class
                            }
                            codeprism_lang_js::NodeKind::Variable => {
                                codeprism_core::ast::NodeKind::Variable
                            }
                            codeprism_lang_js::NodeKind::Module => {
                                codeprism_core::ast::NodeKind::Module
                            }
                            codeprism_lang_js::NodeKind::Import => {
                                codeprism_core::ast::NodeKind::Import
                            }
                            codeprism_lang_js::NodeKind::Parameter => {
                                codeprism_core::ast::NodeKind::Parameter
                            }
                            codeprism_lang_js::NodeKind::Method => {
                                codeprism_core::ast::NodeKind::Method
                            }
                            codeprism_lang_js::NodeKind::Call => {
                                codeprism_core::ast::NodeKind::Call
                            }
                            codeprism_lang_js::NodeKind::Literal => {
                                codeprism_core::ast::NodeKind::Literal
                            }
                            codeprism_lang_js::NodeKind::Route => {
                                codeprism_core::ast::NodeKind::Route
                            }
                            codeprism_lang_js::NodeKind::SqlQuery => {
                                codeprism_core::ast::NodeKind::SqlQuery
                            }
                            codeprism_lang_js::NodeKind::Event => {
                                codeprism_core::ast::NodeKind::Event
                            }
                            codeprism_lang_js::NodeKind::Unknown => {
                                codeprism_core::ast::NodeKind::Unknown
                            }
                        };

                        // Convert Span
                        let codeprism_span = codeprism_core::ast::Span::new(
                            js_node.span.start_byte,
                            js_node.span.end_byte,
                            js_node.span.start_line,
                            js_node.span.end_line,
                            js_node.span.start_column,
                            js_node.span.end_column,
                        );

                        Node::new(
                            &context.repo_id,
                            codeprism_kind,
                            js_node.name,
                            Language::JavaScript,
                            context.file_path.clone(),
                            codeprism_span,
                        )
                    })
                    .collect();

                let edges: Vec<Edge> = js_edges
                    .into_iter()
                    .map(|js_edge| {
                        // Convert EdgeKind
                        let codeprism_edge_kind = match js_edge.kind {
                            codeprism_lang_js::EdgeKind::Calls => {
                                codeprism_core::ast::EdgeKind::Calls
                            }
                            codeprism_lang_js::EdgeKind::Reads => {
                                codeprism_core::ast::EdgeKind::Reads
                            }
                            codeprism_lang_js::EdgeKind::Writes => {
                                codeprism_core::ast::EdgeKind::Writes
                            }
                            codeprism_lang_js::EdgeKind::Imports => {
                                codeprism_core::ast::EdgeKind::Imports
                            }
                            codeprism_lang_js::EdgeKind::Emits => {
                                codeprism_core::ast::EdgeKind::Emits
                            }
                            codeprism_lang_js::EdgeKind::RoutesTo => {
                                codeprism_core::ast::EdgeKind::RoutesTo
                            }
                            codeprism_lang_js::EdgeKind::Raises => {
                                codeprism_core::ast::EdgeKind::Raises
                            }
                            codeprism_lang_js::EdgeKind::Extends => {
                                codeprism_core::ast::EdgeKind::Extends
                            }
                            codeprism_lang_js::EdgeKind::Implements => {
                                codeprism_core::ast::EdgeKind::Implements
                            }
                        };

                        // Convert NodeIds by using hex representation
                        let codecodeprism_source =
                            codeprism_core::ast::NodeId::from_hex(&js_edge.source.to_hex())
                                .unwrap();
                        let codeprism_target =
                            codeprism_core::ast::NodeId::from_hex(&js_edge.target.to_hex())
                                .unwrap();

                        Edge::new(codecodeprism_source, codeprism_target, codeprism_edge_kind)
                    })
                    .collect();

                Ok(ParseResult { tree, nodes, edges })
            }
            Err(e) => Err(codeprism_core::error::Error::parse(
                &context.file_path,
                format!("JavaScript parsing failed: {}", e),
            )),
        }
    }
}

/// MCP Server implementation that integrates with CodePrism Phase 2.5 components
pub struct CodePrismMcpServer {
    /// Repository manager from Phase 2.5
    repository_manager: RepositoryManager,
    /// Repository scanner for file discovery
    scanner: RepositoryScanner,
    /// Bulk indexer for processing files
    indexer: BulkIndexer,
    /// Parser engine for language processing
    parser_engine: std::sync::Arc<ParserEngine>,
    /// Graph store for code intelligence
    graph_store: Arc<GraphStore>,
    /// Graph query engine
    graph_query: GraphQuery,
    /// Content search manager for full-text search
    content_search: Arc<codeprism_core::ContentSearchManager>,
    /// Server capabilities
    capabilities: ServerCapabilities,
    /// Current repository path
    repository_path: Option<std::path::PathBuf>,
}

impl CodePrismMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Result<Self> {
        let language_registry =
            std::sync::Arc::new(codeprism_core::parser::LanguageRegistry::new());

        // Register language parsers
        language_registry.register(Arc::new(PythonParserAdapter));
        language_registry.register(Arc::new(JavaScriptParserAdapter));

        let parser_engine = std::sync::Arc::new(ParserEngine::new(language_registry.clone()));
        let repository_manager = RepositoryManager::new(language_registry);
        let scanner = RepositoryScanner::new();
        let indexer = BulkIndexer::new(
            codeprism_core::indexer::IndexingConfig::new("mcp".to_string(), "default".to_string()),
            parser_engine.clone(),
        );

        let graph_store = Arc::new(GraphStore::new());
        let graph_query = GraphQuery::new(graph_store.clone());
        let content_search = Arc::new(codeprism_core::ContentSearchManager::with_graph_store(
            graph_store.clone(),
        ));

        let capabilities = ServerCapabilities {
            resources: Some(resources::ResourceCapabilities {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(tools::ToolCapabilities {
                list_changed: Some(true),
            }),
            prompts: Some(prompts::PromptCapabilities {
                list_changed: Some(false),
            }),
            experimental: Some(HashMap::new()),
        };

        Ok(Self {
            repository_manager,
            scanner,
            indexer,
            parser_engine,
            graph_store,
            graph_query,
            content_search,
            capabilities,
            repository_path: None,
        })
    }

    /// Create a new MCP server instance with custom configuration
    pub fn new_with_config(
        memory_limit_mb: usize,
        batch_size: usize,
        max_file_size_mb: usize,
        disable_memory_limit: bool,
        exclude_dirs: Vec<String>,
        include_extensions: Option<Vec<String>>,
        dependency_mode: Option<String>,
    ) -> Result<Self> {
        let language_registry =
            std::sync::Arc::new(codeprism_core::parser::LanguageRegistry::new());

        // Register language parsers
        language_registry.register(Arc::new(PythonParserAdapter));
        language_registry.register(Arc::new(JavaScriptParserAdapter));

        let parser_engine = std::sync::Arc::new(ParserEngine::new(language_registry.clone()));

        // Parse dependency mode
        let dep_mode = match dependency_mode.as_deref() {
            Some("include_all") => codeprism_core::scanner::DependencyMode::IncludeAll,
            Some("smart") => codeprism_core::scanner::DependencyMode::Smart,
            _ => codeprism_core::scanner::DependencyMode::Exclude,
        };

        // Create repository manager with custom configuration
        let repository_manager = RepositoryManager::new_with_config(
            language_registry,
            Some(exclude_dirs.clone()),
            include_extensions.clone(),
            Some(dep_mode.clone()),
        );

        // Create scanner with custom configuration for direct use
        let scanner = if !exclude_dirs.is_empty() {
            let mut scanner = RepositoryScanner::with_exclude_dirs(exclude_dirs.clone())
                .with_dependency_mode(dep_mode.clone());
            if let Some(ref extensions) = include_extensions {
                scanner = scanner.with_extensions(extensions.clone());
            }
            scanner
        } else {
            RepositoryScanner::new().with_dependency_mode(dep_mode.clone())
        };

        // Create custom indexing config with user settings
        let mut indexing_config =
            codeprism_core::indexer::IndexingConfig::new("mcp".to_string(), "default".to_string());
        indexing_config.batch_size = batch_size;

        if disable_memory_limit {
            indexing_config.memory_limit = None;
            tracing::warn!(
                "Memory limit checking disabled - use with caution for large repositories"
            );
        } else {
            indexing_config.memory_limit = Some(memory_limit_mb * 1024 * 1024); // Convert MB to bytes
        }

        let indexer = BulkIndexer::new(indexing_config, parser_engine.clone());

        let graph_store = Arc::new(GraphStore::new());
        let graph_query = GraphQuery::new(graph_store.clone());
        let content_search = Arc::new(codeprism_core::ContentSearchManager::with_graph_store(
            graph_store.clone(),
        ));

        let capabilities = ServerCapabilities {
            resources: Some(resources::ResourceCapabilities {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(tools::ToolCapabilities {
                list_changed: Some(true),
            }),
            prompts: Some(prompts::PromptCapabilities {
                list_changed: Some(false),
            }),
            experimental: Some(HashMap::new()),
        };

        tracing::info!("MCP server configured with:");
        tracing::info!(
            "  Memory limit: {}MB{}",
            memory_limit_mb,
            if disable_memory_limit {
                " (disabled)"
            } else {
                ""
            }
        );
        tracing::info!("  Batch size: {}", batch_size);
        tracing::info!("  Max file size: {}MB", max_file_size_mb);
        tracing::info!("  Dependency mode: {:?}", dep_mode);
        tracing::info!("  Exclude directories: {:?}", exclude_dirs);
        if let Some(ref exts) = include_extensions {
            tracing::info!("  Include extensions: {:?}", exts);
        }

        Ok(Self {
            repository_manager,
            scanner,
            indexer,
            parser_engine,
            graph_store,
            graph_query,
            content_search,
            capabilities,
            repository_path: None,
        })
    }

    /// Initialize the server with a repository path
    pub async fn initialize_with_repository<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        // Create repository config
        let repo_id = format!(
            "mcp-{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repository")
        );

        let repo_config = codeprism_core::repository::RepositoryConfig::new(repo_id.clone(), &path)
            .with_name(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("repository")
                    .to_string(),
            );

        // Register repository
        self.repository_manager.register_repository(repo_config)?;

        // Perform initial scan and indexing for code symbols
        let indexing_result = self
            .repository_manager
            .index_repository(&repo_id, None)
            .await?;

        // Populate graph store with indexed data
        for patch in &indexing_result.patches {
            for node in &patch.nodes_add {
                self.graph_store.add_node(node.clone());
            }
            for edge in &patch.edges_add {
                self.graph_store.add_edge(edge.clone());
            }
        }

        // Index content for documentation, configuration files, and comments
        self.index_repository_content(&path).await?;

        self.repository_path = Some(path);
        tracing::info!(
            "MCP server initialized with repository: {:?}",
            self.repository_path
        );

        Ok(())
    }

    /// Index repository content including documentation, configuration, and comments
    async fn index_repository_content(&self, repo_path: &Path) -> Result<()> {
        tracing::info!("Starting content indexing for repository: {:?}", repo_path);

        // Discover all files in the repository
        let files = self.scanner.discover_files(repo_path)?;
        let mut indexed_count = 0;
        let mut error_count = 0;

        for file_path in files {
            if let Err(e) = self.index_file_content(&file_path).await {
                tracing::warn!("Failed to index content for {}: {}", file_path.display(), e);
                error_count += 1;
            } else {
                indexed_count += 1;
            }
        }

        tracing::info!(
            "Content indexing completed: {} files indexed, {} errors",
            indexed_count,
            error_count
        );
        Ok(())
    }

    /// Index content for a single file
    async fn index_file_content(&self, file_path: &Path) -> Result<()> {
        // Read file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                // Skip binary files or files that can't be read as text
                return Ok(());
            }
        };

        // Skip empty files
        if content.trim().is_empty() {
            return Ok(());
        }

        let _language = self.detect_language(file_path);

        // Handle different file types appropriately
        // Current implementation uses simple file indexing for all content types
        // Future enhancement: tree-sitter integration for improved parsing
        // to extract comments and provide better source code content indexing
        self.content_search.index_file(file_path, &content)?;

        Ok(())
    }

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Option<codeprism_core::ast::Language> {
        let extension = file_path.extension()?.to_str()?;
        let lang = codeprism_core::ast::Language::from_extension(extension);
        if matches!(lang, codeprism_core::ast::Language::Unknown) {
            None
        } else {
            Some(lang)
        }
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.capabilities
    }

    /// Get repository manager for accessing Phase 2.5 functionality
    pub fn repository_manager(&self) -> &RepositoryManager {
        &self.repository_manager
    }

    /// Get repository scanner
    pub fn scanner(&self) -> &RepositoryScanner {
        &self.scanner
    }

    /// Get bulk indexer
    pub fn indexer(&self) -> &BulkIndexer {
        &self.indexer
    }

    /// Get parser engine
    pub fn parser_engine(&self) -> &std::sync::Arc<ParserEngine> {
        &self.parser_engine
    }

    /// Get graph store
    pub fn graph_store(&self) -> &Arc<GraphStore> {
        &self.graph_store
    }

    /// Get graph query engine
    pub fn graph_query(&self) -> &GraphQuery {
        &self.graph_query
    }

    /// Get content search manager
    pub fn content_search(&self) -> &Arc<codeprism_core::ContentSearchManager> {
        &self.content_search
    }

    /// Get current repository path
    pub fn repository_path(&self) -> Option<&Path> {
        self.repository_path.as_deref()
    }
}

impl Default for CodePrismMcpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default MCP server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = CodePrismMcpServer::new().expect("Failed to create MCP server");

        // Verify capabilities are properly set
        assert!(server.capabilities().resources.is_some());
        assert!(server.capabilities().tools.is_some());
        assert!(server.capabilities().prompts.is_some());

        // Verify no repository is set initially
        assert!(server.repository_path().is_none());
    }

    #[tokio::test]
    async fn test_mcp_server_initialize_with_repository() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Create a test file
        fs::write(repo_path.join("test.py"), "print('hello world')").unwrap();

        let mut server = CodePrismMcpServer::new().expect("Failed to create MCP server");
        server
            .initialize_with_repository(repo_path)
            .await
            .expect("Failed to initialize with repository");

        // Verify repository is set
        assert!(server.repository_path().is_some());
        assert_eq!(server.repository_path().unwrap(), repo_path);
    }

    #[tokio::test]
    async fn test_mcp_server_capabilities() {
        let server = CodePrismMcpServer::new().expect("Failed to create MCP server");
        let capabilities = server.capabilities();

        // Verify resource capabilities
        let resource_caps = capabilities.resources.as_ref().unwrap();
        assert_eq!(resource_caps.subscribe, Some(true));
        assert_eq!(resource_caps.list_changed, Some(true));

        // Verify tool capabilities
        let tool_caps = capabilities.tools.as_ref().unwrap();
        assert_eq!(tool_caps.list_changed, Some(true));

        // Verify prompt capabilities
        let prompt_caps = capabilities.prompts.as_ref().unwrap();
        assert_eq!(prompt_caps.list_changed, Some(false));
    }
}
