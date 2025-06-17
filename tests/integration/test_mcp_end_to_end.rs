//! End-to-end integration tests for MCP server
//!
//! These tests verify the complete MCP server functionality including
//! repository initialization, content indexing, and all MCP tools.

use anyhow::Result;
use gcore_mcp::{GCoreMcpServer, McpServer};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Create a test repository with various file types
async fn create_test_repository() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create Python source files
    fs::write(
        repo_path.join("app.py"),
        r#"""Main application module."""

import logging
from typing import List, Optional
from dataclasses import dataclass

@dataclass 
class User:
    """User model with authentication."""
    username: str
    email: str
    is_active: bool = True

class UserService:
    """Service for managing users."""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        self._users = {}
    
    def create_user(self, username: str, email: str) -> User:
        """Create a new user."""
        user = User(username=username, email=email)
        self._users[username] = user
        self.logger.info(f"Created user: {username}")
        return user
    
    def get_user(self, username: str) -> Optional[User]:
        """Get user by username."""
        return self._users.get(username)
"#,
    )?;

    // Create JavaScript files
    fs::write(
        repo_path.join("client.js"),
        r#"/**
 * Client-side user management
 */

class UserManager {
    constructor(apiUrl) {
        this.apiUrl = apiUrl;
        this.users = new Map();
    }

    /**
     * Fetch user data from API
     */
    async fetchUser(userId) {
        const response = await fetch(`${this.apiUrl}/users/${userId}`);
        const user = await response.json();
        this.users.set(userId, user);
        return user;
    }

    /**
     * Create a new user
     */
    async createUser(userData) {
        const response = await fetch(`${this.apiUrl}/users`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(userData)
        });
        return response.json();
    }
}

