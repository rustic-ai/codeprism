# Your First Test

A step-by-step guide to running your first MCP server test with the MCP Test Harness.

## 🎯 What You'll Learn

By the end of this guide, you will:
- Understand the basic workflow of testing MCP servers
- Create your first test configuration
- Run a complete test suite
- Interpret test results
- Know how to modify tests for your own MCP server

## 📋 Prerequisites

Before starting, ensure you have:
- ✅ Installed the MCP Test Harness ([Installation Guide](installation.md))
- ✅ A working MCP server to test (or use our sample server)
- ✅ Basic understanding of YAML configuration files

## 🚀 Quick Start (5 Minutes)

### Step 1: Create Your First Configuration

Create a file called `my-first-test.yaml`:

```yaml
# My First MCP Test Configuration
global:
  max_global_concurrency: 2
  global_timeout_seconds: 30
  fail_fast: false

server:
  # Simple echo server for demonstration
  transport: "stdio"
  start_command: "node"
  args: ["-e", "console.log(JSON.stringify({result: 'Hello from MCP server!'}))"]
  startup_timeout_seconds: 5

test_suites:
  - name: "first_test_suite"
    description: "My very first MCP test"
    
    test_cases:
      - id: "basic_hello"
        description: "Test basic server response"
        tool_name: "hello"
        input_params:
          message: "Hello, World!"
        
        expected:
          patterns:
            - key: "result"
              validation: { type: "exists" }
              required: true
          allow_extra_fields: true
```

### Step 2: Run Your First Test

```bash
# Run the test with verbose output
mcp-test-harness test --config my-first-test.yaml --verbose
```

### Step 3: Check the Results

You should see output similar to:

```
🧪 Starting MCP Test Harness v1.0.0
================================================================================

🔧 Configuration loaded: my-first-test.yaml
📊 Test suites: 1, Total tests: 1

🚀 Starting server: node -e console.log(JSON.stringify({result: 'Hello from MCP server!'}))
✅ Server started successfully (PID: 12345)

🧪 Running Test Suite: first_test_suite
================================================================================

✅ PASS: basic_hello
   Duration: 142ms
   Description: Test basic server response
   Server Response: {"result": "Hello from MCP server!"}

================================================================================
Suite Summary: first_test_suite
- Total Tests: 1
- Passed: 1 ✅
- Failed: 0 ❌
- Skipped: 0 ⚠️
- Success Rate: 100.0%
- Total Duration: 142ms
================================================================================

🎉 All tests completed successfully!
```

**Congratulations!** 🎉 You've just run your first MCP test!

## 📊 Understanding What Happened

Let's break down what just occurred:

### 1. Configuration Loading
The test harness loaded your YAML configuration and validated it against the schema.

### 2. Server Startup
A new MCP server process was started using the specified command:
```bash
node -e "console.log(JSON.stringify({result: 'Hello from MCP server!'}))"
```

### 3. Test Execution
The test harness sent a request to the server and validated the response.

### 4. Result Validation
The response was checked against your validation patterns:
- ✅ The `result` field exists
- ✅ No required fields were missing

## 🛠️ Testing a Real MCP Server

Now let's test a more realistic MCP server. We'll use a simple file system server as an example.

### Step 1: Download Sample MCP Server

```bash
# Clone the sample servers repository
git clone https://github.com/modelcontextprotocol/servers.git mcp-servers
cd mcp-servers/src/filesystem

# Install dependencies
npm install

# Test the server manually (optional)
echo '{"method": "tools/list", "params": {}}' | node dist/index.js stdio
```

### Step 2: Create Real Server Configuration

Create `filesystem-test.yaml`:

