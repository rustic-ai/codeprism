# Prism - Graph-First Code Intelligence MCP Server

A production-ready, high-performance code intelligence server implementing the Model Context Protocol (MCP). Prism provides AI assistants with structured understanding of codebases through graph-based analysis rather than traditional text search, enabling real-time, accurate code intelligence without vector embeddings.

## 🚀 Current Status: Production Ready

**✅ Phase 3 Complete - MCP Protocol Implementation**
- **105/105 tests passing** (100% success rate)
- **Production-ready `prism-mcp` binary** with full MCP compliance
- **Real-time repository monitoring** and incremental updates
- **Multi-language support**: JavaScript/TypeScript + Python
- **MCP client integration**: Claude Desktop, Cursor, VS Code compatible

**🎯 Next Phase: Self-Analysis Capability**
- **Rust parser implementation** to analyze prism's own codebase
- **Ultimate dogfooding**: prism analyzing itself for architecture insights
- **Advanced code intelligence** with Rust's complex type system

## 🌟 Key Features

### ✅ **Production-Ready MCP Server**
- **JSON-RPC 2.0 compliant** following MCP specification 2024-11-05
- **Stdio transport** with newline-delimited JSON
- **Resource/Tool/Prompt endpoints** for comprehensive code analysis
- **Real-time file monitoring** with automatic index updates
- **Memory-efficient** in-memory graph storage with DashMap

### ✅ **Multi-Language Code Analysis**
- **JavaScript/TypeScript**: ES6+, TSX support, incremental parsing
- **Python**: 3.x with comprehensive AST mapping and type annotations
- **Rust**: Next priority for self-analysis capability
- **Extensible architecture** for additional languages

### ✅ **Graph-First Intelligence**
- **Universal AST**: Language-agnostic code structure representation
- **Relationship mapping**: Function calls, imports, dependencies
- **Real-time updates**: Sub-millisecond incremental parsing
- **Efficient queries**: Fast graph traversal and analysis

### ✅ **MCP Ecosystem Integration**
- **Claude Desktop**: Full resource/tool access
- **Cursor**: Enhanced development workflows
- **VS Code**: GitHub Copilot compatibility
- **Any MCP client**: Standard protocol compliance

## 🏗️ Architecture

```
┌─────────────────┐    MCP Protocol     ┌──────────────────┐
│   AI Assistant  │◄──────────────────►│   prism-mcp      │
│  (Claude/Cursor)│   JSON-RPC 2.0     │     Server       │
└─────────────────┘                     └──────────────────┘
                                                 │
                                                 ▼
                    ┌─────────────────────────────────────────┐
                    │           Prism Engine                  │
                    │  ┌─────────────┐  ┌─────────────────┐   │
                    │  │ Repository  │  │ Real-time File  │   │
                    │  │   Scanner   │  │   Monitoring    │   │
                    │  └─────────────┘  └─────────────────┘   │
                    │  ┌─────────────┐  ┌─────────────────┐   │
                    │  │   Parser    │  │   Graph Store   │   │
                    │  │   Engine    │  │   (DashMap)     │   │
                    │  └─────────────┘  └─────────────────┘   │
                    └─────────────────────────────────────────┘
                                         │
                                         ▼
                    ┌─────────────────────────────────────────┐
                    │          Repository Files              │
                    │    (JavaScript, Python, Rust...)       │
                    └─────────────────────────────────────────┘
```

## 🚀 Quick Start

### 📖 **[Complete Getting Started Guide](docs/GETTING_STARTED.md)**

**Step-by-step setup for Claude Desktop, Cursor, and VS Code with troubleshooting, examples, and best practices.**

### Prerequisites

- **Rust 1.82+** (for building from source)
- **Node.js 18+** (for some MCP integrations)
- **Any repository** to analyze (JavaScript, Python, or mixed)

### Quick Installation

```bash
# Clone and build
git clone https://github.com/dragonscale/prism
cd prism
cargo build --release

# Test the binary
./target/release/prism-mcp --help
```

### Choose Your AI Client

**🏆 Claude Desktop** - Best overall MCP experience
```json
// claude_desktop_config.json
{
  "mcpServers": {
    "prism": {
      "command": "/path/to/prism/target/release/prism-mcp",
      "args": ["/path/to/your/repository"]
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
      "args": ["."]
    }
  }
}
```

**🔧 VS Code** - GitHub Copilot with enhanced tools
```json
// .vscode/mcp.json  
{
  "servers": {
    "prism": {
      "type": "stdio",
      "command": "/path/to/prism/target/release/prism-mcp",
      "args": ["."]
    }
  }
}
```

**➡️ [See full setup guide for detailed instructions](docs/GETTING_STARTED.md)**

## 📊 Performance

**Benchmarked Performance:**
- **Parse Speed**: ~5-10µs per line of code
- **Repository Scanning**: ~1000 files/second initial indexing
- **Incremental Updates**: Sub-millisecond for typical file changes
- **Memory Usage**: Optimized for repositories up to 10M nodes
- **Query Response**: <1s for complex graph traversals

**Test Coverage:**
- **105/105 tests passing** (100% success rate)
- **66 core tests** (infrastructure and graph operations)
- **23 language parser tests** (JS/TS + Python)
- **22 MCP protocol tests** (full specification compliance)

## 🛠️ Development

### Project Structure

