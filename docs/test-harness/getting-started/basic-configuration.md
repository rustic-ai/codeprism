# Basic Configuration

Learn the fundamentals of configuring the MCP Test Harness for your testing needs.

## 📋 Overview

This guide covers the essential configuration concepts you need to understand to effectively use the MCP Test Harness:

- **Configuration structure and hierarchy**
- **Global settings and their impact**
- **Server configuration options**
- **Test suite organization**
- **Validation patterns**
- **Common configuration patterns**

## 🏗️ Configuration Structure

All MCP Test Harness configurations follow this basic structure:

```yaml
# Configuration file anatomy
global:           # Settings that apply to all tests
  # Global test execution settings

server:          # How to start and communicate with your MCP server
  # Server startup and transport configuration

test_suites:     # Groups of related tests
  - name: "suite_name"
    test_cases:    # Individual test definitions
      - id: "test_id"
        # Test-specific configuration

reporting:       # How to output test results
  # Report generation settings

environment:     # Environment setup and cleanup
  # Environment configuration
```

## 🌍 Global Configuration

Global settings affect all test execution:

### Basic Global Settings

```yaml
global:
  # Concurrency: How many tests run simultaneously
  max_global_concurrency: 4
  
  # Timeouts: Maximum time for any single test
  global_timeout_seconds: 300
  
  # Failure handling: Stop all tests on first failure
  fail_fast: false
  
  # Default paths: Used when tests don't specify paths
  default_project_path: "/path/to/test/project"
```

### Advanced Global Settings

```yaml
global:
  # Retry configuration
  retry:
    max_retries: 2              # Retry failed tests up to 2 times
    retry_delay_ms: 1000        # Wait 1 second between retries
    exponential_backoff: true   # Increase delay for each retry
    retry_on_patterns:          # Only retry on specific errors
      - "connection refused"
      - "timeout"
      - "server not ready"
  
  # Logging configuration
  logging:
    level: "info"               # debug, info, warn, error
    file: "test-harness.log"   # Log file location
    include_server_output: true # Include MCP server logs
  
  # Resource limits
  limits:
    max_memory_mb: 2048        # Maximum memory usage
    max_cpu_seconds: 600       # Maximum CPU time
    max_disk_mb: 1024         # Maximum disk usage
```

## 🖥️ Server Configuration

Configure how to start and communicate with your MCP server:

### stdio Transport (Most Common)

```yaml
server:
  transport: "stdio"
  start_command: "node"
  args: ["server.js", "stdio"]
  
  # Working directory for server process
  working_dir: "/path/to/server"
  
  # Environment variables
  env:
    NODE_ENV: "test"
    DEBUG: "mcp:*"
  
  # Timing configuration
  startup_timeout_seconds: 30
  shutdown_timeout_seconds: 10
```

### HTTP Transport

```yaml
server:
  transport: "http"
  url: "http://localhost:3000"
  
  # Connection settings
  connection_timeout: 10
  read_timeout: 30
  
  # HTTP headers
  headers:
    Authorization: "Bearer ${API_TOKEN}"
    Content-Type: "application/json"
    User-Agent: "MCP-Test-Harness/1.0"
  
  # Health check (optional)
  health_check:
    endpoint: "/health"
    interval_seconds: 10
```

### WebSocket Transport

```yaml
server:
  transport: "websocket"
  url: "ws://localhost:3000/mcp"
  
  # WebSocket settings
  connection_timeout: 10
  ping_interval: 30
  max_frame_size: 65536
  
  # Authentication (if required)
  headers:
    Authorization: "Bearer ${WS_TOKEN}"
```

### Advanced Server Configuration

```yaml
server:
  # Server lifecycle
  start_command: "cargo run --bin my-server"
  args: ["--config", "test-config.toml"]
  
  # Process management
  restart_on_failure: true
  max_restart_attempts: 3
  restart_delay_seconds: 5
  
  # Health monitoring
  health_check:
    enabled: true
    method: "ping"              # ping, endpoint, or command
    interval_seconds: 10
    failure_threshold: 3
    timeout_seconds: 5
  
  # Resource monitoring
  monitoring:
    memory_limit_mb: 512
    cpu_limit_percent: 80
    alert_on_limits: true
```

## 📝 Test Suite Configuration

Organize your tests into logical groups:

### Basic Test Suite

```yaml
test_suites:
  - name: "core_functionality"
    description: "Test core MCP protocol features"
    
    # Suite-level settings
    parallel_execution: true
    max_suite_concurrency: 2
    
    test_cases:
      - id: "initialize"
        description: "Test server initialization"
        tool_name: "initialize"
        
        input_params:
          protocolVersion: "2024-11-05"
        
        expected:
          patterns:
            - key: "protocolVersion"
              validation: { type: "equals", value: "2024-11-05" }
              required: true
```

