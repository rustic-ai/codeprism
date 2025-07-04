---
title: MCP Server Overview
description: Model Context Protocol server implementation with graph-based code intelligence
sidebar_position: 3
---

# CodePrism MCP Server Overview

CodePrism provides a **graph-first code intelligence** MCP server that enables LLM applications to understand and navigate codebases through structured relationships rather than traditional text-based search.

## Key Features ⭐

### Production-Ready Capabilities
- **23 Production-Ready MCP Tools** including complexity analysis, flow tracing, and architectural pattern analysis
- **Quality Metrics Dashboard** with technical debt assessment  
- **Context-Enhanced Responses** with source code snippets
- **Multi-Language Support** (Python, JavaScript/TypeScript)
- **Architectural Intelligence** with pattern detection

### Developer Experience
- **Semantic Parameter Support** - Use human-readable names instead of cryptic IDs
- **Environment Variable Integration** - Automatic repository detection
- **100% Tool Success Rate** - All 23 tools are production-ready with no failures
- **Full MCP Compliance** - Complete protocol implementation

## Architecture Integration

### MCP Compliance
CodePrism's MCP server implements the complete MCP specification with support for:
- ✅ **Resources**: Repository files, code symbols, and graph data
- ✅ **Tools**: Code analysis, path tracing, and graph traversal functions  
- ✅ **Prompts**: Code analysis templates and workflow guidance
- ❌ **Sampling**: Not implemented (focusing on deterministic code analysis)

### Server Configuration
```json
{
  "name": "codeprism-mcp-server",
  "version": "0.2.6",
  "capabilities": {
    "resources": {
      "subscribe": true,
      "listChanged": true
    },
    "tools": {
      "listChanged": true
    },
    "prompts": {
      "listChanged": false
    }
  }
}
```

## Core Functionality

### 1. Repository Scanning and Indexing

When pointed to a codebase, CodePrism performs comprehensive analysis:

**Languages Supported:**
- ✅ **JavaScript/TypeScript**: Complete ES6+ and TSX support with Tree-sitter parsing
- ✅ **Python**: Full Python 3.x with comprehensive AST mapping
- 🚧 **Rust**: Parser framework ready (planned for future release)
- 🚧 **Java**: Parser framework ready (planned for future release)
- 🔄 **Additional languages**: Via extensible parser architecture

**Scanning Process:**
1. **Directory Traversal**: Recursive scanning with configurable ignore patterns
2. **Language Detection**: Automatic file type identification and parser selection
3. **Parallel Processing**: Concurrent parsing of multiple files for performance
4. **Graph Construction**: Real-time building of code relationship graph
5. **Progress Reporting**: Live updates during bulk indexing operations

### 2. Graph-First Code Intelligence

CodePrism maintains a language-agnostic graph structure where each node represents a code symbol (function, class, variable, module) and edges represent relationships (calls, imports, reads, writes).

## Available Resources

### Repository Resources
- `codeprism://repository/` - Repository root information
- `codeprism://repository/stats` - Repository statistics and metrics
- `codeprism://repository/config` - Repository configuration and settings
- `codeprism://repository/tree` - Complete file tree structure
- `codeprism://repository/file/{path}` - Individual file content with analysis

### Graph Resources
- `codeprism://graph/repository` - Graph structure and statistics

### Symbol Resources
- `codeprism://symbols/functions` - All function symbols in the repository
- `codeprism://symbols/classes` - All class symbols in the repository
- `codeprism://symbols/variables` - All variable symbols in the repository
- `codeprism://symbols/modules` - All module symbols in the repository

### Quality Metrics Resources ⭐ 
- `codeprism://metrics/quality_dashboard` - Code quality metrics, complexity analysis, and technical debt assessment

### Architectural Resources ⭐
- `codeprism://architecture/layers` - Architectural layer structure identification
- `codeprism://architecture/patterns` - Detected design patterns and structures
- `codeprism://architecture/dependencies` - High-level dependency analysis

