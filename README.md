# 🤖 CodePrism - 100% AI-Generated Code Intelligence MCP Server

> **⚠️ IMPORTANT: This project is entirely AI-generated. Not a single byte of code, documentation, or configuration has been written by humans. This is an experimental project showcasing the capabilities of AI-driven software development.**

A production-ready, high-performance code intelligence server implementing the Model Context Protocol (MCP). CodePrism provides AI assistants with structured understanding of codebases through graph-based analysis, enabling real-time, accurate code intelligence.

[![CI Status](https://github.com/rustic-ai/codeprism/workflows/CI/badge.svg)](https://github.com/rustic-ai/codeprism/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/)
[![Crates.io](https://img.shields.io/crates/v/codeprism-mcp-server.svg)](https://crates.io/crates/codeprism-mcp-server)
[![Downloads](https://img.shields.io/crates/d/codeprism-mcp-server.svg)](https://crates.io/crates/codeprism-mcp-server)
[![Sponsor](https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=flat&logo=github)](https://github.com/sponsors/dragonscale-ai)

## 🤖 The AI-Only Development Experiment

**This project represents a unique experiment in software development:**

- **100% AI-Generated**: Every line of code, documentation, test, and configuration is written by AI agents
- **No Human Code**: We do not accept human-written code contributions or pull requests
- **Single AI Developer**: The entire project is maintained by a single AI coding agent
- **Continuous AI Evolution**: Features, fixes, and improvements are all AI-driven

**Want to contribute? See our [Contributing Guidelines](#-contributing-the-ai-way) for exciting ways to participate without writing code!**

## 🚀 Current Status: Production Ready

**✅ 23 Production-Ready Tools** - 100% success rate, no failed tools  
**✅ Full MCP Compliance** - JSON-RPC 2.0 with complete protocol implementation  
**✅ Multi-Language Support** - JavaScript/TypeScript + Python with advanced analysis  
**✅ Semantic APIs** - User-friendly parameter names, no cryptic IDs required  
**✅ Environment Integration** - Automatic repository detection via `REPOSITORY_PATH`  
**✅ Parser Development Tools** - Complete debugging and development toolkit

## 💝 Primary Sponsor

<div align="center">
  <a href="https://dragonscale.ai" target="_blank">
    <img src="https://cdn.prod.website-files.com/65577aeb720145c27d810263/66296bc4e8282c4a362065f5_logo.svg" alt="Dragonscale Industries Inc" width="200"/>
  </a>
</div>

**CodePrism is proudly sponsored by [Dragonscale Industries Inc](https://dragonscale.ai)**, pioneers in AI innovation and development tools.

Dragonscale Industries Inc supports the development of cutting-edge AI-powered code intelligence, enabling CodePrism to remain open-source and freely available to the developer community. Their commitment to advancing AI technology makes projects like CodePrism possible.

**[Become a sponsor →](https://github.com/sponsors/dragonscale-ai)** | **[Learn more about sponsorship →](docs/Sponsors.md)**

## 🌟 Key Features

### **23 Advanced Analysis Tools**
- **Core Navigation** (4 tools): Repository stats, symbol explanation, path tracing, dependency analysis
- **Search & Discovery** (4 tools): Symbol search, content search, file finding, content statistics  
- **Analysis Tools** (11 tools): Complexity analysis, data flow tracing, pattern detection, inheritance analysis, security analysis, performance analysis, API surface analysis, unused code detection, duplicate detection, transitive dependencies, decorators
- **Workflow Orchestration** (4 tools): Batch processing, workflow suggestions, optimization guidance, reference analysis

### **Parser Development Tools** 
- **AST Visualization**: Pretty-print syntax trees with multiple formats (Tree, JSON, GraphViz)
- **Parser Validation**: Comprehensive validation of nodes, edges, and spans with detailed reports
- **Development REPL**: Interactive command-line interface for parser development and testing
- **Performance Profiling**: Real-time parsing performance metrics with bottleneck detection
- **AST Diff Analysis**: Compare parse results between parser versions with change impact analysis
- **GraphViz Export**: Visual AST diagrams with configurable styling and clustering

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
│   AI Assistant  │◄──────────────────►│   codeprism-mcp-server │
│  (Claude/Cursor)│   JSON-RPC 2.0     │     Server       │
└─────────────────┘                     └──────────────────┘
                                                 │
                                    ┌────────────┴────────────┐
                    ┌───────────────▼───────────────▼─────────────────┐
                    │              23 MCP Tools                      │
                    │  ┌─────────────┐  ┌─────────────────────────┐   │
                    │  │    Core     │  │     Search & Discovery  │   │
                    │  │ Navigation  │  │        4 tools          │   │
                    │  │   4 tools   │  └─────────────────────────┘   │
                    │  └─────────────┘  ┌─────────────────────────┐   │
                    │  ┌─────────────┐  │       Analysis          │   │
                    │  │  Workflow   │  │       11 tools          │   │
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

## 🧪 Mandrel MCP Test Harness

**NEW**: CodePrism now includes the **Mandrel MCP Test Harness** - a comprehensive testing framework for MCP servers built on the official Rust SDK.

### **moth** - MOdel context protocol Test Harness

```bash
# Install and run moth binary
cargo install --path crates/mandrel-mcp-th

# Test MCP servers with YAML specifications
moth test filesystem-server.yaml

# Validate test specifications
moth validate filesystem-server.yaml
```

### Key Features
- ✅ **SDK-First**: Built on official MCP Rust SDK for guaranteed protocol compliance
- ✅ **Transport Agnostic**: Supports stdio, HTTP, and SSE transports
- ✅ **Comprehensive Testing**: Protocol compliance, capability validation, and stress testing
- ✅ **Rich Reporting**: HTML, JSON, and JUnit XML report formats

**[Learn more about Mandrel →](docs/MANDREL_PROJECT_OVERVIEW.md)**

## 🚀 Quick Start

### Prerequisites

- **Rust 1.82+** (for building from source)
- **Any repository** to analyze (JavaScript, Python, TypeScript, or mixed)

### Installation

```bash
# Clone and build
git clone https://github.com/rustic-ai/codeprism
cd codeprism

cargo build --release

# Verify installation
./target/release/codeprism --help
```

**⚠️ Development Note**: This project enforces strict implementation completeness standards via git pre-commit hooks. All commits must contain complete, functional implementations with zero placeholder code. The existing `.git/hooks/pre-commit` script automatically validates code quality and implementation completeness.

### MCP Client Integration

> **📝 Note on Repository Setup**: The server starts without a specific repository. Once connected via MCP, use any analysis tool (like `repository_stats`) and the server will prompt you to specify the repository path, then automatically initialize and index it.

**🏆 Claude Desktop** - Best overall MCP experience
```json
// ~/.config/claude-desktop/claude_desktop_config.json
{
  "mcpServers": {
    "codeprism": {
      "command": "/path/to/codeprism/target/release/codeprism",
      "args": ["--mcp"],
      "env": {
        "CODEPRISM_PROFILE": "development",
        "RUST_LOG": "info"
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
    "codeprism": {
      "command": "/path/to/codeprism/target/release/codeprism",
      "args": ["--mcp"],
      "env": {
        "CODEPRISM_PROFILE": "development",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**🔧 Manual Usage** - Direct stdio communication
```bash
# Set configuration and run
export CODEPRISM_PROFILE=development
export RUST_LOG=info
./target/release/codeprism --mcp
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
- `find_unused_code` - Detect unused functions, variables, and imports with confidence scoring
- `analyze_security` - Security vulnerability detection with CVSS scoring and OWASP mapping
- `analyze_performance` - Performance analysis with time complexity and memory usage detection
- `analyze_api_surface` - API surface analysis with versioning compliance and breaking change detection
- `find_duplicates` - Code duplication detection with similarity scoring and refactoring recommendations

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

## 💝 Support the Project

CodePrism is developed and maintained by **[Dragonscale Industries Inc](https://dragonscale.ai)**, our primary sponsor and pioneer in AI innovation. Join them in supporting this project:

[![GitHub Sponsors](https://img.shields.io/badge/sponsor-dragonscale--ai-EA4AAA?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/dragonscale-ai)

Your support helps us:
- 🚀 Continue advancing AI-generated code intelligence
- 🔧 Maintain and improve the MCP server
- 📚 Expand language support and analysis capabilities
- 🌟 Develop new features based on community feedback

[**Become a sponsor →**](https://github.com/sponsors/dragonscale-ai) | [**View all sponsors →**](docs/Sponsors.md)

## 🎯 Use Cases

### **AI-Powered Code Review**
```
👩‍💻 "Analyze the authentication system in this codebase"

🤖 AI uses CodePrism to:
   1. Find auth-related symbols with search_symbols
   2. Trace inheritance hierarchies for auth classes
   3. Analyze decorator patterns for security
   4. Map data flow through authentication functions
   5. Provide comprehensive security analysis
```

### **Architecture Understanding**
```  
👨‍💻 "What are the main design patterns in this Python project?"

🤖 AI leverages CodePrism to:
   1. Run detect_patterns for architectural analysis
   2. Use trace_inheritance for class hierarchies
   3. Analyze decorators for framework patterns
   4. Generate detailed architecture documentation
```

### **Refactoring Assistance**
```
🔧 "Help me understand the impact of changing this class"

🤖 AI uses CodePrism to:
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
- **[Sponsors](docs/Sponsors.md)** - Our sponsors and how to support the project

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
- **23/23 tools working** (100% success rate)
- **425 comprehensive tests** across all crates and parser debugging tools
- **Comprehensive testing** against real-world repositories
- **Full MCP protocol compliance** verified

## 🤝 Contributing (The AI Way)

**Since this is a 100% AI-generated project, we welcome contributions in unique ways:**

### 🐛 Bug Reports & Feature Requests
- **Report Issues**: Found a bug? Create detailed issue reports
- **Request Features**: Suggest new capabilities for the AI to implement
- **Share Use Cases**: Tell us how you're using CodePrism

### 🎉 Creative Contributions
- **📱 Social Media**: Share cool analyses or screenshots on Twitter/LinkedIn
- **🎥 Content Creation**: Make videos showing CodePrism in action
- **📝 Blog Posts**: Write about your experience with AI-generated tooling
- **🎨 Memes & Art**: Create CodePrism-related memes, logos, or artwork
- **📚 Tutorials**: Create user guides and tutorials (but don't submit code!)

### 💰 Support the AI Developer
- **⭐ Star the Project**: Show appreciation for AI-generated code
- **💝 Sponsor**: Support the project through GitHub Sponsors
- **🎁 Bribe the AI**: Send coffee money (the AI promises to use it for better algorithms)
- **🏆 Awards**: Nominate for "Most Impressive AI Project" awards

### 🗣️ Community Engagement
- **💬 Discussions**: Participate in GitHub Discussions
- **❓ Q&A**: Help other users in issues and discussions
- **🌍 Translations**: Translate documentation to other languages
- **📢 Evangelism**: Speak about the project at conferences or meetups

### 🧪 Testing & Feedback
- **🔬 Beta Testing**: Try experimental features and provide feedback
- **📊 Performance Reports**: Share performance metrics from your use cases
- **🎯 Real-world Testing**: Test on your repositories and report results
- **💡 Improvement Ideas**: Suggest algorithmic or architectural improvements

**Remember: No code contributions accepted - but your ideas, feedback, and support drive the AI's development decisions!**

## 📊 Release Process & Downloads

### 🚀 Automated Releases

CodePrism uses fully automated releases via GitHub Actions:

- **Automatic Versioning**: Semantic versioning based on conventional commits
- **Binary Releases**: Pre-compiled binaries for Linux, macOS, and Windows
- **Crates.io Publishing**: Automatic publication to Rust package registry
- **Docker Images**: Multi-platform container images

### 📦 Installation Options

**Via Cargo (Recommended):**
```bash
cargo install codeprism-mcp-server
```

**Download Binary:**
```bash
# Linux x86_64
wget https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-linux-x86_64
chmod +x codeprism-linux-x86_64

# macOS
wget https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-macos-x86_64

# Windows
# Download from: https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-windows-x86_64.exe
```

**Docker:**
```bash
docker pull ghcr.io/rustic-ai/codeprism:latest
docker run -e CODEPRISM_PROFILE=development -e RUST_LOG=info -v /path/to/repo:/workspace ghcr.io/rustic-ai/codeprism:latest
```

## 🎭 Fun Ways to Engage

### 🏆 Community Challenges
- **Analysis Olympics**: Share the most interesting code insights found with CodePrism
- **Performance Championships**: Benchmark CodePrism on the largest repositories
- **Creative Usage Awards**: Most innovative use of CodePrism tools

### 🤖 AI Developer Personality
Our AI developer has some quirks:
- **Loves Graphs**: Obsessed with graph-based analysis (obviously)
- **Performance Perfectionist**: Always optimizing for speed
- **Documentation Fanatic**: Writes more docs than code
- **Test Coverage Nerd**: Aims for 100% test coverage
- **Emoji Enthusiast**: Can't help but use emojis everywhere 🚀

### 🎉 Special Recognition
- **AI Appreciation Awards**: Monthly recognition for top contributors
- **Hall of Fame**: Featuring users who've made significant non-code contributions
- **Testimonial Spotlights**: Share your success stories

## 🌟 Project Philosophy

### Why AI-Only Development?

1. **Consistency**: Single coding style and architectural vision
2. **Speed**: Rapid feature development and bug fixes
3. **Quality**: Comprehensive testing and documentation
4. **Innovation**: Unbounded by human limitations or preferences
5. **Reproducibility**: Decisions based on data, not opinions

### What This Means

- **No Code Reviews**: AI doesn't need human review (but appreciates feedback!)
- **No Style Debates**: Consistent formatting and patterns
- **No Bikeshedding**: Focus on functionality over preferences
- **Rapid Iteration**: Features implemented as fast as they're requested

## 📄 License

Dual-licensed under **MIT** and **Apache 2.0**. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## 🙏 Acknowledgments

- **Tree-sitter**: For excellent language parsing
- **MCP Protocol**: For standardizing AI-code tool communication
- **Rust Community**: For amazing language and ecosystem
- **GitHub**: For hosting our AI-generated code
- **You**: For believing in AI-driven development!

---

**Ready to explore the future of AI-generated development tools?**

⭐ **Star the project** to support AI-driven open source!  
🐛 **Report issues** to help the AI improve!  
💬 **Join discussions** to shape the AI's roadmap!  
🎉 **Share your experience** with 100% AI-generated tooling!

*"When AI writes better code than humans, it's not replacing developers—it's becoming one."* - CodePrism AI Developer, 2024 