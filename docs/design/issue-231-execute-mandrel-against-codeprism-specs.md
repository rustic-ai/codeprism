# Issue #231: Execute mandrel-mcp-th against existing codeprism-moth-specs

## Problem Statement

We need to verify that the mandrel-mcp-th CLI can successfully execute the existing comprehensive CodePrism moth specifications. This is the foundation for end-to-end integration testing between the Mandrel MCP Test Harness and CodePrism MCP Server.

## Current State Analysis

**Existing Assets:**
- **mandrel-mcp-th CLI**: Working CLI with `run` command and comprehensive execution framework
- **CodePrism moth specs**: 4 comprehensive specifications with 2,500+ lines total
  - `codeprism-rust-comprehensive.yaml` (664 lines, 18 tools)
  - `codeprism-python-comprehensive.yaml` (631 lines, 18 tools)  
  - `codeprism-java-comprehensive.yaml` (664 lines, 19 tools)
  - `codeprism-javascript-comprehensive.yaml` (629 lines, 17 tools)
- **CodePrism MCP Server**: Real server implementation with 26 tools

## Proposed Solution

### Implementation Approach

Create integration tests that execute the existing comprehensive specifications against a real CodePrism server to validate:

1. **CLI Execution**: `moth run` command successfully loads and executes specifications
2. **Server Integration**: CodePrism server responds to all tool calls from specifications  
3. **Result Validation**: All test cases execute and produce meaningful results
4. **Error Handling**: Failed tests are properly detected and reported

### Test Strategy

```rust
#[tokio::test]
async fn test_execute_rust_comprehensive_spec() {
    // Arrange: Start CodePrism server and prepare spec
    let server = start_codeprism_server().await;
    let spec_path = "crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml";
    
    // Act: Execute mandrel-mcp-th against the spec
    let result = execute_moth_cli(&["run", spec_path]).await;
    
    // Assert: Verify execution succeeded and all tools responded
    assert!(result.success);
    assert_eq!(result.total_tests, 18); // 18 tools in Rust spec
    assert!(result.passed > 0); // At least some tests should pass
}
```

### Expected Outcomes

**Success Criteria:**
- CLI successfully loads all 4 comprehensive specifications
- CodePrism server responds to tool calls without connection errors
- Test execution completes for all 26 unique tools across specifications
- Results are captured in JSON reports for further analysis

**Performance Requirements:**
- Complete execution within 60 seconds per specification
- Memory usage under 500MB during test execution
- No connection timeouts or protocol errors

### Test Implementation Plan

**Phase 1: Basic CLI Execution (RED → GREEN)**
1. Create integration test framework in `tests/integration/`
2. Write failing tests for each comprehensive specification
3. Implement server startup and CLI execution helpers
4. Achieve GREEN tests with basic execution validation

**Phase 2: Result Validation (RED → GREEN)**  
1. Extend tests to validate result structure and content
2. Verify tool coverage matches specification expectations
3. Add performance timing and memory usage validation
4. Ensure error detection works correctly

**Phase 3: Comprehensive Coverage (REFACTOR)**
1. Test all 4 language specifications systematically
2. Validate cross-language tool behavior consistency
3. Add reporting and metrics collection
4. Optimize execution performance

### Integration Points

**With mandrel-mcp-th CLI:**
- Use existing `RunArgs` and `CliApp::run()` functionality
- Leverage `TestSuiteRunner` execution engine
- Utilize `SpecificationLoader` for YAML parsing

**With CodePrism server:**
- Start server using `cargo run --package codeprism-mcp-server` 
- Connect via stdio transport as specified in YAML
- Validate 26 tools respond to MCP protocol calls

**With existing specifications:**
- Use specifications exactly as-is without modification
- Validate specification structure matches CLI expectations
- Ensure all test cases have proper expected results defined

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_specification_loading() {
    let loader = SpecificationLoader::new().unwrap();
    let spec = loader.load_from_file(Path::new("crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-rust-comprehensive.yaml")).await;
    
    assert!(spec.is_ok());
    let spec = spec.unwrap();
    assert_eq!(spec.name, "CodePrism Rust Comprehensive Analysis");
    assert!(spec.tools.is_some());
    assert_eq!(spec.tools.as_ref().unwrap().len(), 18);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_specification_execution() {
    let server_process = spawn_codeprism_server().await;
    let result = execute_comprehensive_spec("rust").await;
    
    assert!(result.success);
    assert!(result.total_tests >= 18);
    assert!(result.execution_time < Duration::from_secs(60));
    
    server_process.kill().await;
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_execution_performance_requirements() {
    let start = Instant::now();
    let result = execute_comprehensive_spec("rust").await;
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(60));
    assert!(result.memory_usage_mb < 500);
}
```

## Success Criteria

**Functional Requirements:**
- [ ] All 4 comprehensive specifications load successfully
- [ ] CodePrism server responds to all tool types (18-19 per spec)
- [ ] CLI execution completes without protocol errors  
- [ ] Results contain meaningful test outcomes and timing data

**Quality Requirements:**
- [ ] 100% test coverage for CLI execution paths
- [ ] Integration tests verify server communication
- [ ] Performance tests validate timing requirements
- [ ] Error scenarios are tested and handled gracefully

**Documentation Requirements:**
- [ ] README updated with execution examples
- [ ] Test results documented with performance metrics
- [ ] Integration setup instructions for CI/CD

## Implementation Dependencies

**Internal Dependencies:**
- mandrel-mcp-th CLI implementation (completed)
- codeprism-mcp-server functionality (completed)
- Comprehensive moth specifications (completed)

**External Dependencies:**
- tokio async runtime for test execution
- assert_cmd for CLI testing
- tempfile for test workspace creation

## Risk Assessment

**Low Risk:**
- CLI and server implementations are stable
- Specifications are comprehensive and well-tested
- Integration pattern follows proven filesystem-server example

**Mitigation Strategies:**
- Start with simplest specification (Rust) to validate approach
- Use existing filesystem-server.yaml as execution template
- Implement timeout and retry logic for server startup
- Add comprehensive logging for debugging execution issues

## Breaking Changes

None expected. This implementation leverages existing components without modification.

## Alternative Approaches Considered

**Option A: Mock Server Testing** - Rejected because real server testing provides authentic validation
**Option B: Specification Modification** - Rejected to maintain specifications as authoritative test cases  
**Option C: Manual Testing Only** - Rejected because automated testing enables CI/CD integration

This design enables systematic validation of the complete MCP test harness workflow using real components and comprehensive test coverage. 