# Production Deployment Guide

Complete guide for deploying MCP Test Harness in production environments with enterprise-grade reliability, security, and monitoring.

## 📋 Overview

This guide covers production deployment scenarios including:
- **Enterprise Environments** - Large-scale deployment with high availability
- **CI/CD Integration** - Automated testing in build pipelines
- **Cloud Deployment** - AWS, GCP, Azure deployment strategies
- **Container Orchestration** - Kubernetes and Docker Swarm deployment
- **Performance Optimization** - Scaling and optimization strategies

## 🎯 Production Requirements

### System Requirements

**Minimum Requirements (Small Teams)**
- **CPU**: 2 cores, 2.4 GHz
- **Memory**: 4 GB RAM
- **Storage**: 20 GB SSD
- **Network**: 100 Mbps
- **OS**: Linux (Ubuntu 20.04+), macOS 11+, Windows 10+

**Recommended Requirements (Enterprise)**
- **CPU**: 8 cores, 3.0 GHz
- **Memory**: 16 GB RAM
- **Storage**: 100 GB SSD (NVMe preferred)
- **Network**: 1 Gbps
- **OS**: Linux (Ubuntu 22.04 LTS)

**High-Scale Requirements (Large Organizations)**
- **CPU**: 16+ cores, 3.5 GHz
- **Memory**: 32+ GB RAM
- **Storage**: 500+ GB SSD (NVMe)
- **Network**: 10 Gbps
- **OS**: Linux (RHEL 8+, Ubuntu 22.04 LTS)

### Dependencies

**Required Dependencies**
```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# System packages (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    nodejs \
    npm
```

**Optional Dependencies**
```bash
# For Docker deployment
sudo apt-get install -y docker.io docker-compose

# For monitoring integration
sudo apt-get install -y prometheus-node-exporter

# For database storage
sudo apt-get install -y postgresql-client redis-tools
```

## 🚀 Installation Methods

### Method 1: Binary Installation (Recommended)

```bash
# Download latest release
curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux-x86_64.tar.gz | tar xz

# Move to system path
sudo mv mcp-test-harness /usr/local/bin/
sudo chmod +x /usr/local/bin/mcp-test-harness

# Verify installation
mcp-test-harness --version
```

### Method 2: Cargo Installation

```bash
# Install from crates.io
cargo install mcp-test-harness

# Or build from source
git clone https://github.com/rustic-ai/codeprism.git
cd prism/mcp-test-harness-standalone
cargo build --release
sudo cp target/release/mcp-test-harness /usr/local/bin/
```

### Method 3: Docker Installation

```bash
# Pull latest image
docker pull mcp-test-harness:latest

# Create alias for convenience
echo 'alias mcp-test-harness="docker run --rm -v $(pwd):/workspace -w /workspace mcp-test-harness"' >> ~/.bashrc
source ~/.bashrc
```

## 🔧 Configuration Management

### Production Configuration Structure

```
/etc/mcp-test-harness/
├── config/
│   ├── production.yaml          # Main production config
│   ├── staging.yaml            # Staging environment config
│   ├── development.yaml        # Development config
│   └── environments/
│       ├── aws.yaml            # AWS-specific settings
│       ├── gcp.yaml            # GCP-specific settings
│       └── on-premise.yaml     # On-premise settings
├── test-suites/
│   ├── core/                   # Core MCP protocol tests
│   ├── security/               # Security compliance tests
│   ├── performance/            # Performance benchmark tests
│   └── custom/                 # Organization-specific tests
├── baselines/
│   ├── performance/            # Performance baselines
│   └── regression/             # Regression test data
└── reports/
    ├── daily/                  # Daily test reports
    ├── weekly/                 # Weekly trend reports
    └── alerts/                 # Alert configurations
```

### Production Configuration Template

