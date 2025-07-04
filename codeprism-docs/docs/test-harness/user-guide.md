---
title: User Guide
description: Complete guide to using the MCP Test Harness for testing Model Context Protocol servers
sidebar_position: 4
---

# User Guide

Complete guide to using the MCP Test Harness for testing Model Context Protocol (MCP) servers.

## 📋 Overview 

The MCP Test Harness is a powerful tool for validating MCP server implementations. It provides:
- **Protocol Compliance Testing** - Validate adherence to MCP specification
- **Performance Monitoring** - Track response times and resource usage
- **Security Validation** - Test authentication, authorization, and data protection
- **Custom Testing** - Write your own validation logic
- **CI/CD Integration** - Automated testing in your development pipeline

## 🚀 Quick Start

### Basic Usage

```bash
# Test your MCP server with default configuration
mcp-test-harness test --config my-server-config.yaml

# Run specific test suite
mcp-test-harness test --suite "core-protocol" --config my-config.yaml

# Generate reports in specific format
mcp-test-harness test --config my-config.yaml --output-format json
```

### Your First Test

1. **Create a configuration file**:
```yaml
# my-first-test.yaml
global:
  max_global_concurrency: 2
  global_timeout_seconds: 30

server:
  start_command: "node"
  args: ["my-mcp-server.js"]
  startup_timeout_seconds: 10

test_suites:
  - name: "basic_test"
    test_cases:
      - id: "initialize"
        tool_name: "initialize"
        input_params:
          protocolVersion: "2024-11-05"
        expected:
          patterns:
            - key: "protocolVersion"
              validation: { type: "exists" }
              required: true
```

2. **Run the test**:
```bash
mcp-test-harness test --config my-first-test.yaml
```

3. **Check the results**:
```
=== Test Execution Summary ===
Test Suites: 1/1 passed
Total Tests: 1/1 passed
  ✅ PASS: basic_test (1/1 tests passed)
```

## 🔧 Command Line Interface

### Main Commands

#### `test` - Run Test Suites
Primary command for executing tests against MCP servers.

```bash
mcp-test-harness test [OPTIONS]

Options:
  -c, --config <FILE>           Configuration file path [default: test-harness.yaml]
  -s, --suite <NAME>           Run specific test suite by name
      --dry-run                Show what would be executed without running tests
  -o, --output <DIR>           Output directory for test reports
  -f, --output-format <FORMAT> Output format: table, json, yaml [default: table]
  -v, --verbose                Enable verbose logging
      --fail-fast              Stop on first test failure
      --parallel <NUM>         Override global concurrency setting
      --timeout <SECONDS>      Override global timeout setting
```

**Examples:**
```bash
# Basic test execution
mcp-test-harness test --config server-config.yaml

# Run specific suite with verbose output
mcp-test-harness test --suite "security-tests" --verbose --config prod.yaml

# Dry run to validate configuration
mcp-test-harness test --dry-run --config new-config.yaml

# Generate JSON report
mcp-test-harness test --output-format json --output ./reports
```

#### `validate` - Validate Configuration
Validate test configuration files without running tests.

```bash
mcp-test-harness validate [OPTIONS]

Options:
  -c, --config <FILE>          Configuration file to validate
      --schema-only            Only validate YAML schema
      --connectivity           Test server connectivity
      --comprehensive          Full validation including test case syntax
```

**Examples:**
```bash
# Validate configuration syntax
mcp-test-harness validate --config my-config.yaml

# Test server connectivity
mcp-test-harness validate --connectivity --config server-config.yaml

# Comprehensive validation
mcp-test-harness validate --comprehensive --config production.yaml
```

#### `template` - Generate Configuration Templates
Generate configuration templates for common MCP server types.

```bash
mcp-test-harness template [OPTIONS]

Options:
      --server-type <TYPE>     Server type: filesystem, database, api, custom
      --transport <TRANSPORT>  Transport: stdio, http, websocket
  -o, --output <FILE>          Output file path
      --minimal                Generate minimal configuration
      --comprehensive          Generate comprehensive configuration with all options
```

