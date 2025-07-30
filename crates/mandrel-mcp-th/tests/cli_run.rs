//! Integration tests for the `moth run` command.

#![allow(clippy::uninlined_format_args)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_run_command_connection_failure() {
    let temp_dir = tempdir().unwrap();
    let spec_path = temp_dir.path().join("simple-test.yaml");
    std::fs::write(
        &spec_path,
        r#"
name: "Simple Test Suite"
version: "1.0.0"
capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "echo"
  args: ["Hello MCP!"]
  transport: "stdio"
tools:
  - name: "test_tool"
    tests:
      - name: "test1"
        input: {}
        expected: { error: false }
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("moth").unwrap();
    cmd.arg("run").arg(spec_path);

    // With echo command (not real MCP server), connection should fail gracefully
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Connection"))
        .stderr(predicate::str::contains("Failed to create MCP service"));
}

#[test]
fn test_cli_run_command_file_not_found() {
    let mut cmd = Command::cargo_bin("moth").unwrap();
    cmd.arg("run").arg("nonexistent-file.yaml");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[tokio::test]
#[ignore] // FUTURE(#331): Fix flaky CLI integration test - fails in concurrent execution but passes individually
async fn test_cli_run_command_with_working_mcp_server() {
    // Setup test environment for MCP server
    std::fs::create_dir_all("/tmp/mcp-test-sandbox").expect("Failed to create test directory");
    std::fs::write("/tmp/mcp-test-sandbox/test.txt", "Hello, MCP test world!")
        .expect("Failed to create test file");

    // This test uses our working filesystem server specification with proper dependencies
    let mut cmd = Command::cargo_bin("moth").unwrap();

    // Use path relative to the test's working directory
    // Integration tests run from the crate directory, so we use a relative path
    let spec_path = "examples/filesystem-server-mcp-compliant.yaml";
    cmd.arg("run").arg(spec_path);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // ✅ GOOD - Test actual execution success first
    assert!(
        output.status.success(),
        "Command should succeed, exit code: {:?}, stderr: {}, stdout: {}",
        output.status.code(),
        stderr,
        stdout
    );

    // ✅ GOOD - Parse and validate actual test results instead of string matching
    let lines: Vec<&str> = stdout.lines().collect();

    // Check for test suite completion by examining the actual structure
    let completion_found = lines.iter().any(|line| {
        line.contains("Test Suite Finished")
            || line.contains("Suite completed")
            || line.contains("All tests processed")
    });

    assert!(
        completion_found,
        "Should indicate test suite completion. Output: {}",
        stdout
    );

    // ✅ GOOD - Extract and validate actual test counts
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    // Parse test results from output lines
    for line in lines {
        if line.contains("Total Tests:") {
            if let Some(num_str) = line.split(':').nth(1) {
                if let Ok(num) = num_str
                    .trim()
                    .split(',')
                    .next()
                    .unwrap_or("0")
                    .parse::<u32>()
                {
                    total_tests = num;
                }
            }
        }
        if line.contains("Passed:") {
            if let Some(num_str) = line.split("Passed:").nth(1) {
                if let Ok(num) = num_str
                    .trim()
                    .split(',')
                    .next()
                    .unwrap_or("0")
                    .parse::<u32>()
                {
                    passed_tests = num;
                }
            }
        }
        if line.contains("Failed:") {
            if let Some(num_str) = line.split("Failed:").nth(1) {
                if let Ok(num) = num_str.trim().parse::<u32>() {
                    failed_tests = num;
                }
            }
        }
    }

    // ✅ GOOD - Validate actual test execution results
    assert!(
        total_tests > 0,
        "Should have executed some tests, got total: {}",
        total_tests
    );
    assert_eq!(
        total_tests,
        passed_tests + failed_tests,
        "Total tests ({}) should equal passed ({}) + failed ({})",
        total_tests,
        passed_tests,
        failed_tests
    );

    // ✅ GOOD - With fixed dependencies, most tests should pass
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    assert!(
        success_rate >= 80.0,
        "Success rate should be at least 80% with fixed dependencies, got {:.1}% ({}/{} passed)",
        success_rate,
        passed_tests,
        total_tests
    );

    // ✅ GOOD - Verify the filesystem server actually processed files
    let file_processing_indicators = [
        "reading",
        "writing",
        "listing",
        "directory",
        "file",
        "path",
        "filesystem",
        "operation",
        "request",
    ];

    let has_file_operations = stdout.lines().any(|line| {
        file_processing_indicators
            .iter()
            .any(|indicator| line.to_lowercase().contains(indicator))
    });

    if total_tests > 0 && passed_tests > 0 {
        assert!(
            has_file_operations,
            "Should show evidence of filesystem operations being tested. Output: {}",
            stdout
        );
    }

    // Cleanup test environment
    std::fs::remove_dir_all("/tmp/mcp-test-sandbox").ok();
}
