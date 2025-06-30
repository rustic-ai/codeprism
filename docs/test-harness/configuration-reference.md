# Configuration Reference

Complete reference for MCP Test Harness configuration files with detailed explanations of all options and validation rules.

## 📋 Overview

The MCP Test Harness uses YAML configuration files to define test suites, server settings, validation patterns, and execution parameters. This reference provides comprehensive documentation for all configuration options.

## 🏗️ Configuration File Structure

```yaml
# Top-level configuration structure
global:                 # Global execution settings
  max_global_concurrency: 4
  global_timeout_seconds: 300
  # ... (global settings)

server:                 # MCP server configuration  
  start_command: "command"
  args: ["arg1", "arg2"]
  # ... (server settings)

test_suites:            # Array of test suite definitions
  - name: "suite1"
    test_cases:
      - id: "test1"
        # ... (test case definition)

performance:            # Performance monitoring settings
  baseline_storage_path: "baselines/"
  # ... (performance settings)

reporting:              # Report generation settings
  output_dir: "reports/"
  formats: ["html", "json"]
  # ... (reporting settings)

environment:            # Environment configuration
  variables: {}
  path_additions: []
  # ... (environment settings)

security:               # Security and compliance settings
  enable_audit_logging: true
  # ... (security settings)

monitoring:             # Monitoring and alerting
  enabled: true
  # ... (monitoring settings)
```

## 🌐 Global Configuration

The `global` section defines settings that apply to all test suites and execution contexts.

### Basic Global Settings

```yaml
global:
  # Execution control
  max_global_concurrency: 4              # Max parallel test executions (1-32)
  global_timeout_seconds: 300            # Global timeout for all operations (1-3600)
  fail_fast: false                       # Stop execution on first failure
  default_project_path: "test-projects/sample"  # Default path for test projects
  
  # Environment identification
  environment: "development"             # Environment name (development, staging, production)
  
  # Logging configuration
  log_level: "info"                     # Log level: trace, debug, info, warn, error
  log_format: "text"                    # Log format: text, json
  log_file: "test-harness.log"          # Log file path (optional)
  console_output: true                  # Enable console output
  
  # Resource limits
  resource_limits:
    max_memory_mb: 1024                 # Maximum memory usage per test (64-8192)
    max_cpu_seconds: 300                # Maximum CPU time per test (1-1800)
    max_disk_usage_mb: 512              # Maximum disk usage (16-2048)
    max_network_connections: 50         # Maximum network connections (1-200)
    max_open_files: 256                 # Maximum open file descriptors (16-1024)
```

### Retry Configuration

```yaml
global:
  retry:
    max_retries: 2                      # Maximum retry attempts (0-10)
    retry_delay_ms: 1000                # Initial delay between retries (100-10000)
    exponential_backoff: true           # Use exponential backoff strategy
    max_backoff_ms: 30000               # Maximum backoff delay (1000-60000)
    jitter: true                        # Add random jitter to backoff
    
    # Patterns that trigger retries
    retry_on_patterns:
      - "connection refused"
      - "timeout"
      - "temporary failure"
      - "resource unavailable"
      - "server busy"
    
    # HTTP status codes that trigger retries
    retry_on_status_codes: [500, 502, 503, 504, 429]
    
    # Don't retry on these patterns (takes precedence)
    no_retry_patterns:
      - "authentication failed"
      - "unauthorized"
      - "forbidden"
      - "not found"
```

### Advanced Global Settings

```yaml
global:
  # Test execution behavior
  stop_on_first_suite_failure: false    # Stop all execution if any suite fails
  continue_on_setup_failure: false      # Continue if setup scripts fail
  parallel_suite_execution: true        # Run test suites in parallel
  
  # Data collection
  collect_metrics: true                 # Collect detailed performance metrics
  collect_traces: false                 # Collect execution traces (debug mode)
  metrics_interval_seconds: 10          # Metrics collection interval
  
  # Cleanup behavior
  cleanup_on_success: true              # Cleanup temporary files on success
  cleanup_on_failure: false             # Keep artifacts for debugging on failure
  preserve_server_logs: true            # Always preserve server logs
  
  # Validation settings
  strict_validation: false              # Strict JSON schema validation
  allow_unknown_fields: true            # Allow unknown fields in responses
  normalize_responses: true             # Normalize response data structures
```

## 🖥️ Server Configuration

The `server` section defines how to start, connect to, and manage the MCP server under test.

### Basic Server Settings

