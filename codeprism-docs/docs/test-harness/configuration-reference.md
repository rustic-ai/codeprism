---
title: Configuration Reference
description: Complete reference for MCP Test Harness configuration files
sidebar_position: 5
---

# Configuration Reference

Complete reference for MCP Test Harness configuration files with detailed explanations of all options and validation rules.

## 📋 Overview

The MCP Test Harness uses YAML configuration files to define test suites, server settings, validation patterns, and execution parameters. This reference provides comprehensive documentation for all configuration options based on the actual implementation and working examples.

## 🏗️ Configuration File Structure

Based on verified working examples, here's the actual configuration structure:

```yaml
# Top-level test specification structure
name: "Server Name"                    # Human-readable name
version: "1.0.0"                      # Specification version
description: "Description of what this tests"  # Brief description

# Server capabilities
capabilities:
  tools: true                         # Tools capability support
  resources: true                     # Resources capability support  
  prompts: false                      # Prompts capability support
  sampling: false                     # LLM sampling capability support
  logging: true                       # Logging capability support
  experimental:                       # Extended experimental features (optional)
    progress_notifications: true
    resource_subscriptions: true

# Server startup configuration
server:
  command: "command"                  # Command to start server
  args: ["arg1", "arg2"]             # Command line arguments
  env:                               # Environment variables
    KEY: "value"
  transport: "stdio"                 # Transport type
  startup_timeout_seconds: 30        # Startup timeout
  shutdown_timeout_seconds: 15       # Shutdown timeout

# Tool testing definitions
tools:
  - name: "tool_name"               # Tool name
    description: "Tool description"  # Tool description
    tests:                          # Array of test cases
      - name: "test_case_name"      # Test case name
        description: "Test description"
        input:                      # Input parameters
          param1: "value1"
        expected:                   # Expected results
          error: false
          fields:
            - path: "$.result"      # JSONPath to validate
              value: "expected"     # Expected value
              required: true        # Required field
        performance:                # Performance requirements (optional)
          max_duration_ms: 1000
        tags: ["category", "type"]  # Test tags

# Resource testing definitions (optional)
resources:
  - uri_template: "scheme://path/{id}"  # Resource URI template
    name: "Resource Name"              # Resource name
    description: "Resource description" # Resource description
    tests:                            # Array of test cases
      - name: "test_case_name"
        # ... (similar structure to tools)

# Test execution configuration
test_config:
  timeout_seconds: 60               # Test timeout
  max_concurrency: 4                # Max parallel tests
  fail_fast: false                  # Stop on first failure
  retry:                            # Retry configuration
    max_retries: 2                  # Maximum retry attempts
    retry_delay_ms: 1000            # Initial retry delay
    exponential_backoff: true       # Use exponential backoff

# Metadata and documentation
metadata:
  author: "Author Name"
  documentation: "https://example.com/docs"
  license: "MIT"
  tags: ["category1", "category2"]
  quality_targets:                  # Quality targets (optional)
    success_rate_percent: 95
    error_handling_coverage_percent: 100
```

## 📝 Core Configuration Sections

### Test Specification Metadata

```yaml
name: "Everything MCP Server (Working Tests)"
version: "2025.7.1"
description: "Only tests that are proven to work with the everything server"
```

**Required Fields:**
- `name`: Human-readable name for the test specification
- `version`: Semantic version of the test specification
- `description`: Brief description of what this specification tests

### Capabilities Declaration

```yaml
capabilities:
  tools: true           # Server supports tools capability
  resources: true       # Server supports resources capability  
  prompts: false        # Server does NOT support prompts
  sampling: false       # Server does NOT support LLM sampling
  logging: true         # Server supports logging capability
  experimental:         # Extended experimental features
    progress_notifications: true
    resource_subscriptions: true
    annotation_system: true
    multi_transport: true
```

**Capability Types:**
- `tools`: Core MCP tools functionality
- `resources`: Resource access and management
- `prompts`: Prompt templates and parameterization
- `sampling`: LLM integration and sampling
- `logging`: Server logging and monitoring
- `experimental`: Extended features beyond core MCP specification

### Server Configuration

```yaml
server:
  command: "npx" 
  args: ["-y", "@modelcontextprotocol/server-everything"]
  env:
    NODE_ENV: "test"
    LOG_LEVEL: "info"
    RUST_LOG: "debug"
  transport: "stdio"
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 15
```

