# Issue #224: Enhanced Dependency Resolution with Circular Detection Design Document

## Problem Statement

The current `DependencyResolver` in the Mandrel MCP Test Harness has three critical issues that prevent reliable test suite execution:

1. **Circular Dependency Detection Failures**: The system fails to detect circular dependencies in complex scenarios, leading to infinite loops or incorrect execution order
2. **Complex Dependency Chain Resolution**: Execution order is incorrect for multi-level dependency chains due to flawed topological sorting
3. **Unwrap() Panics on Malformed Dependencies**: The code crashes instead of gracefully handling invalid dependency specifications (missing dependencies, empty names, self-references)

These issues block the completion of Issue #220 (Test Suite Runner) and prevent reliable test execution ordering.

## Current Implementation Analysis

### Failing Tests
- `test_circular_dependency_detection()` - Should detect and error on circular dependencies
- `test_run_test_suite_with_dependencies()` - Execution order incorrect for complex dependency chains  
- `test_empty_test_suite()` - Dependency resolution causing unwrap() panics

### Current Problems in Code
```rust
// Problem 1: Incomplete circular detection
pub fn detect_circular_dependencies(&self) -> Option<Vec<String>> {
    // Current DFS implementation has edge case bugs
}

// Problem 2: Flawed topological sort 
fn topological_sort(&self) -> Result<Vec<String>> {
    // Kahn's algorithm implementation has ordering issues
}

// Problem 3: Unwrap() usage without validation
let degree = in_degree.get_mut(dep).unwrap(); // PANIC on malformed input
```

## Proposed Solution

### High-Level Approach
Implement a robust three-phase dependency resolution system:

1. **Phase 1: Comprehensive Validation** - Validate all dependency specifications before processing
2. **Phase 2: Robust Circular Detection** - Use proper DFS with recursion stack tracking  
3. **Phase 3: Correct Topological Sorting** - Implement Kahn's algorithm with proper error handling

### Component Architecture

```rust
impl DependencyResolver {
    /// Main entry point - orchestrates all three phases
    pub fn resolve_dependencies(
        &mut self,
        test_cases: &HashMap<String, Vec<String>>
    ) -> Result<Vec<String>> {
        // Phase 1: Validate input
        self.validate_dependencies(test_cases)?;
        
        // Phase 2: Check for cycles
        if let Some(cycle) = self.detect_circular_dependencies() {
            return Err(Error::dependency(format!(
                "Circular dependency detected: {}", 
                cycle.join(" -> ")
            )));
        }
        
        // Phase 3: Topological sort
        self.topological_sort()
    }
    
    /// Phase 1: Comprehensive validation
    fn validate_dependencies(&self, test_cases: &HashMap<String, Vec<String>>) -> Result<()>;
    
    /// Phase 2: Enhanced circular detection  
    pub fn detect_circular_dependencies(&self) -> Option<Vec<String>>;
    
    /// Phase 3: Robust topological sort
    fn topological_sort(&self) -> Result<Vec<String>>;
}
```

## API Design

### Enhanced Validation
```rust
/// Validates all dependency specifications for correctness
///
/// # Validation Rules
/// - No empty dependency names
/// - No self-dependencies  
/// - All dependency references must exist in test suite
/// - No null or malformed dependency entries
///
/// # Errors
/// - `Error::Dependency` with specific validation failure details
fn validate_dependencies(&self, test_cases: &HashMap<String, Vec<String>>) -> Result<()> {
    for (test_name, dependencies) in test_cases {
        for dependency in dependencies {
            // Rule 1: No empty names
            if dependency.is_empty() {
                return Err(Error::dependency(format!(
                    "Empty dependency name for test '{}'", test_name
                )));
            }
            
            // Rule 2: No self-dependencies
            if dependency == test_name {
                return Err(Error::dependency(format!(
                    "Self-dependency detected for test '{}'", test_name
                )));
            }
            
            // Rule 3: Dependency must exist
            if !test_cases.contains_key(dependency) {
                return Err(Error::dependency(format!(
                    "Unknown dependency '{}' for test '{}'", dependency, test_name
                )));
            }
        }
    }
    Ok(())
}
```

