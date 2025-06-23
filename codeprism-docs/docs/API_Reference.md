# CodePrism API Reference

## Available Tools

CodePrism offers a variety of tools for code analysis, navigation, and workflow orchestration. Below is a detailed reference of the available tools and their usage.

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

### Workflow & Orchestration

- **`suggest_analysis_workflow`**: Provides intelligent analysis guidance for specific goals.
- **`batch_analysis`**: Executes multiple tools in parallel with result aggregation.
- **`optimize_workflow`**: Optimizes workflow based on usage patterns.
- **`find_references`**: Conducts complete reference analysis across the codebase.

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

<!-- For more detailed usage instructions, refer to the [full API documentation](./API.md). -->