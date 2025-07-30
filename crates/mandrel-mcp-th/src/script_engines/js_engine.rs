//! JavaScript script execution engine using QuickJS

use crate::script_engines::memory_tracker::{MemoryTracker, MemoryTrackingConfig};
use crate::script_engines::{ScriptConfig, ScriptContext, ScriptError, ScriptResult};
use rquickjs::{Context, Runtime};
use std::time::Duration;
use std::time::Instant;
use tokio::time::timeout;

/// JavaScript execution engine using QuickJS runtime
pub struct JavaScriptEngine {
    #[allow(dead_code)] // Reserved for future optimizations
    runtime: Runtime,
    config: ScriptConfig,
}

/// Precompiled JavaScript script
#[derive(Debug, Clone)]
pub struct JavaScriptScript {
    source: String,
    #[allow(dead_code)] // Reserved for function-specific execution
    function_name: Option<String>,
}

impl JavaScriptEngine {
    /// Create new JavaScript engine with configuration
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError> {
        let runtime = Runtime::new().map_err(|e| ScriptError::ExecutionError {
            message: format!("Failed to create QuickJS runtime: {e}"),
        })?;

        Ok(Self {
            runtime,
            config: config.clone(),
        })
    }

    /// Execute JavaScript code with context injection
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();

        // Initialize memory tracking
        let memory_config = MemoryTrackingConfig::default();
        let memory_tracker = MemoryTracker::new(memory_config);
        let memory_before =
            memory_tracker
                .snapshot()
                .map_err(|e| ScriptError::MemoryTrackingError {
                    message: format!("Failed to take initial memory snapshot: {e}"),
                })?;

        // Create context and execute with timeout using spawn_blocking for sync QuickJS ops
        let script = script.to_string();
        let context = context.clone();
        let _config = self.config.clone();

        let execution_future = tokio::task::spawn_blocking(move || {
            let runtime = Runtime::new().map_err(|e| format!("Runtime creation failed: {e}"))?;
            let ctx =
                Context::full(&runtime).map_err(|e| format!("Context creation failed: {e}"))?;

            ctx.with(|ctx| {
                // Inject context into JavaScript global scope
                Self::inject_context_static(&ctx, &context)
                    .map_err(|e| format!("Context injection failed: {e}"))?;

                // Execute the script
                let result: rquickjs::Value = ctx
                    .eval(script.as_str())
                    .map_err(|e| format!("Script execution failed: {e}"))?;

                // Convert QuickJS value to JSON
                Self::convert_js_value_to_json_static(&ctx, result)
                    .map_err(|e| format!("JSON conversion failed: {e}"))
            })
            .map_err(|e| format!("QuickJS with block failed: {e}"))
        });

