# [Issue 248] Design Document: Add `validation_scripts` Field to TestSpecification and TestCase

## Problem Statement

The current test specification and test case structures in `mandrel-mcp-th` do not support custom script-based validation. To enable advanced, multi-language script validation (JavaScript, Python, Lua), we must add a `validation_scripts` field to both `TestSpecification` and `TestCase` structs, and update YAML parsing and validation logic accordingly.

## Requirements
- Add a `validation_scripts` field to `TestSpecification` and `TestCase`.
- Support parsing of YAML files with and without the new field.
- Ensure backward compatibility for existing specs.
- Validate that scripts are correctly referenced and loaded.
- Provide unit tests for parsing, error cases, and edge conditions.
- Update documentation to reflect the new field.

## Proposed Solution

### Struct/API Changes
- Update the Rust structs in `spec/mod.rs`:
  - `TestSpecification`:
    - Add: `pub validation_scripts: Option<Vec<ValidationScript>>`
  - `TestCase`:
    - Add: `pub validation_scripts: Option<Vec<String>>` (references by name)
- Define a new `ValidationScript` struct:
  ```rust
  #[derive(Debug, Clone, Deserialize, Serialize)]
  pub struct ValidationScript {
      pub name: String,
      pub language: String, // "lua", "python", "javascript"
      pub execution_phase: Option<String>, // "before", "after"
      pub required: Option<bool>,
      pub source: String,
  }
  ```
- Update YAML parsing logic to support the new fields, using `serde` with `#[serde(default)]` for backward compatibility.

### YAML Example
```yaml
validation_scripts:
  - name: "math_precision_validator"
    language: "lua"
    execution_phase: "after"
    required: true
    source: |
      local request = context.request
      local response = context.response
      -- ...

tools:
  - name: "add"
    tests:
      - name: "add_integers"
        input: {"a": 5, "b": 3}
        expected:
          fields:
            - path: "$[0].text"
              pattern: "8"
        validation_scripts: ["math_precision_validator"]
```

### Parsing and Validation
- Use `Option` and `#[serde(default)]` to allow YAML files without `validation_scripts`.
- Validate that all script references in test cases exist in the top-level `validation_scripts`.
- Provide clear error messages for missing or malformed scripts.

## Implementation Plan (TDD)
1. **RED:** Write failing unit tests for YAML parsing with and without `validation_scripts`.
2. **GREEN:** Implement struct changes and parsing logic.
3. **REFACTOR:** Clean up code, improve error handling, and add documentation.
4. Add tests for error cases (missing script, invalid YAML, etc.).
5. Update documentation and examples.

## Acceptance Criteria
- [ ] YAML with and without `validation_scripts` parses correctly.
- [ ] Unit tests cover all parsing and error scenarios.
- [ ] Backward compatibility is maintained.
- [ ] Documentation is updated for the new field.
- [ ] All code follows project standards and passes CI checks.

## Integration Points
- `spec/mod.rs` for struct and parsing changes.
- YAML test specifications in `test-specs/` for real-world examples.
- Documentation in `docs/test-harness/` and code comments.

## Alternatives Considered
- Embedding scripts directly in test cases (rejected for DRY and reusability).
- Using only script file paths (rejected for portability; inline source preferred).

## Success Criteria
- All acceptance criteria above are met.
- No regressions in existing test parsing.
- Scripts can be referenced and loaded in test execution pipeline (future phases). 