```yaml
server:
  # Server identification
  name: "My MCP Server"                 # Human-readable server name
  version: "1.0.0"                     # Server version (for reporting)
  
  # Server lifecycle
  start_command: "cargo run --bin server"  # Command to start server
  args: ["stdio"]                       # Command line arguments
  working_dir: "/path/to/server"        # Working directory for server process
  startup_timeout_seconds: 30          # Time to wait for server startup (5-300)
  shutdown_timeout_seconds: 10         # Time to wait for graceful shutdown (1-60)
  
  # Process management
  auto_start: true                      # Automatically start server
  auto_restart: false                   # Restart server on failure
  restart_delay_seconds: 5             # Delay before restart (1-60)
  max_restart_attempts: 3              # Maximum restart attempts (1-10)
```

### Environment and Process Settings

```yaml
server:
  # Environment variables
  env:
    RUST_LOG: "info"
    NODE_ENV: "test"
    DATABASE_URL: "sqlite::memory:"
    API_KEY: "${SECRET_API_KEY}"        # Environment variable substitution
    
  # Path configuration
  path_additions:
    - "/usr/local/bin"
    - "./target/release"
    
  # Process limits
  limits:
    max_memory_mb: 512                  # Memory limit for server process
    max_cpu_percent: 80                 # CPU usage limit
    nice_level: 0                       # Process priority (-20 to 19)
    
  # Signal handling
  shutdown_signal: "SIGTERM"           # Signal for graceful shutdown
  kill_signal: "SIGKILL"               # Signal for forced termination
  signal_timeout_seconds: 10           # Time between signals
```

### Transport Configuration

#### stdio Transport

```yaml
server:
  transport: "stdio"
  
  # stdio-specific settings
  stdin_buffer_size: 8192              # Input buffer size (1024-65536)
  stdout_buffer_size: 8192             # Output buffer size (1024-65536)
  line_ending: "lf"                    # Line ending: lf, crlf, auto
  
  # Process communication
  pipe_timeout_seconds: 30             # Timeout for pipe operations
  read_timeout_seconds: 10             # Timeout for reading responses
  write_timeout_seconds: 5             # Timeout for writing requests
```

#### HTTP Transport

```yaml
server:
  transport: "http"
  url: "http://localhost:3000"         # Server URL
  
  # Connection settings
  connection_timeout_seconds: 10       # Connection establishment timeout
  request_timeout_seconds: 30          # Individual request timeout
  keep_alive: true                     # Use HTTP keep-alive
  max_connections: 10                  # Connection pool size
  
  # Headers
  headers:
    Authorization: "Bearer ${API_TOKEN}"
    Content-Type: "application/json"
    User-Agent: "MCP-Test-Harness/1.0"
    
  # TLS settings (for HTTPS)
  tls:
    verify_certificates: true          # Verify SSL certificates
    ca_bundle_path: "/path/to/ca.pem"  # Custom CA bundle
    client_cert_path: "/path/to/cert.pem"  # Client certificate
    client_key_path: "/path/to/key.pem"    # Client private key
```

#### WebSocket Transport

```yaml
server:
  transport: "websocket"
  url: "ws://localhost:3000"           # WebSocket URL
  
  # WebSocket settings
  connection_timeout_seconds: 10       # Connection timeout
  ping_interval_seconds: 30            # Ping interval for keep-alive
  pong_timeout_seconds: 10             # Pong response timeout
  max_message_size: 1048576            # Maximum message size (bytes)
  
  # Protocols and extensions
  subprotocols: ["mcp-1.0"]            # WebSocket subprotocols
  
  # Headers (for initial HTTP upgrade)
  headers:
    Authorization: "Bearer ${TOKEN}"
```

### Health Check Configuration

```yaml
server:
  health_check:
    enabled: true                      # Enable health monitoring
    interval_seconds: 10               # Health check interval (5-300)
    timeout_seconds: 5                 # Health check timeout (1-30)
    failure_threshold: 3               # Failures before marking unhealthy (1-10)
    success_threshold: 2               # Successes to mark healthy again (1-5)
    recovery_timeout_seconds: 60       # Time to wait for recovery (10-600)
    
    # Health check method
    method: "ping"                     # Health check method: ping, custom
    custom_tool: "health_check"        # Custom tool for health checking
    
    # Actions on health check failure
    on_failure: "restart"              # Action: restart, stop, ignore
    max_failure_restarts: 3            # Maximum restart attempts
```

## 📋 Test Suite Configuration

Test suites are collections of related test cases with shared configuration and execution parameters.

### Basic Test Suite Structure

