//! # Prism MCP Server
//! 
//! A Model Context Protocol (MCP) compliant server that provides access to code repositories
//! through standardized Resources, Tools, and Prompts.
//! 
//! This implementation follows the MCP specification for JSON-RPC 2.0 communication
//! over stdio transport, enabling integration with MCP clients like Claude Desktop,
//! Cursor, and other AI applications.

use anyhow::Result;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use prism_core::{
    repository::RepositoryManager,
    scanner::RepositoryScanner, 
    indexer::BulkIndexer,
    parser::{ParserEngine, LanguageParser, ParseContext, ParseResult},
    graph::{GraphStore, GraphQuery},
    ast::{Language, Node, Edge},
};

pub mod transport;
pub mod protocol;
pub mod resources;
pub mod tools_legacy;
pub mod tools;
pub mod prompts;
pub mod server;

// Re-export main types
pub use server::McpServer;
pub use transport::{StdioTransport, Transport};
pub use protocol::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcNotification,
    InitializeParams, InitializeResult, ServerCapabilities,
};

/// Python language parser adapter
struct PythonParserAdapter;

impl LanguageParser for PythonParserAdapter {
    fn language(&self) -> Language {
        Language::Python
    }

    fn parse(&self, context: &ParseContext) -> prism_core::error::Result<ParseResult> {
        // Use the Python parser from prism-lang-python
        let python_parser = prism_lang_python::PythonLanguageParser::new();
        
        match prism_lang_python::parse_file(
            &python_parser,
            &context.repo_id,
            context.file_path.clone(),
            context.content.clone(),
            context.old_tree.clone(),
        ) {
            Ok((tree, py_nodes, py_edges)) => {
                // Convert Python parser types to prism types
                let nodes: Vec<Node> = py_nodes.into_iter().map(|py_node| {
                    // Convert NodeKind
                    let prism_kind = match py_node.kind {
                        prism_lang_python::NodeKind::Function => prism_core::ast::NodeKind::Function,
                        prism_lang_python::NodeKind::Class => prism_core::ast::NodeKind::Class,
                        prism_lang_python::NodeKind::Variable => prism_core::ast::NodeKind::Variable,
                        prism_lang_python::NodeKind::Module => prism_core::ast::NodeKind::Module,
                        prism_lang_python::NodeKind::Import => prism_core::ast::NodeKind::Import,
                        prism_lang_python::NodeKind::Parameter => prism_core::ast::NodeKind::Parameter,
                        prism_lang_python::NodeKind::Method => prism_core::ast::NodeKind::Method,
                        prism_lang_python::NodeKind::Call => prism_core::ast::NodeKind::Call,
                        prism_lang_python::NodeKind::Literal => prism_core::ast::NodeKind::Literal,
                        prism_lang_python::NodeKind::Route => prism_core::ast::NodeKind::Route,
                        prism_lang_python::NodeKind::SqlQuery => prism_core::ast::NodeKind::SqlQuery,
                        prism_lang_python::NodeKind::Event => prism_core::ast::NodeKind::Event,
                        prism_lang_python::NodeKind::Unknown => prism_core::ast::NodeKind::Unknown,
                    };
                    
                    // Convert Span
                    let prism_span = prism_core::ast::Span::new(
                        py_node.span.start_byte,
                        py_node.span.end_byte,
                        py_node.span.start_line,
                        py_node.span.end_line,
                        py_node.span.start_column,
                        py_node.span.end_column,
                    );
                    
                    Node::new(
                        &context.repo_id,
                        prism_kind,
                        py_node.name,
                        Language::Python,
                        context.file_path.clone(),
                        prism_span,
                    )
                }).collect();

                let edges: Vec<Edge> = py_edges.into_iter().map(|py_edge| {
                    // Convert EdgeKind
                    let prism_edge_kind = match py_edge.kind {
                        prism_lang_python::EdgeKind::Calls => prism_core::ast::EdgeKind::Calls,
                        prism_lang_python::EdgeKind::Reads => prism_core::ast::EdgeKind::Reads,
                        prism_lang_python::EdgeKind::Writes => prism_core::ast::EdgeKind::Writes,
                        prism_lang_python::EdgeKind::Imports => prism_core::ast::EdgeKind::Imports,
                        prism_lang_python::EdgeKind::Emits => prism_core::ast::EdgeKind::Emits,
                        prism_lang_python::EdgeKind::RoutesTo => prism_core::ast::EdgeKind::RoutesTo,
                        prism_lang_python::EdgeKind::Raises => prism_core::ast::EdgeKind::Raises,
                        prism_lang_python::EdgeKind::Extends => prism_core::ast::EdgeKind::Extends,
                        prism_lang_python::EdgeKind::Implements => prism_core::ast::EdgeKind::Implements,
                    };
                    
                    // Convert NodeIds by using hex representation
                    let prism_source = prism_core::ast::NodeId::from_hex(&py_edge.source.to_hex()).unwrap();
                    let prism_target = prism_core::ast::NodeId::from_hex(&py_edge.target.to_hex()).unwrap();
                    
                    Edge::new(prism_source, prism_target, prism_edge_kind)
                }).collect();

                Ok(ParseResult { tree, nodes, edges })
            }
            Err(e) => Err(prism_core::error::Error::parse(&context.file_path, &format!("Python parsing failed: {}", e))),
        }
    }
}