### Robust Circular Detection
```rust
/// Detects circular dependencies using DFS with recursion stack tracking
///
/// # Algorithm
/// - Maintains visited set and recursion stack
/// - Tracks path during DFS for cycle reconstruction
/// - Returns complete cycle path when detected
///
/// # Returns
/// - `Some(cycle_path)` if circular dependency found
/// - `None` if no cycles detected
pub fn detect_circular_dependencies(&self) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    let mut path = Vec::new();

    for node in self.dependency_graph.keys() {
        if !visited.contains(node) {
            if let Some(cycle) = self.dfs_cycle_detection(
                node, 
                &mut visited, 
                &mut recursion_stack, 
                &mut path
            ) {
                return Some(cycle);
            }
        }
    }
    None
}

/// DFS helper for cycle detection with proper path tracking
fn dfs_cycle_detection(
    &self,
    node: &str,
    visited: &mut HashSet<String>,
    recursion_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    visited.insert(node.to_string());
    recursion_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(dependencies) = self.dependency_graph.get(node) {
        for dep in dependencies {
            if !visited.contains(dep) {
                if let Some(cycle) = self.dfs_cycle_detection(
                    dep, visited, recursion_stack, path
                ) {
                    return Some(cycle);
                }
            } else if recursion_stack.contains(dep) {
                // Found cycle - reconstruct cycle path
                let cycle_start = path.iter().position(|x| x == dep).unwrap();
                let mut cycle = path[cycle_start..].to_vec();
                cycle.push(dep.to_string()); // Complete the cycle
                return Some(cycle);
            }
        }
    }

    path.pop();
    recursion_stack.remove(node);
    None
}
```

### Improved Topological Sort
```rust
/// Performs topological sorting using Kahn's algorithm with proper error handling
///
/// # Algorithm
/// - Calculate in-degrees for all nodes
/// - Process nodes with zero in-degree first
/// - Reduce in-degrees as dependencies are satisfied
/// - Detect remaining cycles if not all nodes processed
///
/// # Returns
/// - `Ok(execution_order)` if successful
/// - `Err(Error::Dependency)` if cycles remain or other issues
fn topological_sort(&self) -> Result<Vec<String>> {
    let mut in_degree = HashMap::new();
    let mut result = Vec::new();
    let mut queue = VecDeque::new();

    // Initialize in-degree count for all nodes
    for node in self.dependency_graph.keys() {
        in_degree.insert(node.clone(), 0);
    }

    // Calculate in-degrees (no unwrap() - use get().unwrap_or())
    for dependencies in self.dependency_graph.values() {
        for dep in dependencies {
            if let Some(degree) = in_degree.get_mut(dep) {
                *degree += 1;
            } else {
                // This should not happen if validation passed
                return Err(Error::dependency(format!(
                    "Internal error: dependency '{}' not in graph", dep
                )));
            }
        }
    }

    // Add nodes with no dependencies to queue
    for (node, degree) in &in_degree {
        if *degree == 0 {
            queue.push_back(node.clone());
        }
    }

    // Process nodes in topological order
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());

        // Reduce in-degree for dependent nodes
        if let Some(dependencies) = self.dependency_graph.get(&node) {
            for dep in dependencies {
                if let Some(degree) = in_degree.get_mut(dep) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
    }

    // Verify all nodes were processed (no remaining cycles)
    if result.len() != self.dependency_graph.len() {
        return Err(Error::dependency(
            "Failed to resolve dependencies - circular dependencies remain after validation"
                .to_string(),
        ));
    }

    Ok(result)
}
```

## Implementation Plan

### Phase 1: Enhanced Validation (RED-GREEN-REFACTOR)
1. **RED**: Write failing tests for all validation scenarios
   - Empty dependency names
   - Self-dependencies  
   - Non-existent dependency references
   - Malformed dependency structures
2. **GREEN**: Implement `validate_dependencies()` method
3. **REFACTOR**: Optimize validation performance and error messages

### Phase 2: Robust Circular Detection (RED-GREEN-REFACTOR)  
1. **RED**: Write failing tests for circular dependency scenarios
   - Simple 2-node cycles (A→B→A)
   - Complex multi-node cycles (A→B→C→A)
   - Multiple independent cycles
   - Self-loops (A→A, caught in validation)
