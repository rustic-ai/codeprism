---
title: CLI Reference
description: Complete command-line reference for the moth binary
sidebar_position: 3
---

# CLI Reference - Mandrel MCP Test Harness

Complete command-line reference for the **moth** binary (MOdel context protocol Test Harness).

## Overview

```bash
moth [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

### Global Options

```bash
-v, --verbose          Enable verbose output
    --output <FORMAT>  Output format [default: json] [possible values: json, html, junit, text]
-h, --help             Print help
-V, --version          Print version
```

## Commands

### `moth test` - Run Test Specifications

Execute test specifications against MCP servers.

```bash
moth test [OPTIONS] <SPEC>
```

#### Arguments

- `<SPEC>` - Path to test specification file or directory

#### Options

```bash
-o, --output-file <OUTPUT_FILE>  Output file for test results
-f, --fail-fast                  Stop execution on first failure
-F, --filter <FILTER>            Test filter pattern (regex)
-c, --concurrency <CONCURRENCY>  Maximum number of concurrent tests [default: 4]
-v, --verbose                    Enable verbose output
    --output <OUTPUT>            Output format [default: json] [possible values: json, html, junit, text]
-h, --help                       Print help
```

#### Examples

```bash
# Run all tests from a specification file
moth test my-server.yaml

# Run tests with JSON output to file
moth test my-server.yaml --output json --output-file results.json

# Run tests with HTML report generation
moth test my-server.yaml --output html --output-file report.html

# Stop on first failure for quick debugging
moth test my-server.yaml --fail-fast

# Run only tests matching a pattern
moth test my-server.yaml --filter "authentication.*"

# Increase concurrency for faster execution
moth test my-server.yaml --concurrency 8

# Verbose output for debugging
moth test my-server.yaml --verbose

# Test all YAML files in a directory
moth test test-specs/
```

#### Output Formats

##### JSON (Default)
```json
{
  "suite_name": "My MCP Server",
  "total_tests": 5,
  "passed": 4,
  "failed": 1,
  "skipped": 0,
  "duration_ms": 2847,
  "test_results": [
    {
      "name": "test_tool_basic",
      "status": "passed",
      "duration_ms": 156,
      "performance": {
        "response_time_ms": 23,
        "memory_usage_bytes": null
      }
    }
  ]
}
```

##### Text/Human-Readable
```
🧪 Starting test execution: My MCP Server v1.0.0
🔗 Connecting to MCP server...
✅ Server connected successfully
🏃 Running 5 test cases...

Tool Test: authenticate::basic_auth
  ✅ Test passed in 156ms
  📊 Response validation: All fields matched

📋 Test Summary:
  Total: 5 tests
  Passed: 4 ✅
  Failed: 1 ❌
  Duration: 2.8s
```

##### JUnit XML
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="My MCP Server" tests="5" failures="1" time="2.847">
  <testcase name="authenticate::basic_auth" time="0.156"/>
  <testcase name="authenticate::invalid_token" time="0.089">
    <failure message="Validation failed">Expected error response</failure>
  </testcase>
</testsuite>
```

##### HTML Report
Generates interactive HTML report with:
- Test execution summary
- Individual test results with details
- Performance charts
- Server communication logs
- Filterable and searchable results

### `moth validate` - Validate Test Specifications

Validate the syntax and structure of test specification files.

```bash
moth validate [OPTIONS] <SPEC>
```

#### Arguments

- `<SPEC>` - Path to test specification file or directory

#### Options

```bash
-v, --verbose          Enable verbose output
    --output <OUTPUT>  Output format [default: json] [possible values: json, text]
-h, --help             Print help
```

#### Examples

```bash
# Validate a single specification file
moth validate my-server.yaml

# Validate all specifications in a directory
moth validate test-specs/

# Verbose validation with detailed feedback
moth validate my-server.yaml --verbose

# JSON output for programmatic parsing
moth validate my-server.yaml --output json
```

#### Output Examples

##### Success (Text)
```
✅ Specification validation successful
  Name: My MCP Server
  Version: 1.0.0
  Description: Test specification for my server
  Tools: 3, Resources: 1, Prompts: 0
  Server: node server.js
```

##### Error (Text)
```
❌ Specification validation failed: YAML parsing error: 
server: missing field `command` at line 23 column 3
```

##### JSON Output
```json
{
  "valid": true,
  "specification": {
    "name": "My MCP Server",
    "version": "1.0.0",
    "description": "Test specification for my server",
    "tools": 3,
    "resources": 1,
    "prompts": 0,
    "server_command": "node server.js"
  }
}
```

### `moth list` - List Available Tests

List all tests defined in a specification without executing them.

```bash
moth list [OPTIONS] <SPEC>
```

#### Arguments

- `<SPEC>` - Path to test specification file or directory

#### Options

```bash
-d, --detailed         Show detailed test information
-v, --verbose          Enable verbose output
    --output <OUTPUT>  Output format [default: text] [possible values: json, text]
-h, --help             Print help
```

#### Examples

```bash
# List tests in a specification
moth list my-server.yaml

# Show detailed test information
moth list my-server.yaml --detailed

# JSON output for programmatic parsing
moth list my-server.yaml --output json

# List tests from all specs in directory
moth list test-specs/ --detailed
```

#### Output Examples

