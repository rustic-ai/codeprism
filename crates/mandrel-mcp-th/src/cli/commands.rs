//! Command implementations for MOTH CLI

use crate::client::{ServerConfig as ClientServerConfig, Transport};
use crate::error::{Error, Result};
use crate::executor::{ExpectedResult, TestCase, TestConfig, TestRunner, TestSuite, TestType};
use crate::spec::SpecificationLoader;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info};

/// Handle the test command
pub async fn handle_test(
    spec: PathBuf,
    output_file: Option<PathBuf>,
    fail_fast: bool,
    filter: Option<String>,
    concurrency: usize,
) -> Result<()> {
    info!("Running tests from specification: {}", spec.display());
    info!("Concurrency: {}, Fail fast: {}", concurrency, fail_fast);

    if let Some(filter) = &filter {
        info!("Test filter: {}", filter);
    }

    if let Some(output_file) = &output_file {
        info!("Output file: {}", output_file.display());
    }

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // Load test specification using existing SpecificationLoader
    info!("Loading test specification...");
    let loader = SpecificationLoader::new()?;
    let test_spec = loader.load_from_file(&spec).await?;
    info!("Successfully loaded specification: {}", test_spec.name);

    // Convert specification to TestSuite for execution
    let test_suite = convert_specification_to_suite(test_spec)?;
    info!(
        "Converted specification to test suite with {} tests",
        test_suite.tests.len()
    );

    // Create TestConfig from CLI parameters
    let config = TestConfig {
        max_concurrency: concurrency,
        fail_fast,
        filter,
        test_timeout: Duration::from_secs(30),
        server_timeout: Duration::from_secs(10),
        retry_attempts: 2,
    };

    // Execute tests using existing TestRunner
    info!("Starting test execution...");
    let mut runner = TestRunner::new(config);
    let results = runner.execute_suite(test_suite).await?;

    // Output results
    output_test_results(&results, output_file).await?;

    // Summary
    info!(
        "Test execution complete: {}/{} passed ({:.1}% success rate)",
        results.passed,
        results.total_tests,
        results.success_rate()
    );

    if results.failed > 0 {
        error!("{} tests failed", results.failed);
        return Err(Error::execution("Some tests failed".to_string()));
    }

    Ok(())
}

/// Handle the validate command
pub async fn handle_validate(spec: PathBuf) -> Result<()> {
    info!("Validating specification: {}", spec.display());

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // Load and validate specification using existing SpecificationLoader
    let loader = SpecificationLoader::new()?;
    match loader.load_from_file(&spec).await {
        Ok(test_spec) => {
            info!("✅ Specification validation successful");
            info!("  Name: {}", test_spec.name);
            info!("  Version: {}", test_spec.version);
            if let Some(desc) = &test_spec.description {
                info!("  Description: {}", desc);
            }

            // Count test components
            let tool_count = test_spec.tools.as_ref().map(|t| t.len()).unwrap_or(0);
            let resource_count = test_spec.resources.as_ref().map(|r| r.len()).unwrap_or(0);
            let prompt_count = test_spec.prompts.as_ref().map(|p| p.len()).unwrap_or(0);

            info!(
                "  Tools: {}, Resources: {}, Prompts: {}",
                tool_count, resource_count, prompt_count
            );
            info!(
                "  Server: {} {}",
                test_spec.server.command,
                test_spec.server.args.join(" ")
            );

            Ok(())
        }
        Err(e) => {
            error!("❌ Specification validation failed: {}", e);
            Err(e)
        }
    }
}