```yaml
test_suites:
  - name: "core-protocol-tests"         # Unique suite name (required)
    description: "Core MCP protocol compliance tests"  # Human-readable description
    
    # Execution control
    enabled: true                       # Enable/disable entire suite
    parallel_execution: true            # Run tests in parallel within suite
    max_concurrency: 4                  # Max parallel tests in this suite (1-16)
    timeout_seconds: 300                # Suite-level timeout (10-3600)
    
    # Dependencies and setup
    depends_on: ["initialization-tests"] # Other suites that must pass first
    setup_script: "scripts/setup.py"    # Script to run before suite
    teardown_script: "scripts/cleanup.py"  # Script to run after suite
    
    # Failure handling
    fail_fast: false                    # Stop suite on first test failure
    continue_on_failure: true           # Continue with other suites if this fails
    required_success_rate: 0.8          # Minimum success rate for suite to pass
    
    test_cases:
      # ... test case definitions
```

### Advanced Test Suite Configuration

```yaml
test_suites:
  - name: "performance-tests"
    
    # Conditional execution
    skip_condition: "!server.capabilities.performance_tools"  # Skip condition
    only_if: "environment == 'production'"  # Only run if condition met
    
    # Resource management
    resource_limits:
      max_memory_mb: 256                # Suite-specific memory limit
      max_execution_time_seconds: 600   # Suite execution time limit
      max_disk_usage_mb: 128            # Disk usage limit
    
    # Retry configuration (overrides global)
    retry:
      max_retries: 1                    # Fewer retries for performance tests
      retry_delay_ms: 500
    
    # Data and context
    variables:                          # Suite-specific variables
      test_dataset: "large_project"
      performance_threshold: 5000
      
    tags: ["performance", "slow", "critical"]  # Tags for filtering
    
    # Reporting
    custom_report_template: "performance-report.html"
    include_in_summary: true            # Include in overall summary
```

## 🧪 Test Case Configuration

Individual test cases define specific interactions with the MCP server and validation rules.

### Basic Test Case Structure

```yaml
test_cases:
  - id: "test_repository_stats"         # Unique test identifier (required)
    description: "Test repository statistics tool"  # Human-readable description
    
    # Execution control
    enabled: true                       # Enable/disable test
    tool_name: "repository_stats"       # MCP tool to invoke (required)
    timeout_override_seconds: 10        # Test-specific timeout
    
    # Input parameters
    input_params:                       # Parameters passed to MCP tool
      path: "${default_project_path}"   # Variable substitution
      include_hidden: false
      format: "json"
    
    # Expected response validation
    expected:
      # ... validation rules (see Validation section)
    
    # Performance requirements
    performance:
      max_execution_time_ms: 5000       # Maximum response time
      max_memory_usage_mb: 64           # Memory usage limit
```

### Advanced Test Case Configuration

```yaml
test_cases:
  - id: "complex_analysis_test"
    
    # Dependencies
    depends_on: ["initialization_test"]  # Tests that must pass first
    setup_steps:                        # Setup actions before test
      - tool: "create_temp_project"
        params: { name: "test_project" }
    
    # Multiple tool calls (for workflow testing)
    steps:
      - tool: "initialize"              # First tool call
        params: { version: "2024-11-05" }
        store_result_as: "init_result"  # Store result for later use
        
      - tool: "repository_stats"        # Second tool call
        params: 
          path: "${init_result.project_path}"  # Use result from previous step
        expected:
          # ... validation for this step
    
    # Conditional execution
    skip_if: "!server.capabilities.advanced_analysis"
    only_if: "test_dataset == 'large_project'"
    
    # Error expectations (for negative tests)
    expect_error: true                  # Test should fail
    expected_error_pattern: "invalid path"  # Expected error message pattern
    expected_error_code: "INVALID_PATH" # Expected error code
    
    # Cleanup
    cleanup_steps:
      - tool: "delete_temp_project"
        params: { name: "test_project" }
```

### Test Case Variables and Context

```yaml
test_cases:
  - id: "parameterized_test"
    
    # Variable definitions
    variables:
      project_paths:
        - "test-projects/small"
        - "test-projects/medium" 
        - "test-projects/large"
      expected_file_counts: [5, 50, 500]
    
    # Parameter matrix (creates multiple test instances)
    matrix:
      project_path: "${project_paths}"
      expected_count: "${expected_file_counts}"
    
    # Use matrix variables in test
    input_params:
      path: "${matrix.project_path}"
    
    expected:
      patterns:
        - key: "result.total_files"
          validation: 
            type: "range"
            min: "${matrix.expected_count * 0.8}"
            max: "${matrix.expected_count * 1.2}"
```

## ✅ Validation Configuration

The validation system supports multiple validation types and custom validation scripts.

### Pattern-Based Validation

#### Existence Validation

```yaml
expected:
  patterns:
    # Check if field exists
    - key: "result"                     # JSON path to field
      validation: { type: "exists" }   # Validation type
      required: true                    # Fail test if validation fails
      
    # Check if nested field exists
    - key: "result.data.items"
      validation: { type: "exists" }
      required: true
      
    # Check if field does not exist
    - key: "result.error"
      validation: { type: "not_exists" }
      required: false                   # Warning only if validation fails
```

