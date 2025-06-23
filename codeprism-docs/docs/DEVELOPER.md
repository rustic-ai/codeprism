# Developer Guide

This guide covers the development workflow, architecture, and best practices for contributing to CodeCodePrism.

## Table of Contents

- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Style](#code-style)
- [Architecture Overview](#architecture-overview)
- [Adding Language Support](#adding-language-support)
- [Debugging](#debugging)
- [Performance](#performance)

## Development Setup

### Prerequisites

- **Rust**: 1.82+ with `rustfmt` and `clippy`
- **Docker**: For development services (Neo4j, Kafka)
- **Make**: For build automation
- **Git**: Version control

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/rustic-ai /codeprism
cd codeprism

# Install Rust toolchain components
rustup component add rustfmt clippy

# Install development tools
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-watch     # File watching
cargo install cargo-expand    # Macro expansion

# Start development services
make dev-up

# Verify setup
make check
```

### Development Services

The project uses Docker Compose for development dependencies:

```yaml
# docker-compose.yml
services:
  neo4j:
    image: neo4j:5.15
    ports: ["7474:7474", "7687:7687"]
    environment:
      NEO4J_AUTH: neo4j/password
      
  kafka:
    image: confluentinc/cp-kafka:latest
    ports: ["9092:9092"]
    
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

Start services:
```bash
make dev-up    # Start all services
make dev-down  # Stop all services
make dev-logs  # View service logs
```

## Project Structure

```
prism/
├── crates/                    # Rust workspace crates
│   ├── codeprism/                # Core library
│   │   ├── src/
│   │   │   ├── lib.rs        # Public API exports
│   │   │   ├── ast/          # Universal AST types
│   │   │   ├── parser/       # Parser engine
│   │   │   ├── patch/        # Graph patch system
│   │   │   ├── watcher/      # File system watcher
│   │   │   └── error.rs      # Error types
│   │   ├── tests/            # Integration tests
│   │   └── Cargo.toml
│   │
│   ├── codeprism-lang-js/        # JavaScript/TypeScript parser
│   │   ├── src/
│   │   │   ├── lib.rs        # Public API
│   │   │   ├── parser.rs     # Main parser logic
│   │   │   ├── ast_mapper.rs # CST to U-AST conversion
│   │   │   ├── adapter.rs    # Integration adapter
│   │   │   ├── types.rs      # Language-specific types
│   │   │   └── error.rs      # Error handling
│   │   ├── tests/
│   │   │   ├── fixtures/     # Test files
│   │   │   └── integration_test.rs
│   │   ├── build.rs          # Build script
│   │   └── Cargo.toml
│   │
│   ├── codeprism-lang-python/    # Python parser (planned)
│   ├── codeprism-lang-java/      # Java parser (planned)
│   ├── codeprism-storage/        # Neo4j integration (planned)
│   ├── codeprism-bus/            # Kafka integration (planned)
│   ├── codeprism-mcp/            # MCP server (planned)
# (CLI and daemon components have been removed)
│
├── docs/                     # Documentation
│   ├── DEVELOPER.md          # This file
│   ├── API.md               # API documentation
│   ├── ARCHITECTURE.md      # System architecture
│   └── LANGUAGE_PARSERS.md  # Language parser guide
│
├── Cargo.toml               # Workspace configuration
├── Makefile                 # Build automation
├── docker-compose.yml       # Development services
└── README.md               # Project overview
```

### Crate Organization

Each crate follows Rust conventions:

- **`src/lib.rs`**: Public API and re-exports
- **`src/error.rs`**: Error types using `thiserror`
- **`src/types.rs`**: Core data structures
- **`tests/`**: Integration tests
- **`benches/`**: Performance benchmarks (when needed)
- **`examples/`**: Usage examples

## Development Workflow

### Daily Development

```bash
# Start file watcher for continuous testing
cargo watch -x "test --all"

# Run specific crate tests
cargo test -p codeprism-lang-js

# Check code formatting and linting
make check

# Generate documentation
make doc

# Run benchmarks
cargo bench
```

### Making Changes

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/new-language-parser
   ```

2. **Write tests first** (TDD approach):
   ```bash
   # Add test cases
   cargo test --test integration_test -- --nocapture
   ```

3. **Implement the feature**:
   ```bash
   # Use cargo-expand to debug macros
   cargo expand --package codeprism-lang-js
   ```

4. **Verify quality**:
   ```bash
   make check      # Format, lint, test
   make coverage   # Generate coverage report
   ```

5. **Update documentation**:
   ```bash
   cargo doc --no-deps --open
   ```

### Code Quality Checks

The project enforces quality through:

```bash
# Formatting (required)
cargo fmt --all

# Linting (required)
cargo clippy --all-targets --all-features -- -D warnings

# Testing (required)
cargo test --all

# Coverage (target: 80%+)
cargo tarpaulin --out Html --all-features

# Documentation (required for public APIs)
cargo doc --no-deps
```

## Testing

### Test Organization

```
tests/
├── fixtures/           # Test data files
│   ├── simple.js      # Basic JavaScript
│   ├── typescript.ts  # TypeScript features
│   └── imports.js     # Import/export patterns
├── integration_test.rs # End-to-end tests
└── common/            # Test utilities
    └── mod.rs
```

### Test Categories

1. **Unit Tests**: In `src/` files using `#[cfg(test)]`
2. **Integration Tests**: In `tests/` directory
3. **Documentation Tests**: In doc comments
4. **Benchmark Tests**: In `benches/` directory

### Writing Tests

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generation() {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        let id1 = NodeId::new("repo", Path::new("file.js"), &span, &NodeKind::Function);
        let id2 = NodeId::new("repo", Path::new("file.js"), &span, &NodeKind::Function);
        
        assert_eq!(id1, id2); // Same inputs = same ID
    }
}

// Integration test example
#[test]
fn test_parse_real_file() {
    let fixture_path = get_fixture_path("complex.js");
    let content = fs::read_to_string(&fixture_path).unwrap();
    
    let mut parser = JavaScriptParser::new();
    let context = ParseContext {
        repo_id: "test".to_string(),
        file_path: fixture_path,
        old_tree: None,
        content,
    };
    
    let result = parser.parse(&context).unwrap();
    
    // Verify expected nodes
    assert!(result.nodes.iter().any(|n| n.name == "expectedFunction"));
}
```

### Test Fixtures

Create realistic test files in `tests/fixtures/`:

```javascript
// tests/fixtures/react-component.jsx
import React, { useState } from 'react';

export function Counter({ initialValue = 0 }) {
    const [count, setCount] = useState(initialValue);
    
    const increment = () => setCount(c => c + 1);
    
    return (
        <div>
            <span>Count: {count}</span>
            <button onClick={increment}>+</button>
        </div>
    );
}
```

### Coverage Requirements

- **Overall**: 80%+ test coverage
- **Core crates**: 85%+ coverage
- **Language parsers**: 80%+ coverage
- **Critical paths**: 95%+ coverage

Generate coverage reports:
```bash
make coverage
open tarpaulin-report.html
```

## Code Style

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) with these additions:

1. **Error Handling**: Use `thiserror` for error types
2. **Async Code**: Use `tokio` for async runtime
3. **Serialization**: Use `serde` with appropriate derives
4. **Documentation**: Document all public APIs

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse {file}: {message}")]
    Parse { file: PathBuf, message: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, ParseError>;
```

### Documentation Standards

```rust
/// Parse a JavaScript or TypeScript file into Universal AST.
///
/// This function performs incremental parsing when an old tree is provided,
/// which significantly improves performance for small edits.
///
/// # Arguments
///
/// * `context` - Parse context containing file path, content, and optional old tree
///
/// # Returns
///
/// Returns a `ParseResult` containing the syntax tree, extracted nodes, and edges.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - The file contains syntax errors
/// - The file encoding is invalid
/// - Tree-sitter fails to parse
///
/// # Examples
///
/// ```rust
/// use codeprism_lang_js::{JavaScriptParser, ParseContext};
/// 
/// let mut parser = JavaScriptParser::new();
/// let context = ParseContext {
///     repo_id: "my-repo".to_string(),
///     file_path: PathBuf::from("app.js"),
///     old_tree: None,
///     content: "function hello() {}".to_string(),
/// };
/// 
/// let result = parser.parse(&context)?;
/// assert!(!result.nodes.is_empty());
/// ```
pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
    // Implementation...
}
```

## Architecture Overview

### Core Components

1. **Universal AST** (codeprism::ast`):
   - Language-agnostic representation
   - Stable NodeId generation with Blake3
   - Serializable types

2. **Parser Engine** (codeprism::parser`):
   - Language registry for parser plugins
   - Incremental parsing support
   - Thread-safe operation

3. **File Watcher** (codeprism::watcher`):
   - Real-time file system monitoring
   - Debouncing for performance
   - Async event streams

4. **Graph Patches** (codeprism::patch`):
   - Incremental graph updates
   - Serializable patch format
   - Batch operations

### Data Flow

```
File Change → Watcher → Parser → AST → Patch → Storage → MCP Server → LLM
```

### Thread Safety

All components are designed for concurrent access:

- **Parser Engine**: Uses `DashMap` for thread-safe registry
- **File Watcher**: Async with `tokio`
- **Language Parsers**: Wrapped in `Mutex` for safety

## Adding Language Support

### 1. Create New Crate

```bash
# Create crate structure
mkdir crates/codeprism-lang-python
cd crates/codeprism-lang-python

# Initialize Cargo.toml
cat > Cargo.toml << EOF
[package]
name = "codeprism-lang-python"
version.workspace = true
edition.workspace = true

[dependencies]
tree-sitter.workspace = true
tree-sitter-python = "0.20"
# ... other dependencies
EOF
```

### 2. Implement Parser

```rust
// src/parser.rs
use tree_sitter::{Parser, Tree};

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::language())
            .expect("Failed to load Python grammar");
        
        Self { parser }
    }
    
    pub fn parse(&mut self, context: &ParseContext) -> Result<ParseResult> {
        // Implementation similar to JavaScript parser
    }
}
```

### 3. Implement AST Mapper

```rust
// src/ast_mapper.rs
impl AstMapper {
    fn visit_node(&mut self, cursor: &TreeCursor) -> Result<()> {
        match cursor.node().kind() {
            "function_definition" => self.handle_function(cursor)?,
            "class_definition" => self.handle_class(cursor)?,
            "import_statement" => self.handle_import(cursor)?,
            // ... other node types
            _ => {}
        }
        Ok(())
    }
}
```

### 4. Add Tests

```rust
// tests/integration_test.rs
#[test]
fn test_parse_python_function() {
    let content = r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"
    
class Person:
    def __init__(self, name: str):
        self.name = name
    "#;
    
    let result = parse_python(content).unwrap();
    assert!(result.nodes.iter().any(|n| n.name == "greet"));
}
```

## Debugging

### Logging

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(content))]
pub fn parse_file(path: &Path, content: &str) -> Result<ParseResult> {
    info!("Parsing file: {}", path.display());
    debug!("Content length: {} bytes", content.len());
    
    // ... parsing logic
    
    info!("Extracted {} nodes", result.nodes.len());
    Ok(result)
}
```

