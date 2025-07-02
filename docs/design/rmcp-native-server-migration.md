# RMCP Native Server Migration Design Document

## Executive Summary

This document outlines the complete migration from our custom MCP implementation to the official Rust SDK (RMCP) for the Model Context Protocol. The migration eliminates technical debt from our custom JSON-RPC implementation and adopts the official toolbox patterns with a **modular router architecture** for maximum maintainability and scalability.

## Background

### Current State Issues
- **Custom MCP Implementation**: Manual JSON-RPC 2.0 handling with 4,000+ lines of custom protocol code
- **Technical Debt**: Bridge patterns between custom MCP and modular tools via `ToolManager`
- **Maintenance Burden**: Custom serialization, transport handling, and protocol compliance
- **Testing Complexity**: Manual protocol testing instead of using official SDK compliance
- **Monolithic Tool Management**: All tools managed through single ToolManager bridge

### Target State Benefits
- **Official SDK**: Native RMCP with automatic protocol compliance
- **Modular Router Architecture**: Separate routers per tool category with composition
- **Transport Abstraction**: Built-in stdio, SSE, and other transport support
- **Schema Generation**: Automatic OpenAPI-style schemas from Rust types
- **Feature Flags**: Conditional compilation of tool categories
- **Independent Development**: Teams can work on tool categories in parallel
- **Reduced Codebase**: ~4,000 lines of custom code eliminated

## RMCP SDK Architecture Overview

### Core Components

```rust
// 1. Category-Based Tool Routers
#[derive(Clone)]
pub struct BasicToolsRouter {
    server: Arc<RwLock<CodePrismMcpServer>>,
}

#[tool_router(router = basic_tools_router, vis = pub)]
impl BasicToolsRouter {
    #[tool(description = "Get repository statistics")]
    pub async fn repository_stats(&self) -> Result<CallToolResult, McpError> {
        // Implementation
    }
}

// 2. Main Server with Combined Routers
#[derive(Clone)]
pub struct CodePrismRmcpServer {
    core_server: Arc<RwLock<CodePrismMcpServer>>,
    combined_router: ToolRouter<Self>,
    config: Arc<ServerConfig>,
}

impl CodePrismRmcpServer {
    pub fn new() -> Result<Self> {
        // Combine category routers
        let combined_router = BasicToolsRouter::basic_tools_router()
            + AnalysisToolsRouter::analysis_tools_router()
            + QualityToolsRouter::quality_tools_router()
            + SearchToolsRouter::search_tools_router()
            + WorkflowToolsRouter::workflow_tools_router();
            
        Ok(Self { core_server, combined_router, config })
    }
}

// 3. Server Handler with Combined Router
#[tool_handler(router = self.combined_router)]
impl ServerHandler for CodePrismRmcpServer {
    fn get_info(&self) -> ServerInfo { ... }
    // call_tool and list_tools auto-generated
}
```

### Key RMCP Patterns

#### 1. Modular Router Pattern
```rust
// Official SDK router combination syntax
let combined_router = router_a + router_b + router_c;

// Each category as independent router
#[tool_router(router = category_router, vis = pub)]
impl CategoryRouter {
    #[tool(description = "Tool description")]
    pub async fn tool_name(&self, params...) -> Result<CallToolResult, McpError> {
        // Implementation calling modular tools
    }
}
```

#### 2. Transport Abstraction
```rust
// Main function transport handling remains the same
match transport.as_str() {
    "stdio" => {
        server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
    }
    "sse" => {
        let sse_server = SseServer::serve(address.parse()?).await?
            .with_service(move || server.clone());
    }
}
```

#### 3. Feature-Based Conditional Compilation
```rust
// Cargo.toml features for tool categories
[features]
default = ["basic-tools", "analysis-tools"]
basic-tools = []
analysis-tools = []
quality-tools = []

// Conditional router inclusion
#[cfg(feature = "basic-tools")]
{ combined_router = combined_router + BasicToolsRouter::basic_tools_router(); }
```

## Detailed Migration Plan

### Phase 1: Tool Category Infrastructure

