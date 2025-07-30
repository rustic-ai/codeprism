//! Integration Test Framework
//!
//! Provides framework for running end-to-end integration tests
//! with real MCP servers and the complete MOTH test harness.

use std::path::PathBuf;
use tokio::process::Command;

#[derive(Default)]
pub struct IntegrationTestFramework {
    test_data_dir: PathBuf,
}

impl IntegrationTestFramework {
    pub fn new() -> Self {
        Self {
            test_data_dir: PathBuf::from("tests/fixtures"),
        }
    }

    /// Execute a tool test using the real mandrel-mcp-th CLI
    pub async fn execute_tool_test(
        &self,
        tool_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary YAML configuration for the specific tool
        let config_content = self.generate_tool_config(tool_name)?;

        // Write config to temporary file
        let temp_config = self.test_data_dir.join(format!("{tool_name}_config.yaml"));
        std::fs::create_dir_all(&self.test_data_dir)?;
        std::fs::write(&temp_config, config_content)?;

        // Execute moth CLI with the generated config
        self.execute_moth_cli(&["--quiet", "run", temp_config.to_str().unwrap()])
            .await
    }

    /// Execute the moth CLI with given arguments
    pub async fn execute_moth_cli(
        &self,
        _args: &[&str],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Build the binary first to ensure it's available
        let output = Command::new("cargo")
            .args(["build", "--release", "--bin", "moth"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(format!(
                "Failed to build moth binary: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        // Execute the built binary
        let binary_path = std::env::current_dir()?.join("target/release/moth");

        let output = Command::new(binary_path).args(_args).output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "moth CLI failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into())
        }
    }

    /// Generate a YAML configuration for testing a specific tool
    fn generate_tool_config(&self, tool_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Use a simple test project path instead of fixtures
        let test_project = "test-projects/rust-test-project";

        let config = format!(
            r#"name: "Test {} Tool"
version: "1.0.0"
description: "Integration test for {} tool"

capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: true

server:
  command: "cargo"
  args: ["run", "--package", "codeprism-mcp-server", "--bin", "codeprism-mcp-server"]
  env:
    RUST_LOG: "info"
    MCP_PROTOCOL_VERSION: "2025-06-18"
  transport: "stdio"
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10

tools:
  - name: "{}"
    description: "Test {} tool functionality"
    tests:
      - name: "basic_{}_test"
        description: "Basic functionality test for {}"
        input:
          project_path: "{}"
          language: "rust"
        expected:
          error: false
        performance:
          max_duration_ms: 10000
          max_memory_mb: 100
        tags: ["integration", "tool_test"]
"#,
            tool_name, tool_name, tool_name, tool_name, tool_name, tool_name, test_project
        );

        Ok(config)
    }
}