### Tree-Sitter Debugging

Debug tree-sitter parsing:

```rust
#[cfg(test)]
fn debug_tree_structure() {
    let tree = parser.parse(content, None).unwrap();
    let mut cursor = tree.walk();
    
    fn print_tree(cursor: &mut TreeCursor, depth: usize) {
        let node = cursor.node();
        println!("{}{} [{:?}]", 
            "  ".repeat(depth), 
            node.kind(), 
            node.start_byte()..node.end_byte()
        );
        
        if cursor.goto_first_child() {
            loop {
                print_tree(cursor, depth + 1);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
    
    print_tree(&mut cursor, 0);
}
```

### Performance Profiling

Use `criterion` for benchmarking:

```rust
// benches/parse_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_large_file(c: &mut Criterion) {
    let content = include_str!("../tests/fixtures/large.js");
    let mut parser = JavaScriptParser::new();
    
    c.bench_function("parse_large_js", |b| {
        b.iter(|| {
            let context = ParseContext {
                repo_id: "bench".to_string(),
                file_path: PathBuf::from("large.js"),
                old_tree: None,
                content: black_box(content.to_string()),
            };
            parser.parse(&context).unwrap()
        })
    });
}

criterion_group!(benches, bench_parse_large_file);
criterion_main!(benches);
```