#### 1.1 Tool Category Structure
```
src/tools/
├── basic/
│   ├── mod.rs              # BasicToolsRouter definition
│   ├── repository.rs       # Existing repository tools (no changes)
│   ├── search.rs          # Existing search tools (no changes)
│   └── files.rs           # Existing file tools (no changes)
├── analysis/
│   ├── mod.rs              # AnalysisToolsRouter definition
│   ├── complexity.rs       # Existing complexity tools (no changes)
│   └── patterns.rs         # Existing pattern tools (no changes)
├── quality/
│   ├── mod.rs              # QualityToolsRouter definition
│   ├── security.rs         # Existing security tools (no changes)
│   └── duplicates.rs       # Existing duplicate tools (no changes)
├── search/
│   ├── mod.rs              # SearchToolsRouter definition
│   └── advanced.rs         # Existing advanced search tools (no changes)
├── workflow/
│   ├── mod.rs              # WorkflowToolsRouter definition
│   └── development.rs      # Existing workflow tools (no changes)
└── mod.rs                  # Tool category exports (removes ToolManager)
```

#### 1.2 Basic Tools Router Implementation
```rust
// src/tools/basic/mod.rs
use crate::CodePrismMcpServer;
use rmcp::{tool, tool_router, model::*, Error as McpError};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod repository;
pub mod search;
pub mod files;

#[derive(Clone)]
pub struct BasicToolsRouter {
    server: Arc<RwLock<CodePrismMcpServer>>,
}

#[tool_router(router = basic_tools_router, vis = pub)]
impl BasicToolsRouter {
    pub fn new(server: Arc<RwLock<CodePrismMcpServer>>) -> Self {
        Self { server }
    }

    #[tool(description = "Get comprehensive repository statistics including file counts, languages, and project structure")]
    pub async fn repository_stats(&self) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        let stats = repository::repository_stats(&server).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&stats)?
        )]))
    }

    #[tool(description = "Get content analysis statistics including size metrics and type distributions")]
    pub async fn content_stats(&self) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        let stats = repository::content_stats(&server).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&stats)?
        )]))
    }

    #[tool(description = "Search for symbols with filtering options by type, scope, and documentation")]
    pub async fn search_symbols(
        &self,
        #[tool(param)]
        #[schemars(description = "Pattern to search for in symbol names")]
        pattern: String,
        
        #[tool(param)]
        #[schemars(description = "Filter by symbol types: 'function', 'class', 'variable', 'module'")]
        symbol_types: Option<Vec<String>>,
        
        #[tool(param)]
        #[schemars(description = "Maximum number of results (1-1000)", minimum = 1, maximum = 1000)]
        limit: Option<u32>,
        
        #[tool(param)]
        #[schemars(description = "Include symbol documentation in results")]
        include_docs: Option<bool>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        
        let search_params = search::SymbolSearchParams {
            pattern,
            symbol_types: symbol_types.unwrap_or_default(),
            limit: limit.unwrap_or(100) as usize,
            include_docs: include_docs.unwrap_or(false),
        };
        
        let results = search::search_symbols(&server, search_params).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
            
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results)?
        )]))
    }

    #[tool(description = "Search file contents with regex support and context")]
    pub async fn search_content(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query (supports regex patterns)")]
        query: String,
        
        #[tool(param)]
        #[schemars(description = "Maximum number of results", minimum = 1, maximum = 1000)]
        limit: Option<u32>,
        
        #[tool(param)]
        #[schemars(description = "File patterns to include (e.g., '*.rs', '*.py')")]
        include_patterns: Option<Vec<String>>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        
        let search_params = search::ContentSearchParams {
            query,
            limit: limit.unwrap_or(100) as usize,
            include_patterns: include_patterns.unwrap_or_default(),
        };
        
        let results = search::search_content(&server, search_params).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
            
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results)?
        )]))
    }

    #[tool(description = "Find files by name or path pattern with glob support")]
    pub async fn find_files(
        &self,
        #[tool(param)]
        #[schemars(description = "File name or path pattern (supports glob syntax)")]
        pattern: String,
        
        #[tool(param)]
        #[schemars(description = "Maximum number of results", minimum = 1, maximum = 1000)]
        limit: Option<u32>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        
        let results = files::find_files(&server, &pattern, limit.unwrap_or(100) as usize).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
            
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results)?
        )]))
    }

    #[tool(description = "Find references to a symbol across the codebase")]
    pub async fn find_references(
        &self,
        #[tool(param)]
        #[schemars(description = "Symbol identifier to find references for")]
        symbol_id: String,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        
        // Try indexed search first, fall back to text search
        if let Ok(result) = search::find_references_indexed(&server, &symbol_id).await {
            return Ok(CallToolResult::success(vec![Content::text(result)]));
        }
        
        if let Ok(result) = search::find_references_grep(&server, &symbol_id).await {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Found via text search (index unavailable): {}", result)
            )]));
        }
        
        Ok(CallToolResult::error(vec![Content::text(
            "Unable to find references: symbol not found and search failed"
        )]))
    }

    #[tool(description = "Get detailed explanation of a symbol including definition, usage, and context")]
    pub async fn explain_symbol(
        &self,
        #[tool(param)]
        #[schemars(description = "Symbol identifier to explain")]
        symbol_id: String,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        
        let explanation = search::explain_symbol(&server, &symbol_id).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
            
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&explanation)?
        )]))
    }
}
```