**Examples:**
```bash
# Generate filesystem server template
mcp-test-harness template --server-type filesystem --output fs-config.yaml

# Generate HTTP server template
mcp-test-harness template --server-type api --transport http --output api-config.yaml

# Generate minimal configuration
mcp-test-harness template --minimal --output basic-config.yaml
```

#### `benchmark` - Performance Benchmarking
Run performance benchmarks and compare against baselines.

```bash
mcp-test-harness benchmark [OPTIONS]

Options:
  -c, --config <FILE>          Configuration file path
      --iterations <NUM>       Number of benchmark iterations [default: 100]
      --duration <SECONDS>     Maximum benchmark duration [default: 60]
      --warmup <SECONDS>       Warmup duration before benchmarking [default: 5]
      --establish-baseline     Establish new performance baseline
      --compare-baseline       Compare against existing baseline
      --suite <NAME>           Benchmark specific test suite
```

**Examples:**
```bash
# Run standard benchmark
mcp-test-harness benchmark --config server-config.yaml

# Establish new baseline
mcp-test-harness benchmark --establish-baseline --iterations 200

# Compare performance against baseline
mcp-test-harness benchmark --compare-baseline --suite "core-tools"
```

#### `discover` - Server Discovery
Discover running MCP servers on the network.

```bash
mcp-test-harness discover [OPTIONS]

Options:
      --port-range <RANGE>     Port range to scan [default: 3000-3100]
      --timeout <SECONDS>      Discovery timeout per port [default: 2]
      --protocol <PROTOCOL>    Protocol to discover: http, websocket, all [default: all]
      --output-format <FORMAT> Output format: table, json [default: table]
```

**Examples:**
```bash
# Discover HTTP servers on default ports
mcp-test-harness discover

# Scan specific port range
mcp-test-harness discover --port-range "8000-9000"

# Output as JSON
mcp-test-harness discover --output-format json
```

#### `report` - Generate Reports
Generate comprehensive test reports from previous runs.

```bash
mcp-test-harness report [OPTIONS]

Options:
  -i, --input <DIR>            Input directory with test results
  -o, --output <DIR>           Output directory for reports
  -f, --format <FORMAT>        Report format: html, json, junit, pdf [default: html]
      --template <TEMPLATE>    Custom report template
      --include-charts         Include performance charts
      --include-trends         Include trend analysis
```

## 📊 Understanding Test Results

### Test Output Format

```
🧪 Running Test Suite: core-protocol-tests
================================================================================

✅ PASS: initialize
   Duration: 150ms
   Server Response: {"protocolVersion": "2024-11-05", "capabilities": {...}}

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

### Detailed Error Information

When tests fail, detailed diagnostic information is provided:

```
❌ FAIL: repository_stats
   Duration: 2.3s
   
   Request Sent:
   {
     "tool": "repository_stats",
     "arguments": {
       "path": "/test/project"
     }
   }
   
   Server Response:
   {
     "error": "Path not found: /test/project"
   }
   
   Validation Failure:
   - Expected field 'result.total_files' not found
   - Response contains error instead of expected data
   
   Suggestion:
   - Verify the test project path exists
   - Check server file system access permissions
```

## ⚙️ Configuration Guide

### Configuration File Structure

```yaml
# Global settings apply to all test suites
global:
  max_global_concurrency: 4        # Max parallel tests
  global_timeout_seconds: 300      # Global timeout
  fail_fast: false                 # Stop on first failure
  default_project_path: "test-projects/sample"
  
  # Retry configuration
  retry:
    max_retries: 2
    retry_delay_ms: 1000
    exponential_backoff: true
    retry_on_patterns:
      - "connection refused"
      - "timeout"

# Server configuration
server:
  start_command: "cargo run --bin my-mcp-server"
  args: ["stdio"]
  env:
    RUST_LOG: "info"
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10
  
  # Health check configuration
  health_check:
    enabled: true
    interval_seconds: 10
    failure_threshold: 3
    timeout_seconds: 5

