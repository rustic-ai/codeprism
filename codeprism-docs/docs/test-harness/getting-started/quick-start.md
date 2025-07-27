---
title: Quick Start Guide
description: Get up and running with Mandrel MCP Test Harness in under 5 minutes
sidebar_position: 1
---

# Quick Start Guide - Mandrel MCP Test Harness

Get up and running with Mandrel in under 5 minutes! This guide will walk you through installing the test harness and running your first MCP server test using our verified working examples.

## 🚀 Prerequisites

- **Rust 1.70+** - Download from [rustup.rs](https://rustup.rs/)
- **Node.js 18+** - For running example MCP servers
- **Git** - For cloning the repository
- **Linux/macOS** - Required for filesystem examples

## 📦 Installation

### Option 1: Build from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Build and run the moth binary
cargo build --release --bin moth

# Verify installation
./target/release/moth --version
```

### Option 2: Development Build

```bash
# In the codeprism directory
cd crates/mandrel-mcp-th
cargo build

# Binary will be available at ../../target/debug/moth
../../target/debug/moth --version
```

## 🎯 Your First Test - Filesystem Server

Let's start with a real, working example that we've verified to achieve 100% success rate.

### Step 1: Set Up Test Environment

```bash
# Create sandbox directory for filesystem tests
mkdir -p /tmp/mcp-test-sandbox
echo "Hello, MCP Test Harness!" > /tmp/mcp-test-sandbox/test.txt
```

### Step 2: Run Verified Filesystem Test

```bash
# Navigate to the project root
cd /path/to/codeprism

# Run the verified filesystem server test
cargo run --bin moth -- run codeprism-docs/docs/test-harness/examples/filesystem-server.yaml

# Or with verbose output for more details
cargo run --bin moth -- -v run codeprism-docs/docs/test-harness/examples/filesystem-server.yaml
```

**Expected Output:**
```
📝 Generating comprehensive test reports...
  📄 Generated json report: ./reports/test_report.json
  📄 Generated html report: ./reports/test_report.html
  📄 Generated junit report: ./reports/test_report.xml

✅ Test Suite Finished ✅
Suite: Filesystem MCP Server (MCP-Compliant)
Total Tests: 8, Passed: 8, Failed: 0
Duration: 2.3s
```

### Step 3: Explore the Test Results

```bash
# View the JSON results
cat ./reports/test_report.json

# Open the HTML report in your browser (if available)
open ./reports/test_report.html
```

## 🌟 Advanced Example - Everything Server

Once you've verified the filesystem test works, try our comprehensive "everything server" test:

### Step 1: Run Everything Server Test

```bash
# Run the verified everything server test (100% success rate)
cargo run --bin moth -- run codeprism-docs/docs/test-harness/examples/everything-server.yaml

# With output customization
cargo run --bin moth -- run codeprism-docs/docs/test-harness/examples/everything-server.yaml --output ./my-test-results
```

**Expected Output:**
```
✅ Test Suite Finished ✅
Suite: Everything MCP Server (Working Tests)
Total Tests: 8, Passed: 8, Failed: 0
Duration: 10.02s
Success Rate: 100%
```

### Step 2: Understand What Was Tested

The everything server test validates:
- **Mathematical operations** (0-1ms response time)
- **Text processing with Unicode** (0-1ms response time)
- **Environment variable access** (1ms response time)
- **Long-running operations** (10+ seconds)
- **Resource management** (basic resource access)
- **Error handling** (invalid resource IDs)

## 🔍 Validate Before Running

Always validate your test specifications before running:

```bash
# Validate a test specification
cargo run --bin moth -- validate codeprism-docs/docs/test-harness/examples/filesystem-server.yaml

# Comprehensive validation with detailed output
cargo run --bin moth -- validate --detailed --check-all codeprism-docs/docs/test-harness/examples/everything-server.yaml

# Generate validation report
cargo run --bin moth -- validate --formats html everything-server.yaml --output ./validation-reports
```

## 📝 Create Your First Test Specification

Now let's create a simple test for your own MCP server:

### Step 1: Create a Basic Test File

Create `my-server-test.yaml`:

```yaml
name: "My First MCP Server Test"
version: "1.0.0"
description: "Basic test for my MCP server"

# Be honest about capabilities - only claim what your server actually supports
capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false

# Configure your server startup
server:
  command: "node"                    # Your server command
  args: ["my-mcp-server.js"]        # Your server arguments
  env:
    NODE_ENV: "test"
  transport: "stdio"
  startup_timeout_seconds: 30

# Test your tools
tools:
  - name: "echo"                     # Use exact tool name from your server
    description: "Echo tool test"
    tests:
      - name: "basic_echo"
        description: "Test basic echo functionality"
        input:
          message: "Hello, World!"
        expected:
          error: false
          fields:
            - path: "$[0].text"      # Use actual server response format
              contains: "Hello"
              required: true
        performance:
          max_duration_ms: 1000
        tags: ["basic", "echo"]

test_config:
  timeout_seconds: 60
  max_concurrency: 2
  fail_fast: false

metadata:
  author: "Your Name"
  tags: ["custom", "basic"]
```

### Step 2: Validate Your Test

```bash
# Validate your test specification
cargo run --bin moth -- validate my-server-test.yaml --detailed

# Check for common issues
cargo run --bin moth -- validate --check-all my-server-test.yaml
```

### Step 3: Run Your Test

```bash
# Run your test
cargo run --bin moth -- run my-server-test.yaml

# Run with verbose output for debugging
cargo run --bin moth -- -v run my-server-test.yaml
```

## 🛠️ Troubleshooting Common Issues

### Server Startup Issues

**Problem:** Server fails to start
```bash
# Check if your server command is correct
node my-mcp-server.js  # Test manually

# Increase startup timeout
server:
  startup_timeout_seconds: 60  # Increase from 30
```

**Problem:** Transport connection failed
```bash
# Verify your server supports stdio transport
# Check server output for MCP protocol compliance
```

### Test Failures

**Problem:** Wrong output format
```bash
# Check actual server response by examining the JSON report
cat ./reports/test_report.json

# Common issues:
# - Using "$.result" instead of "$[0].text"
# - Wrong tool names
# - Incorrect capability claims
```

**Problem:** Timeouts
```bash
# Increase test timeouts based on actual performance
performance:
  max_duration_ms: 5000  # Increase from 1000
```

### Validation Errors

**Problem:** YAML syntax errors
```bash
# Use a YAML validator
python3 -c "
import yaml
with open('my-server-test.yaml', 'r') as f:
    yaml.safe_load(f)
print('YAML is valid')
"
```

**Problem:** JSONPath errors
```bash
# Common JSONPath issues:
# ❌ "result" (missing $ prefix)
# ✅ "$.result" (correct)
# ❌ "$[0].text" for object responses
# ✅ "$.content" for object responses
```

## 📊 Understanding Test Reports

### JSON Report Structure
```json
{
  "suite_name": "My First MCP Server Test",
  "total_tests": 1,
  "passed": 1,
  "failed": 0,
  "success_rate": 100.0,
  "test_results": [
    {
      "test_name": "basic_echo",
      "status": "Passed",
      "response_time_ms": 15
    }
  ]
}
```

### HTML Report Features
- **Interactive charts** showing test results
- **Performance metrics** with response times
- **Detailed test breakdowns** with validation results
- **Error analysis** with suggested fixes
- **Filterable results** by status and category

## 🎯 Next Steps

### Explore More Examples
- Review our [verified examples](../examples/) for advanced patterns
- Study the [Configuration Reference](../configuration-reference) for complete options
- Learn about [performance testing](../performance-tuning) for optimization

### Production Usage
- Set up [CI/CD integration](../production-deployment) for automated testing
- Configure [monitoring and alerting](../production-deployment#monitoring) for production
- Implement [security testing](../configuration-reference#security-configuration) for compliance

### Community
- Check out the [User Guide](../user-guide) for comprehensive usage patterns
- Review [Troubleshooting Guide](../troubleshooting) for common issues
- Contribute to the project via [Contributing Guide](../../development/contributing)

## 🏆 Success Checklist

After completing this guide, you should have:

- ✅ **Installed and verified** the Mandrel MCP Test Harness
- ✅ **Run a verified working test** (filesystem or everything server)
- ✅ **Created your first test specification** for your own server
- ✅ **Validated and run** your custom test
- ✅ **Understood test results** and report formats
- ✅ **Learned troubleshooting** for common issues

---

**Congratulations!** You've successfully set up the MCP Test Harness and run your first tests. You're ready to create comprehensive test suites for your MCP servers! 🎉

**Need more help?** Check our [User Guide](../user-guide) for advanced usage patterns or visit our [Examples](../examples/) for more verified test specifications. 