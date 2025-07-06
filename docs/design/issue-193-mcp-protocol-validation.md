# Issue #193: MCP Protocol Validation Engine Design

## Problem Statement

The Mandrel MCP Test Harness needs a comprehensive validation engine to verify that MCP server responses conform to:
- Expected JSON schemas
- JSONPath expression constraints
- MCP protocol-specific field requirements
- Custom validation rules defined in test specifications

Currently, our existing `ValidationEngine` only handles configuration file validation. We need a dedicated engine for validating actual MCP protocol responses during test execution.

## Analysis of Requirements

### Core Validation Capabilities Required

1. **JSONPath Field Validation**
   - Evaluate JSONPath expressions against response data
   - Support field existence, type, value, and pattern validation
   - Handle nested object and array validation
   - Provide detailed path-specific error reporting

2. **JSON Schema Validation**
   - Validate entire response structures against JSON schemas
   - Support MCP-specific schema definitions
   - Handle schema composition and references
   - Generate comprehensive schema violation reports

3. **MCP Protocol Validation**
   - Validate JSON-RPC 2.0 message structure
   - Verify MCP-specific method responses
   - Check required vs optional fields per MCP specification
   - Validate capability announcements and tool/resource definitions

4. **Custom Validation Rules**
   - Support user-defined validation logic
   - Enable test-specific validation scenarios
   - Provide extension points for domain-specific validation
   - Support conditional validation based on context

## Proposed Architecture

### Core Components

```rust
// Main validation engine
pub struct McpValidationEngine {
    schema_validator: SchemaValidator,
    jsonpath_evaluator: JsonPathEvaluator,
    protocol_validator: ProtocolValidator,
    custom_validators: Vec<Box<dyn CustomValidator>>,
}

// Validation result with detailed diagnostics
#[derive(Debug, Clone)]
pub struct McpValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub field_results: HashMap<String, FieldValidationResult>,
    pub schema_violations: Vec<SchemaViolation>,
    pub protocol_issues: Vec<ProtocolIssue>,
}

// Individual field validation
#[derive(Debug, Clone)]
pub struct FieldValidationResult {
    pub field_path: String,
    pub expected_type: Option<String>,
    pub actual_type: String,
    pub expected_value: Option<serde_json::Value>,
    pub actual_value: serde_json::Value,
    pub validation_rules: Vec<ValidationRule>,
    pub is_valid: bool,
    pub errors: Vec<String>,
}

// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationSpec {
    pub schema: Option<serde_json::Value>,
    pub jsonpath_rules: Vec<JsonPathRule>,
    pub protocol_requirements: ProtocolRequirements,
    pub custom_rules: Vec<CustomRule>,
    pub strict_mode: bool,
}
```

### JSONPath Validation Engine

```rust
pub struct JsonPathEvaluator {
    engine: jsonpath_lib::Selector,
}

#[derive(Debug, Clone)]
pub struct JsonPathRule {
    pub path: String,
    pub constraint: PathConstraint,
    pub description: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone)]
pub enum PathConstraint {
    Exists,
    NotExists,
    Equals(serde_json::Value),
    NotEquals(serde_json::Value),
    Contains(String),
    MatchesPattern(String),
    HasType(JsonType),
    InRange(f64, f64),
    ArrayLength(usize, Option<usize>),
    Custom(Box<dyn Fn(&serde_json::Value) -> bool>),
}

impl JsonPathEvaluator {
    pub fn evaluate_rule(
        &self,
        data: &serde_json::Value,
        rule: &JsonPathRule,
    ) -> FieldValidationResult;
    
    pub fn extract_values(
        &self,
        data: &serde_json::Value,
        path: &str,
    ) -> Result<Vec<serde_json::Value>, JsonPathError>;
}
```

### Schema Validation Engine