```yaml
global:
  max_global_concurrency: 3
  global_timeout_seconds: 60
  default_project_path: "/tmp/test-project"
  
  retry:
    max_retries: 2
    retry_delay_ms: 1000

server:
  transport: "stdio"
  start_command: "node"
  args: ["dist/index.js", "stdio"]
  working_dir: "mcp-servers/src/filesystem"
  
  env:
    NODE_ENV: "test"
  
  startup_timeout_seconds: 10
  shutdown_timeout_seconds: 5

# Create test directory structure first
setup_commands:
  - "mkdir -p /tmp/test-project"
  - "echo 'Hello World' > /tmp/test-project/hello.txt"
  - "echo 'Test file content' > /tmp/test-project/test.txt"

test_suites:
  - name: "filesystem_capabilities"
    description: "Test filesystem MCP server capabilities"
    
    test_cases:
      - id: "list_tools"
        description: "List available tools"
        tool_name: "tools/list"
        input_params: {}
        
        expected:
          patterns:
            - key: "tools"
              validation: { type: "array" }
              required: true
            - key: "tools[0].name"
              validation: { type: "exists" }
              required: true
          
          custom_scripts:
            - script: |
                import json
                import sys
                
                response = json.loads(sys.argv[1])
                tools = response.get('tools', [])
                
                # Check for expected filesystem tools
                tool_names = [tool['name'] for tool in tools]
                expected_tools = ['read_file', 'write_file', 'list_directory']
                
                for tool in expected_tools:
                    if tool not in tool_names:
                        print(f"Missing expected tool: {tool}")
                        sys.exit(1)
                
                print("All expected tools found")
              language: "python"

      - id: "read_existing_file"
        description: "Read an existing file"
        tool_name: "read_file"
        input_params:
          path: "/tmp/test-project/hello.txt"
        
        expected:
          patterns:
            - key: "content"
              validation: { type: "string" }
              required: true
            - key: "content"
              validation: { type: "contains", value: "Hello World" }
              required: true

      - id: "list_directory"
        description: "List directory contents"
        tool_name: "list_directory"
        input_params:
          path: "/tmp/test-project"
        
        expected:
          patterns:
            - key: "files"
              validation: { type: "array" }
              required: true
          
          custom_scripts:
            - script: |
                import json
                import sys
                
                response = json.loads(sys.argv[1])
                files = response.get('files', [])
                filenames = [f['name'] for f in files]
                
                if 'hello.txt' not in filenames:
                    print("hello.txt not found in directory listing")
                    sys.exit(1)
                    
                print(f"Found {len(files)} files in directory")
              language: "python"

      - id: "write_new_file"
        description: "Write a new file"
        tool_name: "write_file"
        input_params:
          path: "/tmp/test-project/new-file.txt"
          content: "This is a test file created by MCP Test Harness"
        
        expected:
          patterns:
            - key: "success"
              validation: { type: "equals", value: true }
              required: true

# Cleanup after tests
teardown_commands:
  - "rm -rf /tmp/test-project"

reporting:
  output_dir: "test-reports"
  formats: ["html", "json"]
  include_debug_info: true
```

### Step 3: Run the Real Test

```bash
# Run the filesystem server test
mcp-test-harness test --config filesystem-test.yaml --verbose
```

### Expected Output

```
🧪 Starting MCP Test Harness v1.0.0
================================================================================

🔧 Configuration loaded: filesystem-test.yaml
📊 Test suites: 1, Total tests: 4

🛠️  Running setup commands...
✅ Setup completed: 3 commands executed

🚀 Starting server: node dist/index.js stdio
✅ Server started successfully (PID: 23456)

🧪 Running Test Suite: filesystem_capabilities
================================================================================

✅ PASS: list_tools
   Duration: 234ms
   Description: List available tools
   Validation: ✅ Expected tools found: read_file, write_file, list_directory

✅ PASS: read_existing_file  
   Duration: 89ms
   Description: Read an existing file
   Server Response: {"content": "Hello World\n"}

✅ PASS: list_directory
   Duration: 156ms
   Description: List directory contents
   Custom Script: Found 2 files in directory

✅ PASS: write_new_file
   Duration: 203ms
   Description: Write a new file
   Server Response: {"success": true}

================================================================================
Suite Summary: filesystem_capabilities
- Total Tests: 4
- Passed: 4 ✅
- Failed: 0 ❌
- Skipped: 0 ⚠️
- Success Rate: 100.0%
- Total Duration: 682ms
================================================================================

🧹 Running teardown commands...
✅ Teardown completed: 1 command executed

📊 Report generated: test-reports/report.html
🎉 All tests completed successfully!
```

## 🔍 Exploring Test Results

### HTML Report

Open the generated HTML report in your browser:

```bash
# Open the report
open test-reports/report.html  # macOS
xdg-open test-reports/report.html  # Linux
start test-reports/report.html  # Windows
```