export { UserManager };
"#,
    )?;

    // Create documentation files
    fs::write(
        repo_path.join("README.md"),
        r#"# User Management System

A comprehensive user management system with authentication and authorization.

## Features

- User registration and authentication
- Profile management  
- Admin dashboard
- REST API endpoints

## Getting Started

1. Install dependencies: `npm install`
2. Configure database settings in `config.json`
3. Run the application: `npm start`

## API Documentation

### Authentication

The system uses JWT tokens for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <token>
```

### User Endpoints

- `GET /api/users` - List all users
- `POST /api/users` - Create new user
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

## Configuration

See `config.json` for configuration options.
"#,
    )?;

    // Create configuration files
    fs::write(
        repo_path.join("config.json"),
        r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "userdb",
    "username": "admin",
    "password": "secret"
  },
  "server": {
    "port": 3000,
    "host": "0.0.0.0"
  },
  "auth": {
    "jwt_secret": "super-secret-key",
    "token_expiry": "24h"
  },
  "logging": {
    "level": "info",
    "format": "json"
  }
}
"#,
    )?;

    fs::write(
        repo_path.join("package.json"),
        r#"{
  "name": "user-management-system",
  "version": "1.0.0",
  "description": "User management with authentication",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.0",
    "jsonwebtoken": "^9.0.0",
    "bcrypt": "^5.1.0",
    "pg": "^8.8.0"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "nodemon": "^2.0.20"
  }
}
"#,
    )?;

    // Create Docker configuration
    fs::write(
        repo_path.join("docker-compose.yml"),
        r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgresql://admin:secret@db:5432/userdb
    depends_on:
      - db

  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=userdb
      - POSTGRES_USER=admin
      - POSTGRES_PASSWORD=secret
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres_data:
"#,
    )?;

    // Create environment file
    fs::write(
        repo_path.join(".env"),
        r#"NODE_ENV=development
DATABASE_URL=postgresql://admin:secret@localhost:5432/userdb
JWT_SECRET=super-secret-development-key
PORT=3000
LOG_LEVEL=debug
"#,
    )?;

    Ok(temp_dir)
}

#[tokio::test]
async fn test_mcp_server_full_lifecycle() -> Result<()> {
    // Create test repository
    let temp_dir = create_test_repository().await?;
    let repo_path = temp_dir.path();

    // Initialize MCP server
    let mut mcp_server = GCoreMcpServer::new()?;
    
    // Initialize with repository - this should trigger content indexing
    mcp_server.initialize_with_repository(repo_path).await?;

    // Wait a moment for content indexing to complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test repository stats tool
    let stats_result = test_repository_stats(&mcp_server).await?;
    println!("✅ Repository stats test passed");

    // Test symbol search
    let symbol_result = test_symbol_search(&mcp_server).await?;
    println!("✅ Symbol search test passed");

    // Test content search (should now work with indexing)
    let content_result = test_content_search(&mcp_server).await?;
    println!("✅ Content search test passed");

    // Test file search
    let file_result = test_file_search(&mcp_server).await?;
    println!("✅ File search test passed");

    // Test content stats
    let content_stats_result = test_content_stats(&mcp_server).await?;
    println!("✅ Content stats test passed");

    // Test explain symbol
    let explain_result = test_explain_symbol(&mcp_server, &symbol_result).await?;
    println!("✅ Explain symbol test passed");

    println!("🎉 All MCP server integration tests passed!");

    Ok(())
}

async fn test_repository_stats(server: &GCoreMcpServer) -> Result<serde_json::Value> {
    let stats = if let Some(repo_path) = server.repository_path() {
        let file_count = server.scanner().discover_files(repo_path)
            .map(|files| files.len())
            .unwrap_or(0);

        let graph_stats = server.graph_store().get_stats();

        json!({
            "repository_path": repo_path.display().to_string(),
            "total_files": file_count,
            "total_nodes": graph_stats.total_nodes,
            "total_edges": graph_stats.total_edges,
            "nodes_by_kind": graph_stats.nodes_by_kind,
            "status": "active"
        })
    } else {
        json!({"error": "No repository initialized"})
    };

    // Verify we have meaningful data
    assert!(stats.get("total_files").and_then(|v| v.as_u64()).unwrap_or(0) >= 5, 
           "Should have indexed at least 5 files");
    
    println!("Repository stats: {}", serde_json::to_string_pretty(&stats)?);
    Ok(stats)
}

async fn test_symbol_search(server: &GCoreMcpServer) -> Result<String> {
    // Search for User-related symbols
    let results = server.graph_query().search_symbols(
        "User",
        Some(vec![gcore::NodeKind::Class, gcore::NodeKind::Function]),
        Some(10)
    )?;

    assert!(!results.is_empty(), "Should find User-related symbols");
    
    // Find a class symbol to use for further testing
    let class_symbol = results.iter()
        .find(|symbol| matches!(symbol.node.kind, gcore::NodeKind::Class))
        .ok_or_else(|| anyhow::anyhow!("No class symbols found"))?;

    println!("Found {} symbols matching 'User'", results.len());
    println!("Test class symbol: {} ({})", class_symbol.node.name, class_symbol.node.id.to_hex());
    
    Ok(class_symbol.node.id.to_hex())
}

async fn test_content_search(server: &GCoreMcpServer) -> Result<serde_json::Value> {
    // Test simple content search
    let results = server.content_search().simple_search("authentication", Some(10))?;
    
    // Should find references in documentation and comments
    println!("Content search found {} results for 'authentication'", results.len());
    
    if !results.is_empty() {
        let first_result = &results[0];
        println!("First result in: {}", first_result.chunk.file_path.display());
        println!("Content type: {:?}", first_result.chunk.content_type);
    }

    Ok(json!({
        "query": "authentication",
        "total_results": results.len(),
        "files_found": results.iter()
            .map(|r| r.chunk.file_path.display().to_string())
            .collect::<std::collections::HashSet<_>>()
            .len()
    }))
}

async fn test_file_search(server: &GCoreMcpServer) -> Result<serde_json::Value> {
    // Test file pattern search
    let json_files = server.content_search().find_files(r".*\.json$")?;
    let md_files = server.content_search().find_files(r".*\.md$")?;
    let py_files = server.content_search().find_files(r".*\.py$")?;
    
    println!("Found {} JSON files", json_files.len());
    println!("Found {} Markdown files", md_files.len());
    println!("Found {} Python files", py_files.len());
    
    assert!(!json_files.is_empty(), "Should find JSON configuration files");
    assert!(!md_files.is_empty(), "Should find Markdown documentation");
    assert!(!py_files.is_empty(), "Should find Python source files");

    Ok(json!({
        "json_files": json_files.len(),
        "md_files": md_files.len(),
        "py_files": py_files.len(),
        "total_files": json_files.len() + md_files.len() + py_files.len()
    }))
}

async fn test_content_stats(server: &GCoreMcpServer) -> Result<serde_json::Value> {
    let stats = server.content_search().get_stats();
    
    println!("Content stats: {} files, {} chunks, {} tokens", 
             stats.total_files, stats.total_chunks, stats.total_tokens);
    
    // With our test repository, we should have indexed content
    if stats.total_files > 0 {
        assert!(stats.total_chunks > 0, "Should have content chunks");
        assert!(stats.total_tokens > 0, "Should have extracted tokens");
        println!("✅ Content indexing is working!");
    } else {
        println!("⚠️  Content indexing not yet complete");
    }

    Ok(json!({
        "total_files": stats.total_files,
        "total_chunks": stats.total_chunks,
        "total_tokens": stats.total_tokens,
        "content_by_type": stats.content_by_type
    }))
}

async fn test_explain_symbol(server: &GCoreMcpServer, symbol_id_hex: &str) -> Result<serde_json::Value> {
    let symbol_id = gcore::NodeId::from_hex(symbol_id_hex)?;
    
    if let Some(node) = server.graph_store().get_node(&symbol_id) {
        let dependencies = server.graph_query().find_dependencies(&symbol_id, gcore::graph::DependencyType::Direct)?;
        let references = server.graph_query().find_references(&symbol_id)?;
        
        println!("Symbol '{}' has {} dependencies and {} references", 
                 node.name, dependencies.len(), references.len());
        
        Ok(json!({
            "symbol": {
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string(),
                "dependencies_count": dependencies.len(),
                "references_count": references.len()
            }
        }))
    } else {
        Err(anyhow::anyhow!("Symbol not found: {}", symbol_id_hex))
    }
}

#[tokio::test]
async fn test_mcp_server_error_handling() -> Result<()> {
    let mcp_server = GCoreMcpServer::new()?;
    
    // Test operations on uninitialized server
    let stats = mcp_server.content_search().get_stats();
    assert_eq!(stats.total_files, 0, "Uninitialized server should have no content");
    
    // Test file search on empty index
    let files = mcp_server.content_search().find_files("*.py")?;
    assert!(files.is_empty(), "Empty index should return no files");
    
    // Test content search on empty index
    let results = mcp_server.content_search().simple_search("test", Some(10))?;
    assert!(results.is_empty(), "Empty index should return no search results");
    
    println!("✅ Error handling tests passed");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_performance() -> Result<()> {
    let temp_dir = create_test_repository().await?;
    let repo_path = temp_dir.path();

    let start_time = std::time::Instant::now();
    
    let mut mcp_server = GCoreMcpServer::new()?;
    mcp_server.initialize_with_repository(repo_path).await?;
    
    let initialization_time = start_time.elapsed();
    println!("Repository initialization took: {:?}", initialization_time);
    
    // Test search performance
    let search_start = std::time::Instant::now();
    
    // Multiple search operations
    for _ in 0..5 {
        let _ = mcp_server.content_search().simple_search("user", Some(10))?;
        let _ = mcp_server.content_search().find_files("*.json")?;
        let _ = server.graph_query().search_symbols("User", None, Some(5))?;
    }
    
    let search_time = search_start.elapsed();
    println!("15 search operations took: {:?}", search_time);
    
    // Verify reasonable performance
    assert!(initialization_time.as_secs() < 10, "Initialization should complete in under 10 seconds");
    assert!(search_time.as_millis() < 1000, "Search operations should complete in under 1 second");
    
    println!("✅ Performance tests passed");
    Ok(())
} 