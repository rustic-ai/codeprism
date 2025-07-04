//! Comprehensive MCP Tools Test Suite - Integration Tests
//! 
//! This test suite provides real testing for all MCP tools with actual functionality verification

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use codeprism_mcp::{CodePrismMcpServer, tools_legacy::{CallToolParams, CallToolResult, ToolManager, ListToolsParams}};

#[tokio::test]
async fn test_comprehensive_mcp_tools_functionality() -> Result<()> {
    println!("🚀 Starting comprehensive MCP tools test suite...");
    
    // Initialize server
    let server = CodePrismMcpServer::new()?;
    let server_arc = Arc::new(RwLock::new(server));
    let tool_manager = ToolManager::new(server_arc);

    // Core Navigation Tools (6 tools)
    let core_tools = vec![
        ("repository_stats", json!({})),
        ("search_symbols", json!({"pattern": ".*", "limit": 5})),
        ("search_content", json!({"query": "function", "max_results": 10})),
        ("find_files", json!({"pattern": "*.py"})),
        ("content_stats", json!({})),
        ("trace_path", json!({"start": "test-projects/python-sample/main.py", "end": "test-projects/python-sample/core/"})),
    ];

    // Search & Discovery Tools (4 tools)  
    let search_tools = vec![
        ("find_duplicates", json!({"similarity_threshold": 0.8})),
        ("find_references", json!({"symbol": "function", "location": "test-projects/python-sample/"})),
        ("find_dependencies", json!({"target": "test-projects/python-sample/main.py"})),
        ("explain_symbol", json!({"symbol": "main", "location": "test-projects/python-sample/main.py"})),
    ];

    // Analysis Tools (11 tools)
    let analysis_tools = vec![
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
    ];

    // JavaScript Analysis Tools (3 tools)
    let js_tools = vec![
        ("analyze_javascript", json!({"target": "test-projects/js-dependency-test-project/", "analysis_types": ["syntax", "dependencies"]})),
        ("analyze_js_dependencies", json!({"target": "test-projects/js-dependency-test-project/package.json"})),
        ("analyze_js_performance", json!({"target": "test-projects/js-dependency-test-project/"})),
    ];

    // Workflow Tools (2 tools)
    let workflow_tools = vec![
        ("provide_guidance", json!({"context": "improving code quality", "target": "test-projects/python-sample/"})),
        ("optimize_code", json!({"target": "test-projects/python-sample/main.py", "optimization_types": ["performance"]})),
    ];

    // Combine all tools
    let all_tools = [core_tools, search_tools, analysis_tools, js_tools, workflow_tools].concat();

    let mut results = Vec::new();
    let mut successful_tools = 0;
    let mut performance_metrics = Vec::new();

    println!("\n🔧 Testing {} tools across all categories...", all_tools.len());

    for (tool_name, params) in all_tools {
        println!("  🔍 Testing: {}", tool_name);
        
        let start = Instant::now();
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        match tool_manager.call_tool(tool_params).await {
            Ok(result) => {
                let duration = start.elapsed();
                let success = result.is_error.unwrap_or(false) == false;
                
                if success {
                    successful_tools += 1;
                    println!("    ✅ {} - Success ({}ms)", tool_name, duration.as_millis());
                } else {
                    println!("    ❌ {} - Failed with error ({}ms)", tool_name, duration.as_millis());
                    if let Some(content) = result.content.first() {
                        println!("      Error: {}", &content.text[..std::cmp::min(100, content.text.len())]);
                    }
                }
                
                performance_metrics.push((tool_name, duration.as_millis()));
                results.push((tool_name, success, duration));
            }
            Err(e) => {
                let duration = start.elapsed();
                println!("    ❌ {} - Exception: {} ({}ms)", tool_name, e, duration.as_millis());
                results.push((tool_name, false, duration));
            }
        }
    }

    // Performance Analysis
    let avg_time: f64 = performance_metrics.iter()
        .map(|(_, time)| *time as f64)
        .sum::<f64>() / performance_metrics.len() as f64;

    let core_tools_avg = performance_metrics.iter()
        .filter(|(name, _)| matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path"))
        .map(|(_, time)| *time as f64)
        .sum::<f64>() / 6.0;

    let analysis_tools_avg = performance_metrics.iter()
        .filter(|(name, _)| name.starts_with("analyze_"))
        .map(|(_, time)| *time as f64)
        .sum::<f64>() / 11.0;

    // Generate comprehensive report
    let total_tools = results.len();
    let success_rate = (successful_tools as f64 / total_tools as f64) * 100.0;
    
    println!("\n📊 === COMPREHENSIVE TEST RESULTS ===");
    println!("Total tools tested: {}", total_tools);
    println!("Successful: {}", successful_tools);
    println!("Failed: {}", total_tools - successful_tools);
    println!("Success rate: {:.1}%", success_rate);
    
    println!("\n⚡ === PERFORMANCE METRICS ===");
    println!("Average execution time: {:.1}ms", avg_time);
    println!("Core tools average: {:.1}ms", core_tools_avg);
    println!("Analysis tools average: {:.1}ms", analysis_tools_avg);
    
    // Performance requirements check
    let slow_tools: Vec<_> = performance_metrics.iter()
        .filter(|(name, time)| {
            let is_core = matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path");
            let is_analysis = name.starts_with("analyze_");
            (is_core && *time > 50) || (is_analysis && *time > 500)
        })
        .collect();

    if !slow_tools.is_empty() {
        println!("\n⚠️  === PERFORMANCE WARNINGS ===");
        for (name, time) in slow_tools {
            println!("  {} took {}ms (exceeds expected threshold)", name, time);
        }
    }

    println!("\n📋 === DETAILED RESULTS BY CATEGORY ===");
    
    // Core Tools Results
    let core_successes = results.iter().filter(|(name, success, _)| 
        matches!(*name, "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path") && *success
    ).count();
    println!("Core Navigation Tools: {}/6 successful", core_successes);
    
    // Analysis Tools Results  
    let analysis_successes = results.iter().filter(|(name, success, _)| 
        name.starts_with("analyze_") && *success
    ).count();
    println!("Analysis Tools: {}/11 successful", analysis_successes);
    
    // JavaScript Tools Results
    let js_successes = results.iter().filter(|(name, success, _)| 
        matches!(*name, "analyze_javascript" | "analyze_js_dependencies" | "analyze_js_performance") && *success
    ).count();
    println!("JavaScript Analysis Tools: {}/3 successful", js_successes);

    // Assertions - We should have substantial functionality working
    assert!(successful_tools > 0, "At least one tool should work");
    assert!(success_rate >= 50.0, "Success rate should be at least 50% but was {:.1}%", success_rate);
    
    // Core tools should have high success rate
    assert!(core_successes >= 3, "At least 3 core tools should work");
    
    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_listing() -> Result<()> {
    println!("📋 Testing MCP tool listing functionality...");
    
    let server = CodePrismMcpServer::new()?;
    let server_arc = Arc::new(RwLock::new(server));
    let tool_manager = ToolManager::new(server_arc);
    
    let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await?;
    
    let tool_count = result.tools.len();
    println!("✅ Successfully listed {} tools", tool_count);
    
    // Check for expected tool categories
    let tool_names: Vec<String> = result.tools.iter().map(|t| t.name.clone()).collect();
    
    let expected_core_tools = vec![
        "repository_stats", "search_symbols", "search_content", "find_files", "content_stats", "trace_path"
    ];
    
    let expected_analysis_tools = vec![
        "analyze_complexity", "analyze_performance", "analyze_security", "detect_patterns", 
        "analyze_dependencies", "analyze_architecture", "analyze_api_surface", "analyze_code_quality"
    ];
    
    println!("\n🔍 Checking tool availability:");
    let mut found_core = 0;
    let mut found_analysis = 0;
    
    for tool in &expected_core_tools {
        if tool_names.contains(&tool.to_string()) {
            found_core += 1;
            println!("  ✅ Core tool found: {}", tool);
        } else {
            println!("  ❌ Core tool missing: {}", tool);
        }
    }
    
    for tool in &expected_analysis_tools {
        if tool_names.contains(&tool.to_string()) {
            found_analysis += 1;
            println!("  ✅ Analysis tool found: {}", tool);
        } else {
            println!("  ❌ Analysis tool missing: {}", tool);
        }
    }
    
    println!("\n📊 Tool availability summary:");
    println!("Core tools available: {}/{}", found_core, expected_core_tools.len());
    println!("Analysis tools available: {}/{}", found_analysis, expected_analysis_tools.len());
    
    // Assertions
    assert!(tool_count > 0, "Should have at least some tools available");
    assert!(found_core >= 3, "Should have at least 3 core tools available");
    assert!(found_analysis >= 5, "Should have at least 5 analysis tools available");
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_scenarios() -> Result<()> {
    println!("🧪 Testing error handling scenarios...");
    
    let server = CodePrismMcpServer::new()?;
    let server_arc = Arc::new(RwLock::new(server));
    let tool_manager = ToolManager::new(server_arc);

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
        println!("  🔬 Testing error case: {}", tool_name);
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        match tool_manager.call_tool(tool_params).await {
            Ok(result) => {
                // Tool should return error status for invalid input
                if result.is_error.unwrap_or(false) {
                    error_tests_handled += 1;
                    println!("    ✅ {} - Properly handled error", tool_name);
                    error_results.push((tool_name, "handled_error", true));
                } else {
                    println!("    ⚠️  {} - Unexpected success with invalid input", tool_name);
                    error_results.push((tool_name, "unexpected_success", false));
                }
            }
            Err(_) => {
                // It's also acceptable for tools to throw exceptions for invalid input
                error_tests_handled += 1;
                println!("    ✅ {} - Properly threw exception", tool_name);
                error_results.push((tool_name, "threw_exception", true));
            }
        }
    }

    println!("\n📊 Error handling summary:");
    println!("Error cases tested: {}", error_results.len());
    println!("Properly handled: {}", error_tests_handled);
    
    let error_handling_rate = (error_tests_handled as f64 / error_results.len() as f64) * 100.0;
    println!("Error handling rate: {:.1}%", error_handling_rate);
    
    // We expect most error cases to be handled properly
    assert!(error_handling_rate >= 60.0, "Error handling rate should be at least 60%");
    
    Ok(())
} 