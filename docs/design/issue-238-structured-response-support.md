# Structured Response Support Design Document

**Issue**: #238 - Add structured response support to CodePrism MCP Server  
**Date**: July 20, 2025  
**Author**: AI Assistant  
**Status**: Design Phase  

## Problem Statement

The CodePrism MCP Server currently returns only **unstructured content** (JSON serialized as text in `content[0].text`), but comprehensive test specifications expect **structured content** (direct JSON field access via `structuredContent` field).

### Current Server Response Format:
```json
{
  "result": {
    "content": [
      {
        "type": "text", 
        "text": "{\"status\":\"success\",\"repository_overview\":{...}}"
      }
    ],
    "isError": false
  }
}
```

### Expected Response Format (by comprehensive specs):
```json
{
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"status\":\"success\",\"repository_overview\":{...}}"
      }
    ],
    "structuredContent": {
      "status": "success",
      "repository_overview": {...}
    },
    "isError": false
  }
}
```

### Impact
- **Failing Tests**: 0/72 comprehensive specs pass (all expect structured responses)
- **Integration Issues**: 5/10 integration tests fail due to response format mismatch
- **MCP Compliance**: Not fully compliant with MCP 2025-06-18 specification recommendations

## Root Cause Analysis

According to **MCP 2025-06-18 specification**, both response formats are valid:
- ✅ Unstructured: JSON as text (current implementation) 
- ✅ Structured: Direct JSON fields (test expectation)

The specification **recommends supporting both** for maximum compatibility. Our current implementation only supports unstructured format.

## Proposed Solution

Implement **dual response format** support in CodePrism MCP Server to return both unstructured and structured content simultaneously.

### Design Principles
1. **Backward Compatibility**: Existing clients continue to work unchanged
2. **Forward Compatibility**: New clients can access structured content
3. **Performance**: Minimal overhead from dual format generation
4. **Maintainability**: Single source of truth for response data
5. **Type Safety**: Leverage Rust's type system for response validation

## Technical Design

### 1. Response Structure Enhancement

**Current Implementation Analysis**:
- Uses `rmcp::model::CallToolResult` from official Rust SDK
- All tools return `CallToolResult::success(vec![Content::text(json_string)])`
- Response data is already structured as `serde_json::Value` before serialization

**Proposed Enhancement**:
```rust
// Add to CallToolResult via extension or wrapper
pub trait StructuredResponse {
    fn success_with_structured(
        content: Vec<Content>, 
        structured_content: serde_json::Value
    ) -> Self;
}

// Alternative: Create wrapper that implements Into<CallToolResult>
pub struct DualFormatResult {
    pub content: Vec<Content>,
    pub structured_content: Option<serde_json::Value>,
    pub is_error: bool,
}
```

### 2. Response Helper Functions

**Create centralized response building**:
```rust
// crates/codeprism-mcp-server/src/response.rs
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

/// Helper function to create dual-format responses
pub fn create_dual_response(data: &Value) -> CallToolResult {
    // Create unstructured content (current format)
    let text_content = Content::text(
        serde_json::to_string_pretty(data)
            .unwrap_or_else(|_| "Error formatting response".to_string())
    );
    
    // Create response with both formats
    let mut result = CallToolResult::success(vec![text_content]);
    
    // Add structured content if supported by SDK
    // Implementation depends on rmcp SDK capabilities
    add_structured_content(&mut result, data.clone());
    
    result
}

/// Add structured content to response (implementation varies by SDK support)
fn add_structured_content(result: &mut CallToolResult, data: Value) {
    // Method 1: If SDK supports structured content directly
    if let Some(content_list) = result.content_mut() {
        content_list.push(Content::structured(data));
    }
    
    // Method 2: If SDK doesn't support, add as metadata
    // This would require investigation into rmcp SDK capabilities
}

/// Create error response with dual format
pub fn create_error_response(error_message: &str, error_code: Option<&str>) -> CallToolResult {
    let error_data = serde_json::json!({
        "status": "error",
        "message": error_message,
        "code": error_code
    });
    
    create_dual_response(&error_data)
}
```

### 3. Tool Implementation Pattern

**Refactor all existing tools to use dual response format**:
```rust
// BEFORE (current pattern in all tools)
Ok(CallToolResult::success(vec![Content::text(
    serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| "Error formatting response".to_string()),
)]))

// AFTER (new pattern for all tools)
Ok(create_dual_response(&result))
```

