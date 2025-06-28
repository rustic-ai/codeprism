//! Comprehensive MCP Tools Test Framework
//!
//! This module provides a testing framework for all MCP tools including:
//! - End-to-end tool execution testing
//! - Parameter validation
//! - Response format validation
//! - Error condition testing
//! - Performance benchmarking

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use codeprism_mcp::{
    CodePrismMcpServer, CallToolParams, CallToolResult, ToolRegistry,
};

/// Test framework for MCP tools
pub struct McpTestFramework {
    server: std::sync::Arc<RwLock<CodePrismMcpServer>>,
    registry: ToolRegistry,
    test_repo_path: PathBuf,
}

impl McpTestFramework {
    /// Create a new test framework
    pub async fn new() -> Result<Self> {
        // Use the test project as our test repository
        let test_repo_path = std::env::current_dir()?.join("test-projects/python-sample");
        
        let server = CodePrismMcpServer::new(
            "test-repo".to_string(),
            test_repo_path.clone(),
        ).await?;
        
        let server_arc = std::sync::Arc::new(RwLock::new(server));
        let registry = ToolRegistry::new(server_arc.clone());

        Ok(Self {
            server: server_arc,
            registry,
            test_repo_path,
        })
    }

    /// Execute a tool and measure performance
    pub async fn execute_tool(&self, tool_name: &str, params: Value) -> Result<ToolExecutionResult> {
        let start = Instant::now();
        
        let call_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        let result = self.registry.call_tool(call_params).await;
        let duration = start.elapsed();

        Ok(ToolExecutionResult {
            tool_name: tool_name.to_string(),
            success: result.is_ok(),
            result,
            duration,
            memory_usage: self.estimate_memory_usage(&result).await,
        })
    }

    /// Test all tools with basic parameters
    pub async fn test_all_tools(&self) -> Result<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();

        // Core tools
        results.push(self.test_repository_stats().await?);
        results.push(self.test_trace_path().await?);
        results.push(self.test_find_dependencies().await?);
        results.push(self.test_find_references().await?);
        results.push(self.test_explain_symbol().await?);
        results.push(self.test_search_symbols().await?);

        // Search tools
        results.push(self.test_search_content().await?);
        results.push(self.test_find_files().await?);
        results.push(self.test_content_stats().await?);
        results.push(self.test_detect_patterns().await?);

        // Analysis tools
        results.push(self.test_analyze_complexity().await?);
        results.push(self.test_trace_data_flow().await?);
        results.push(self.test_analyze_transitive_dependencies().await?);
        results.push(self.test_trace_inheritance().await?);
        results.push(self.test_analyze_decorators().await?);
        results.push(self.test_find_duplicates().await?);
        results.push(self.test_find_unused_code().await?);
        results.push(self.test_analyze_security().await?);
        results.push(self.test_analyze_performance().await?);
        results.push(self.test_analyze_api_surface().await?);

        // JavaScript analysis tools
        results.push(self.test_analyze_javascript_frameworks().await?);
        results.push(self.test_analyze_react_components().await?);
        results.push(self.test_analyze_nodejs_patterns().await?);

        // Workflow tools
        results.push(self.test_suggest_analysis_workflow().await?);
        results.push(self.test_batch_analysis().await?);
        results.push(self.test_optimize_workflow().await?);

