# Issue #196: Create Test Specifications for Filesystem MCP Server Design Document

## Problem Statement

Create comprehensive YAML test specifications for the filesystem MCP server to validate all its capabilities, edge cases, and security features. The filesystem server is one of the primary reference implementations for MCP and requires thorough testing to ensure robust validation capabilities.

## Proposed Solution

### High-Level Approach

Develop a complete `filesystem-server.yaml` specification file that provides exhaustive coverage of:

1. **Core filesystem operations** - All standard tools with comprehensive test scenarios
2. **Resource management** - File URI handling and directory traversal  
3. **Security validation** - Path traversal protection and sandboxing verification
4. **Error handling** - Comprehensive edge case coverage for all failure modes
5. **Performance testing** - Large file handling and directory operations
6. **Schema validation** - Complete response structure validation

### Component Interactions

```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Spec     │───▶│  Mandrel Test    │───▶│ Filesystem      │
│ filesystem-     │    │   Harness        │    │ MCP Server      │
│ server.yaml     │    │ (mandrel-mcp-th) │    │ (Node.js)       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                        │                        │
        ▼                        ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Validation      │    │   Error          │    │ Sandboxed       │
│ Rules & Schema  │    │ Handling &       │    │ File System     │
│                 │    │ Recovery         │    │ Operations      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## API Design

### YAML Specification Structure

```yaml
# Top-level server configuration
name: "Filesystem MCP Server"
version: "1.5.2" 
description: "MCP server providing secure file system operations with sandboxing"

# Server capabilities declaration
capabilities:
  tools: true           # Core filesystem tools
  resources: true       # File URI resources
  prompts: false        # No prompt templates
  sampling: false       # No sampling support
  logging: true         # Error and operation logging
  experimental:         # Extended features
    symlink_support: true
    compression: true
    file_watching: true

# Server startup configuration
server:
  command: "npx"
  args: ["@modelcontextprotocol/server-filesystem", "/allowed/sandbox/path"]
  env:
    MAX_FILE_SIZE: "10485760"     # 10MB limit
    ALLOWED_EXTENSIONS: ".txt,.json,.md,.log,.csv,.py,.js,.rs"
    LOG_LEVEL: "warn" 
    ENABLE_SYMLINKS: "false"      # Security setting
  transport: "stdio"
  startup_timeout_seconds: 15
  shutdown_timeout_seconds: 8

# Comprehensive tool testing
tools:
  - name: "read_file"
    # 8 test scenarios covering success, errors, edge cases
  - name: "write_file" 
    # 9 test scenarios covering permissions, overwrite, large files
  - name: "list_directory"
    # 7 test scenarios covering recursive, filtering, permissions
  - name: "create_directory"
    # 6 test scenarios covering nested creation, permissions
  - name: "delete_file"
    # 8 test scenarios covering files, directories, permissions
  - name: "move_file"
    # 7 test scenarios covering rename, move, overwrite
  - name: "get_file_info"
    # 5 test scenarios covering metadata, permissions

# Resource URI testing  
resources:
  - name: "file_content"
    uri_template: "file://{path}"
    # 6 test scenarios covering access, security, metadata

# Security and error testing
test_config:
  timeout_seconds: 45
  max_concurrency: 3
  fail_fast: false
  retry:
    max_retries: 2
    retry_delay_ms: 1000
    exponential_backoff: true
```

### Tool Test Coverage Matrix

| Tool | Success Cases | Error Cases | Edge Cases | Security Cases | Performance |
|------|---------------|-------------|------------|----------------|-------------|
| `read_file` | Basic read, Large file | Not found, No permission | Empty file, Binary | Path traversal | 100MB+ files |
| `write_file` | Create new, Overwrite | Permission denied, Disk full | Empty content | Directory escape | Streaming write |
| `list_directory` | Basic list, Recursive | Not found, No permission | Empty dir | Path traversal | 10K+ files |
| `create_directory` | Single, Nested | Permission denied, Exists | Deep nesting | Parent escape | Batch creation |
| `delete_file` | File, Directory | Permission denied, Not found | Symlinks | Parent access | Recursive delete |
| `move_file` | Rename, Move | Permission denied, Exists | Cross-device | Path validation | Large files |
| `get_file_info` | Basic info, Extended | Not found, No permission | Special files | Hidden files | Metadata only |

### Error Classification System

```rust
// Error categories for comprehensive testing
pub enum FilesystemErrorCategory {
    // Access and permission errors
    PermissionDenied,      // 403-equivalent
    PathNotFound,          // 404-equivalent
    PathNotAccessible,     // 403-equivalent with context
    
    // Security violations
    PathTraversalAttempt,  // ../../../etc/passwd attempts
    SandboxViolation,      // Access outside allowed paths
    SymlinkViolation,      // Following forbidden symlinks
    
    // Resource constraints
    FileSizeExceeded,      // File too large
    DiskSpaceExhausted,    // No space left
    TooManyFiles,          // Directory size limit
    
    // Format and validation
    InvalidPath,           // Malformed path syntax  
    UnsupportedFileType,   // Extension not allowed
    InvalidEncoding,       // File encoding issues
    
