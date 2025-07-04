---
title: API Reference
description: Complete API reference for all 23 production-ready CodePrism tools and capabilities
sidebar_position: 3
---

# CodePrism API Reference

## Available Tools

CodePrism offers **23 production-ready tools** for code analysis, navigation, and workflow orchestration, plus comprehensive **parser development tools**. Below is a detailed reference of the available tools and their usage.

### Core Navigation & Understanding (4 Tools)

- **`repository_stats`**: Provides a comprehensive overview and statistics of the repository.
- **`explain_symbol`**: Offers detailed analysis of a symbol with context, accepting semantic names like "UserManager".
- **`trace_path`**: Finds execution paths between code elements.
- **`find_dependencies`**: Analyzes what a symbol or file depends on.

### Search & Discovery (4 Tools)

- **`search_symbols`**: Performs advanced symbol search with regex and inheritance filtering.
- **`search_content`**: Conducts full-text search across all repository content.
- **`find_files`**: Discovers files using glob and regex pattern support.
- **`content_stats`**: Provides detailed content and complexity statistics.

### Analysis Tools (11 Tools)

#### Production-Ready Milestone 2 Tools (4 Tools)
- **`find_unused_code`**: **PRODUCTION-READY v2.0.0** - Graph-based unused code detection with confidence scoring and potential savings metrics.
- **`analyze_security`**: **PRODUCTION-READY v2.0.0** - Advanced vulnerability detection with CVSS scoring and OWASP mapping.
- **`analyze_performance`**: **PRODUCTION-READY v2.0.0** - Performance analysis with time complexity and memory usage detection.
- **`analyze_api_surface`**: **PRODUCTION-READY v2.0.0** - API surface analysis with versioning compliance and breaking change detection.

#### Core Analysis Tools (7 Tools)
- **`analyze_complexity`**: Measures code complexity and maintainability.
- **`trace_data_flow`**: **PRODUCTION-READY** - Bidirectional data flow tracing with transformation tracking.
- **`analyze_transitive_dependencies`**: **PRODUCTION-READY** - Complete dependency chain analysis with cycle detection.
- **`detect_patterns`**: Recognizes architectural and design patterns.
- **`trace_inheritance`**: **PRODUCTION-READY** - Python inheritance hierarchy analysis with metaclass and MRO support.
- **`analyze_decorators`**: Comprehensive Python decorator usage with framework detection.
- **`find_duplicates`**: Duplicate code pattern detection with similarity scoring.

### Workflow & Orchestration (4 Tools)

- **`suggest_analysis_workflow`**: Provides intelligent analysis guidance for specific goals.
- **`batch_analysis`**: Executes multiple tools in parallel with result aggregation.
- **`optimize_workflow`**: Optimizes workflow based on usage patterns.
- **`find_references`**: Conducts complete reference analysis across the codebase.

### Parser Development Tools (codeprism-dev-tools)

- **AST Visualizer**: Pretty-print syntax trees with multiple formats (Tree, List, JSON, S-Expression, Compact).
- **Parser Validator**: Comprehensive validation including span overlap detection and edge consistency checking.
- **GraphViz Exporter**: Export ASTs to DOT format for visual analysis with configurable styling.
- **Performance Profiler**: Real-time parsing performance metrics with bottleneck identification.
- **AST Diff Comparison**: Compare parse results between parser versions with detailed change analysis.
- **Development REPL**: Interactive command-line interface for parser development and testing.

**Testing Coverage**: 22 comprehensive unit tests covering all development tool functionality.

## Example Usage

### Repository Analysis

```json
// Get repository overview
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "repository_stats", "arguments": {}}}

// Analyze specific symbol  
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "explain_symbol", "arguments": {"symbol_id": "UserManager"}}}

// Search for patterns
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "search_symbols", "arguments": {"pattern": "^Agent.*", "symbol_types": ["class"]}}}
```

### Production Analysis Tools

```json
// Find unused code with high confidence
{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "find_unused_code", "arguments": {"analyze_types": ["functions", "classes"], "confidence_threshold": 0.9}}}

// Security vulnerability analysis
{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "analyze_security", "arguments": {"vulnerability_types": ["injection", "xss"], "severity_threshold": "medium"}}}

// Performance bottleneck detection
{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "analyze_performance", "arguments": {"analysis_types": ["time_complexity", "hot_spots"], "complexity_threshold": "medium"}}}
```

### Python-Specific Analysis

```json
// Trace inheritance hierarchies
{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "trace_inheritance", "arguments": {"class_id": "Agent", "include_mro_analysis": true}}}

// Analyze decorator usage
{"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "analyze_decorators", "arguments": {"scope": "global", "framework_detection": true}}}

// Detect metaprogramming patterns
{"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {"name": "detect_patterns", "arguments": {"pattern_types": ["design_patterns", "anti_patterns"]}}}
```

### Workflow Orchestration

```json
// Get analysis recommendations
{"jsonrpc": "2.0", "id": 10, "method": "tools/call", "params": {"name": "suggest_analysis_workflow", "arguments": {"goal": "understand_codebase"}}}

// Run multiple tools in parallel
{"jsonrpc": "2.0", "id": 11, "method": "tools/call", "params": {"name": "batch_analysis", "arguments": {"tool_calls": [{"tool_name": "repository_stats"}, {"tool_name": "content_stats"}], "execution_strategy": "parallel"}}}
```

## Tool Response Format

All tools return responses in the standardized MCP format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Tool response content in JSON format"
      }
    ],
    "isError": false
  }
}
```

## Additional Resources

- **[Parser Development Tools Guide](../parsers/development-tools)**: Comprehensive guide for parser debugging and development
- **[Current Status](../architecture/current-status)**: Latest implementation status and achievements
- **[MCP Server Overview](overview)**: MCP server implementation details

**Total Test Coverage**: 425 tests across all crates with 100% tool success rate.