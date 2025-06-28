# Parser Development Tools Documentation

The `codeprism-dev-tools` crate provides a comprehensive suite of debugging and development utilities specifically designed for CodePrism parser development. These tools enable efficient AST development, validation, performance optimization, and debugging.

## Overview

The parser development tools are organized into six main utilities:

1. **AST Visualizer** - Pretty-print and format syntax trees
2. **Parser Validator** - Comprehensive validation and quality checks
3. **GraphViz Exporter** - Visual diagram generation for ASTs
4. **Performance Profiler** - Real-time performance monitoring and analysis
5. **AST Diff Comparison** - Compare parser versions and track changes
6. **Development REPL** - Interactive parser development environment

## Quick Start

### Installation

The development tools are automatically included when building CodePrism:

```bash
git clone https://github.com/rustic-ai/codeprism
cd codeprism
cargo build --release
```

### Basic Usage

```rust
use codeprism_dev_tools::DevTools;

// Create development tools with default configuration
let dev_tools = DevTools::new();

// Analyze a parse result comprehensively
let report = dev_tools.analyze_parse_result(&parse_result, &source_code)?;
println!("{}", report.format_report());

// Start interactive REPL
dev_tools.start_repl(Some("python")).await?;
```

## Key Features

### AST Visualizer
- **Multiple Output Formats**: Tree, List, JSON, S-Expression, Compact
- **Configurable Display**: Show positions, byte ranges, text content
- **Statistics Collection**: Node counts, depth analysis, type distribution
- **Color Support**: Syntax highlighting for better readability

### Parser Validator
- **Span Overlap Detection**: Identifies inappropriate node overlaps
- **Edge Consistency**: Validates that all edges connect valid nodes
- **Text Coverage**: Ensures complete source text coverage
- **Syntax Tree Structure**: Validates tree-sitter tree integrity

### GraphViz Exporter
- **Multiple Layout Engines**: dot, neato, fdp, circo support
- **Configurable Styling**: Node and edge colors, shapes, labels
- **Clustering Options**: Group nodes by file or type
- **Filtering Support**: Show only specific node or edge types

### Performance Profiler
- **Parse Time**: Time to parse each file
- **Memory Usage**: Peak memory consumption
- **Node Creation**: Number of AST nodes created
- **Bottleneck Detection**: Automatic identification of performance issues

### AST Diff Comparison
- **Structural Comparison**: Detect added, removed, and modified nodes
- **Similarity Scoring**: Quantify changes with percentage similarity
- **Change Impact Assessment**: Classify changes as significant or minor
- **Version Tracking**: Compare parser behavior across versions

### Development REPL
- **Interactive Commands**: Parse, validate, visualize, and profile on-demand
- **Language Support**: Multi-language parser testing
- **Real-time Feedback**: Immediate results for development iteration
- **Export Capabilities**: Generate reports and visualizations

## Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `parse <code>` | Parse source code | `parse function test() {}` |
| `load <file>` | Load and parse file | `load examples/sample.js` |
| `show <target>` | Display AST, stats, etc. | `show ast`, `show stats` |
| `export <format>` | Export visualization | `export graphviz output.dot` |
| `validate` | Run validation checks | `validate` |
| `profile <cmd>` | Profile command execution | `profile parse complex.js` |
| `compare <old> <new>` | Compare two results | `compare old.js new.js` |
| `help` | Show available commands | `help` |

## Testing

The development tools include comprehensive test coverage:

- **22 unit tests** covering all major functionality
- **Integration tests** with real parser output
- **Performance benchmarks** for profiling accuracy
- **Validation test cases** for edge conditions

Run tests with:

```bash
cd crates/codeprism-dev-tools
cargo test
```

## Integration

The development tools integrate seamlessly with the CodePrism ecosystem and can be used to debug and improve MCP tool implementations, validate parser output, and profile performance across the entire system.