### Phase 2: Category Router Implementation

#### 2.1 Analysis Tools Router
```rust
// src/tools/analysis/mod.rs
#[derive(Clone)]
pub struct AnalysisToolsRouter {
    server: Arc<RwLock<CodePrismMcpServer>>,
}

#[tool_router(router = analysis_tools_router, vis = pub)]
impl AnalysisToolsRouter {
    pub fn new(server: Arc<RwLock<CodePrismMcpServer>>) -> Self {
        Self { server }
    }

    #[tool(description = "Analyze code complexity metrics including cyclomatic complexity, cognitive complexity, and maintainability index")]
    pub async fn analyze_complexity(
        &self,
        #[tool(param)]
        #[schemars(description = "Target path to analyze (file or directory)")]
        target: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        let result = complexity::analyze_complexity(&server, target).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)?
        )]))
    }

    #[tool(description = "Detect design patterns and architectural patterns in the codebase")]
    pub async fn detect_patterns(
        &self,
        #[tool(param)]
        #[schemars(description = "Scope of analysis: 'project', 'directory', or specific path")]
        scope: Option<String>,
        
        #[tool(param)]
        #[schemars(description = "Pattern types to detect: 'creational', 'structural', 'behavioral'")]
        pattern_types: Option<Vec<String>>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        let result = patterns::detect_patterns(&server, scope, pattern_types).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)?
        )]))
    }

    #[tool(description = "Analyze project dependencies including transitive dependencies and version conflicts")]
    pub async fn analyze_dependencies(
        &self,
        #[tool(param)]
        #[schemars(description = "Target to analyze: 'direct', 'transitive', or 'all'")]
        target: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let server = self.server.read().await;
        let result = dependencies::analyze_dependencies(&server, target).await
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)?
        )]))
    }
}
```

#### 2.2 Similar Pattern for Other Categories
```rust
// src/tools/quality/mod.rs - QualityToolsRouter
// src/tools/search/mod.rs - SearchToolsRouter  
// src/tools/workflow/mod.rs - WorkflowToolsRouter
```

### Phase 3: Main Server Integration