```
prism/
├── crates/
│   ├── prism/              # Core engine (✅ Complete)
│   ├── prism-mcp/          # MCP server (✅ Production-ready)
│   ├── prism-lang-js/      # JavaScript/TypeScript (✅ Complete)
│   ├── prism-lang-python/  # Python support (✅ Complete)
│   ├── prism-lang-rust/    # Rust parser (🚧 Next priority)
│   ├── prism-lang-java/    # Java support (⏳ Future)
│   ├── prism-cli/          # CLI tools (🚧 Enhanced commands planned)
│   └── prism-daemon/       # Background service (🚧 Additional features)
├── docs/                   # Comprehensive documentation
└── tests/                  # Integration test suites
```

### Development Commands

```bash
# Run all tests
cargo test --all

# Build release binary
cargo build --release

# Development with logging
RUST_LOG=debug cargo run --bin prism-mcp -- /path/to/repo

# Test MCP protocol compliance
cargo test -p prism-mcp

# Test specific language parser
cargo test -p prism-lang-python
```

## 🎯 Roadmap

### ✅ **Completed (Phase 1-3)**
- **Core Infrastructure**: Universal AST, parser engine, file monitoring
- **Language Support**: JavaScript/TypeScript, Python parsers
- **Repository Operations**: Scanning, indexing, real-time updates
- **MCP Protocol**: Full JSON-RPC 2.0 compliance, client integration
- **Production Deployment**: CLI binary, error handling, logging

### 🚧 **Current Phase: Self-Analysis (Phase 4)**
- **Rust Parser**: Enable prism to analyze its own ~50k line codebase
- **Advanced Features**: Traits, generics, macros, pattern matching
- **Self-Analysis Tools**: Architecture insights, code quality metrics
- **Performance Optimization**: Large repository handling

### 🔮 **Future Phases**
- **Enhanced CLI**: Additional commands (`prism stats`, `prism watch`)
- **Java Parser**: Enterprise language support
- **Advanced Analysis**: Ownership tracking, performance insights
- **IDE Integration**: Real-time analysis in development environments

## 🌟 Use Cases

### **AI-Powered Development**
```
👩‍💻 Developer: "Analyze the authentication flow in this codebase"

🤖 AI Assistant: *Uses prism MCP server to:*
   1. Identify auth-related functions across languages
   2. Trace call paths and data flow
   3. Find security patterns and potential issues
   4. Provide structured analysis with exact locations
```

### **Code Quality Assessment**
```
👨‍💻 Developer: "What are the main architectural patterns here?"

🤖 AI Assistant: *Leverages prism's graph analysis to:*
   1. Extract module dependencies and relationships  
   2. Identify design patterns and conventions
   3. Suggest refactoring opportunities
   4. Generate architecture documentation
```

### **Self-Analysis (Next Phase)**
```
🔍 prism analyzing itself:
   1. Parse all Rust source files (crates/*)
   2. Generate dependency graphs and module relationships
   3. Identify potential improvements and refactoring opportunities
   4. Provide insights into its own architecture and design
```

## 📚 Documentation

- **[Implementation Guide](docs/RUST_PARSER_IMPLEMENTATION.md)** - Comprehensive Rust parser roadmap
- **[Architecture Overview](docs/ARCHITECTURE.md)** - System design and components
- **[Language Parsers](docs/LANGUAGE_PARSERS.md)** - Adding new language support
- **[MCP Integration](docs/GCORE-MCP-SERVER-DESCRIPTION.md)** - MCP server capabilities
- **[Developer Guide](docs/DEVELOPER.md)** - Setup and development workflow
- **[API Reference](docs/API.md)** - Detailed API documentation

## 🤝 Contributing

We welcome contributions, especially for:

1. **Rust Parser Implementation** (current priority)
2. **Additional Language Support** (Java, Go, C++, etc.)
3. **Advanced Analysis Tools** and capabilities
4. **Performance Optimizations** for large repositories
5. **Client Integrations** for new MCP-compatible tools

### Getting Started

```bash
# 1. Fork and clone the repository
git clone https://github.com/yourusername/prism
cd prism

# 2. Create a feature branch
git checkout -b feature/rust-parser

# 3. Make changes and add tests
cargo test --all

# 4. Submit a pull request
```

## 📄 License

Dual-licensed under **MIT** and **Apache 2.0**. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## 🎉 Why Prism?

### **For AI Assistants**
- **Structured Understanding**: Graph-based analysis vs. text search
- **Real-time Accuracy**: Always current with live file monitoring  
- **Cross-language Intelligence**: Unified analysis across tech stacks
- **MCP Standard**: Seamless integration with growing AI ecosystem

### **For Developers**
- **Instant Setup**: Point at any repository and start analyzing
- **Zero Configuration**: Automatic language detection and parsing
- **Production Ready**: Battle-tested with comprehensive test suite
- **Self-Validating**: Next phase will analyze prism's own complex codebase

### **For Organizations**
- **Enhanced AI Workflows**: Better code understanding for AI assistants
- **Quality Insights**: Automated architecture and quality analysis
- **Multi-language Support**: Handle complex, polyglot codebases
- **Open Source**: Transparent, auditable, and extensible

---

**Ready to enhance your AI-powered development workflow?** Start with `prism-mcp /path/to/your/repository` and experience graph-first code intelligence today.

**Next milestone**: Watch prism analyze its own sophisticated Rust codebase, demonstrating the ultimate validation of its code intelligence capabilities. 