# Mandrel MCP Test Harness Design Document

## Problem Statement

We need a simple, focused MCP test harness that acts as an MCP client to validate MCP server functionality based on test specifications. The current implementation has scope creep, compilation issues, and doesn't align with the official MCP protocol specification.

**Core Problem**: Test any MCP server for protocol compliance and functional correctness without performance monitoring or unnecessary complexity.

## Proposed Solution

### High-Level Approach

A focused MCP test harness that operates as a **MCP client** connecting to servers under test via stdio transport, validating their capabilities and testing their functionality against expected specifications.

The framework provides the **moth** binary (MOdel context protocol Test Harness) for command-line testing operations.

### Component Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Suite    │    │  MCP Test        │    │  Test Report    │
│ Configuration   │───▶│  Harness         │───▶│   Generator     │
│    (YAML)       │    │   (Client)       │    │  (JSON/Table)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │   MCP Server     │
                       │  (Under Test)    │
                       │   stdio/JSON-RPC │
                       └──────────────────┘
```

## Requirements (5-Step Process)

Based on user-defined scope, the test harness must implement exactly these steps:

1. **Test Suite Input**: Accept test suite with server spec and expected capabilities
2. **Server Launch**: Launch MCP server as subprocess via stdio transport  
3. **Schema Validation**: Validate all tools/prompts/resources are present with correct I/O schemas
4. **Protocol Testing**: Test all built-in/protocol-specified API calls work as expected
5. **Test Execution & Reporting**: Run all test suites and publish comprehensive report

## API Design

### Core Components

```rust
/// Main test harness orchestrator
pub struct McpTestHarness {
    config: TestSuiteConfig,
    client: McpClient,
    validator: SchemaValidator,
    reporter: TestReporter,
}

/// MCP client for stdio communication
pub struct McpClient {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    request_id: AtomicU64,
}

/// Schema validation for MCP capabilities
pub struct SchemaValidator {
    json_schema_validator: JsonSchemaValidator,
}

/// Test result reporting
pub struct TestReporter {
    results: Vec<TestResult>,
    output_format: OutputFormat,
}
```

### Test Suite Configuration Format

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteConfig {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Expected capabilities from the server
    pub expected_capabilities: ExpectedCapabilities,
    
    /// Individual test cases to execute
    pub test_cases: Vec<TestCase>,
    
    /// Global test configuration
    pub settings: TestSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to launch the MCP server
    pub command: String,
    
    /// Arguments for the server command
    pub args: Vec<String>,
    
    /// Working directory for the server
    pub working_dir: Option<String>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Startup timeout in seconds
    pub startup_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedCapabilities {
    /// Expected tools with their schemas
    pub tools: Vec<ExpectedTool>,
    
    /// Expected prompts with their schemas
    pub prompts: Vec<ExpectedPrompt>,
    
    /// Expected resources with their schemas  
    pub resources: Vec<ExpectedResource>,
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (P0)
1. Fix compilation issues
2. Basic MCP client implementation with stdio transport
3. JSON-RPC 2.0 message handling per MCP spec

### Phase 2: MCP Protocol Implementation (P1)
1. MCP initialization handshake (`initialize`)
2. Capability discovery (`list_tools`, `list_prompts`, `list_resources`)
3. Functional testing (`tools/call`, `prompts/get`, `resources/read`)

### Phase 3: Validation & Reporting (P1)
1. Schema validation for expected vs actual capabilities
2. Test execution engine
3. Comprehensive reporting (table, JSON, YAML)

## Configuration Examples

### Basic Tool Testing
```yaml
server:
  command: "node"
  args: ["filesystem-server.js", "/tmp"]
  working_dir: "/path/to/server"
  startup_timeout: 10

expected_capabilities:
  tools:
    - name: "read_file"
      description: "Read contents of a file"
      input_schema:
        type: "object"
        properties:
          path:
            type: "string"
            description: "Path to the file to read"
        required: ["path"]
      required: true

test_cases:
  - id: "test_read_existing_file"
    description: "Test reading an existing file"
    test_type:
      ToolCall:
        tool_name: "read_file"
    input:
      path: "/tmp/test.txt"
    expected_result:
      success: true
```

## Success Criteria

### Functional Requirements
- [ ] Can launch any MCP server via stdio transport
- [ ] Validates server capabilities match expected schemas  
- [ ] Tests all MCP protocol APIs (initialize, list_*, tools/call, etc.)
- [ ] Generates comprehensive test reports
- [ ] Handles errors gracefully with detailed messages
- [ ] Simple YAML configuration format

### Out of Scope (Explicitly)
- Performance monitoring/benchmarking  
- HTTP transport (stdio only for now)
- Custom validation scripts
- Advanced metrics beyond pass/fail

## Alternative Approaches Considered

### 1. Performance-First Approach
**Rejected**: Added unnecessary complexity for benchmarking when the goal is functional validation.

### 2. Multi-Transport Support  
**Rejected**: HTTP transport adds complexity; stdio covers majority of MCP servers and is simpler to implement reliably.

### 3. Plugin-Based Validation
**Rejected**: Custom validation scripts add security concerns and implementation complexity beyond the core goal.

The chosen approach focuses on **simplicity, reliability, and clear scope** for MCP protocol compliance testing without feature creep.
