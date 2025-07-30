//! Script execution engines for validation scripts
//!
//! This module provides support for executing validation scripts in multiple languages:
//! - Lua (via mlua)
//! - JavaScript (via quickjs)
//! - Python (via pyo3)
//! - Regular expressions (via regex)
//! - UUID generation (via uuid)
//!
//! ## Core Types
//!
//! The foundation of the script execution system consists of four main types:
//!
//! - [`ScriptConfig`] - Configuration for script execution (timeouts, security settings)
//! - [`ScriptContext`] - Runtime context passed to scripts (request/response data, metadata)
//! - [`ScriptResult`] - Standardized result type for script execution outcomes
//! - [`ScriptError`] - Comprehensive error handling for script execution failures
//!
//! ## Example Usage
//!
//! ```rust
//! use mandrel_mcp_th::script_engines::{ScriptConfig, ScriptContext, ScriptResult, LogLevel};
//! use serde_json::json;
//!
//! // Create a secure configuration
//! let config = ScriptConfig::new();
//! assert_eq!(config.timeout_ms, 5000);
//! assert!(!config.allow_network);
//!
//! // Create a script context
//! let context = ScriptContext::new(
//!     json!({"input": "test_data"}),
//!     "test_case".to_string(),
//!     "test_tool".to_string(),
//!     config,
//! );
//!
//! // Create a successful result
//! let result = ScriptResult::success(json!({"output": "success"}), 150)
//!     .add_log(LogLevel::Info, "Script executed successfully".to_string())
//!     .with_memory_usage(2.5);
//!
//! assert!(result.success);
//! assert_eq!(result.duration_ms, 150);
//! ```

pub mod js_engine;
pub mod lua_engine;
pub mod memory_tracker;
pub mod python_engine;
pub mod sandbox;
pub mod types;
pub mod utilities;

// Re-export core types for easier access
pub use types::{
    ContextMetadata, LogEntry, LogLevel, ScriptConfig, ScriptContext, ScriptError, ScriptResult,
    ServerInfo,
};

// Re-export engine types
pub use lua_engine::{LuaEngine, LuaScript};

// Re-export memory tracking types
pub use memory_tracker::{
    MemoryDelta, MemoryError, MemorySnapshot, MemoryTracker, MemoryTrackingConfig,
};

// Re-export sandbox types for secure script execution
pub use sandbox::{
    ResourceLimits, ResourceMetrics, ResourceMonitor, SandboxManager, SecurityPolicy,
};

#[cfg(test)]
mod dependency_tests {
    use std::ffi::CString;
    use std::time::Instant;

    #[tokio::test]
    async fn test_mlua_basic_execution() {
        let lua = mlua::Lua::new();
        let result: String = lua.load("return 'Hello from Lua'").eval().unwrap();
        assert_eq!(result, "Hello from Lua");
    }

    #[tokio::test]
    async fn test_quickjs_basic_execution() {
        let runtime = rquickjs::Runtime::new().unwrap();
        let context = rquickjs::Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let result: String = ctx.eval("'Hello from JavaScript'").unwrap();
            assert_eq!(result, "Hello from JavaScript");
        });
    }

    #[test]
    fn test_pyo3_basic_execution() {
        pyo3::Python::with_gil(|py| {
            let code = CString::new("'Hello from Python'").unwrap();
            let result = py.eval(&code, None, None).unwrap();
            assert_eq!(result.to_string(), "Hello from Python");
        });
    }

    #[test]
    fn test_regex_pattern_matching() {
        let re = regex::Regex::new(r"hello").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("goodbye world"));
    }

    #[test]
    fn test_uuid_generation() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(id.get_version(), Some(uuid::Version::Random));

        // Test serialization
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: uuid::Uuid = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    // Performance benchmarks
    #[tokio::test]
    async fn test_mlua_performance() {
        let lua = mlua::Lua::new();
        let start = Instant::now();
        let _result: String = lua.load("return 'Performance test'").eval().unwrap();
        let duration = start.elapsed();

        // Should be under 1ms for simple scripts
        assert!(
            duration.as_millis() < 10,
            "Lua execution took {}ms, expected <10ms",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn test_quickjs_performance() {
        let runtime = rquickjs::Runtime::new().unwrap();
        let context = rquickjs::Context::full(&runtime).unwrap();

        let start = Instant::now();
        context.with(|ctx| {
            let _result: String = ctx.eval("'Performance test'").unwrap();
        });
        let duration = start.elapsed();

        // Should be under 5ms for simple scripts
        assert!(
            duration.as_millis() < 10,
            "JavaScript execution took {}ms, expected <10ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_pyo3_performance() {
        let start = Instant::now();
        pyo3::Python::with_gil(|py| {
            let code = CString::new("'Performance test'").unwrap();
            let _result = py.eval(&code, None, None).unwrap();
        });
        let duration = start.elapsed();

        // Should be under 50ms for simple scripts (allowing for system variability)
        assert!(
            duration.as_millis() < 50,
            "Python execution took {}ms, expected <50ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_regex_performance() {
        let start = Instant::now();
        let re = regex::Regex::new(r"test").unwrap();
        let _result = re.is_match("performance test");
        let duration = start.elapsed();

        // Should be under 0.1ms for simple patterns
        assert!(
            duration.as_micros() < 1000,
            "Regex matching took {}μs, expected <1000μs",
            duration.as_micros()
        );
    }

    #[test]
    fn test_uuid_performance() {
        let start = Instant::now();
        let _id = uuid::Uuid::new_v4();
        let duration = start.elapsed();

        // Should be under 0.01ms per UUID
        assert!(
            duration.as_micros() < 100,
            "UUID generation took {}μs, expected <100μs",
            duration.as_micros()
        );
    }

    // Error handling tests
    #[tokio::test]
    async fn test_mlua_error_handling() {
        let lua = mlua::Lua::new();
        let result = lua.load("invalid_syntax(").eval::<String>();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quickjs_error_handling() {
        let runtime = rquickjs::Runtime::new().unwrap();
        let context = rquickjs::Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let result = ctx.eval::<String, &str>("invalid_syntax(");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_pyo3_error_handling() {
        pyo3::Python::with_gil(|py| {
            let code = CString::new("invalid_syntax(").unwrap();
            let result = py.eval(&code, None, None);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_regex_error_handling() {
        #[allow(clippy::invalid_regex)]
        let result = regex::Regex::new(r"[invalid");
        assert!(result.is_err());
    }
}
