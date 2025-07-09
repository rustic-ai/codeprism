# Issue #242: CLI moth validate Command Implementation

## Problem Statement

The `moth validate` command is completely unimplemented (placeholder) despite having a comprehensive enterprise-grade validation system already built in the backend. Currently it only prints "Configuration validation not yet implemented" and returns success.

## Proposed Solution

Wire up the existing `ValidationEngine` and `McpValidationEngine` components to the CLI `handle_validate_command` function to provide comprehensive configuration validation.

## Current State Analysis

### ✅ **What's Already Implemented:**

#### **1. ValidationEngine (in cli/mod.rs)**
- Input file validation (existence, permissions, format)
- Template validation (HTML templates, variable checking)
- Configuration validation (report configs, structure)

#### **2. McpValidationEngine (in validation/engine.rs)**
- MCP protocol compliance validation
- JSONPath expression validation
- JSON schema validation
- Custom business rule validation

#### **3. Specialized Validators**
- **JsonPathEvaluator**: Validate JSONPath expressions in test cases
- **SchemaValidator**: JSON schema compliance checking
- **ProtocolValidator**: MCP protocol specification compliance
- **SecurityValidator**: Check for sensitive data patterns
- **BusinessRuleValidator**: Custom validation rules

### ❌ **What's Missing:**
- CLI argument parsing for validation options
- Integration between CLI and validation engines
- Validation result formatting and output
- Error reporting and suggestions

## API Design

### **Enhanced ValidateArgs Structure**
```rust
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Configuration file to validate
    #[arg()]
    pub config: PathBuf,

    /// Enable strict validation mode (fail on warnings)
    #[arg(long)]
    pub strict: bool,

    /// Output directory for validation reports
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Report formats to generate for validation results
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,

    /// Check JSONPath expressions in test cases
    #[arg(long)]
    pub check_jsonpath: bool,

    /// Validate JSON schema compliance
    #[arg(long)]
    pub check_schema: bool,

    /// Validate MCP protocol compliance
    #[arg(long)]
    pub check_protocol: bool,

    /// Enable detailed validation diagnostics
    #[arg(long)]
    pub detailed: bool,

    /// Only validate, don't suggest fixes
    #[arg(long)]
    pub no_suggestions: bool,
}
```

### **Core Validation Function**
```rust
impl CliApp {
    async fn handle_validate_command(&self, args: &ValidateArgs) -> Result<i32> {
        // 1. Load and parse configuration file
        let config_content = std::fs::read_to_string(&args.config)?;
        let spec = SpecificationLoader::load_from_content(&config_content)?;
        
        // 2. Create validation engines
        let cli_validator = ValidationEngine::new()?;
        let mcp_validator = McpValidationEngine::new()?;
        
        // 3. Perform comprehensive validation
        let mut validation_results = Vec::new();
        
        // File and structure validation
        let file_result = cli_validator.validate_input_file(&args.config)?;
        validation_results.push(("File Structure", file_result));
        
        // MCP protocol validation if enabled
        if args.check_protocol {
            let protocol_result = self.validate_mcp_protocol(&spec, &mcp_validator).await?;
            validation_results.push(("MCP Protocol", protocol_result));
        }
        
        // JSONPath validation if enabled
        if args.check_jsonpath {
            let jsonpath_result = self.validate_jsonpath_expressions(&spec, &mcp_validator)?;
            validation_results.push(("JSONPath Expressions", jsonpath_result));
        }
        
        // Schema validation if enabled  
        if args.check_schema {
            let schema_result = self.validate_schemas(&spec, &mcp_validator)?;
            validation_results.push(("JSON Schemas", schema_result));
        }
        
        // 4. Generate validation reports
        if let Some(output_dir) = &args.output {
            self.generate_validation_reports(&validation_results, output_dir, &args.formats).await?;
        }
        
        // 5. Display summary and determine exit code
        let (is_valid, exit_code) = self.display_validation_summary(&validation_results, args.strict);
        
        Ok(exit_code)
    }
}
```

## Implementation Plan

### **Phase 1: Enhanced CLI Arguments**
1. Extend `ValidateArgs` struct with advanced validation options
2. Update `clap` argument definitions for new options
3. Add validation for argument combinations and defaults

### **Phase 2: Core Validation Logic**
1. Implement `handle_validate_command` with comprehensive validation
2. Wire up existing `ValidationEngine` for file/structure validation
3. Wire up `McpValidationEngine` for protocol/schema/JSONPath validation
4. Add configuration parsing and validation orchestration

### **Phase 3: Validation Reporting**
1. Create validation-specific report formatting
2. Integrate with existing reporting system for output generation
3. Add detailed error messages and suggestions
4. Implement summary display with actionable feedback

### **Phase 4: Testing and Integration**
1. Add comprehensive unit tests for all validation scenarios
2. Add integration tests with real configuration files
3. Test error cases and edge conditions
4. Verify CLI argument parsing and validation

## Success Criteria

### **Functional Requirements**
- `moth validate config.yaml` performs comprehensive validation
- `moth validate config.yaml --strict` fails on warnings
- `moth validate config.yaml --output reports/ --formats json,html` generates reports
- `moth validate config.yaml --check-all` enables all validation types
- Proper exit codes (0 = valid, 1 = invalid, 2 = error)

### **Quality Requirements**
- All validation types covered (file, protocol, JSONPath, schema)
- Detailed error messages with location information
- Actionable suggestions for fixing validation issues
- Performance: <5 seconds for typical configurations
- Memory usage: <100MB for large configurations

### **Integration Requirements**
- Compatible with existing test specifications
- Works with all supported MCP protocol versions
- Integrates with existing reporting system
- Maintains backward compatibility with simple usage

## Breaking Changes

**None** - This is purely additive functionality. Existing `moth validate config.yaml` usage will work exactly as before, but now it will actually perform validation instead of printing a placeholder message.

## Alternative Approaches Considered

### **Option A: Minimal Implementation**
- Only wire up basic file validation
- **Rejected**: Doesn't leverage existing comprehensive validation infrastructure

### **Option B: Separate Validation Binary**
- Create separate `moth-validate` binary
- **Rejected**: Increases complexity, users expect integrated CLI

### **Option C: Configuration-driven Validation**
- Require separate validation configuration files
- **Rejected**: Adds complexity for users, should work with existing specs

## Testing Strategy

### **Unit Tests**
- Test each validation type independently
- Test CLI argument parsing and validation
- Test error handling and edge cases
- Test exit code generation

### **Integration Tests**  
- Test with real CodePrism MCP specifications
- Test with invalid configurations (various error types)
- Test output generation in different formats
- Test strict mode vs. permissive mode

### **Performance Tests**
- Validate large configuration files
- Test memory usage with complex specifications
- Measure validation speed for CI/CD usage

## Rollout Plan

### **Phase 1: Core Implementation**
- Wire up basic validation with existing engines
- Support simple `moth validate config.yaml` usage
- Add basic error reporting

### **Phase 2: Advanced Features**
- Add all CLI options (strict, output, check-* flags)
- Integrate with reporting system
- Add comprehensive error messages

### **Phase 3: Polish and Documentation**
- Add detailed help and examples
- Update documentation with validation guide
- Add CI/CD integration examples 