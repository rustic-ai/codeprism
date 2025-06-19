# Dependency Test Project

This test project is designed to thoroughly test and validate the dependency scanning modes implemented in the CodePrism MCP server. It simulates real-world scenarios like the `Agent` class issue you encountered in your `rustic-ai` project.

## Project Structure

```
dependency-test-project/
├── core/                          # Core application modules
│   ├── agent.py                   # Main Agent class (like your original issue)
│   ├── messaging.py               # Message handling system
│   ├── state.py                   # State management
│   └── utils.py                   # Core utilities
├── services/                      # Service layer modules
│   ├── auth.py                    # Authentication service
│   └── database.py                # Database adapter
├── utils/                         # Utility modules
│   ├── config.py                  # Configuration management
│   └── logger.py                  # Logging utilities
├── venv/                          # Mock virtual environment
│   └── lib/python3.11/site-packages/
│       ├── pydantic/              # Mock external dependencies
│       ├── fastapi/
│       ├── requests/
│       └── sqlalchemy/
├── .tox/                          # Mock testing environment
│   └── py311/lib/python3.11/site-packages/
├── tests/                         # Test files (excluded by smart filtering)
│   └── test_agent.py
└── main.py                        # Main application entry point
```

## Key Features

### 1. **Agent Class** (`core/agent.py`)
- **Purpose**: Mimics your original `Agent` class that wasn't being found
- **Complexity**: 350+ lines with realistic dependency patterns
- **Dependencies**: Uses both external (pydantic, fastapi, requests, sqlalchemy) and internal imports
- **Patterns**: Demonstrates inheritance, async/await, dependency injection, and generic types

### 2. **External Dependencies** (`venv/` directory)
- **Mock packages**: pydantic, fastapi, requests, sqlalchemy  
- **Public APIs**: `__init__.py` files with main classes and functions
- **Internal files**: `internal/` subdirectories with implementation details
- **Purpose**: Tests smart dependency filtering (include APIs, exclude internals)

### 3. **Testing Environments** (`.tox/` directory)
- **Simulates**: The `.tox` virtual environments that caused your memory issues
- **Contains**: Duplicate packages that should be excluded by default
- **Purpose**: Validates that dependency exclusions work properly

## Dependency Scanning Modes

This project tests all three dependency scanning modes:

### 1. **Minimal Mode** (Default)
```bash
./target/release/codeprism-mcp test-projects/dependency-test-project
```
- **Excludes**: All dependency directories (`venv/`, `.tox/`, etc.)
- **Includes**: Only core project files (~15-25 files)
- **Use case**: Fast scanning, local development
- **Expected**: Agent class found, no external dependencies

### 2. **Smart Mode** 
```bash
./target/release/codeprism-mcp --smart-deps test-projects/dependency-test-project
```
- **Includes**: Dependency APIs (`__init__.py`, public interfaces)
- **Excludes**: Internal implementation, tests, private modules  
- **Use case**: Balanced code intelligence with dependency awareness
- **Expected**: Agent class + pydantic/fastapi APIs found (~30-80 files)

### 3. **Complete Mode**
```bash
./target/release/codeprism-mcp --include-deps test-projects/dependency-test-project
```
- **Includes**: All dependencies including internal implementation
- **Use case**: Complete code analysis, following all import chains
- **Expected**: Everything found (~50-200 files)

## Test Scenarios Covered

### ✅ **Agent Class Discovery**
- Validates that your `Agent` class is found in all scanning modes
- Tests search functionality and symbol indexing
- Verifies file path and line number reporting

### ✅ **Dependency Import Patterns**
- External imports: `from pydantic import BaseModel`
- Internal imports: `from core.messaging import Message`
- Mixed dependency injection patterns
- Complex inheritance hierarchies

### ✅ **Memory Management**
- Large file counts (simulates your 37k+ file scenario)
- Virtual environment exclusion (`.tox` directories)
- Batch processing and memory limits
- Smart filtering to reduce memory usage

### ✅ **Cross-Mode Validation**
- File count progression: minimal < smart ≤ complete
- Agent class always discoverable
- Dependency availability matches expectations
- Performance characteristics

## Running Tests

### Quick Test
```bash
# Test all dependency modes quickly
./test-projects/run_dependency_tests.sh
```

### Individual Mode Testing
```bash
# Test minimal mode
./target/release/codeprism-mcp test-projects/dependency-test-project

# Test smart mode  
./target/release/codeprism-mcp --smart-deps test-projects/dependency-test-project

# Test complete mode
./target/release/codeprism-mcp --include-deps test-projects/dependency-test-project
```

### Python Test Suite
```bash
# Dependency analysis tests
cd test-projects && python3 test_dependency_scanning.py

# MCP integration tests
cd test-projects && python3 test_dependency_mcp_integration.py
```

## Expected Results

### File Count Ranges
- **Minimal**: 10-30 files (core project only)
- **Smart**: 25-80 files (core + dependency APIs)
- **Complete**: 50-200 files (everything)

### Agent Class Discovery
- ✅ Found in **all modes**
- 📍 Location: `core/agent.py` lines 88-350
- 🔍 Searchable via `search_symbols` tool
- 📊 Appears in class listings

### Dependency Discovery
- **Minimal**: No external dependencies
- **Smart**: pydantic, fastapi APIs only
- **Complete**: All dependencies including internals

## Real-World Mapping

This test project directly maps to your original issue:

| **Your Issue** | **Test Project** | **Validation** |
|---|---|---|
| `Agent` class not found | `core/agent.py` Agent class | ✅ Always discoverable |
| 37k+ files in `.tox` | Mock `.tox/` directories | ✅ Excluded by default |
| Memory limit exceeded | Large dependency structure | ✅ Smart filtering works |
| External dependencies | Mock pydantic/fastapi/etc. | ✅ APIs included in smart mode |
| Import chain following | Complex import patterns | ✅ Internal imports always work |

## Integration with Main Test Suite

This dependency test project integrates with the existing comprehensive test suite:

```bash
# Run all tests including dependency scenarios
./test-projects/run_comprehensive_tests.sh

# Include in CI/CD pipelines
./test-projects/run_dependency_tests.sh
```

## Key Learnings Validated

1. **Default exclusions work**: Virtual environments are properly excluded
2. **Agent class always found**: Core project files are always indexed
3. **Smart filtering balance**: APIs included, internals excluded  
4. **Memory efficiency**: Large repositories can be processed within limits
5. **Flexible configuration**: Users can choose appropriate trade-offs

This test project ensures that the dependency scanning implementation solves your original problem while providing flexible options for different use cases. 