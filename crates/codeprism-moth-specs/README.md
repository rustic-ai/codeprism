# CodePrism Moth Test Specifications

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Comprehensive test specifications for the CodePrism MCP Server using the Mandrel (Moth) test harness.**

This crate contains YAML-based test specifications that comprehensively validate all 26 MCP tools provided by the CodePrism MCP server across multiple programming languages and use cases.

## Overview

The CodePrism MCP server exposes powerful code analysis capabilities through the Model Context Protocol (MCP). This test specification suite ensures:

- ✅ **Complete tool coverage** - All 26 MCP tools tested
- ✅ **Multi-language support** - Java, JavaScript/TypeScript, Python, Rust  
- ✅ **Real-world scenarios** - Enterprise patterns, framework analysis, security scanning
- ✅ **Performance validation** - Latency and memory requirements verified
- ✅ **Error handling** - Comprehensive edge case and error scenario testing

## Directory Structure

```
crates/codeprism-moth-specs/
├── codeprism/
│   ├── comprehensive/          # Language-specific comprehensive test suites
│   │   ├── codeprism-java-comprehensive.yaml
│   │   ├── codeprism-javascript-comprehensive.yaml  
│   │   ├── codeprism-python-comprehensive.yaml
│   │   └── codeprism-rust-comprehensive.yaml
│   ├── tools/                  # Tool category-focused test suites
│   │   ├── codeprism-complexity-analysis.yaml
│   │   ├── codeprism-core-navigation.yaml
│   │   ├── codeprism-flow-analysis.yaml
│   │   ├── codeprism-javascript-analysis.yaml
│   │   ├── codeprism-search-discovery.yaml
│   │   └── codeprism-specialized-analysis.yaml
│   └── workflows/              # Workflow and orchestration tests
│       └── codeprism-workflow-orchestration.yaml
└── docs/                       # Documentation and guides
```

## Test Categories

### 🏗️ **Comprehensive Language Suites**
Complete validation for specific programming languages:
- **Java**: Enterprise patterns, Spring framework, Maven/Gradle, JVM optimization (19 tools)
- **JavaScript/TypeScript**: React components, Node.js patterns, bundle optimization (23 tools)  
- **Python**: Django/Flask patterns, async programming, package management (26 tools)
- **Rust**: Memory safety, systems programming, Cargo ecosystem (18 tools)

### 🔧 **Tool Category Suites**
Focused testing by tool functionality:
- **Core Navigation**: Repository stats, path tracing, symbol explanation, dependency analysis
- **Complexity Analysis**: Code complexity metrics, duplicate detection, quality scoring
- **Search & Discovery**: Content search, file finding, symbol searching
- **Flow Analysis**: Data flow tracking, pattern detection, usage analysis
- **Specialized Analysis**: Security scanning, performance analysis, API analysis
- **JavaScript-Specific**: Framework analysis, React component analysis, Node.js patterns

### 🔄 **Workflow Orchestration**
End-to-end workflow testing:
- **Batch Analysis**: Multi-tool execution, parallel processing
- **Workflow Suggestions**: Analysis pipeline recommendations
- **Result Aggregation**: Cross-tool result combination

## Usage with Mandrel (Moth)

Run these specifications using the Mandrel MCP test harness:

```bash
# Install moth (Mandrel test harness)
cargo install --path crates/mandrel-mcp-th

# Run comprehensive Python testing
moth test crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml

# Run all core navigation tests
moth test crates/codeprism-moth-specs/codeprism/tools/codeprism-core-navigation.yaml

# Run with specific filters
moth test crates/codeprism-moth-specs/codeprism/tools/codeprism-search-discovery.yaml \
  --filter "search_symbols" \
  --output json \
  --fail-fast

# Validate all specifications
find crates/codeprism-moth-specs -name "*.yaml" -exec moth validate {} \;
```

## Test Coverage

Each specification provides comprehensive coverage:

| **Test Type** | **Coverage** | **Example** |
|---------------|--------------|-------------|
| **Tool Tests** | 26/26 tools (100%) | All MCP tools individually tested |
| **Language Tests** | 4 major languages | Java, JS/TS, Python, Rust |
| **Error Scenarios** | ~50 error cases | Invalid inputs, missing files, timeouts |
| **Performance Tests** | Latency + Memory | <5s execution, <100MB memory |
| **Integration Tests** | End-to-end workflows | Multi-tool analysis pipelines |

## Specification Features

### 🎯 **Realistic Test Scenarios**
- **Enterprise codebases**: Large-scale application testing
- **Framework patterns**: Spring, React, Django, Rocket detection
- **Real projects**: Testing against actual test-projects/ directory
- **Performance benchmarks**: Production-level latency requirements

### 🛡️ **Comprehensive Error Handling**
- **Invalid inputs**: Malformed parameters, missing fields
- **File system errors**: Non-existent paths, permission issues  
- **Timeout scenarios**: Long-running analysis operations
- **Resource exhaustion**: Large file processing, memory limits

### 📊 **Detailed Validation**
- **JSONPath validation**: Precise result structure verification
- **Schema compliance**: JSON Schema validation for all responses
- **Performance metrics**: Response time and memory usage tracking
- **Content verification**: Actual analysis result validation

## Example Test Specification

```yaml
name: "CodePrism Core Navigation Tools"
description: "Test specification for core navigation and analysis tools"

server:
  command: "cargo"
  args: ["run", "--package", "codeprism-mcp-server", "--bin", "codeprism-mcp-server"]
  transport: "stdio"

tools:
  - name: "repository_stats"
    tests:
      - name: "analyze_python_repository"
        input:
          project_path: "test-projects/python-sample"
          language: "python"
        expected:
          error: false
          fields:
            - path: "$.result.repository_overview"
              field_type: "object"
              required: true
        performance:
          max_duration_ms: 5000
          max_memory_mb: 50
```

## Contributing

1. **Follow the naming convention**: `codeprism-{category}-{description}.yaml`
2. **Include comprehensive scenarios**: Success, error, and edge cases
3. **Add performance requirements**: Realistic latency and memory limits
4. **Validate specifications**: Use `moth validate` before submitting
5. **Test against real projects**: Use test-projects/ directory for realistic data

## Development

```bash
# Validate all specifications
cargo test --features validation

# Run individual specification validation
moth validate crates/codeprism-moth-specs/codeprism/tools/codeprism-core-navigation.yaml

# Check specification syntax
serde_yaml::from_str validation in tests
```

## Related Projects

- **[Mandrel MCP Test Harness](../mandrel-mcp-th/)** - The test execution engine
- **[CodePrism MCP Server](../codeprism-mcp-server/)** - The server being tested
- **[MCP Specification](../../specification/)** - Protocol specification reference

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details. 