---
title: Examples
description: Real-world test specification examples for the MCP Test Harness
sidebar_position: 9
---

# Test Specification Examples

Real-world examples of test specifications for different types of MCP servers.

## Available Examples

### [CodePrism MCP Server](codeprism-mcp.yaml)
Comprehensive test specification for the CodePrism MCP server, demonstrating:
- **Tool testing** - repository stats, complexity analysis, symbol search
- **Performance requirements** - response time and memory constraints
- **Error handling** - validation patterns and expected behaviors
- **Configuration** - server setup and transport configuration

```yaml
name: "CodePrism MCP Server"
version: "1.0.0"
description: "Test specification for CodePrism MCP server capabilities"

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
  startup_timeout_seconds: 45
```

## Usage Examples

### Basic Testing
```bash
# Test the CodePrism server
moth test codeprism-mcp.yaml

# Generate HTML report
moth test codeprism-mcp.yaml --output html --output-file report.html

# Run specific tests only
moth test codeprism-mcp.yaml --filter "repository_stats"
```

### Validation and Listing
```bash
# Validate the specification
moth validate codeprism-mcp.yaml

# List all tests that would run
moth list codeprism-mcp.yaml --detailed
```

## Creating Your Own Examples

When creating test specifications for your MCP server:

1. **Start Simple** - Begin with basic tool tests
2. **Add Validation** - Include response validation patterns
3. **Set Performance Requirements** - Define acceptable response times
4. **Handle Errors** - Test error conditions and edge cases
5. **Document Thoroughly** - Include clear descriptions

### Minimal Example Template
```yaml
name: "My MCP Server"
version: "1.0.0"
description: "Basic test specification"

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
  - name: "my_tool"
    description: "Test my tool functionality"
    tests:
      - name: "basic_test"
        description: "Basic functionality test"
        input:
          param1: "value1"
        expected:
          error: false
          fields:
            - path: "$.result"
              field_type: "object"
              required: true

test_config:
  timeout_seconds: 60
  max_concurrency: 2
  fail_fast: false
```

## Best Practices

### Specification Organization
- **Use descriptive names** for tests and tools
- **Group related tests** logically
- **Include performance requirements** where appropriate
- **Document expected behaviors** clearly

### Testing Strategy  
- **Test success cases** first
- **Add error condition tests** for robustness
- **Include edge cases** for completeness
- **Set realistic timeouts** based on actual performance

### Maintenance
- **Keep examples updated** with current API
- **Test specifications regularly** against actual servers
- **Document changes** and versioning
- **Share examples** with the community

---

**Need help creating test specifications?** Check out our [Configuration Reference](../configuration-reference) for complete documentation of all available options. 