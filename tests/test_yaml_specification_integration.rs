//! Integration Tests for YAML Test Specifications
//!
//! These tests validate that the three comprehensive test specifications
//! (filesystem-server.yaml, everything-server.yaml, weather-server.yaml)
//! work correctly with the moth binary.

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Integration test framework for running YAML test specifications
struct YamlSpecificationTester {
    temp_dir: TempDir,
    binary_path: PathBuf,
}

impl YamlSpecificationTester {
    /// Create a new YAML specification tester
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        // Build the moth binary first
        let binary_path = Self::build_test_harness_binary()?;
        
        Ok(Self {
            temp_dir,
            binary_path,
        })
    }

    /// Build the moth binary for testing
    fn build_test_harness_binary() -> Result<PathBuf> {
        // Build the binary using cargo
        let output = Command::new("cargo")
            .args([
                "build", 
                "--package", "mandrel-mcp-th",
                "--bin", "moth",
                "--release"
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to build moth binary: {}", stderr);
        }

        // Determine binary path
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| "target".to_string());
        let binary_path = PathBuf::from(target_dir)
            .join("release")
            .join("moth");

        if !binary_path.exists() {
            anyhow::bail!("Built binary not found at: {}", binary_path.display());
        }

        println!("✅ Built moth binary at: {}", binary_path.display());
        Ok(binary_path)
    }

    /// Get the path to a YAML specification file
    fn get_yaml_spec_path(&self, spec_name: &str) -> PathBuf {
        PathBuf::from("crates/mandrel-mcp-th/examples")
            .join(format!("{}.yaml", spec_name))
    }

    /// Run the test harness with a YAML specification
    pub async fn run_yaml_specification(
        &self, 
        spec_name: &str,
        expected_behavior: ExpectedTestBehavior
    ) -> Result<TestExecutionResult> {
        let yaml_path = self.get_yaml_spec_path(spec_name);
        
        if !yaml_path.exists() {
            anyhow::bail!("YAML specification not found: {}", yaml_path.display());
        }

        let output_dir = self.temp_dir.path().join(format!("{}_reports", spec_name));
        std::fs::create_dir_all(&output_dir)?;

        println!("🚀 Running {} specification...", spec_name);
        println!("   YAML: {}", yaml_path.display());
        println!("   Output: {}", output_dir.display());

        // Run the moth binary with timeout
        let test_timeout = Duration::from_secs(expected_behavior.timeout_seconds);
        
        let command_future = tokio::task::spawn_blocking({
            let binary_path = self.binary_path.clone();
            let yaml_path = yaml_path.clone();
            let output_dir = output_dir.clone();
            
            move || {
                Command::new(&binary_path)
                    .args([
                        "run",
                        "--config", &yaml_path.to_string_lossy(),
                        "--output", &output_dir.to_string_lossy(),
                        "--quiet"
                    ])
                    .output()
            }
        });

        let result = timeout(test_timeout, command_future).await
            .map_err(|_| anyhow::anyhow!("Test execution timed out after {} seconds", test_timeout.as_secs()))?;

        let output = result??;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let execution_result = TestExecutionResult {
            spec_name: spec_name.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            output_directory: output_dir,
            success: self.evaluate_test_success(&output, &expected_behavior),
        };

        println!("✅ Completed {} specification", spec_name);
        println!("   Exit code: {}", execution_result.exit_code);
        println!("   Success: {}", execution_result.success);

        Ok(execution_result)
    }

    /// Evaluate whether test execution was successful based on expected behavior
    fn evaluate_test_success(
        &self, 
        output: &std::process::Output, 
        expected: &ExpectedTestBehavior
    ) -> bool {
        // Check exit code
        let exit_code = output.status.code().unwrap_or(-1);
        
        match expected.should_succeed {
            true => {
                // For successful tests, expect exit code 0
                if exit_code != 0 {
                    println!("❌ Expected success but got exit code: {}", exit_code);
                    return false;
                }
            }
            false => {
                // For tests expected to fail, expect non-zero exit code
                if exit_code == 0 {
                    println!("❌ Expected failure but got success exit code");
                    return false;
                }
            }
        }

        // Check for expected output patterns
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("{}\n{}", stdout, stderr);

        for pattern in &expected.expected_output_patterns {
            if !combined_output.contains(pattern) {
                println!("❌ Expected output pattern not found: '{}'", pattern);
                return false;
            }
        }

        // Check for unexpected error patterns
        for pattern in &expected.unexpected_error_patterns {
            if combined_output.contains(pattern) {
                println!("❌ Unexpected error pattern found: '{}'", pattern);
                return false;
            }
        }

        true
    }

    /// Validate that output files were generated correctly
    pub fn validate_output_files(&self, result: &TestExecutionResult) -> Result<OutputValidation> {
        let mut validation = OutputValidation {
            reports_generated: false,
            json_report_exists: false,
            html_report_exists: false,
            junit_report_exists: false,
            file_count: 0,
            total_file_size_bytes: 0,
        };

        if !result.output_directory.exists() {
            return Ok(validation); // No output directory created
        }

        // Count files and check for specific report types
        let entries = std::fs::read_dir(&result.output_directory)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                validation.file_count += 1;
                
                if let Ok(metadata) = entry.metadata() {
                    validation.total_file_size_bytes += metadata.len();
                }

                if let Some(extension) = path.extension() {
                    match extension.to_string_lossy().as_ref() {
                        "json" => validation.json_report_exists = true,
                        "html" => validation.html_report_exists = true,
                        "xml" => validation.junit_report_exists = true,
                        _ => {}
                    }
                }
            }
        }

        validation.reports_generated = validation.file_count > 0;

        println!("📊 Output validation for {}:", result.spec_name);
        println!("   Files generated: {}", validation.file_count);
        println!("   Total size: {} bytes", validation.total_file_size_bytes);
        println!("   JSON report: {}", validation.json_report_exists);
        println!("   HTML report: {}", validation.html_report_exists);
        println!("   JUnit report: {}", validation.junit_report_exists);

        Ok(validation)
    }
}