```rust
use jsonschema::{Draft, JSONSchema};

pub struct SchemaValidator {
    compiled_schemas: HashMap<String, JSONSchema>,
    draft_version: Draft,
}

impl SchemaValidator {
    pub fn new() -> Self;
    
    pub fn add_schema(
        &mut self,
        name: String,
        schema: serde_json::Value,
    ) -> Result<(), SchemaError>;
    
    pub fn validate_against_schema(
        &self,
        data: &serde_json::Value,
        schema_name: &str,
    ) -> Result<SchemaValidationResult, SchemaError>;
    
    pub fn validate_with_inline_schema(
        &self,
        data: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> SchemaValidationResult;
}

#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub violations: Vec<SchemaViolation>,
}

#[derive(Debug, Clone)]
pub struct SchemaViolation {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
    pub violation_type: ViolationType,
}
```

### MCP Protocol Validator

```rust
pub struct ProtocolValidator {
    mcp_version: String,
    capabilities: McpCapabilities,
}

#[derive(Debug, Clone)]
pub struct ProtocolRequirements {
    pub method: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub expected_error_codes: Vec<i32>,
    pub capability_requirements: Vec<String>,
}

impl ProtocolValidator {
    pub fn validate_jsonrpc_structure(
        &self,
        response: &serde_json::Value,
    ) -> Vec<ProtocolIssue>;
    
    pub fn validate_mcp_method_response(
        &self,
        method: &str,
        response: &serde_json::Value,
        requirements: &ProtocolRequirements,
    ) -> Vec<ProtocolIssue>;
    
    pub fn validate_capability_announcement(
        &self,
        capabilities: &serde_json::Value,
    ) -> Vec<ProtocolIssue>;
    
    pub fn validate_tool_definition(
        &self,
        tool: &serde_json::Value,
    ) -> Vec<ProtocolIssue>;
}

#[derive(Debug, Clone)]
pub struct ProtocolIssue {
    pub category: ProtocolCategory,
    pub severity: ValidationSeverity,
    pub message: String,
    pub field_path: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ProtocolCategory {
    JsonRpcStructure,
    McpMethodCompliance,
    CapabilityMismatch,
    FieldMissing,
    FieldType,
    FieldValue,
}
```

## Implementation Plan

### Phase 1: Core Validation Structures (TDD)

**RED Phase**: Write failing tests for basic validation structures
```rust
#[test]
fn test_mcp_validation_engine_creation() {
    let engine = McpValidationEngine::new();
    assert!(engine.is_ok());
}

#[test]
fn test_validation_spec_deserialization() {
    let spec_json = r#"
    {
        "jsonpath_rules": [
            {
                "path": "$.result.content[0].text",
                "constraint": {"Exists": null},
                "description": "Response must contain text content"
            }
        ],
        "protocol_requirements": {
            "method": "tools/call",
            "required_fields": ["result"]
        }
    }"#;
    
    let spec: ValidationSpec = serde_json::from_str(spec_json).unwrap();
    assert_eq!(spec.jsonpath_rules.len(), 1);
}
```

**GREEN Phase**: Implement minimal structures to pass tests

**REFACTOR Phase**: Optimize and enhance structure design

### Phase 2: JSONPath Validation (TDD)

**RED Phase**: Write failing tests for JSONPath evaluation
```rust
#[test]
fn test_jsonpath_field_exists_validation() {
    let evaluator = JsonPathEvaluator::new();
    let data = json!({
        "result": {
            "content": [{"text": "Hello"}]
        }
    });
    
    let rule = JsonPathRule {
        path: "$.result.content[0].text".to_string(),
        constraint: PathConstraint::Exists,
        description: "Text field must exist".to_string(),
        severity: ValidationSeverity::Error,
    };
    
    let result = evaluator.evaluate_rule(&data, &rule);
    assert!(result.is_valid);
}

#[test]
fn test_jsonpath_value_equals_validation() {
    let evaluator = JsonPathEvaluator::new();
    let data = json!({"status": "success"});
    
    let rule = JsonPathRule {
        path: "$.status".to_string(),
        constraint: PathConstraint::Equals(json!("success")),
        description: "Status must be success".to_string(),
        severity: ValidationSeverity::Error,
    };
    
    let result = evaluator.evaluate_rule(&data, &rule);
    assert!(result.is_valid);
}
```

**GREEN Phase**: Implement JSONPath evaluation using `jsonpath-lib`
**REFACTOR Phase**: Optimize performance and error handling

### Phase 3: JSON Schema Validation (TDD)