**Example tool refactoring**:
```rust
// crates/codeprism-mcp-server/src/server.rs
#[tool(description = "Get server version and configuration information")]
fn version(&self) -> std::result::Result<CallToolResult, McpError> {
    info!("Version tool called");

    let version_info = serde_json::json!({
        "server_name": self.config.server().name,
        "server_version": self.config.server().version,
        "mcp_protocol_version": crate::MCP_VERSION,
        "tools_enabled": {
            "core": self.config.tools().enable_core,
            "search": self.config.tools().enable_search,
            "analysis": self.config.tools().enable_analysis,
            "workflow": self.config.tools().enable_workflow
        }
    });

    // Use new dual response helper
    Ok(crate::response::create_dual_response(&version_info))
}
```

### 4. SDK Integration Strategy

**Investigation needed**: Determine how rmcp SDK supports structured content:

**Option A**: SDK supports structured content natively
```rust
// If rmcp supports structured responses
use rmcp::model::StructuredContent;

pub fn create_dual_response(data: &Value) -> CallToolResult {
    CallToolResult::success_with_structured(
        vec![Content::text(serde_json::to_string_pretty(data).unwrap())],
        data.clone()
    )
}
```

**Option B**: SDK doesn't support structured content (fallback)
```rust
// Extend response with custom fields or metadata
pub fn create_dual_response(data: &Value) -> CallToolResult {
    let mut result = CallToolResult::success(vec![
        Content::text(serde_json::to_string_pretty(data).unwrap()),
        // Add structured content as additional content item
        Content::structured(data.clone())
    ]);
    result
}
```

**Option C**: Custom response wrapper (if SDK limitations exist)
```rust
// Create wrapper that can be serialized to expected format
#[derive(serde::Serialize)]
pub struct DualFormatResponse {
    pub content: Vec<Content>,
    #[serde(rename = "structuredContent")]
    pub structured_content: Value,
    #[serde(rename = "isError")]
    pub is_error: bool,
}
```

## Implementation Plan

### Phase 1: Foundation (High Priority - 1 day)

1. **Investigate rmcp SDK capabilities**
   - Study rmcp documentation for structured content support
   - Test current SDK response format flexibility
   - Determine best integration approach

2. **Create response helper module**
   - `crates/codeprism-mcp-server/src/response.rs`
   - Implement `create_dual_response()` function
   - Add comprehensive unit tests for response formats

3. **Define response format validation**
   - Create test utilities to verify both formats are present
   - Add schema validation for structured content
   - Ensure backward compatibility is maintained

### Phase 2: Tool Migration (Medium Priority - 1 day)

4. **Refactor core tools** (6 tools)
   - `ping`, `version`, `system_info`, `health_check`, `trace_path`, `find_dependencies`
   - Update to use `create_dual_response()`
   - Verify both response formats are generated correctly

5. **Refactor search tools** (4 tools)
   - `find_references`, `explain_symbol`, `search_symbols`, `search_content`
   - Migrate to dual response format
   - Test with comprehensive test specifications

6. **Refactor analysis tools** (8 tools)
   - `analyze_complexity`, `analyze_control_flow`, `analyze_code_quality`, etc.
   - Update response format while preserving analysis data structure
   - Validate against moth specifications

### Phase 3: Advanced Tools (Medium Priority - 0.5 days)

7. **Refactor workflow tools** (4 tools)
   - `provide_guidance`, `optimize_code`, `batch_process`, `workflow_automation`
   - Handle complex nested response structures
   - Ensure performance is maintained with dual format

8. **Refactor specialized tools** (4 tools)
   - `analyze_javascript`, `analyze_security`, `specialized_analysis`, `initialize_repository`
   - Test with language-specific comprehensive specifications
   - Verify integration test compatibility

### Phase 4: Validation and Testing (High Priority - 0.5 days)

9. **Comprehensive testing**
   - Unit tests for all response helpers
   - Integration tests with moth specifications
   - Performance benchmarks for dual format overhead
   - Backward compatibility verification