#### Value Validation

```yaml
expected:
  patterns:
    # Exact value match
    - key: "status"
      validation: 
        type: "equals"
        value: "success"
      required: true
      
    # One of multiple values
    - key: "status"
      validation:
        type: "one_of"
        values: ["success", "ok", "complete"]
      required: true
      
    # Not equal to value
    - key: "error"
      validation:
        type: "not_equals"
        value: null
      required: false
```

#### Numeric Validation

```yaml
expected:
  patterns:
    # Numeric range
    - key: "result.total_files"
      validation:
        type: "range"
        min: 1.0                        # Minimum value (inclusive)
        max: 1000.0                     # Maximum value (inclusive)
      required: true
      
    # Greater than
    - key: "result.timestamp"
      validation:
        type: "greater_than"
        value: 1640995200               # Unix timestamp
      required: true
      
    # Less than or equal
    - key: "result.score"
      validation:
        type: "less_equal"
        value: 100.0
      required: true
      
    # Numeric type with precision
    - key: "result.percentage"
      validation:
        type: "numeric"
        min_value: 0.0
        max_value: 100.0
        decimal_places: 2               # Maximum decimal places
      required: true
```

#### String Validation

```yaml
expected:
  patterns:
    # Regular expression
    - key: "result.id"
      validation:
        type: "regex"
        pattern: "^[a-zA-Z0-9_-]+$"    # Regex pattern
        flags: "i"                      # Regex flags (optional)
      required: true
      
    # String length
    - key: "result.name"
      validation:
        type: "string_length"
        min: 1                          # Minimum length
        max: 100                        # Maximum length
      required: true
      
    # Contains substring
    - key: "result.message"
      validation:
        type: "contains"
        value: "success"
        case_sensitive: false           # Case-insensitive matching
      required: true
      
    # Starts with
    - key: "result.path"
      validation:
        type: "starts_with"
        value: "/project/"
      required: true
      
    # Ends with
    - key: "result.filename"
      validation:
        type: "ends_with"
        value: ".json"
      required: true
```

#### Array Validation

```yaml
expected:
  patterns:
    # Array type
    - key: "result.items"
      validation: { type: "array" }
      required: true
      
    # Array length
    - key: "result.items"
      validation:
        type: "array_length"
        min: 1                          # Minimum array length
        max: 100                        # Maximum array length
      required: true
      
    # Array contains value
    - key: "result.tags"
      validation:
        type: "array_contains"
        value: "important"
      required: true
      
    # Array contains all values
    - key: "result.required_fields"
      validation:
        type: "array_contains_all"
        values: ["id", "name", "timestamp"]
      required: true
      
    # Array element validation
    - key: "result.items[*].id"         # Validate all array elements
      validation:
        type: "regex"
        pattern: "^item_\\d+$"
      required: true
```

#### Object Validation

```yaml
expected:
  patterns:
    # Object type
    - key: "result.metadata"
      validation: { type: "object" }
      required: true
      
    # Object has required keys
    - key: "result.metadata"
      validation:
        type: "object_keys"
        keys: ["id", "name", "timestamp"]  # Required keys
        strict: false                   # Allow additional keys
      required: true
      
    # Object key count
    - key: "result.data"
      validation:
        type: "object_size"
        min: 1                          # Minimum number of keys
        max: 50                         # Maximum number of keys
      required: true
```

#### Boolean and Null Validation

```yaml
expected:
  patterns:
    # Boolean value
    - key: "result.enabled"
      validation:
        type: "boolean"
        value: true                     # Expected boolean value
      required: true
      
    # Null value
    - key: "result.optional_field"
      validation: { type: "null" }
      required: false
      
    # Not null
    - key: "result.required_field"
      validation: { type: "not_null" }
      required: true
```

### Custom Validation Scripts

```yaml
expected:
  custom_scripts:
    - script: "scripts/validate_response.py"  # Path to validation script
      timeout_seconds: 10               # Script execution timeout
      
      # Script parameters
      params:
        threshold: 0.8
        strict_mode: true
        
      # Expected script output
      expected_exit_code: 0             # Expected exit code (0 = success)
      min_score: 0.7                    # Minimum validation score
      
    - script: "scripts/security_check.js"
      engine: "node"                    # Script interpreter
      working_dir: "scripts/"           # Script working directory
      env:                              # Environment variables for script
        CHECK_MODE: "strict"
```

### Validation Options