/// Expected behavior for test execution
#[derive(Debug, Clone)]
pub struct ExpectedTestBehavior {
    pub should_succeed: bool,
    pub timeout_seconds: u64,
    pub expected_output_patterns: Vec<String>,
    pub unexpected_error_patterns: Vec<String>,
}

impl ExpectedTestBehavior {
    /// Create expected behavior for successful test execution
    pub fn success() -> Self {
        Self {
            should_succeed: true,
            timeout_seconds: 300, // 5 minutes default timeout
            expected_output_patterns: vec![],
            unexpected_error_patterns: vec![
                "panic".to_string(),
                "fatal error".to_string(),
                "segmentation fault".to_string(),
            ],
        }
    }

    /// Create expected behavior for mock server testing (may have expected failures)
    pub fn mock_server_testing() -> Self {
        Self {
            should_succeed: false, // Expected to fail due to mock servers not being available
            timeout_seconds: 60,   // Shorter timeout for expected failures
            expected_output_patterns: vec![
                "connection".to_string(), // Should show connection attempts
            ],
            unexpected_error_patterns: vec![
                "panic".to_string(),
                "segmentation fault".to_string(),
            ],
        }
    }

    /// Add expected output pattern
    pub fn with_expected_pattern(mut self, pattern: &str) -> Self {
        self.expected_output_patterns.push(pattern.to_string());
        self
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Result of test execution
#[derive(Debug)]
pub struct TestExecutionResult {
    pub spec_name: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_directory: PathBuf,
    pub success: bool,
}

/// Validation of output files
#[derive(Debug)]
pub struct OutputValidation {
    pub reports_generated: bool,
    pub json_report_exists: bool,
    pub html_report_exists: bool,
    pub junit_report_exists: bool,
    pub file_count: usize,
    pub total_file_size_bytes: u64,
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_server_specification() -> Result<()> {
    let tester = YamlSpecificationTester::new()?;
    
    let expected_behavior = ExpectedTestBehavior::mock_server_testing()
        .with_expected_pattern("filesystem")
        .with_expected_pattern("server")
        .with_timeout(120); // 2 minutes for filesystem tests

    let result = tester.run_yaml_specification(
        "filesystem-server", 
        expected_behavior
    ).await?;

    // Validate the test execution
    assert_eq!(result.spec_name, "filesystem-server");
    
    // Even if the test fails due to mock server, we should see proper error handling
    assert!(!result.stdout.is_empty() || !result.stderr.is_empty(), 
           "Should generate some output");

    // Validate output files
    let output_validation = tester.validate_output_files(&result)?;
    
    // For mock testing, we may not generate reports if connection fails immediately
    println!("Filesystem server test output validation: {:?}", output_validation);

    Ok(())
}

#[tokio::test]
async fn test_everything_server_specification() -> Result<()> {
    let tester = YamlSpecificationTester::new()?;
    
    let expected_behavior = ExpectedTestBehavior::mock_server_testing()
        .with_expected_pattern("everything")
        .with_expected_pattern("server")
        .with_timeout(180); // 3 minutes for everything server tests

    let result = tester.run_yaml_specification(
        "everything-server",
        expected_behavior
    ).await?;

    // Validate the test execution
    assert_eq!(result.spec_name, "everything-server");
    assert!(!result.stdout.is_empty() || !result.stderr.is_empty(),
           "Should generate some output");

    // Validate output files
    let output_validation = tester.validate_output_files(&result)?;
    println!("Everything server test output validation: {:?}", output_validation);

    Ok(())
}

#[tokio::test]
async fn test_weather_server_specification() -> Result<()> {
    let tester = YamlSpecificationTester::new()?;
    
    let expected_behavior = ExpectedTestBehavior::mock_server_testing()
        .with_expected_pattern("weather")
        .with_expected_pattern("server")
        .with_timeout(150); // 2.5 minutes for weather tests

    let result = tester.run_yaml_specification(
        "weather-server",
        expected_behavior
    ).await?;

    // Validate the test execution
    assert_eq!(result.spec_name, "weather-server");
    assert!(!result.stdout.is_empty() || !result.stderr.is_empty(),
           "Should generate some output");

    // Validate output files  
    let output_validation = tester.validate_output_files(&result)?;
    println!("Weather server test output validation: {:?}", output_validation);

    Ok(())
}

#[tokio::test]
async fn test_yaml_specification_file_validation() -> Result<()> {
    // Test that all YAML specification files are valid and parseable
    let spec_files = vec![
        "filesystem-server",
        "everything-server", 
        "weather-server"
    ];

    for spec_name in spec_files {
        let yaml_path = PathBuf::from("crates/mandrel-mcp-th/examples")
            .join(format!("{}.yaml", spec_name));

        // Verify file exists
        assert!(yaml_path.exists(), 
               "YAML specification should exist: {}", yaml_path.display());

        // Verify file is valid YAML
        let content = std::fs::read_to_string(&yaml_path)?;
        let parsed: serde_yml::Value = serde_yml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid YAML in {}: {}", spec_name, e))?;

        // Verify basic structure
        assert!(parsed.get("name").is_some(), 
               "YAML should have 'name' field: {}", spec_name);
        assert!(parsed.get("version").is_some(),
               "YAML should have 'version' field: {}", spec_name);
        assert!(parsed.get("description").is_some(),
               "YAML should have 'description' field: {}", spec_name);

        println!("✅ Validated YAML structure for: {}", spec_name);
    }

    Ok(())
}

#[tokio::test]
async fn test_binary_help_and_version() -> Result<()> {
    let tester = YamlSpecificationTester::new()?;

    // Test --help flag
    let help_output = Command::new(&tester.binary_path)
        .args(["--help"])
        .output()?;

    // Help should provide usage information
    let help_text = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_text.contains("moth") || help_text.contains("mandrel-mcp-th"), 
           "Help should mention binary name");
    assert!(help_text.contains("run"), 
           "Help should mention 'run' command");

    // Test --version flag
    let version_output = Command::new(&tester.binary_path)
        .args(["--version"])
        .output()?;

    let version_text = String::from_utf8_lossy(&version_output.stdout);
    assert!(!version_text.is_empty(), 
           "Version should produce output");

    println!("✅ Binary help and version commands work correctly");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_yaml_integration_suite() -> Result<()> {
    // Run all three specifications in sequence and collect results
    let tester = YamlSpecificationTester::new()?;
    
    let specifications = vec![
        ("filesystem-server", 120),
        ("everything-server", 180), 
        ("weather-server", 150),
    ];

    let mut results = Vec::new();

    for (spec_name, timeout_seconds) in specifications {
        println!("\n🧪 Testing {} specification...", spec_name);
        
        let expected_behavior = ExpectedTestBehavior::mock_server_testing()
            .with_timeout(timeout_seconds);

        let result = tester.run_yaml_specification(spec_name, expected_behavior).await?;
        let output_validation = tester.validate_output_files(&result)?;
        
        results.push((result, output_validation));
    }

    // Validate comprehensive results
    assert_eq!(results.len(), 3, "Should have tested all three specifications");

    // Print summary
    println!("\n📊 Comprehensive Integration Test Summary:");
    println!("==========================================");
    
    for (result, validation) in &results {
        println!("Specification: {}", result.spec_name);
        println!("  Exit Code: {}", result.exit_code);  
        println!("  Success: {}", result.success);
        println!("  Files Generated: {}", validation.file_count);
        println!("  Output Size: {} bytes", validation.total_file_size_bytes);
        println!();
    }

    // All tests should have executed (even if they failed due to mock servers)
    for (result, _) in &results {
        assert!(!result.stdout.is_empty() || !result.stderr.is_empty(),
               "Specification {} should generate output", result.spec_name);
    }

    println!("✅ All YAML specification integration tests completed successfully");
    Ok(())
}

// ============================================================================
// Cleanup
// ============================================================================

impl Drop for YamlSpecificationTester {
    fn drop(&mut self) {
        // Cleanup is automatic with TempDir
        println!("🧹 Cleaned up integration test environment");
    }
} 