# Mandrel MCP Test Harness - Crate Structure Design Document

## Problem Statement

Create the foundational structure for the new Mandrel MCP Test Harness crate with proper Rust project setup and dependencies. This will be a complete greenfield implementation using the official rmcp SDK.

## Proposed Solution

### High-Level Architecture

```
mandrel-mcp-th/
├── Cargo.toml              # Dependencies and binary configuration
├── README.md               # Project description and usage
├── src/
│   ├── lib.rs              # Library exports and main modules
│   ├── main.rs             # CLI binary entry point
│   ├── error.rs            # Comprehensive error types
│   ├── client/             # MCP client wrapper
│   │   ├── mod.rs
│   │   └── connection.rs
│   ├── executor/           # Test execution framework
│   │   ├── mod.rs
│   │   └── runner.rs
│   ├── spec/              # YAML specification parsing
│   │   ├── mod.rs
│   │   └── parser.rs
│   ├── validation/        # Protocol and response validation
│   │   ├── mod.rs
│   │   └── engine.rs
│   ├── reporting/         # Test result reporting
│   │   ├── mod.rs
│   │   └── json.rs
│   └── cli/              # Command-line interface
│       ├── mod.rs
│       └── commands.rs
└── tests/                # Integration tests
    ├── integration_test.rs
    └── fixtures/
        ├── filesystem-server.yaml
        ├── everything-server.yaml
        └── weather-server.yaml
```

### Core Dependencies

```toml
[dependencies]
# Official MCP Rust SDK
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main", features = ["client", "transport-child-process", "transport-sse-client"] }

# Core async runtime
tokio = { workspace = true, features = ["full"] }
futures = "0.3"

# CLI framework
clap = { version = "4.0", features = ["derive", "env", "color"] }

# Serialization and configuration
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_yaml = "0.9"

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging and tracing
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Validation
jsonpath-lib = "0.3"
jsonschema = "0.18"

# Utilities
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Implementation Plan

### Step 1: Basic Crate Structure
1. Create `crates/mandrel-mcp-th/` directory
2. Set up `Cargo.toml` with proper dependencies and binary target
3. Create basic module structure with placeholder implementations
4. Add to workspace `Cargo.toml`

### Step 2: Error Type Foundation
1. Define comprehensive error types using `thiserror`
2. Create error categories for different failure modes
3. Implement error context and debugging support

### Step 3: Basic CLI Structure
1. Create `main.rs` with clap-based CLI
2. Define basic command structure (`test`, `validate`, `list`, `version`)
3. Add global flags for verbosity and output format

### Step 4: Core Type Definitions
1. Define basic types for test specifications
2. Create result and report structures
3. Add configuration types

### Step 5: Integration Setup
1. Add crate to workspace
2. Create basic README
3. Set up initial tests

## API Design

### Error Types
```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("MCP protocol error: {0}")]
    Mcp(#[from] rmcp::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Test specification error: {0}")]
    Spec(String),
    
    #[error("Server connection error: {0}")]
    Connection(String),
    
    #[error("Test execution error: {0}")]
    Execution(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### CLI Structure
```rust
#[derive(Parser)]
#[command(name = "moth")]
#[command(about = "Mandrel MCP Test Harness - moth binary for command-line testing")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    #[arg(short, long, global = true, default_value = "json")]
    pub output: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run test specifications
    Test {
        /// Path to test specification file or directory
        spec: PathBuf,
        
        #[arg(short, long)]
        output_file: Option<PathBuf>,
        
        #[arg(short, long)]
        fail_fast: bool,
    },
    
    /// Validate test specifications
    Validate {
        spec: PathBuf,
    },
    
    /// List available tests
    List {
        spec: PathBuf,
    },
    
    /// Show version information
    Version,
}
```

### Core Types
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestSpecification {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub capabilities: McpCapabilities,
    pub server: ServerConfig,
    pub tools: Vec<ToolSpec>,
    pub resources: Vec<ResourceSpec>,
    pub prompts: Vec<PromptSpec>,
    pub test_config: TestConfig,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub transport: Transport,
    pub startup_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestResult {
    pub spec_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub test_results: Vec<IndividualTestResult>,
    pub server_info: Option<McpServerInfo>,
}
```

## Testing Strategy

### Unit Tests
- Error type creation and conversion
- CLI argument parsing
- Basic type serialization/deserialization
- Module structure validation

### Integration Tests
- Crate compilation and basic functionality
- CLI help and version commands
- Module loading and initialization

## Success Criteria

1. ✅ Crate compiles without errors
2. ✅ All dependencies resolve correctly
3. ✅ Basic CLI commands work (help, version)
4. ✅ Module structure is clean and logical
5. ✅ Error types are comprehensive
6. ✅ Project integrates with workspace
7. ✅ README provides clear project description
8. ✅ All tests pass

## Performance Requirements

- Crate compilation time: <30 seconds
- Binary size: <50MB (release build)
- Memory usage: <10MB baseline
- CLI startup time: <100ms

## Security Considerations

- Input validation for all CLI arguments
- Path sanitization for file operations
- Process isolation for server execution
- Resource limits for test execution

## Alternative Approaches Considered

1. **Extending existing test harness**: Rejected due to accumulated technical debt
2. **Using different MCP SDK**: Rejected in favor of official rmcp SDK
3. **Different language (Python/TypeScript)**: Rejected for performance and type safety

## References

- docs/Building_MCP_Clients_with_Rust_SDK.md
- docs/MCP_Test_Harness_Product_Document.md
- external_repos/rust-sdk/
- Issue #188: https://github.com/rustic-ai/codeprism/issues/188 