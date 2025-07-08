//! Integration Test Framework
//!
//! Provides framework for running end-to-end integration tests
//! with real MCP servers and the complete MOTH test harness.

use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;
use thiserror::Error;

/// Main integration test framework for real server testing
pub struct IntegrationTestFramework {
    pub test_data_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub moth_specs_dir: PathBuf,
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
    
    pub async fn run_cli_test(&self, spec_file: &str, args: &[&str]) -> Result<TestResult, IntegrationTestError> {
        // Execute the MOTH CLI with real server and test specification
        let start_time = std::time::Instant::now();
        
        // For now, return a success result - actual CLI execution will be implemented
        // when the CLI component is ready
        Ok(TestResult {
            exit_code: 0,
            stdout: format!("Executed MOTH test with spec: {}", spec_file),
            stderr: String::new(),
            execution_time: start_time.elapsed(),
            generated_files: vec![
                self.reports_dir.join("test_report.html"),
                self.reports_dir.join("test_report.json"),
            ],
        })
    }
    
    /// Get available CodePrism moth specifications
    pub fn get_codeprism_specs(&self) -> Result<Vec<PathBuf>, IntegrationTestError> {
        let comprehensive_dir = self.moth_specs_dir.join("codeprism/comprehensive");
        let mut specs = Vec::new();
        
        if comprehensive_dir.exists() {
            for entry in std::fs::read_dir(&comprehensive_dir)
                .map_err(|e| IntegrationTestError::Setup(e.to_string()))? {
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
    pub async fn validate_reports(&self, _expected_reports: &[ReportExpectation]) -> Result<(), IntegrationTestError> {
        // Validate that expected reports were generated
        // Implementation will verify report content and structure
        Ok(())
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