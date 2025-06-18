# Prism - Graph-First Code Intelligence MCP Server

A production-ready, high-performance code intelligence server implementing the Model Context Protocol (MCP). Prism provides AI assistants with structured understanding of codebases through graph-based analysis, enabling real-time, accurate code intelligence.

## 🚀 Current Status: Production Ready

**✅ 18 Production-Ready Tools** - 100% success rate, no failed tools  
**✅ Full MCP Compliance** - JSON-RPC 2.0 with complete protocol implementation  
**✅ Multi-Language Support** - JavaScript/TypeScript + Python with advanced analysis  
**✅ Semantic APIs** - User-friendly parameter names, no cryptic IDs required  
**✅ Environment Integration** - Automatic repository detection via `REPOSITORY_PATH`  

## 🌟 Key Features

### **18 Advanced Analysis Tools**
- **Core Navigation** (4 tools): Repository stats, symbol explanation, path tracing, dependency analysis
- **Search & Discovery** (4 tools): Symbol search, content search, file finding, content statistics  
- **Analysis Tools** (6 tools): Complexity analysis, data flow tracing, pattern detection, inheritance analysis
- **Workflow Orchestration** (4 tools): Batch processing, workflow suggestions, optimization guidance

### **Advanced Python Analysis**
- **Inheritance Tracing**: Complete hierarchy analysis with metaclass support
- **Decorator Analysis**: Framework detection (Flask, Django, FastAPI) and pattern recognition
- **Metaprogramming Support**: Complex pattern detection and dynamic behavior analysis

### **Graph-First Intelligence**
- **Universal AST**: Language-agnostic code structure representation
- **Relationship Mapping**: Function calls, imports, dependencies, inheritance
- **Real-time Updates**: Sub-millisecond incremental parsing
- **Efficient Queries**: Fast graph traversal and semantic search

## 🏗️ Architecture

```
┌─────────────────┐    MCP Protocol     ┌──────────────────┐
│   AI Assistant  │◄──────────────────►│   prism-mcp      │
│  (Claude/Cursor)│   JSON-RPC 2.0     │     Server       │
└─────────────────┘                     └──────────────────┘
                                                 │
                                    ┌────────────┴────────────┐
                    ┌───────────────▼───────────────▼─────────────────┐
                    │              18 MCP Tools                      │
                    │  ┌─────────────┐  ┌─────────────────────────┐   │
                    │  │    Core     │  │     Search & Discovery  │   │
                    │  │ Navigation  │  │        4 tools          │   │
                    │  │   4 tools   │  └─────────────────────────┘   │
                    │  └─────────────┘  ┌─────────────────────────┐   │
                    │  ┌─────────────┐  │       Analysis          │   │
                    │  │  Workflow   │  │       6 tools           │   │
                    │  │ 4 tools     │  │                         │   │
                    │  └─────────────┘  └─────────────────────────┘   │
                    └─────────────────────────────────────────────────┘
                                         │
                                         ▼
                    ┌─────────────────────────────────────────────────┐
                    │          Graph-Based Code Analysis              │
                    │    JavaScript/TypeScript + Python Support      │
                    └─────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.82+** (for building from source)
- **Any repository** to analyze (JavaScript, Python, TypeScript, or mixed)

### Installation

```bash
# Clone and build
git clone https://github.com/dragonscale/prism
cd prism
cargo build --release

# Verify installation
./target/release/prism-mcp --help
```

### MCP Client Integration

**🏆 Claude Desktop** - Best overall MCP experience
```json
// ~/.config/claude-desktop/claude_desktop_config.json
{
  "mcpServers": {
    "prism": {
      "command": "/path/to/prism/target/release/prism-mcp",
      "env": {
        "REPOSITORY_PATH": "/path/to/your/repository"
      }
    }
  }
}
```

**⚡ Cursor** - AI pair programming with code intelligence
```json
// .cursor/mcp.json
{
  "mcpServers": {
    "prism": {
      "command": "/path/to/prism/target/release/prism-mcp",
      "env": {
        "REPOSITORY_PATH": "."
      }
    }
  }
}
```

**🔧 Manual Usage** - Direct stdio communication
```bash
# Set repository path and run
export REPOSITORY_PATH=/path/to/your/repository
./target/release/prism-mcp
```

## 🛠️ Available Tools

### **Core Navigation & Understanding**
- `repository_stats` - Get comprehensive repository overview and statistics
- `explain_symbol` - Detailed symbol analysis with context (accepts semantic names like "UserManager")
- `trace_path` - Find execution paths between code elements
- `find_dependencies` - Analyze what a symbol or file depends on

### **Search & Discovery**
- `search_symbols` - Advanced symbol search with regex and inheritance filtering
- `search_content` - Full-text search across all repository content
- `find_files` - File discovery with glob and regex pattern support
- `content_stats` - Detailed content and complexity statistics

### **Analysis Tools**
- `analyze_complexity` - Code complexity metrics and maintainability analysis
- `trace_data_flow` - Forward and backward data flow analysis
- `analyze_transitive_dependencies` - Complete dependency chains with cycle detection
- `detect_patterns` - Architectural and design pattern recognition
- `trace_inheritance` - Python inheritance hierarchy with metaclass analysis
- `analyze_decorators` - Python decorator analysis with framework detection

### **Workflow & Orchestration**
- `suggest_analysis_workflow` - Intelligent analysis guidance for specific goals
- `batch_analysis` - Parallel execution of multiple tools with result aggregation
- `optimize_workflow` - Workflow optimization based on usage patterns
- `find_references` - Complete reference analysis across the codebase

## 📊 Example Usage

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

## 🎯 Use Cases

### **AI-Powered Code Review**
```
👩‍💻 "Analyze the authentication system in this codebase"

