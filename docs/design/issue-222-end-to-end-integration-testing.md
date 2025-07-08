# Issue #222: End-to-End Integration Testing Design Document

## Problem Statement

The Mandrel MCP Test Harness has individual components (executor, runner, CLI) but lacks comprehensive end-to-end integration testing to ensure the complete system works together correctly with real MCP servers. Without proper integration testing, we cannot verify that the complete workflow from CLI invocation through test execution to report generation functions correctly.

## Proposed Solution

Implement a comprehensive end-to-end integration testing framework that validates the complete test harness functionality using both mock and real MCP servers. This includes testing all component integrations, error scenarios, and report generation capabilities.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Integration Test Suite                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Mock Server   │  │  Real Server    │  │   Test Data     │ │
│  │   Testing       │  │   Testing       │  │   & Fixtures    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Component     │  │   Error         │  │   Performance   │ │
│  │   Integration   │  │   Handling      │  │   Testing       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   CLI Testing   │  │   Report        │  │   Validation    │ │
│  │                 │  │   Generation    │  │   Engine        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Component Design

#### 1. Real MCP Server Integration

```rust
// Real server configuration and management
pub struct RealServerConfig {
    pub server_type: ServerType,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub startup_timeout: Duration,
}

pub enum ServerType {
    CodePrism,    // crates/codeprism-mcp-server
    Filesystem,   // @modelcontextprotocol/server-filesystem
    Memory,       // @modelcontextprotocol/server-memory
    Weather,      // Weather API server
    Git,          // Git operations server
}

impl RealServerConfig {
    pub fn codeprism_server() -> Self;
    pub fn filesystem_server(allowed_path: &str) -> Self;
    pub fn memory_server() -> Self;
    pub fn weather_server() -> Self;
}
```

#### 2. Integration Test Framework

```rust
// Main integration test framework
pub struct IntegrationTestFramework {
    pub mock_server: Option<MockMcpServer>,
    pub test_data_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl IntegrationTestFramework {
    pub fn new() -> Self;
    pub async fn setup(&mut self) -> Result<(), IntegrationTestError>;
    pub async fn teardown(&mut self) -> Result<(), IntegrationTestError>;
    pub async fn run_cli_test(&self, spec_file: &str, args: &[&str]) -> Result<TestResult, IntegrationTestError>;
    pub async fn validate_reports(&self, expected_reports: &[ReportExpectation]) -> Result<(), IntegrationTestError>;
}

// Test result structure
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub generated_files: Vec<PathBuf>,
}

// Report validation expectations
pub struct ReportExpectation {
    pub format: ReportFormat,
    pub file_path: PathBuf,
    pub expected_content: Vec<ContentExpectation>,
}

pub enum ContentExpectation {
    ContainsText(String),
    JsonPath(String, Value),
    ElementCount(String, usize),
    FileSize(Range<u64>),
}
```

#### 3. Test Data and Fixtures

```rust
// Test data management
pub struct TestFixtures {
    pub spec_files: HashMap<String, PathBuf>,
    pub expected_outputs: HashMap<String, Value>,
    pub error_scenarios: Vec<ErrorScenario>,
}

impl TestFixtures {
    pub fn load_all() -> Result<Self, TestFixturesError>;
    pub fn get_spec(&self, name: &str) -> Option<&PathBuf>;
    pub fn get_expected_output(&self, spec_name: &str) -> Option<&Value>;
    pub fn create_temp_spec(&self, content: &str) -> Result<PathBuf, TestFixturesError>;
}

// Error scenario testing
pub struct ErrorScenario {
    pub name: String,
    pub spec_content: String,
    pub expected_error: ExpectedError,
}

pub enum ExpectedError {
    InvalidYaml,
    ServerConnectionFailure,
    ToolExecutionTimeout,
    MalformedResponse,
    ValidationFailure,
}
```

## Implementation Plan

### Phase 1: Real Server Configuration
1. **Create Real Server Management** (`crates/mandrel-mcp-th/src/testing/server_config.rs`)
   - Implement configurations for CodePrism, filesystem, memory servers
   - Add process management and lifecycle handling
   - Include startup validation and health checks