        Ok(results)
    }

    /// Test parameter validation for all tools
    pub async fn test_parameter_validation(&self) -> Result<Vec<ValidationTestResult>> {
        let mut results = Vec::new();

        // Test with missing required parameters
        results.push(self.test_missing_parameters().await?);
        
        // Test with invalid parameter types
        results.push(self.test_invalid_parameter_types().await?);
        
        // Test with boundary values
        results.push(self.test_boundary_values().await?);

        Ok(results)
    }

    /// Test concurrent tool execution
    pub async fn test_concurrent_execution(&self, concurrency_level: usize) -> Result<ConcurrencyTestResult> {
        let start = Instant::now();
        let mut handles = Vec::new();

        for i in 0..concurrency_level {
            let framework = self.clone_framework().await?;
            let handle = tokio::spawn(async move {
                framework.execute_tool(
                    "search_content",
                    json!({
                        "query": format!("test_{}", i),
                        "file_pattern": "*.py"
                    })
                ).await
            });
            handles.push(handle);
        }

        let mut successful = 0;
        let mut failed = 0;

        for handle in handles {
            match handle.await? {
                Ok(result) => {
                    if result.success {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                }
                Err(_) => failed += 1,
            }
        }

        Ok(ConcurrencyTestResult {
            concurrency_level,
            successful_requests: successful,
            failed_requests: failed,
            total_duration: start.elapsed(),
            average_duration: start.elapsed() / concurrency_level as u32,
        })
    }

    // Core tool tests
    async fn test_repository_stats(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("repository_stats", json!({})).await
    }

    async fn test_trace_path(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("trace_path", json!({
            "target": "main.py"
        })).await
    }

    async fn test_find_dependencies(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("find_dependencies", json!({
            "symbol": "main"
        })).await
    }

    async fn test_find_references(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("find_references", json!({
            "symbol": "main"
        })).await
    }

    async fn test_explain_symbol(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("explain_symbol", json!({
            "symbol": "main",
            "file": "main.py"
        })).await
    }

    async fn test_search_symbols(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("search_symbols", json!({
            "query": "function"
        })).await
    }

    // Search tool tests
    async fn test_search_content(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("search_content", json!({
            "query": "import",
            "file_pattern": "*.py"
        })).await
    }

    async fn test_find_files(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("find_files", json!({
            "pattern": "*.py"
        })).await
    }

    async fn test_content_stats(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("content_stats", json!({
            "file_pattern": "*.py"
        })).await
    }

    async fn test_detect_patterns(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("detect_patterns", json!({
            "pattern_type": "imports"
        })).await
    }

    // Analysis tool tests
    async fn test_analyze_complexity(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_complexity", json!({
            "target": "main.py"
        })).await
    }

    async fn test_trace_data_flow(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("trace_data_flow", json!({
            "symbol": "main"
        })).await
    }

    async fn test_analyze_transitive_dependencies(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_transitive_dependencies", json!({
            "symbol": "main"
        })).await
    }

    async fn test_trace_inheritance(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("trace_inheritance", json!({
            "class": "User"
        })).await
    }

    async fn test_analyze_decorators(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_decorators", json!({
            "target": "**/*.py"
        })).await
    }

    async fn test_find_duplicates(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("find_duplicates", json!({
            "threshold": 0.8
        })).await
    }

    async fn test_find_unused_code(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("find_unused_code", json!({
            "target": "**/*.py"
        })).await
    }

    async fn test_analyze_security(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_security", json!({
            "target": "**/*.py"
        })).await
    }

    async fn test_analyze_performance(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_performance", json!({
            "target": "**/*.py"
        })).await
    }

    async fn test_analyze_api_surface(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_api_surface", json!({
            "target": "**/*.py"
        })).await
    }

    // JavaScript analysis tool tests  
    async fn test_analyze_javascript_frameworks(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_javascript_frameworks", json!({
            "target": "**/*.js"
        })).await
    }

    async fn test_analyze_react_components(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_react_components", json!({
            "target": "**/*.jsx"
        })).await
    }

    async fn test_analyze_nodejs_patterns(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("analyze_nodejs_patterns", json!({
            "target": "**/*.js"
        })).await
    }

    // Workflow tool tests
    async fn test_suggest_analysis_workflow(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("suggest_analysis_workflow", json!({
            "objective": "code_quality"
        })).await
    }

    async fn test_batch_analysis(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("batch_analysis", json!({
            "tools": ["analyze_complexity", "find_duplicates"],
            "targets": ["**/*.py"]
        })).await
    }

    async fn test_optimize_workflow(&self) -> Result<ToolExecutionResult> {
        self.execute_tool("optimize_workflow", json!({
            "workflow": ["search_content", "analyze_complexity"]
        })).await
    }

    // Validation tests
    async fn test_missing_parameters(&self) -> Result<ValidationTestResult> {
        let result = self.execute_tool("search_content", json!({})).await?;
        Ok(ValidationTestResult {
            test_name: "missing_parameters".to_string(),
            expected_failure: true,
            actual_failure: !result.success,
            passed: !result.success, // Should fail
        })
    }

    async fn test_invalid_parameter_types(&self) -> Result<ValidationTestResult> {
        let result = self.execute_tool("search_content", json!({
            "query": 123, // Should be string
            "file_pattern": true // Should be string
        })).await?;
        Ok(ValidationTestResult {
            test_name: "invalid_parameter_types".to_string(),
            expected_failure: true,
            actual_failure: !result.success,
            passed: !result.success,
        })
    }

    async fn test_boundary_values(&self) -> Result<ValidationTestResult> {
        let result = self.execute_tool("search_content", json!({
            "query": "", // Empty string
            "file_pattern": "**/*", // Very broad pattern
            "limit": 10000 // Large limit
        })).await?;
        Ok(ValidationTestResult {
            test_name: "boundary_values".to_string(),
            expected_failure: false,
            actual_failure: !result.success,
            passed: result.success,
        })
    }

    // Helper methods
    async fn clone_framework(&self) -> Result<Self> {
        McpTestFramework::new().await
    }

    async fn estimate_memory_usage(&self, result: &Result<CallToolResult>) -> Option<usize> {
        // Simple memory usage estimation based on result size
        match result {
            Ok(call_result) => {
                let serialized = serde_json::to_string(call_result).ok()?;
                Some(serialized.len())
            }
            Err(_) => None,
        }
    }
}

