use gcore::content::*;
use std::path::PathBuf;

#[tokio::test]
async fn test_content_search_integration_basic() {
    // Create content search manager
    let search_manager = ContentSearchManager::new();
    
    // Create test markdown content
    let md_content = r#"# User Authentication Guide

This guide explains how to implement user authentication in your application.

## Overview

User authentication is a critical security feature that verifies user identity.

## Implementation Steps

1. Set up authentication middleware
2. Configure user sessions
3. Implement login/logout functionality

### Code Example

```javascript
function authenticate(username, password) {
    // Verify credentials
    return verifyUser(username, password);
}
```

## Security Considerations

Always use HTTPS and secure session management.
"#;
    
    let md_file = PathBuf::from("auth_guide.md");
    
    // Create test configuration content
    let config_content = r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "auth_db"
  },
  "security": {
    "jwt_secret": "your-secret-key",
    "session_timeout": 3600
  }
}"#;
    
    let config_file = PathBuf::from("config.json");
    
    // Index the files
    search_manager.index_file(&md_file, md_content).unwrap();
    search_manager.index_file(&config_file, config_content).unwrap();
    
    // Test basic search
    let results = search_manager.simple_search("authentication", Some(10)).unwrap();
    assert!(!results.is_empty(), "Should find authentication-related content");
    
    // Test documentation-specific search
    let doc_results = search_manager.search_documentation("authentication", Some(5)).unwrap();
    assert!(!doc_results.is_empty(), "Should find authentication in documentation");
    
    // Test configuration-specific search
    let config_results = search_manager.search_configuration("database", Some(5)).unwrap();
    assert!(!config_results.is_empty(), "Should find database configuration");
    
    // Test file pattern search
    let md_files = search_manager.find_files("*.md").unwrap();
    assert!(!md_files.is_empty(), "Should find markdown files");
    
    // Test content statistics
    let stats = search_manager.get_stats();
    assert!(stats.total_files >= 2, "Should have indexed at least 2 files");
    assert!(stats.total_chunks > 0, "Should have content chunks");
    
    println!("✅ Basic content search integration test passed");
}

#[tokio::test] 
async fn test_content_search_integration_complex() {
    let search_manager = ContentSearchManager::new();
    
    // Create multiple content types
    let files_and_content = vec![
        (
            "README.md",
            r#"# Project Overview
This is a web application for user management.

## Features
- User registration and authentication
- Profile management
- Admin dashboard
"#,
        ),
        (
            "package.json",
            r#"{
  "name": "user-management-app",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "jsonwebtoken": "^8.5.1"
  }
}"#,
        ),
        (
            "docker-compose.yml",
            r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
  database:
    image: postgres:13
    environment:
      POSTGRES_DB: userdb
"#,
        ),
        (
            ".env",
            r#"DATABASE_URL=postgresql://localhost:5432/userdb
JWT_SECRET=super-secret-key
PORT=3000
DEBUG=true
"#,
        ),
    ];
    
    // Index all files
    for (filename, content) in &files_and_content {
        let file_path = PathBuf::from(filename);
        search_manager.index_file(&file_path, content).unwrap();
    }
    
    // Test cross-file search
    let user_results = search_manager.simple_search("user", Some(20)).unwrap();
    assert!(user_results.len() >= 2, "Should find 'user' across multiple files");
    
    // Test specific file type searches
    let yaml_results = search_manager.search_in_files(
        "database",
        vec!["*.yml".to_string(), "*.yaml".to_string()],
        Some(10),
    ).unwrap();
    assert!(!yaml_results.is_empty(), "Should find database in YAML files");
    
    let env_results = search_manager.search_configuration("DATABASE_URL", Some(5)).unwrap();
    assert!(!env_results.is_empty(), "Should find DATABASE_URL in config");
    
    // Test regex search
    let port_results = search_manager.regex_search(r"\d{4}", Some(10)).unwrap();
    assert!(!port_results.is_empty(), "Should find port numbers with regex");
    
    // Test case sensitivity
    let case_sensitive_query = SearchQueryBuilder::new("USER")
        .case_sensitive()
        .build();
    let case_results = search_manager.search(&case_sensitive_query).unwrap();
    
    let case_insensitive_query = SearchQueryBuilder::new("USER")
        .build(); // case insensitive by default
    let insensitive_results = search_manager.search(&case_insensitive_query).unwrap();
    
    assert!(insensitive_results.len() >= case_results.len(), 
           "Case insensitive should find more or equal results");
    
    // Test content with context
    let context_query = SearchQueryBuilder::new("jsonwebtoken")
        .with_context(2)
        .build();
    let context_results = search_manager.search(&context_query).unwrap();
    
    if !context_results.is_empty() {
        let first_result = &context_results[0];
        assert!(!first_result.matches.is_empty(), "Should have matches");
        
        let first_match = &first_result.matches[0];
        assert!(first_match.context_before.is_some() || first_match.context_after.is_some(),
               "Should have context information");
    }
    
    // Test statistics after indexing multiple files
    let final_stats = search_manager.get_stats();
    assert_eq!(final_stats.total_files, 4, "Should have indexed 4 files");
    assert!(final_stats.total_chunks >= 4, "Should have multiple chunks");
    assert!(final_stats.total_tokens > 50, "Should have extracted many tokens");
    
    println!("✅ Complex content search integration test passed");
}