2. **GREEN**: Implement enhanced `detect_circular_dependencies()` and `dfs_cycle_detection()`
3. **REFACTOR**: Optimize cycle detection performance

### Phase 3: Improved Topological Sort (RED-GREEN-REFACTOR)
1. **RED**: Write failing tests for complex dependency chains  
   - Linear chains (A→B→C→D)
   - Tree structures (A→B,C; B→D; C→D)
   - Diamond dependencies (A→B,C; B,C→D)
2. **GREEN**: Implement robust `topological_sort()` method
3. **REFACTOR**: Optimize sorting performance and memory usage

### Phase 4: Integration Testing
1. Update `TestSuiteRunner::resolve_dependencies()` to use enhanced resolver
2. Verify all failing tests now pass
3. Add comprehensive integration tests

## Error Handling Strategy

### Error Types and Messages
```rust
// Validation errors
Error::dependency("Empty dependency name for test 'test1'")
Error::dependency("Self-dependency detected for test 'test1'") 
Error::dependency("Unknown dependency 'missing_test' for test 'test1'")

// Circular dependency errors  
Error::dependency("Circular dependency detected: test_a -> test_b -> test_c -> test_a")

// Topological sort errors
Error::dependency("Failed to resolve dependencies - circular dependencies remain")
```

### Error Recovery
- All errors are non-recoverable for dependency resolution
- Errors provide specific details for debugging
- Errors include test names and dependency chains for context

## Performance Requirements

### Scalability Targets
- **Small suites** (1-10 tests): <1ms resolution time
- **Medium suites** (10-100 tests): <10ms resolution time  
- **Large suites** (100+ tests): <100ms resolution time

### Algorithm Complexity
- **Validation**: O(D) where D = total dependency count
- **Circular Detection**: O(V + E) where V = tests, E = dependencies
- **Topological Sort**: O(V + E) where V = tests, E = dependencies
- **Overall**: O(V + E) - optimal for dependency resolution

## Testing Strategy

### Unit Test Coverage
- **Validation**: 100% coverage of all validation rules
- **Circular Detection**: 100% coverage of cycle detection scenarios
- **Topological Sort**: 100% coverage of ordering scenarios
- **Error Handling**: 100% coverage of all error paths

### Integration Test Coverage
- **TestSuiteRunner Integration**: Verify resolver works with real YAML specs
- **End-to-End**: Complete test suite execution with complex dependencies
- **Performance Tests**: Verify resolution times meet requirements

### Property-Based Testing
- Generate random dependency graphs and verify:
  - No false positive cycle detection
  - Topological order respects all dependencies
  - Consistent results across multiple runs

## Success Criteria

### Functional Requirements
- [ ] All three failing tests pass consistently
- [ ] Circular dependencies detected and reported with clear error messages
- [ ] Complex dependency chains resolve to correct execution order
- [ ] Malformed dependencies handled gracefully without panics
- [ ] Zero `unwrap()` calls in dependency resolution code

### Performance Requirements  
- [ ] Resolution time <100ms for test suites with 100+ tests
- [ ] Memory usage scales linearly with test count
- [ ] No memory leaks during resolution process

### Quality Requirements
- [ ] 100% test coverage for new validation and detection logic
- [ ] Comprehensive error messages with actionable debugging information
- [ ] Integration tests verify end-to-end functionality

## Alternatives Considered

### Alternative 1: Third-Party Dependency Resolution Library
**Pros**: Proven algorithms, comprehensive testing
**Cons**: Additional dependency, may not fit our specific error handling needs
**Decision**: Implement in-house for better control and integration

### Alternative 2: Simple Validation Only (No Circular Detection)
**Pros**: Simpler implementation, faster development
**Cons**: Doesn't solve the core circular dependency problem
**Decision**: Full implementation needed for robust test execution

### Alternative 3: Split Into Multiple Issues  
**Pros**: Smaller incremental changes, easier review
**Cons**: Interdependent changes, incomplete solutions until all parts done
**Decision**: Keep together as cohesive dependency resolution improvement

## Breaking Changes

**No breaking changes** - this is an internal enhancement to existing `DependencyResolver` API. All public interfaces remain unchanged.

## Migration Path

No migration required - existing test specifications will work unchanged with improved reliability and error reporting. 