# Test suite definitions
test_suites:
  - name: "core-functionality"
    description: "Test core MCP functionality"
    parallel_execution: true
    setup_script: "scripts/setup.py"
    teardown_script: "scripts/cleanup.py"
    
    test_cases:
      - id: "test_initialize"
        description: "Test server initialization"
        tool_name: "initialize"
        enabled: true
        timeout_override_seconds: 10
        
        # Input parameters
        input_params:
          protocolVersion: "2024-11-05"
        
        # Expected response validation
        expected:
          patterns:
            - key: "protocolVersion"
              validation: { type: "equals", value: "2024-11-05" }
              required: true
            - key: "capabilities"
              validation: { type: "object" }
              required: true
          
          custom_scripts:
            - script: "scripts/validate_capabilities.py"
              timeout_seconds: 5
          
          allow_extra_fields: true
        
        # Performance requirements
        performance:
          max_execution_time_ms: 5000
          max_memory_usage_mb: 64

# Reporting configuration
reporting:
  output_dir: "test-reports"
  formats: ["html", "json"]
  include_debug_info: true
  open_html: false
  
  # Chart configuration
  charts:
    enabled: true
    types: ["response_time", "success_rate"]
    size:
      width: 800
      height: 400

# Environment configuration
environment:
  variables:
    TEST_ENV: "automated"
  path_additions: ["/usr/local/bin"]
  limits:
    max_memory_mb: 1024
    max_cpu_seconds: 300
```

### Server Types Configuration

#### stdio Server
```yaml
server:
  transport: "stdio"
  start_command: "node"
  args: ["server.js"]
  working_dir: "/path/to/server"
  env:
    NODE_ENV: "test"
```

#### HTTP Server
```yaml
server:
  transport: "http"
  url: "http://localhost:3000"
  connection_timeout: 10
  headers:
    Authorization: "Bearer ${API_TOKEN}"
    Content-Type: "application/json"
```

#### WebSocket Server
```yaml
server:
  transport: "websocket"
  url: "ws://localhost:3000"
  connection_timeout: 10
  ping_interval: 30
```

### Validation Patterns

#### Basic Patterns
```yaml
expected:
  patterns:
    # Check if field exists
    - key: "status"
      validation: { type: "exists" }
      required: true
    
    # Check exact value
    - key: "status"
      validation: { type: "equals", value: "success" }
      required: true
    
    # Check value in list
    - key: "status"
      validation: { type: "one_of", values: ["success", "ok", "complete"] }
      required: true
```

#### Numeric Validation
```yaml
expected:
  patterns:
    # Numeric range
    - key: "count"
      validation: { type: "range", min: 1.0, max: 100.0 }
      required: true
    
    # Greater than
    - key: "timestamp"
      validation: { type: "greater_than", value: 1640995200 }
      required: true
    
    # Less than or equal
    - key: "score"
      validation: { type: "less_equal", value: 100.0 }
      required: false
```

#### String Validation
```yaml
expected:
  patterns:
    # Regex pattern
    - key: "id"
      validation: { type: "regex", pattern: "^[a-zA-Z0-9_-]+$" }
      required: true
    
    # String length
    - key: "name"
      validation: { type: "string_length", min: 1, max: 50 }
      required: true
    
    # Contains substring
    - key: "message"
      validation: { type: "contains", value: "success" }
      required: false
```

#### Array Validation
```yaml
expected:
  patterns:
    # Array type
    - key: "items"
      validation: { type: "array" }
      required: true
    
    # Array length
    - key: "items"
      validation: { type: "array_length", min: 1, max: 10 }
      required: true
    
    # Array contains
    - key: "tags"
      validation: { type: "array_contains", value: "important" }
      required: false
```

#### Object Validation
```yaml
expected:
  patterns:
    # Object type
    - key: "metadata"
      validation: { type: "object" }
      required: true
    
    # Object has keys
    - key: "metadata"
      validation: { type: "object_keys", keys: ["id", "name", "timestamp"] }
      required: true
    
    # Nested validation
    - key: "result.data.count"
      validation: { type: "range", min: 0.0, max: 1000.0 }
      required: true