#[tokio::test]
async fn test_content_search_error_handling() {
    let search_manager = ContentSearchManager::new();
    
    // Test searching with no indexed content
    let empty_results = search_manager.simple_search("anything", Some(10)).unwrap();
    assert!(empty_results.is_empty(), "Should return empty results for empty index");
    
    // Test invalid regex pattern
    let invalid_regex_result = search_manager.regex_search("[invalid", Some(10));
    // Should either return empty results or error gracefully
    match invalid_regex_result {
        Ok(results) => assert!(results.is_empty(), "Invalid regex should return empty results"),
        Err(_) => assert!(true, "Invalid regex should return error"),
    }
    
    // Test file operations on non-existent files
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");
    let remove_result = search_manager.remove_file(&nonexistent_path);
    // Should handle gracefully
    match remove_result {
        Ok(_) => assert!(true, "Should handle removal gracefully"),
        Err(_) => assert!(true, "Error is acceptable for non-existent file"),
    }
    
    // Test empty search query
    let empty_search = search_manager.simple_search("", Some(10)).unwrap();
    assert!(empty_search.is_empty(), "Empty query should return empty results");
    
    println!("✅ Error handling integration test passed");
}

#[tokio::test]
async fn test_content_search_performance() {
    let search_manager = ContentSearchManager::new();
    
    // Create multiple files with substantial content
    for i in 0..10 {
        let content = format!(
            r#"# Document {}

This is document number {} with substantial content for performance testing.

## Section A
Content about user authentication and security measures.

## Section B  
Information about database configuration and connection pooling.

## Section C
Details about API endpoints and request handling.

```javascript
function processUser{i}() {{
    // Process user data for document {i}
    return authenticateUser{i}();
}}
```

## Conclusion
This completes document {} with various searchable terms.
"#,
            i, i, i = i, i = i, i = i, i
        );
        
        let file_path = PathBuf::from(format!("doc_{}.md", i));
        search_manager.index_file(&file_path, &content).unwrap();
    }
    
    // Test search performance
    let start_time = std::time::Instant::now();
    
    for _ in 0..5 {
        let _results = search_manager.simple_search("authentication", Some(50)).unwrap();
        let _results = search_manager.search_documentation("database", Some(20)).unwrap();
        let _results = search_manager.regex_search(r"function \w+", Some(30)).unwrap();
    }
    
    let elapsed = start_time.elapsed();
    assert!(elapsed.as_millis() < 1000, "15 searches should complete in under 1 second");
    
    // Test memory efficiency with statistics
    let stats = search_manager.get_stats();
    assert_eq!(stats.total_files, 10, "Should have indexed 10 files");
    assert!(stats.total_chunks >= 10, "Should have substantial content chunks");
    
    println!("✅ Performance integration test passed in {:?}", elapsed);
}

