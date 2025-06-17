//! Integration tests for content search fixes
//!
//! This test verifies that the MCP server correctly indexes content and
//! provides working content search, file search, and content stats functionality.

use anyhow::Result;
use prism_core_mcp::PrismMcpServer;
use std::fs;
use tempfile::TempDir;
use tokio::time::Duration;

/// Create a simple test repository with different file types
async fn create_test_repository() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create Python file with classes and functions
    fs::write(
        repo_path.join("user_service.py"),
        r#"""User service module."""

class User:
    """User model for authentication."""
    
    def __init__(self, username: str, email: str):
        self.username = username
        self.email = email
        self.is_active = True
    
    def authenticate(self, password: str) -> bool:
        """Authenticate user with password."""
        # Simple authentication logic
        return password == "secret"

class UserService:
    """Service for managing users."""
    
    def __init__(self):
        self.users = {}
    
    def create_user(self, username: str, email: str) -> User:
        """Create a new user."""
        user = User(username, email)
        self.users[username] = user
        return user
    
    def get_user(self, username: str) -> User:
        """Get user by username."""
        return self.users.get(username)
"#,
    )?;

    // Create documentation file
    fs::write(
        repo_path.join("README.md"),
        r#"# User Management System

A simple user management system with authentication.

## Features

- User registration
- User authentication  
- Password validation

## API Documentation

### User Class

The `User` class represents a user in the system with authentication capabilities.

### UserService Class

The `UserService` class manages user operations and authentication.
"#,
    )?;

    // Create configuration file
    fs::write(
        repo_path.join("config.json"),
        r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "userdb"
  },
  "auth": {
    "secret_key": "super-secret-key",
    "token_expiry": "24h"
  }
}
"#,
    )?;

    Ok(temp_dir)
}

#[tokio::test]
async fn test_content_search_integration_fixed() -> Result<()> {
    // Create test repository
    let temp_dir = create_test_repository().await?;
    let repo_path = temp_dir.path();

    // Initialize MCP server
    let mut mcp_server = PrismMcpServer::new()?;
    mcp_server.initialize_with_repository(repo_path).await?;

    // Wait for content indexing to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test 1: Content stats should show indexed files
    let stats = mcp_server.content_search().get_stats();
    assert!(stats.total_files >= 3, "Should have indexed at least 3 files, got {}", stats.total_files);
    assert!(stats.total_chunks > 0, "Should have content chunks, got {}", stats.total_chunks);
    assert!(stats.total_tokens > 0, "Should have extracted tokens, got {}", stats.total_tokens);
    
    println!("✅ Content stats: {} files, {} chunks, {} tokens", 
             stats.total_files, stats.total_chunks, stats.total_tokens);

    // Test 2: File search should find specific file types
    let py_files = mcp_server.content_search().find_files(r".*\.py$")?;
    assert!(!py_files.is_empty(), "Should find Python files");
    assert!(py_files.iter().any(|p| p.to_string_lossy().contains("user_service.py")), 
           "Should find user_service.py");
    
    let md_files = mcp_server.content_search().find_files(r".*\.md$")?;
    assert!(!md_files.is_empty(), "Should find Markdown files");
    
    let json_files = mcp_server.content_search().find_files(r".*\.json$")?;
    assert!(!json_files.is_empty(), "Should find JSON files");
    
    println!("✅ File search: {} Python, {} Markdown, {} JSON files", 
             py_files.len(), md_files.len(), json_files.len());

    // Test 3: Content search should find text across file types
    let user_results = mcp_server.content_search().simple_search("user", Some(10))?;
    assert!(!user_results.is_empty(), "Should find 'user' in content");
    
    // Should find results in multiple file types
    let file_types: std::collections::HashSet<_> = user_results.iter()
        .map(|r| r.chunk.file_path.extension().and_then(|ext| ext.to_str()).unwrap_or(""))
        .collect();
    assert!(file_types.len() >= 2, "Should find 'user' in multiple file types");
    
    println!("✅ Content search: {} results for 'user' across {} file types", 
             user_results.len(), file_types.len());

    // Test 4: Documentation-specific search
    let doc_results = mcp_server.content_search().search_documentation("authentication", Some(5))?;
    assert!(!doc_results.is_empty(), "Should find 'authentication' in documentation");
    
    println!("✅ Documentation search: {} results for 'authentication'", doc_results.len());

    // Test 5: Configuration search
    let config_results = mcp_server.content_search().search_configuration("database", Some(5))?;
    assert!(!config_results.is_empty(), "Should find 'database' in configuration");
    
    println!("✅ Configuration search: {} results for 'database'", config_results.len());

    // Test 6: Symbol search still works (regression test)
    let symbol_results = mcp_server.graph_query().search_symbols(
        "User",
        Some(vec![prism_core::NodeKind::Class]),
        Some(10)
    )?;
    assert!(!symbol_results.is_empty(), "Should find User class symbols");
    
    println!("✅ Symbol search: {} User class symbols found", symbol_results.len());

    println!("🎉 All content search integration tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_content_search_empty_repository() -> Result<()> {
    // Test with empty repository
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let mut mcp_server = PrismMcpServer::new()?;
    mcp_server.initialize_with_repository(repo_path).await?;

    // Wait briefly
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test empty repository behavior
    let stats = mcp_server.content_search().get_stats();
    assert_eq!(stats.total_files, 0, "Empty repository should have 0 files");
    assert_eq!(stats.total_chunks, 0, "Empty repository should have 0 chunks");

    let files = mcp_server.content_search().find_files(".*")?;
    assert!(files.is_empty(), "Empty repository should find no files");

    let results = mcp_server.content_search().simple_search("anything", Some(10))?;
    assert!(results.is_empty(), "Empty repository should find no content");

    println!("✅ Empty repository handling test passed");
    Ok(())
}

#[tokio::test]
async fn test_content_search_performance() -> Result<()> {
    let temp_dir = create_test_repository().await?;
    let repo_path = temp_dir.path();

    let start_time = std::time::Instant::now();
    
    // Initialize and index repository
    let mut mcp_server = PrismMcpServer::new()?;
    mcp_server.initialize_with_repository(repo_path).await?;
    
    let init_time = start_time.elapsed();
    
    // Wait for indexing
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Perform multiple search operations
    let search_start = std::time::Instant::now();
    
    for _ in 0..5 {
        let _ = mcp_server.content_search().simple_search("user", Some(10))?;
        let _ = mcp_server.content_search().find_files(".*\\.py$")?;
        let _ = mcp_server.content_search().get_stats();
    }
    
    let search_time = search_start.elapsed();
    
    // Verify reasonable performance
    assert!(init_time.as_secs() < 5, "Initialization should complete in under 5 seconds");
    assert!(search_time.as_millis() < 500, "15 operations should complete in under 500ms");
    
    println!("✅ Performance test: init {}ms, 15 operations {}ms", 
             init_time.as_millis(), search_time.as_millis());
    
    Ok(())
} 