---
title: User Guide
description: Complete guide to using the MCP Test Harness for testing Model Context Protocol servers
sidebar_position: 3
---

# User Guide

Complete guide to using the MCP Test Harness for testing Model Context Protocol (MCP) servers.

## 📋 Overview 

The MCP Test Harness (`moth`) is a comprehensive tool for validating MCP server implementations. It provides:
- **Protocol Compliance Testing** - Validate adherence to MCP specification
- **Performance Monitoring** - Track response times and resource usage
- **Security Validation** - Test authentication, authorization, and data protection
- **Custom Testing** - Write your own validation logic
- **CI/CD Integration** - Automated testing in your development pipeline

> **New to the Test Harness?** Start with the [Quick Start Guide](getting-started/quick-start) for a 5-minute tutorial, or check [Installation](getting-started/installation) for setup instructions.

## 🔧 Command Overview

The `moth` binary provides several commands for testing MCP servers:

- **`moth test`** - Execute test specifications against servers
- **`moth validate`** - Validate YAML specification syntax
- **`moth list`** - List available tests without running them  
- **`moth version`** - Show version information

**Basic Usage:**
```bash
# Run tests
moth test my-server.yaml

# Validate configuration
moth validate my-server.yaml --verbose

# List available tests
moth list my-server.yaml --detailed
```

> **Complete Command Reference:** See [CLI Reference](cli-reference) for detailed command documentation, options, and examples.

## 🎯 Common Usage Patterns

### Development Workflow

**1. Validate Configuration First**
```bash
# Always validate before running tests
moth validate my-server.yaml --verbose

# Check what tests will run
moth list my-server.yaml --detailed
```

**2. Iterative Development**
```bash
# Run specific tests during development
moth test my-server.yaml --filter "authentication.*" --fail-fast

# Use text output for immediate feedback
moth test my-server.yaml --output text --verbose

# Test single functionality quickly
moth test my-server.yaml --filter "specific_tool" --concurrency 1
```

**3. Debug Test Failures**
```bash
# Stop on first failure for debugging
moth test my-server.yaml --fail-fast --verbose

# Run problematic test in isolation
moth test my-server.yaml --filter "failing_test_name"
```

### CI/CD Integration

**1. Automated Testing Pipeline**
```bash
# Validate all specs
moth validate test-specs/

# Run tests with CI-friendly output
moth test my-server.yaml --output junit --output-file results.xml

# Generate HTML reports for artifacts
moth test my-server.yaml --output html --output-file report.html
```

**2. Performance Monitoring**
```bash
# Performance regression testing
moth test performance-spec.yaml --concurrency 8

# Compare against performance baselines
moth test my-server.yaml --filter "performance.*"
```

## 📊 Understanding Test Results

### Test Output Format

```
🧪 Running Test Suite: core-protocol-tests
================================================================================

✅ PASS: initialize
   Duration: 150ms
   Server Response: {"protocolVersion": "2025-06-18", "capabilities": {...}}

❌ FAIL: list_resources  
   Duration: 5000ms (TIMEOUT)
   Error: Server did not respond within timeout period
   Expected: Response with resource list
   Actual: No response received

⚠️  SKIP: optional_feature
   Reason: Server does not advertise this capability

================================================================================
Suite Summary: core-protocol-tests
- Total Tests: 3
- Passed: 1
- Failed: 1  
- Skipped: 1
- Success Rate: 33.3%
- Total Duration: 5.15s
================================================================================
```

### Result Status Codes

- **✅ PASS** - Test completed successfully, all validations passed
- **❌ FAIL** - Test failed validation or encountered an error
- **⚠️ SKIP** - Test was skipped (disabled, unsupported capability, etc.)
- **🔄 RETRY** - Test is being retried after failure
- **⏱️ TIMEOUT** - Test exceeded maximum execution time

### Performance Metrics

```
Performance Summary:
- Average Response Time: 245ms
- 95th Percentile: 800ms
- 99th Percentile: 1.2s
- Fastest Test: initialize (98ms)
- Slowest Test: complex_analysis (1.8s)
- Memory Usage: Peak 156MB, Average 89MB
- Regression Status: ✅ No performance regressions detected
```

## ⚙️ YAML Configuration Essentials

> **Complete Configuration Reference:** See [Configuration Reference](configuration-reference) for detailed documentation of all YAML options and advanced features.

### Basic Server Configuration

```yaml
# Minimal configuration for stdio server
name: "My MCP Server Test"
version: "1.0.0"

capabilities:
  tools: true
  resources: false

server:
  command: "cargo run --bin my-mcp-server"
  args: ["stdio"]
  transport: "stdio"
  startup_timeout_seconds: 30

tools:
  - name: "echo"
    description: "Test echo functionality"
    tests:
      - name: "basic_echo"
        input:
          message: "Hello, World!"
        expected:
          error: false
          fields:
            - path: "$.result"
              field_type: "string"
              required: true
```

### HTTP Server Configuration

```yaml
server:
  transport: "http"
  url: "http://localhost:3000"
  connection_timeout: 10
  headers:
    Authorization: "Bearer ${API_TOKEN}"
    Content-Type: "application/json"
```

## 🚀 Advanced Testing Scenarios

### Custom Validation Scripts

Use Python scripts for complex validation logic:

```yaml
tools:
  - name: "complex_analysis"
    tests:
      - name: "custom_validation"
        input:
          project_path: "test-project"
        expected:
          error: false
          custom_validation:
            script: "scripts/validate_analysis.py"
            timeout_seconds: 30
```

**Custom validation script example:**
```python
#!/usr/bin/env python3
# scripts/validate_analysis.py

import json
import sys

def validate_response(response_data):
    """Custom validation logic"""
    result = response_data.get('result', {})
    
    # Custom business logic validation
    if result.get('total_files', 0) < 1:
        return False, "No files analyzed"
    
    if 'languages' not in result:
        return False, "Missing language analysis"
    
    return True, "Validation passed"

if __name__ == "__main__":
    response = json.load(sys.stdin)
    success, message = validate_response(response)
    
    if success:
        print(f"✅ {message}")
        sys.exit(0)
    else:
        print(f"❌ {message}")
        sys.exit(1)
```

## 📈 Best Practices

### 1. Test Organization
- **Group related tests** in logical tool/resource sections
- **Use descriptive test names** that explain the scenario
- **Include both success and error test cases**
- **Test edge cases and boundary conditions**

### 2. Configuration Management
- **Start with simple configurations** and add complexity gradually
- **Use environment variables** for sensitive data
- **Validate configurations** before running tests
- **Version your test specifications** alongside your server code

### 3. Performance Considerations
- **Set appropriate timeouts** based on expected response times
- **Use concurrency judiciously** - don't overwhelm your server
- **Monitor resource usage** during test execution
- **Include performance requirements** in critical tests

### 4. Debugging and Troubleshooting
- **Use `--verbose` flag** for detailed output during debugging
- **Run single tests** with `--filter` to isolate issues
- **Check server logs** when tests fail unexpectedly
- **Validate your YAML** before running tests

## 🔗 Next Steps

- **[Configuration Reference](configuration-reference)** - Complete YAML specification documentation
- **[CLI Reference](cli-reference)** - Detailed command-line options and examples
- **[Performance Tuning](performance-tuning)** - Optimize test execution and server performance
- **[Production Deployment](production-deployment)** - Enterprise deployment and CI/CD integration
- **[Troubleshooting Guide](troubleshooting)** - Solutions for common issues and debugging techniques

---

**Need Help?** Check the [Troubleshooting Guide](troubleshooting) or [open an issue](https://github.com/rustic-ai/codeprism/issues) for assistance. 