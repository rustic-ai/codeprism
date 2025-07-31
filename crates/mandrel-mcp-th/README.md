# Mandrel MCP Test Harness

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Mandrel** is a modern, comprehensive testing framework for MCP (Model Context Protocol) servers built on the official Rust SDK. It provides validation, compliance testing, and detailed reporting for MCP server implementations.

The project includes the **moth** binary (MOdel context protocol Test Harness) for command-line testing operations.

## Features

- ✅ **SDK-First**: Built on the official MCP Rust SDK for guaranteed protocol compliance
- ✅ **Transport Agnostic**: Supports stdio, HTTP, and SSE transports  
- ✅ **Comprehensive Testing**: Protocol compliance, capability validation, and stress testing
- ✅ **Rich Reporting**: HTML, JSON, and JUnit XML report formats
- ✅ **Developer Friendly**: Clear error messages, detailed logs, and interactive CLI

## Quick Start

### Installation

```bash
# Install from source (in project root)
cargo install --path crates/mandrel-mcp-th

# Verify installation
moth --version
```

### Basic Usage

```bash
# Run tests from a specification file
moth run my-server.yaml

# Validate a test specification
moth validate my-server.yaml

# Generate reports from test results
moth report --format html --output results.html

# Show version information
moth --version
```

### Test Specification Example

```yaml
name: "Filesystem MCP Server"
version: "1.0.0"
description: "Test specification for filesystem operations"

capabilities:
  tools: true
  resources: true

server:
  command: "node"
  args: ["filesystem-server.js", "--sandbox", "/allowed/path"]
  transport: "stdio"
  startup_timeout_seconds: 10

tools:
  - name: "read_file"
    description: "Read file contents"
    tests:
      - name: "read_existing_file"
        input:
          path: "/allowed/path/test.txt"
        expected:
          error: false
          schema:
            type: object
            required: ["content"]
            properties:
              content:
                type: string
```

## Architecture

```text
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Config    │───▶│   Client    │───▶│  Executor   │
│  (YAML)     │    │ (MCP/rmcp)  │    │ (Test Run)  │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Validation  │    │   Server    │    │  Reporting  │
│  (Schema)   │    │ (Process)   │    │ (JSON/HTML) │
└─────────────┘    └─────────────┘    └─────────────┘
```

## CLI Commands

### `moth run`

Run test specifications against MCP servers.

```bash
moth run [OPTIONS] <SPEC>

Options:
  --output <DIR>               Output directory for test results
  --formats <FORMAT>           Output formats (json, html, junit)
```

### `moth validate`

Validate test specification syntax and structure.

```bash
moth validate <SPEC>
```

### `moth report`

Generate reports from test execution results.

```bash
moth report [OPTIONS]

Options:
  --format <FORMAT>            Report format (json, html, junit)
  --output <FILE>              Output file path
```

## Library Usage

Mandrel can also be used as a library in your Rust projects:

```rust
use mandrel_mcp_th::{cli::Commands, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Use Mandrel programmatically
    // Implementation details coming soon...
    Ok(())
}
```

## Development

### Building

```bash
# Build the project
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## 📚 Documentation

For complete documentation, see:

- **[Quick Start Guide](../../docs/test-harness/getting-started/quick-start.md)** - Get up and running in 5 minutes
- **[Installation Guide](../../docs/test-harness/getting-started/installation.md)** - Complete installation instructions for all platforms
- **[CLI Reference](../../docs/test-harness/cli-reference.md)** - Complete command-line documentation
- **[Configuration Reference](../../docs/test-harness/configuration-reference.md)** - Complete YAML specification format
- **[User Guide](../../docs/test-harness/user-guide.md)** - Comprehensive testing guide
- **[Examples](../../docs/test-harness/examples/)** - Real-world test specifications
- **[Troubleshooting](../../docs/test-harness/troubleshooting.md)** - Common issues and solutions

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

See our [Contributing Guide](../../docs/test-harness/contributing.md) for detailed development instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Projects

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk) - Official MCP Rust implementation
- [MCP Specification](https://spec.modelcontextprotocol.io/) - Protocol specification
- [MCP Servers](https://github.com/modelcontextprotocol/servers) - Reference server implementations 