## Tool Categories

### Core Navigation & Understanding (4 tools) ✅
1. **`repository_stats`** - Get comprehensive statistics about the repository
2. **`explain_symbol`** - Provide detailed explanation of a code symbol with context
3. **`trace_path`** - Find execution paths between two code symbols  
4. **`find_dependencies`** - Analyze dependencies for a code symbol or file

### Search & Discovery (4 tools) ✅
5. **`search_symbols`** - Search for symbols by name pattern (with regex support)
6. **`search_content`** - Search across all content including documentation and comments
7. **`find_files`** - Find files by name or path pattern
8. **`content_stats`** - Get statistics about indexed content

### Advanced Analysis (11 tools) ✅
9. **`find_unused_code`** - Graph-based unused code detection with confidence scoring
10. **`analyze_security`** - Advanced vulnerability detection with CVSS scoring
11. **`analyze_performance`** - Performance analysis with time complexity detection
12. **`analyze_api_surface`** - API surface analysis with versioning compliance
13. **`analyze_complexity`** - Calculate complexity metrics (cyclomatic, maintainability index)
14. **`trace_data_flow`** - Bidirectional data flow tracing with transformation tracking
15. **`analyze_transitive_dependencies`** - Complete dependency chain analysis with cycle detection
16. **`detect_patterns`** - Identify design patterns and architectural structures
17. **`trace_inheritance`** - Python inheritance hierarchy analysis with MRO support
18. **`analyze_decorators`** - Comprehensive Python decorator analysis with framework detection
19. **`find_duplicates`** - Duplicate code pattern detection with similarity scoring

### Workflow & Orchestration (4 tools) ✅
20. **`suggest_analysis_workflow`** - Get intelligent analysis recommendations for specific goals
21. **`batch_analysis`** - Execute multiple analysis tools in parallel with result aggregation
22. **`optimize_workflow`** - Optimize analysis workflows based on usage patterns
23. **`find_references`** - Find all references to a symbol across the codebase

## Available Prompts

### 1. Repository Overview Prompt
**Name:** `repository_overview`
**Description:** Generate a comprehensive overview of the repository structure and contents

**Arguments:**
- `focus_area` (optional): Area to focus on (architecture, dependencies, entry_points, etc.)

### 2. Code Analysis Prompt
**Name:** `code_analysis`
**Description:** Analyze code quality, patterns, and potential improvements

**Arguments:**
- `file_pattern` (optional): File pattern to focus analysis on
- `analysis_type` (optional): Type of analysis (quality, security, performance, architecture)

### 3. Debug Assistance Prompt
**Name:** `debug_assistance`
**Description:** Help debug issues in the codebase with contextual information

**Arguments:**
- `issue_description` (required): Description of the issue or error
- `affected_files` (optional): Files related to the issue

## Client Configuration

### Claude Desktop
Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "codeprism": {
      "command": "codeprism",
      "args": ["--mcp"],
      "env": {
        "CODEPRISM_PROJECT_ROOT": "/path/to/your/project"
      }
    }
  }
}
```

### VS Code
```json
{
  "mcp.servers": [
    {
      "name": "codeprism",
      "command": "codeprism",
      "args": ["--mcp"],
      "workspaceFolder": "${workspaceFolder}"
    }
  ]
}
```

### Cursor
```json
{
  "mcp.servers": {
    "codeprism": {
      "command": "codeprism",
      "args": ["--mcp", "--project-root", "${workspaceFolder}"]
    }
  }
}
```

## Next Steps

- **[API Reference](api-reference)** - Complete tools and capabilities documentation
- **[Tools Documentation](tools-documentation)** - Detailed tool usage examples
- **[Installation Guide](../getting-started/installation)** - Set up CodePrism MCP server
- **[Architecture Overview](../architecture/overview)** - Technical architecture details

---

**Ready to use CodePrism?** Follow the [Installation Guide](../getting-started/installation) to get started with MCP integration. 