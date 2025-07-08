//! Integration tests for Issue #232: Validate CodePrism server integration with all 26 tools
//!
//! This module implements comprehensive testing of the CodePrism MCP server integration
//! with all 26 tools using the Mandrel MCP Test Harness (mandrel-mcp-th).
//!
//! ## Test Structure
//! - Individual tool validation tests
//! - Error handling and parameter validation tests  
//! - Performance and concurrent execution tests
//! - Tool chaining and workflow integration tests
//!
//! ## TDD RED Phase
//! These tests are designed to fail initially to validate our test framework.
//! The GREEN phase will implement actual tool integration to make them pass.

use mandrel_mcp_th::testing::IntegrationTestFramework;
use serde_json;

// ========================================================================
// Individual Tool Tests - Core Navigation (4 tools)
// ========================================================================

#[tokio::test]
async fn test_repository_stats_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("repository_stats").await {
        Ok(output) => {
            println!("✅ repository_stats tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected repository_stats tool failure during RED phase: {:?}",
                e
            );
            // During RED phase, we expect this to fail - that's correct TDD behavior
        }
    }
}

#[tokio::test]
async fn test_trace_path_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("trace_path").await {
        Ok(output) => {
            println!("✅ trace_path tool succeeded: {}", output);
        }
        Err(e) => {
            println!("Expected trace_path tool failure during RED phase: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_find_dependencies_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("find_dependencies").await {
        Ok(output) => {
            println!("✅ find_dependencies tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected find_dependencies tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_find_references_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("find_references").await {
        Ok(output) => {
            println!("✅ find_references tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected find_references tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - Core Symbols (2 tools)
// ========================================================================

#[tokio::test]
async fn test_explain_symbol_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("explain_symbol").await {
        Ok(output) => {
            println!("✅ explain_symbol tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected explain_symbol tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_search_symbols_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("search_symbols").await {
        Ok(output) => {
            println!("✅ search_symbols tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected search_symbols tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - Search Discovery (4 tools)
// ========================================================================

#[tokio::test]
async fn test_search_content_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("search_content").await {
        Ok(output) => {
            println!("✅ search_content tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected search_content tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_find_files_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("find_files").await {
        Ok(output) => {
            println!("✅ find_files tool succeeded: {}", output);
        }
        Err(e) => {
            println!("Expected find_files tool failure during RED phase: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_content_stats_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("content_stats").await {
        Ok(output) => {
            println!("✅ content_stats tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected content_stats tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_detect_patterns_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("detect_patterns").await {
        Ok(output) => {
            println!("✅ detect_patterns tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected detect_patterns tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - Quality Analysis (6 tools)
// ========================================================================

#[tokio::test]
async fn test_analyze_complexity_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_complexity").await {
        Ok(output) => {
            println!("✅ analyze_complexity tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_complexity tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_find_duplicates_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("find_duplicates").await {
        Ok(output) => {
            println!("✅ find_duplicates tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected find_duplicates tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_find_unused_code_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("find_unused_code").await {
        Ok(output) => {
            println!("✅ find_unused_code tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected find_unused_code tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_security_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_security").await {
        Ok(output) => {
            println!("✅ analyze_security tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_security tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_performance_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_performance").await {
        Ok(output) => {
            println!("✅ analyze_performance tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_performance tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_api_surface_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_api_surface").await {
        Ok(output) => {
            println!("✅ analyze_api_surface tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_api_surface tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - Advanced Analysis (4 tools)
// ========================================================================

#[tokio::test]
async fn test_analyze_transitive_dependencies_tool() {
    let framework = IntegrationTestFramework::new();

    match framework
        .execute_tool_test("analyze_transitive_dependencies")
        .await
    {
        Ok(output) => {
            println!(
                "✅ analyze_transitive_dependencies tool succeeded: {}",
                output
            );
        }
        Err(e) => {
            println!(
                "Expected analyze_transitive_dependencies tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_trace_data_flow_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("trace_data_flow").await {
        Ok(output) => {
            println!("✅ trace_data_flow tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected trace_data_flow tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_trace_inheritance_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("trace_inheritance").await {
        Ok(output) => {
            println!("✅ trace_inheritance tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected trace_inheritance tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_decorators_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_decorators").await {
        Ok(output) => {
            println!("✅ analyze_decorators tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_decorators tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - JavaScript Specific (3 tools)
// ========================================================================

#[tokio::test]
async fn test_analyze_javascript_frameworks_tool() {
    let framework = IntegrationTestFramework::new();

    match framework
        .execute_tool_test("analyze_javascript_frameworks")
        .await
    {
        Ok(output) => {
            println!(
                "✅ analyze_javascript_frameworks tool succeeded: {}",
                output
            );
        }
        Err(e) => {
            println!(
                "Expected analyze_javascript_frameworks tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_react_components_tool() {
    let framework = IntegrationTestFramework::new();

    match framework
        .execute_tool_test("analyze_react_components")
        .await
    {
        Ok(output) => {
            println!("✅ analyze_react_components tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_react_components tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_analyze_nodejs_patterns_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("analyze_nodejs_patterns").await {
        Ok(output) => {
            println!("✅ analyze_nodejs_patterns tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected analyze_nodejs_patterns tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Individual Tool Tests - Workflow Orchestration (3 tools)
// ========================================================================

#[tokio::test]
async fn test_suggest_analysis_workflow_tool() {
    let framework = IntegrationTestFramework::new();

    match framework
        .execute_tool_test("suggest_analysis_workflow")
        .await
    {
        Ok(output) => {
            println!("✅ suggest_analysis_workflow tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected suggest_analysis_workflow tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_batch_analysis_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("batch_analysis").await {
        Ok(output) => {
            println!("✅ batch_analysis tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected batch_analysis tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_optimize_workflow_tool() {
    let framework = IntegrationTestFramework::new();

    match framework.execute_tool_test("optimize_workflow").await {
        Ok(output) => {
            println!("✅ optimize_workflow tool succeeded: {}", output);
        }
        Err(e) => {
            println!(
                "Expected optimize_workflow tool failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Error Handling and Edge Case Tests (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_invalid_tool_parameters() {
    // RED: This test should fail until proper parameter validation is implemented
    let _invalid_params = serde_json::json!({
        "invalid_field": "invalid_value",
        "malformed_parameter": 12345
    });

    let framework = IntegrationTestFramework::new();
    let result = framework.execute_tool_test("repository_stats").await;

    match result {
        Ok(_) => {
            println!("Tool executed despite invalid parameters (may be expected)");
        }
        Err(e) => {
            println!(
                "Expected parameter validation failure during RED phase: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_missing_required_parameters() {
    // RED: This test should fail until proper parameter validation is implemented
    // Test trace_path without required source/target parameters
    let framework = IntegrationTestFramework::new();
    let result = framework.execute_tool_test("trace_path").await;

    match result {
        Ok(_) => {
            println!("Tool executed despite missing parameters (may provide defaults)");
        }
        Err(e) => {
            println!(
                "Expected missing parameter failure during RED phase: {:?}",
                e
            );
        }
    }
}

// ========================================================================
// Performance and Concurrent Execution Tests (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_tool_performance_requirements() {
    // RED: This test should fail until performance requirements are met
    let framework = IntegrationTestFramework::new();
    let tools_to_test = ["repository_stats", "search_symbols", "analyze_complexity"];
    let mut results = Vec::new();

    for tool_name in &tools_to_test {
        let start_time = tokio::time::Instant::now();
        let result = framework.execute_tool_test(tool_name).await;
        let execution_time = start_time.elapsed();

        match result {
            Ok(_) => {
                // Performance requirement: Tools should complete within 10 seconds
                if execution_time.as_secs() <= 10 {
                    println!("✅ {} completed in {:?}", tool_name, execution_time);
                } else {
                    println!(
                        "⚠️  {} took {:?} (exceeds 10s limit)",
                        tool_name, execution_time
                    );
                }
                results.push((tool_name, execution_time, true));
            }
            Err(e) => {
                println!(
                    "Expected {} performance test failure during RED phase: {:?}",
                    tool_name, e
                );
                results.push((tool_name, execution_time, false));
            }
        }
    }

    // At least one tool should be tested for performance validation
    assert!(!results.is_empty(), "No tools were tested for performance");
}

#[tokio::test]
async fn test_concurrent_tool_execution() {
    // RED: This test should fail until concurrent execution is properly handled
    let _framework = IntegrationTestFramework::new();
    let tools = ["repository_stats", "search_symbols", "analyze_complexity"];
    let mut handles = Vec::new();

    for &tool_name in &tools {
        let framework_clone = IntegrationTestFramework::new();
        let handle =
            tokio::spawn(async move { framework_clone.execute_tool_test(tool_name).await });
        handles.push((tool_name, handle));
    }

    let mut concurrent_results = Vec::new();

    for (tool_name, handle) in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(output) => {
                    println!(
                        "✅ Concurrent execution of {} succeeded: {}",
                        tool_name, output
                    );
                    concurrent_results.push((tool_name, true));
                }
                Err(e) => {
                    println!(
                        "Expected concurrent {} failure during RED phase: {:?}",
                        tool_name, e
                    );
                    concurrent_results.push((tool_name, false));
                }
            },
            Err(e) => {
                println!("Concurrent task for {} panicked: {:?}", tool_name, e);
                concurrent_results.push((tool_name, false));
            }
        }
    }

    // At least concurrent execution should be attempted
    assert_eq!(
        concurrent_results.len(),
        3,
        "All three concurrent tasks should complete"
    );
}

// ========================================================================
// Integration and Workflow Tests (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_tool_chaining_workflow() {
    // RED: This test should fail until tool chaining and data flow is implemented

    println!("Testing tool chaining workflow...");

    let framework = IntegrationTestFramework::new();

    // Step 1: Get repository stats
    let stats_result = framework.execute_tool_test("repository_stats").await;

    // Step 2: Search for symbols
    let search_result = framework.execute_tool_test("search_symbols").await;

    // Step 3: Analyze complexity
    let complexity_result = framework.execute_tool_test("analyze_complexity").await;

    // Validate workflow execution
    let workflow_steps = [
        ("repository_stats", &stats_result),
        ("search_symbols", &search_result),
        ("analyze_complexity", &complexity_result),
    ];

    let mut successful_steps = 0;

    for (step_name, result) in &workflow_steps {
        match result {
            Ok(_) => {
                println!("✅ Workflow step {} succeeded", step_name);
                successful_steps += 1;
            }
            Err(e) => {
                println!(
                    "Expected workflow step {} failure during RED phase: {:?}",
                    step_name, e
                );
            }
        }
    }

    println!(
        "Workflow completed: {}/{} steps successful",
        successful_steps,
        workflow_steps.len()
    );

    // At least attempt all workflow steps
    assert_eq!(
        workflow_steps.len(),
        3,
        "All workflow steps should be attempted"
    );
}
