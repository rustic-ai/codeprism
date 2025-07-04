---
title: MCP Server Overview
description: Model Context Protocol server implementation with graph-based code intelligence
sidebar_position: 1
---

# CodePrism MCP Server Overview

CodePrism provides a **production-ready MCP server** that enables AI assistants to understand and navigate codebases through the Model Context Protocol. This page covers the MCP-specific implementation, tools, and integration details.

> **New to CodePrism?** See the [Introduction](../intro) for a general overview, or check the [Architecture](architecture/overview) for technical design details.

## MCP Integration Features

### 🔌 **Complete MCP Compliance**
- **Resources**: Repository files, code symbols, and graph data
- **Tools**: 23 production-ready code analysis and navigation tools  
- **Prompts**: Code analysis templates and workflow guidance
- **Real-time Updates**: Live resource and tool updates

### 🚀 **Production-Ready Capabilities**
- **100% Tool Success Rate** - All 23 tools tested and production-ready
- **Semantic Parameter Support** - Human-readable names instead of cryptic IDs
- **Environment Variable Integration** - Automatic repository detection
- **Context-Enhanced Responses** - Source code snippets included

## MCP Server Configuration

### Server Capabilities
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

### Transport Protocol
- **JSON-RPC 2.0** over stdio
- **Request/Response Model** for tools and resource access
- **Structured Error Handling** with detailed context
- **Real-time Notifications** for resource changes

## Available Resources

### Repository Resources
- `codeprism://repository/` - Repository root information and metadata
- `codeprism://repository/stats` - Comprehensive repository statistics
- `codeprism://repository/config` - Configuration and settings
- `codeprism://repository/tree` - Complete file tree structure
- `codeprism://repository/file/{path}` - Individual file content with analysis

### Graph & Symbol Resources
- `codeprism://graph/repository` - Graph structure and statistics
- `codeprism://symbols/functions` - All function symbols in the repository
- `codeprism://symbols/classes` - All class symbols in the repository
- `codeprism://symbols/variables` - All variable symbols in the repository
- `codeprism://symbols/modules` - All module symbols in the repository

### Analysis Resources 
- `codeprism://metrics/quality_dashboard` - Code quality metrics and technical debt assessment
- `codeprism://architecture/layers` - Architectural layer structure identification
- `codeprism://architecture/patterns` - Detected design patterns and structures
- `codeprism://architecture/dependencies` - High-level dependency analysis

## Production Tools (23 Available)

### Core Navigation & Understanding (4 tools)
1. **`repository_stats`** - Get comprehensive statistics about the repository
2. **`explain_symbol`** - Provide detailed explanation of a code symbol with context
3. **`trace_path`** - Find execution paths between two code symbols  
4. **`find_dependencies`** - Analyze dependencies for a code symbol or file

### Search & Discovery (4 tools)
5. **`search_symbols`** - Search for symbols by name pattern (with regex support)
6. **`search_content`** - Search across all content including documentation and comments
7. **`find_files`** - Find files by name or path pattern
8. **`content_stats`** - Get statistics about indexed content

### Advanced Analysis (11 tools)
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

### Workflow & Orchestration (4 tools)
20. **`suggest_analysis_workflow`** - Get intelligent analysis recommendations for specific goals
21. **`batch_analysis`** - Execute multiple analysis tools in parallel with result aggregation
22. **`optimize_workflow`** - Optimize analysis workflows based on usage patterns
23. **`find_references`** - Find all references to a symbol across the codebase

## Available Prompts

### Repository Overview Prompt
**Name:** `repository_overview`  
**Description:** Generate a comprehensive overview of the repository structure and contents  
**Arguments:**
- `focus_area` (optional): Area to focus on (architecture, dependencies, entry_points, etc.)

### Code Analysis Prompt
**Name:** `code_analysis`  
**Description:** Analyze code quality, patterns, and potential improvements  
**Arguments:**
- `file_pattern` (optional): File pattern to focus analysis on
- `analysis_type` (optional): Type of analysis (quality, security, performance, architecture)

### Debug Assistance Prompt
**Name:** `debug_assistance`  
**Description:** Help debug issues in the codebase with contextual information  
**Arguments:**
- `issue_description` (required): Description of the issue or error
- `affected_files` (optional): Files related to the issue

## Client Integration

### Claude Desktop
Add to your Claude Desktop configuration (`claude_desktop_config.json`):

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

## Usage Patterns

### Basic Tool Usage
```typescript
// Example tool call via MCP
{
  "method": "tools/call",
  "params": {
    "name": "repository_stats",
    "arguments": {
      "include_metrics": true
    }
  }
}
```

### Resource Access
```typescript
// Example resource access
{
  "method": "resources/read",
  "params": {
    "uri": "codeprism://repository/stats"
  }
}
```

### Workflow Optimization
Use the `suggest_analysis_workflow` tool to get intelligent recommendations for your analysis goals, then use `batch_analysis` for efficient parallel execution.

## Next Steps

- **[Installation Guide](getting-started/installation)** - Set up CodePrism MCP server
- **[API Reference](api-reference)** - Complete tools and capabilities documentation
- **[Tools Documentation](tools-documentation)** - Detailed tool usage examples

---

**Ready to use CodePrism?** Follow the [Installation Guide](getting-started/installation) to get started with MCP integration. 