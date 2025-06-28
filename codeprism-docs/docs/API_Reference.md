# CodePrism API Reference

## Available Tools

CodePrism offers **20 production-ready tools** for code analysis, navigation, and workflow orchestration, plus comprehensive **parser development tools**. Below is a detailed reference of the available tools and their usage.

### Core Navigation & Understanding

- **`repository_stats`**: Provides a comprehensive overview and statistics of the repository.
- **`explain_symbol`**: Offers detailed analysis of a symbol with context, accepting semantic names like "UserManager".
- **`trace_path`**: Finds execution paths between code elements.
- **`find_dependencies`**: Analyzes what a symbol or file depends on.

### Search & Discovery

- **`search_symbols`**: Performs advanced symbol search with regex and inheritance filtering.
- **`search_content`**: Conducts full-text search across all repository content.
- **`find_files`**: Discovers files using glob and regex pattern support.
- **`content_stats`**: Provides detailed content and complexity statistics.

### Analysis Tools

- **`analyze_complexity`**: Measures code complexity and maintainability.
- **`trace_data_flow`**: Analyzes forward and backward data flow.
- **`analyze_transitive_dependencies`**: Identifies complete dependency chains with cycle detection.
- **`detect_patterns`**: Recognizes architectural and design patterns.
- **`trace_inheritance`**: Analyzes Python inheritance hierarchy with metaclass support.
- **`analyze_decorators`**: Examines Python decorator usage with framework detection.
- **`find_unused_code`**: **PRODUCTION-READY** - Detects unused functions, variables, and imports with confidence scoring.
- **`analyze_security`**: **PRODUCTION-READY** - Security vulnerability detection with CVSS scoring and OWASP mapping.
- **`analyze_performance`**: **PRODUCTION-READY** - Performance analysis with time complexity and memory usage detection.
- **`analyze_api_surface`**: **PRODUCTION-READY** - API surface analysis with versioning compliance and breaking change detection.

### Workflow & Orchestration

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

```bash
# Get repository overview
{"name": "repository_stats", "arguments": {}}

# Analyze specific symbol  
{"name": "explain_symbol", "arguments": {"symbol": "UserManager"}}

# Search for patterns
{"name": "search_symbols", "arguments": {"pattern": "^Agent.*", "symbol_type": "class"}}
```

### Python-Specific Analysis

```bash
# Trace inheritance hierarchies
{"name": "trace_inheritance", "arguments": {"class_name": "Agent", "include_metaclasses": true}}

# Analyze decorator usage
{"name": "analyze_decorators", "arguments": {"decorator_pattern": "@app.route"}}

# Detect metaprogramming patterns
{"name": "detect_patterns", "arguments": {"pattern_types": ["metaprogramming_patterns"]}}
```

### Workflow Orchestration

```bash
# Get analysis recommendations
{"name": "suggest_analysis_workflow", "arguments": {"goal": "understand_architecture"}}

# Run multiple tools in parallel
{"name": "batch_analysis", "arguments": {"tools": ["repository_stats", "content_stats", "detect_patterns"]}}
```

## Additional Resources

- **[Complete API Documentation](API_Reference)**: Detailed API specifications and examples
- **[Parser Development Tools Guide](./PARSER_DEVELOPMENT_TOOLS.md)**: Comprehensive guide for parser debugging and development
- **[Current Status](./CURRENT_STATUS.md)**: Latest implementation status and achievements

**Total Test Coverage**: 425 tests across all crates with 100% tool success rate.