#### 3.1 Combined Router Server
```rust
// src/lib.rs or src/server.rs
use crate::tools::{
    basic::BasicToolsRouter,
    analysis::AnalysisToolsRouter,
    quality::QualityToolsRouter,
    search::SearchToolsRouter,
    workflow::WorkflowToolsRouter,
};

#[derive(Clone)]
pub struct CodePrismRmcpServer {
    /// Core CodePrism functionality
    core_server: Arc<RwLock<CodePrismMcpServer>>,
    
    /// Combined tool router from all categories
    combined_router: ToolRouter<Self>,
    
    /// Current repository path for tools
    repository_path: Option<std::path::PathBuf>,
    
    /// Configuration and settings
    config: Arc<ServerConfig>,
}

impl CodePrismRmcpServer {
    pub fn new() -> Result<Self> {
        let core_server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
        
        // Create category routers with shared server instance
        let mut combined_router = ToolRouter::default();
        
        #[cfg(feature = "basic-tools")]
        {
            combined_router = combined_router + BasicToolsRouter::basic_tools_router();
        }
        
        #[cfg(feature = "analysis-tools")]
        {
            combined_router = combined_router + AnalysisToolsRouter::analysis_tools_router();
        }
        
        #[cfg(feature = "quality-tools")]
        {
            combined_router = combined_router + QualityToolsRouter::quality_tools_router();
        }
        
        #[cfg(feature = "search-tools")]
        {
            combined_router = combined_router + SearchToolsRouter::search_tools_router();
        }
        
        #[cfg(feature = "workflow-tools")]
        {
            combined_router = combined_router + WorkflowToolsRouter::workflow_tools_router();
        }
        
        Ok(Self {
            core_server,
            combined_router,
            repository_path: None,
            config: Arc::new(ServerConfig::default()),
        })
    }
    
    pub async fn initialize_repository<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        {
            let mut server = self.core_server.write().await;
            server.initialize_with_repository(&path).await?;
        }
        self.repository_path = Some(path);
        Ok(())
    }
}

#[tool_handler(router = self.combined_router)]
impl ServerHandler for CodePrismRmcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("CodePrism MCP Server with modular tool categories: basic (repository analysis), analysis (complexity, patterns), quality (security, duplicates), search (advanced search), and workflow (development tools)".to_string()),
        }
    }
}
```

#### 3.2 Feature Configuration
```toml
# Cargo.toml
[features]
default = ["basic-tools", "analysis-tools"]
basic-tools = []
analysis-tools = []
quality-tools = []
search-tools = []
workflow-tools = []
all-tools = ["basic-tools", "analysis-tools", "quality-tools", "search-tools", "workflow-tools"]
```

### Phase 4: Transport and Main Function

#### 4.1 Main Function (Unchanged Pattern)
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    
    // Create server instance with combined routers
    let mut server = CodePrismRmcpServer::new()?;
    
    // Initialize with repository if provided
    if let Some(repo_path) = args.repository {
        server.initialize_repository(&repo_path).await?;
        tracing::info!("Initialized with repository: {}", repo_path.display());
    }
    
    // Handle transport (same as before)
    match args.transport.as_str() {
        "stdio" => {
            tracing::info!("Starting RMCP server with stdio transport");
            let service = server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
            service.waiting().await?;
        }
        "sse" => {
            tracing::info!("Starting RMCP server with SSE transport on {}", args.address);
            let sse_server = SseServer::serve(args.address.parse()?)
                .await?
                .with_service(move || server.clone());
            
            tokio::signal::ctrl_c().await?;
            sse_server.cancel();
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid transport: {}", args.transport));
        }
    }
    
    Ok(())
}
```

## Testing Strategy

### Category-Level Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_tools_category() {
        let server = Arc::new(RwLock::new(CodePrismMcpServer::new().unwrap()));
        let basic_tools = BasicToolsRouter::new(server);
        
        // Test individual tools in category
        let result = basic_tools.repository_stats().await.unwrap();
        assert!(result.is_success());
        
        let result = basic_tools.search_symbols(
            "test".to_string(), 
            Some(vec!["function".to_string()]), 
            Some(10),
            Some(true)
        ).await.unwrap();
        assert!(result.is_success());
    }
    
    #[tokio::test]
    async fn test_analysis_tools_category() {
        let server = Arc::new(RwLock::new(CodePrismMcpServer::new().unwrap()));
        let analysis_tools = AnalysisToolsRouter::new(server);
        
        let result = analysis_tools.analyze_complexity(None).await.unwrap();
        assert!(result.is_success());
    }
    
    #[tokio::test]
    async fn test_combined_router_integration() {
        let server = CodePrismRmcpServer::new().unwrap();
        
        // Test that all category tools are available
        let tools = server.combined_router.list_all();
        assert!(tools.len() > 10); // Should have tools from all categories
        
        // Verify specific tools from different categories exist
        assert!(tools.iter().any(|t| t.name == "repository_stats"));
        assert!(tools.iter().any(|t| t.name == "analyze_complexity"));
        assert!(tools.iter().any(|t| t.name == "find_duplicates"));
    }
}
```

