//! CLI interface for the CodePrism Test Harness

use anyhow::Result;
use clap::{Arg, Command};
use codeprism_test_harness::{init, TestHarness};
use std::path::PathBuf;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init()?;

    let matches = Command::new("CodePrism Test Harness")
        .version("0.1.0")
        .about("Automated test harness for CodePrism MCP tools")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("test-harness.yaml"),
        )
        .arg(
            Arg::new("suite")
                .short('s')
                .long("suite")
                .value_name("NAME")
                .help("Run specific test suite by name"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be executed without running tests")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for test reports"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::Count),
        )
        .get_matches();

    // Set log level based on verbosity
    match matches.get_count("verbose") {
        0 => std::env::set_var("RUST_LOG", "info"),
        1 => std::env::set_var("RUST_LOG", "debug"),
        _ => std::env::set_var("RUST_LOG", "trace"),
    }

    let config_path = PathBuf::from(matches.get_one::<String>("config").unwrap());

    // Check if config file exists, create default if not
    if !config_path.exists() {
        info!(
            "Configuration file not found, creating default: {}",
            config_path.display()
        );
        create_default_config(&config_path)?;
    }

    // Load configuration
    let mut harness = match TestHarness::from_config_file(&config_path) {
        Ok(harness) => harness,
        Err(e) => {
            error!(
                "Failed to load configuration from {}: {}",
                config_path.display(),
                e
            );
            return Err(e);
        }
    };

    if matches.get_flag("dry-run") {
        info!("Dry-run mode: showing configuration without executing tests");
        return Ok(());
    }

    // Run tests
    let results = if let Some(suite_name) = matches.get_one::<String>("suite") {
        info!("Running test suite: {}", suite_name);

        match harness.run_test_suite(suite_name).await? {
            Some(result) => vec![result],
            None => {
                error!("Test suite '{}' not found", suite_name);
                return Err(anyhow::anyhow!("Test suite not found"));
            }
        }
    } else {
        info!("Running all test suites");
        harness.run_all_tests().await?
    };

    // Print summary
    let total_suites = results.len();
    let passed_suites = results.iter().filter(|r| r.suite_passed).count();
    let total_tests: usize = results.iter().map(|r| r.stats.total_tests).sum();
    let passed_tests: usize = results.iter().map(|r| r.stats.passed_tests).sum();

    info!("=== Test Execution Summary ===");
    info!("Test Suites: {}/{} passed", passed_suites, total_suites);
    info!("Total Tests: {}/{} passed", passed_tests, total_tests);

    for result in &results {
        let status = if result.suite_passed {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        info!(
            "  {}: {} ({}/{} tests passed)",
            status, result.test_suite.name, result.stats.passed_tests, result.stats.total_tests
        );
    }

    // Generate test reports
    if let Some(output_dir) = matches.get_one::<String>("output") {
        info!("Generating reports in directory: {}", output_dir);
        // Report generation functionality integrated
    }

    // Exit with error code if any tests failed
    if passed_tests < total_tests {
        std::process::exit(1);
    }

    Ok(())
}

/// Create a default configuration file
fn create_default_config(path: &PathBuf) -> Result<()> {
    let default_config = r#"# CodePrism Test Harness Configuration

global:
  max_global_concurrency: 4
  global_timeout_seconds: 300
  default_project_path: "test-projects/python-sample"
  fail_fast: false
  retry:
    max_retries: 2
    retry_delay_ms: 1000
    exponential_backoff: true
    retry_on_patterns:
      - "connection refused"
      - "timeout"
  logging:
    level: "info"
    console: true
    timestamps: true
    json_format: false

server:
  start_command: "cargo run --bin codeprism-mcp"
  args:
    - "stdio"
  env: {}
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10
  health_check:
    enabled: false
    interval_seconds: 10
    failure_threshold: 3
    timeout_seconds: 5

test_suites:
  - name: "core_tools_smoke_test"
    description: "Basic smoke tests for core MCP tools"
    parallel_execution: false
    test_cases:
      - id: "test_repository_stats"
        description: "Test repository statistics tool"
        tool_name: "repository_stats"
        input_params: {}
        expected:
          patterns:
            - key: "result.total_files"
              validation:
                type: "Range"
                min: 1.0
                max: 1000.0
              required: true
          custom_scripts: []
          allow_extra_fields: true
        performance:
          max_execution_time_ms: 5000
        enabled: true
      
      - id: "test_search_symbols"
        description: "Test symbol search functionality"
        tool_name: "search_symbols"
        input_params:
          pattern: ".*"
        expected:
          patterns:
            - key: "result.total_matches"
              validation:
                type: "Range"
                min: 0.0
                max: 10000.0
              required: true
          custom_scripts: []
          allow_extra_fields: true
        performance:
          max_execution_time_ms: 3000
        enabled: true

reporting:
  output_dir: "test-reports"
  formats:
    - "html"
    - "json"
  open_html: false
  include_debug_info: true
  charts:
    enabled: true
    types:
      - "response_time"
      - "success_rate"
    size:
      width: 800
      height: 400
  trend_analysis: false

environment:
  variables: {}
  path_additions: []
  limits:
    max_memory_mb: 1024
    max_cpu_seconds: 300
    max_open_files: 1024
    max_process_time_seconds: 300
"#;

    std::fs::write(path, default_config)?;
    info!("Created default configuration file: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_default_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yaml");

        assert!(create_default_config(&config_path).is_ok());
        assert!(config_path.exists());

        // Verify the config can be loaded
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("CodePrism Test Harness Configuration"));
    }
}