```yaml
expected:
  # Global validation settings
  allow_extra_fields: true              # Allow additional fields in response
  strict_mode: false                    # Strict validation (fail on warnings)
  ignore_order: true                    # Ignore array element order
  case_sensitive: true                  # Case-sensitive string comparisons
  
  # Error handling
  continue_on_validation_error: true    # Continue with other validations
  max_validation_errors: 10             # Maximum validation errors before stopping
  
  # Transformation options
  normalize_whitespace: true            # Normalize whitespace in strings
  trim_strings: true                    # Trim leading/trailing whitespace
  convert_types: false                  # Auto-convert compatible types
```

## 📊 Performance Configuration

Performance monitoring tracks execution metrics and compares against baselines.

### Basic Performance Settings

```yaml
performance:
  # Monitoring control
  enable_monitoring: true               # Enable performance monitoring
  collect_detailed_metrics: true       # Collect detailed timing data
  
  # Storage configuration
  baseline_storage_path: "baselines/"  # Directory for baseline data
  metrics_storage_path: "metrics/"     # Directory for metrics data
  retention_days: 365                  # Data retention period
  
  # Baseline management
  auto_establish_baselines: false       # Automatically create baselines
  auto_update_baselines: false          # Update baselines on improvements
  baseline_confidence_threshold: 0.95   # Statistical confidence for baselines
  baseline_sample_size: 20              # Minimum samples for baseline
```

### Performance Requirements

```yaml
# Test-level performance requirements
performance:
  max_execution_time_ms: 5000           # Maximum response time (required)
  max_memory_usage_mb: 128              # Maximum memory usage
  max_cpu_usage_percent: 80             # Maximum CPU usage
  max_disk_io_mb: 10                    # Maximum disk I/O
  max_network_requests: 5               # Maximum network requests
  
  # Percentile requirements
  percentiles:
    p50: 1000                           # 50th percentile <= 1s
    p90: 3000                           # 90th percentile <= 3s
    p95: 4000                           # 95th percentile <= 4s
    p99: 5000                           # 99th percentile <= 5s
  
  # Throughput requirements
  min_requests_per_second: 10           # Minimum throughput
  max_requests_per_second: 100          # Maximum throughput (for load testing)
```

### Regression Detection

```yaml
performance:
  regression_detection:
    enabled: true                       # Enable regression detection
    
    # Thresholds
    warning_threshold_percent: 25       # Warning if 25% slower than baseline
    error_threshold_percent: 50         # Error if 50% slower than baseline
    critical_threshold_percent: 100     # Critical if 100% slower than baseline
    
    # Statistical settings
    confidence_level: 0.95              # Statistical confidence level
    minimum_samples: 5                  # Minimum samples for comparison
    outlier_detection: true             # Remove statistical outliers
    
    # Actions
    fail_on_regression: false           # Fail test on performance regression
    alert_on_regression: true           # Send alerts on regression
    create_regression_report: true      # Generate regression analysis report
```

### Benchmarking Configuration

```yaml
performance:
  benchmarking:
    enabled: true                       # Enable benchmarking mode
    
    # Benchmark parameters
    warmup_iterations: 5                # Warmup runs before benchmarking
    benchmark_iterations: 100           # Number of benchmark iterations
    max_benchmark_duration_seconds: 300 # Maximum benchmark duration
    
    # Statistical analysis
    remove_outliers: true               # Remove statistical outliers
    outlier_threshold: 2.0              # Standard deviations for outlier detection
    
    # Comparison
    compare_against_baseline: true      # Compare results to baseline
    establish_new_baseline: false       # Create new baseline from results
    baseline_update_threshold: 0.1      # Update baseline if 10% improvement
```

## 📊 Reporting Configuration

The reporting system generates comprehensive test reports in multiple formats.

### Basic Reporting Settings

```yaml
reporting:
  # Output configuration
  output_dir: "test-reports"            # Report output directory
  file_prefix: "test-report"            # Report file name prefix
  timestamp_format: "%Y%m%d_%H%M%S"     # Timestamp format for file names
  
  # Report formats
  formats: ["html", "json", "junit"]    # Output formats
  default_format: "html"                # Default format when not specified
  
  # Content control
  include_debug_info: false             # Include debug information
  include_server_logs: true             # Include server log excerpts
  include_request_response: true        # Include full request/response data
  include_environment_info: true        # Include system environment info
  
  # Report behavior
  open_html: false                      # Automatically open HTML reports
  overwrite_existing: true              # Overwrite existing reports
```

### Advanced Reporting Features