```

### Custom Validation Scripts

#### Python Validation Script
```python
#!/usr/bin/env python3
# scripts/validate_response.py

import json
import sys

def validate_response(response_data, test_context):
    """
    Custom validation logic for MCP responses
    
    Args:
        response_data: Parsed JSON response from MCP server
        test_context: Test execution context (test_id, input_params, etc.)
    
    Returns:
        dict: Validation result with success, score, and message
    """
    
    # Example: Validate repository statistics
    if test_context.get('tool_name') == 'repository_stats':
        result = response_data.get('result', {})
        
        # Check required fields
        required_fields = ['total_files', 'languages_detected', 'file_extensions']
        missing_fields = []
        
        for field in required_fields:
            if field not in result:
                missing_fields.append(field)
        
        if missing_fields:
            return {
                'success': False,
                'score': 0.0,
                'message': f"Missing required fields: {missing_fields}"
            }
        
        # Validate data types and ranges
        total_files = result.get('total_files', 0)
        if not isinstance(total_files, int) or total_files < 0:
            return {
                'success': False,
                'score': 0.3,
                'message': f"Invalid total_files: {total_files}"
            }
        
        # Calculate quality score based on completeness
        score = 1.0
        if not result.get('languages_detected'):
            score -= 0.2
        if not result.get('file_extensions'):
            score -= 0.2
        
        return {
            'success': True,
            'score': score,
            'message': f"Repository stats validation passed (score: {score})"
        }
    
    # Default validation for other tools
    if 'error' in response_data:
        return {
            'success': False,
            'score': 0.0,
            'message': f"Server returned error: {response_data['error']}"
        }
    
    return {
        'success': True,
        'score': 1.0,
        'message': "Basic validation passed"
    }

# Script entry point
if __name__ == "__main__":
    # Read response data and test context from stdin
    input_data = json.load(sys.stdin)
    response_data = input_data['response']
    test_context = input_data['context']
    
    # Run validation
    result = validate_response(response_data, test_context)
    
    # Output result
    json.dump(result, sys.stdout)
```

#### JavaScript Validation Script
```javascript
// scripts/validate_response.js

function validateResponse(responseData, testContext) {
    // Example: Validate search results
    if (testContext.toolName === 'search_symbols') {
        const result = responseData.result || {};
        
        if (!result.matches || !Array.isArray(result.matches)) {
            return {
                success: false,
                score: 0.0,
                message: "Missing or invalid matches array"
            };
        }
        
        // Validate each match has required fields
        const requiredFields = ['symbol', 'file', 'line'];
        let validMatches = 0;
        
        for (const match of result.matches) {
            const hasAllFields = requiredFields.every(field => field in match);
            if (hasAllFields) {
                validMatches++;
            }
        }
        
        const score = result.matches.length > 0 ? validMatches / result.matches.length : 0;
        
        return {
            success: score > 0.8,
            score: score,
            message: `Search validation: ${validMatches}/${result.matches.length} valid matches`
        };
    }
    
    // Default validation
    return {
        success: !responseData.error,
        score: responseData.error ? 0.0 : 1.0,
        message: responseData.error || "Basic validation passed"
    };
}

// Node.js entry point
if (require.main === module) {
    let inputData = '';
    
    process.stdin.on('data', (chunk) => {
        inputData += chunk;
    });
    
    process.stdin.on('end', () => {
        const input = JSON.parse(inputData);
        const result = validateResponse(input.response, input.context);
        console.log(JSON.stringify(result));
    });
}