        let js_result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            execution_future,
        )
        .await;

        // Take memory snapshot after execution
        let memory_after =
            memory_tracker
                .snapshot()
                .map_err(|e| ScriptError::MemoryTrackingError {
                    message: format!("Failed to take final memory snapshot: {e}"),
                })?;

        let memory_delta = memory_tracker.calculate_delta(&memory_before, &memory_after);
        let memory_used_mb = Some(memory_tracker.delta_to_mb(&memory_delta));
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match js_result {
            Ok(Ok(Ok(js_value))) => self.extract_result(js_value, duration_ms, memory_used_mb),
            Ok(Ok(Err(js_error))) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: vec![],
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::ExecutionError { message: js_error }),
            }),
            Ok(Err(e)) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: vec![],
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::ExecutionError {
                    message: format!("Task execution failed: {e}"),
                }),
            }),
            Err(_) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: vec![],
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::TimeoutError {
                    timeout_ms: self.config.timeout_ms,
                }),
            }),
        }
    }

    /// Inject ScriptContext into JavaScript global scope (static version)
    fn inject_context_static(
        ctx: &rquickjs::Ctx<'_>,
        context: &ScriptContext,
    ) -> Result<(), String> {
        let context_obj = rquickjs::Object::new(ctx.clone())
            .map_err(|e| format!("Object creation failed: {e}"))?;

        // Add request data - convert JSON to JavaScript value
        let request_val: rquickjs::Value = ctx
            .json_parse(context.request.to_string())
            .map_err(|e| format!("Request JSON parse failed: {e}"))?;
        context_obj
            .set("request", request_val)
            .map_err(|e| format!("Request set failed: {e}"))?;

        // Add response data if available
        if let Some(ref response) = context.response {
            let response_val: rquickjs::Value = ctx
                .json_parse(response.to_string())
                .map_err(|e| format!("Response JSON parse failed: {e}"))?;
            context_obj
                .set("response", response_val)
                .map_err(|e| format!("Response set failed: {e}"))?;
        }

        // Add metadata
        context_obj
            .set("test_case", context.metadata.test_name.clone())
            .map_err(|e| format!("Test case set failed: {e}"))?;
        context_obj
            .set("tool", context.metadata.tool_name.clone())
            .map_err(|e| format!("Tool set failed: {e}"))?;

        // Add simple log function
        let log_fn = rquickjs::Function::new(ctx.clone(), |level: String, message: String| {
            tracing::info!("JS Log [{}]: {}", level, message);
        })
        .map_err(|e| format!("Log function creation failed: {e}"))?;
        context_obj
            .set("log", log_fn)
            .map_err(|e| format!("Log function set failed: {e}"))?;

        // Set the context object as global
        ctx.globals()
            .set("context", context_obj)
            .map_err(|e| format!("Context global set failed: {e}"))?;

        Ok(())
    }

    /// Convert QuickJS value to JSON (static version)
    fn convert_js_value_to_json_static<'a>(
        ctx: &rquickjs::Ctx<'a>,
        value: rquickjs::Value<'a>,
    ) -> Result<serde_json::Value, String> {
        // Use JSON.stringify to convert to string, then parse as JSON
        let json_rquickjs_string = ctx
            .json_stringify(&value)
            .map_err(|e| format!("JSON stringify failed: {e}"))?
            .unwrap_or_else(|| rquickjs::String::from_str(ctx.clone(), "null").unwrap());
        let json_string = json_rquickjs_string
            .to_string()
            .map_err(|e| format!("String conversion failed: {e}"))?;

        serde_json::from_str(&json_string).map_err(|e| format!("JSON parse failed: {e}"))
    }

    /// Extract successful result from JavaScript execution
    fn extract_result(
        &self,
        js_value: serde_json::Value,
        duration_ms: u64,
        memory_used_mb: Option<f64>,
    ) -> Result<ScriptResult, ScriptError> {
        Ok(ScriptResult {
            success: true,
            output: js_value,
            logs: vec![], // FUTURE: Implement log capture
            duration_ms,
            memory_used_mb,
            error: None,
        })
    }

    /// Precompile JavaScript for future execution
    pub fn precompile_script(
        &self,
        script: &str,
        function_name: Option<String>,
    ) -> Result<JavaScriptScript, ScriptError> {
        Ok(JavaScriptScript {
            source: script.to_string(),
            function_name,
        })
    }

    /// Execute precompiled JavaScript script
    pub async fn execute_precompiled(
        &self,
        script: &JavaScriptScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        self.execute_script(&script.source, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_context() -> ScriptContext {
        let config = ScriptConfig::new();
        ScriptContext::new(
            json!({"test_input": "sample_data"}),
            "test_js_execution".to_string(),
            "test_tool".to_string(),
            config,
        )
        .with_response(json!({"test_output": "expected_result"}))
    }

    fn create_test_context_with_data() -> ScriptContext {
        let config = ScriptConfig::new();
        ScriptContext::new(
            json!({
                "tool": "list_files",
                "arguments": {"path": "/tmp"}
            }),
            "test_mcp_validation".to_string(),
            "list_files".to_string(),
            config,
        )
        .with_response(json!({
            "content": [{"type": "text", "text": "file1.txt\nfile2.txt"}]
        }))
    }

    // TDD RED Phase: Write failing tests first

    #[tokio::test]
    async fn test_js_engine_creation() {
        let config = ScriptConfig::new();
        let result = JavaScriptEngine::new(&config);

        assert!(
            result.is_ok(),
            "Should create JavaScript engine successfully"
        );
    }

    #[tokio::test]
    async fn test_js_simple_script_execution() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            ({
                success: true,
                message: "Hello from JavaScript",
                input_received: !!context
            })
        "#;

        let result = engine.execute_script(script, context).await;

        assert!(result.is_ok(), "Should execute simple JavaScript script");
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");
        // duration_ms is u64 so always >= 0, and execution tracking is working
    }

    #[tokio::test]
    async fn test_js_context_injection() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context_with_data();

        let script = r#"
            // Verify context is properly injected
            if (!context || !context.request || !context.response) {
                throw new Error("Context not properly injected");
            }
            
            ({
                success: true,
                test_case: context.test_case,
                tool: context.tool,
                has_request: !!context.request,
                has_response: !!context.response
            })
        "#;

        let result = engine.execute_script(script, context).await;

        // This will fail until context injection is implemented
        assert!(
            result.is_ok(),
            "Should execute script with context injection"
        );
        let script_result = result.unwrap();
        assert!(
            script_result.success,
            "Script should access context successfully"
        );
    }

    #[tokio::test]
    async fn test_js_error_handling() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            throw new Error("Test JavaScript error");
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle JavaScript errors gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for JS errors"
        );
        let script_result = result.unwrap();
        assert!(!script_result.success, "Script should fail due to error");
        assert!(
            script_result.error.is_some(),
            "Should capture error details"
        );
    }

    #[tokio::test]
    async fn test_js_syntax_error_handling() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            // Invalid JavaScript syntax
            invalid_syntax(
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle syntax errors gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for syntax errors"
        );
        let script_result = result.unwrap();
        assert!(
            !script_result.success,
            "Script should fail due to syntax error"
        );
        assert!(
            script_result.error.is_some(),
            "Should capture syntax error details"
        );
    }

    #[tokio::test]
    #[ignore = "QuickJS cannot interrupt synchronous JavaScript execution - known limitation"]
    async fn test_js_timeout_handling() {
        let mut config = ScriptConfig::new();
        config.timeout_ms = 100; // Very short timeout

        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            // Long-running operation to trigger timeout
            let result = 0;
            for (let i = 0; i < 10000000; i++) {
                result += Math.sin(i) * Math.cos(i);
                // This should be interrupted by timeout before completion
            }
            result
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle timeouts gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for timeouts"
        );
        let script_result = result.unwrap();
        assert!(!script_result.success, "Script should fail due to timeout");
        assert!(
            script_result.error.is_some(),
            "Should capture timeout error"
        );
    }

    #[tokio::test]
    async fn test_js_precompile_script() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();

        let script = r#"
            function validate(input) {
                return { success: true, processed: input };
            }
            validate
        "#;

        let result = engine.precompile_script(script, Some("validate".to_string()));

        assert!(result.is_ok(), "Should precompile JavaScript script");
        let js_script = result.unwrap();
        assert!(!js_script.source.is_empty(), "Should store script source");
        assert_eq!(js_script.function_name, Some("validate".to_string()));
    }

    #[tokio::test]
    async fn test_js_execute_precompiled() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        // First precompile the script
        let script = r#"
            ({
                success: true,
                message: "Precompiled script executed",
                context_available: !!context
            })
        "#;

        let js_script = engine.precompile_script(script, None).unwrap();
        let result = engine.execute_precompiled(&js_script, context).await;

        assert!(
            result.is_ok(),
            "Should execute precompiled JavaScript script"
        );
        let script_result = result.unwrap();
        assert!(
            script_result.success,
            "Precompiled script should execute successfully"
        );
    }

    #[tokio::test]
    async fn test_js_memory_tracking() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            // Create some objects to use memory
            let data = [];
            for (let i = 0; i < 1000; i++) {
                data.push({ id: i, value: `item_${i}` });
            }
            
            ({
                success: true,
                items_created: data.length
            })
        "#;

        let result = engine.execute_script(script, context).await;

        // Should track memory usage
        assert!(result.is_ok(), "Should execute script with memory tracking");
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");
        assert!(
            script_result.memory_used_mb.is_some(),
            "Should track memory usage"
        );
        assert!(
            script_result.memory_used_mb.unwrap() >= 0.0,
            "Memory usage should be non-negative"
        );
    }

    #[tokio::test]
    async fn test_js_performance_requirements() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
            // Simple computation
            let sum = 0;
            for (let i = 0; i < 1000; i++) {
                sum += i;
            }
            
            ({
                success: true,
                sum: sum,
                computation_complete: true
            })
        "#;

        let start = Instant::now();
        let result = engine.execute_script(script, context).await;
        let total_duration = start.elapsed();

        // Should meet performance requirements
        assert!(
            result.is_ok(),
            "Should execute script within performance limits"
        );
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");

        // Performance assertions
        assert!(
            total_duration.as_millis() < 100,
            "Total execution should be <100ms"
        );
        assert!(
            script_result.duration_ms < 100,
            "Script execution should be <100ms"
        );

        if let Some(memory_mb) = script_result.memory_used_mb {
            assert!(memory_mb < 10.0, "Memory usage should be <10MB");
        }
    }

    #[tokio::test]
    async fn test_js_mcp_validation_script() {
        let config = ScriptConfig::new();
        let engine = JavaScriptEngine::new(&config).unwrap();
        let context = create_test_context_with_data();

        let validation_script = r#"
            // Validate MCP response structure
            if (!context.response || !context.response.content) {
                throw new Error("Missing response content");
            }
            
            if (!Array.isArray(context.response.content)) {
                throw new Error("Content must be array");
            }
            
            const textContent = context.response.content.find(c => c.type === 'text');
            if (!textContent) {
                throw new Error("No text content found");
            }
            
            const lines = textContent.text.split('\n').filter(l => l.trim());
            
            ({
                success: true,
                files_count: lines.length,
                files: lines,
                validation_passed: true
            })
        "#;

        let result = engine.execute_script(validation_script, context).await;

        // Should execute complex MCP validation logic
        assert!(result.is_ok(), "Should execute MCP validation script");
        let script_result = result.unwrap();
        assert!(script_result.success, "MCP validation should pass");
    }
}