```yaml
reporting:
  # Report distribution
  distribution:
    email:
      enabled: true                     # Enable email reports
      smtp_server: "smtp.company.com"   # SMTP server
      smtp_port: 587                    # SMTP port
      username: "${SMTP_USERNAME}"      # SMTP username
      password: "${SMTP_PASSWORD}"      # SMTP password
      from_address: "tests@company.com" # From email address
      to_addresses: ["team@company.com"] # Recipient addresses
      
      # Email content
      subject_template: "MCP Test Results - ${date}"
      send_on_success: false            # Send email on successful tests
      send_on_failure: true             # Send email on test failures
      send_summary: true                # Send daily/weekly summaries
      
    slack:
      enabled: true                     # Enable Slack notifications
      webhook_url: "${SLACK_WEBHOOK_URL}" # Slack webhook URL
      channel: "#test-reports"          # Slack channel
      username: "MCP Test Bot"          # Bot username
      
      # Notification triggers
      notify_on_failure: true           # Notify on test failures
      notify_on_success: false          # Notify on successful tests
      notify_on_regression: true        # Notify on performance regressions
      
      # Message formatting
      include_summary: true             # Include test summary
      include_failures: true            # Include failure details
      max_message_length: 2000          # Maximum message length
  
  # Charts and visualizations
  charts:
    enabled: true                       # Enable chart generation
    
    # Chart types
    types:
      - "response_time"                 # Response time charts
      - "success_rate"                  # Success rate over time
      - "error_distribution"            # Error type distribution
      - "performance_trends"            # Performance trend analysis
      
    # Chart configuration
    size:
      width: 800                        # Chart width in pixels
      height: 400                       # Chart height in pixels
    theme: "light"                      # Chart theme: light, dark
    format: "png"                       # Chart format: png, svg, pdf
    
    # Data settings
    max_data_points: 100                # Maximum data points per chart
    aggregation_interval: "1h"          # Data aggregation interval
```

### Custom Report Templates

```yaml
reporting:
  # Template configuration
  custom_templates:
    html:
      template_path: "templates/custom-report.html"
      css_path: "templates/custom-styles.css"
      js_path: "templates/custom-scripts.js"
      
    json:
      template_path: "templates/custom-report.json"
      schema_validation: true           # Validate against JSON schema
      
  # Template variables
  template_variables:
    company_name: "Acme Corp"
    report_title: "MCP Server Validation"
    contact_email: "support@acme.com"
    custom_logo: "assets/logo.png"
```

## 🌍 Environment Configuration

Environment settings control the execution context and system integration.

### Basic Environment Settings

```yaml
environment:
  # Environment variables
  variables:
    TEST_ENV: "automated"               # Custom environment variables
    DEBUG_MODE: "false"
    API_BASE_URL: "https://api.example.com"
    DATABASE_URL: "${SECRET_DATABASE_URL}"  # Secret substitution
    
  # Path configuration
  path_additions:                       # Additional PATH directories
    - "/usr/local/bin"
    - "./scripts"
    - "${HOME}/.local/bin"
    
  # Working directory
  working_directory: "."                # Base working directory
  temp_directory: "/tmp/mcp-tests"      # Temporary file directory
  
  # File system
  create_temp_dirs: true                # Create temporary directories
  cleanup_temp_files: true              # Clean up temporary files
```

### Resource Limits

```yaml
environment:
  limits:
    # Memory limits
    max_memory_mb: 1024                 # Maximum memory per test
    max_swap_mb: 512                    # Maximum swap usage
    
    # CPU limits  
    max_cpu_seconds: 300                # Maximum CPU time per test
    max_cpu_percent: 80                 # Maximum CPU usage percentage
    
    # I/O limits
    max_disk_usage_mb: 256              # Maximum disk usage
    max_network_bandwidth_mbps: 10      # Maximum network bandwidth
    max_open_files: 256                 # Maximum open file descriptors
    max_processes: 16                   # Maximum child processes
    
    # Time limits
    max_wall_time_seconds: 600          # Maximum wall clock time
    max_idle_time_seconds: 60           # Maximum idle time
```

### System Integration

```yaml
environment:
  # Container integration
  container:
    enabled: false                      # Run tests in containers
    image: "ubuntu:22.04"               # Container image
    network: "bridge"                   # Container network mode
    volumes: ["/tmp:/tmp"]              # Volume mounts
    
  # Cloud integration
  cloud:
    provider: "aws"                     # Cloud provider: aws, gcp, azure
    region: "us-west-2"                 # Cloud region
    instance_type: "t3.medium"          # Instance type for cloud execution
    
  # Monitoring integration
  monitoring:
    prometheus_pushgateway: "http://localhost:9091"
    jaeger_endpoint: "http://localhost:14268"
    datadog_api_key: "${DATADOG_API_KEY}"
```

## 🔐 Security Configuration

Security settings control authentication, authorization, and compliance features.

### Basic Security Settings