```yaml
# /etc/mcp-test-harness/config/production.yaml
global:
  environment: "production"
  max_global_concurrency: 8
  global_timeout_seconds: 600
  fail_fast: false
  log_level: "info"
  
  # Resource limits for production
  resource_limits:
    max_memory_mb: 2048
    max_cpu_seconds: 300
    max_disk_usage_mb: 1024
    max_network_connections: 100
  
  # Retry configuration for production reliability
  retry:
    max_retries: 3
    retry_delay_ms: 2000
    exponential_backoff: true
    retry_on_patterns:
      - "connection refused"
      - "timeout"
      - "temporary failure"
      - "resource unavailable"

# High-availability server configuration
server:
  health_check:
    enabled: true
    interval_seconds: 30
    failure_threshold: 5
    timeout_seconds: 10
    recovery_timeout_seconds: 60
  
  # Connection pooling for better performance
  connection_pool:
    max_connections: 20
    connection_timeout_seconds: 30
    idle_timeout_seconds: 300
    keepalive_enabled: true

# Production monitoring and alerting
monitoring:
  enabled: true
  metrics_port: 9090
  health_check_port: 8080
  
  # Prometheus integration
  prometheus:
    enabled: true
    endpoint: "http://prometheus:9090"
    push_gateway: "http://prometheus-pushgateway:9091"
    job_name: "mcp-test-harness"
  
  # Alert thresholds
  alerts:
    performance_regression_threshold: 30
    error_rate_threshold: 5
    success_rate_threshold: 95
    response_time_threshold_ms: 10000

# Performance baseline management
performance:
  baseline_storage_path: "/etc/mcp-test-harness/baselines"
  enable_trend_analysis: true
  retention_days: 365
  
  # Performance thresholds
  thresholds:
    warning_threshold_percent: 25
    error_threshold_percent: 50
    critical_threshold_percent: 100
  
  # Automatic baseline updates
  baseline_updates:
    enabled: true
    confidence_threshold: 0.95
    minimum_samples: 10
    update_frequency_days: 7

# Comprehensive reporting
reporting:
  output_dir: "/var/log/mcp-test-harness/reports"
  formats: ["html", "json", "junit"]
  retention_days: 90
  
  # Report distribution
  distribution:
    email:
      enabled: true
      smtp_server: "smtp.company.com"
      recipients: ["team@company.com"]
      on_failure: true
      daily_summary: true
    
    slack:
      enabled: true
      webhook_url: "${SLACK_WEBHOOK_URL}"
      channel: "#test-reports"
      on_failure: true
      weekly_summary: true

# Security configuration
security:
  enable_audit_logging: true
  audit_log_path: "/var/log/mcp-test-harness/audit.log"
  
  # Access control
  rbac:
    enabled: true
    config_file: "/etc/mcp-test-harness/rbac.yaml"
  
  # Data protection
  data_protection:
    encrypt_sensitive_data: true
    key_management: "vault"
    vault_endpoint: "${VAULT_ENDPOINT}"

# Environment-specific overrides
environments:
  production:
    log_level: "warn"
    max_global_concurrency: 16
    performance_thresholds:
      critical_threshold_percent: 75
  
  staging:
    log_level: "info"
    max_global_concurrency: 4
    alerts:
      error_rate_threshold: 10
  
  development:
    log_level: "debug"
    max_global_concurrency: 2
    fail_fast: true
```

## 🔄 Deployment Strategies

### Blue-Green Deployment

```bash
#!/bin/bash
# blue-green-deploy.sh

# Configuration
BLUE_ENV="blue"
GREEN_ENV="green"
HEALTH_CHECK_URL="http://localhost:8080/health"
TIMEOUT=300

# Deploy to green environment
echo "Deploying to green environment..."
docker-compose -f docker-compose.green.yaml up -d mcp-test-harness

# Wait for health check
echo "Waiting for green environment to be healthy..."
timeout $TIMEOUT bash -c '
  until curl -f '"$HEALTH_CHECK_URL"' > /dev/null 2>&1; do
    echo "Waiting for health check..."
    sleep 10
  done
'

if [ $? -eq 0 ]; then
  echo "Green environment is healthy. Switching traffic..."
  
  # Update load balancer to point to green
  # (Implementation depends on your load balancer)
  
  # Wait for traffic to settle
  sleep 30
  
  # Stop blue environment
  docker-compose -f docker-compose.blue.yaml down
  
  echo "Blue-green deployment completed successfully"
else
  echo "Green environment failed health check. Rolling back..."
  docker-compose -f docker-compose.green.yaml down
  exit 1
fi
```

### Rolling Deployment

```yaml
# kubernetes-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-test-harness
  namespace: testing
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: mcp-test-harness
  template:
    metadata:
      labels:
        app: mcp-test-harness
    spec:
      containers:
      - name: mcp-test-harness
        image: mcp-test-harness:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: CONFIG_PATH
          value: "/etc/config/production.yaml"
        volumeMounts:
        - name: config
          mountPath: /etc/config
        - name: data
          mountPath: /var/lib/mcp-test-harness
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          limits:
            cpu: "2"
            memory: "4Gi"
          requests:
            cpu: "1"
            memory: "2Gi"
      volumes:
      - name: config
        configMap:
          name: mcp-test-harness-config
      - name: data
        persistentVolumeClaim:
          claimName: mcp-test-harness-data
```