/// Handle the list command
pub async fn handle_list(spec: PathBuf, detailed: bool) -> Result<()> {
    info!("Listing tests from specification: {}", spec.display());
    info!("Detailed output: {}", detailed);

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // Load specification using existing SpecificationLoader
    let loader = SpecificationLoader::new()?;
    let test_spec = loader.load_from_file(&spec).await?;

    // Display specification information
    println!("📋 Test Specification: {}", test_spec.name);
    println!("   Version: {}", test_spec.version);
    if let Some(desc) = &test_spec.description {
        println!("   Description: {}", desc);
    }
    println!();

    // List tools
    if let Some(tools) = &test_spec.tools {
        println!("🔧 Tools ({}):", tools.len());
        for tool in tools {
            if detailed {
                println!(
                    "  • {} - {}",
                    tool.name,
                    tool.description
                        .as_ref()
                        .unwrap_or(&"No description".to_string())
                );
                for test in &tool.tests {
                    println!("    ├─ {}", test.name);
                    if let Some(desc) = &test.description {
                        println!("    │  {}", desc);
                    }
                }
            } else {
                let test_count = tool.tests.len();
                println!("  • {} ({} tests)", tool.name, test_count);
            }
        }
        println!();
    }

    // List resources
    if let Some(resources) = &test_spec.resources {
        println!("📁 Resources ({}):", resources.len());
        for resource in resources {
            if detailed {
                println!("  • {} - {}", resource.uri_template, resource.name);
                for test in &resource.tests {
                    println!("    ├─ {}", test.name);
                }
            } else {
                let test_count = resource.tests.len();
                println!("  • {} ({} tests)", resource.uri_template, test_count);
            }
        }
        println!();
    }

    // List prompts
    if let Some(prompts) = &test_spec.prompts {
        println!("💬 Prompts ({}):", prompts.len());
        for prompt in prompts {
            if detailed {
                println!(
                    "  • {} - {}",
                    prompt.name,
                    prompt
                        .description
                        .as_ref()
                        .unwrap_or(&"No description".to_string())
                );
                for test in &prompt.tests {
                    println!("    ├─ {}", test.name);
                }
            } else {
                let test_count = prompt.tests.len();
                println!("  • {} ({} tests)", prompt.name, test_count);
            }
        }
        println!();
    }

    // Summary
    let total_tests = count_total_tests(&test_spec);
    println!("📊 Total Tests: {}", total_tests);

    Ok(())
}

/// Handle the version command
pub fn handle_version() -> Result<()> {
    println!(
        "moth {} - Mandrel MCP Test Harness",
        env!("CARGO_PKG_VERSION")
    );
    println!("MOdel context protocol Test Harness binary");
    println!("Built with official rmcp SDK");
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    Ok(())
}

/// Convert TestSpecification to TestSuite for execution
fn convert_specification_to_suite(spec: crate::spec::TestSpecification) -> Result<TestSuite> {
    let mut tests = Vec::new();

    // Convert tool tests
    if let Some(tools) = &spec.tools {
        for tool in tools {
            for test in &tool.tests {
                let test_case = TestCase {
                    name: format!("{}::{}", tool.name, test.name),
                    description: test.description.clone(),
                    test_type: TestType::ToolCall {
                        tool_name: tool.name.clone(),
                    },
                    parameters: test.input.clone(),
                    expected: convert_expected_output(&test.expected)?,
                    timeout: test
                        .performance
                        .as_ref()
                        .and_then(|p| p.max_duration_ms.map(|ms| Duration::from_millis(ms as u64))),
                    retry_attempts: None,
                };
                tests.push(test_case);
            }
        }
    }

    // Convert resource tests
    if let Some(resources) = &spec.resources {
        for resource in resources {
            for test in &resource.tests {
                let test_case = TestCase {
                    name: format!("{}::{}", resource.name, test.name),
                    description: test.description.clone(),
                    test_type: TestType::ResourceRead {
                        resource_uri: resource.uri_template.clone(),
                    },
                    parameters: test.input.clone(),
                    expected: convert_expected_output(&test.expected)?,
                    timeout: test
                        .performance
                        .as_ref()
                        .and_then(|p| p.max_duration_ms.map(|ms| Duration::from_millis(ms as u64))),
                    retry_attempts: None,
                };
                tests.push(test_case);
            }
        }
    }

    // Add capability check test
    if spec.capabilities.tools || spec.capabilities.resources || spec.capabilities.prompts {
        let capability_test = TestCase {
            name: "capabilities::check".to_string(),
            description: Some("Verify server capabilities match specification".to_string()),
            test_type: TestType::CapabilityCheck,
            parameters: serde_json::Value::Null,
            expected: ExpectedResult {
                should_succeed: true,
                content_patterns: vec![],
                performance: None,
            },
            timeout: Some(Duration::from_secs(5)),
            retry_attempts: None,
        };
        tests.push(capability_test);
    }

    Ok(TestSuite {
        name: spec.name,
        version: spec.version,
        description: spec.description,
        server: convert_server_config(spec.server)?,
        tests,
        config: TestConfig::default(),
    })
}

