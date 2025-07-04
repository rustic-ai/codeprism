# Mandrel MCP Test Harness - Project Overview

## Project Identity

**Project Name**: Mandrel MCP Test Harness  
**Binary Name**: moth (MOdel context protocol Test Harness)  
**Crate Name**: mandrel-mcp-th  
**GitHub Repository**: Part of the CodePrism workspace  

## What is Mandrel?

**Mandrel** is a comprehensive, modern testing framework for Model Context Protocol (MCP) servers built on the official Rust SDK. It provides validation, compliance testing, and detailed reporting for MCP server implementations.

The name "Mandrel" reflects the project's role as a core testing infrastructure component - like a mandrel (a shaft or spindle) that supports and shapes the testing process for MCP servers.

## Project Components

### Core Binary: `moth`
The **moth** binary is the command-line interface for the Mandrel test harness:
- **Full Name**: MOdel context protocol Test Harness
- **Purpose**: Command-line testing operations and workflows
- **Etymology**: A reference to Grace Hopper's famous "first computer bug" - a moth found in the Harvard Mark II computer

### Library: `mandrel-mcp-th`
The underlying Rust library that powers the moth binary:
- **Crate Name**: mandrel-mcp-th
- **Purpose**: Programmatic API for embedding testing capabilities
- **Features**: Complete MCP testing framework as a library

## Architecture Overview

```text
┌─────────────────────┐
│   Mandrel Project   │
│  (Test Framework)   │
└─────────────────────┘
          │
          ▼
┌─────────────────────┐    ┌─────────────────────┐
│   moth Binary      │    │  mandrel-mcp-th    │
│  (CLI Interface)    │───▶│    (Library)        │
└─────────────────────┘    └─────────────────────┘
          │                          │
          ▼                          ▼
┌─────────────────────┐    ┌─────────────────────┐
│   Test Execution    │    │   MCP Client SDK    │
│   (Test Runner)     │    │  (rmcp official)    │
└─────────────────────┘    └─────────────────────┘
```

## Key Features

### 🔧 **Universal Server Testing**
- **Server Agnostic**: Tests any MCP server implementation
- **Protocol Compliance**: Validates full JSON-RPC 2.0 and MCP protocol adherence
- **Capability Discovery**: Automatically discovers and validates server capabilities
- **Transport Support**: Supports stdio transport initially, HTTP/SSE planned

### 📋 **Specification-Driven Testing**
- **YAML Configuration**: Human-readable test specifications
- **Schema Validation**: JSON Schema validation for input/output compliance
- **Field-Level Validation**: JSONPath-based field validation
- **Error Scenario Testing**: Comprehensive error handling validation

### 🚀 **Advanced Execution Engine**
- **Parallel Execution**: Configurable concurrency for efficient testing
- **Retry Logic**: Intelligent retry mechanisms with exponential backoff
- **Test Isolation**: Per-test process isolation for reliable results
- **Performance Monitoring**: Optional latency and throughput tracking

### 📊 **Rich Reporting**
- **Multiple Formats**: JSON, HTML, JUnit XML, and text reports
- **Detailed Diagnostics**: Comprehensive error analysis and context
- **CI/CD Integration**: Seamless integration with GitHub Actions
- **Progress Tracking**: Real-time test execution monitoring

## Usage Examples

### Command Line Usage
```bash
# Run tests from a specification file
moth test filesystem-server.yaml

# Validate a test specification
moth validate filesystem-server.yaml

# List available tests with details
moth list filesystem-server.yaml --detailed

# Run tests with specific output format
moth test filesystem-server.yaml --output json --output-file results.json
```

### Library Usage
```rust
use mandrel_mcp_th::{McpClient, TestRunner, TestSpecification};

#[tokio::main]
async fn main() -> Result<()> {
    // Load test specification
    let spec = TestSpecification::load("filesystem-server.yaml").await?;
    
    // Create MCP client
    let client = McpClient::new(spec.server_config()).await?;
    
    // Run tests
    let runner = TestRunner::new(client);
    let results = runner.run_tests(&spec.test_cases).await?;
    
    // Generate report
    results.generate_report("results.json").await?;
    
    Ok(())
}
```

## Project Structure

```
crates/mandrel-mcp-th/
├── Cargo.toml              # Project dependencies and metadata
├── README.md               # Project documentation
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # moth binary entry point
│   ├── error.rs            # Error types and handling
│   ├── client/             # MCP client implementation
│   ├── executor/           # Test execution engine
│   ├── spec/               # Test specification parsing
│   ├── validation/         # Result validation
│   ├── reporting/          # Report generation
│   └── cli/                # Command-line interface
└── tests/                  # Integration tests
```

## Development Status

### Completed (P0 - Foundation) ✅
- **Issue #188**: Complete crate structure and setup
- **Issue #189**: Basic MCP client using rmcp SDK
- **Issue #190**: CLI interface with moth binary
- **Issue #191**: Test execution framework

### In Progress (P1 - Core Features)
- **Issue #192**: YAML specification parser
- **Issue #193**: MCP protocol validation engine
- **Issue #194**: JSON reporting system
- **Issue #195**: Error handling and logging

### Planned (P2+ - Advanced Features)
- **Parallel Execution**: Concurrent test execution
- **Advanced Reporting**: HTML and JUnit XML formats
- **Performance Monitoring**: Latency and throughput tracking
- **Template System**: Common test patterns and templates

## Target MCP Servers

The Mandrel test harness is designed to work with any MCP server, with initial focus on:

1. **Filesystem Server**: File system operations and management
2. **Everything Server**: Comprehensive MCP capability demonstration
3. **Weather Server**: External API integration and data fetching

## Contributing

The Mandrel project follows strict development standards:

- **Design-First Development**: All features require design documents
- **Test-Driven Development**: Comprehensive test coverage required
- **Documentation Standards**: All public APIs must be documented
- **Code Quality**: Zero warnings policy with full clippy compliance

## References

- **[MCP Specification](https://spec.modelcontextprotocol.io/)**: Official protocol specification
- **[MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)**: Official Rust implementation
- **[Design Documents](design/)**: Technical architecture and implementation plans
- **[Test Harness Documentation](test-harness/)**: Complete usage and configuration guides

---

**The Mandrel MCP Test Harness provides the robust testing infrastructure needed to ensure MCP server implementations are reliable, compliant, and ready for production use.** 