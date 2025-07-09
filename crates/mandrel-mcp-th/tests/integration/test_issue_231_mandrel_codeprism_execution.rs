//! Integration tests for Issue #231: Mandrel CodePrism execution
//!
//! Tests the execution of comprehensive CodePrism specifications against the actual
//! CodePrism MCP server to ensure correct tool compatibility and response handling.

use std::path::Path;
use std::process::Command;
use tokio;

/// Test executing Rust comprehensive specification against CodePrism server
#[tokio::test]
async fn test_execute_rust_comprehensive_against_codeprism_server() {
    let spec_path = Path::new(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml",
    );

    if !spec_path.exists() {
        eprintln!("Skipping test - spec file not found: {:?}", spec_path);
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg(spec_path)
        .arg("--output")
        .arg("./target/test-reports/rust-comprehensive");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    // The test should pass (exit code 0) and include all expected tool executions
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Rust comprehensive test failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    // Parse output to verify execution
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify the test contains the expected number of tools (12 working tools)
    // After removing non-existent tools: find_duplicates, trace_inheritance, analyze_decorators, etc.
    let tool_count = stdout.matches("tool called").count();
    assert_eq!(tool_count, 12, "Rust spec should have 12 tools");

    // Verify specific tool calls were made
    assert!(
        stdout.contains("Get repository info tool called"),
        "Should call get_repository_info"
    );
    assert!(
        stdout.contains("Trace path tool called"),
        "Should call trace_path"
    );
    assert!(
        stdout.contains("Explain symbol tool called"),
        "Should call explain_symbol"
    );
    assert!(
        stdout.contains("Search symbols tool called"),
        "Should call search_symbols"
    );
    assert!(
        stdout.contains("Search content tool called"),
        "Should call search_content"
    );
    assert!(
        stdout.contains("Analyze complexity tool called"),
        "Should call analyze_complexity"
    );
    assert!(
        stdout.contains("Find dependencies tool called"),
        "Should call find_dependencies"
    );
    assert!(
        stdout.contains("Find references tool called"),
        "Should call find_references"
    );
    assert!(
        stdout.contains("Provide guidance tool called"),
        "Should call provide_guidance"
    );
    assert!(
        stdout.contains("Optimize code tool called"),
        "Should call optimize_code"
    );
    assert!(
        stdout.contains("Batch process tool called"),
        "Should call batch_process"
    );
    assert!(
        stdout.contains("Workflow automation tool called"),
        "Should call workflow_automation"
    );

    // Verify successful execution
    assert!(
        stdout.contains("✅ Test Suite Finished ✅"),
        "Should complete successfully"
    );
    assert!(
        stdout.contains("Total Tests: 12, Passed: 12, Failed: 0"),
        "All tests should pass"
    );
}

/// Test executing Python comprehensive specification against CodePrism server
#[tokio::test]
async fn test_execute_python_comprehensive_against_codeprism_server() {
    let spec_path = Path::new(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml",
    );

    if !spec_path.exists() {
        eprintln!("Skipping test - spec file not found: {:?}", spec_path);
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg(spec_path)
        .arg("--output")
        .arg("./target/test-reports/python-comprehensive");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Python comprehensive test failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify the test contains the expected number of tools (12 working tools)
    let tool_count = stdout.matches("tool called").count();
    assert_eq!(tool_count, 12, "Python spec should have 12 tools");

    // Verify successful execution
    assert!(
        stdout.contains("✅ Test Suite Finished ✅"),
        "Should complete successfully"
    );
    assert!(
        stdout.contains("Total Tests: 12, Passed: 12, Failed: 0"),
        "All tests should pass"
    );
}

/// Test executing Java comprehensive specification against CodePrism server
#[tokio::test]
async fn test_execute_java_comprehensive_against_codeprism_server() {
    let spec_path = Path::new(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-java-comprehensive.yaml",
    );

    if !spec_path.exists() {
        eprintln!("Skipping test - spec file not found: {:?}", spec_path);
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg(spec_path)
        .arg("--output")
        .arg("./target/test-reports/java-comprehensive");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Java comprehensive test failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify the test contains the expected number of tools (12 working tools)
    let tool_count = stdout.matches("tool called").count();
    assert_eq!(tool_count, 12, "Java spec should have 12 tools");

    // Verify successful execution
    assert!(
        stdout.contains("✅ Test Suite Finished ✅"),
        "Should complete successfully"
    );
    assert!(
        stdout.contains("Total Tests: 12, Passed: 12, Failed: 0"),
        "All tests should pass"
    );
}

/// Test executing JavaScript comprehensive specification against CodePrism server
#[tokio::test]
async fn test_execute_javascript_comprehensive_against_codeprism_server() {
    let spec_path = Path::new("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-javascript-comprehensive.yaml");

    if !spec_path.exists() {
        eprintln!("Skipping test - spec file not found: {:?}", spec_path);
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg(spec_path)
        .arg("--output")
        .arg("./target/test-reports/javascript-comprehensive");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "JavaScript comprehensive test failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify the test contains the expected number of tools (12 working tools)
    let tool_count = stdout.matches("tool called").count();
    assert_eq!(tool_count, 12, "JavaScript spec should have 12 tools");

    // Verify successful execution
    assert!(
        stdout.contains("✅ Test Suite Finished ✅"),
        "Should complete successfully"
    );
    assert!(
        stdout.contains("Total Tests: 12, Passed: 12, Failed: 0"),
        "All tests should pass"
    );
}

/// Test comprehensive tool coverage across all language specifications
#[tokio::test]
async fn test_comprehensive_tool_coverage() {
    let specs = [
        ("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml", 12),
        ("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml", 12),
        ("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-java-comprehensive.yaml", 12),
        ("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-javascript-comprehensive.yaml", 12),
    ];

    for (spec_path, expected_tools) in specs.iter() {
        let path = Path::new(spec_path);
        if !path.exists() {
            eprintln!("Skipping spec - file not found: {:?}", path);
            continue;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "moth"])
            .arg("run")
            .arg(path)
            .arg("--output")
            .arg("./target/test-reports/comprehensive-coverage");

        let output = cmd
            .output()
            .expect("Failed to execute mandrel test command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            panic!(
                "Comprehensive coverage test failed for {:?}!\nSTDOUT:\n{}\nSTDERR:\n{}",
                path, stdout, stderr
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let tool_count = stdout.matches("tool called").count();

        let spec_name = path.file_stem().unwrap().to_str().unwrap();
        assert_eq!(
            tool_count, *expected_tools,
            "{} spec should have {} tools, found {}",
            spec_name, expected_tools, tool_count
        );
    }
}

/// Test basic configuration validation for comprehensive specifications
#[tokio::test]
async fn test_comprehensive_spec_validation() {
    let specs = [
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml",
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml",
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-java-comprehensive.yaml",
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-javascript-comprehensive.yaml",
    ];

    for spec_path in specs.iter() {
        let path = Path::new(spec_path);
        if !path.exists() {
            eprintln!("Skipping validation - spec file not found: {:?}", path);
            continue;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "moth"]).arg("validate").arg(path);

        let output = cmd.output().expect("Failed to execute validation command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            panic!(
                "Comprehensive spec validation failed for {:?}!\nSTDOUT:\n{}\nSTDERR:\n{}",
                path, stdout, stderr
            );
        }
    }
}

/// Test server startup and basic connectivity
#[tokio::test]
async fn test_codeprism_server_connectivity() {
    // This test ensures the CodePrism MCP server can start and respond to basic requests
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--package",
        "codeprism-mcp-server",
        "--bin",
        "codeprism-mcp-server",
    ])
    .arg("--help");

    let output = cmd.output().expect("Failed to execute server help check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Server help check failed: {}", stderr);
    }
}