**Server Fields:**
- `command`: Executable command to start the server
- `args`: Array of command-line arguments
- `env`: Environment variables (key-value pairs)
- `transport`: Transport protocol ("stdio", "http", "websocket")
- `startup_timeout_seconds`: Time to wait for server startup (5-300)
- `shutdown_timeout_seconds`: Time to wait for graceful shutdown (1-60)

### Tool Testing Configuration

```yaml
tools:
  - name: "add"
    description: "Mathematical addition tool"
    tests:
      - name: "integer_addition"
        description: "Test integer addition"
        input:
          a: 42
          b: 58
        expected:
          error: false
          fields:
            - path: "$[0].text"
              contains: "100"
              required: true
        performance:
          max_duration_ms: 50
        tags: ["math", "basic"]
        
      - name: "error_test"
        description: "Test error handling"
        input:
          a: "invalid"
          b: 5
        expected:
          error: true
          error_code: -32602
          error_message_contains: "invalid"
        tags: ["math", "error"]
```

**Tool Test Fields:**
- `name`: Unique test case name
- `description`: Human-readable test description
- `input`: Object containing tool input parameters
- `expected`: Expected results validation
- `performance`: Performance requirements (optional)
- `tags`: Array of categorization tags

### Expected Results Validation

```yaml
expected:
  error: false                      # Whether an error is expected
  error_code: -32603               # Expected MCP error code (for errors)
  error_message_contains: "text"   # Text that should appear in error message
  fields:                          # Array of field validations
    - path: "$[0].text"            # JSONPath to the field
      value: "exact_match"         # Exact value match
      contains: "substring"        # Substring match
      field_type: "string"         # Type validation
      min_length: 5               # Minimum string length
      max_length: 100             # Maximum string length
      required: true              # Whether field is required
  security_constraints:           # Security validation (optional)
    - constraint_type: "no_passwords"
      enabled: true
```

**Validation Types:**
- **Exact Match**: `value: "expected_value"`
- **Substring Match**: `contains: "substring"`
- **Type Validation**: `field_type: "string|number|boolean|object|array"`
- **Range Validation**: `min_length`, `max_length` for strings
- **Required Fields**: `required: true|false`

### Resource Testing Configuration

```yaml
resources:
  - uri_template: "test://static/resource/{id}"
    name: "Test Resources"
    description: "Test resource access"
    tests:
      - name: "valid_resource_reference"
        description: "Test valid resource ID"
        input:
          resourceId: 25
        expected:
          error: false
          fields:
            - path: "$.content"
              field_type: "string"
              required: true
        tags: ["resources", "valid"]
        
      - name: "invalid_resource_id"
        description: "Test invalid resource ID"
        input:
          resourceId: 0
        expected:
          error: true
          error_code: -32603
          error_message_contains: "Number must be greater than or equal to 1"
        tags: ["resources", "error"]
```

### Test Execution Configuration

```yaml
test_config:
  timeout_seconds: 60               # Default test timeout
  max_concurrency: 4                # Maximum parallel test execution
  fail_fast: false                  # Stop on first failure
  retry:                            # Retry configuration
    max_retries: 2                  # Maximum retry attempts
    retry_delay_ms: 1000            # Initial retry delay
    exponential_backoff: true       # Use exponential backoff
  
  # Progress notification testing (optional)
  progress_notifications:
    enabled: true
    timeout_seconds: 30
    min_updates_expected: 2
  
  # Logging testing (optional)
  logging_tests:
    enabled: true
    log_collection_duration_seconds: 20
    expected_log_levels: ["info", "warn", "error", "debug"]
    min_log_messages_expected: 1
```

### Performance Requirements

```yaml
performance:
  max_duration_ms: 1000            # Maximum test execution time
  expect_progress_notifications: true  # Expect progress updates
  max_memory_usage_mb: 100         # Memory usage limit (optional)
  min_throughput_per_second: 10    # Minimum throughput (optional)
```

### Metadata Section

```yaml
metadata:
  author: "MCP Test Harness Team"
  documentation: "https://spec.modelcontextprotocol.io/"
  license: "MIT"
  tags: ["verified-working", "server-reality", "corrected"]
  quality_targets:
    success_rate_percent: 100
    error_handling_coverage_percent: 100
```

## 🎯 Working Examples

### Filesystem Server Example