/// JavaScript language parser adapter
struct JavaScriptParserAdapter;

impl LanguageParser for JavaScriptParserAdapter {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn parse(&self, context: &ParseContext) -> prism_core::error::Result<ParseResult> {
        // Use the JavaScript parser from prism-lang-js
        let js_parser = prism_lang_js::JavaScriptLanguageParser::new();
        
        match prism_lang_js::parse_file(
            &js_parser,
            &context.repo_id,
            context.file_path.clone(),
            context.content.clone(),
            context.old_tree.clone(),
        ) {
            Ok((tree, js_nodes, js_edges)) => {
                // Convert JavaScript parser types to prism types
                let nodes: Vec<Node> = js_nodes.into_iter().map(|js_node| {
                    // Convert NodeKind
                    let prism_kind = match js_node.kind {
                        prism_lang_js::NodeKind::Function => prism_core::ast::NodeKind::Function,
                        prism_lang_js::NodeKind::Class => prism_core::ast::NodeKind::Class,
                        prism_lang_js::NodeKind::Variable => prism_core::ast::NodeKind::Variable,
                        prism_lang_js::NodeKind::Module => prism_core::ast::NodeKind::Module,
                        prism_lang_js::NodeKind::Import => prism_core::ast::NodeKind::Import,
                        prism_lang_js::NodeKind::Parameter => prism_core::ast::NodeKind::Parameter,
                        prism_lang_js::NodeKind::Method => prism_core::ast::NodeKind::Method,
                        prism_lang_js::NodeKind::Call => prism_core::ast::NodeKind::Call,
                        prism_lang_js::NodeKind::Literal => prism_core::ast::NodeKind::Literal,
                        prism_lang_js::NodeKind::Route => prism_core::ast::NodeKind::Route,
                        prism_lang_js::NodeKind::SqlQuery => prism_core::ast::NodeKind::SqlQuery,
                        prism_lang_js::NodeKind::Event => prism_core::ast::NodeKind::Event,
                        prism_lang_js::NodeKind::Unknown => prism_core::ast::NodeKind::Unknown,
                    };
                    
                    // Convert Span
                    let prism_span = prism_core::ast::Span::new(
                        js_node.span.start_byte,
                        js_node.span.end_byte,
                        js_node.span.start_line,
                        js_node.span.end_line,
                        js_node.span.start_column,
                        js_node.span.end_column,
                    );
                    
                    Node::new(
                        &context.repo_id,
                        prism_kind,
                        js_node.name,
                        Language::JavaScript,
                        context.file_path.clone(),
                        prism_span,
                    )
                }).collect();

                let edges: Vec<Edge> = js_edges.into_iter().map(|js_edge| {
                    // Convert EdgeKind
                    let prism_edge_kind = match js_edge.kind {
                        prism_lang_js::EdgeKind::Calls => prism_core::ast::EdgeKind::Calls,
                        prism_lang_js::EdgeKind::Reads => prism_core::ast::EdgeKind::Reads,
                        prism_lang_js::EdgeKind::Writes => prism_core::ast::EdgeKind::Writes,
                        prism_lang_js::EdgeKind::Imports => prism_core::ast::EdgeKind::Imports,
                        prism_lang_js::EdgeKind::Emits => prism_core::ast::EdgeKind::Emits,
                        prism_lang_js::EdgeKind::RoutesTo => prism_core::ast::EdgeKind::RoutesTo,
                        prism_lang_js::EdgeKind::Raises => prism_core::ast::EdgeKind::Raises,
                        prism_lang_js::EdgeKind::Extends => prism_core::ast::EdgeKind::Extends,
                        prism_lang_js::EdgeKind::Implements => prism_core::ast::EdgeKind::Implements,
                    };
                    
                    // Convert NodeIds by using hex representation
                    let prism_source = prism_core::ast::NodeId::from_hex(&js_edge.source.to_hex()).unwrap();
                    let prism_target = prism_core::ast::NodeId::from_hex(&js_edge.target.to_hex()).unwrap();
                    
                    Edge::new(prism_source, prism_target, prism_edge_kind)
                }).collect();

                Ok(ParseResult { tree, nodes, edges })
            }
            Err(e) => Err(prism_core::error::Error::parse(&context.file_path, &format!("JavaScript parsing failed: {}", e))),
        }
    }
}