### Integration Testing
```rust
use rmcp::transport::test::{TestClient, TestServer};

#[tokio::test]
async fn test_modular_mcp_protocol_compliance() {
    let server = CodePrismRmcpServer::new().unwrap();
    let (client, server_task) = TestClient::new(server).await.unwrap();
    
    // Test server info includes all categories
    let info = client.get_server_info().await.unwrap();
    assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    
    // Test tool listing shows tools from all categories
    let tools = client.list_tools().await.unwrap();
    assert!(tools.tools.len() > 15); // Should have tools from all categories
    
    // Test tools from different categories work
    let basic_result = client.call_tool("repository_stats", None).await.unwrap();
    assert!(basic_result.is_success());
    
    let analysis_result = client.call_tool("analyze_complexity", None).await.unwrap();
    assert!(analysis_result.is_success());
    
    server_task.abort();
}
```

## Migration Execution Plan

### Week 1: Category Infrastructure
1. **Create tool category structure** with separate mod.rs files
2. **Implement BasicToolsRouter** with 3 core tools (repository_stats, search_symbols, find_files)
3. **Test router combination** mechanism with basic + analysis stub
4. **Update main server** to use combined router pattern

### Week 2: Core Category Migration  
1. **Complete BasicToolsRouter** (all basic/ tools)
2. **Implement AnalysisToolsRouter** (complexity, patterns, dependencies)
3. **Add feature flags** for conditional compilation
4. **Integration testing** between categories

### Week 3: Remaining Categories
1. **Implement QualityToolsRouter** (security, duplicates, unused code)
2. **Implement SearchToolsRouter** (advanced search, traces, references)
3. **Implement WorkflowToolsRouter** (development workflow tools)
4. **Comprehensive testing** of all categories

### Week 4: Polish & Optimization
1. **Performance optimization** of router combination
2. **End-to-end testing** with real MCP clients
3. **Documentation** of category architecture
4. **Remove ToolManager** and legacy custom MCP code

## Advantages of Modular Router Architecture

### Development Benefits
- **Parallel Development**: Teams can work on different categories simultaneously
- **Independent Testing**: Each category can be tested in isolation
- **Feature Flags**: Enable/disable tool categories based on deployment needs
- **Reusability**: Tool categories can be used in other projects

### Maintenance Benefits  
- **Separation of Concerns**: Each category has focused responsibility
- **Easier Debugging**: Issues can be isolated to specific tool categories
- **Incremental Updates**: Categories can be updated independently
- **Cleaner Code**: Smaller, focused router implementations

### Scalability Benefits
- **Conditional Compilation**: Only include needed tool categories
- **Memory Efficiency**: Unused categories don't consume resources
- **Performance**: Tool routing is distributed across category routers
- **Extensibility**: New tool categories can be added easily

## Success Criteria

### Functional Requirements
- ✅ All 20+ tools working with native RMCP through category routers
- ✅ Protocol compliance with MCP specification  
- ✅ Router combination working seamlessly
- ✅ Feature flags enabling selective tool categories
- ✅ Independent category testing and validation

### Quality Requirements  
- ✅ <5ms tool response times for simple tools
- ✅ <100ms for complex analysis tools
- ✅ 90%+ test coverage for each tool category
- ✅ Zero memory leaks in long-running scenarios
- ✅ Clean router combination without conflicts

### Architecture Requirements
- ✅ ToolManager eliminated completely
- ✅ 4,000+ lines of custom MCP code removed
- ✅ Modular tools integrated directly through category routers
- ✅ Category-based feature flag system functional
- ✅ Independent development workflow established

## Risk Mitigation

### Technical Risks
- **Router combination complexity**: Start with 2 categories, add incrementally
- **Parameter serialization**: Use consistent serde-compatible types across categories
- **Performance regression**: Benchmark category router vs monolithic approach
- **Tool naming conflicts**: Implement category-prefixed tool names if needed

### Operational Risks
- **Category interdependencies**: Design categories to be independent
- **Feature flag testing**: Test all feature flag combinations in CI/CD
- **Migration coordination**: Migrate categories in dependency order
- **Deployment complexity**: Maintain backward compatibility during transition

This modular router architecture provides a scalable, maintainable foundation for the CodePrism MCP server while leveraging the full power of the official RMCP SDK. 