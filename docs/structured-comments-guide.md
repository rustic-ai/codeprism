# Structured Comments Guide

This guide explains how to use the structured comment system that distinguishes between legitimate future work and incomplete implementations.

## Overview

The pre-commit hook now uses intelligent pattern matching to:
- ✅ **Allow** structured markers for legitimate future work
- ❌ **Block** unstructured TODOs and placeholder implementations

## Structured Markers (ALLOWED)

### `FUTURE:` - Planned Enhancements
Use for features planned for future releases that don't affect current functionality.

```rust
// FUTURE: Add support for HTTP/2 transport in v2.0
// FUTURE: Implement connection pooling for better performance  
// FUTURE: Add real-time monitoring dashboard for test execution
pub fn create_connection() -> Connection {
    // Current working implementation
    Connection::new()
}
```

### `NOTE:` - Design Documentation
Use for documenting design decisions, architectural notes, or implementation context.

```rust
// NOTE: We use stdio transport here because it's most compatible across platforms
// NOTE: Error handling follows the established pattern from the validation module
// NOTE: This approach was chosen for compatibility with legacy MCP servers
pub fn establish_transport() -> StdioTransport {
    StdioTransport::new()
}
```

### `ENHANCEMENT:` - Performance/Quality Improvements
Use for identified optimizations that aren't critical for current functionality.

```rust
// ENHANCEMENT: Could optimize this with a HashMap for O(1) lookups instead of O(n)
// ENHANCEMENT: Add caching layer to reduce repeated server connections
// ENHANCEMENT: Consider using async streams for large test result sets
pub fn find_tests(&self, criteria: &SearchCriteria) -> Vec<TestCase> {
    // Current working implementation that meets requirements
    self.tests.iter().filter(|t| criteria.matches(t)).cloned().collect()
}
```

### `PLANNED(#issue):` - Tracked Future Work
Use for work items tracked in GitHub issues. Include issue number for traceability.

```rust
// PLANNED(#125): Add user authentication middleware for secure test execution
// PLANNED(#126): Implement rate limiting for API endpoints
// PLANNED(#127): Add database migration system for test result history
pub fn handle_request(&self, request: Request) -> Response {
    // Current implementation handles basic requests
    self.process_request(request)
}
```

### `CONSIDER:` - Alternative Approaches
Use for documenting alternative implementation approaches that were considered.

```rust
// CONSIDER: Alternative approach using trait objects instead of generics for flexibility
// CONSIDER: Could use macros to reduce boilerplate in similar test functions
// CONSIDER: Async version for non-blocking operations in high-throughput scenarios
pub fn execute_sync<T: TestExecutor>(&self, executor: T) -> Result<TestOutput> {
    // Current synchronous implementation that works for our use cases
    executor.execute(&self.test_data)
}
```

### `TODO(#issue):` - Linked TODOs
Traditional TODOs are allowed ONLY when linked to specific GitHub issues.

```rust
// TODO(#128): Refactor this function to use the new validation framework
// TODO(#129): Add comprehensive error handling for edge cases
pub fn legacy_validator(&self, input: &str) -> bool {
    // Current implementation that works but needs improvement
    !input.is_empty()
}
```

## Blocked Patterns (REJECTED)

These patterns indicate incomplete implementations and WILL block commits:

```rust
// ❌ BLOCKED - Generic TODOs without structure
// TODO: Implement this function
// TODO: Fix this
// TODO: Make this work properly
// TODO: Add error handling

// ❌ BLOCKED - Placeholder implementations
// FIXME: This is broken and needs repair
// XXX: Hack that needs proper solution
// HACK: Temporary workaround
// stub implementation
// placeholder implementation  
// For now, just return placeholder
// Not implemented yet
// Will implement later

// ❌ BLOCKED - Unimplemented macros
fn my_function() -> Result<()> {
    unimplemented!()  // ❌ Will block commit
}

fn my_other_function() -> Result<()> {
    todo!()  // ❌ Will block commit
}

// ❌ BLOCKED - Placeholder tests
#[test]
fn test_something() {
    assert!(true);  // ❌ Meaningless test will block commit
}
```

## Migration Process