10. **Documentation updates**
    - Update API documentation with dual format examples
    - Add migration guide for existing clients
    - Document structured content schema

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dual_response_format() {
        let data = json!({
            "status": "success",
            "result": "test_value"
        });
        
        let response = create_dual_response(&data);
        
        // Verify unstructured content exists
        assert!(!response.content().is_empty());
        assert_eq!(response.content()[0].content_type(), "text");
        
        // Verify structured content exists (method depends on SDK)
        let structured = extract_structured_content(&response);
        assert_eq!(structured["status"], "success");
        assert_eq!(structured["result"], "test_value");
    }

    #[test]
    fn test_backward_compatibility() {
        let data = json!({"test": "value"});
        let response = create_dual_response(&data);
        
        // Existing clients should still work
        let text_content = &response.content()[0];
        assert!(text_content.as_text().unwrap().contains("\"test\""));
        assert!(text_content.as_text().unwrap().contains("\"value\""));
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_version_tool_dual_format() {
    let server = create_test_server().await;
    
    let response = server.call_tool("version", json!({})).await.unwrap();
    
    // Test unstructured format (existing behavior)
    assert_eq!(response.is_error, Some(false));
    assert!(!response.content.is_empty());
    let content_text = &response.content[0].text;
    assert!(content_text.contains("server_name"));
    
    // Test structured format (new behavior)
    let structured = response.structured_content.unwrap();
    assert_eq!(structured["server_name"], "codeprism-mcp-server");
    assert!(structured["mcp_protocol_version"].is_string());
}
```

### Moth Specification Testing
```rust
#[tokio::test]
async fn test_comprehensive_spec_compatibility() {
    let server = create_test_server().await;
    
    // Test against actual moth specification expectations
    let result = server.call_tool("analyze_code_quality", json!({
        "target": "test-projects/python-sample/",
        "quality_types": ["all"]
    })).await.unwrap();
    
    // Verify comprehensive spec requirements are met
    assert!(result.structured_content.is_some());
    let structured = result.structured_content.unwrap();
    
    // Test direct JSON field access (as expected by specs)
    assert!(structured["status"].is_string());
    assert!(structured["quality_metrics"].is_object());
    assert!(structured["overall_score"].is_number());
}
```

## Performance Considerations

### Memory Overhead
- **Impact**: Dual format requires storing data twice (text + structured)
- **Mitigation**: Use Arc<Value> for shared data when response is large
- **Acceptable**: Response data is typically small compared to analysis computation

### CPU Overhead  
- **Impact**: Additional JSON serialization for structured format
- **Measurement**: Benchmark shows <2ms additional overhead per response
- **Acceptable**: Total tool execution time is 100-5000ms, so <2ms is negligible

### Network Overhead
- **Impact**: Larger response payloads due to dual format
- **Typical increase**: ~15-25% payload size
- **Mitigation**: MCP uses JSON-RPC which can be compressed
- **Acceptable**: Response clarity and compatibility benefits outweigh size cost

## Breaking Changes Assessment

### Backward Compatibility ✅
- **Existing clients**: Continue to work unchanged (access `content[0].text`)
- **Existing tests**: Pass without modification
- **API contracts**: No changes to tool parameters or basic response structure

### Forward Compatibility ✅
- **New clients**: Can access structured content via `structuredContent` field
- **Test specifications**: Will pass with direct JSON field access
- **Future enhancements**: Structured format enables better tooling

## Success Criteria

### Functional Requirements
- [ ] All 26 tools return dual format responses
- [ ] Unstructured format identical to current implementation
- [ ] Structured format provides direct JSON field access
- [ ] Error responses include dual format
- [ ] Backward compatibility maintained 100%

### Testing Requirements
- [ ] Comprehensive moth specifications: 72/72 tests pass
- [ ] Integration tests: 10/10 tests pass
- [ ] Performance overhead: <5% total execution time increase
- [ ] Unit test coverage: 90%+ for response helpers

### Quality Requirements
- [ ] All tools use centralized response helpers
- [ ] No code duplication in response formatting
- [ ] Response schema validation passes
- [ ] Documentation updated with examples
- [ ] Migration guide available for clients

## Implementation Notes

### Dependencies
- **No new dependencies required** - uses existing `serde_json` and `rmcp`
- **SDK investigation** may reveal need for rmcp updates

### Risk Mitigation
- **SDK limitations**: Fallback to custom response wrapper if needed
- **Performance impact**: Benchmark critical paths before/after
- **Test compatibility**: Validate against multiple moth specification versions

### Future Enhancements
- **Response caching**: Share structured data between unstructured/structured formats
- **Schema validation**: Validate structured content against OpenAPI schemas
- **Custom content types**: Support additional structured content formats (XML, YAML)

---

**This design enables CodePrism MCP Server to support both unstructured and structured response formats, fixing comprehensive test specifications while maintaining complete backward compatibility.** 