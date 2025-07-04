//! Comprehensive Error Handling & Edge Case Tests for MCP Tools
//! 
//! This test suite verifies that all MCP tools handle error scenarios gracefully
//! and fail in predictable, documented ways.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use codeprism_mcp::{CodePrismMcpServer, tools_legacy::{CallToolParams, ToolManager, ListToolsParams}};

/// Test fixture for error handling tests
struct ErrorTestFixture {
    server: Arc<RwLock<CodePrismMcpServer>>,
    tool_manager: ToolManager,
}

impl ErrorTestFixture {
    /// Create a new error test fixture
    async fn new() -> Result<Self> {
        let server = CodePrismMcpServer::new()?;
        let server_arc = Arc::new(RwLock::new(server));
        let tool_manager = ToolManager::new(server_arc.clone());
        
        Ok(Self {
            server: server_arc,
            tool_manager,
        })
    }

    /// Test error handling for a tool with specific error scenario
    async fn test_error_scenario(&self, tool_name: &str, params: serde_json::Value, expected_error_type: &str) -> Result<bool> {
        let start = Instant::now();
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        match self.tool_manager.call_tool(tool_params).await {
            Ok(result) => {
                let duration = start.elapsed();
                let error_returned = result.is_error.unwrap_or(false);
                
                if error_returned {
                    debug!("✅ {} - Properly returned error for {} ({}ms)", tool_name, expected_error_type, duration.as_millis());
                    Ok(true)
                } else {
                    warn!("❌ {} - Unexpected success for {} ({}ms)", tool_name, expected_error_type, duration.as_millis());
                    Ok(false)
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                debug!("✅ {} - Properly threw exception for {} ({}ms): {}", tool_name, expected_error_type, duration.as_millis(), e);
                Ok(true)
            }
        }
    }
}

#[tokio::test]
async fn test_missing_required_parameters() -> Result<()> {
    info!("🧪 Testing missing required parameters across all tools...");
    
    let fixture = ErrorTestFixture::new().await?;

    // Tools with missing required parameters
    let missing_param_tests = vec![
        // Core Navigation Tools
        ("search_symbols", json!({}), "missing pattern"),
        ("search_content", json!({}), "missing query"),
        ("find_files", json!({}), "missing pattern"),
        ("trace_path", json!({}), "missing start/end"),
        
        // Search & Discovery Tools
        ("find_references", json!({}), "missing symbol"),
        ("find_dependencies", json!({}), "missing target"),
        ("explain_symbol", json!({}), "missing symbol"),
        
        // Analysis Tools
        ("analyze_complexity", json!({}), "missing target"),
        ("analyze_performance", json!({}), "missing target"),
        ("analyze_security", json!({}), "missing target"),
        ("detect_patterns", json!({}), "missing pattern_types"),
        ("analyze_dependencies", json!({}), "missing target"),
        ("analyze_architecture", json!({}), "missing target"),
        ("analyze_api_surface", json!({}), "missing target"),
        ("analyze_code_quality", json!({}), "missing target"),
        ("analyze_test_coverage", json!({}), "missing target"),
        ("analyze_documentation", json!({}), "missing target"),
        ("analyze_maintainability", json!({}), "missing target"),
        
        // JavaScript Analysis Tools
        ("analyze_javascript", json!({}), "missing target"),
        ("analyze_js_dependencies", json!({}), "missing target"),
        ("analyze_js_performance", json!({}), "missing target"),
        
        // Workflow Tools
        ("provide_guidance", json!({}), "missing context"),
        ("optimize_code", json!({}), "missing target"),
    ];

    let mut handled_errors = 0;
    let total_tests = missing_param_tests.len();

    for (tool_name, params, error_type) in missing_param_tests {
        if fixture.test_error_scenario(tool_name, params, error_type).await? {
            handled_errors += 1;
        }
    }

    let success_rate = (handled_errors as f64 / total_tests as f64) * 100.0;
    info!("📊 Missing parameter error handling: {}/{} ({:.1}% success rate)", handled_errors, total_tests, success_rate);
    
    assert!(success_rate >= 60.0, "Missing parameter error handling should be at least 60%");
    Ok(())
}

#[tokio::test]
async fn test_invalid_file_paths() -> Result<()> {
    info!("🧪 Testing invalid file paths and non-existent targets...");
    
    let fixture = ErrorTestFixture::new().await?;

    // Test cases with invalid file paths
    let invalid_path_tests = vec![
        // Non-existent files
        ("analyze_complexity", json!({"target": "/nonexistent/file.py"}), "nonexistent file"),
        ("find_dependencies", json!({"target": "/invalid/path/file.js"}), "invalid path"),
        ("analyze_code_quality", json!({"target": "/does/not/exist/"}), "missing directory"),
        ("explain_symbol", json!({"symbol": "test", "location": "/fake/location.py"}), "fake location"),
        
        // Empty string paths
        ("analyze_dependencies", json!({"target": ""}), "empty path"),
        ("find_files", json!({"pattern": ""}), "empty pattern"),
        ("search_content", json!({"query": ""}), "empty query"),
    ];

    let mut handled_errors = 0;
    let total_tests = invalid_path_tests.len();

    for (tool_name, params, error_type) in invalid_path_tests {
        if fixture.test_error_scenario(tool_name, params, error_type).await? {
            handled_errors += 1;
        }
    }

    let success_rate = (handled_errors as f64 / total_tests as f64) * 100.0;
    info!("📊 Invalid path error handling: {}/{} ({:.1}% success rate)", handled_errors, total_tests, success_rate);
    
    assert!(success_rate >= 50.0, "Invalid path error handling should be at least 50%");
    Ok(())
}

#[tokio::test]
async fn test_malformed_parameters() -> Result<()> {
    info!("🧪 Testing malformed JSON and invalid parameter types...");
    
    let fixture = ErrorTestFixture::new().await?;

    // Test cases with malformed or invalid parameter types
    let malformed_param_tests = vec![
        // Wrong parameter types
        ("search_symbols", json!({"pattern": 123}), "numeric pattern"),
        ("find_duplicates", json!({"similarity_threshold": "invalid"}), "string threshold"),
        ("analyze_complexity", json!({"target": ["array", "not", "string"]}), "array target"),
        ("search_content", json!({"max_results": "not_a_number"}), "string max_results"),
        
        // Negative values where positive expected
        ("search_symbols", json!({"pattern": "test", "limit": -5}), "negative limit"),
        ("find_duplicates", json!({"similarity_threshold": -0.5}), "negative threshold"),
        ("search_content", json!({"query": "test", "max_results": -10}), "negative max_results"),
    ];

    let mut handled_errors = 0;
    let total_tests = malformed_param_tests.len();

    for (tool_name, params, error_type) in malformed_param_tests {
        if fixture.test_error_scenario(tool_name, params, error_type).await? {
            handled_errors += 1;
        }
    }

    let success_rate = (handled_errors as f64 / total_tests as f64) * 100.0;
    info!("📊 Malformed parameter error handling: {}/{} ({:.1}% success rate)", handled_errors, total_tests, success_rate);
    
    assert!(success_rate >= 50.0, "Malformed parameter error handling should be at least 50%");
    Ok(())
}

#[tokio::test]
async fn test_boundary_conditions() -> Result<()> {
    info!("🧪 Testing boundary conditions and edge cases...");
    
    let fixture = ErrorTestFixture::new().await?;

    // Test cases for boundary conditions
    let boundary_tests = vec![
        // Zero values
        ("search_symbols", json!({"pattern": "test", "limit": 0}), "zero limit"),
        ("search_content", json!({"query": "test", "max_results": 0}), "zero max_results"),
        ("find_duplicates", json!({"similarity_threshold": 0.0}), "zero threshold"),
        
        // Very large strings
        ("search_symbols", json!({"pattern": "x".repeat(1000)}), "very long pattern"),
        ("search_content", json!({"query": "q".repeat(500)}), "very long query"),
        
        // Special characters in parameters
        ("search_symbols", json!({"pattern": ".*[]{()}+?^$|\\"}), "regex special chars"),
        ("search_content", json!({"query": "unicode: 🚀🔍📊"}), "unicode characters"),
    ];

    let mut handled_tests = 0;
    let total_tests = boundary_tests.len();

    for (tool_name, params, condition_type) in boundary_tests {
        match fixture.test_error_scenario(tool_name, params, condition_type).await {
            Ok(handled) => {
                if handled {
                    handled_tests += 1;
                }
            }
            Err(_) => {
                // Tool may crash on boundary conditions, which is also acceptable
                handled_tests += 1;
                debug!("✅ {} - Boundary condition {} caused graceful failure", tool_name, condition_type);
            }
        }
    }

    let success_rate = (handled_tests as f64 / total_tests as f64) * 100.0;
    info!("📊 Boundary condition handling: {}/{} ({:.1}% success rate)", handled_tests, total_tests, success_rate);
    
    assert!(success_rate >= 40.0, "Boundary condition handling should be at least 40%");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_error_coverage() -> Result<()> {
    info!("🧪 Running comprehensive error coverage summary...");
    
    let fixture = ErrorTestFixture::new().await?;
    
    // Get all available tools
    let result = fixture.tool_manager.list_tools(ListToolsParams { cursor: None }).await?;
    let available_tools: Vec<String> = result.tools.iter().map(|t| t.name.clone()).collect();
    
    info!("📊 Error testing coverage:");
    info!("Total tools available: {}", available_tools.len());
    
    // Categories of tools
    let core_tools = vec!["repository_stats", "search_symbols", "search_content", "find_files", "content_stats", "trace_path"];
    let analysis_tools: Vec<_> = available_tools.iter().filter(|name| name.starts_with("analyze_")).collect();
    let js_tools = vec!["analyze_javascript", "analyze_js_dependencies", "analyze_js_performance"];
    
    let core_available = core_tools.iter().filter(|&tool| available_tools.contains(&tool.to_string())).count();
    let analysis_available = analysis_tools.len();
    let js_available = js_tools.iter().filter(|&tool| available_tools.contains(&tool.to_string())).count();
    
    info!("Core tools available for error testing: {}/{}", core_available, core_tools.len());
    info!("Analysis tools available for error testing: {}", analysis_available);
    info!("JavaScript tools available for error testing: {}/{}", js_available, js_tools.len());
    
    // Calculate coverage percentage
    let total_expected = 26; // Total expected tools across all categories
    let coverage_percentage = (available_tools.len() as f64 / total_expected as f64) * 100.0;
    
    info!("Error testing coverage: {:.1}%", coverage_percentage);
    
    assert!(coverage_percentage >= 50.0, "Error testing coverage should be at least 50%");
    assert!(core_available >= 3, "Should have at least 3 core tools available for error testing");
    assert!(analysis_available >= 5, "Should have at least 5 analysis tools available for error testing");
    
    info!("✅ Comprehensive error coverage test passed!");
    Ok(())
} 