## 📊 Performance Baseline Establishment

### Initial Baseline Setup

```bash
#!/bin/bash
# establish-baselines.sh

echo "Establishing performance baselines for production..."

# Run baseline tests for each test suite
SUITES=("core-protocol" "security-compliance" "performance-benchmarks")

for suite in "${SUITES[@]}"; do
  echo "Running baseline for $suite..."
  
  # Run multiple iterations to establish reliable baseline
  mcp-test-harness benchmark \
    --config "/etc/mcp-test-harness/config/production.yaml" \
    --suite "$suite" \
    --iterations 20 \
    --duration 300 \
    --establish-baseline \
    --output-dir "/etc/mcp-test-harness/baselines/$suite"
  
  if [ $? -eq 0 ]; then
    echo "✅ Baseline established for $suite"
  else
    echo "❌ Failed to establish baseline for $suite"
    exit 1
  fi
done

echo "🎉 All baselines established successfully"
```

### Baseline Validation Script

```python
#!/usr/bin/env python3
# validate-baselines.py

import json
import statistics
from pathlib import Path

def validate_baseline(baseline_file):
    """Validate that baseline data is statistically sound"""
    
    with open(baseline_file) as f:
        data = json.load(f)
    
    response_times = data.get('response_times', [])
    
    if len(response_times) < 10:
        return False, "Insufficient data points"
    
    # Check for reasonable variance
    mean_time = statistics.mean(response_times)
    std_dev = statistics.stdev(response_times)
    
    # Coefficient of variation should be < 50%
    cv = (std_dev / mean_time) * 100 if mean_time > 0 else 100
    
    if cv > 50:
        return False, f"High variance: CV = {cv:.1f}%"
    
    # Check for outliers (values > 3 standard deviations)
    outliers = [t for t in response_times if abs(t - mean_time) > 3 * std_dev]
    outlier_ratio = len(outliers) / len(response_times)
    
    if outlier_ratio > 0.1:
        return False, f"Too many outliers: {outlier_ratio:.1%}"
    
    return True, f"Valid baseline: {len(response_times)} samples, CV = {cv:.1f}%"

# Validate all baselines
baseline_dir = Path("/etc/mcp-test-harness/baselines")
all_valid = True

for baseline_file in baseline_dir.glob("**/baseline.json"):
    is_valid, message = validate_baseline(baseline_file)
    status = "✅" if is_valid else "❌"
    print(f"{status} {baseline_file.parent.name}: {message}")
    
    if not is_valid:
        all_valid = False

exit(0 if all_valid else 1)
```

## 🔍 Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "mcp-test-harness-rules.yml"

scrape_configs:
  - job_name: 'mcp-test-harness'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 10s
    metrics_path: /metrics
    
  - job_name: 'mcp-test-harness-health'
    static_configs:
      - targets: ['localhost:8080']
    scrape_interval: 30s
    metrics_path: /health/metrics

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alert Rules

```yaml
# mcp-test-harness-rules.yml
groups:
  - name: mcp-test-harness
    rules:
      - alert: MCPTestHarnessDown
        expr: up{job="mcp-test-harness"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "MCP Test Harness is down"
          description: "MCP Test Harness has been down for more than 1 minute"
      
      - alert: HighErrorRate
        expr: mcp_test_error_rate > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} for 5 minutes"
      
      - alert: PerformanceRegression
        expr: mcp_test_response_time_p95 > mcp_test_baseline_p95 * 1.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Performance regression detected"
          description: "95th percentile response time is 50% higher than baseline"
      
      - alert: LowSuccessRate
        expr: mcp_test_success_rate < 0.95
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Low success rate"
          description: "Success rate is {{ $value | humanizePercentage }}"
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "MCP Test Harness Production Dashboard",
    "panels": [
      {
        "title": "Test Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "mcp_test_success_rate",
            "legendFormat": "Success Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "min": 0,
            "max": 1,
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 0.9},
                {"color": "green", "value": 0.95}
              ]
            }
          }
        }
      },
      {
        "title": "Response Time Distribution",
        "type": "histogram",
        "targets": [
          {
            "expr": "mcp_test_response_time_bucket",
            "legendFormat": "Response Time"
          }
        ]
      },
      {
        "title": "Test Execution Volume",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(mcp_test_total[5m])",
            "legendFormat": "Tests per second"
          }
        ]
      }
    ]
  }
}
```

## 🔐 Security Configuration

### Role-Based Access Control