The HTML report includes:
- **Executive Summary**: High-level test results
- **Detailed Results**: Test-by-test breakdown
- **Performance Charts**: Response time visualization
- **Server Logs**: Complete server interaction logs
- **Configuration**: The exact configuration used

### JSON Report

For programmatic access, check the JSON report:

```bash
# Pretty-print the JSON report
cat test-reports/report.json | python -m json.tool
```

## 🎛️ Common Test Patterns

### Pattern 1: Capability Testing

```yaml
test_cases:
  - id: "check_capabilities"
    tool_name: "initialize"
    input_params:
      protocolVersion: "2024-11-05"
    expected:
      patterns:
        - key: "capabilities.tools"
          validation: { type: "equals", value: true }
        - key: "capabilities.resources"
          validation: { type: "boolean" }
```

### Pattern 2: Error Handling

```yaml
test_cases:
  - id: "test_invalid_input"
    tool_name: "read_file"
    input_params:
      path: "/nonexistent/file.txt"
    expected:
      error_expected: true
      patterns:
        - key: "error.code"
          validation: { type: "one_of", values: ["FILE_NOT_FOUND", "ENOENT"] }
```

### Pattern 3: Performance Testing

```yaml
test_cases:
  - id: "performance_test"
    tool_name: "large_operation"
    input_params: {}
    performance:
      max_execution_time_ms: 5000
      max_memory_usage_mb: 128
    expected:
      patterns:
        - key: "result"
          validation: { type: "exists" }
```

## 🐛 Troubleshooting Common Issues

### Issue 1: Server Won't Start

**Error:**
```
❌ Server startup failed: Command not found
```

**Solution:**
```yaml
# Check your server command
server:
  start_command: "which node && node"  # Verify node exists
  args: ["--version"]  # Test with simple command first
```

### Issue 2: Tests Timing Out

**Error:**
```
❌ TIMEOUT: test_name (30000ms)
```

**Solution:**
```yaml
# Increase timeouts
global:
  global_timeout_seconds: 120

test_cases:
  - id: "slow_test"
    timeout_override_seconds: 60  # Per-test timeout
```

### Issue 3: Validation Failures

**Error:**
```
❌ FAIL: Pattern validation failed for key 'result'
```

**Solution:**
```yaml
# Use allow_extra_fields and debug
expected:
  allow_extra_fields: true
  debug_response: true  # Shows full response
  patterns:
    - key: "result"
      validation: { type: "exists" }
      required: false  # Make optional for debugging
```

## 📚 Next Steps

Now that you've run your first tests, explore these topics:

### Immediate Next Steps
1. **[Basic Configuration](basic-configuration.md)** - Learn configuration fundamentals
2. **[Configuration Reference](../configuration-reference.md)** - Complete YAML schema
3. **[Example Configurations](../examples/)** - Real-world examples

### Advanced Topics
4. **[Testing Different Transports](../testing-guides/transport-testing.md)** - HTTP, WebSocket testing
5. **[Custom Validation Scripts](../testing-guides/custom-validation.md)** - Write your own validators
6. **[Performance Testing](../testing-guides/performance-testing.md)** - Benchmarking and baselines

### Production Usage
7. **[CI/CD Integration](../cicd-integration.md)** - Automate testing
8. **[Production Deployment](../production-deployment.md)** - Enterprise setup
9. **[Monitoring & Alerting](../monitoring-alerting.md)** - Production monitoring

## 🎯 Key Takeaways

From this guide, you learned:

✅ **Basic Workflow**: Configuration → Server Start → Test Execution → Results  
✅ **Configuration Structure**: Global settings, server config, test suites  
✅ **Validation Patterns**: Field checking, type validation, custom scripts  
✅ **Result Interpretation**: Understanding PASS/FAIL/SKIP status  
✅ **Troubleshooting**: Common issues and solutions  

## 🆘 Getting Help

If you encounter issues:

1. **Check the logs**: Use `--verbose` flag for detailed output
2. **Validate configuration**: Use `mcp-test-harness validate --config your-config.yaml`
3. **Test manually**: Try running your MCP server directly first
4. **Read troubleshooting**: Check [Troubleshooting Guide](../troubleshooting.md)
5. **Ask for help**: Create an issue on [GitHub](https://github.com/rustic-ai/codeprism/issues)

---

**Congratulations!** 🎉 You're now ready to test any MCP server with confidence. 