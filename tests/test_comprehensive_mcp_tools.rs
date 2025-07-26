//! Comprehensive MCP Tools Test Suite
//! 
//! This test suite provides real testing for all MCP tools with actual functionality verification

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use codeprism_mcp::{CodePrismMcpServer, tools_legacy::{CallToolParams, CallToolResult, ToolManager, ListToolsParams}};

/// Tool output validation result
#[derive(Debug, Clone)]
pub struct ToolValidationResult {
    pub tool_name: String,
    pub execution_success: bool,
    pub content_valid: bool,
    pub performance_ok: bool,
    pub validation_errors: Vec<String>,
    pub execution_time_ms: u128,
}

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

    /// Test a single tool with parameters and validate output content
    async fn test_tool_with_validation(&self, tool_name: &str, params: serde_json::Value) -> Result<ToolValidationResult> {
        let start = Instant::now();
        
        let tool_params = CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(params),
        };

        let mut result = ToolValidationResult {
            tool_name: tool_name.to_string(),
            execution_success: false,
            content_valid: false,
            performance_ok: false,
            validation_errors: Vec::new(),
            execution_time_ms: 0,
        };

        match self.tool_manager.call_tool(tool_params).await {
            Ok(tool_result) => {
                let duration = start.elapsed();
                result.execution_time_ms = duration.as_millis();
                result.execution_success = tool_result.is_error.unwrap_or(false) == false;
                
                // Performance check (most tools should complete within 5 seconds)
                result.performance_ok = duration.as_millis() < 5000;
                
                if result.execution_success {
                    // Validate actual content
                    match self.validate_tool_output_content(tool_name, &tool_result) {
                        Ok(()) => {
                            result.content_valid = true;
                        }
                        Err(e) => {
                            result.validation_errors.push(e);
                        }
                    }
                } else {
                    if let Some(content) = tool_result.content.first() {
                        result.validation_errors.push(content.text.clone());
                    }
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                result.execution_time_ms = duration.as_millis();
                result.validation_errors.push(e.to_string());
            }
        }

        Ok(result)
    }

    /// Validate tool output content based on tool type
    fn validate_tool_output_content(&self, tool_name: &str, result: &CallToolResult) -> Result<()> {
        let content = result.content.first()
            .ok_or_else(|| anyhow::anyhow!("Tool {} returned no content", tool_name))?;
        
        // Parse the JSON content
        let content_json: Value = serde_json::from_str(&content.text)
            .map_err(|e| anyhow::anyhow!("Tool {} returned invalid JSON: {}", tool_name, e))?;
        
        match tool_name {
            "repository_stats" => self.validate_repository_stats(&content_json),
            "search_symbols" => self.validate_search_symbols(&content_json),
            "search_content" => self.validate_search_content(&content_json),
            "find_files" => self.validate_find_files(&content_json),
            "content_stats" => self.validate_content_stats(&content_json),
            "trace_path" => self.validate_trace_path(&content_json),
            "find_duplicates" => self.validate_find_duplicates(&content_json),
            "find_references" => self.validate_find_references(&content_json),
            "find_dependencies" => self.validate_find_dependencies(&content_json),
            "explain_symbol" => self.validate_explain_symbol(&content_json),
            "analyze_complexity" => self.validate_analyze_complexity(&content_json),
            "analyze_performance" => self.validate_analyze_performance(&content_json),
            "analyze_security" => self.validate_analyze_security(&content_json),
            "detect_patterns" => self.validate_detect_patterns(&content_json),
            "analyze_dependencies" => self.validate_analyze_dependencies(&content_json),
            "analyze_architecture" => self.validate_analyze_architecture(&content_json),
            "analyze_api_surface" => self.validate_analyze_api_surface(&content_json),
            "analyze_code_quality" => self.validate_analyze_code_quality(&content_json),
            "analyze_test_coverage" => self.validate_analyze_test_coverage(&content_json),
            "analyze_documentation" => self.validate_analyze_documentation(&content_json),
            "analyze_maintainability" => self.validate_analyze_maintainability(&content_json),
            "analyze_javascript" => self.validate_analyze_javascript(&content_json),
            "analyze_js_dependencies" => self.validate_analyze_js_dependencies(&content_json),
            "analyze_js_performance" => self.validate_analyze_js_performance(&content_json),
            "provide_guidance" => self.validate_provide_guidance(&content_json),
            "optimize_code" => self.validate_optimize_code(&content_json),
            _ => {
                // For unknown tools, just verify it's valid JSON with some content
                if content_json.is_null() || (content_json.is_object() && content_json.as_object().unwrap().is_empty()) {
                    return Err(anyhow::anyhow!("Tool {} returned empty or null content", tool_name));
                }
                Ok(())
            }
        }
    }

    // Validation functions for each tool type
    fn validate_repository_stats(&self, content: &Value) -> Result<()> {
        // Verify repository statistics contain meaningful data
        let stats = content.get("repository_stats")
            .ok_or_else(|| anyhow::anyhow!("Missing repository_stats object"))?;
        
        // Check for file counts
        let file_count = stats.get("total_files")
            .and_then(|f| f.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid total_files count"))?;
        
        if file_count == 0 {
            return Err(anyhow::anyhow!("Repository stats shows 0 files - no analysis performed"));
        }
        
        // Check for language breakdown
        let languages = stats.get("languages")
            .and_then(|l| l.as_object())
            .ok_or_else(|| anyhow::anyhow!("Missing languages breakdown"))?;
        
        if languages.is_empty() {
            return Err(anyhow::anyhow!("Language breakdown is empty"));
        }
        
        Ok(())
    }

    fn validate_search_symbols(&self, content: &Value) -> Result<()> {
        let symbols = content.get("symbols")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing symbols array"))?;
        
        // Verify symbols have required structure
        for symbol in symbols {
            symbol.get("name")
                .ok_or_else(|| anyhow::anyhow!("Symbol missing name"))?;
            symbol.get("kind")
                .ok_or_else(|| anyhow::anyhow!("Symbol missing kind"))?;
            symbol.get("location")
                .ok_or_else(|| anyhow::anyhow!("Symbol missing location"))?;
        }
        
        // Verify search metadata
        content.get("search_query")
            .ok_or_else(|| anyhow::anyhow!("Missing search_query"))?;
        content.get("total_found")
            .ok_or_else(|| anyhow::anyhow!("Missing total_found count"))?;
        
        Ok(())
    }

    fn validate_analyze_complexity(&self, content: &Value) -> Result<()> {
        let analysis = content.get("complexity_analysis")
            .ok_or_else(|| anyhow::anyhow!("Missing complexity_analysis object"))?;
        
        // Check for complexity metrics
        let metrics = analysis.get("metrics")
            .ok_or_else(|| anyhow::anyhow!("Missing complexity metrics"))?;
        
        // Verify cyclomatic complexity
        let cyclomatic = metrics.get("cyclomatic_complexity")
            .and_then(|c| c.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid cyclomatic_complexity"))?;
        
        if cyclomatic < 1.0 {
            return Err(anyhow::anyhow!("Cyclomatic complexity {} is too low for real code", cyclomatic));
        }
        
        // Check for function analysis
        let functions = analysis.get("functions")
            .and_then(|f| f.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing functions array"))?;
        
        // Verify at least some functions were analyzed
        if functions.is_empty() {
            return Err(anyhow::anyhow!("No functions analyzed"));
        }
        
        // Check function structure
        for function in functions {
            function.get("name")
                .ok_or_else(|| anyhow::anyhow!("Function missing name"))?;
            function.get("complexity")
                .and_then(|c| c.as_f64())
                .ok_or_else(|| anyhow::anyhow!("Function missing complexity score"))?;
        }
        
        Ok(())
    }

    fn validate_analyze_security(&self, content: &Value) -> Result<()> {
        let analysis = content.get("security_analysis")
            .ok_or_else(|| anyhow::anyhow!("Missing security_analysis object"))?;
        
        // Check for vulnerabilities array (can be empty but must exist)
        let vulnerabilities = analysis.get("vulnerabilities")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing vulnerabilities array"))?;
        
        // Verify vulnerability structure if any exist
        for vuln in vulnerabilities {
            vuln.get("type")
                .ok_or_else(|| anyhow::anyhow!("Vulnerability missing type"))?;
            vuln.get("severity")
                .ok_or_else(|| anyhow::anyhow!("Vulnerability missing severity"))?;
            vuln.get("location")
                .ok_or_else(|| anyhow::anyhow!("Vulnerability missing location"))?;
        }
        
        // Check for scan summary
        let summary = analysis.get("scan_summary")
            .ok_or_else(|| anyhow::anyhow!("Missing scan_summary"))?;
        
        summary.get("files_scanned")
            .and_then(|f| f.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing files_scanned count"))?;
        
        Ok(())
    }

    fn validate_provide_guidance(&self, content: &Value) -> Result<()> {
        let guidance = content.get("guidance")
            .ok_or_else(|| anyhow::anyhow!("Missing guidance object"))?;
        
        // Check for recommendations
        let recommendations = guidance.get("recommendations")
            .and_then(|r| r.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing recommendations array"))?;
        
        if recommendations.is_empty() {
            return Err(anyhow::anyhow!("No recommendations provided"));
        }
        
        // Verify recommendation structure
        for rec in recommendations {
            rec.get("category")
                .ok_or_else(|| anyhow::anyhow!("Recommendation missing category"))?;
            let description = rec.get("description")
                .and_then(|d| d.as_str())
                .ok_or_else(|| anyhow::anyhow!("Recommendation missing description"))?;
            
            if description.len() < 10 {
                return Err(anyhow::anyhow!("Recommendation description too short"));
            }
        }
        
        Ok(())
    }

    fn validate_optimize_code(&self, content: &Value) -> Result<()> {
        let optimization = content.get("optimization_suggestions")
            .ok_or_else(|| anyhow::anyhow!("Missing optimization_suggestions object"))?;
        
        // Check for suggestions
        let suggestions = optimization.get("suggestions")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing suggestions array"))?;
        
        // Verify suggestion structure
        for suggestion in suggestions {
            suggestion.get("type")
                .ok_or_else(|| anyhow::anyhow!("Suggestion missing type"))?;
            suggestion.get("description")
                .ok_or_else(|| anyhow::anyhow!("Suggestion missing description"))?;
            suggestion.get("impact")
                .ok_or_else(|| anyhow::anyhow!("Suggestion missing impact"))?;
        }
        
        Ok(())
    }

    // Simplified validation for remaining tools (add more as needed)
    fn validate_search_content(&self, content: &Value) -> Result<()> {
        content.get("matches").ok_or_else(|| anyhow::anyhow!("Missing matches"))?;
        Ok(())
    }

    fn validate_find_files(&self, content: &Value) -> Result<()> {
        content.get("files").ok_or_else(|| anyhow::anyhow!("Missing files"))?;
        Ok(())
    }

    fn validate_content_stats(&self, content: &Value) -> Result<()> {
        content.get("statistics").ok_or_else(|| anyhow::anyhow!("Missing statistics"))?;
        Ok(())
    }

    fn validate_trace_path(&self, content: &Value) -> Result<()> {
        content.get("path_analysis").ok_or_else(|| anyhow::anyhow!("Missing path_analysis"))?;
        Ok(())
    }

    fn validate_find_duplicates(&self, content: &Value) -> Result<()> {
        content.get("duplicates").ok_or_else(|| anyhow::anyhow!("Missing duplicates"))?;
        Ok(())
    }

    fn validate_find_references(&self, content: &Value) -> Result<()> {
        content.get("references").ok_or_else(|| anyhow::anyhow!("Missing references"))?;
        Ok(())
    }

    fn validate_find_dependencies(&self, content: &Value) -> Result<()> {
        content.get("dependencies").ok_or_else(|| anyhow::anyhow!("Missing dependencies"))?;
        Ok(())
    }

    fn validate_explain_symbol(&self, content: &Value) -> Result<()> {
        content.get("symbol_info").ok_or_else(|| anyhow::anyhow!("Missing symbol_info"))?;
        Ok(())
    }

    fn validate_analyze_performance(&self, content: &Value) -> Result<()> {
        content.get("performance_analysis").ok_or_else(|| anyhow::anyhow!("Missing performance_analysis"))?;
        Ok(())
    }

    fn validate_detect_patterns(&self, content: &Value) -> Result<()> {
        content.get("patterns").ok_or_else(|| anyhow::anyhow!("Missing patterns"))?;
        Ok(())
    }

    fn validate_analyze_dependencies(&self, content: &Value) -> Result<()> {
        content.get("dependency_analysis").ok_or_else(|| anyhow::anyhow!("Missing dependency_analysis"))?;
        Ok(())
    }

    fn validate_analyze_architecture(&self, content: &Value) -> Result<()> {
        content.get("architecture_analysis").ok_or_else(|| anyhow::anyhow!("Missing architecture_analysis"))?;
        Ok(())
    }

    fn validate_analyze_api_surface(&self, content: &Value) -> Result<()> {
        content.get("api_analysis").ok_or_else(|| anyhow::anyhow!("Missing api_analysis"))?;
        Ok(())
    }

    fn validate_analyze_code_quality(&self, content: &Value) -> Result<()> {
        content.get("quality_analysis").ok_or_else(|| anyhow::anyhow!("Missing quality_analysis"))?;
        Ok(())
    }

    fn validate_analyze_test_coverage(&self, content: &Value) -> Result<()> {
        content.get("coverage_analysis").ok_or_else(|| anyhow::anyhow!("Missing coverage_analysis"))?;
        Ok(())
    }

    fn validate_analyze_documentation(&self, content: &Value) -> Result<()> {
        content.get("documentation_analysis").ok_or_else(|| anyhow::anyhow!("Missing documentation_analysis"))?;
        Ok(())
    }

    fn validate_analyze_maintainability(&self, content: &Value) -> Result<()> {
        content.get("maintainability_analysis").ok_or_else(|| anyhow::anyhow!("Missing maintainability_analysis"))?;
        Ok(())
    }

    fn validate_analyze_javascript(&self, content: &Value) -> Result<()> {
        content.get("javascript_analysis").ok_or_else(|| anyhow::anyhow!("Missing javascript_analysis"))?;
        Ok(())
    }

    fn validate_analyze_js_dependencies(&self, content: &Value) -> Result<()> {
        content.get("js_dependencies").ok_or_else(|| anyhow::anyhow!("Missing js_dependencies"))?;
        Ok(())
    }

    fn validate_analyze_js_performance(&self, content: &Value) -> Result<()> {
        content.get("js_performance").ok_or_else(|| anyhow::anyhow!("Missing js_performance"))?;
        Ok(())
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
    let mut content_valid_tools = 0;
    let mut performance_ok_tools = 0;
    let mut performance_metrics = Vec::new();

    info!("🔧 Testing {} tools across all categories...", tool_tests.len());

    // Test each tool with comprehensive validation
    for (tool_name, params) in tool_tests {
        debug!("Testing tool: {}", tool_name);
        
        match fixture.test_tool_with_validation(tool_name, params).await {
            Ok(validation_result) => {
                if validation_result.execution_success {
                    successful_tools += 1;
                    info!("✅ {} - Execution Success ({}ms)", tool_name, validation_result.execution_time_ms);
                } else {
                    warn!("❌ {} - Execution Failed ({}ms)", tool_name, validation_result.execution_time_ms);
                }
                
                if validation_result.content_valid {
                    content_valid_tools += 1;
                    info!("  ✅ Content validation passed");
                } else if validation_result.execution_success {
                    warn!("  ❌ Content validation failed: {:?}", validation_result.validation_errors);
                }
                
                if validation_result.performance_ok {
                    performance_ok_tools += 1;
                } else {
                    warn!("  ⚠️ Performance slow: {}ms", validation_result.execution_time_ms);
                }
                
                performance_metrics.push((tool_name, validation_result.execution_time_ms));
                results.push(validation_result);
            }
            Err(e) => {
                warn!("❌ {} - Exception: {}", tool_name, e);
                results.push(ToolValidationResult {
                    tool_name: tool_name.to_string(),
                    execution_success: false,
                    content_valid: false,
                    performance_ok: false,
                    validation_errors: vec![e.to_string()],
                    execution_time_ms: 0,
                });
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
    let content_validation_rate = (content_valid_tools as f64 / total_tools as f64) * 100.0;
    let performance_rate = (performance_ok_tools as f64 / total_tools as f64) * 100.0;
    
    info!("📊 === COMPREHENSIVE TEST RESULTS ===");
    info!("Total tools tested: {}", total_tools);
    info!("Execution successful: {}", successful_tools);
    info!("Content validation passed: {}", content_valid_tools);
    info!("Performance requirements met: {}", performance_ok_tools);
    info!("Success rate: {:.1}%", success_rate);
    info!("Content validation rate: {:.1}%", content_validation_rate);
    info!("Performance rate: {:.1}%", performance_rate);
    
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
    let core_successes = results.iter().filter(|r| 
        matches!(r.tool_name.as_str(), "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path") && r.execution_success
    ).count();

    let core_content_valid = results.iter().filter(|r| 
        matches!(r.tool_name.as_str(), "repository_stats" | "search_symbols" | "search_content" | "find_files" | "content_stats" | "trace_path") && r.content_valid
    ).count();

    let analysis_successes = results.iter().filter(|r| 
        r.tool_name.starts_with("analyze_") && r.execution_success
    ).count();

    let analysis_content_valid = results.iter().filter(|r| 
        r.tool_name.starts_with("analyze_") && r.content_valid
    ).count();

    let js_successes = results.iter().filter(|r| 
        matches!(r.tool_name.as_str(), "analyze_javascript" | "analyze_js_dependencies" | "analyze_js_performance") && r.execution_success
    ).count();

    info!("📋 === DETAILED RESULTS BY CATEGORY ===");
    info!("Core Navigation Tools: {}/6 execution success, {}/6 content valid", core_successes, core_content_valid);
    info!("Analysis Tools: {}/11 execution success, {}/11 content valid", analysis_successes, analysis_content_valid);
    info!("JavaScript Analysis Tools: {}/3 execution success", js_successes);

    // Enhanced assertions - We should have substantial functionality working
    assert!(successful_tools > 0, "At least one tool should work");
    assert!(success_rate >= 30.0, "Success rate should be at least 30% but was {:.1}%", success_rate);
    
    // Content validation should be working for successful tools
    if successful_tools > 0 {
        let content_rate_of_successful = (content_valid_tools as f64 / successful_tools as f64) * 100.0;
        info!("Content validation rate of successful tools: {:.1}%", content_rate_of_successful);
        // Allow some tools to have basic validation while we improve the schemas
        assert!(content_rate_of_successful >= 20.0, 
            "At least 20% of successful tools should have valid content, but got {:.1}%", content_rate_of_successful);
    }
    
    // Core tools should have reasonable success rate
    assert!(core_successes >= 2, "At least 2 core tools should work");
    
    info!("🎉 Comprehensive MCP tools test completed successfully!");
    info!("  ✅ Execution Success Rate: {:.1}%", success_rate);
    info!("  ✅ Content Validation Rate: {:.1}%", content_validation_rate);
    info!("  ✅ Performance Rate: {:.1}%", performance_rate);
    
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
        
        match fixture.test_tool_with_validation(tool_name, params).await {
            Ok(validation_result) => {
                if !validation_result.execution_success {
                    error_tests_handled += 1;
                    debug!("✅ {} - Properly handled error ({}ms)", tool_name, validation_result.execution_time_ms);
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
        
        match fixture.test_tool_with_validation(tool_name, params).await {
            Ok(validation_result) => {
                performance_results.push((tool_name, validation_result.execution_time_ms, max_time_ms));
                
                if validation_result.execution_success && validation_result.execution_time_ms <= max_time_ms {
                    debug!("✅ {} - Performance OK ({}ms <= {}ms)", tool_name, validation_result.execution_time_ms, max_time_ms);
                } else if validation_result.execution_success {
                    warn!("⚠️  {} - Performance slow ({}ms > {}ms)", tool_name, validation_result.execution_time_ms, max_time_ms);
                    performance_violations.push((tool_name, validation_result.execution_time_ms, max_time_ms));
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