**RED Phase**: Write failing tests for schema validation
```rust
#[test]
fn test_schema_validation_success() {
    let mut validator = SchemaValidator::new();
    
    let schema = json!({
        "type": "object",
        "properties": {
            "result": {
                "type": "object",
                "properties": {
                    "content": {
                        "type": "array",
                        "items": {"type": "object"}
                    }
                },
                "required": ["content"]
            }
        },
        "required": ["result"]
    });
    
    validator.add_schema("tool_response".to_string(), schema).unwrap();
    
    let data = json!({
        "result": {
            "content": [{"text": "Hello"}]
        }
    });
    
    let result = validator.validate_against_schema(&data, "tool_response").unwrap();
    assert!(result.is_valid);
}

#[test]
fn test_schema_validation_failure() {
    let mut validator = SchemaValidator::new();
    let schema = json!({
        "type": "object",
        "properties": {"required_field": {"type": "string"}},
        "required": ["required_field"]
    });
    
    validator.add_schema("test_schema".to_string(), schema).unwrap();
    
    let invalid_data = json!({"wrong_field": "value"});
    let result = validator.validate_against_schema(&invalid_data, "test_schema").unwrap();
    
    assert!(!result.is_valid);
    assert!(!result.violations.is_empty());
}
```

**GREEN Phase**: Implement schema validation using `jsonschema` crate
**REFACTOR Phase**: Add schema caching and performance optimization

### Phase 4: MCP Protocol Validation (TDD)

**RED Phase**: Write failing tests for MCP protocol compliance
```rust
#[test]
fn test_jsonrpc_structure_validation() {
    let validator = ProtocolValidator::new("1.0".to_string());
    
    let valid_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"status": "success"}
    });
    
    let issues = validator.validate_jsonrpc_structure(&valid_response);
    assert!(issues.is_empty());
    
    let invalid_response = json!({
        "id": 1,
        "result": {"status": "success"}
        // Missing "jsonrpc" field
    });
    
    let issues = validator.validate_jsonrpc_structure(&invalid_response);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|issue| issue.category == ProtocolCategory::JsonRpcStructure));
}

#[test]
fn test_mcp_tool_call_validation() {
    let validator = ProtocolValidator::new("1.0".to_string());
    
    let requirements = ProtocolRequirements {
        method: "tools/call".to_string(),
        required_fields: vec!["result".to_string()],
        optional_fields: vec!["meta".to_string()],
        expected_error_codes: vec![],
        capability_requirements: vec!["tools".to_string()],
    };
    
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{"type": "text", "text": "Analysis complete"}],
            "isError": false
        }
    });
    
    let issues = validator.validate_mcp_method_response("tools/call", &response, &requirements);
    assert!(issues.is_empty());
}
```

**GREEN Phase**: Implement MCP protocol-specific validation rules
**REFACTOR Phase**: Add comprehensive MCP specification compliance

### Phase 5: Integration and Testing (TDD)

**RED Phase**: Write integration tests with CLI and execution framework
```rust
#[tokio::test]
async fn test_validation_engine_integration() {
    let engine = McpValidationEngine::new();
    
    let spec = ValidationSpec {
        schema: Some(json!({
            "type": "object",
            "required": ["result"]
        })),
        jsonpath_rules: vec![
            JsonPathRule {
                path: "$.result.content".to_string(),
                constraint: PathConstraint::Exists,
                description: "Content must exist".to_string(),
                severity: ValidationSeverity::Error,
            }
        ],
        protocol_requirements: ProtocolRequirements {
            method: "tools/call".to_string(),
            required_fields: vec!["result".to_string()],
            optional_fields: vec![],
            expected_error_codes: vec![],
            capability_requirements: vec![],
        },
        custom_rules: vec![],
        strict_mode: true,
    };
    
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{"type": "text", "text": "Hello"}],
            "isError": false
        }
    });
    
    let result = engine.validate_response(&response, &spec).await.unwrap();
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}
```

**GREEN Phase**: Implement full integration with existing test execution framework
**REFACTOR Phase**: Optimize performance and add comprehensive error handling

## API Design

### Main Validation Interface

