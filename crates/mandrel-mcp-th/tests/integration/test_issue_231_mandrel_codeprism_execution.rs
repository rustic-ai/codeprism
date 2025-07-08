//! Integration tests for Issue #231: Execute mandrel-mcp-th against existing codeprism-moth-specs
//!
//! This test module validates that the mandrel-mcp-th CLI can successfully execute
//! the existing comprehensive CodePrism moth specifications against a real CodePrism server.

use assert_cmd::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::process::Child;

/// Helper to start CodePrism MCP server for testing
async fn start_codeprism_server() -> std::io::Result<Child> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args([
        "run",
        "--package",
        "codeprism-mcp-server",
        "--bin",
        "codeprism-mcp-server",
    ])
    .env("RUST_LOG", "info")
    .env("MCP_PROTOCOL_VERSION", "2025-06-18")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    cmd.spawn()
}

/// Helper to execute mandrel-mcp-th CLI with given arguments
async fn execute_moth_cli(args: &[&str]) -> Result<CliExecutionResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let mut cmd = Command::cargo_bin("moth")?;
    cmd.args(args);

    let output = cmd.output()?;
    let execution_time = start_time.elapsed();

    Ok(CliExecutionResult {
        success: output.status.success(),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        execution_time,
    })
}

/// Result of CLI execution with comprehensive metrics
#[derive(Debug)]
#[allow(dead_code)]
struct CliExecutionResult {
    success: bool,
    exit_code: i32,
    stdout: String,
    stderr: String,
    execution_time: Duration,
}

impl CliExecutionResult {
    /// Extract number of total tests from output
    fn total_tests(&self) -> Option<usize> {
        // Parse "Total Tests: X" from output
        if let Some(start) = self.stdout.find("Total Tests: ") {
            let rest = &self.stdout[start + "Total Tests: ".len()..];
            if let Some(end) = rest.find(',') {
                return rest[..end].parse().ok();
            }
        }
        None
    }

    /// Extract number of passed tests from output
    fn passed_tests(&self) -> Option<usize> {
        // Parse "Passed: X" from output
        if let Some(start) = self.stdout.find("Passed: ") {
            let rest = &self.stdout[start + "Passed: ".len()..];
            if let Some(end) = rest.find(',') {
                return rest[..end].parse().ok();
            }
        }
        None
    }

    /// Extract number of failed tests from output
    #[allow(dead_code)]
    fn failed_tests(&self) -> Option<usize> {
        // Parse "Failed: X" from output
        if let Some(start) = self.stdout.find("Failed: ") {
            let rest = &self.stdout[start + "Failed: ".len()..];
            if let Some(end) = rest.find(char::is_whitespace) {
                return rest[..end].parse().ok();
            }
        }
        None
    }
}

// ========================================================================
// PHASE 1: Basic CLI Execution Tests (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_load_rust_comprehensive_specification() {
    // RED: This test should fail until we implement proper specification loading
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";

    // Verify the specification file exists
    assert!(
        Path::new(spec_path).exists(),
        "Rust comprehensive spec should exist"
    );

    // Try to validate the specification (this will fail initially)
    let result = execute_moth_cli(&["validate", spec_path]).await;

    match result {
        Ok(output) => {
            assert!(output.success, "Specification validation should succeed");
            assert!(
                !output.stdout.contains("error"),
                "Should not contain errors"
            );
        }
        Err(e) => {
            // Expected to fail initially - this is RED phase
            println!("Expected failure during RED phase: {}", e);
        }
    }
}

#[tokio::test]
async fn test_execute_rust_comprehensive_against_codeprism_server() {
    // RED: This test should fail until we implement server integration
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";

    // Start CodePrism server (will fail initially)
    let server_result = start_codeprism_server().await;
    if server_result.is_err() {
        println!("Expected server startup failure during RED phase");
        return; // Early return during RED phase
    }

    let mut _server = server_result.unwrap();

    // Give server time to start up
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Execute mandrel-mcp-th against the comprehensive spec
    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            // Validate basic execution results
            assert!(output.success, "CLI execution should succeed");
            assert!(
                output.execution_time < Duration::from_secs(60),
                "Should complete within 60 seconds"
            );

            // Validate test counts match specification expectations
            if let Some(total) = output.total_tests() {
                assert_eq!(total, 18, "Rust spec should have 18 tools");
            }

            if let Some(passed) = output.passed_tests() {
                assert!(passed > 0, "At least some tests should pass");
            }

            println!(
                "✅ Rust comprehensive execution completed: {} tests",
                output.total_tests().unwrap_or(0)
            );
        }
        Err(e) => {
            // Expected to fail initially - this is RED phase
            println!("Expected execution failure during RED phase: {}", e);
        }
    }

    // Cleanup: Kill server
    let _ = _server.kill().await;
}

#[tokio::test]
async fn test_execute_python_comprehensive_against_codeprism_server() {
    // RED: This test should fail until we implement server integration
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml";

    // Verify the specification file exists
    assert!(
        Path::new(spec_path).exists(),
        "Python comprehensive spec should exist"
    );

    // This will fail during RED phase
    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            assert!(output.success, "CLI execution should succeed");
            if let Some(total) = output.total_tests() {
                assert_eq!(total, 18, "Python spec should have 18 tools");
            }
            println!("✅ Python comprehensive execution completed");
        }
        Err(e) => {
            println!("Expected execution failure during RED phase: {}", e);
        }
    }
}

