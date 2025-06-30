# Comment Standards - Future Work vs Incomplete Implementation

**Purpose:** Distinguish between legitimate future work markers and incomplete placeholder implementations. Allows proper project planning while maintaining implementation quality standards.

**When to use:** Any time you need to mark future work, document planned enhancements, or indicate areas for improvement without blocking commits.

## Allowed Comment Markers

**Rule: Use structured markers for legitimate future work that should NOT block commits.**
Why: These represent planned enhancements, design decisions, or documented technical debt - not incomplete current implementations.

### FUTURE: - Planned Enhancements
Use for features planned for future releases that don't affect current functionality.

```rust
// FUTURE: Add support for HTTP/2 transport in v2.0
// FUTURE: Implement connection pooling for better performance
// FUTURE: Add metrics collection for monitoring dashboard
pub fn create_connection() -> Connection {
    // Current working implementation
    Connection::new()
}
```

### NOTE: - Design Documentation
Use for documenting design decisions, architectural notes, or implementation context.

```rust
// NOTE: We use stdio transport here because it's most compatible across platforms
// NOTE: Error handling follows the established pattern from the validation module
// NOTE: Performance is acceptable for current use cases (<100 users)
pub fn establish_transport() -> StdioTransport {
    StdioTransport::new()
}
```

### ENHANCEMENT: - Performance/Quality Improvements
Use for identified optimizations that aren't critical for current functionality.

```rust
// ENHANCEMENT: Could optimize this with a HashMap for O(1) lookups instead of O(n)
// ENHANCEMENT: Add caching layer to reduce database queries
// ENHANCEMENT: Consider using async stream for large result sets
pub fn find_users(&self, criteria: &SearchCriteria) -> Vec<User> {
    // Current working implementation that meets requirements
    self.users.iter().filter(|u| criteria.matches(u)).cloned().collect()
}
```

### PLANNED(#issue): - Tracked Future Work
Use for work items tracked in GitHub issues. Include issue number for traceability.

```rust
// PLANNED(#125): Add user authentication middleware
// PLANNED(#126): Implement rate limiting for API endpoints  
// PLANNED(#127): Add database migration system
pub fn handle_request(&self, request: Request) -> Response {
    // Current implementation handles unauthenticated requests
    self.process_request(request)
}
```

### CONSIDER: - Alternative Approaches
Use for documenting alternative implementation approaches that were considered.

```rust
// CONSIDER: Alternative approach using trait objects instead of generics
// CONSIDER: Could use macro to reduce boilerplate in similar functions
// CONSIDER: Async version for non-blocking operations
pub fn process_sync<T: Processor>(&self, processor: T) -> Result<Output> {
    // Current synchronous implementation that works for our use case
    processor.process(&self.data)
}
```

## Blocked Comment Markers

**Rule: These markers indicate incomplete implementations and MUST be resolved before commit.**
Why: These suggest the current code doesn't work properly or is missing essential functionality.

### Blocked Patterns (Pre-commit will reject):
```rust
// TODO: Implement this function
// FIXME: This is broken and needs repair
// XXX: Hack that needs proper solution
// HACK: Temporary workaround
// stub implementation
// placeholder implementation
// For now, just return placeholder
// Not implemented yet
// Will implement later
```

## Implementation Guidelines

### Format Requirements
```rust
// ✅ GOOD - Structured with clear category
// FUTURE: Add connection retry logic with exponential backoff
// ENHANCEMENT: Optimize memory usage by reusing allocations
// NOTE: This approach was chosen for compatibility with legacy systems

// ❌ BAD - Vague or indicates incomplete work
// TODO: Fix this
// TODO: Make this work properly
// TODO: Implement error handling
```

### Linking to Issues
```rust
// ✅ GOOD - Links to tracked work
// PLANNED(#123): Implement user roles and permissions system
// FUTURE(#124): Add support for custom validation rules

// ✅ GOOD - Internal planning without external tracking
// FUTURE: Consider adding audit logging for compliance requirements
// ENHANCEMENT: Profile this function to identify optimization opportunities
```

### Context Requirements
All future work markers should include sufficient context:

```rust
// ✅ GOOD - Explains what, why, and when
// FUTURE: Add WebSocket transport support for real-time notifications
//         Planned for v2.0 when we have real-time requirements
//         Will require protocol extension for bidirectional communication

// ❌ BAD - Lacks context  
// FUTURE: Add WebSocket support
```

## Pre-commit Hook Configuration

The pre-commit hook should be updated to:

### Block These Patterns:
- `TODO:` (without issue number)
- `FIXME:`
- `XXX:`
- `HACK:`
- `stub implementation`
- `placeholder implementation`
- `Not implemented`
- `Will implement later`
- Functions returning `unimplemented!()` or `todo!()`

### Allow These Patterns:
- `FUTURE:`
- `NOTE:`
- `ENHANCEMENT:`
- `PLANNED(#\d+):`
- `CONSIDER:`
- `TODO(#\d+):` (when linked to issue)

## Documentation Integration

### Generate Future Work Reports
```bash
# Extract all future work markers
grep -r "FUTURE:\|PLANNED:\|ENHANCEMENT:" src/ --include="*.rs" > future_work.md

# Generate by category
grep -r "FUTURE:" src/ --include="*.rs" | sed 's/.*FUTURE:/FUTURE:/' > planned_features.md
```

### Review Process
```markdown
## Code Review Checklist

### Future Work Markers
- [ ] All TODO comments are either resolved or converted to appropriate markers
- [ ] PLANNED items reference actual GitHub issues
- [ ] FUTURE items have sufficient context for future developers
- [ ] ENHANCEMENT items are genuine improvements, not missing functionality
- [ ] NOTE items document important design decisions
```

## Migration Guide

### Converting Existing TODOs
```bash
# 1. Review all existing TODOs
grep -r "TODO:" src/ --include="*.rs"

# 2. Categorize each one:
#    - Is this missing functionality? → Fix before commit
#    - Is this planned future work? → Convert to FUTURE: or PLANNED(#issue):
#    - Is this a known optimization? → Convert to ENHANCEMENT:
#    - Is this documentation? → Convert to NOTE:

# 3. Update pre-commit hook to use new patterns
```

### Example Conversion
```rust
// OLD - Would be blocked
// TODO: Add connection pooling

// NEW - Allowed patterns
// ENHANCEMENT: Add connection pooling to improve performance under high load
// PLANNED(#145): Implement connection pooling for production scalability  
// FUTURE: Consider connection pooling when user base grows beyond 1000 concurrent users
```

This system allows legitimate future work documentation while maintaining strict standards for current implementation completeness. 