### Advanced Test Suite

```yaml
test_suites:
  - name: "filesystem_operations"
    description: "Test filesystem-related functionality"
    
    # Setup/teardown for the entire suite
    setup_script: "scripts/setup_filesystem.py"
    teardown_script: "scripts/cleanup_filesystem.py"
    
    # Suite-level configuration
    timeout_override_seconds: 120
    retry_config:
      max_retries: 1
      retry_delay_ms: 500
    
    # Environment for this suite
    environment:
      TEST_DATA_DIR: "/tmp/mcp-test-data"
      FILESYSTEM_ROOT: "/tmp/mcp-fs-root"
    
    test_cases:
      - id: "create_directory"
        # Test configuration here
      - id: "list_directory"
        # Test configuration here
```

## 🧪 Test Case Configuration

Individual test definitions:

### Basic Test Case

```yaml
- id: "simple_test"
  description: "A simple test example"
  enabled: true                    # Can disable tests
  
  tool_name: "get_info"
  input_params:
    path: "/example"
  
  expected:
    patterns:
      - key: "info.type"
        validation: { type: "equals", value: "directory" }
        required: true
```

### Advanced Test Case

```yaml
- id: "complex_validation"
  description: "Complex validation example"
  
  # Test-specific overrides
  timeout_override_seconds: 60
  max_retries: 3
  
  # Multiple validation approaches
  tool_name: "analyze_code"
  input_params:
    file_path: "example.py"
    analysis_type: "complexity"
  
  expected:
    # Pattern-based validation
    patterns:
      - key: "complexity.cyclomatic"
        validation: { type: "range", min: 1.0, max: 10.0 }
        required: true
      
      - key: "complexity.lines"
        validation: { type: "greater_than", value: 0 }
        required: true
    
    # Custom script validation
    custom_scripts:
      - script: "scripts/validate_complexity.py"
        timeout_seconds: 30
        language: "python"
    
    # Performance requirements
    performance:
      max_execution_time_ms: 5000
      max_memory_usage_mb: 128
    
    # Allow extra fields in response
    allow_extra_fields: true
```

## ✅ Validation Patterns

Configure how responses are validated:

### Basic Validations

```yaml
expected:
  patterns:
    # Check if field exists
    - key: "result"
      validation: { type: "exists" }
      required: true
    
    # Check exact value
    - key: "status"
      validation: { type: "equals", value: "success" }
      required: true
    
    # Check type
    - key: "count"
      validation: { type: "integer" }
      required: true
    
    # Check if value is in list
    - key: "category"
      validation: { type: "one_of", values: ["A", "B", "C"] }
      required: true
```

### Advanced Validations

```yaml
expected:
  patterns:
    # Numeric ranges
    - key: "score"
      validation: { type: "range", min: 0.0, max: 100.0 }
      required: true
    
    # String patterns
    - key: "id"
      validation: { type: "regex", pattern: "^[a-zA-Z0-9_-]+$" }
      required: true
    
    # String length
    - key: "description"
      validation: { type: "string_length", min: 10, max: 500 }
      required: false
    
    # Array validation
    - key: "items"
      validation: { type: "array" }
      required: true
    
    - key: "items"
      validation: { type: "array_length", min: 1, max: 100 }
      required: true
    
    # Nested object validation
    - key: "metadata.version"
      validation: { type: "equals", value: "1.0" }
      required: true
```

### Error Handling

```yaml
expected:
  # Expect an error response
  error_expected: true
  
  patterns:
    - key: "error.code"
      validation: { type: "one_of", values: ["NOT_FOUND", "PERMISSION_DENIED"] }
      required: true
    
    - key: "error.message"
      validation: { type: "string_length", min: 10 }
      required: true
```

## 📊 Reporting Configuration

Configure how test results are output:

### Basic Reporting

```yaml
reporting:
  output_dir: "test-reports"
  formats: ["html", "json"]
  open_html: false              # Auto-open HTML report
```

### Advanced Reporting

```yaml
reporting:
  output_dir: "test-reports"
  formats: ["html", "json", "junit", "text"]
  
  # Report customization
  include_debug_info: true
  include_server_logs: true
  include_configuration: true
  
  # HTML report settings
  html:
    template: "custom-template.html"
    include_charts: true
    chart_types: ["response_time", "success_rate", "memory_usage"]
  
  # JSON report settings
  json:
    pretty_print: true
    include_raw_responses: false
  
  # JUnit XML (for CI integration)
  junit:
    filename: "test-results.xml"
    include_stdout: true
```

## 🔧 Environment Configuration

Set up the testing environment:

### Basic Environment

```yaml
environment:
  variables:
    TEST_MODE: "true"
    LOG_LEVEL: "debug"
  
  path_additions:
    - "/usr/local/bin"
    - "$HOME/.cargo/bin"
```

