# Issue #228: Real YAML Test Case Extraction Design Document

## Problem Statement

The `TestSuiteRunner` currently uses a mock implementation of `extract_test_cases()` that returns a hardcoded list of test cases. This prevents us from:
- Testing the dependency resolution with real data from YAML specifications
- Executing tests based on their actual definitions
- Validating the `test_run_test_suite_with_dependencies` test case, which is currently ignored

This blocks further development and testing of the `TestSuiteRunner`.

## Current Implementation

```rust
// Current mock implementation in `crates/mandrel-mcp-th/src/runner/mod.rs`
fn extract_test_cases(&self, _specification: &TestSpecification) -> Result<Vec<TestCase>> {
    // Returns hardcoded "test1", "test2", "test3"
    Ok(vec![
        TestCase { name: "test1".to_string(), dependencies: vec![] },
        // ...
    ])
}
```

## Proposed Solution

Implement the `extract_test_cases` method to correctly parse the `tools`, `resources`, and `prompts` sections of a `TestSpecification` and create a unified list of `TestCase` structs.

### Core Logic

The implementation will iterate through all possible test sources within the `TestSpecification` and extract the test cases into a single `Vec<TestCase>`.

```rust
// In crates/mandrel-mcp-th/src/runner/mod.rs

/// Extract test cases from the loaded YAML specification
fn extract_test_cases(&self, specification: &TestSpecification) -> Result<Vec<TestCase>> {
    let mut test_cases = Vec::new();

    // 1. Extract from `tools` section
    if let Some(tools) = &specification.tools {
        for tool_spec in tools {
            for test in &tool_spec.tests {
                test_cases.push(TestCase {
                    name: test.name.clone(),
                    // Extract real dependencies from the spec
                    dependencies: test.dependencies.clone().unwrap_or_default(),
                });
            }
        }
    }

    // 2. Extract from `resources` section
    if let Some(resources) = &specification.resources {
        for resource_spec in resources {
            for test in &resource_spec.tests {
                test_cases.push(TestCase {
                    name: test.name.clone(),
                    dependencies: test.dependencies.clone().unwrap_or_default(),
                });
            }
        }
    }

    // 3. Extract from `prompts` section
    if let Some(prompts) = &specification.prompts {
        for prompt_spec in prompts {
            for test in &prompt_spec.tests {
                test_cases.push(TestCase {
                    name: test.name.clone(),
                    dependencies: test.dependencies.clone().unwrap_or_default(),
                });
            }
        }
    }
    
    // NOTE: This now returns a Vec<spec::TestCase>, which is what we need.
    // The local runner::TestCase will be removed.
    Ok(test_cases)
}
```

## Implementation Plan

1.  **Refactor `runner::mod.rs`**: Remove the temporary `TestCase` struct from `runner/mod.rs` and update all usages to point to `crate::spec::TestCase`.
2.  **Implement `extract_test_cases`**: Replace the mock implementation with the real YAML parsing logic described above.
3.  **Update `resolve_dependencies`**: Ensure the dependency resolver correctly consumes the list of `spec::TestCase`.
4.  **Un-ignore and Fix Tests**: Re-enable the `test_run_test_suite_with_dependencies` test and fix any issues that arise.
5.  **Add New Tests**: Create specific tests to validate extraction from `tools`, `resources`, and `prompts` sections.

## Testing Strategy

-   **Unit test** `extract_test_cases` with a YAML file containing tests in all three sections (`tools`, `resources`, `prompts`).
-   **Unit test** with a YAML file containing dependencies to ensure they are extracted correctly.
-   **Integration test** by running `test_run_test_suite_with_dependencies` and verifying the execution order is correct based on the dependencies in the YAML file.
-   **Edge case test**: Handle specifications where some or all of the sections are missing.

## Success Criteria

-   The `extract_test_cases` method correctly parses all test cases from a `TestSpecification`.
-   The `test_run_test_suite_with_dependencies` test passes.
-   The system correctly identifies and orders tests based on dependencies defined in the YAML file.
-   No more `unwrap()` panics related to missing test names.

## Dependencies
- This work is a prerequisite for correctly implementing `TestSuiteRunner` and unblocks other tests.

This implementation will replace the placeholder logic with a real, functioning test case extraction mechanism, which is a critical step toward a fully operational test harness. 