```yaml
security:
  # Audit logging
  enable_audit_logging: true            # Enable security audit logs
  audit_log_path: "logs/audit.log"     # Audit log file path
  audit_log_format: "json"             # Audit log format: json, text
  audit_log_rotation: true             # Enable log rotation
  audit_retention_days: 90              # Audit log retention period
  
  # Authentication
  authentication:
    enabled: true                       # Enable authentication
    method: "token"                     # Auth method: token, oauth, cert
    token_header: "Authorization"       # Token header name
    token_prefix: "Bearer "             # Token prefix
    
  # Data protection
  data_protection:
    encrypt_sensitive_data: true        # Encrypt sensitive test data
    hash_pii: true                      # Hash personally identifiable information
    mask_secrets: true                  # Mask secrets in logs and reports
    
    # Encryption settings
    encryption_algorithm: "AES-256-GCM" # Encryption algorithm
    key_derivation: "PBKDF2"            # Key derivation function
    
  # Access control
  access_control:
    enable_rbac: true                   # Enable role-based access control
    rbac_config_path: "config/rbac.yaml" # RBAC configuration file
    
    # Default permissions
    default_permissions: ["test:read"]  # Default user permissions
    admin_permissions: ["*"]            # Administrator permissions
```

### Compliance Settings

```yaml
security:
  compliance:
    # Standards compliance
    gdpr_compliance: true               # GDPR compliance mode
    hipaa_compliance: false             # HIPAA compliance mode
    sox_compliance: false               # SOX compliance mode
    
    # Data handling
    data_classification: true           # Classify data sensitivity
    data_retention_policy: true         # Enforce data retention policies
    
    # Validation requirements
    require_security_tests: true        # Require security test validation
    security_test_suites: ["security-basic", "auth-tests"]
    
    # Reporting requirements
    generate_compliance_report: true    # Generate compliance reports
    compliance_report_format: "pdf"     # Compliance report format
```

### Vulnerability Scanning

```yaml
security:
  vulnerability_scanning:
    enabled: true                       # Enable vulnerability scanning
    
    # Scanner configuration
    scanner: "custom"                   # Scanner type: custom, owasp-zap
    scanner_timeout_seconds: 300        # Scanner timeout
    
    # Scan targets
    scan_inputs: true                   # Scan input parameters
    scan_outputs: true                  # Scan output responses
    scan_network: false                 # Scan network communications
    
    # Vulnerability database
    vuln_db_path: "db/vulnerabilities.json"
    update_vuln_db: true                # Update vulnerability database
    
    # Severity thresholds
    fail_on_critical: true              # Fail tests on critical vulnerabilities
    fail_on_high: false                 # Fail tests on high severity issues
    max_medium_vulnerabilities: 5       # Maximum medium severity issues
```

## 📈 Monitoring Configuration

Monitoring settings enable integration with external monitoring and alerting systems.

### Basic Monitoring Settings

```yaml
monitoring:
  enabled: true                         # Enable monitoring integration
  
  # Metrics collection
  metrics:
    enabled: true                       # Enable metrics collection
    port: 9090                          # Metrics server port
    path: "/metrics"                    # Metrics endpoint path
    format: "prometheus"                # Metrics format: prometheus, json
    
    # Collection settings
    collection_interval_seconds: 10     # Metrics collection interval
    retention_hours: 168                # Metrics retention (1 week)
    
  # Health monitoring  
  health:
    enabled: true                       # Enable health monitoring
    port: 8080                          # Health check port
    path: "/health"                     # Health check endpoint
    timeout_seconds: 5                  # Health check timeout
```

### Prometheus Integration

```yaml
monitoring:
  prometheus:
    enabled: true                       # Enable Prometheus integration
    
    # Push Gateway
    push_gateway:
      enabled: true                     # Enable push gateway
      url: "http://localhost:9091"      # Push gateway URL
      job_name: "mcp-test-harness"      # Prometheus job name
      push_interval_seconds: 30         # Push interval
      
    # Custom metrics
    custom_metrics:
      - name: "test_duration_seconds"   # Metric name
        type: "histogram"               # Metric type: counter, gauge, histogram
        help: "Test execution duration"  # Metric description
        labels: ["test_name", "status"] # Metric labels
        
      - name: "test_success_total"
        type: "counter"
        help: "Total successful tests"
        labels: ["suite_name"]
```

### Alerting Configuration