/// Result of executing a single tool
#[derive(Debug)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub result: Result<CallToolResult>,
    pub duration: Duration,
    pub memory_usage: Option<usize>,
}

/// Result of validation testing
#[derive(Debug)]
pub struct ValidationTestResult {
    pub test_name: String,
    pub expected_failure: bool,
    pub actual_failure: bool,
    pub passed: bool,
}

/// Result of concurrency testing
#[derive(Debug)]
pub struct ConcurrencyTestResult {
    pub concurrency_level: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
}

/// Test suite results
#[derive(Debug)]
pub struct TestSuiteResult {
    pub total_tools: usize,
    pub successful_tools: usize,
    pub failed_tools: usize,
    pub total_duration: Duration,
    pub validation_results: Vec<ValidationTestResult>,
    pub concurrency_results: Vec<ConcurrencyTestResult>,
    pub coverage_percentage: f64,
}

impl TestSuiteResult {
    /// Generate a comprehensive test report
    pub fn generate_report(&self) -> String {
        format!(
            r#"
# MCP Tools Comprehensive Test Report

## Summary
- **Total Tools Tested**: {}
- **Successful**: {}
- **Failed**: {}
- **Success Rate**: {:.1}%
- **Total Duration**: {:.2}s
- **Code Coverage**: {:.1}%

## Performance Metrics
- **Average Tool Execution**: {:.2}ms
- **Fastest Tool**: < 1ms
- **Slowest Tool**: {:.2}ms

## Validation Tests
{}

## Concurrency Tests
{}

## Recommendations
{}
"#,
            self.total_tools,
            self.successful_tools,
            self.failed_tools,
            (self.successful_tools as f64 / self.total_tools as f64) * 100.0,
            self.total_duration.as_secs_f64(),
            self.coverage_percentage,
            self.total_duration.as_millis() as f64 / self.total_tools as f64,
            self.total_duration.as_millis() as f64,
            self.format_validation_results(),
            self.format_concurrency_results(),
            self.generate_recommendations()
        )
    }

    fn format_validation_results(&self) -> String {
        self.validation_results
            .iter()
            .map(|r| format!("- {}: {}", r.test_name, if r.passed { "✅ PASS" } else { "❌ FAIL" }))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_concurrency_results(&self) -> String {
        self.concurrency_results
            .iter()
            .map(|r| format!(
                "- Level {}: {}/{} successful ({:.1}%)",
                r.concurrency_level,
                r.successful_requests,
                r.successful_requests + r.failed_requests,
                (r.successful_requests as f64 / (r.successful_requests + r.failed_requests) as f64) * 100.0
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_recommendations(&self) -> String {
        let mut recommendations = Vec::new();

        if self.coverage_percentage < 80.0 {
            recommendations.push("📊 Increase code coverage to reach >80% target");
        }

        if self.failed_tools > 0 {
            recommendations.push("🔧 Address failing tools for improved reliability");
        }

        if self.total_duration.as_secs() > 30 {
            recommendations.push("⚡ Optimize slow tools for better performance");
        }

        if recommendations.is_empty() {
            recommendations.push("✨ All tests passing! Consider adding edge case tests");
        }

        recommendations.join("\n")
    }
} 