2. **Implement Test Specifications Loader** (`crates/mandrel-mcp-th/src/testing/spec_loader.rs`)
   - Load codeprism-moth-specs YAML configurations
   - Parse comprehensive test specifications
   - Validate test parameters and expectations

### Phase 2: Integration Test Framework
1. **Create Integration Test Framework** (`crates/mandrel-mcp-th/src/testing/integration_framework.rs`)
   - Implement test setup/teardown
   - Add CLI execution helpers
   - Include report validation logic

2. **Implement Test Data Management** (`crates/mandrel-mcp-th/src/testing/fixtures.rs`)
   - Create test specification files
   - Add expected output samples
   - Include error scenario definitions

### Phase 3: Component Integration Tests
1. **CLI Integration Tests** (`crates/mandrel-mcp-th/tests/integration/cli_tests.rs`)
   - Test complete CLI workflow
   - Validate argument parsing and execution
   - Test error handling and user feedback

2. **Runner Integration Tests** (`crates/mandrel-mcp-th/tests/integration/runner_tests.rs`)
   - Test TestSuiteRunner with real specifications
   - Validate test execution orchestration
   - Test concurrent execution capabilities

3. **Executor Integration Tests** (`crates/mandrel-mcp-th/tests/integration/executor_tests.rs`)
   - Test TestCaseExecutor with mock servers
   - Validate MCP protocol handling
   - Test error recovery mechanisms

### Phase 4: Real Server Integration Tests
1. **CodePrism Server Tests** (`crates/mandrel-mcp-th/tests/integration/codeprism_tests.rs`)
   - Test with real CodePrism MCP server
   - Validate all 26 tools execution
   - Test multi-language support

2. **Performance Tests** (`crates/mandrel-mcp-th/tests/integration/performance_tests.rs`)
   - Test execution time requirements
   - Validate memory usage patterns
   - Test concurrent execution limits

### Phase 5: Error Handling and Report Generation Tests
1. **Error Scenario Tests** (`crates/mandrel-mcp-th/tests/integration/error_tests.rs`)
   - Test server disconnection scenarios
   - Validate timeout handling
   - Test malformed specification handling

2. **Report Generation Tests** (`crates/mandrel-mcp-th/tests/integration/report_tests.rs`)
   - Test HTML report generation
   - Validate JSON report format
   - Test JUnit XML compatibility

## Testing Strategy

### Test Categories

1. **Unit Tests** (90% coverage target)
   - Mock server components
   - Test framework utilities
   - Report validation logic

2. **Integration Tests** (80% coverage target)
   - Complete CLI workflow
   - Component integration
   - Real server interactions

3. **End-to-End Tests** (100% scenario coverage)
   - Full workflow validation
   - Error scenario handling
   - Performance requirements

### Test Data Organization

```
crates/mandrel-mcp-th/tests/
├── integration/
│   ├── cli_tests.rs
│   ├── runner_tests.rs
│   ├── executor_tests.rs
│   ├── codeprism_tests.rs
│   ├── performance_tests.rs
│   ├── error_tests.rs
│   └── report_tests.rs
├── fixtures/
│   ├── specifications/
│   │   ├── comprehensive.yaml
│   │   ├── tools-only.yaml
│   │   ├── workflow.yaml
│   │   └── error-scenarios/
│   ├── expected_outputs/
│   └── test_data/
└── utils/
    ├── mod.rs
    └── test_helpers.rs
```

### Performance Requirements

- **CLI Execution**: <2 seconds for basic operations
- **Comprehensive Test Suite**: <10 seconds total execution
- **Report Generation**: <1 second for all formats
- **Memory Usage**: <100MB during execution
- **Concurrent Tests**: Support up to 10 parallel test cases

## Success Criteria

### Functional Requirements
- [ ] Complete CLI workflow executes successfully
- [ ] All CodePrism test specifications execute correctly
- [ ] Report generation works in all formats (HTML, JSON, JUnit XML)
- [ ] Error handling provides actionable feedback
- [ ] Mock server supports all required testing scenarios

