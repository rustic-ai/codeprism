# MCP Test Harness

A universal testing tool for Model Context Protocol (MCP) servers. Test any MCP server implementation for protocol compliance, performance, and reliability.

[![Crate](https://img.shields.io/crates/v/mcp-test-harness.svg)](https://crates.io/crates/mcp-test-harness)
[![Documentation](https://docs.rs/mcp-test-harness/badge.svg)](https://docs.rs/mcp-test-harness)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- 🔧 **Universal MCP Server Testing** - Works with any MCP server implementation
- 🚀 **Multiple Transport Support** - stdio, HTTP
- 🛡️ **Protocol Compliance** - Comprehensive MCP protocol validation
- 🐳 **Docker Ready** - Containerized testing for CI/CD pipelines
- 📋 **Flexible Configuration** - YAML/JSON configuration with templates
- �� **Auto-Discovery** - Automatic MCP server detection
- 📈 **Rich Reporting** - Table, JSON, YAML output formats

## Quick Start

### Installation

```bash
# Install from crates.io
cargo install mcp-test-harness

# Or build from source
git clone https://github.com/rustic-ai/codeprism
cd prism/mcp-test-harness-standalone
cargo install --path .
```

### Basic Usage

```bash
# Test a stdio MCP server
mcp-test-harness test \
  --config configs/examples/basic-mcp-server.yaml \
  --server-cmd "node my-mcp-server.js"

# Test an HTTP MCP server
mcp-test-harness test \
  --config my-http-server-config.yaml

# Generate a configuration template
mcp-test-harness template --server-type filesystem --output my-config.yaml

# Discover running MCP servers
mcp-test-harness discover --port-range "3000-3010"


```

### Docker Usage

```bash
# Pull the image
docker pull mcp-test-harness:latest

# Run tests with Docker
docker run --rm -v $(pwd)/configs:/configs \
  mcp-test-harness test --config /configs/my-server.yaml

# Use in CI/CD
docker run --rm -v $(pwd):/workspace -w /workspace \
  mcp-test-harness test --config test-config.yaml --output json
```

## Configuration

### Basic Configuration Structure

```yaml
global:
  max_global_concurrency: 2
  timeout_seconds: 30
  fail_fast: false

server:
  transport: "stdio"  # or "http"
  command: "node server.js"
  args: ["--port", "3000"]
  working_dir: "/path/to/server"
  env:
    NODE_ENV: "test"



test_suites:
  - name: "MCP Protocol Compliance"
    test_cases:
      - id: "initialize"
        tool_name: "initialize"
        enabled: true
        input_params:
          protocolVersion: "2024-11-05"
        expected:
          patterns:
            - key: "protocolVersion"
              validation: { type: "exists" }
              required: true
```

### Server Types

#### stdio Server
```yaml
server:
  transport: "stdio"
  command: "node"
  args: ["server.js"]
```

#### HTTP Server
```yaml
server:
  transport: "http"
  url: "http://localhost:3000"
  connection_timeout: 10
```



## Validation Patterns

The test harness supports comprehensive response validation:

```yaml
expected:
  patterns:
    # Check if field exists
    - key: "status"
      validation: { type: "exists" }
      required: true
    
    # Check exact value
    - key: "status"
      validation: { type: "equals", value: "ok" }
      required: true
    
    # Check numeric range
    - key: "count"
      validation: { type: "range", min: 1.0, max: 100.0 }
      required: true
    
    # Check array structure
    - key: "items"
      validation: { type: "array" }
      required: true
    
    # Check array length
    - key: "items"
      validation: { type: "array_min_length", min_length: 1 }
      required: true
    
    # Check boolean value
    - key: "enabled"
      validation: { type: "boolean", value: true }
      required: true
```



## Templates

Generate configuration templates for common server types:

```bash
# Filesystem server template
mcp-test-harness template --server-type filesystem --output fs-config.yaml

# Database server template
mcp-test-harness template --server-type database --output db-config.yaml

# API wrapper template
mcp-test-harness template --server-type api --output api-config.yaml

# List available templates
mcp-test-harness list
```

## CI/CD Integration

### GitHub Actions

```yaml
name: MCP Server Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test MCP Server
        run: |
          docker run --rm -v $(pwd):/workspace -w /workspace \
            mcp-test-harness test \
            --config .github/mcp-test-config.yaml \
            --output json > test-results.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-results.json
```

### GitLab CI

```yaml
mcp-tests:
  image: mcp-test-harness:latest
  script:
    - mcp-test-harness test --config ci-config.yaml --output junit > results.xml
  artifacts:
    reports:
      junit: results.xml
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Support

- 📖 [Documentation](https://docs.rs/mcp-test-harness)
- 🐛 [Issue Tracker](https://github.com/rustic-ai/codeprism/issues)
- 💬 [Discussions](https://github.com/rustic-ai/codeprism/discussions)
- �� [Email Support](mailto:team@codeprism.ai)