/// Convert spec::ServerConfig to client::ServerConfig
fn convert_server_config(spec_config: crate::spec::ServerConfig) -> Result<ClientServerConfig> {
    let transport = match spec_config.transport.as_str() {
        "stdio" => Transport::Stdio,
        "http" => return Err(Error::spec("HTTP transport not yet supported".to_string())),
        "sse" => return Err(Error::spec("SSE transport not yet supported".to_string())),
        _ => {
            return Err(Error::spec(format!(
                "Unknown transport: {}",
                spec_config.transport
            )))
        }
    };

    Ok(ClientServerConfig {
        command: spec_config.command,
        args: spec_config.args,
        env: spec_config.env,
        working_dir: spec_config.working_dir.map(PathBuf::from),
        transport,
        startup_timeout: Duration::from_secs(spec_config.startup_timeout_seconds as u64),
        shutdown_timeout: Duration::from_secs(spec_config.shutdown_timeout_seconds as u64),
        operation_timeout: Duration::from_secs(30),
        max_retries: 3,
    })
}

/// Convert ExpectedOutput to ExpectedResult
fn convert_expected_output(expected: &crate::spec::ExpectedOutput) -> Result<ExpectedResult> {
    // Extract content patterns from field validations
    let mut content_patterns = Vec::new();
    for field in &expected.fields {
        if let Some(value) = &field.value {
            content_patterns.push(format!("\"{}\"", value));
        }
        if let Some(pattern) = &field.pattern {
            content_patterns.push(pattern.clone());
        }
    }

    // Performance expectations from schema/error checks
    let performance = None; // Will be set from test.performance if available

    Ok(ExpectedResult {
        should_succeed: !expected.error,
        content_patterns,
        performance,
    })
}

/// Output test results in the specified format
async fn output_test_results(
    results: &crate::executor::SuiteResult,
    output_file: Option<PathBuf>,
) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;

    match output_file {
        Some(file) => {
            tokio::fs::write(&file, &json_output).await?;
            info!("Test results written to: {}", file.display());
        }
        None => {
            println!("{}", json_output);
        }
    }

    Ok(())
}

/// Count total tests in a specification
fn count_total_tests(spec: &crate::spec::TestSpecification) -> usize {
    let mut count = 0;

    if let Some(tools) = &spec.tools {
        for tool in tools {
            count += tool.tests.len();
        }
    }

    if let Some(resources) = &spec.resources {
        for resource in resources {
            count += resource.tests.len();
        }
    }

    if let Some(prompts) = &spec.prompts {
        for prompt in prompts {
            count += prompt.tests.len();
        }
    }

    // Add 1 for capability check if any capabilities are enabled
    if spec.capabilities.tools || spec.capabilities.resources || spec.capabilities.prompts {
        count += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Create a minimal valid YAML specification for testing
    fn create_valid_yaml_spec() -> &'static str {
        r#"
name: "Test Server"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
"#
    }

    #[tokio::test]
    async fn test_handle_test_missing_spec() {
        let result = handle_test(PathBuf::from("nonexistent.yaml"), None, false, None, 4).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_test_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, create_valid_yaml_spec()).unwrap();

        let result = handle_test(spec_file, None, false, None, 4).await;

        // Test execution will fail because test-server doesn't exist, but we should get a connection error
        match result {
            Ok(_) => panic!("Expected connection failure since test-server doesn't exist"),
            Err(e) => {
                // Should be a connection error
                println!("Expected error: {}", e);
                assert!(
                    e.to_string().contains("Connection")
                        || e.to_string().contains("connection")
                        || e.to_string().contains("Failed to create")
                        || e.to_string().contains("not found")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_handle_test_with_all_options() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        let output_file = temp_dir.path().join("output.json");
        fs::write(&spec_file, create_valid_yaml_spec()).unwrap();

        let result = handle_test(
            spec_file,
            Some(output_file),
            true,
            Some("filesystem".to_string()),
            8,
        )
        .await;

        // Test execution will fail because test-server doesn't exist, but we should get a connection error
        match result {
            Ok(_) => panic!("Expected connection failure since test-server doesn't exist"),
            Err(e) => {
                // Should be a connection error
                println!("Expected error: {}", e);
                assert!(
                    e.to_string().contains("Connection")
                        || e.to_string().contains("connection")
                        || e.to_string().contains("Failed to create")
                        || e.to_string().contains("not found")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_handle_validate_missing_spec() {
        let result = handle_validate(PathBuf::from("nonexistent.yaml")).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_validate_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, create_valid_yaml_spec()).unwrap();

        let result = handle_validate(spec_file).await;

        // Should succeed with valid YAML
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_list_missing_spec() {
        let result = handle_list(PathBuf::from("nonexistent.yaml"), false).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_list_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, create_valid_yaml_spec()).unwrap();

        let result = handle_list(spec_file, true).await;

        // Should succeed with valid YAML
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_version() {
        let result = handle_version();
        assert!(result.is_ok());
    }
}