/// MCP Server implementation that integrates with Prism Phase 2.5 components
pub struct PrismMcpServer {
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
    content_search: Arc<prism_core::ContentSearchManager>,
    /// Server capabilities
    capabilities: ServerCapabilities,
    /// Current repository path
    repository_path: Option<std::path::PathBuf>,
}

impl PrismMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Result<Self> {
        let language_registry = std::sync::Arc::new(prism_core::parser::LanguageRegistry::new());
        
        // Register language parsers
        language_registry.register(Arc::new(PythonParserAdapter));
        language_registry.register(Arc::new(JavaScriptParserAdapter));
        
        let parser_engine = std::sync::Arc::new(ParserEngine::new(language_registry.clone()));
        let repository_manager = RepositoryManager::new(language_registry);
        let scanner = RepositoryScanner::new();
        let indexer = BulkIndexer::new(
            prism_core::indexer::IndexingConfig::new("mcp".to_string(), "default".to_string()),
            parser_engine.clone()
        );

        let graph_store = Arc::new(GraphStore::new());
        let graph_query = GraphQuery::new(graph_store.clone());
        let content_search = Arc::new(prism_core::ContentSearchManager::with_graph_store(graph_store.clone()));

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
        let language_registry = std::sync::Arc::new(prism_core::parser::LanguageRegistry::new());
        
        // Register language parsers
        language_registry.register(Arc::new(PythonParserAdapter));
        language_registry.register(Arc::new(JavaScriptParserAdapter));
        
        let parser_engine = std::sync::Arc::new(ParserEngine::new(language_registry.clone()));
        
        // Parse dependency mode
        let dep_mode = match dependency_mode.as_deref() {
            Some("include_all") => prism_core::scanner::DependencyMode::IncludeAll,
            Some("smart") => prism_core::scanner::DependencyMode::Smart,
            _ => prism_core::scanner::DependencyMode::Exclude,
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
        let mut indexing_config = prism_core::indexer::IndexingConfig::new("mcp".to_string(), "default".to_string());
        indexing_config.batch_size = batch_size;
        
        if disable_memory_limit {
            indexing_config.memory_limit = None;
            tracing::warn!("Memory limit checking disabled - use with caution for large repositories");
        } else {
            indexing_config.memory_limit = Some(memory_limit_mb * 1024 * 1024); // Convert MB to bytes
        }

        let indexer = BulkIndexer::new(indexing_config, parser_engine.clone());

        let graph_store = Arc::new(GraphStore::new());
        let graph_query = GraphQuery::new(graph_store.clone());
        let content_search = Arc::new(prism_core::ContentSearchManager::with_graph_store(graph_store.clone()));

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
        tracing::info!("  Memory limit: {}MB{}", memory_limit_mb, if disable_memory_limit { " (disabled)" } else { "" });
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
        let repo_id = format!("mcp-{}", path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository"));
        
        let repo_config = prism_core::repository::RepositoryConfig::new(repo_id.clone(), &path)
            .with_name(path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repository")
                .to_string());
        
        // Register repository
        self.repository_manager.register_repository(repo_config)?;
        
        // Perform initial scan and indexing for code symbols
        let indexing_result = self.repository_manager
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
        tracing::info!("MCP server initialized with repository: {:?}", self.repository_path);
        
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
        
        tracing::info!("Content indexing completed: {} files indexed, {} errors", indexed_count, error_count);
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
        // For now, use simple file indexing for all content types
        // TODO: In the future, we can enhance this with tree-sitter integration
        // to extract comments and provide better source code content indexing
        self.content_search.index_file(file_path, &content)?;
        
        Ok(())
    }
    
    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Option<prism_core::ast::Language> {
        let extension = file_path.extension()?.to_str()?;
        let lang = prism_core::ast::Language::from_extension(extension);
        if matches!(lang, prism_core::ast::Language::Unknown) {
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
    pub fn content_search(&self) -> &Arc<prism_core::ContentSearchManager> {
        &self.content_search
    }

    /// Get current repository path
    pub fn repository_path(&self) -> Option<&Path> {
        self.repository_path.as_deref()
    }
}

impl Default for PrismMcpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default MCP server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = PrismMcpServer::new().expect("Failed to create MCP server");
        
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
        
        let mut server = PrismMcpServer::new().expect("Failed to create MCP server");
        server.initialize_with_repository(repo_path).await
            .expect("Failed to initialize with repository");
        
        // Verify repository is set
        assert!(server.repository_path().is_some());
        assert_eq!(server.repository_path().unwrap(), repo_path);
    }

    #[tokio::test]
    async fn test_mcp_server_capabilities() {
        let server = PrismMcpServer::new().expect("Failed to create MCP server");
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