```yaml
# rbac.yaml
roles:
  - name: "test-operator"
    permissions:
      - "test:run"
      - "test:view"
      - "report:view"
    
  - name: "test-admin"
    permissions:
      - "test:*"
      - "config:*"
      - "baseline:*"
      - "report:*"
    
  - name: "read-only"
    permissions:
      - "test:view"
      - "report:view"

users:
  - username: "ci-system"
    role: "test-operator"
    api_key_hash: "sha256:..."
    
  - username: "admin"
    role: "test-admin"
    api_key_hash: "sha256:..."
```

### SSL/TLS Configuration

```yaml
# tls-config.yaml
tls:
  enabled: true
  cert_file: "/etc/ssl/certs/mcp-test-harness.crt"
  key_file: "/etc/ssl/private/mcp-test-harness.key"
  ca_file: "/etc/ssl/certs/ca.crt"
  
  # Client certificate validation
  client_auth: "require"
  client_ca_file: "/etc/ssl/certs/client-ca.crt"
  
  # TLS configuration
  min_version: "1.2"
  cipher_suites:
    - "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
    - "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
```

## 💾 Backup and Recovery

### Backup Strategy

```bash
#!/bin/bash
# backup-production.sh

BACKUP_DIR="/var/backups/mcp-test-harness"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

echo "Starting MCP Test Harness backup..."

# Create backup directory structure
mkdir -p "$BACKUP_DIR/$DATE"

# Backup configuration
echo "Backing up configuration..."
tar -czf "$BACKUP_DIR/$DATE/config.tar.gz" \
  /etc/mcp-test-harness/config/ \
  /etc/mcp-test-harness/test-suites/

# Backup baselines and historical data
echo "Backing up baselines..."
tar -czf "$BACKUP_DIR/$DATE/baselines.tar.gz" \
  /etc/mcp-test-harness/baselines/

# Backup reports (last 30 days)
echo "Backing up recent reports..."
find /var/log/mcp-test-harness/reports -type f -mtime -30 | \
  tar -czf "$BACKUP_DIR/$DATE/reports.tar.gz" -T -

# Backup database (if used)
if systemctl is-active --quiet postgresql; then
  echo "Backing up database..."
  pg_dump mcp_test_harness > "$BACKUP_DIR/$DATE/database.sql"
fi

# Create backup manifest
echo "Creating backup manifest..."
cat > "$BACKUP_DIR/$DATE/manifest.json" << EOF
{
  "backup_date": "$(date -Iseconds)",
  "backup_type": "full",
  "components": [
    "configuration",
    "baselines",
    "reports",
    "database"
  ],
  "version": "$(mcp-test-harness --version)",
  "retention_days": $RETENTION_DAYS
}
EOF

# Cleanup old backups
echo "Cleaning up backups older than $RETENTION_DAYS days..."
find "$BACKUP_DIR" -type d -mtime +$RETENTION_DAYS -exec rm -rf {} +

echo "✅ Backup completed: $BACKUP_DIR/$DATE"
```

### Recovery Procedures

```bash
#!/bin/bash
# restore-production.sh

BACKUP_DIR="/var/backups/mcp-test-harness"
RESTORE_DATE="$1"

if [ -z "$RESTORE_DATE" ]; then
  echo "Usage: $0 <backup_date>"
  echo "Available backups:"
  ls -1 "$BACKUP_DIR" | sort -r | head -10
  exit 1
fi

RESTORE_PATH="$BACKUP_DIR/$RESTORE_DATE"

if [ ! -d "$RESTORE_PATH" ]; then
  echo "❌ Backup not found: $RESTORE_PATH"
  exit 1
fi

echo "🔄 Restoring from backup: $RESTORE_DATE"

# Stop services
echo "Stopping services..."
systemctl stop mcp-test-harness

# Restore configuration
echo "Restoring configuration..."
tar -xzf "$RESTORE_PATH/config.tar.gz" -C /

# Restore baselines
echo "Restoring baselines..."
tar -xzf "$RESTORE_PATH/baselines.tar.gz" -C /

# Restore reports
echo "Restoring reports..."
tar -xzf "$RESTORE_PATH/reports.tar.gz" -C /

# Restore database
if [ -f "$RESTORE_PATH/database.sql" ]; then
  echo "Restoring database..."
  dropdb mcp_test_harness
  createdb mcp_test_harness
  psql mcp_test_harness < "$RESTORE_PATH/database.sql"
fi

# Start services
echo "Starting services..."
systemctl start mcp-test-harness

# Verify restoration
echo "Verifying restoration..."
sleep 10
if mcp-test-harness validate --config /etc/mcp-test-harness/config/production.yaml; then
  echo "✅ Restoration completed successfully"
else
  echo "❌ Restoration verification failed"
  exit 1
fi
```

