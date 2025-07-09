//! Integration tests for the `moth run` command.

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

#[test]
fn test_cli_run_command_with_working_mcp_server() {
    // This test uses our working filesystem server specification with proper dependencies
    let mut cmd = Command::cargo_bin("moth").unwrap();
    cmd.arg("run")
        .arg("examples/filesystem-server-mcp-compliant.yaml");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify the command succeeds
    assert!(
        output.status.success(),
        "Command should succeed, exit code: {:?}, stderr: {}, stdout: {}",
        output.status.code(),
        stderr,
        stdout
    );

    // Verify test suite completion
    assert!(
        stdout.contains("Test Suite Finished"),
        "Should contain completion message, got: {}",
        stdout
    );

    // With fixed dependencies, all tests should pass consistently
    assert!(
        stdout.contains("Total Tests: 6"),
        "Should have exactly 6 tests, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Passed: 6"),
        "All 6 tests should pass with fixed dependencies, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Failed: 0"),
        "No tests should fail with fixed dependencies, got: {}",
        stdout
    );
}
