//! Performance & Integration Testing for MCP Tool Suite
//! 
//! This test suite validates performance requirements, load testing,
//! and integration workflows for all MCP tools.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use codeprism_mcp::{CodePrismMcpServer, tools_legacy::{CallToolParams, ToolManager}};

/// Performance test fixture
struct PerformanceTestFixture {
    server: Arc<RwLock<CodePrismMcpServer>>,
    tool_manager: ToolManager,
}

impl PerformanceTestFixture {
    /// Create a new performance test fixture
    async fn new() -> Result<Self> {
        let server = CodePrismMcpServer::new()?;
        let server_arc = Arc::new(RwLock::new(server));
        let tool_manager = ToolManager::new(server_arc.clone());
        
        Ok(Self {
            server: server_arc,
            tool_manager,
        })
    }

    /// Execute a tool and measure performance
    async fn measure_tool_performance(&self, tool_name: &str, params: serde_json::Value) -> Result<(bool, u128, usize)> {
        let start = Instant::now();
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        match self.tool_manager.call_tool(tool_params).await {
            Ok(result) => {
                let duration = start.elapsed();
                let success = result.is_error.unwrap_or(false) == false;
                let output_size = result.content.iter()
                    .map(|c| c.text.len())
                    .sum::<usize>();
                
                Ok((success, duration.as_millis(), output_size))
            }
            Err(e) => {
                let duration = start.elapsed();
                debug!("Tool {} failed: {}", tool_name, e);
                Ok((false, duration.as_millis(), 0))
            }
        }
    }
}