module.exports = { validateResponse };
```

## 🔍 Test Suite Organization

### Recommended Test Suite Structure

```
test-suites/
├── protocol/
│   ├── initialization.yaml      # Server handshake and capability negotiation
│   ├── resource-management.yaml # Resource listing and access
│   ├── tool-execution.yaml     # Tool calling and response handling
│   └── error-handling.yaml     # Error conditions and recovery
├── functionality/
│   ├── core-tools.yaml         # Essential tool functionality
│   ├── search-tools.yaml       # Search and discovery tools
│   ├── analysis-tools.yaml     # Code analysis and metrics
│   └── workflow-tools.yaml     # Batch operations and workflows
├── performance/
│   ├── latency-tests.yaml      # Response time requirements
│   ├── throughput-tests.yaml   # Request handling capacity
│   ├── memory-tests.yaml       # Memory usage and limits
│   └── stress-tests.yaml       # High-load scenarios
├── security/
│   ├── authentication.yaml     # Authentication mechanisms
│   ├── authorization.yaml      # Access control validation
│   ├── input-validation.yaml   # Input sanitization tests
│   └── data-protection.yaml    # Sensitive data handling
└── integration/
    ├── client-compatibility.yaml # Different client implementations
    ├── transport-tests.yaml      # stdio, HTTP, WebSocket
    ├── error-recovery.yaml       # Failure scenarios
    └── edge-cases.yaml           # Boundary conditions
```

### Suite Execution Strategies

#### Parallel Execution
```yaml
test_suites:
  - name: "fast-tests"
    parallel_execution: true
    max_concurrency: 8
    test_cases:
      # Multiple independent tests that can run simultaneously
```

#### Sequential Execution
```yaml
test_suites:
  - name: "setup-dependent-tests"
    parallel_execution: false
    test_cases:
      # Tests that depend on previous test state
```

#### Conditional Execution
```yaml
test_suites:
  - name: "capability-dependent-tests"
    skip_condition: "!server.capabilities.advanced_features"
    test_cases:
      # Tests that only run if server supports advanced features
```

## 📈 Performance Testing

### Performance Requirements
```yaml
performance:
  max_execution_time_ms: 5000      # Maximum response time
  max_memory_usage_mb: 128         # Memory limit during execution
  min_success_rate: 0.95           # Minimum success rate
  max_error_rate: 0.05             # Maximum error rate
  
  # Percentile requirements
  percentiles:
    p50: 1000    # 50th percentile <= 1s
    p95: 3000    # 95th percentile <= 3s  
    p99: 5000    # 99th percentile <= 5s
```

### Baseline Management
```bash
# Establish new baseline
mcp-test-harness benchmark --establish-baseline --config prod.yaml

# Compare against baseline
mcp-test-harness benchmark --compare-baseline --config prod.yaml

# Update baseline after performance improvements
mcp-test-harness benchmark --update-baseline --config prod.yaml
```

### Performance Regression Detection
The test harness automatically detects performance regressions by comparing current results against established baselines:

- **Green** (✅): Performance within expected range
- **Yellow** (⚠️): Minor regression (25-50% slower than baseline)
- **Red** (❌): Significant regression (>50% slower than baseline)

## 🔐 Security Testing

### Authentication Testing
```yaml
test_cases:
  - id: "test_auth_required"
    description: "Verify authentication is required"
    tool_name: "list_resources"
    skip_auth: true
    expected:
      patterns:
        - key: "error"
          validation: { type: "contains", value: "authentication" }
          required: true
```

### Authorization Testing
```yaml
test_cases:
  - id: "test_unauthorized_access"
    description: "Test access to restricted resources"
    tool_name: "read_resource"
    auth_token: "limited_user_token"
    input_params:
      uri: "file:///etc/passwd"
    expected:
      patterns:
        - key: "error"
          validation: { type: "contains", value: "unauthorized" }
          required: true
```

### Input Validation Testing
```yaml
test_cases:
  - id: "test_malicious_input"
    description: "Test protection against malicious input"
    tool_name: "execute_command"
    input_params:
      command: "rm -rf /"
    expected:
      patterns:
        - key: "error"
          validation: { type: "contains", value: "invalid" }
          required: true
```

## 🚨 Error Handling and Debugging

### Common Error Messages

#### Configuration Errors
```
❌ Configuration Error: Invalid YAML syntax in test-harness.yaml at line 15
   Fix: Check YAML indentation and syntax

❌ Configuration Error: Missing required field 'server.start_command'
   Fix: Add server startup command to configuration