### Quality Requirements
- [ ] 90% unit test coverage for new components
- [ ] 80% integration test coverage for workflows
- [ ] 100% error scenario coverage
- [ ] All tests pass consistently in CI/CD

### Performance Requirements
- [ ] CLI execution completes in <2 seconds
- [ ] Comprehensive test suite completes in <10 seconds
- [ ] Memory usage remains below 100MB
- [ ] Concurrent execution supports 10+ test cases

### Reliability Requirements
- [ ] Tests pass consistently across environments
- [ ] Error recovery mechanisms work correctly
- [ ] Report generation is deterministic
- [ ] Mock server handles edge cases gracefully

## Alternative Approaches Considered

### 1. External Test Runner Integration
**Approach**: Use existing test runners like pytest or cargo test
**Pros**: Leverages existing tooling, familiar patterns
**Cons**: Doesn't test complete CLI workflow, limited report format control
**Decision**: Rejected - Need to test complete CLI integration

### 2. Docker-based Testing
**Approach**: Run all tests in containerized environments
**Pros**: Consistent environments, isolation
**Cons**: Increased complexity, slower execution
**Decision**: Considered for future enhancement, not initial implementation

### 3. Property-based Testing
**Approach**: Use property-based testing frameworks
**Pros**: Comprehensive edge case coverage
**Cons**: Complex setup, longer execution times
**Decision**: Included for specific components, not primary strategy

## Breaking Changes

**None expected** - This is purely additive testing infrastructure that doesn't affect existing APIs or user-facing functionality.

## Testing Implementation

### Real Server Testing
```rust
#[tokio::test]
async fn test_codeprism_server_basic_functionality() {
    let config = RealServerConfig::codeprism_server();
    let mut server = RealMcpServer::start(config).await.unwrap();
    
    // Test server responds to actual CodePrism tools
    let client = server.get_client().await.unwrap();
    let response = client.call_tool("analyze_code_quality", json!({
        "target": "test-projects/rust-test-project/src/main.rs",
        "quality_types": ["all"]
    })).await.unwrap();
    
    assert!(response.is_success());
    let content = response.content.unwrap();
    assert!(content.get("analysis").is_some());
    
    server.stop().await.unwrap();
}
```

### CLI Integration Testing
```rust
#[tokio::test]
async fn test_complete_cli_workflow_with_codeprism() {
    let framework = IntegrationTestFramework::new();
    framework.setup().await.unwrap();
    
    // Test CLI execution with real CodePrism moth specifications
    let result = framework.run_cli_test(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml",
        &["--output", "reports", "--format", "html"]
    ).await.unwrap();
    
    assert_eq!(result.exit_code, 0);
    
    // Validate reports were generated with real tool results
    framework.validate_reports(&[
        ReportExpectation {
            format: ReportFormat::Html,
            file_path: PathBuf::from("reports/codeprism_test_report.html"),
            expected_content: vec![
                ContentExpectation::ContainsText("CodePrism Analysis Results".to_string()),
                ContentExpectation::ElementCount("tool-result", 26), // All 26 CodePrism tools
                ContentExpectation::ContainsText("analyze_code_quality"),
                ContentExpectation::ContainsText("search_symbols"),
            ],
        }
    ]).await.unwrap();
    
    framework.teardown().await.unwrap();
}
```

### Error Scenario Testing
```rust
#[tokio::test]
async fn test_server_disconnection_handling() {
    let config = RealServerConfig::codeprism_server();
    let mut server = RealMcpServer::start(config).await.unwrap();
    
    // Start test execution
    let framework = IntegrationTestFramework::new();
    
    // Stop server mid-execution to simulate disconnection
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        server.force_stop().await.unwrap();
    });
    
    let result = framework.run_cli_test(
        "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml", 
        &[]
    ).await.unwrap();
    
    // Should handle gracefully and provide useful error message
    assert_ne!(result.exit_code, 0);
    assert!(result.stderr.contains("Server connection lost") || 
            result.stderr.contains("CodePrism server disconnected"));
}
```

This comprehensive design ensures complete end-to-end integration testing coverage while maintaining high quality standards and following TDD principles. 