### Advanced Environment

```yaml
environment:
  # Environment variables
  variables:
    DATABASE_URL: "sqlite:///tmp/test.db"
    REDIS_URL: "redis://localhost:6379/1"
    API_BASE_URL: "http://localhost:3000"
  
  # Load from files
  env_files:
    - ".env.test"
    - "secrets.env"
  
  # Setup and teardown commands
  setup_commands:
    - "mkdir -p test-data"
    - "python scripts/seed_database.py"
  
  teardown_commands:
    - "rm -rf test-data"
    - "python scripts/cleanup_database.py"
  
  # Resource limits
  limits:
    max_memory_mb: 1024
    max_cpu_seconds: 300
    max_processes: 10
```

## 🎯 Common Configuration Patterns

### Pattern 1: Simple HTTP API Testing

```yaml
global:
  max_global_concurrency: 3
  global_timeout_seconds: 30

server:
  transport: "http"
  url: "http://localhost:8000"
  headers:
    Content-Type: "application/json"

test_suites:
  - name: "api_tests"
    test_cases:
      - id: "health_check"
        tool_name: "health"
        input_params: {}
        expected:
          patterns:
            - key: "status"
              validation: { type: "equals", value: "healthy" }
```

### Pattern 2: File System Server Testing

```yaml
global:
  max_global_concurrency: 2
  default_project_path: "/tmp/test-fs"

server:
  transport: "stdio"
  start_command: "node"
  args: ["fs-server.js"]

environment:
  setup_commands:
    - "mkdir -p /tmp/test-fs"
    - "echo 'test content' > /tmp/test-fs/test.txt"
  teardown_commands:
    - "rm -rf /tmp/test-fs"

test_suites:
  - name: "filesystem"
    test_cases:
      - id: "read_file"
        tool_name: "read_file"
        input_params:
          path: "/tmp/test-fs/test.txt"
        expected:
          patterns:
            - key: "content"
              validation: { type: "contains", value: "test content" }
```

### Pattern 3: Performance Testing

```yaml
global:
  max_global_concurrency: 1  # Single-threaded for performance testing

server:
  transport: "stdio"
  start_command: "my-server"

test_suites:
  - name: "performance"
    test_cases:
      - id: "benchmark_operation"
        tool_name: "heavy_computation"
        input_params:
          size: 1000
        
        performance:
          max_execution_time_ms: 5000
          max_memory_usage_mb: 256
        
        expected:
          patterns:
            - key: "result"
              validation: { type: "exists" }
```

## 🔍 Configuration Validation

Always validate your configuration before running tests:

```bash
# Validate configuration syntax
mcp-test-harness validate --config my-config.yaml

# Test server connectivity
mcp-test-harness validate --connectivity --config my-config.yaml

# Comprehensive validation
mcp-test-harness validate --comprehensive --config my-config.yaml
```

## 🛠️ Configuration Tips

### 1. Start Simple
Begin with minimal configuration and add complexity as needed:

```yaml
# Start with this
global:
  max_global_concurrency: 1

server:
  transport: "stdio"
  start_command: "my-server"

test_suites:
  - name: "basic"
    test_cases:
      - id: "ping"
        tool_name: "ping"
        input_params: {}
        expected:
          allow_any_response: true
```

### 2. Use Environment Variables
Keep sensitive data out of configuration files:

```yaml
server:
  env:
    API_KEY: "${MCP_API_KEY}"     # From environment
    DATABASE_URL: "${DB_URL}"     # From environment
```

### 3. Organize with Multiple Files
Split large configurations:

```yaml
# main-config.yaml
global:
  max_global_concurrency: 4

server: !include server-config.yaml
test_suites: !include test-suites.yaml
```

### 4. Use Configuration Templates
Generate configurations for common patterns:

```bash
# Generate a template
mcp-test-harness template --server-type filesystem --output fs-config.yaml
```

## 📚 Next Steps

Now that you understand basic configuration:

1. **[Configuration Reference](../configuration-reference.md)** - Complete YAML schema
2. **[Testing Guides](../testing-guides/)** - Specific testing scenarios
3. **[Example Configurations](../examples/)** - Real-world examples
4. **[User Guide](../user-guide.md)** - Complete usage instructions

## 🐛 Troubleshooting Configuration

### Common Issues

**Issue**: Configuration validation fails
```yaml
# Check YAML syntax
global:
  max_global_concurrency: 4  # Not "four"
```

**Issue**: Server won't start
```yaml
# Verify command and path
server:
  start_command: "which node && node"  # Test command exists
  working_dir: "/correct/path"         # Verify path exists
```

**Issue**: Tests timeout
```yaml
# Increase timeouts
global:
  global_timeout_seconds: 120  # From 30 to 120 seconds
```

---

**You now understand the fundamentals of MCP Test Harness configuration!** 🎉 