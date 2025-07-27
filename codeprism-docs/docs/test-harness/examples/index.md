---
title: Examples
description: Real-world test specification examples for the MCP Test Harness
sidebar_position: 9
---

# Test Specification Examples

Real-world examples of test specifications for different types of MCP servers. All examples have been verified to work with the actual implementation.

## Available Examples

### [Filesystem MCP Server](filesystem-server.yaml)
**Status: ✅ 100% Verified Working**

Comprehensive test specification for the `@modelcontextprotocol/server-filesystem`, demonstrating:
- **File operations** - create, read, write, move, delete operations
- **Directory management** - creation and listing operations
- **Error handling** - proper error responses for invalid operations
- **MCP compliance** - follows MCP 2025-06-18 specification format
- **Performance validation** - response time requirements

```yaml
name: "Filesystem MCP Server (MCP-Compliant)"
version: "1.0.0"
description: "Testing @modelcontextprotocol/server-filesystem according to MCP specification"

capabilities:
  tools: true           # Filesystem operations work
  resources: false      # Resources not used
  prompts: false        # Not supported
  sampling: false       # Not supported
  logging: false        # Not enabled

server:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp/mcp-test-sandbox"]
  transport: "stdio"
  startup_timeout_seconds: 30
```

### [Everything MCP Server](everything-server.yaml)
**Status: ✅ 100% Verified Working (8/8 tests passing)**

Optimized test specification for the `@modelcontextprotocol/server-everything`, featuring:
- **Mathematical operations** - addition with various number types
- **Text processing** - echo functionality with Unicode support
- **Environment access** - system environment variable debugging
- **Long-running operations** - progress notification testing
- **Resource management** - basic resource access and validation
- **Server-reality alignment** - only tests features that actually work

```yaml
name: "Everything MCP Server (Working Tests)"
version: "2025.7.1"
description: "Only tests that are proven to work with the everything server"

capabilities:
  tools: true           # These specific tools work
  resources: true       # Basic resource access works
  prompts: false        # Not supported
  sampling: false       # Not supported - returns MCP error -32601
  logging: true         # Works

server:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-everything"]
  transport: "stdio"
```

### [CodePrism MCP Server](codeprism-mcp.yaml)
Basic test specification for the CodePrism MCP server, demonstrating:
- **Tool testing** - repository stats, complexity analysis, symbol search
- **Performance requirements** - response time and memory constraints
- **Error handling** - validation patterns and expected behaviors
- **Configuration** - server setup and transport configuration

## Usage Examples

### Running Test Specifications

```bash
# Test the filesystem server
moth run codeprism-docs/docs/test-harness/examples/filesystem-server.yaml

# Test the everything server (verified 100% working)
moth run codeprism-docs/docs/test-harness/examples/everything-server.yaml

# Validate a specification before running
moth validate codeprism-docs/docs/test-harness/examples/filesystem-server.yaml

# Run with verbose output for debugging
moth -v run codeprism-docs/docs/test-harness/examples/everything-server.yaml
```

### Configuration Validation

```bash
# Validate specification syntax and structure
moth validate filesystem-server.yaml --detailed

# Check all validation aspects
moth validate everything-server.yaml --check-all

# Generate validation report
moth validate codeprism-mcp.yaml --formats html --output ./validation-reports
```

## Working Examples - Real Test Results

### Filesystem Server Results
```
✅ Test Suite Finished ✅
Suite: Filesystem MCP Server (MCP-Compliant)
Total Tests: 8, Passed: 8, Failed: 0
Duration: 2.3s
Success Rate: 100%
```

### Everything Server Results
```
✅ Test Suite Finished ✅
Suite: Everything MCP Server (Working Tests)
Total Tests: 8, Passed: 8, Failed: 0
Duration: 10.02s
Success Rate: 100%
```

## Creating Your Own Test Specifications

When creating test specifications for your MCP server:

1. **Start with Working Examples** - Use our verified examples as templates
2. **Test Server Reality** - Only claim capabilities your server actually supports
3. **Use Correct Tool Names** - Verify tool names match server implementation
4. **Validate Output Formats** - Check actual server response structure
5. **Set Realistic Timeouts** - Base timeouts on actual server performance
6. **Include Error Tests** - Test both success and failure scenarios

### Minimal Working Template

```yaml
name: "My MCP Server"
version: "1.0.0"
description: "Basic test specification"

# Only claim capabilities your server actually supports
capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false

server:
  command: "my-server"
  args: ["stdio"]
  transport: "stdio"
  startup_timeout_seconds: 30

tools:
  - name: "my_tool"                    # Use exact tool name from server
    description: "Test my tool functionality"
    tests:
      - name: "basic_test"
        description: "Basic functionality test"
        input:
          param1: "value1"
        expected:
          error: false
          fields:
            - path: "$[0].text"        # Use actual server response format
              field_type: "string"
              required: true

test_config:
  timeout_seconds: 60
  max_concurrency: 2
  fail_fast: false
```

## Server-Reality Best Practices

Based on our testing experience:

### ✅ Do This
- **Verify tool names** against actual server implementation
- **Test response formats** to understand actual output structure
- **Set capabilities accurately** (false for unsupported features)
- **Use realistic timeouts** based on actual performance
- **Include both success and error test cases**
- **Validate with `moth validate` before running**

### ❌ Avoid This
- **Claiming false capabilities** (e.g., `sampling: true` when unsupported)
- **Using wrong tool names** (e.g., `longOperation` vs `longRunningOperation`)
- **Expecting wrong output formats** (e.g., `$.result` vs `$[0].text`)
- **Setting unrealistic timeouts** (too short for actual server performance)
- **Only testing success cases** (missing error scenario validation)

## Specification Evolution

### Version 1.0 vs 2.0 Patterns

**Version 1.0 (Documentation):**
```yaml
capabilities:
  sampling: true          # ❌ Often incorrect
  prompts: true          # ❌ Often unsupported

expected:
  path: "$.result"       # ❌ Wrong format
  value: 8               # ❌ Wrong expectations
```

**Version 2.0 (Server Reality):**
```yaml
capabilities:
  sampling: false        # ✅ Accurate to server
  prompts: false        # ✅ Tested and verified

expected:
  fields:
    - path: "$[0].text"  # ✅ Actual server format
      contains: "100"    # ✅ Realistic validation
```

## Testing Your Examples

Before sharing test specifications:

1. **Run the specification** against the actual server
2. **Achieve 100% success rate** or document expected failures
3. **Validate with all checks** using `moth validate --check-all`
4. **Test on clean environment** to ensure reproducibility
5. **Document any setup requirements** (sandbox directories, etc.)

---

**Need help creating test specifications?** 
- Check our [Configuration Reference](../configuration-reference) for complete YAML documentation
- Review our [Working Examples](filesystem-server.yaml) for proven patterns
- Use [Quick Start Guide](../getting-started/quick-start) for step-by-step setup 