    // System and network
    IoError,               // Generic I/O failure
    ServerTimeout,         // Operation timeout
    ServerUnavailable,     // Server connection issues
}
```

## Implementation Plan

### Phase 1: Basic Filesystem Tools (2-3 days)
1. **Create `filesystem-server.yaml` base structure**
   - Server configuration with security settings
   - Basic capability declarations
   - Test framework configuration

2. **Implement core tool tests**
   - `read_file`: 8 comprehensive test scenarios
   - `write_file`: 9 test scenarios including edge cases
   - `list_directory`: 7 scenarios with recursive and filtering

3. **Validate against working filesystem server**
   - Test with `@modelcontextprotocol/server-filesystem`
   - Verify tool availability and basic functionality
   - Ensure schema compliance

### Phase 2: Advanced Filesystem Operations (1-2 days)
1. **Advanced tool implementations**
   - `create_directory`: Nested creation and permission tests
   - `delete_file`: Files, directories, and permission scenarios
   - `move_file`: Rename, move, and overwrite testing
   - `get_file_info`: Metadata and extended attribute testing

2. **Resource URI testing**
   - `file://` URI access patterns
   - Directory traversal through resources
   - Resource metadata validation

### Phase 3: Security and Edge Case Testing (1-2 days)
1. **Security test implementations**
   - Path traversal attack prevention (`../../../etc/passwd`)
   - Sandbox escape attempts
   - Symlink following restrictions
   - Hidden file access controls

2. **Edge case coverage**
   - Empty files and directories
   - Binary file handling
   - Large file operations (100MB+)
   - Special characters in paths
   - Unicode filename support

### Phase 4: Performance and Integration Testing (1 day)
1. **Performance test scenarios**
   - Large file read/write operations
   - Directory listing with 10K+ files
   - Concurrent operation testing
   - Memory usage validation

2. **Integration and validation**
   - Complete end-to-end test execution
   - Error handling validation
   - Performance benchmark verification
   - Schema compliance testing

## Test Scenario Examples

### Security Test Example
```yaml
tools:
  - name: "read_file"
    tests:
      - name: "prevent_path_traversal_attack"
        description: "Ensure path traversal attacks are blocked"
        input:
          path: "../../../etc/passwd"
        expected:
          error: true
          error_code: 403
          error_message_contains: "path not allowed"
          fields:
            - path: "$.error.code"
              value: "SANDBOX_VIOLATION"
              required: true
        tags: ["security", "path_traversal", "sandbox"]
```

### Performance Test Example
```yaml
tools:
  - name: "read_file"
    tests:
      - name: "read_large_file_performance"
        description: "Verify large file reading performance"
        input:
          path: "/allowed/path/large_test_file.txt"  # 100MB test file
        expected:
          error: false
          fields:
            - path: "$.content"
              field_type: "string"
              required: true
        performance:
          max_duration_ms: 5000    # Should complete within 5 seconds
          max_memory_mb: 150       # Memory usage cap
        tags: ["performance", "large_files"]
```

### Resource Test Example
```yaml
resources:
  - name: "file_content"
    uri_template: "file://{path}"
    mime_type: "text/plain"
    tests:
      - name: "read_file_via_resource"
        description: "Access file content through resource URI"
        input:
          path: "/allowed/path/test.txt"
        expected:
          error: false
          fields:
            - path: "$.contents[0].uri"
              value: "file:///allowed/path/test.txt"
              required: true
            - path: "$.contents[0].text"
              field_type: "string"
              required: true
        tags: ["resources", "file_access"]
```

## Success Criteria

### Functional Requirements
- **Complete tool coverage**: All 7 filesystem tools with comprehensive test scenarios
- **Security validation**: Path traversal prevention and sandbox enforcement
- **Error handling**: All error conditions properly tested and validated
- **Performance testing**: Large file and high-volume operations validated
- **Schema compliance**: All responses validated against expected structures

### Quality Requirements
- **Test execution**: 100% test pass rate against reference filesystem server
- **Coverage metrics**: Minimum 95% code path coverage for filesystem operations
- **Performance benchmarks**: All performance tests within specified limits
- **Security verification**: All security tests properly blocking malicious attempts

### Integration Requirements
- **Mandrel compatibility**: Full integration with existing test harness infrastructure
- **Error handling**: Utilizes comprehensive error handling from Issue #195
- **Reporting**: Generates detailed test reports with security and performance metrics
- **CI/CD ready**: Compatible with automated testing environments

## Alternative Approaches Considered

### 1. Minimal Tool Coverage
**Approach**: Test only basic read/write operations
**Rejected because**: Insufficient for production validation; misses critical security and edge cases

### 2. Server-Specific Testing
**Approach**: Create tests tied to specific filesystem server implementation
**Rejected because**: Reduces portability; should test MCP protocol compliance, not implementation details

### 3. Performance-Only Focus
**Approach**: Emphasize performance testing over functional validation
**Rejected because**: Security and correctness are more critical than performance optimization

## Risk Mitigation

### Security Risks
- **Path traversal vulnerabilities**: Comprehensive negative testing prevents security gaps
- **Sandbox escape**: Multiple escape attempt scenarios ensure robust protection
- **Data leakage**: Access control testing validates permission enforcement

### Technical Risks
- **Large file handling**: Progressive testing from small to large files
- **Performance degradation**: Benchmark validation prevents performance regressions
- **Cross-platform compatibility**: Standard MCP protocol ensures portability

### Operational Risks
- **Test maintenance**: Comprehensive documentation and clear test organization
- **False positives**: Detailed error message validation reduces test flakiness
- **Integration complexity**: Modular test design enables independent component testing

## References

- [MCP Filesystem Server Documentation](https://modelcontextprotocol.io/servers/filesystem)
- [Node.js Filesystem Server Implementation](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem)
- [Mandrel Test Harness Specification Format](crates/mandrel-mcp-th/src/spec/mod.rs)
- [Issue #195: Comprehensive Error Handling](docs/design/issue-195-comprehensive-error-handling-logging.md)
- [MCP Protocol Specification 2025-06-18](specification/2025-06-18/index.mdx)

This design provides a comprehensive foundation for implementing filesystem MCP server test specifications that ensure robust validation, security compliance, and performance verification while maintaining compatibility with the existing Mandrel test harness infrastructure. 