```

#### Server Connection Errors
```
❌ Connection Error: Failed to connect to MCP server
   Possible causes:
   - Server is not running
   - Incorrect connection parameters
   - Network connectivity issues
   
   Troubleshooting:
   1. Verify server is running: ps aux | grep mcp-server
   2. Check logs: tail -f server.log
   3. Test connectivity: telnet localhost 3000
```

#### Test Execution Errors
```
❌ Test Error: Tool 'unknown_tool' not found
   Fix: Check tool name spelling and server capabilities

❌ Validation Error: Response missing required field 'result'
   Fix: Update expected patterns or fix server response format
```

### Debug Mode
Enable verbose debugging with multiple levels:

```bash
# Basic debugging
mcp-test-harness test --verbose --config my-config.yaml

# Maximum debugging (includes request/response details)
mcp-test-harness test -vvv --config my-config.yaml

# Debug specific test
mcp-test-harness test --suite "core-tests" --test "initialize" --debug
```

### Log Analysis
Test harness generates detailed logs for troubleshooting:

```bash
# View recent logs
tail -f ~/.mcp-test-harness/logs/test-harness.log

# Search for errors
grep "ERROR" ~/.mcp-test-harness/logs/test-harness.log

# Analyze performance
grep "PERF" ~/.mcp-test-harness/logs/test-harness.log
```

## 🔄 Integration with CI/CD

### GitHub Actions
```yaml
name: MCP Server Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install MCP Test Harness
        run: |
          curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux-x86_64.tar.gz | tar xz
          sudo mv mcp-test-harness /usr/local/bin/
      
      - name: Run Tests
        run: |
          mcp-test-harness test \
            --config .github/test-config.yaml \
            --output-format junit \
            --output test-results/
      
      - name: Upload Results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: test-results/
```

### GitLab CI
```yaml
test-mcp-server:
  image: ubuntu:22.04
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux-x86_64.tar.gz | tar xz
    - mv mcp-test-harness /usr/local/bin/
  script:
    - mcp-test-harness test --config ci-config.yaml --output-format junit
  artifacts:
    reports:
      junit: test-results.xml
    expire_in: 30 days
```

### Jenkins Pipeline
```groovy
pipeline {
    agent any
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux-x86_64.tar.gz | tar xz
                    sudo mv mcp-test-harness /usr/local/bin/
                '''
            }
        }
        
        stage('Test') {
            steps {
                sh '''
                    mcp-test-harness test \
                        --config jenkins/test-config.yaml \
                        --output-format junit \
                        --output test-results/
                '''
            }
            post {
                always {
                    junit 'test-results/*.xml'
                    archiveArtifacts artifacts: 'test-results/**', allowEmptyArchive: true
                }
            }
        }
    }
}
```

## 🎯 Best Practices

### Configuration Management
- **Version Control**: Keep test configurations in version control
- **Environment Separation**: Separate configs for dev/staging/production
- **Secrets Management**: Use environment variables for sensitive data
- **Validation**: Always validate configs before deployment

### Test Design
- **Atomic Tests**: Each test should be independent and focused
- **Clear Naming**: Use descriptive test IDs and descriptions
- **Comprehensive Coverage**: Test happy path, edge cases, and error conditions
- **Performance Awareness**: Set appropriate timeouts and resource limits

### Maintenance
- **Regular Updates**: Keep baselines current with performance improvements
- **Log Monitoring**: Regularly review test logs for issues
- **Configuration Reviews**: Periodically review and update test configurations
- **Documentation**: Keep test documentation up to date

### Troubleshooting
- **Start Simple**: Begin with basic connectivity tests
- **Check Logs**: Always examine server and test harness logs
- **Isolate Issues**: Run individual tests to isolate problems
- **Use Debug Mode**: Enable verbose logging for detailed diagnostics

---

## 📚 Additional Resources

- [Configuration Reference](configuration-reference.md) - Complete configuration documentation
- [Production Deployment](production-deployment.md) - Enterprise deployment guide
- [Contributing Guide](../Contributing) - Contributing to the project
- [Troubleshooting Guide](troubleshooting) - Common issues and solutions
- [API Reference](../API_Reference) - Complete API documentation

**Last Updated**: 2025-01-07  
**Version**: 1.0.0 