```rust
impl McpValidationEngine {
    pub fn new() -> Result<Self, ValidationEngineError>;
    
    pub async fn validate_response(
        &self,
        response: &serde_json::Value,
        spec: &ValidationSpec,
    ) -> Result<McpValidationResult, ValidationEngineError>;
    
    pub fn add_custom_validator(
        &mut self,
        validator: Box<dyn CustomValidator>,
    ) -> Result<(), ValidationEngineError>;
    
    pub fn load_validation_spec_from_file(
        &self,
        path: &Path,
    ) -> Result<ValidationSpec, ValidationEngineError>;
    
    pub fn precompile_schemas(
        &mut self,
        schemas: HashMap<String, serde_json::Value>,
    ) -> Result<(), ValidationEngineError>;
}
```

### Custom Validation Extension

```rust
pub trait CustomValidator: Send + Sync {
    fn name(&self) -> &str;
    fn validate(
        &self,
        data: &serde_json::Value,
        context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>>;
}

pub struct ValidationContext {
    pub method: String,
    pub request_id: Option<serde_json::Value>,
    pub server_capabilities: Option<serde_json::Value>,
    pub test_metadata: HashMap<String, String>,
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationEngineError {
    #[error("JSONPath evaluation failed: {0}")]
    JsonPathError(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaError(String),
    
    #[error("Protocol validation failed: {0}")]
    ProtocolError(String),
    
    #[error("Custom validator error: {0}")]
    CustomValidatorError(String),
    
    #[error("Validation specification error: {0}")]
    SpecificationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}
```

## Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
# JSON Path evaluation
jsonpath-lib = "0.3"

# JSON Schema validation  
jsonschema = "0.18"

# Regular expressions for pattern matching
regex = { workspace = true }

# Async support (already included)
tokio = { workspace = true }
futures = { workspace = true }

# Error handling (already included)
thiserror = { workspace = true }
anyhow = { workspace = true }

# Serialization (already included)
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
```

## Performance Considerations

1. **Schema Compilation Caching** - Pre-compile and cache JSON schemas for reuse
2. **JSONPath Expression Caching** - Cache compiled JSONPath selectors
3. **Parallel Validation** - Run independent validation rules in parallel
4. **Memory-Efficient Processing** - Stream large responses for validation
5. **Incremental Validation** - Support partial validation for large datasets

## Security Considerations

1. **Input Sanitization** - Validate all JSONPath expressions and schemas
2. **Resource Limits** - Prevent DoS through complex validation expressions
3. **Safe Evaluation** - Sandbox custom validation logic execution
4. **Schema Injection Prevention** - Validate schema definitions before compilation

## Success Criteria

### Functional Requirements
- [ ] Support JSONPath expression evaluation with all major constraint types
- [ ] Implement comprehensive JSON Schema validation with detailed error reporting
- [ ] Validate MCP protocol compliance including JSON-RPC 2.0 structure
- [ ] Support custom validation rules with extension points
- [ ] Generate detailed, actionable validation reports
- [ ] Handle complex nested structures and arrays efficiently

### Quality Requirements
- [ ] 95%+ test coverage with comprehensive unit and integration tests
- [ ] Sub-100ms validation time for typical MCP responses
- [ ] Memory usage <10MB for large response validation
- [ ] Support for concurrent validation of multiple responses
- [ ] Comprehensive error handling with detailed diagnostics

### Integration Requirements
- [ ] Seamless integration with existing test execution framework
- [ ] CLI integration for standalone validation operations
- [ ] Support for test specification file formats (JSON, YAML)
- [ ] Compatibility with existing reporting system design
- [ ] Extension points for custom domain-specific validation

## References

- [MCP Specification](mdc:specification/2025-06-18/) - Protocol requirements
- [JSONPath Specification](https://goessner.net/articles/JsonPath/) - Expression syntax
- [JSON Schema Specification](https://json-schema.org/) - Schema validation
- [Issue #194: Reporting System](mdc:docs/design/issue-194-mandrel-reporting-system.md) - Integration requirements
- [Current ValidationEngine](mdc:crates/mandrel-mcp-th/src/cli/mod.rs) - Configuration validation patterns 