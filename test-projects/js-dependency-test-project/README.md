# JavaScript Dependency Test Project

This test project validates JavaScript/Node.js dependency scanning modes in the CodePrism MCP server, complementing the Python dependency tests. It demonstrates real-world React/Node.js patterns similar to your original Agent class discovery issue but for JavaScript ecosystems.

## Project Structure

```
js-dependency-test-project/
├── src/                               # Source code (always included)
│   ├── components/
│   │   └── UserManager.jsx           # Main React component (like Agent class)
│   ├── services/
│   │   └── UserService.js            # Service layer
│   ├── utils/
│   │   ├── ApiClient.js              # HTTP client
│   │   └── Logger.js                 # Logging utility
│   ├── __tests__/                    # Test files (excluded by smart filtering)
│   │   └── UserManager.test.jsx
│   └── index.js                      # Main entry point
├── node_modules/                      # Node.js dependencies
│   ├── react/
│   │   ├── index.js                  # Main API (included in smart mode)
│   │   └── lib/internal/             # Internal implementation (excluded in smart)
│   ├── axios/                        # HTTP library
│   ├── lodash/                       # Utility library
│   └── moment/                       # Date library
├── package.json                      # Project configuration (always included)
└── README.md                         # This file
```

## Key Features

### 1. **UserManager Component** (`src/components/UserManager.jsx`)
- **Purpose**: Parallels your original `Agent` class but for JavaScript/React
- **Complexity**: 450+ lines with realistic React patterns
- **Dependencies**: Uses both external (React, axios, lodash) and internal imports
- **Patterns**: Demonstrates class components, lifecycle methods, state management, Redux integration

### 2. **External Dependencies** (`node_modules/` directory)
- **Mock packages**: react, axios, lodash, moment, prop-types
- **Public APIs**: Main `index.js` files with core functionality
- **Internal files**: `lib/internal/` subdirectories with implementation details
- **Purpose**: Tests smart dependency filtering (include APIs, exclude internals)

### 3. **Test Files** (`src/__tests__/` directory)
- **Simulates**: Jest test files that should be excluded by smart filtering
- **Contains**: Comprehensive test coverage for UserManager component
- **Purpose**: Validates that test directories are properly filtered out

## Dependency Scanning Modes

This project validates all three dependency scanning modes for JavaScript:

### 1. **Minimal Mode** (Default)
```bash
./target/release/codeprism-mcp test-projects/js-dependency-test-project
```
- **Excludes**: All `node_modules/`, test files, build artifacts
- **Includes**: Only core project files (~8-15 files)
- **Use case**: Fast development, local code intelligence
- **Expected**: UserManager component found, no external dependencies

### 2. **Smart Mode** 
```bash
./target/release/codeprism-mcp --smart-deps test-projects/js-dependency-test-project
```
- **Includes**: Dependency APIs (`index.js`, `package.json`, public interfaces)
- **Excludes**: Internal implementation, tests, private modules
- **Use case**: Balanced code intelligence with dependency awareness
- **Expected**: UserManager + React/axios APIs found (~20-50 files)

### 3. **Complete Mode**
```bash
./target/release/codeprism-mcp --include-deps test-projects/js-dependency-test-project
```
- **Includes**: All dependencies including internal implementation
- **Use case**: Complete code analysis, following all import chains
- **Expected**: Everything found (~40-100+ files)

## JavaScript-Specific Test Scenarios

### ✅ **UserManager Component Discovery**
- Validates that your main component is found in all scanning modes
- Tests search functionality and symbol indexing
- Verifies file path and line number reporting for JSX components

### ✅ **Node.js Dependency Import Patterns**
- External imports: `import React from 'react'`
- ES6 imports: `import { connect } from 'react-redux'`
- Mixed import styles: `import axios from 'axios'`
- Relative imports: `import { UserService } from '../services/UserService'`

### ✅ **Node.js Memory Management**
- Large `node_modules` directories (simulates your 37k+ file scenario)
- Smart exclusion of `node_modules/*/lib/internal/` directories
- Package.json and main entry point inclusion
- Test file exclusion (`__tests__/`, `*.test.js`, `*.spec.js`)

### ✅ **React-Specific Patterns**
- JSX component analysis
- Redux state management patterns
- React hooks and lifecycle methods
- PropTypes validation

## Expected Results

### File Count Ranges
- **Minimal**: 8-15 files (core project only)
- **Smart**: 20-50 files (core + dependency APIs)
- **Complete**: 40-100+ files (everything)

### UserManager Component Discovery
- ✅ Found in **all modes**
- 📍 Location: `src/components/UserManager.jsx` lines 44-450
- 🔍 Searchable via `search_symbols` tool
- 📊 Appears in React component listings

### Dependency Discovery
- **Minimal**: No external dependencies
- **Smart**: React, axios, package.json APIs only
- **Complete**: All dependencies including internals

## Language Mapping

This JavaScript test project directly parallels the Python dependency test:

| **Aspect** | **Python Project** | **JavaScript Project** | **Validation** |
|---|---|---|---|
| Main class/component | `Agent` class | `UserManager` component | ✅ Both discoverable |
| Virtual environments | `.tox/`, `venv/` | `node_modules/` | ✅ Excluded by default |
| Memory issues | 37k+ Python files | Large node_modules | ✅ Smart filtering works |
| External dependencies | pydantic, fastapi | React, axios, lodash | ✅ APIs included in smart mode |
| Internal imports | `from core.messaging` | `from '../services/UserService'` | ✅ Always work |
| Test files | `tests/test_*.py` | `__tests__/*.test.jsx` | ✅ Excluded by smart filtering |

## Integration with Test Suite

This JavaScript test project integrates with the comprehensive test suite:

```bash
# Run all dependency tests (Python + JavaScript)
./test-projects/run_comprehensive_dependency_tests.sh

# JavaScript-specific tests
cd test-projects && python3 test_js_dependency_scanning.py

# Cross-language validation
./target/release/codeprism-mcp --smart-deps /path/to/mixed/project
```

## Key Learnings Validated

1. **Default exclusions work**: `node_modules` properly excluded like Python's `.tox`
2. **UserManager always found**: Core JavaScript files always indexed  
3. **Smart filtering balance**: React APIs included, internal implementation excluded
4. **Memory efficiency**: Large node_modules handled within limits
5. **Cross-language support**: Both Python and JavaScript work seamlessly

## Real-World JavaScript Patterns

This test project validates dependency scanning for:

- **React Applications**: Component discovery, JSX parsing, hooks usage
- **Node.js Projects**: Module resolution, package.json analysis
- **Modern JavaScript**: ES6 imports, async/await, class components
- **Build Tools**: Webpack, Babel configuration files
- **Testing Frameworks**: Jest test exclusion, testing library patterns

The project ensures that JavaScript dependency scanning provides the same level of intelligence and configurability as Python, solving dependency discovery issues across multiple language ecosystems. 