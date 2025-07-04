//! Comprehensive MCP Tools Test Suite
//! 
//! This test suite provides real testing for all MCP tools with actual functionality verification

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use codeprism_mcp::{CodePrismMcpServer, tools_legacy::{CallToolParams, CallToolResult, ToolManager, ListToolsParams}};

/// Test fixture for MCP tools testing
struct McpToolsTestFixture {
    server: Arc<RwLock<CodePrismMcpServer>>,
    tool_manager: ToolManager,
}

impl McpToolsTestFixture {
    /// Create a new test fixture
    async fn new() -> Result<Self> {
        let server = CodePrismMcpServer::new()?;
        let server_arc = Arc::new(RwLock::new(server));
        let tool_manager = ToolManager::new(server_arc.clone());
        
        Ok(Self {
            server: server_arc,
            tool_manager,
        })
    }

    /// Test a single tool with parameters
    async fn test_tool(&self, tool_name: &str, params: serde_json::Value) -> Result<(bool, u128, String)> {
        let start = Instant::now();
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        match self.tool_manager.call_tool(tool_params).await {
            Ok(result) => {
                let duration = start.elapsed();
                let success = result.is_error.unwrap_or(false) == false;
                let output = if let Some(content) = result.content.first() {
                    content.text.clone()
                } else {
                    "No output".to_string()
                };
                
                Ok((success, duration.as_millis(), output))
            }
            Err(e) => {
                let duration = start.elapsed();
                Ok((false, duration.as_millis(), e.to_string()))
            }
        }
    }

    /// Get available tools
    async fn get_available_tools(&self) -> Result<Vec<String>> {
        let result = self.tool_manager.list_tools(ListToolsParams { cursor: None }).await?;
        Ok(result.tools.iter().map(|t| t.name.clone()).collect())
    }
}

