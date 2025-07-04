---
title: Quick Start Guide
description: Get up and running with Mandrel MCP Test Harness in under 5 minutes
sidebar_position: 1
---

# Quick Start Guide - Mandrel MCP Test Harness

Get up and running with Mandrel in under 5 minutes! This guide will walk you through installing the test harness and running your first MCP server test.

## 🚀 Prerequisites

- **Rust 1.70+** - Download from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning examples
- An MCP server to test (we'll provide a simple example)

## 📦 Installation

### Option 1: Install from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Build and install moth binary
cargo install --path crates/mandrel-mcp-th

# Verify installation
moth --version
```

### Option 2: Build Locally

```bash
# In the codeprism directory
cd crates/mandrel-mcp-th
cargo build --release

# Binary will be available at ../../target/release/moth
../../target/release/moth --version
```

## 🎯 Your First Test

### Step 1: Create a Simple Test Server

Let's create a minimal "echo" MCP server for testing purposes:

```bash
# Create a simple echo script that mimics MCP protocol
cat > echo-mcp-server.sh << 'EOF'
#!/bin/bash
# Simple echo server that responds to basic MCP initialization

while IFS= read -r line; do
    if echo "$line" | grep -q '"method":"initialize"'; then
        echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","serverInfo":{"name":"echo-server","version":"1.0.0"},"capabilities":{"tools":true}}}'
    elif echo "$line" | grep -q '"method":"tools/list"'; then
        echo '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"echo","description":"Echo back input","inputSchema":{"type":"object","properties":{"message":{"type":"string"}}}}]}}'
    else
        echo '{"jsonrpc":"2.0","id":null,"error":{"code":-32601,"message":"Method not found"}}'
    fi
done
EOF

chmod +x echo-mcp-server.sh
```

### Step 2: Create a Test Specification

Create a file called `echo-test.yaml`:

```yaml
name: "Echo MCP Server Test"
version: "1.0.0"
description: "Basic test for echo MCP server"

capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false

server:
  command: "./echo-mcp-server.sh"
  args: []
  transport: "stdio"
  startup_timeout_seconds: 10
  shutdown_timeout_seconds: 5

tools:
  - name: "echo"
    description: "Test echo tool functionality"
    tests:
      - name: "basic_echo"
        description: "Test basic echo functionality"
        input:
          message: "Hello, MCP!"
        expected:
          error: false
          fields:
            - path: "$.result"
              field_type: "object"
              required: true

test_config:
  timeout_seconds: 30
  max_concurrency: 1
  fail_fast: true
```

### Step 3: Validate Your Test Specification

```bash
# Check if your YAML is valid
moth validate echo-test.yaml
```

Expected output:
```
✅ Specification validation successful
- Server: Echo MCP Server Test v1.0.0
- Capabilities: tools
- Tests: 1 test case found
- Configuration: Valid
```

### Step 4: List Available Tests

```bash
# See what tests will be executed
moth list echo-test.yaml --detailed
```

Expected output:
```
📋 Test Specification: Echo MCP Server Test
   Version: 1.0.0
   Description: Basic test for echo MCP server

🔧 Tools (1):
  • echo - Test echo tool functionality
    ├─ basic_echo
    │  Test basic echo functionality

📊 Total Tests: 2
```

### Step 5: Run Your First Test

```bash
# Execute the test suite
moth test echo-test.yaml --output text
```

Expected output:
```
🧪 Starting test execution: Echo MCP Server Test v1.0.0
🔗 Connecting to MCP server...
✅ Server connected successfully
🏃 Running 1 test case...

Tool Test: echo::basic_echo
  ✅ Test passed in 45ms
  📊 Response validation: All fields matched

📋 Test Summary:
  Total: 1 test
  Passed: 1 ✅
  Failed: 0 ❌
  Duration: 1.2s

✅ All tests passed!
```

## 🛠️ Testing a Real MCP Server

### Option 1: Test the CodePrism MCP Server

If you have the full CodePrism project, you can test the built-in MCP server:

```yaml
name: "CodePrism MCP Server"
version: "1.0.0"
description: "Test CodePrism code analysis capabilities"

capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false

server:
  command: "cargo"
  args: ["run", "--bin", "codeprism-mcp", "--", "stdio"]
  env:
    RUST_LOG: "info"
  transport: "stdio"
  startup_timeout_seconds: 30

tools:
  - name: "repository_stats"
    description: "Test repository statistics"
    tests:
      - name: "get_repo_stats"
        description: "Get basic repository statistics"
        input:
          path: "."
        expected:
          error: false
          fields:
            - path: "$.total_files"
              field_type: "integer"
              required: true
            - path: "$.languages"
              field_type: "object"
              required: true
```

### Option 2: Test an External MCP Server

For external servers, adjust the `server` section:

```yaml
server:
  command: "node"
  args: ["path/to/your/server.js"]
  env:
    NODE_ENV: "test"
  working_dir: "/path/to/server"
  transport: "stdio"
```

## 📊 Understanding Test Output

### JSON Output (Default)

```bash
moth test echo-test.yaml --output json
```

Produces structured results perfect for CI/CD integration:

```json
{
  "suite_name": "Echo MCP Server Test",
  "total_tests": 1,
  "passed": 1,
  "failed": 0,
  "skipped": 0,
  "duration_ms": 1245,
  "test_results": [
    {
      "name": "basic_echo",
      "status": "passed",
      "duration_ms": 45,
      "performance": {
        "response_time_ms": 12,
        "memory_usage_bytes": null
      }
    }
  ]
}
```

### HTML Report

```bash
moth test echo-test.yaml --output html --output-file report.html
```

Generates a comprehensive HTML report with:
- Interactive test results
- Performance charts
- Detailed error information
- Server communication logs

### JUnit XML

```bash
moth test echo-test.yaml --output junit --output-file results.xml
```

Perfect for CI/CD systems like Jenkins, GitHub Actions, or GitLab CI.

## ⚡ CLI Options Reference

### `moth test` - Run Tests

```bash
moth test [OPTIONS] <SPEC>

Arguments:
  <SPEC>  Path to test specification file or directory

Options:
  -o, --output-file <FILE>    Output file for test results
  -f, --fail-fast             Stop execution on first failure
  -F, --filter <PATTERN>      Test filter pattern (regex)
  -c, --concurrency <N>       Maximum concurrent tests [default: 4]
      --output <FORMAT>       Output format [json, html, junit, text]
  -v, --verbose               Enable verbose output
```

### `moth validate` - Validate Specification

```bash
moth validate [OPTIONS] <SPEC>

Options:
  -v, --verbose     Enable verbose output
      --output <FORMAT>  Output format [json, text]
```

### `moth list` - List Tests

```bash
moth list [OPTIONS] <SPEC>

Options:
  -d, --detailed    Show detailed test information
  -v, --verbose     Enable verbose output
      --output <FORMAT>  Output format [json, text]
```

## 🚦 Common Patterns

### 1. Basic Tool Testing

```yaml
tools:
  - name: "my_tool"
    tests:
      - name: "success_case"
        input:
          param1: "value1"
        expected:
          error: false
          fields:
            - path: "$.result.data"
              field_type: "string"
              required: true
```

### 2. Error Condition Testing

```yaml
tools:
  - name: "my_tool"
    tests:
      - name: "error_case"
        input:
          invalid_param: null
        expected:
          error: true
          fields:
            - path: "$.error.code"
              field_type: "integer"
              required: true
```

### 3. Performance Testing

```yaml
tools:
  - name: "my_tool"
    tests:
      - name: "performance_test"
        input:
          large_dataset: "..."
        expected:
          error: false
        performance:
          max_duration_ms: 1000
          max_memory_mb: 100
```

## 🐛 Troubleshooting

### Issue: "Server connection error"

**Problem**: The test harness can't connect to your MCP server.

**Solutions**:
1. Verify your server command is correct and executable
2. Check that the server implements MCP protocol properly
3. Increase `startup_timeout_seconds`
4. Check server logs with `--verbose` flag

### Issue: "YAML parsing error"

**Problem**: Your test specification has syntax errors.

**Solutions**:
1. Validate YAML syntax with `moth validate spec.yaml`
2. Check required fields: `name`, `version`, `capabilities`, `server`
3. Ensure proper indentation (use spaces, not tabs)

### Issue: "Method not found"

**Problem**: Your server doesn't implement the requested tool/resource.

**Solutions**:
1. Use `moth list` to see what tests will be executed
2. Verify your server actually implements the MCP method
3. Check capability declarations match what your server supports

## 🎯 Next Steps

1. **[YAML Specification Guide](../configuration-reference.md)** - Learn the complete YAML specification format
2. **[Advanced Testing](../user-guide.md)** - Resource testing, prompt testing, and complex scenarios
3. **[Performance Testing](../performance-tuning.md)** - Optimize test execution and server performance
4. **[Production Deployment](../production-deployment)** - Enterprise deployment and CI/CD integration
5. **[Troubleshooting Guide](../troubleshooting.md)** - Solutions for common issues

## 💡 Pro Tips

- **Start simple**: Begin with basic tool tests before adding complex validation
- **Use filters**: Test specific functionality with `--filter "pattern"`
- **Incremental development**: Add tests as you develop new MCP server features
- **Performance monitoring**: Include performance requirements in your tests
- **CI/CD ready**: Use JSON/JUnit output formats for automated testing

---

**Need help?** Check out our [troubleshooting guide](../troubleshooting.md) or [open an issue](https://github.com/rustic-ai/codeprism/issues) on GitHub. 