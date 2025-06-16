# Rust Parser Implementation Plan

## Overview

The Rust parser implementation enables gcore to analyze its own source code, providing the ultimate "dogfooding" capability. This parser will handle Rust's unique features like ownership, traits, macros, and complex type system.

## 🎯 Primary Goal: Self-Analysis

**Use Case**: Enable gcore to analyze its own Rust codebase for:
- Code quality assessment
- Dependency analysis  
- Refactoring opportunities
- Architecture understanding
- Performance optimization insights

## 🏗️ Implementation Roadmap

### Phase 1: Basic Structure (Week 1)
1. **Crate Setup** (`crates/gcore-lang-rust/`)
   - Cargo.toml with tree-sitter-rust dependency
   - Basic module structure following established pattern
   - Initial error handling and types

2. **Core Parser Implementation**
   - Tree-sitter integration with Rust grammar
   - Language detection for `.rs` files
   - Basic incremental parsing support

### Phase 2: AST Mapping (Week 2-3)
1. **Basic Node Types**
   - Functions (`fn`, `async fn`, `const fn`, `unsafe fn`)
   - Structs (`struct`, `tuple struct`, `unit struct`)
   - Enums (`enum` with variants)
   - Modules (`mod`, `use` declarations)
   - Constants and static variables

2. **Advanced Node Types**
   - Traits (`trait`, `impl` blocks)
   - Generics and lifetime parameters
   - Pattern matching (`match`, `if let`, `while let`)
   - Macros (`macro_rules!`, procedural macros)

### Phase 3: Relationship Analysis (Week 4)
1. **Basic Edges**
   - Function calls
   - Module imports (`use`)
   - Struct field access
   - Method calls

2. **Advanced Edges**
   - Trait implementations
   - Generic constraints
   - Lifetime relationships
   - Macro invocations

### Phase 4: Rust-Specific Features (Week 5-6)
1. **Ownership Analysis**
   - Borrow checker implications
   - Move semantics
   - Reference relationships

2. **Type System**
   - Type aliases
   - Associated types
   - Where clauses
   - Complex generics

## 📋 Detailed Implementation Guide

### Crate Structure

```
crates/gcore-lang-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── parser.rs           # Main parser implementation
│   ├── ast_mapper.rs       # CST to U-AST conversion
│   ├── rust_nodes.rs       # Rust-specific node handling
│   ├── traits.rs           # Trait and impl analysis
│   ├── macros.rs           # Macro analysis
│   ├── types.rs            # Type system analysis
│   ├── patterns.rs         # Pattern matching analysis
│   └── error.rs            # Error handling
├── tests/
│   ├── fixtures/
│   │   ├── simple.rs       # Basic Rust features
│   │   ├── advanced.rs     # Complex generics and traits
│   │   ├── macros.rs       # Macro usage
│   │   ├── patterns.rs     # Pattern matching
│   │   └── gcore_sample.rs # Real gcore code samples
│   └── integration_test.rs
└── benches/
    └── parse_benchmark.rs
```

### Cargo.toml

```toml
[package]
name = "gcore-lang-rust"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Rust language support for gcore - enables self-analysis"

[dependencies]
# Core dependencies
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
serde_json.workspace = true

# Tree-sitter
tree-sitter.workspace = true
tree-sitter-rust.workspace = true

# GCore types
blake3.workspace = true
hex.workspace = true

[dev-dependencies]
insta.workspace = true
tempfile.workspace = true
tokio = { workspace = true, features = ["test-util"] }

[build-dependencies]
cc = "1.0"
```

### Key Implementation Challenges

#### 1. Macro Analysis
```rust
// Challenge: Analyze macro invocations and expansions
// Examples from gcore codebase:
tracing::info!("Starting server");
serde_json::json!({ "key": value });
```

**Approach**:
- Extract macro name and arguments
- Track macro definition locations
- Analyze macro usage patterns

#### 2. Trait Implementation Analysis
```rust
// Challenge: Map trait bounds and implementations
impl<T: Clone + Debug> Display for Wrapper<T> 
where 
    T: Send + Sync,
{
    // Implementation
}
```

**Approach**:
- Extract trait names and bounds
- Map implementation relationships
- Track generic constraints

#### 3. Pattern Matching
```rust
// Challenge: Analyze complex pattern matching
match result {
    Ok(ParseResult { nodes, edges, .. }) => {
        // Handle success
    }
    Err(Error::Parse { file, message }) => {
        // Handle parse error
    }
}
```

**Approach**:
- Extract pattern structures
- Map variable bindings
- Track control flow

#### 4. Module System
```rust
// Challenge: Track complex module relationships
use gcore::{
    ast::{Node, Edge},
    parser::ParserEngine,
};
```

**Approach**:
- Parse `use` declarations
- Track module hierarchy
- Map public/private visibility

### Rust-Specific Node Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RustNodeKind {
    // Basic items
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    
    // Type system
    TypeAlias,
    AssociatedType,
    GenericParam,
    LifetimeParam,
    
    // Patterns
    MatchArm,
    Pattern,
    
    // Macros
    MacroDefinition,
    MacroInvocation,
    
    // Expressions
    MethodCall,
    FieldAccess,
    TupleAccess,
    
    // Statements
    LetBinding,
    UseDeclaration,
}
```

### Rust-Specific Edge Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RustEdgeKind {
    // Trait relationships
    Implements,     // impl Trait for Type
    TraitBound,     // T: Trait
    
    // Ownership
    Borrows,        // &value
    MutBorrows,     // &mut value
    Moves,          // ownership transfer
    
    // Type relationships
    HasType,        // variable: Type
    GenericArg,     // Vec<T>
    
    // Macro relationships
    Expands,        // macro expansion
    Invokes,        // macro call
    
    // Module system
    ReExports,      // pub use
    Imports,        // use path
}
```

