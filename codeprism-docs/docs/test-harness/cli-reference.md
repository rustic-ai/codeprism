---
title: CLI Reference
description: Complete command-line reference for the moth binary
sidebar_position: 4
---

# CLI Reference - Mandrel MCP Test Harness

Complete command-line reference for the **moth** binary (MOdel context protocol Test Harness).

## Overview

```bash
moth [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

### Global Options

```bash
-v, --verbose...                 Enable verbose output (can be used multiple times)
-q, --quiet                      Suppress non-essential output
-p, --profile <PROFILE>          Use named configuration profile
    --detect-ci                  Auto-detect CI environment and apply optimizations
    --simulate-ci <CI_TYPE>      Override environment detection (for testing CI configurations locally)
                                 [possible values: git-hub-actions, jenkins, git-lab-ci, circle-ci, 
                                  travis, buildkite, team-city, azure-dev-ops]
    --config-dir <CONFIG_DIR>    Configuration directory for profiles and settings 
                                 [default: ~/.config/mandrel-mcp-th]
-h, --help                       Print help
-V, --version                    Print version
```

## Commands

### `moth run` - Execute Test Specifications

Execute test specifications against MCP servers with comprehensive reporting.

```bash
moth run [OPTIONS] <CONFIG>
```

#### Arguments

- `<CONFIG>` - Path to test specification file (YAML format)

#### Options

```bash
-o, --output <DIRECTORY>    Output directory for generated reports [default: ./reports]
    --parallel              Run tests in parallel (default behavior)
    --fail-fast             Stop execution on first test failure
-h, --help                  Print help
```

#### Examples

```bash
# Run tests with default reporting
moth run my-server.yaml

# Run tests with custom output directory
moth run my-server.yaml --output ./test-results

# Stop on first failure for quick debugging
moth run my-server.yaml --fail-fast

# Run with verbose output
moth -v run my-server.yaml

# Run with maximum verbosity
moth -vvv run my-server.yaml
```

#### Generated Reports

The `run` command automatically generates multiple report formats:

- **JSON Report**: `test_report.json` - Machine-readable results
- **HTML Report**: `test_report.html` - Interactive web report with charts
- **JUnit XML**: `test_report.xml` - CI/CD integration format

### `moth validate` - Validate Configuration Files

Validate test specifications for syntax, schema compliance, and best practices.

```bash
moth validate [OPTIONS] <CONFIG>
```

#### Arguments

- `<CONFIG>` - Configuration file to validate

#### Options

```bash
    --strict                     Enable strict validation mode (fail on warnings)
-o, --output <DIRECTORY>         Output directory for validation reports
-f, --formats <FORMATS>          Report formats to generate [possible values: json, junit, html, markdown]
    --check-jsonpath             Check JSONPath expressions in test cases
    --check-schema               Validate JSON schema compliance
    --check-protocol             Validate MCP protocol compliance
    --check-all                  Enable all validation checks
    --detailed                   Enable detailed validation diagnostics
    --no-suggestions             Only validate, don't suggest fixes
-h, --help                       Print help
```

#### Examples

```bash
# Basic validation
moth validate my-server.yaml

# Strict validation with all checks
moth validate --strict --check-all my-server.yaml

# Generate detailed validation report
moth validate --detailed --formats html my-server.yaml

# Validate with specific checks
moth validate --check-jsonpath --check-protocol my-server.yaml
```

### `moth report` - Generate Reports from Results

Generate or regenerate reports from existing test execution results.

```bash
moth report [OPTIONS] <RESULTS_DIR>
```

#### Arguments

- `<RESULTS_DIR>` - Directory containing test execution results

#### Examples

```bash
# Regenerate reports from existing results
moth report ./reports

# Generate specific format reports
moth report --formats html,markdown ./reports
```

### `moth profile` - Configuration Profile Management

Manage configuration profiles for different environments and setups.

```bash
moth profile <SUBCOMMAND>
```

#### Subcommands

- `list` - List available profiles
- `show <PROFILE>` - Show profile configuration
- `create <PROFILE>` - Create new profile
- `delete <PROFILE>` - Delete profile

#### Examples

```bash
# List all profiles
moth profile list

# Show development profile
moth profile show development

# Use specific profile for testing
moth --profile production run my-server.yaml
```

### `moth watch` - Auto-Generate Reports

Watch files and automatically regenerate reports on changes.

```bash
moth watch [OPTIONS] <WATCH_DIR>
```

#### Arguments

- `<WATCH_DIR>` - Directory to watch for changes

#### Examples

```bash
# Watch for changes and auto-regenerate reports
moth watch ./test-results

# Watch with verbose output
moth -v watch ./test-results
```

## Working with Real Examples

### Filesystem Server Testing

```bash
# Test the filesystem MCP server
moth run crates/mandrel-mcp-th/examples/filesystem-server-mcp-compliant.yaml

# With verbose output to see detailed execution
moth -v run crates/mandrel-mcp-th/examples/filesystem-server-mcp-compliant.yaml
```

### Everything Server Testing

```bash
# Test the everything MCP server (comprehensive test)
moth run crates/mandrel-mcp-th/examples/everything-server-working.yaml

# Validate the specification first
moth validate crates/mandrel-mcp-th/examples/everything-server-working.yaml
```

## CI/CD Integration

### GitHub Actions Integration

```bash
# Auto-detect GitHub Actions environment
moth --detect-ci run my-server.yaml

# Simulate GitHub Actions locally
moth --simulate-ci github-actions run my-server.yaml
```

### Common CI Patterns

```bash
# Fail fast for quick CI feedback
moth run --fail-fast my-server.yaml

# Generate JUnit XML for CI integration
moth run my-server.yaml  # JUnit XML automatically generated

# Quiet mode for clean CI logs
moth --quiet run my-server.yaml
```

## Exit Codes

- **0**: All tests passed successfully
- **1**: One or more tests failed
- **2**: Configuration/validation error
- **3**: Server startup failure
- **4**: Runtime error (unexpected failure)

## Performance Tips

### Optimization for Large Test Suites

```bash
# Use parallel execution (default)
moth run --parallel my-server.yaml

# Custom configuration directory for faster profile loading
moth --config-dir ./project-config run my-server.yaml

# Reduce verbosity for faster execution
moth --quiet run my-server.yaml
```

### Debugging Failed Tests

```bash
# Maximum verbosity for debugging
moth -vvv run my-server.yaml

# Stop on first failure and preserve artifacts
moth run --fail-fast my-server.yaml

# Validate configuration before running
moth validate --detailed my-server.yaml
```

## Output Format Examples

### JSON Report Structure
```json
{
  "suite_name": "Everything MCP Server (Working Tests)",
  "total_tests": 8,
  "passed": 8,
  "failed": 0,
  "skipped": 0,
  "total_duration": {
    "secs": 10,
    "nanos": 17847712
  },
  "success_rate": 100.0,
  "test_results": [
    {
      "test_name": "integer_addition",
      "status": "Passed",
      "response_time_ms": 0,
      "performance": {
        "response_time_ms": 0,
        "memory_usage_bytes": null
      }
    }
  ]
}
```

### HTML Report Features
- **Interactive charts** showing test results and performance metrics
- **Detailed test breakdowns** with response times and validation results
- **Error analysis** with suggested fixes
- **Performance trends** and regression detection
- **Filterable test results** by status, category, or performance

---

**Need help with configuration?** Check out our [Configuration Reference](configuration-reference) for complete YAML specification format documentation. 