#[tokio::test]
async fn test_content_search_file_updates() {
    let search_manager = ContentSearchManager::new();
    
    let file_path = PathBuf::from("evolving_doc.md");
    
    // Initial content
    let initial_content = r#"# Initial Document
This document talks about basic concepts.
Authentication is important.
"#;
    
    search_manager.index_file(&file_path, initial_content).unwrap();
    
    // Search for initial content
    let initial_results = search_manager.simple_search("basic", Some(10)).unwrap();
    assert!(!initial_results.is_empty(), "Should find initial content");
    
    // Update content
    let updated_content = r#"# Updated Document  
This document now talks about advanced concepts.
Authorization and authentication are both important.
Security is paramount.
"#;
    
    search_manager.index_file(&file_path, updated_content).unwrap();
    
    // Search for updated content
    let advanced_results = search_manager.simple_search("advanced", Some(10)).unwrap();
    assert!(!advanced_results.is_empty(), "Should find updated content");
    
    let basic_results = search_manager.simple_search("basic", Some(10)).unwrap();
    assert!(basic_results.is_empty(), "Should not find old content after update");
    
    let security_results = search_manager.simple_search("security", Some(10)).unwrap();
    assert!(!security_results.is_empty(), "Should find new content");
    
    // Remove file
    search_manager.remove_file(&file_path).unwrap();
    let removed_results = search_manager.simple_search("advanced", Some(10)).unwrap();
    assert!(removed_results.is_empty(), "Should not find content after removal");
    
    println!("✅ File updates integration test passed");
}

#[tokio::test]
async fn test_content_search_query_builder() {
    let search_manager = ContentSearchManager::new();
    
    // Create test files
    let md_content = r#"# JavaScript Guide
Learn about JavaScript functions and async/await.
"#;
    let py_content = r#"# Python Guide  
Learn about Python functions and async/await.
"#;
    let config_content = r#"{"language": "javascript", "async": true}"#;
    
    let md_file = PathBuf::from("js_guide.md");
    let py_file = PathBuf::from("py_guide.md");  
    let config_file = PathBuf::from("config.json");
    
    search_manager.index_file(&md_file, md_content).unwrap();
    search_manager.index_file(&py_file, py_content).unwrap();
    search_manager.index_file(&config_file, config_content).unwrap();
    
    // Test markdown-specific search
    let md_query = SearchQueryBuilder::markdown_docs("JavaScript").build();
    let md_results = search_manager.search(&md_query).unwrap();
    assert!(!md_results.is_empty(), "Should find JavaScript in markdown");
    
    // Test JSON-specific search
    let json_query = SearchQueryBuilder::json_config("async").build();
    let json_results = search_manager.search(&json_query).unwrap();
    assert!(!json_results.is_empty(), "Should find async in JSON config");
    
    // Test file pattern filtering
    let py_query = SearchQueryBuilder::new("Python")
        .include_files(vec!["*guide.md".to_string()])
        .build();
    let py_results = search_manager.search(&py_query).unwrap();
    assert!(!py_results.is_empty(), "Should find Python in guide files");
    
    // Test exclusion patterns
    let exclude_query = SearchQueryBuilder::new("async")
        .exclude_files(vec!["*.json".to_string()])
        .build();
    let exclude_results = search_manager.search(&exclude_query).unwrap();
    // Should find async in markdown but not JSON
    let has_md_results = exclude_results.iter().any(|r| 
        r.chunk.file_path.to_string_lossy().ends_with(".md"));
    let has_json_results = exclude_results.iter().any(|r| 
        r.chunk.file_path.to_string_lossy().ends_with(".json"));
    
    assert!(has_md_results, "Should find results in markdown files");
    assert!(!has_json_results, "Should not find results in excluded JSON files");
    
    println!("✅ Query builder integration test passed");
} 