## Performance

### Optimization Guidelines

1. **Minimize Allocations**: Use string slices where possible
2. **Batch Operations**: Group related operations
3. **Cache Results**: Cache expensive computations
4. **Profile Regularly**: Use `cargo flamegraph`

### Performance Targets

- **Parse Speed**: < 5µs per line of code
- **Memory Usage**: < 2GB for 10M nodes
- **Update Latency**: < 250ms for typical file
- **Query Response**: < 1s for complex queries

### Monitoring

```rust
use std::time::Instant;

#[instrument]
pub fn parse_with_timing(&mut self, context: &ParseContext) -> Result<ParseResult> {
    let start = Instant::now();
    let result = self.parse(context)?;
    let duration = start.elapsed();
    
    let lines = context.content.lines().count();
    let us_per_line = duration.as_micros() as f64 / lines as f64;
    
    info!("Parsed {} lines in {:?} ({:.2}µs/line)", 
          lines, duration, us_per_line);
    
    Ok(result)
}
```

## Continuous Integration

The project uses GitHub Actions for CI:

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

### Pre-commit Hooks

Set up pre-commit hooks:

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."
make check

echo "All checks passed!"
```

## Getting Help

- **Documentation**: `cargo doc --open`
- **Issues**: GitHub Issues for bugs and features
- **Discussions**: GitHub Discussions for questions
- **Code Review**: All changes require review

## Next Steps

1. Read the API Documentation (coming soon)
2. Review the [Architecture Guide](./Architecture.md)
3. Try implementing a simple language parser
4. Contribute to existing parsers or core functionality 