## 📋 Maintenance Procedures

### Scheduled Maintenance Script

```bash
#!/bin/bash
# maintenance.sh

echo "Starting scheduled maintenance..."

# Update performance baselines
echo "Updating performance baselines..."
mcp-test-harness baseline update \
  --config /etc/mcp-test-harness/config/production.yaml \
  --auto-approve

# Cleanup old reports
echo "Cleaning up old reports..."
find /var/log/mcp-test-harness/reports -type f -mtime +90 -delete

# Rotate logs
echo "Rotating logs..."
logrotate /etc/logrotate.d/mcp-test-harness

# Update test suites
echo "Checking for test suite updates..."
git -C /etc/mcp-test-harness/test-suites pull origin main

# Health check
echo "Running health check..."
mcp-test-harness health-check \
  --config /etc/mcp-test-harness/config/production.yaml \
  --comprehensive

echo "✅ Scheduled maintenance completed"
```

### Update Procedures

```bash
#!/bin/bash
# update-production.sh

NEW_VERSION="$1"
if [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

echo "Updating MCP Test Harness to version $NEW_VERSION"

# Create backup before update
./backup-production.sh

# Download new version
curl -L "https://github.com/rustic-ai/codeprism/releases/download/$NEW_VERSION/mcp-test-harness-linux-x86_64.tar.gz" | tar xz

# Test new version
echo "Testing new version..."
./mcp-test-harness --version
if [ $? -ne 0 ]; then
  echo "❌ New version failed basic test"
  exit 1
fi

# Stop current service
systemctl stop mcp-test-harness

# Install new version
sudo mv mcp-test-harness /usr/local/bin/
sudo chmod +x /usr/local/bin/mcp-test-harness

# Start service
systemctl start mcp-test-harness

# Verify update
sleep 30
if systemctl is-active --quiet mcp-test-harness; then
  echo "✅ Update completed successfully"
  echo "New version: $(mcp-test-harness --version)"
else
  echo "❌ Update failed, rolling back..."
  ./restore-production.sh $(ls -1 /var/backups/mcp-test-harness | sort -r | head -1)
fi
```

## 🔧 Troubleshooting

### Common Production Issues

**Issue: High Memory Usage**
```bash
# Check memory usage
mcp-test-harness stats --memory

# Adjust memory limits
# Edit /etc/mcp-test-harness/config/production.yaml
resource_limits:
  max_memory_mb: 4096  # Increase limit
```

**Issue: Performance Degradation**
```bash
# Check performance metrics
mcp-test-harness benchmark --quick

# Update baselines if needed
mcp-test-harness baseline update --force
```

**Issue: Connection Timeouts**
```bash
# Check network connectivity
mcp-test-harness network-test

# Increase timeouts
# Edit configuration
global:
  global_timeout_seconds: 900  # Increase timeout
```

### Emergency Procedures

**Service Recovery**
```bash
# Quick service restart
systemctl restart mcp-test-harness

# Full service recovery
systemctl stop mcp-test-harness
systemctl reset-failed mcp-test-harness
systemctl start mcp-test-harness
```

**Data Corruption Recovery**
```bash
# Restore from backup
./restore-production.sh $(ls -1 /var/backups/mcp-test-harness | sort -r | head -1)

# Rebuild baselines if needed
mcp-test-harness baseline rebuild --all
```

## 📊 Production Metrics

### Key Performance Indicators

- **Availability**: Target 99.9% uptime
- **Success Rate**: Target 99% test success rate
- **Response Time**: Target 95th percentile < 5 seconds
- **Error Rate**: Target < 1% error rate
- **Resource Usage**: Target < 80% CPU/Memory utilization

### Monitoring Checklist

- [ ] Service health checks running
- [ ] Performance baselines established
- [ ] Alert rules configured
- [ ] Dashboard monitoring in place
- [ ] Backup procedures tested
- [ ] Maintenance schedule defined
- [ ] Incident response plan documented

---

## 📚 Additional Resources

- [Configuration Reference](configuration-reference.md) - Complete configuration documentation
- [Performance Tuning](performance-tuning.md) - Performance optimization guide
- [Monitoring & Alerting](monitoring-alerting.md) - Detailed monitoring setup
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions
- [CI/CD Integration](cicd-integration.md) - Automated deployment strategies

**Last Updated**: 2025-01-07  
**Version**: 1.0.0 