## 🧪 Testing Strategy

### Unit Tests
1. **Parser Tests**
   - Basic Rust syntax parsing
   - Error recovery
   - Incremental updates

2. **AST Mapper Tests**
   - Node extraction accuracy
   - Edge relationship correctness
   - Rust-specific feature handling

### Integration Tests
1. **Real Code Analysis**
   - Parse actual gcore source files
   - Verify extracted relationships
   - Performance benchmarks

2. **Self-Analysis Tests**
   - Analyze gcore-lang-rust itself
   - Cross-reference with known structure
   - Validate completeness

### Test Fixtures

#### `tests/fixtures/simple.rs`
```rust
// Basic Rust features for testing
use std::collections::HashMap;

pub struct User {
    pub name: String,
    age: u32,
}

impl User {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
    
    pub fn greet(&self) -> String {
        format!("Hello, I'm {}", self.name)
    }
}

pub fn create_user(name: &str, age: u32) -> User {
    User::new(name.to_string(), age)
}
```

#### `tests/fixtures/advanced.rs`
```rust
// Advanced Rust features
use std::marker::PhantomData;

pub trait Parser<T> {
    type Error;
    type Output;
    
    fn parse(&self, input: T) -> Result<Self::Output, Self::Error>;
}

pub struct LanguageParser<L>
where
    L: Language + Clone,
{
    language: L,
    _phantom: PhantomData<L>,
}

impl<L> Parser<&str> for LanguageParser<L>
where
    L: Language + Clone + Send + Sync,
{
    type Error = ParseError;
    type Output = ParseResult;
    
    fn parse(&self, input: &str) -> Result<Self::Output, Self::Error> {
        // Implementation
        todo!()
    }
}
```

#### `tests/fixtures/gcore_sample.rs`
```rust
// Real gcore code sample for testing
use anyhow::Result;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub span: Span,
}

impl Node {
    pub fn new(
        repo_id: &str,
        kind: NodeKind,
        name: String,
        span: Span,
    ) -> Self {
        let id = NodeId::generate(repo_id, &span, &kind);
        Self { id, kind, name, span }
    }
}
```

## 🚀 Integration with Existing System

### Registry Integration
```rust
// In crates/gcore/src/parser/mod.rs
impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Register existing parsers
        #[cfg(feature = "javascript")]
        registry.register_javascript();
        
        #[cfg(feature = "python")]
        registry.register_python();
        
        // Register Rust parser
        #[cfg(feature = "rust")]
        registry.register_rust();
        
        registry
    }
    
    #[cfg(feature = "rust")]
    fn register_rust(&mut self) {
        use gcore_lang_rust::RustLanguageParser;
        self.register(Box::new(RustLanguageParser::new()));
    }
}
```

### MCP Server Integration
The Rust parser will automatically be available through the MCP server for:
- Repository analysis including Rust files
- Cross-language dependency tracking
- Self-analysis capabilities

### CLI Integration
```bash
# Analyze gcore itself
gcore-mcp /path/to/gcore

# Focus on Rust files only
gcore analyze --language rust /path/to/gcore
```

## 📊 Success Metrics

### Functionality Metrics
- [ ] Parse 100% of gcore Rust source files without errors
- [ ] Extract 95%+ of function/struct/trait definitions
- [ ] Correctly identify 90%+ of function calls and dependencies
- [ ] Handle complex generics and trait bounds

### Performance Metrics
- [ ] Parse gcore codebase (~50k LOC) in < 2 seconds
- [ ] Incremental updates < 10ms for typical file changes
- [ ] Memory usage < 100MB for full gcore analysis

### Self-Analysis Capabilities
- [ ] Generate accurate module dependency graph
- [ ] Identify circular dependencies
- [ ] Extract trait implementation hierarchy
- [ ] Analyze macro usage patterns

## 🎯 Future Enhancements

### Advanced Analysis
1. **Ownership Analysis**
   - Track borrow checker implications
   - Identify potential memory issues
   - Suggest ownership optimizations

2. **Performance Analysis**
   - Identify allocation patterns
   - Suggest performance improvements
   - Track async/await usage

3. **Architecture Analysis**
   - Module cohesion metrics
   - Trait design patterns
   - API surface analysis

### Integration Features
1. **IDE Integration**
   - Real-time analysis in IDEs
   - Refactoring suggestions
   - Code quality metrics

2. **CI/CD Integration**
   - Automated architecture checks
   - Dependency drift detection
   - Code quality gates

## 🎉 Benefits for gcore Project

### Immediate Benefits
1. **Self-Analysis**: Understand gcore's own architecture
2. **Quality Assurance**: Automated code quality checks
3. **Refactoring Support**: Safe restructuring with dependency awareness

### Long-term Benefits
1. **Architecture Evolution**: Track and guide architectural changes
2. **Performance Optimization**: Data-driven performance improvements
3. **Educational Value**: Demonstrate gcore capabilities on complex Rust code

### Community Benefits
1. **Reference Implementation**: Example of advanced Rust parsing
2. **Open Source Contribution**: Enhance tree-sitter-rust ecosystem
3. **Tool Validation**: Real-world validation of gcore capabilities

---

This implementation plan provides a comprehensive roadmap for adding Rust parser support to gcore, enabling powerful self-analysis capabilities while following established patterns and maintaining high code quality standards. 