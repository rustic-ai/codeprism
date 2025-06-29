# Rust Test Project

A comprehensive Rust test project designed to demonstrate various Rust patterns, features, and best practices for CodePrism MCP analysis.

## Features Demonstrated

### Ownership and Borrowing
- Ownership transfer and move semantics
- Immutable and mutable borrowing
- Reference counting with `Arc<T>`
- Interior mutability patterns

### Error Handling
- Result types and error propagation
- Custom error types with `thiserror`
- Error chaining and context
- Comprehensive error reporting

### Async Programming
- `async`/`await` patterns
- Concurrent processing with `tokio::join!`
- Channel-based communication
- Stream processing

### Concurrency
- Thread-safe shared state with `Arc<RwLock<T>>`
- Message passing with channels
- Parallel processing with Rayon
- Lock-free programming patterns

### Design Patterns
- Singleton pattern (thread-safe)
- Observer pattern
- Builder pattern
- Strategy pattern
- Repository pattern

### Unsafe Code
- Raw pointer operations
- Memory alignment
- FFI (Foreign Function Interface)
- Unchecked operations

### Performance Optimization
- Memory pre-allocation
- Iterator chaining and lazy evaluation
- Parallel processing
- Efficient string building

## Project Structure

```
src/
├── main.rs              # Main entry point with comprehensive demos
├── models.rs            # Data models and repository patterns
├── services.rs          # Business logic and service layer
├── patterns.rs          # Design patterns implementation
├── async_patterns.rs    # Async programming patterns
├── unsafe_ops.rs        # Unsafe operations (educational)
├── performance.rs       # Performance optimization patterns
└── errors.rs            # Custom error types and handling
```

## Building and Running

### Prerequisites
- Rust 1.70.0 or later
- Cargo

### Building
```bash
cargo build
```

### Running
```bash
# Run all demonstrations
cargo run

# Run with verbose logging
cargo run -- --verbose

# Run specific demonstration
cargo run -- --demo ownership
cargo run -- --demo async
cargo run -- --demo concurrency
cargo run -- --demo unsafe
cargo run -- --demo performance
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test models::tests
```

### Benchmarks
```bash
cargo bench
```

## Code Analysis Features

This project is designed to test CodePrism's analysis capabilities for:

1. **Ownership Analysis**: Detection of move semantics, borrowing patterns, and lifetime issues
2. **Concurrency Analysis**: Identification of thread-safety patterns and potential race conditions
3. **Error Handling**: Analysis of Result types and error propagation patterns
4. **Performance Patterns**: Recognition of efficient Rust idioms and anti-patterns
5. **Safety Analysis**: Detection of unsafe code blocks and their usage patterns
6. **Design Patterns**: Recognition of common Rust design patterns
7. **Async Patterns**: Analysis of async/await usage and concurrent programming

## Dependencies

The project uses various popular Rust crates to demonstrate real-world patterns:

- **tokio**: Async runtime and utilities
- **serde**: Serialization framework
- **anyhow/thiserror**: Error handling
- **tracing**: Structured logging
- **rayon**: Data parallelism
- **uuid**: Unique identifiers
- **clap**: Command-line parsing

## Safety Notes

This project includes unsafe code demonstrations for educational purposes. All unsafe code is:
- Clearly marked and documented
- Used only for demonstration
- Accompanied by safety explanations
- Not recommended for production use without careful review

## Contributing

This is a test project for CodePrism analysis. When adding new patterns:

1. Ensure comprehensive documentation
2. Include relevant tests
3. Demonstrate both good and questionable practices
4. Add appropriate safety warnings for unsafe code

## License

This project is dual-licensed under MIT OR Apache-2.0, following Rust community conventions. 