/// Test error handling for non-existent specification files
#[tokio::test]
async fn test_nonexistent_spec_handling() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg("nonexistent-spec.yaml")
        .arg("--output")
        .arg("./target/test-reports/nonexistent");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    // Should fail gracefully with non-zero exit code
    assert!(!output.status.success(), "Should fail for nonexistent spec");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("File not found") || stderr.contains("No such file"),
        "Should provide clear error message for missing file"
    );
}

/// Test mandrel test harness performance with comprehensive specifications  
#[tokio::test]
async fn test_comprehensive_spec_performance() {
    let spec_path = Path::new(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml",
    );

    if !spec_path.exists() {
        eprintln!(
            "Skipping performance test - spec file not found: {:?}",
            spec_path
        );
        return;
    }

    let start_time = std::time::Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "moth"])
        .arg("run")
        .arg(spec_path)
        .arg("--output")
        .arg("./target/test-reports/performance");

    let output = cmd
        .output()
        .expect("Failed to execute mandrel test command");

    let duration = start_time.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Performance test failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    // Comprehensive specs should complete within reasonable time (< 30 seconds)
    assert!(
        duration.as_secs() < 30,
        "Comprehensive spec execution took too long: {:?}",
        duration
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✅ Test Suite Finished ✅"),
        "Should complete successfully"
    );
}