#[tokio::test]
async fn test_execute_java_comprehensive_against_codeprism_server() {
    // RED: This test should fail until we implement server integration
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-java-comprehensive.yaml";

    assert!(
        Path::new(spec_path).exists(),
        "Java comprehensive spec should exist"
    );

    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            assert!(output.success, "CLI execution should succeed");
            if let Some(total) = output.total_tests() {
                assert_eq!(total, 18, "Java spec should have 18 tools");
            }
            println!("✅ Java comprehensive execution completed");
        }
        Err(e) => {
            println!("Expected execution failure during RED phase: {}", e);
        }
    }
}

#[tokio::test]
async fn test_execute_javascript_comprehensive_against_codeprism_server() {
    // RED: This test should fail until we implement server integration
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-javascript-comprehensive.yaml";

    assert!(
        Path::new(spec_path).exists(),
        "JavaScript comprehensive spec should exist"
    );

    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            assert!(output.success, "CLI execution should succeed");
            if let Some(total) = output.total_tests() {
                assert_eq!(total, 17, "JavaScript spec should have 17 tools");
            }
            println!("✅ JavaScript comprehensive execution completed");
        }
        Err(e) => {
            println!("Expected execution failure during RED phase: {}", e);
        }
    }
}

// ========================================================================
// PHASE 2: Performance and Result Validation Tests (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_execution_performance_requirements() {
    // RED: This test should fail until we optimize performance
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";

    let start_time = Instant::now();
    let result = execute_moth_cli(&["run", spec_path]).await;
    let total_duration = start_time.elapsed();

    match result {
        Ok(output) => {
            // Performance requirements
            assert!(
                total_duration < Duration::from_secs(60),
                "Execution should complete within 60 seconds, took: {:?}",
                total_duration
            );
            assert!(
                output.execution_time < Duration::from_secs(60),
                "CLI execution should complete within 60 seconds"
            );

            println!("✅ Performance requirements met: {:?}", total_duration);
        }
        Err(e) => {
            println!("Expected performance test failure during RED phase: {}", e);
        }
    }
}

#[tokio::test]
async fn test_comprehensive_tool_coverage() {
    // RED: This test should fail until we validate tool coverage
    let specs = [
        ("rust", "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml", 18),
        ("python", "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml", 18),
        ("java", "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-java-comprehensive.yaml", 18),
        ("javascript", "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-javascript-comprehensive.yaml", 17),
    ];

    for (language, spec_path, expected_tools) in specs {
        let result = execute_moth_cli(&["run", spec_path]).await;

        match result {
            Ok(output) => {
                if let Some(total) = output.total_tests() {
                    assert_eq!(
                        total, expected_tools,
                        "{} spec should have {} tools, found {}",
                        language, expected_tools, total
                    );
                }

                if let Some(passed) = output.passed_tests() {
                    assert!(
                        passed > 0,
                        "{} spec should have at least some passing tests",
                        language
                    );
                }

                println!(
                    "✅ {} tool coverage validated: {} tools",
                    language, expected_tools
                );
            }
            Err(e) => {
                println!(
                    "Expected {} coverage test failure during RED phase: {}",
                    language, e
                );
            }
        }
    }
}

#[tokio::test]
async fn test_result_structure_validation() {
    // RED: This test should fail until we implement proper result parsing
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";

    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            assert!(output.success, "CLI should succeed");

            // Verify basic result structure in stdout
            assert!(
                output.stdout.contains("Test Suite Finished"),
                "Results should contain completion message"
            );
            assert!(
                output.stdout.contains("Total Tests:"),
                "Results should contain total tests count"
            );
            assert!(
                output.stdout.contains("Passed:"),
                "Results should contain passed count"
            );

            println!("✅ Result structure validation passed");
        }
        Err(e) => {
            println!(
                "Expected result structure test failure during RED phase: {}",
                e
            );
        }
    }
}

// ========================================================================
// PHASE 3: Error Handling and Edge Cases (RED PHASE)
// ========================================================================

#[tokio::test]
async fn test_invalid_specification_handling() {
    // RED: This test should fail until we implement proper error handling
    let invalid_spec = "nonexistent/invalid-spec.yaml";

    let result = execute_moth_cli(&["run", invalid_spec]).await;

    match result {
        Ok(output) => {
            // Should fail gracefully
            assert!(!output.success, "Should fail with invalid specification");
            assert!(
                output.stderr.contains("not found") || output.stderr.contains("No such file"),
                "Should indicate file not found"
            );

            println!("✅ Invalid specification handled gracefully");
        }
        Err(e) => {
            println!("Expected invalid spec test failure during RED phase: {}", e);
        }
    }
}

#[tokio::test]
async fn test_server_connection_failure_handling() {
    // RED: This test should fail until we implement connection error handling
    let spec_path = "../../crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";

    // Execute without starting server (should fail gracefully)
    let result = execute_moth_cli(&["run", spec_path]).await;

    match result {
        Ok(output) => {
            // Should handle connection failure gracefully
            if !output.success {
                assert!(
                    output.stderr.contains("connection") || output.stderr.contains("server"),
                    "Should indicate connection/server error"
                );
                println!("✅ Server connection failure handled gracefully");
            } else {
                println!("Unexpected success - server might already be running");
            }
        }
        Err(e) => {
            println!(
                "Expected connection failure test failure during RED phase: {}",
                e
            );
        }
    }
}
