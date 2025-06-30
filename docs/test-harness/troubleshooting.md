# Troubleshooting Guide

Comprehensive guide for diagnosing and resolving common issues with the MCP Test Harness.

## 📋 Overview

This guide covers common problems, error messages, debugging techniques, and solutions to help you quickly resolve issues with the MCP Test Harness.

## 🚨 Quick Diagnosis

### Is your issue in this category?

- **🔧 [Installation & Setup](#installation--setup-issues)** - Problems installing or setting up the test harness
- **⚙️ [Configuration](#configuration-issues)** - YAML syntax, missing fields, invalid values
- **🖥️ [Server Connection](#server-connection-issues)** - Cannot connect to or start MCP server
- **🧪 [Test Execution](#test-execution-issues)** - Tests failing, timeouts, validation errors
- **📊 [Performance](#performance-issues)** - Slow execution, memory issues, resource limits
- **🔐 [Security](#security-issues)** - Authentication, authorization, compliance problems
- **📈 [Monitoring](#monitoring-issues)** - Metrics collection, alerting, reporting problems

### Emergency Checklist

If tests are completely failing, check these first:

1. **Basic connectivity**: Can you connect to your MCP server manually?
2. **Configuration syntax**: Is your YAML configuration valid?
3. **Required fields**: Are all required configuration fields present?
4. **File paths**: Do all referenced files and directories exist?
5. **Permissions**: Does the test harness have necessary file/network permissions?
6. **Resource availability**: Is there enough memory/disk/network capacity?

## 🔧 Installation & Setup Issues

### Issue: Command Not Found

**Symptoms:**
```bash
$ mcp-test-harness --version
-bash: mcp-test-harness: command not found
```

**Solutions:**

1. **Verify Installation**:
```bash
# Check if binary exists
which mcp-test-harness
ls -la /usr/local/bin/mcp-test-harness

# If not found, reinstall
curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux-x86_64.tar.gz | tar xz
sudo mv mcp-test-harness /usr/local/bin/
sudo chmod +x /usr/local/bin/mcp-test-harness
```

2. **Check PATH**:
```bash
# Add to PATH if needed
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

3. **Use Full Path**:
```bash
# Use absolute path
/usr/local/bin/mcp-test-harness --version
```

### Issue: Permission Denied

**Symptoms:**
```bash
$ mcp-test-harness test --config my-config.yaml
Permission denied (os error 13)
```

**Solutions:**

1. **Fix Binary Permissions**:
```bash
sudo chmod +x /usr/local/bin/mcp-test-harness
```

2. **Check File Permissions**:
```bash
# Check config file permissions
ls -la my-config.yaml
chmod 644 my-config.yaml

# Check directory permissions
ls -la test-projects/
chmod 755 test-projects/
```

3. **Run with Proper User**:
```bash
# Ensure you're running as the correct user
whoami
# If needed, switch user or use sudo appropriately
```

### Issue: Missing Dependencies

**Symptoms:**
```bash
$ mcp-test-harness test --config my-config.yaml
error while loading shared libraries: libssl.so.1.1: cannot open shared object file
```

**Solutions:**

1. **Install System Dependencies**:
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config build-essential

# CentOS/RHEL
sudo yum install -y openssl-devel pkgconfig gcc

# macOS
brew install openssl pkg-config
```

2. **Update Library Cache**:
```bash
sudo ldconfig
```

## ⚙️ Configuration Issues

### Issue: Invalid YAML Syntax

**Symptoms:**
```
❌ Configuration Error: Invalid YAML syntax in test-harness.yaml at line 15
   Error: mapping values are not allowed here
```

**Solutions:**

1. **Check YAML Indentation**:
```yaml
# ❌ Incorrect indentation
global:
max_global_concurrency: 4
  global_timeout_seconds: 300

# ✅ Correct indentation
global:
  max_global_concurrency: 4
  global_timeout_seconds: 300
```

2. **Validate YAML Online**:
   - Use online YAML validator: https://yamlchecker.com/
   - Copy your configuration and check for syntax errors

3. **Use YAML Linter**:
```bash
# Install yamllint
pip install yamllint

# Check configuration
yamllint test-harness.yaml
```

### Issue: Missing Required Fields

**Symptoms:**
```
❌ Configuration Error: Missing required field 'server.start_command'
```

**Solutions:**

1. **Add Missing Fields**:
```yaml
server:
  start_command: "cargo run --bin my-mcp-server"  # Add this line
  args: ["stdio"]
```

2. **Use Configuration Template**:
```bash
# Generate a valid template
mcp-test-harness template --minimal --output template.yaml
# Copy required fields from template
```

3. **Check Configuration Reference**:
   - See [Configuration Reference](configuration-reference.md) for all required fields

### Issue: Invalid Field Values

**Symptoms:**
```
❌ Configuration Error: Invalid value for 'global.max_global_concurrency': 0
   Expected: Integer between 1 and 32
```

**Solutions:**

1. **Fix Value Ranges**:
```yaml
global:
  max_global_concurrency: 4  # Must be 1-32
  global_timeout_seconds: 300  # Must be 1-3600
```

2. **Check Data Types**:
```yaml
# ❌ Incorrect types
global:
  max_global_concurrency: "4"    # String instead of integer
  fail_fast: "false"             # String instead of boolean

# ✅ Correct types
global:
  max_global_concurrency: 4      # Integer
  fail_fast: false               # Boolean
```

### Issue: File Path Not Found

**Symptoms:**
```
❌ Configuration Error: Script file not found: 'scripts/setup.py'
```

**Solutions:**

1. **Check File Exists**:
```bash
ls -la scripts/setup.py
```

2. **Use Correct Relative Paths**:
```yaml
# ❌ Incorrect path
setup_script: "setup.py"

# ✅ Correct path
setup_script: "scripts/setup.py"
# or use absolute path
setup_script: "/home/user/project/scripts/setup.py"
```

3. **Create Missing Files**:
```bash
mkdir -p scripts
touch scripts/setup.py
chmod +x scripts/setup.py
```

## 🖥️ Server Connection Issues

### Issue: Cannot Connect to MCP Server

**Symptoms:**
```
❌ Connection Error: Failed to connect to MCP server
   Transport: stdio
   Command: cargo run --bin my-mcp-server
   Error: No such file or directory
```

**Solutions:**

1. **Verify Server Command**:
```bash
# Test server command manually
cargo run --bin my-mcp-server stdio

# If command fails, check:
# - Is the binary name correct?
# - Is the project built?
# - Are you in the right directory?
```

2. **Check Working Directory**:
```yaml
server:
  start_command: "cargo run --bin my-mcp-server"
  working_dir: "/path/to/server/project"  # Add this
```

3. **Use Absolute Paths**:
```yaml
server:
  start_command: "/home/user/.cargo/bin/cargo"
  args: ["run", "--bin", "my-mcp-server", "stdio"]
```

### Issue: Server Startup Timeout

**Symptoms:**
```
❌ Server Error: Server startup timeout after 30 seconds
   Server command: cargo run --bin my-mcp-server
   Status: Still starting
```

**Solutions:**

1. **Increase Startup Timeout**:
```yaml
server:
  startup_timeout_seconds: 60  # Increase from default 30
```

2. **Check Server Logs**:
```bash
# Run server manually to see error messages
cargo run --bin my-mcp-server stdio
```

3. **Optimize Server Build**:
```bash
# Use release build for faster startup
cargo build --release
```

```yaml
server:
  start_command: "cargo run --release --bin my-mcp-server"
```

### Issue: HTTP Connection Refused

**Symptoms:**
```
❌ Connection Error: Connection refused
   URL: http://localhost:3000
   Error: Cannot connect to server
```

**Solutions:**

1. **Check Server is Running**:
```bash
# Check if server is listening
netstat -tlnp | grep 3000
# or
ss -tlnp | grep 3000

# Test connection manually
curl http://localhost:3000/health
```

2. **Verify URL and Port**:
```yaml
server:
  transport: "http"
  url: "http://localhost:3000"  # Check port is correct
```

3. **Check Firewall Settings**:
```bash
# Check firewall rules
sudo ufw status
sudo iptables -L

# Allow port if needed
sudo ufw allow 3000
```

### Issue: WebSocket Connection Failed

**Symptoms:**
```
❌ WebSocket Error: Connection failed
   URL: ws://localhost:3000
   Error: HTTP 404 Not Found
```

**Solutions:**

1. **Check WebSocket Endpoint**:
```yaml
server:
  transport: "websocket"
  url: "ws://localhost:3000/ws"  # Add correct path
```

2. **Test WebSocket Manually**:
```bash
# Use websocat to test
websocat ws://localhost:3000

# Or use browser dev tools
```

3. **Check Server WebSocket Support**:
   - Verify your MCP server supports WebSocket transport
   - Check server documentation for correct WebSocket URL format

## 🧪 Test Execution Issues

### Issue: Test Timeout

**Symptoms:**
```
❌ FAIL: repository_stats
   Duration: 30000ms (TIMEOUT)
   Error: Test exceeded maximum execution time
```

**Solutions:**

1. **Increase Test Timeout**:
```yaml
test_cases:
  - id: "repository_stats"
    timeout_override_seconds: 60  # Increase timeout
```

2. **Check Server Performance**:
```bash
# Monitor server resource usage
top -p $(pgrep my-mcp-server)
htop
```

3. **Optimize Test Data**:
```yaml
# Use smaller test datasets
input_params:
  path: "test-projects/small"  # Instead of large project
```

### Issue: Validation Failure

**Symptoms:**
```
❌ FAIL: repository_stats
   Validation Failure:
   - Expected field 'result.total_files' not found
   - Response: {"error": "Path not found"}
```

**Solutions:**

1. **Check Server Response**:
```bash
# Debug server response manually
echo '{"tool": "repository_stats", "arguments": {"path": "/test/project"}}' | cargo run --bin my-mcp-server stdio
```

2. **Update Validation Patterns**:
```yaml
# Handle error responses
expected:
  patterns:
    - key: "result.total_files"
      validation: { type: "exists" }
      required: false  # Make optional if server might return errors
```

3. **Fix Input Parameters**:
```yaml
input_params:
  path: "${default_project_path}"  # Use valid path
```

### Issue: Script Execution Failed

**Symptoms:**
```
❌ Script Error: Custom validation script failed
   Script: scripts/validate_response.py
   Exit Code: 1
   Error: ModuleNotFoundError: No module named 'json'
```

**Solutions:**

1. **Check Script Dependencies**:
```bash
# Test script manually
python3 scripts/validate_response.py < test_input.json

# Install missing dependencies
pip3 install -r scripts/requirements.txt
```

2. **Fix Script Permissions**:
```bash
chmod +x scripts/validate_response.py
```

3. **Use Correct Python Path**:
```yaml
custom_scripts:
  - script: "scripts/validate_response.py"
    engine: "python3"  # Specify interpreter
```

### Issue: Concurrent Test Failures

**Symptoms:**
```
❌ Multiple tests failing with resource exhaustion
   Error: Cannot create thread: Resource temporarily unavailable
```

**Solutions:**

1. **Reduce Concurrency**:
```yaml
global:
  max_global_concurrency: 2  # Reduce from default 4

test_suites:
  - name: "my-suite"
    parallel_execution: false  # Disable parallel execution
```

2. **Increase Resource Limits**:
```bash
# Increase file descriptor limit
ulimit -n 4096

# Increase process limit
ulimit -u 2048
```

3. **Add Resource Monitoring**:
```yaml
environment:
  limits:
    max_memory_mb: 512  # Set reasonable limits
    max_open_files: 256
```

## 📊 Performance Issues

### Issue: Slow Test Execution

**Symptoms:**
```
Performance Warning: Test suite taking longer than expected
Average response time: 15 seconds (baseline: 2 seconds)
```

**Solutions:**

1. **Profile Server Performance**:
```bash
# Use profiling tools
perf record -g cargo run --bin my-mcp-server
perf report

# Monitor system resources
iostat 1
vmstat 1
```

2. **Optimize Test Configuration**:
```yaml
# Reduce test parallelism if causing contention
global:
  max_global_concurrency: 1

# Use smaller test datasets
input_params:
  max_results: 100  # Limit result size
```

3. **Update Performance Baselines**:
```bash
# Re-establish baselines if hardware changed
mcp-test-harness benchmark --establish-baseline --config prod.yaml
```

### Issue: Memory Exhaustion

**Symptoms:**
```
❌ Resource Error: Memory limit exceeded
   Used: 2048 MB
   Limit: 1024 MB
   Test: large_repository_analysis
```

**Solutions:**

1. **Increase Memory Limits**:
```yaml
environment:
  limits:
    max_memory_mb: 4096  # Increase limit

# Or per-test basis
performance:
  max_memory_usage_mb: 2048
```

2. **Optimize Memory Usage**:
```yaml
# Process data in chunks
input_params:
  batch_size: 100
  stream_results: true
```

3. **Monitor Memory Usage**:
```bash
# Check system memory
free -h
htop

# Check for memory leaks
valgrind --tool=memcheck cargo run --bin my-mcp-server
```

### Issue: Disk Space Exhaustion

**Symptoms:**
```
❌ Storage Error: No space left on device
   Available: 0 MB
   Required: 500 MB
```

**Solutions:**

1. **Clean Up Temporary Files**:
```bash
# Clean test artifacts
rm -rf test-reports/old-reports/
rm -rf /tmp/mcp-test-*

# Clean up Docker if using containers
docker system prune -a
```

2. **Configure Cleanup**:
```yaml
global:
  cleanup_on_success: true
  cleanup_on_failure: false  # Keep for debugging

reporting:
  retention_days: 30  # Automatically clean old reports
```

3. **Use External Storage**:
```yaml
reporting:
  output_dir: "/mnt/external/test-reports"  # Use external disk
```

## 🔐 Security Issues

### Issue: Authentication Failed

**Symptoms:**
```
❌ Authentication Error: Invalid credentials
   Status: 401 Unauthorized
   Error: Authentication token invalid or expired
```

**Solutions:**

1. **Check Token Configuration**:
```yaml
server:
  headers:
    Authorization: "Bearer ${API_TOKEN}"  # Verify token format

# Set environment variable
export API_TOKEN="your-actual-token"
```

2. **Verify Token Validity**:
```bash
# Test authentication manually
curl -H "Authorization: Bearer $API_TOKEN" http://localhost:3000/auth/verify
```

3. **Update Authentication Method**:
```yaml
# Use different auth method if needed
server:
  transport: "http"
  headers:
    X-API-Key: "${API_KEY}"  # Alternative auth header
```

### Issue: Certificate Verification Failed

**Symptoms:**
```
❌ TLS Error: Certificate verification failed
   Error: self signed certificate in certificate chain
```

**Solutions:**

1. **Disable Certificate Verification** (Development Only):
```yaml
server:
  tls:
    verify_certificates: false  # ⚠️ Only for development
```

2. **Add Custom CA Bundle**:
```yaml
server:
  tls:
    ca_bundle_path: "/path/to/custom-ca.pem"
```

3. **Use Self-Signed Certificate**:
```bash
# Create self-signed certificate for testing
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes
```

### Issue: Audit Log Errors

**Symptoms:**
```
❌ Audit Error: Cannot write to audit log
   Path: /var/log/mcp-test-harness/audit.log
   Error: Permission denied
```

**Solutions:**

1. **Fix Log Directory Permissions**:
```bash
sudo mkdir -p /var/log/mcp-test-harness
sudo chown $USER:$USER /var/log/mcp-test-harness
chmod 755 /var/log/mcp-test-harness
```

2. **Use Alternative Log Path**:
```yaml
security:
  audit_log_path: "./logs/audit.log"  # Use local directory
```

3. **Disable Audit Logging** (Temporary):
```yaml
security:
  enable_audit_logging: false  # Temporary workaround
```

## 📈 Monitoring Issues

### Issue: Metrics Collection Failed

**Symptoms:**
```
❌ Metrics Error: Cannot connect to Prometheus push gateway
   URL: http://localhost:9091
   Error: Connection refused
```

**Solutions:**

1. **Start Prometheus Push Gateway**:
```bash
# Start push gateway
docker run -d -p 9091:9091 prom/pushgateway

# Or install locally
prometheus-pushgateway --web.listen-address=:9091
```

2. **Disable Metrics Collection**:
```yaml
monitoring:
  enabled: false  # Temporary workaround
```

3. **Use Alternative Metrics Endpoint**:
```yaml
monitoring:
  prometheus:
    push_gateway:
      url: "http://monitoring-server:9091"  # Different server
```

### Issue: Alert Rules Not Working

**Symptoms:**
```
⚠️ Alert Warning: No alerts triggered despite test failures
Expected: Failure rate alert
Actual: No alerts sent
```

**Solutions:**

1. **Check Alert Configuration**:
```yaml
monitoring:
  alerting:
    enabled: true  # Ensure alerting is enabled
    rules:
      - name: "test_failure_rate"
        condition: "test_failure_rate > 0.1"  # Check condition logic
```

2. **Test Alert Channels**:
```bash
# Test email configuration
echo "Test email" | mail -s "Alert Test" team@company.com

# Test Slack webhook
curl -X POST -H 'Content-type: application/json' \
  --data '{"text":"Test alert"}' \
  $SLACK_WEBHOOK_URL
```

3. **Check Alert Thresholds**:
```yaml
# Lower thresholds for testing
monitoring:
  alerts:
    error_rate_threshold: 0.01  # 1% instead of 5%
```

## 🔧 Advanced Debugging

### Enable Debug Logging

```bash
# Maximum debug output
RUST_LOG=debug mcp-test-harness test --verbose --config my-config.yaml

# Trace level logging
RUST_LOG=trace mcp-test-harness test -vvv --config my-config.yaml
```

### Analyze Log Files

```bash
# View recent logs
tail -f ~/.mcp-test-harness/logs/test-harness.log

# Search for specific errors
grep "ERROR" ~/.mcp-test-harness/logs/test-harness.log

# Analyze performance
grep "PERF" ~/.mcp-test-harness/logs/test-harness.log | tail -20
```

### Network Debugging

```bash
# Monitor network traffic
sudo tcpdump -i lo -A port 3000

# Check network connections
netstat -tulpn | grep mcp

# Test connectivity
telnet localhost 3000
```

### Process Debugging

```bash
# Monitor process tree
pstree -p mcp-test-harness

# Check file descriptors
lsof -p $(pgrep mcp-test-harness)

# Monitor system calls
strace -p $(pgrep mcp-test-harness)
```

### Configuration Debugging

```bash
# Validate configuration
mcp-test-harness validate --comprehensive --config my-config.yaml

# Dry run to see execution plan
mcp-test-harness test --dry-run --config my-config.yaml

# Export effective configuration
mcp-test-harness config export --config my-config.yaml
```

## 🆘 Getting Additional Help

### Collecting Diagnostic Information

When reporting issues, collect this information:

```bash
#!/bin/bash
# diagnostic-info.sh

echo "=== System Information ==="
uname -a
cat /etc/os-release

echo "=== MCP Test Harness Version ==="
mcp-test-harness --version

echo "=== Configuration Validation ==="
mcp-test-harness validate --config test-harness.yaml

echo "=== Resource Usage ==="
free -h
df -h

echo "=== Network Connectivity ==="
netstat -tulpn | grep -E ":(3000|8080|9090)"

echo "=== Recent Errors ==="
tail -20 ~/.mcp-test-harness/logs/test-harness.log | grep ERROR

echo "=== Environment Variables ==="
env | grep -E "MCP|RUST|PATH"
```

### Creating Minimal Reproduction

Create a minimal configuration that reproduces the issue:

```yaml
# minimal-repro.yaml
global:
  max_global_concurrency: 1
  log_level: "debug"

server:
  start_command: "echo"  # Simple command for testing
  args: ["hello"]
  transport: "stdio"

test_suites:
  - name: "simple-test"
    test_cases:
      - id: "basic-test"
        tool_name: "echo"
        input_params: {}
        expected:
          patterns:
            - key: "result"
              validation: { type: "exists" }
```

### Support Channels

- **GitHub Issues**: [Report bugs and issues](https://github.com/rustic-ai/codeprism/issues)
- **GitHub Discussions**: [Community Q&A](https://github.com/rustic-ai/codeprism/discussions)
- **Documentation**: [Complete reference](https://docs.codeprism.ai/test-harness)
- **Email Support**: [Direct technical support](mailto:support@codeprism.ai)

### Issue Template

When reporting issues, use this template:

```
## Problem Description
Brief description of the issue

## Environment
- OS: Ubuntu 22.04
- MCP Test Harness Version: 1.0.0
- Server Type: HTTP/stdio/WebSocket
- Server Implementation: Custom/CodePrism/Other

## Configuration
```yaml
# Your configuration (sanitized)
```

## Expected Behavior
What you expected to happen

## Actual Behavior
What actually happened

## Error Messages
```
Complete error messages and stack traces
```

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Additional Context
Any other relevant information
```

---

## 📚 Additional Resources

- [User Guide](user-guide.md) - Complete user manual
- [Configuration Reference](configuration-reference.md) - Complete configuration documentation
- [Production Deployment](production-deployment.md) - Enterprise deployment guide
- [Developer Guide](developer-guide.md) - Extending the test harness

**Last Updated**: 2025-01-07  
**Version**: 1.0.0 