```yaml
monitoring:
  alerting:
    enabled: true                       # Enable alerting
    
    # Alert rules
    rules:
      - name: "high_failure_rate"       # Alert rule name
        condition: "test_failure_rate > 0.1"  # Alert condition
        duration: "5m"                  # Alert duration threshold
        severity: "warning"             # Alert severity
        description: "Test failure rate is above 10%"
        
        # Alert actions
        actions:
          - type: "email"               # Action type
            recipients: ["team@company.com"]
            
          - type: "webhook"             # Webhook action
            url: "https://hooks.slack.com/..."
            
      - name: "performance_regression"
        condition: "test_response_time_p95 > baseline_p95 * 1.5"
        duration: "10m"
        severity: "critical"
        description: "Performance regression detected"
    
    # Alert channels
    channels:
      email:
        smtp_server: "smtp.company.com"
        smtp_port: 587
        username: "${ALERT_EMAIL_USER}"
        password: "${ALERT_EMAIL_PASS}"
        
      slack:
        webhook_url: "${SLACK_ALERT_WEBHOOK}"
        channel: "#alerts"
        username: "MCP Alert Bot"
```

## 🔧 Advanced Configuration

### Configuration Validation

```yaml
# Schema validation settings
validation:
  strict_schema: false                  # Strict YAML schema validation
  validate_references: true            # Validate internal references
  check_file_paths: true               # Validate file path references
  warn_on_unknown_fields: true         # Warn about unknown configuration fields
```

### Configuration Inheritance

```yaml
# Base configuration
extends: "config/base.yaml"             # Inherit from base configuration

# Override specific settings
global:
  environment: "production"             # Override environment
  
# Merge arrays (default is replace)
test_suites:
  _merge_mode: "append"                 # Append to inherited test suites
  - name: "production-specific-tests"
    # ... additional test suite
```

### Environment-Specific Configuration

```yaml
# Multi-environment configuration
environments:
  development:
    global:
      log_level: "debug"
      fail_fast: true
    server:
      startup_timeout_seconds: 60
      
  staging:
    global:
      log_level: "info"
      max_global_concurrency: 2
    performance:
      regression_detection:
        warning_threshold_percent: 50
        
  production:
    global:
      log_level: "warn"
      max_global_concurrency: 8
    security:
      enable_audit_logging: true
    monitoring:
      enabled: true
```

### Variable Substitution

```yaml
# Variable definitions
variables:
  project_root: "/home/user/projects"
  test_timeout: 30
  api_version: "v1"

# Environment variable substitution
server:
  start_command: "${MCP_SERVER_COMMAND}"
  env:
    API_KEY: "${SECRET_API_KEY}"        # Environment variable
    PROJECT_PATH: "${project_root}"     # Configuration variable
    
# Complex substitution
test_cases:
  - id: "test_${api_version}_endpoint"
    input_params:
      timeout: "${test_timeout}"
      base_url: "${API_BASE_URL:-http://localhost:3000}"  # Default value
```

### Conditional Configuration

```yaml
# Conditional sections
test_suites:
  - name: "performance-tests"
    enabled: "${ENVIRONMENT != 'development'}"  # Skip in development
    
  - name: "security-tests"
    skip_if: "${SKIP_SECURITY_TESTS}"  # Skip if environment variable is set
    only_if: "${SECURITY_ENABLED}"     # Only run if security is enabled
```

## 📋 Configuration Validation and Troubleshooting

### Validation Commands

```bash
# Validate configuration syntax
mcp-test-harness validate --config my-config.yaml

# Comprehensive validation
mcp-test-harness validate --comprehensive --config my-config.yaml

# Check server connectivity
mcp-test-harness validate --connectivity --config my-config.yaml

# Schema validation only
mcp-test-harness validate --schema-only --config my-config.yaml
```

### Common Configuration Errors

**Invalid YAML Syntax**
```
❌ Configuration Error: Invalid YAML syntax at line 15
   Error: mapping values are not allowed here
   Fix: Check indentation and YAML structure
```

**Missing Required Fields**
```
❌ Configuration Error: Missing required field 'server.start_command'
   Fix: Add server startup command to configuration
```

**Invalid Field Values**
```
❌ Configuration Error: Invalid value for 'global.max_global_concurrency': 0
   Fix: Value must be between 1 and 32
```

**File Not Found**
```
❌ Configuration Error: Script file not found: 'scripts/setup.py'
   Fix: Ensure script file exists and path is correct
```

### Configuration Best Practices

1. **Version Control**: Keep configurations in version control
2. **Environment Separation**: Use separate configs for different environments
3. **Secret Management**: Use environment variables for sensitive data
4. **Validation**: Always validate configs before deployment
5. **Documentation**: Comment complex configuration sections
6. **Modularity**: Use configuration inheritance for shared settings

---

## 📚 Additional Resources

- [User Guide](user-guide.md) - Complete user manual
- [Production Deployment](production-deployment.md) - Enterprise deployment guide
- [Developer Guide](developer-guide.md) - Extending the test harness
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions

**Last Updated**: 2025-01-07  
**Version**: 1.0.0 