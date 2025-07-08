//! Integration Test Framework
//!
//! Provides framework for running end-to-end integration tests
//! with real MCP servers and the complete MOTH test harness.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Main integration test framework for real server testing
pub struct IntegrationTestFramework {
    pub test_data_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub moth_specs_dir: PathBuf,
}

impl Default for IntegrationTestFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationTestFramework {
    pub fn new() -> Self {
        Self {
            test_data_dir: PathBuf::from("tests/fixtures"),
            reports_dir: PathBuf::from("target/test-reports"),
            temp_dir: PathBuf::from("target/tmp"),
            moth_specs_dir: PathBuf::from("crates/codeprism-moth-specs"),
        }
    }

    pub async fn setup(&mut self) -> Result<(), IntegrationTestError> {
        // Setup test environment
        Ok(())
    }

    pub async fn teardown(&mut self) -> Result<(), IntegrationTestError> {
        // Cleanup test environment
        Ok(())
    }

    pub async fn run_cli_test(
        &self,
        spec_file: &str,
        _args: &[&str],
    ) -> Result<TestResult, IntegrationTestError> {
        // Execute the MOTH CLI with real server and test specification
        let start_time = std::time::Instant::now();

        // Execute the actual CLI command using the mandrel-mcp-th binary
        let output = std::process::Command::new("cargo")
            .args(["run", "--bin", "moth", "--", "run", spec_file])
            .output()
            .map_err(|e| {
                IntegrationTestError::CliExecution(format!("Failed to execute CLI: {}", e))
            })?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(TestResult {
            exit_code,
            stdout,
            stderr,
            execution_time: start_time.elapsed(),
            generated_files: vec![
                self.reports_dir.join("test_report.html"),
                self.reports_dir.join("test_report.json"),
            ],
        })
    }

    pub async fn execute_tool_test(
        &self,
        tool_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create a temporary test file for tools that need it
        let temp_file = self.create_temp_test_file().await?;

        // Generate proper YAML configuration
        let config_content = Self::generate_tool_config(tool_name, &temp_file);

        // Create temporary config file
        let config_file = format!(
            "{}/test_config_{}.yaml",
            std::env::temp_dir().display(),
            tool_name
        );

        tokio::fs::write(&config_file, config_content)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        // Use the correct path to the moth binary
        let moth_binary = if cfg!(debug_assertions) {
            "../../target/debug/moth"
        } else {
            "../../target/release/moth"
        };

        // Execute mandrel-mcp-th CLI
        let output = std::process::Command::new(moth_binary)
            .arg("--quiet")
            .arg("run")
            .arg(&config_file)
            .output()
            .map_err(|e| format!("Failed to execute moth command: {}", e))?;

        // Clean up temporary files
        let _ = std::fs::remove_file(&config_file);
        let _ = std::fs::remove_file(&temp_file);

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Return stderr as error for debugging
            Err(format!(
                "Tool test failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into())
        }
    }

    async fn create_temp_test_file(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = std::env::temp_dir();
        let temp_file = format!("{}/test_rust_code.rs", temp_dir.display());

        // Create a sample Rust file for testing
        let rust_content = r#"// Sample Rust code for testing
use std::collections::HashMap;

pub struct User {
    id: u64,
    name: String,
    email: String,
}

impl User {
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
    
    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

pub fn create_user_map() -> HashMap<u64, User> {
    let mut users = HashMap::new();
    users.insert(1, User::new(1, "Alice".to_string(), "alice@example.com".to_string()));
    users.insert(2, User::new(2, "Bob".to_string(), "bob@example.com".to_string()));
    users
}

// Function with some complexity for analysis
pub fn complex_function(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if *item > 0 {
            if *item % 2 == 0 {
                result.push(*item * 2);
            } else {
                result.push(*item + 1);
            }
        }
    }
    result
}
"#;

        tokio::fs::write(&temp_file, rust_content)
            .await
            .map_err(|e| format!("Failed to create temp test file: {}", e))?;

        Ok(temp_file)
    }

    /// Get available CodePrism moth specifications
    pub fn get_codeprism_specs(&self) -> Result<Vec<PathBuf>, IntegrationTestError> {
        let comprehensive_dir = self.moth_specs_dir.join("codeprism/comprehensive");
        let mut specs = Vec::new();

        if comprehensive_dir.exists() {
            for entry in std::fs::read_dir(&comprehensive_dir)
                .map_err(|e| IntegrationTestError::Setup(e.to_string()))?
            {
                let entry = entry.map_err(|e| IntegrationTestError::Setup(e.to_string()))?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    specs.push(path);
                }
            }
        }

        Ok(specs)
    }

    /// Validate report generation
    pub async fn validate_reports(
        &self,
        _expected_reports: &[ReportExpectation],
    ) -> Result<(), IntegrationTestError> {
        // Validate that expected reports were generated
        // Implementation will verify report content and structure
        Ok(())
    }

    fn generate_tool_config(tool_name: &str, target_file: &str) -> String {
        format!(
            r#"name: "CodePrism MCP Server Tool Test - {}"
version: "1.0.0"
description: "Test configuration for {} tool validation"

capabilities:
  tools: true
  resources: true
  prompts: false
  sampling: false
  logging: true

server:
  command: "codeprism-mcp-server"
  args: []
  env:
    LOG_LEVEL: "warn"
  transport: "stdio"
  startup_timeout_seconds: 15
  shutdown_timeout_seconds: 8

tools:
  - name: "{}"
    description: "Test {} tool functionality"
    tests:
      - name: "test_{}_basic"
        description: "Basic functionality test for {}"
        input:
          target: "{}"
        expected:
          error: false
          fields:
            - path: "$.content"
              field_type: "string"
              required: true
        tags: ["codeprism", "{}"]
"#,
            tool_name,
            tool_name,
            tool_name,
            tool_name,
            tool_name,
            tool_name,
            target_file,
            tool_name
        )
    }
}

/// Test result structure
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub generated_files: Vec<PathBuf>,
}

/// Report validation expectations
pub struct ReportExpectation {
    pub format: ReportFormat,
    pub file_path: PathBuf,
    pub expected_content: Vec<ContentExpectation>,
}

/// Report formats
pub enum ReportFormat {
    Html,
    Json,
    JunitXml,
    Markdown,
}

/// Content validation expectations
pub enum ContentExpectation {
    ContainsText(String),
    ElementCount(String, usize),
    JsonPath(String, serde_json::Value),
    FileSize(std::ops::Range<u64>),
}

/// Integration test errors
#[derive(Error, Debug)]
pub enum IntegrationTestError {
    #[error("Setup error: {0}")]
    Setup(String),

    #[error("Teardown error: {0}")]
    Teardown(String),

    #[error("CLI execution error: {0}")]
    CliExecution(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