### Step 1: Identify Current TODOs
```bash
# Find all existing TODOs
grep -r "TODO:" src/ --include="*.rs"
```

### Step 2: Categorize Each TODO
For each TODO, ask:
- **Is this missing functionality?** → Fix it before commit
- **Is this planned future work?** → Convert to `FUTURE:` or `PLANNED(#issue):`
- **Is this a known optimization?** → Convert to `ENHANCEMENT:`
- **Is this documentation?** → Convert to `NOTE:`

### Step 3: Convert Examples

```rust
// OLD - Would be blocked
// TODO: Add connection pooling

// NEW - Choose the appropriate marker:

// Option A: Future enhancement
// FUTURE: Add connection pooling to improve performance under high load

// Option B: Tracked work item  
// PLANNED(#145): Implement connection pooling for production scalability

// Option C: Performance optimization
// ENHANCEMENT: Consider connection pooling when user base grows beyond 1000 concurrent users
```

## Pre-commit Hook Behavior

### What Gets Checked
- All `.rs` files in `src/` and `crates/*/src/` directories
- Test files in `tests/` and `crates/*/tests/` directories
- Searches for both comment patterns and `unimplemented!()`/`todo!()` macros

### Example Output
```bash
🔍 Step 1/7: Implementation Completeness Check
ℹ️  Checking for incomplete implementations...

✅ No incomplete implementations found

ℹ️  Found future work markers (allowed):
./src/transport/stdio.rs:15:    // FUTURE: Add process management for server lifecycle
./src/protocol/mod.rs:23:       // ENHANCEMENT: Add connection pooling for concurrent tests
./src/testing/runner.rs:45:     // PLANNED(#125): Implement comprehensive protocol testing
... and 12 more

✅ Implementation completeness check passed
```

## Best Practices

### 1. Provide Context
Always explain what, why, and when:

```rust
// ✅ GOOD - Explains what, why, and when
// FUTURE: Add WebSocket transport support for real-time test notifications
//         Planned for v2.0 when we implement live test result streaming
//         Will require protocol extension for bidirectional communication

// ❌ BAD - Lacks context
// FUTURE: Add WebSocket support
```

### 2. Link to Issues When Possible
```rust
// ✅ GOOD - Traceable work
// PLANNED(#123): Implement user roles and permissions system
// TODO(#124): Refactor authentication to use new token system

// ✅ GOOD - Internal planning without external tracking  
// FUTURE: Consider adding audit logging for compliance requirements
```

### 3. Use Appropriate Categories
- `FUTURE:` for new features
- `ENHANCEMENT:` for optimizations
- `NOTE:` for design decisions
- `PLANNED(#issue):` for tracked work
- `CONSIDER:` for alternatives

### 4. Keep Working Code Working
Never use structured markers to hide broken code:

```rust
// ❌ BAD - Hiding broken implementation
// FUTURE: Fix this function that currently crashes
pub fn broken_function() -> Result<()> {
    panic!("This doesn't work")  // This will be blocked!
}

// ✅ GOOD - Working implementation with future enhancement
// ENHANCEMENT: Optimize this function for better performance  
pub fn working_function() -> Result<()> {
    // Current implementation that works correctly
    Ok(())
}
```

## Generating Reports

Extract all future work markers for planning:

```bash
# Generate future work report
grep -r "FUTURE:\|PLANNED:\|ENHANCEMENT:" src/ --include="*.rs" > future_work.md

# Generate by category
grep -r "FUTURE:" src/ --include="*.rs" | sed 's/.*FUTURE:/FUTURE:/' > planned_features.md
grep -r "ENHANCEMENT:" src/ --include="*.rs" > optimizations.md
grep -r "PLANNED(" src/ --include="*.rs" > tracked_work.md
```

## Summary

This system enables you to:
- ✅ Document legitimate future work without blocking commits
- ✅ Maintain high code quality standards
- ✅ Track planned enhancements and optimizations
- ✅ Provide context for future developers
- ❌ Prevent incomplete implementations from being committed
- ❌ Eliminate vague TODOs that provide no value

The key principle: **Current functionality must be complete and working, future work must be well-documented and structured.** 