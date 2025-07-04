# MCP Protocol Validation Engine Design Document

## Problem Statement
Create a comprehensive validation engine that can validate MCP responses against expected schemas, field values, and JSONPath expressions to ensure test correctness and provide detailed validation feedback.

## Proposed Solution

### Architecture Overview
The validation engine will provide comprehensive validation capabilities for MCP responses including JSONPath expression evaluation, JSON schema validation, field-level validation, and detailed error reporting.

### Core Components

#### ValidationEngine
```rust
pub struct ValidationEngine {
    schema_validator: Option<jsonschema::Validator>,
    jsonpath_cache: HashMap<String, JsonPath>,
    validation_config: ValidationConfig,
}

impl ValidationEngine {
    pub fn new(config: ValidationConfig) -> Self;
    pub async fn validate_response(&self, response: &serde_json::Value, expected: &ExpectedOutput) -> ValidationResult;
    pub fn validate_field(&self, value: &serde_json::Value, validation: &FieldValidation) -> FieldValidationResult;
    pub fn validate_schema(&self, value: &serde_json::Value, schema: &serde_json::Value) -> SchemaValidationResult;
}
```

#### ValidationConfig
```rust
pub struct ValidationConfig {
    pub strict_mode: bool,        // Fail on any validation error
    pub fail_fast: bool,          // Stop on first error
    pub max_errors: usize,        // Maximum errors to collect
    pub enable_caching: bool,     // Cache JSONPath expressions
    pub max_cache_size: usize,    // Maximum cache entries
}
```

#### ValidationResult
```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub validation_errors: Vec<ValidationError>,
    pub field_results: Vec<FieldValidationResult>,
    pub schema_result: Option<SchemaValidationResult>,
    pub performance_metrics: ValidationMetrics,
}
```

#### FieldValidationResult
```rust
pub struct FieldValidationResult {
    pub field_path: String,
    pub validation_type: FieldValidationType,
    pub is_valid: bool,
    pub actual_value: Option<serde_json::Value>,
    pub expected_value: Option<serde_json::Value>,
    pub error_message: Option<String>,
}
```

### Validation Types

#### JSONPath Validation
- Support for JSONPath expressions in field validation
- Caching of compiled JSONPath expressions for performance
- Detailed error messages for path resolution failures

#### Schema Validation  
- JSON Schema validation using jsonschema crate
- MCP protocol-specific schema definitions
- Error response schema validation

#### Field-Level Validation
- Type validation (string, number, boolean, array, object)
- Value validation (exact match, pattern match, range)
- Presence validation (required fields, optional fields)
- Custom validation rules support

### Implementation Plan

#### Phase 1: Core Infrastructure
1. Create validation module structure
2. Define error types and result structures
3. Implement basic ValidationEngine constructor
4. Add comprehensive unit tests (TDD)

#### Phase 2: JSONPath Support
1. Integrate jsonpath_lib for expression evaluation
2. Implement JSONPath caching mechanism
3. Add field validation with JSONPath queries
4. Create extensive JSONPath test scenarios

#### Phase 3: Schema Validation
1. Integrate jsonschema crate for schema validation
2. Define MCP protocol schemas
3. Implement response schema validation
4. Add schema validation error handling

#### Phase 4: Integration
1. Integrate with TestCase execution in executor
2. Update ExpectedOutput processing
3. Add validation to reporting system
4. Create end-to-end validation tests

## Integration Points

### With Existing Code
- **executor/mod.rs**: Integrate validation into test execution
- **spec/mod.rs**: Use existing FieldValidation and ExpectedOutput
- **error.rs**: Add validation-specific error types
- **reporting/mod.rs**: Include validation results in reports

### Dependencies
- **jsonpath_lib**: JSONPath expression evaluation
- **jsonschema**: JSON Schema validation  
- **serde_json**: JSON processing

## Performance Considerations
- JSONPath expression caching for repeated queries
- Schema validator caching for common schemas
- Configurable validation depth limits
- Memory-efficient error collection

## Testing Strategy
- Comprehensive unit tests for each validation type
- Integration tests with real MCP responses
- Performance tests for large responses
- Error scenario coverage
- Edge case testing for complex structures

## Success Criteria
- Complete JSONPath expression evaluation
- JSON Schema validation for MCP protocols
- Field-level validation for all data types
- Detailed validation failure diagnostics
- Performance: <50ms for typical responses
- 90%+ test coverage
- Seamless integration with existing framework 