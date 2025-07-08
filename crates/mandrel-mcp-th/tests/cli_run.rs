//! Integration tests for the `moth run` command.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_run_command_success() {
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

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test Suite Finished"))
        .stdout(predicate::str::contains("Passed: 1"));
}

#[test]
fn test_cli_run_command_failure() {
    let temp_dir = tempdir().unwrap();
    let spec_path = temp_dir.path().join("failing-test.yaml");
    std::fs::write(
        &spec_path,
        r#"
name: "Failing Test Suite"
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
  - name: "failing_tool"
    tests:
      - name: "failing_test"
        input: { "value": "fail" }
        expected: { error: true }
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("moth").unwrap();
    cmd.arg("run").arg(spec_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Test Suite Finished"))
        .stdout(predicate::str::contains("Failed: 1"));
}