##### Basic Listing
```
📋 Test Specification: My MCP Server
   Tools: 3 tests
   Resources: 1 test
   Total Tests: 4
```

##### Detailed Listing
```
📋 Test Specification: My MCP Server
   Version: 1.0.0
   Description: Test specification for my server

🔧 Tools (3):
  • authenticate - User authentication tool
    ├─ basic_auth
    │  Test basic authentication flow
    ├─ invalid_token
    │  Test invalid token handling

  • get_user - User information retrieval
    ├─ valid_user
    │  Get valid user information

📦 Resources (1):
  • user_profile - User profile resource
    ├─ get_profile
    │  Retrieve user profile data

📊 Total Tests: 4
```

##### JSON Output
```json
{
  "specification": {
    "name": "My MCP Server",
    "version": "1.0.0",
    "description": "Test specification for my server"
  },
  "total_tests": 4,
  "tools": [
    {
      "name": "authenticate",
      "description": "User authentication tool",
      "tests": [
        {
          "name": "basic_auth",
          "description": "Test basic authentication flow"
        },
        {
          "name": "invalid_token",
          "description": "Test invalid token handling"
        }
      ]
    }
  ],
  "resources": [
    {
      "name": "user_profile",
      "description": "User profile resource",
      "tests": [
        {
          "name": "get_profile",
          "description": "Retrieve user profile data"
        }
      ]
    }
  ]
}
```

### `moth version` - Show Version Information

Display version and build information for the moth binary.

```bash
moth version
```

#### Output

```
2025-07-04T17:49:39.940047Z  INFO Starting moth binary - Mandrel MCP Test Harness
moth 0.1.0 - Mandrel MCP Test Harness
MOdel context protocol Test Harness binary
Built with official rmcp SDK
Repository: https://github.com/rustic-ai/codeprism
```

## Common Usage Patterns

### 1. Development Workflow

```bash
# Validate specification during development
moth validate my-server.yaml

# List tests to verify configuration
moth list my-server.yaml --detailed

# Run tests with immediate feedback
moth test my-server.yaml --output text --verbose

# Debug specific test failures
moth test my-server.yaml --filter "problematic_test" --fail-fast --verbose
```

### 2. CI/CD Integration

```bash
# Run all tests with JUnit output for CI
moth test my-server.yaml --output junit --output-file test-results.xml

# Generate HTML report for artifact collection
moth test my-server.yaml --output html --output-file test-report.html

# Fail fast for quick feedback in CI
moth test my-server.yaml --fail-fast
```

### 3. Performance Testing

```bash
# High concurrency for performance testing
moth test performance-spec.yaml --concurrency 16

# Filter to performance-specific tests
moth test my-server.yaml --filter "performance.*"
```

### 4. Test Organization

```bash
# Test specific functionality
moth test auth-tests.yaml
moth test tool-tests.yaml
moth test resource-tests.yaml

# Test entire test suite directory
moth test test-specs/

# Validate all specifications before committing
moth validate test-specs/
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Success - All tests passed or validation successful |
| 1    | Test failures or validation errors |
| 2    | Configuration or argument errors |
| 3    | Server connection failures |
| 4    | File I/O errors |

## Environment Variables

### `RUST_LOG`
Controls logging verbosity for debugging:

```bash
# Enable debug logging
RUST_LOG=debug moth test my-server.yaml

# Enable trace logging for maximum detail
RUST_LOG=trace moth test my-server.yaml

# Filter logs to specific modules
RUST_LOG=mandrel_mcp_th=debug moth test my-server.yaml
```

### `MOTH_CONFIG_DIR`
Override default configuration directory:

```bash
export MOTH_CONFIG_DIR=/custom/config/path
moth test my-server.yaml
```

## Error Handling

### Common Error Patterns

#### Connection Errors
```
❌ Server connection error: Failed to start server process: No such file or directory
```
**Solution**: Verify server command path and permissions

#### Validation Errors
```
❌ Specification validation failed: YAML parsing error: server: missing field `command`
```
**Solution**: Check YAML syntax and required fields

#### Test Execution Errors
```
❌ Test execution error: Timeout waiting for server response
```
**Solution**: Increase timeout values or check server performance

### Debugging Commands

```bash
# Maximum verbosity for debugging
moth test my-server.yaml --verbose --output text

# Validate specification first
moth validate my-server.yaml --verbose

# Check what tests would run
moth list my-server.yaml --detailed

# Test single functionality
moth test my-server.yaml --filter "specific_test" --fail-fast
```

## Best Practices

### 1. Specification Organization
- Use descriptive test names
- Group related tests in logical tool/resource sections
- Include performance requirements where appropriate
- Validate specifications before committing

### 2. Test Execution
- Use `--fail-fast` during development for quick feedback
- Use appropriate concurrency levels for your hardware
- Generate HTML reports for comprehensive analysis
- Filter tests for focused debugging

### 3. CI/CD Integration
- Always validate specifications in CI
- Use JUnit output for integration with CI systems
- Set appropriate timeouts for CI environments
- Archive HTML reports as build artifacts

### 4. Performance Monitoring
- Include performance requirements in tests
- Monitor test execution times over time
- Use high concurrency for stress testing
- Regular baseline performance validation

---

**See Also:**
- [Quick Start Guide](getting-started/quick-start.md) - Get started quickly
- [Configuration Reference](configuration-reference.md) - Complete YAML specification
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions 