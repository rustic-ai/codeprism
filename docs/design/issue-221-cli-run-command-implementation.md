# Issue #221: CLI `run` Command Implementation Design Document

## 1. Problem Statement

The Mandrel Test Harness (`moth`) has a defined CLI structure with a `run` command, but the implementation is currently a placeholder. It does not connect to the `TestSuiteRunner` and cannot execute any tests. This issue is about implementing the logic to make the `moth run` command fully functional.

## 2. Proposed Solution

The solution is to implement the `handle_run_command` function in `crates/mandrel-mcp-th/src/cli/mod.rs`. This function will:
1.  Parse the `RunArgs` to configure the test run.
2.  Initialize the `McpClient` and `TestCaseExecutor`.
3.  Initialize the `TestSuiteRunner` with the correct configuration.
4.  Execute the test suite specified in the arguments.
5.  Pass the results to the `TestReporter`.
6.  Print a summary to the console and return an appropriate exit code.

### High-Level Architecture

```
┌──────────────┐      ┌─────────────────────┐      ┌───────────────────┐
│              │      │                     │      │                   │
│  CLI (`moth`)├─────▶│   `handle_run_command`  ├─────▶│  `TestSuiteRunner`  │
│              │      │      (in cli/mod.rs)    │      │ (in runner/mod.rs)│
└──────────────┘      └─────────────────────┘      └───────────────────┘
      ▲                        │                           │
      │                        │                           ▼
      │                        │                  ┌──────────────────┐
      │                        └─────────────────▶│  `TestReporter`  │
      │                                           │(in reporting/mod.rs)│
      └───────────────────────────────────────────┘
```

## 3. API Design

The main changes will be within the `handle_run_command` and a new `display_summary` function.

### `handle_run_command` in `cli/mod.rs`
```rust
// in `crates/mandrel-mcp-th/src/cli/mod.rs`
async fn handle_run_command(&self, args: &RunArgs) -> Result<i32> {
    // 1. Load the specification to get server config
    let spec_loader = SpecificationLoader::new();
    let spec = spec_loader.load_from_file(&args.config).await?;

    // 2. Initialize the client and executor
    // NOTE: This uses the ServerConfig from the loaded YAML spec
    let client = McpClient::new(spec.server.into()).await?;
    let executor_config = ExecutorConfig::default();
    let executor = TestCaseExecutor::new(Arc::new(Mutex::new(client)), executor_config);

    // 3. Initialize the TestSuiteRunner
    let runner_config = RunnerConfig::new()
        .with_parallel_execution(args.parallel)
        .with_fail_fast(args.fail_fast);
    let mut runner = TestSuiteRunner::new(executor, runner_config);

    // 4. Execute the test suite
    let suite_result = runner.run_test_suite(&args.config).await?;

    // 5. Generate a report
    // PLANNED(#194): This will be a more robust reporting implementation
    if let Some(output_dir) = &args.output {
        let report_path = output_dir.join("report.json");
        let report_json = serde_json::to_string_pretty(&suite_result)?;
        tokio::fs::write(report_path, report_json).await?;
    }
    
    // 6. Display summary and return exit code
    self.display_summary(&suite_result);
    Ok(if suite_result.failed == 0 { 0 } else { 1 })
}

fn display_summary(&self, result: &TestSuiteResult) {
    // Pretty-print a summary of the results to the console
    println!("\n✅ Test Suite Finished ✅");
    println!("Suite: {}", result.suite_name);
    println!("Total Tests: {}, Passed: {}, Failed: {}", 
        result.total_tests, result.passed, result.failed);
    println!("Duration: {:.2}s", result.total_duration.as_secs_f64());
}
```

### `RunArgs` in `cli/args.rs`
The existing `RunArgs` struct will be used to pass configuration to the `handle_run_command` function. We need to ensure it has all the necessary fields.

```rust
// in `crates/mandrel-mcp-th/src/cli/args.rs`
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Path to the test specification file.
    #[arg()]
    pub config: PathBuf,

    /// Output directory for generated reports.
    #[arg(short, long, default_value = "./reports")]
    pub output: Option<PathBuf>,

    /// Run tests in parallel.
    #[arg(long)]
    pub parallel: bool,

    /// Stop execution on the first test failure.
    #[arg(long)]
    pub fail_fast: bool,
}
```

## 4. Implementation Plan

1.  **Update `RunArgs`**: Add `parallel` and `fail_fast` flags to the `RunArgs` struct in `cli/args.rs`.
2.  **Implement `handle_run_command`**:
    *   Replace the placeholder `println!` with the logic described in the API design.
    *   Load the `TestSpecification` from the path in `RunArgs`.
    *   Use the `server` config from the spec to initialize `McpClient`.
    *   Create a `TestCaseExecutor`.
    *   Create and configure the `TestSuiteRunner` using arguments from `RunArgs`.
    *   Call `runner.run_test_suite()`.
3.  **Implement `display_summary`**: Add a private helper function to `CliApp` to print a formatted summary of `TestSuiteResult`.
4.  **Testing**:
    *   Add a new integration test in `crates/mandrel-mcp-th/tests/cli.rs`.
    *   The test will execute `moth run` against a simple YAML file.
    *   It will assert that the command exits with the correct code (0 for success, 1 for failure).
    *   It will check the console output for the summary.

## 5. Testing Strategy

*   **Integration Test (`cli_run_command_success`)**: Run `moth run` with `simple-test.yaml`. Assert exit code is 0 and stdout contains "Passed: 3".
*   **Integration Test (`cli_run_command_failure`)**: Run `moth run` with a spec where one test is expected to fail. Assert exit code is 1 and stdout contains "Failed: 1".
*   **Integration Test (`cli_run_command_fail_fast`)**: Run `moth run --fail-fast` on the failing spec. Assert that the number of executed tests is less than the total.
*   **Argument Parsing Tests**: Add unit tests in `cli/mod.rs` to ensure flags like `--parallel` and `--fail-fast` are parsed correctly.

This approach will provide a fully functional `run` command and set the stage for the final integration and documentation work. 