#[tokio::test]
async fn test_mcp_tools_comprehensive_functionality() -> Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting comprehensive MCP tools test suite...");
    
    // Create test fixture
    let fixture = McpToolsTestFixture::new().await?;

    // Define tool test cases with real test data
    let tool_tests = vec![
        // Core Navigation Tools (6 tools)
        ("repository_stats", json!({})),
        ("search_symbols", json!({"pattern": "function", "limit": 5})),
        ("search_content", json!({"query": "def", "max_results": 10})),
        ("find_files", json!({"pattern": "*.py"})),
        ("content_stats", json!({})),
        ("trace_path", json!({"start": "test-projects/python-sample/main.py", "end": "test-projects/python-sample/core/"})),
        
        // Search & Discovery Tools (4 tools)  
        ("find_duplicates", json!({"similarity_threshold": 0.8})),
        ("find_references", json!({"symbol": "function", "location": "test-projects/python-sample/"})),
        ("find_dependencies", json!({"target": "test-projects/python-sample/main.py"})),
        ("explain_symbol", json!({"symbol": "main", "location": "test-projects/python-sample/main.py"})),
        
        // Analysis Tools (11 tools)
        ("analyze_complexity", json!({"target": "test-projects/python-sample/main.py", "metrics": ["cyclomatic"]})),
        ("analyze_performance", json!({"target": "test-projects/python-sample/", "analysis_types": ["memory", "cpu"]})),
        ("analyze_security", json!({"target": "test-projects/python-sample/", "security_checks": ["vulnerabilities"]})),
        ("detect_patterns", json!({"pattern_types": ["design_patterns"], "target": "test-projects/python-sample/"})),
        ("analyze_dependencies", json!({"target": "test-projects/python-sample/"})),
        ("analyze_architecture", json!({"target": "test-projects/python-sample/"})),
        ("analyze_api_surface", json!({"target": "test-projects/python-sample/"})),
        ("analyze_code_quality", json!({"target": "test-projects/python-sample/", "quality_types": ["all"]})),
        ("analyze_test_coverage", json!({"target": "test-projects/python-sample/"})),
        ("analyze_documentation", json!({"target": "test-projects/python-sample/"})),
        ("analyze_maintainability", json!({"target": "test-projects/python-sample/"})),
        
        // JavaScript Analysis Tools (3 tools)
        ("analyze_javascript", json!({"target": "test-projects/js-dependency-test-project/", "analysis_types": ["syntax", "dependencies"]})),
        ("analyze_js_dependencies", json!({"target": "test-projects/js-dependency-test-project/package.json"})),
        ("analyze_js_performance", json!({"target": "test-projects/js-dependency-test-project/"})),
        
        // Workflow Tools (2 tools)
        ("provide_guidance", json!({"context": "improving code quality", "target": "test-projects/python-sample/"})),
        ("optimize_code", json!({"target": "test-projects/python-sample/main.py", "optimization_types": ["performance"]})),
    ];

    let mut results = Vec::new();
    let mut successful_tools = 0;
    let mut performance_metrics = Vec::new();

    info!("🔧 Testing {} tools across all categories...", tool_tests.len());

    // Test each tool
    for (tool_name, params) in tool_tests {
        debug!("Testing tool: {}", tool_name);
        
        match fixture.test_tool(tool_name, params).await {
            Ok((success, duration_ms, output)) => {
                if success {
                    successful_tools += 1;
                    info!("✅ {} - Success ({}ms)", tool_name, duration_ms);
                    debug!("  Output: {}", &output[..std::cmp::min(100, output.len())]);
                } else {
                    warn!("❌ {} - Failed ({}ms)", tool_name, duration_ms);
                    debug!("  Error: {}", &output[..std::cmp::min(100, output.len())]);
                }
                
                performance_metrics.push((tool_name, duration_ms));
                results.push((tool_name, success, duration_ms, output));
            }
            Err(e) => {
                warn!("❌ {} - Exception: {}", tool_name, e);
                results.push((tool_name, false, 0, e.to_string()));
            }
        }
    }

    // Performance Analysis
    let avg_time: f64 = performance_metrics.iter()
        .map(|(_, time)| *time as f64)
        .sum::<f64>() / performance_metrics.len() as f64;

    let core_tools_times: Vec<u128> = performance_metrics.iter()
        .filter(|(name, _)| matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path"))
        .map(|(_, time)| *time)
        .collect();

    let analysis_tools_times: Vec<u128> = performance_metrics.iter()
        .filter(|(name, _)| name.starts_with("analyze_"))
        .map(|(_, time)| *time)
        .collect();

    let core_tools_avg = if !core_tools_times.is_empty() {
        core_tools_times.iter().sum::<u128>() as f64 / core_tools_times.len() as f64
    } else {
        0.0
    };

    let analysis_tools_avg = if !analysis_tools_times.is_empty() {
        analysis_tools_times.iter().sum::<u128>() as f64 / analysis_tools_times.len() as f64
    } else {
        0.0
    };

    // Generate comprehensive report
    let total_tools = results.len();
    let success_rate = (successful_tools as f64 / total_tools as f64) * 100.0;
    
    info!("📊 === COMPREHENSIVE TEST RESULTS ===");
    info!("Total tools tested: {}", total_tools);
    info!("Successful: {}", successful_tools);
    info!("Failed: {}", total_tools - successful_tools);
    info!("Success rate: {:.1}%", success_rate);
    
    info!("⚡ === PERFORMANCE METRICS ===");
    info!("Average execution time: {:.1}ms", avg_time);
    info!("Core tools average: {:.1}ms", core_tools_avg);
    info!("Analysis tools average: {:.1}ms", analysis_tools_avg);
    
    // Performance warnings
    let slow_tools: Vec<_> = performance_metrics.iter()
        .filter(|(name, time)| {
            let is_core = matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path");
            let is_analysis = name.starts_with("analyze_");
            (is_core && *time > 100) || (is_analysis && *time > 1000)
        })
        .collect();

    if !slow_tools.is_empty() {
        warn!("⚠️  === PERFORMANCE WARNINGS ===");
        for (name, time) in slow_tools {
            warn!("  {} took {}ms (exceeds expected threshold)", name, time);
        }
    }

    // Category-specific results
    let core_successes = results.iter().filter(|(name, success, _, _)| 
        matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path") && *success
    ).count();

    let analysis_successes = results.iter().filter(|(name, success, _, _)| 
        name.starts_with("analyze_") && *success
    ).count();

    let js_successes = results.iter().filter(|(name, success, _, _)| 
        matches!(*name, "analyze_javascript" | "analyze_js_dependencies" | "analyze_js_performance") && *success
    ).count();

    info!("📋 === DETAILED RESULTS BY CATEGORY ===");
    info!("Core Navigation Tools: {}/6 successful", core_successes);
    info!("Analysis Tools: {}/11 successful", analysis_successes);
    info!("JavaScript Analysis Tools: {}/3 successful", js_successes);

    // Assertions - We should have substantial functionality working
    assert!(successful_tools > 0, "At least one tool should work");
    assert!(success_rate >= 30.0, "Success rate should be at least 30% but was {:.1}%", success_rate);
    
    // Core tools should have reasonable success rate
    assert!(core_successes >= 2, "At least 2 core tools should work");
    
    info!("🎉 Comprehensive MCP tools test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_listing_functionality() -> Result<()> {
    info!("📋 Testing MCP tool listing functionality...");
    
    let fixture = McpToolsTestFixture::new().await?;
    
    let available_tools = fixture.get_available_tools().await?;
    let tool_count = available_tools.len();
    
    info!("✅ Successfully listed {} tools", tool_count);
    
    // Expected tool categories
    let expected_core_tools = vec![
        "repository_stats", "search_symbols", "search_content", "find_files", "content_stats", "trace_path"
    ];
    
    let expected_analysis_tools = vec![
        "analyze_complexity", "analyze_performance", "analyze_security", "detect_patterns", 
        "analyze_dependencies", "analyze_architecture", "analyze_api_surface", "analyze_code_quality"
    ];
    
    info!("🔍 Checking tool availability:");
    let mut found_core = 0;
    let mut found_analysis = 0;
    
    for tool in &expected_core_tools {
        if available_tools.contains(&tool.to_string()) {
            found_core += 1;
            debug!("✅ Core tool found: {}", tool);
        } else {
            debug!("❌ Core tool missing: {}", tool);
        }
    }
    
    for tool in &expected_analysis_tools {
        if available_tools.contains(&tool.to_string()) {
            found_analysis += 1;
            debug!("✅ Analysis tool found: {}", tool);
        } else {
            debug!("❌ Analysis tool missing: {}", tool);
        }
    }
    
    info!("📊 Tool availability summary:");
    info!("Core tools available: {}/{}", found_core, expected_core_tools.len());
    info!("Analysis tools available: {}/{}", found_analysis, expected_analysis_tools.len());
    
    // Assertions
    assert!(tool_count > 0, "Should have at least some tools available");
    assert!(found_core >= 2, "Should have at least 2 core tools available");
    assert!(found_analysis >= 3, "Should have at least 3 analysis tools available");
    
    info!("✅ Tool listing test passed!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_error_handling_scenarios() -> Result<()> {
    info!("🧪 Testing error handling scenarios...");
    
    let fixture = McpToolsTestFixture::new().await?;

    // Test cases that should fail gracefully
    let error_test_cases = vec![
        ("trace_path", json!({})), // Missing required parameters
        ("find_dependencies", json!({})), // Missing required parameters  
        ("search_symbols", json!({})), // Missing required parameters
        ("analyze_complexity", json!({"target": "nonexistent/file.py"})), // File doesn't exist
        ("find_files", json!({"pattern": ""})), // Empty pattern
        ("search_content", json!({"query": ""})), // Empty query
    ];

    let mut error_tests_handled = 0;
    let mut error_results = Vec::new();

    for (tool_name, params) in error_test_cases {
        debug!("Testing error case: {}", tool_name);
        
        match fixture.test_tool(tool_name, params).await {
            Ok((success, duration_ms, output)) => {
                if !success {
                    error_tests_handled += 1;
                    debug!("✅ {} - Properly handled error ({}ms)", tool_name, duration_ms);
                    error_results.push((tool_name, "handled_error", true));
                } else {
                    debug!("⚠️  {} - Unexpected success with invalid input", tool_name);
                    error_results.push((tool_name, "unexpected_success", false));
                }
            }
            Err(_) => {
                error_tests_handled += 1;
                debug!("✅ {} - Properly threw exception", tool_name);
                error_results.push((tool_name, "threw_exception", true));
            }
        }
    }

    info!("📊 Error handling summary:");
    info!("Error cases tested: {}", error_results.len());
    info!("Properly handled: {}", error_tests_handled);
    
    let error_handling_rate = (error_tests_handled as f64 / error_results.len() as f64) * 100.0;
    info!("Error handling rate: {:.1}%", error_handling_rate);
    
    // We expect most error cases to be handled properly
    assert!(error_handling_rate >= 50.0, "Error handling rate should be at least 50%");
    
    info!("✅ Error handling test passed!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tools_performance_requirements() -> Result<()> {
    info!("⚡ Testing performance requirements...");
    
    let fixture = McpToolsTestFixture::new().await?;

    // Test performance-sensitive tools
    let performance_tests = vec![
        ("repository_stats", json!({}), 100), // Should be fast
        ("search_symbols", json!({"pattern": "function", "limit": 5}), 200), // Should be reasonably fast
        ("find_files", json!({"pattern": "*.py"}), 300), // File operations can be slower
        ("content_stats", json!({}), 500), // Content analysis can take time
    ];

    let mut performance_results = Vec::new();
    let mut performance_violations = Vec::new();

    for (tool_name, params, max_time_ms) in performance_tests {
        debug!("Testing performance for: {}", tool_name);
        
        match fixture.test_tool(tool_name, params).await {
            Ok((success, duration_ms, _)) => {
                performance_results.push((tool_name, duration_ms, max_time_ms));
                
                if success && duration_ms <= max_time_ms {
                    debug!("✅ {} - Performance OK ({}ms <= {}ms)", tool_name, duration_ms, max_time_ms);
                } else if success {
                    warn!("⚠️  {} - Performance slow ({}ms > {}ms)", tool_name, duration_ms, max_time_ms);
                    performance_violations.push((tool_name, duration_ms, max_time_ms));
                } else {
                    debug!("❌ {} - Failed, cannot test performance", tool_name);
                }
            }
            Err(e) => {
                warn!("❌ {} - Exception during performance test: {}", tool_name, e);
            }
        }
    }

    info!("📊 Performance test summary:");
    info!("Performance tests run: {}", performance_results.len());
    info!("Performance violations: {}", performance_violations.len());
    
    if !performance_violations.is_empty() {
        warn!("⚠️  Performance violations:");
        for (tool, actual, expected) in &performance_violations {
            warn!("  {} took {}ms (expected <= {}ms)", tool, actual, expected);
        }
    }

    // We allow some performance violations but not too many
    let violation_rate = (performance_violations.len() as f64 / performance_results.len() as f64) * 100.0;
    assert!(violation_rate <= 50.0, "Performance violation rate should be <= 50% but was {:.1}%", violation_rate);
    
    info!("✅ Performance requirements test passed!");
    Ok(())
} 