#[tokio::test]
async fn test_core_tools_performance_requirements() -> Result<()> {
    info!("⚡ Testing core tools performance requirements...");
    
    let fixture = PerformanceTestFixture::new().await?;

    // Core tools should be reasonably fast
    let core_tool_tests = vec![
        ("repository_stats", json!({}) ),
        ("search_symbols", json!({"pattern": "function", "limit": 10})),
        ("search_content", json!({"query": "def", "max_results": 20})),
        ("find_files", json!({"pattern": "*.py"})),
        ("content_stats", json!({})),
    ];

    let mut performance_results = Vec::new();
    let max_acceptable_ms = 500; // 500ms for core tools

    for (tool_name, params) in core_tool_tests {
        info!("Testing performance of core tool: {}", tool_name);
        
        match fixture.measure_tool_performance(tool_name, params).await {
            Ok((success, duration, output_size)) => {
                let meets_requirement = duration <= max_acceptable_ms && success;
                
                info!("  {} - {}ms, {} bytes - {}", 
                      tool_name, duration, output_size,
                      if meets_requirement { "✅ PASS" } else { "❌ SLOW" });
                
                performance_results.push((tool_name, meets_requirement));
            }
            Err(e) => {
                warn!("  {} - Error: {}", tool_name, e);
                performance_results.push((tool_name, false));
            }
        }
    }

    let passing_tools = performance_results.iter().filter(|(_, meets)| *meets).count();
    let total_tools = performance_results.len();
    let pass_rate = (passing_tools as f64 / total_tools as f64) * 100.0;

    info!("📊 Core tools performance: {}/{} tools meet requirements ({:.1}%)", passing_tools, total_tools, pass_rate);
    
    assert!(pass_rate >= 40.0, "At least 40% of core tools should meet performance requirements");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_tool_execution() -> Result<()> {
    info!("🔄 Testing concurrent tool execution...");
    
    let fixture = PerformanceTestFixture::new().await?;

    // Test concurrent execution of multiple tools
    let concurrent_tasks = vec![
        ("repository_stats", json!({})),
        ("search_symbols", json!({"pattern": "function", "limit": 5})),
        ("find_files", json!({"pattern": "*.py"})),
        ("content_stats", json!({})),
    ];

    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Launch all tasks concurrently
    for (tool_name, params) in concurrent_tasks {
        let fixture_ref = &fixture;
        let tool_name = tool_name.to_string();
        
        let handle = tokio::spawn(async move {
            fixture_ref.measure_tool_performance(&tool_name, params).await
        });
        
        handles.push((tool_name, handle));
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for (tool_name, handle) in handles {
        match handle.await {
            Ok(Ok((success, duration, output_size))) => {
                results.push((tool_name, success, duration, output_size));
            }
            Ok(Err(e)) => {
                warn!("Concurrent task {} failed: {}", tool_name, e);
            }
            Err(_) => {
                warn!("Concurrent task {} panicked", tool_name);
            }
        }
    }

    let total_duration = start_time.elapsed();
    let successful_tasks = results.iter().filter(|(_, success, _, _)| *success).count();
    
    info!("📊 Concurrent execution results:");
    info!("  Total execution time: {}ms", total_duration.as_millis());
    info!("  Successful tasks: {}/{}", successful_tasks, results.len());
    
    for (tool_name, success, duration, output_size) in &results {
        info!("  {} - {} ({}ms, {} bytes)", tool_name, 
              if *success { "✅ SUCCESS" } else { "❌ FAILED" }, 
              duration, output_size);
    }

    // Concurrent execution should not take excessively long
    assert!(total_duration.as_secs() <= 10, "Concurrent execution should complete within 10 seconds");
    assert!(successful_tasks >= results.len() / 2, "At least half of concurrent tasks should succeed");
    
    Ok(())
}

#[tokio::test]
async fn test_load_testing_scenarios() -> Result<()> {
    info!("📈 Testing load scenarios with multiple requests...");
    
    let fixture = PerformanceTestFixture::new().await?;

    // Test rapid successive requests to the same tool
    let load_test_tool = "repository_stats";
    let test_params = json!({});
    let num_requests = 5;

    let start_time = Instant::now();
    let mut successful_requests = 0;

    for i in 0..num_requests {
        match fixture.measure_tool_performance(load_test_tool, test_params.clone()).await {
            Ok((success, duration, _)) => {
                if success {
                    successful_requests += 1;
                }
                debug!("Request {} - {} ({}ms)", i + 1, 
                       if success { "SUCCESS" } else { "FAILED" }, duration);
            }
            Err(e) => {
                warn!("Request {} failed: {}", i + 1, e);
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let total_test_time = start_time.elapsed();
    let success_rate = (successful_requests as f64 / num_requests as f64) * 100.0;
    let throughput = num_requests as f64 / total_test_time.as_secs_f64();

    info!("📊 Load testing results:");
    info!("  Total test time: {}ms", total_test_time.as_millis());
    info!("  Success rate: {:.1}%", success_rate);
    info!("  Throughput: {:.1} requests/second", throughput);

    // Load testing requirements
    assert!(success_rate >= 60.0, "Load test success rate should be at least 60%");
    assert!(throughput >= 0.5, "Should handle at least 0.5 requests per second");

    Ok(())
}

#[tokio::test]
async fn test_integration_workflow_scenarios() -> Result<()> {
    info!("🔗 Testing integration workflow scenarios...");
    
    let fixture = PerformanceTestFixture::new().await?;

    // Test realistic workflow: analyze a project step by step
    let workflow_steps = vec![
        ("repository_stats", json!({}), "Get repository overview"),
        ("find_files", json!({"pattern": "*.py"}), "Find Python files"),
        ("search_symbols", json!({"pattern": "def", "limit": 10}), "Find function definitions"),
        ("content_stats", json!({}), "Get content statistics"),
    ];

    let workflow_start = Instant::now();
    let mut workflow_results = Vec::new();

    for (step_num, (tool_name, params, description)) in workflow_steps.iter().enumerate() {
        info!("Workflow Step {}: {} - {}", step_num + 1, description, tool_name);
        
        match fixture.measure_tool_performance(tool_name, params.clone()).await {
            Ok((success, duration, output_size)) => {
                workflow_results.push((tool_name, success, duration, output_size));
                
                info!("  Step {} - {} ({}ms, {} bytes)", step_num + 1,
                      if success { "✅ SUCCESS" } else { "❌ FAILED" }, 
                      duration, output_size);
            }
            Err(e) => {
                warn!("  Step {} failed: {}", step_num + 1, e);
                workflow_results.push((tool_name, false, 0, 0));
            }
        }
    }

    let total_workflow_duration = workflow_start.elapsed();
    let successful_steps = workflow_results.iter().filter(|(_, success, _, _)| *success).count();
    let workflow_success_rate = (successful_steps as f64 / workflow_results.len() as f64) * 100.0;

    info!("📊 Integration workflow results:");
    info!("  Total workflow time: {}ms", total_workflow_duration.as_millis());
    info!("  Successful steps: {}/{}", successful_steps, workflow_results.len());
    info!("  Workflow success rate: {:.1}%", workflow_success_rate);

    // Integration workflow requirements
    assert!(workflow_success_rate >= 50.0, "Integration workflow should have at least 50% success rate");
    assert!(total_workflow_duration.as_secs() <= 15, "Complete workflow should finish within 15 seconds");
    assert!(successful_steps >= 2, "At least 2 workflow steps should succeed");

    Ok(())
}

#[tokio::test]
async fn test_scalability_with_different_inputs() -> Result<()> {
    info!("📏 Testing scalability with different input sizes...");
    
    let fixture = PerformanceTestFixture::new().await?;

    // Test scalability by varying input parameters
    let scalability_tests = vec![
        ("search_symbols", json!({"pattern": "function", "limit": 5}), "Small limit"),
        ("search_symbols", json!({"pattern": "function", "limit": 20}), "Medium limit"),
        ("search_content", json!({"query": "def", "max_results": 10}), "Small results"),
        ("search_content", json!({"query": "def", "max_results": 50}), "Medium results"),
    ];

    let mut scalability_results = Vec::new();

    for (tool_name, params, test_description) in scalability_tests {
        info!("Testing scalability: {} - {}", tool_name, test_description);
        
        match fixture.measure_tool_performance(tool_name, params).await {
            Ok((success, duration, output_size)) => {
                scalability_results.push((test_description, success, duration, output_size));
                
                info!("  {} - {} ({}ms, {} bytes)", test_description,
                      if success { "✅ SUCCESS" } else { "❌ FAILED" }, 
                      duration, output_size);
            }
            Err(e) => {
                warn!("  {} failed: {}", test_description, e);
            }
        }
    }

    let successful_tests = scalability_results.iter().filter(|(_, success, _, _)| *success).count();
    let total_tests = scalability_results.len();
    let scalability_success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;

    info!("📊 Scalability test results:");
    info!("  Successful scalability tests: {}/{}", successful_tests, total_tests);
    info!("  Scalability success rate: {:.1}%", scalability_success_rate);

    assert!(scalability_success_rate >= 50.0, "Scalability tests should have at least 50% success rate");
    
    Ok(())
} 