🤖 AI uses Prism to:
   1. Find auth-related symbols with search_symbols
   2. Trace inheritance hierarchies for auth classes
   3. Analyze decorator patterns for security
   4. Map data flow through authentication functions
   5. Provide comprehensive security analysis
```

### **Architecture Understanding**
```  
👨‍💻 "What are the main design patterns in this Python project?"

🤖 AI leverages Prism to:
   1. Run detect_patterns for architectural analysis
   2. Use trace_inheritance for class hierarchies
   3. Analyze decorators for framework patterns
   4. Generate detailed architecture documentation
```

### **Refactoring Assistance**
```
🔧 "Help me understand the impact of changing this class"

🤖 AI uses Prism to:
   1. Find all references with find_references
   2. Analyze transitive dependencies
   3. Trace inheritance impact on subclasses
   4. Assess complexity before/after changes
```

## 📚 Documentation

### **Setup & Usage**
- **[Getting Started Guide](docs/GETTING_STARTED.md)** - Complete setup instructions for all MCP clients
- **[API Documentation](docs/API.md)** - Detailed tool and resource reference
- **[Current Status](docs/CURRENT_STATUS.md)** - Implementation status and capabilities

### **Technical Documentation**
- **[Architecture Overview](docs/ARCHITECTURE.md)** - System design and components
- **[MCP Server Description](docs/PRISM-MCP-SERVER-DESCRIPTION.md)** - Complete MCP capabilities
- **[Language Parsers](docs/LANGUAGE_PARSERS.md)** - Multi-language support details
- **[Developer Guide](docs/DEVELOPER.md)** - Development setup and contribution guide

### **Planning & Roadmap**
- **[Future Roadmap](docs/FUTURE_ROADMAP.md)** - Potential future enhancements
- **[Large Repository Guide](docs/LARGE_REPOSITORY_GUIDE.md)** - Performance optimization tips

## 🚀 Performance

**Benchmarked Performance:**
- **Repository Indexing**: ~1000 files/second for initial scanning
- **Tool Response Time**: <1s for complex analysis on 3000+ file repositories  
- **Memory Efficiency**: Optimized for repositories up to 10M+ nodes
- **Query Speed**: Sub-millisecond for most symbol and content searches

**Test Coverage:**
- **18/18 tools working** (100% success rate)
- **Comprehensive testing** against real-world repositories
- **Full MCP protocol compliance** verified

## 🤝 Contributing

We welcome contributions for:

1. **Additional Language Support** (Rust, Java, Go, C++, etc.)
2. **Enhanced Analysis Tools** (security, performance, quality tools)
3. **MCP Client Integrations** for new AI tools
4. **Performance Optimizations** for large repositories

### Getting Started

```bash
# Fork and clone
git clone https://github.com/yourusername/prism
cd prism

# Run tests
cargo test --all

# Build and test
cargo build --release
./target/release/prism-mcp --help
```

## 📄 License

Dual-licensed under **MIT** and **Apache 2.0**. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## 🎉 Why Prism?

### **For AI Assistants**
- **Structured Code Understanding**: Graph-based analysis vs. text search
- **Semantic Parameter Support**: Use meaningful names like "UserManager" instead of hex IDs
- **Real-time Accuracy**: Always current with incremental updates
- **Advanced Python Analysis**: Metaclass, inheritance, and decorator expertise

### **For Developers**  
- **Zero Configuration**: Point at any repository and start analyzing
- **Production Ready**: 18 tools with 100% success rate
- **Comprehensive Analysis**: From basic navigation to advanced metaprogramming
- **MCP Standard**: Works with Claude Desktop, Cursor, and future AI tools

### **For Teams**
- **Enhanced AI Workflows**: Better code understanding for AI assistants
- **Quality Insights**: Automated architecture and complexity analysis
- **Multi-language Support**: JavaScript, TypeScript, and Python codebases
- **Open Source**: Transparent, auditable, and extensible

---

**Ready to enhance your AI-powered development workflow?**

Set `REPOSITORY_PATH` to your repository and experience graph-first code intelligence with 18 production-ready analysis tools.

For detailed setup instructions, see **[Getting Started Guide](docs/GETTING_STARTED.md)**. 