```yaml
name: "Filesystem MCP Server (MCP-Compliant)"
version: "1.0.0"
description: "Testing @modelcontextprotocol/server-filesystem according to MCP specification"

capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false

server:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp/mcp-test-sandbox"]
  env:
    NODE_ENV: "test"
  transport: "stdio"
  startup_timeout_seconds: 30

tools:
  - name: "write_file"
    description: "Write content to a file"
    tests:
      - name: "write_simple_text_file"
        description: "Successfully write a text file"
        input:
          path: "/tmp/mcp-test-sandbox/test_write.txt"
          content: "Hello, MCP Test Harness!"
        expected:
          error: false
          fields:
            - path: "$.content[0].text"
              contains: "successfully"
              required: true
        tags: ["write", "success", "basic"]
```

### Everything Server Example

```yaml
name: "Everything MCP Server (Working Tests)"
version: "2025.7.1"
description: "Only tests that are proven to work with the everything server"

capabilities:
  tools: true
  resources: true
  prompts: false        # Not supported by server
  sampling: false       # Not supported by server
  logging: true

server:
  command: "npx" 
  args: ["-y", "@modelcontextprotocol/server-everything"]
  env:
    NODE_ENV: "test"
    LOG_LEVEL: "info"
  transport: "stdio"
  startup_timeout_seconds: 30

tools:
  - name: "add"
    description: "Mathematical addition tool"
    tests:
      - name: "integer_addition"
        description: "Test integer addition"
        input:
          a: 42
          b: 58
        expected:
          error: false
          fields:
            - path: "$[0].text"
              contains: "100"
              required: true
        performance:
          max_duration_ms: 50
        tags: ["math", "basic"]
```

## 🔍 Validation Rules

### Required Fields

**Top Level:**
- `name` (string)
- `version` (string)
- `description` (string)
- `capabilities` (object)
- `server` (object)

**Server Section:**
- `command` (string)
- `transport` (string)

**Tool/Resource Tests:**
- `name` (string)
- `description` (string)
- `input` (object)
- `expected` (object)

### Field Constraints

**Timeouts:**
- `startup_timeout_seconds`: 5-300
- `shutdown_timeout_seconds`: 1-60
- `timeout_seconds`: 1-3600

**Concurrency:**
- `max_concurrency`: 1-32

**Performance:**
- `max_duration_ms`: 1-3600000 (1ms to 1 hour)

### JSONPath Expressions

Valid JSONPath expressions for field validation:

```yaml
fields:
  - path: "$.result"              # Root level field
  - path: "$[0].text"            # First array element's text field
  - path: "$.content[0].type"    # Nested array access
  - path: "$.messages[*].role"   # All array elements
  - path: "$.errors.length()"    # Array length function
```

## 🚨 Common Validation Errors

### Server Configuration Errors

```yaml
# ❌ WRONG - Missing required command
server:
  args: ["stdio"]
  transport: "stdio"

# ✅ CORRECT
server:
  command: "my-server"
  args: ["stdio"]
  transport: "stdio"
```

### Expected Results Errors

```yaml
# ❌ WRONG - Invalid JSONPath
expected:
  fields:
    - path: "result"              # Missing $ prefix

# ✅ CORRECT
expected:
  fields:
    - path: "$.result"            # Valid JSONPath
```

### Capability Mismatch Errors

```yaml
# ❌ WRONG - Claiming unsupported capabilities
capabilities:
  sampling: true                  # Server doesn't support sampling

# ✅ CORRECT - Accurate capability declaration
capabilities:
  sampling: false                 # Correctly marked as unsupported
```

## 🎨 Best Practices

### Test Organization

1. **Use Descriptive Names**: Make test names clearly indicate what they test
2. **Group Related Tests**: Organize tests by functionality or feature area
3. **Include Error Cases**: Test both success and failure scenarios
4. **Set Realistic Timeouts**: Base timeouts on actual server performance

### Configuration Management

1. **Document Capabilities Accurately**: Only claim capabilities the server actually supports
2. **Use Environment Variables**: Leverage `env` section for configuration
3. **Include Performance Requirements**: Set appropriate `max_duration_ms` values
4. **Tag Tests Appropriately**: Use tags for filtering and organization

### Validation Patterns

1. **Use Specific JSONPaths**: Target exact fields rather than broad patterns
2. **Combine Validation Types**: Use both exact matches and contains checks
3. **Test Required vs Optional Fields**: Mark fields appropriately
4. **Include Error Code Validation**: Validate specific MCP error codes

### Version Management

1. **Semantic Versioning**: Use semantic versioning for specifications
2. **Update Protocol Versions**: Keep MCP protocol versions current
3. **Document Changes**: Include change descriptions in version updates
4. **Maintain Backward Compatibility**: When possible, maintain compatibility

---

**Need examples?** Check out our [Working Examples](examples/) section with verified test specifications for real MCP servers. 