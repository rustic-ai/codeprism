# Issue #197: Create Test Specifications for Everything MCP Server Design Document

## Problem Statement

Create comprehensive YAML test specifications for the "everything" MCP server to validate the complete range of MCP capabilities and tool types. The everything server is the reference implementation that exercises all MCP protocol features, making it critical for comprehensive validation.

## Proposed Solution

### High-Level Approach

Develop a complete `everything-server.yaml` specification file that provides exhaustive coverage of:

1. **All 8 server tools** - Complete testing of echo, mathematical operations, sampling, imaging, environment access, annotations, and resource references
2. **Resource management system** - 100 test resources with pagination, subscriptions, and different data types  
3. **Prompt template system** - All 3 prompt types with argument handling and resource embedding
4. **Advanced MCP features** - Sampling, logging, progress notifications, annotations, and experimental capabilities
5. **Transport protocols** - stdio, SSE, and streamable HTTP support
6. **Performance testing** - Long-running operations, large resource sets, and concurrent access
7. **Error simulation** - Comprehensive error handling and edge case validation

### Target MCP Server Analysis

**Server Package:** `@modelcontextprotocol/server-everything`  
**Purpose:** Reference implementation exercising all MCP protocol features  
**Transport Support:** stdio (default), SSE, streamable HTTP  
**Unique Features:** Progress notifications, resource subscriptions, LLM sampling, annotation system

## Implementation Plan

### TDD Phase 1: Server Configuration and Basic Tools (2 hours)
- Create `everything-server.yaml` with server startup configuration  
- Implement test cases for basic tools: `echo`, `add`, `getTinyImage`, `printEnv`
- Add fundamental capability validation and transport testing

### TDD Phase 2: Advanced Tools and Features (3 hours)
- Implement test cases for advanced tools: `longRunningOperation`, `sampleLLM`, `annotatedMessage`, `getResourceReference`
- Add progress notification testing and LLM sampling validation
- Create annotation system tests with different message types

### TDD Phase 3: Resource System Testing (2 hours)  
- Create comprehensive resource tests for all 100 test resources
- Add pagination testing (10 items per page)
- Implement subscription and auto-update testing (5-second intervals)
- Add binary vs plaintext resource differentiation tests

### TDD Phase 4: Prompt Template System (2 hours)
- Implement tests for all 3 prompt types: `simple_prompt`, `complex_prompt`, `resource_prompt`
- Add argument validation for temperature, style, and resourceId parameters
- Create resource embedding tests within prompts

### TDD Phase 5: Advanced Protocol Features (2 hours)
- Add logging notification testing (15-second random messages)
- Implement experimental capability testing
- Create transport protocol switching tests (stdio ↔ SSE ↔ HTTP)
- Add concurrent operation testing

### TDD Phase 6: Performance and Error Testing (2 hours)
- Create performance tests for long-running operations and large resource sets
- Implement comprehensive error simulation and handling
- Add timeout and resource limit testing
- Create stress testing scenarios

## API Design

### Core Tool Testing Matrix

```yaml
tools:
  # Text Processing Tools
  - name: "echo"
    test_scenarios: ["basic_echo", "empty_message", "large_text", "unicode_support"]
    
  # Mathematical Tools  
  - name: "add"
    test_scenarios: ["integer_addition", "decimal_precision", "large_numbers", "edge_cases"]
    
  # Progress & Timing Tools
  - name: "longRunningOperation" 
    test_scenarios: ["default_operation", "custom_duration", "progress_notifications", "cancellation"]
    
  # LLM Integration Tools
  - name: "sampleLLM"
    test_scenarios: ["basic_sampling", "token_limits", "prompt_validation", "response_quality"]
    
  # Media Tools
  - name: "getTinyImage"
    test_scenarios: ["image_generation", "base64_validation", "format_verification"]
    
  # System Tools
  - name: "printEnv"
    test_scenarios: ["env_access", "security_validation", "data_format"]
    
  # Annotation Tools
  - name: "annotatedMessage"
    test_scenarios: ["error_annotations", "success_annotations", "debug_annotations", "image_inclusion"]
    
  # Resource Integration Tools
  - name: "getResourceReference" 
    test_scenarios: ["valid_resource_refs", "invalid_resource_ids", "resource_embedding"]
```

### Resource Testing Strategy

```yaml
resources:
  # Pagination Testing
  pagination_tests:
    - page_1_10_items
    - page_boundary_conditions  
    - large_page_requests
    
  # Resource Type Testing
  content_tests:
    - even_numbered_plaintext  # Resources 2, 4, 6, ..., 100
    - odd_numbered_binary      # Resources 1, 3, 5, ..., 99
    - uri_pattern_validation   # test://static/resource/{id}
    
  # Subscription Testing  
  subscription_tests:
    - subscribe_to_updates
    - auto_update_5_second_interval
    - unsubscribe_functionality
```

### Prompt Template Validation

```yaml
prompts:
  # Basic Prompt (No Arguments)
  - name: "simple_prompt"
    test_scenarios: ["no_args_required", "response_format", "single_message"]
    
  # Advanced Prompt (Required + Optional Arguments)
  - name: "complex_prompt" 
    required_args: ["temperature"]
    optional_args: ["style"]
    test_scenarios: ["required_only", "all_arguments", "invalid_temperature", "style_variations"]
    
  # Resource-Embedded Prompt
  - name: "resource_prompt"
    required_args: ["resourceId"]  
    test_scenarios: ["valid_resource_embed", "invalid_resource_id", "resource_content_inclusion"]
```

## Error Classification and Testing

### Tool Error Categories
- **Parameter Validation Errors**: Invalid types, missing required parameters, out-of-range values
- **Resource Access Errors**: Invalid resource IDs, pagination errors, subscription failures  
- **Sampling Errors**: LLM unavailability, token limit exceeded, invalid prompts
- **Progress Notification Errors**: Operation cancellation, timeout handling, notification failures
- **Transport Errors**: Connection failures, protocol switching, message corruption

### Expected Error Response Format
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "parameter": "resourceId", 
      "reason": "Must be between 1 and 100",
      "received": "150"
    }
  }
}
```

## Performance Requirements

### Response Time Targets
- **Basic Tools** (echo, add, getTinyImage, printEnv): < 100ms
- **Resource Access**: < 200ms for single resource, < 500ms for paginated list
- **Prompt Generation**: < 1000ms for simple prompts, < 3000ms for complex prompts
- **LLM Sampling**: < 30000ms (configurable timeout)
- **Long Running Operations**: Progress updates every 20% completion

### Throughput Requirements  
- **Concurrent Tool Calls**: 10 simultaneous operations
- **Resource Pagination**: Handle 1000+ resource requests efficiently
- **Subscription Updates**: Support 50+ concurrent subscriptions

### Memory Constraints
- **Peak Memory Usage**: < 256MB during normal operations
- **Resource Caching**: Efficient memory management for 100 resources
- **Base64 Image Storage**: Minimal memory footprint for getTinyImage

## Integration Testing Scenarios

### Transport Protocol Switching
1. **stdio → SSE Migration**: Test seamless protocol transition
2. **SSE → HTTP Transition**: Validate connection persistence  
3. **Multi-Transport Validation**: Ensure feature parity across protocols

### Capability Negotiation Testing
1. **Full Capability Handshake**: Validate all capabilities are properly advertised
2. **Selective Capability Testing**: Test partial capability subsets
3. **Experimental Feature Validation**: Ensure experimental features work correctly

### Concurrent Operation Testing
1. **Mixed Tool Execution**: Run multiple different tools simultaneously
2. **Resource Access Concurrency**: Concurrent resource reads with subscriptions
3. **Prompt + Tool Interaction**: Complex multi-capability scenarios

## Security Testing Framework

### Input Validation Testing
- **Parameter Injection**: Test for injection attacks through tool parameters
- **Resource ID Manipulation**: Validate resource access controls
- **Environment Variable Exposure**: Ensure printEnv doesn't leak sensitive data

### Resource Access Security
- **URI Pattern Validation**: Prevent path traversal through resource URIs
- **Subscription DoS Prevention**: Limit subscription counts and update frequency
- **Binary Resource Validation**: Ensure binary content is properly sandboxed

## Success Criteria

### Functional Completeness
- [ ] All 8 tools tested with comprehensive scenarios (32+ test cases)
- [ ] Complete resource system validation (100 resources + pagination + subscriptions)
- [ ] All 3 prompt types tested with argument combinations (12+ test cases)
- [ ] LLM sampling integration fully validated
- [ ] Progress notification system thoroughly tested
- [ ] Annotation system validated across all message types

### Quality Gates
- [ ] 95%+ test success rate across all scenarios
- [ ] All performance targets met consistently  
- [ ] Comprehensive error handling coverage
- [ ] Transport protocol feature parity verified
- [ ] Security validation for all user inputs

### Documentation Standards
- [ ] Complete YAML specification with inline documentation
- [ ] Error scenario documentation with expected responses
- [ ] Performance benchmark documentation  
- [ ] Integration examples for common usage patterns

## Alternative Approaches Considered

### Option 1: Minimal Tool Testing
**Rejected**: Would not exercise the full breadth of MCP capabilities that everything server is designed to test.

### Option 2: Transport-Specific Testing  
**Rejected**: Would require separate test files for each transport protocol, reducing maintainability.

### Option 3: Capability-Focused Organization
**Selected**: Organize tests by MCP capability (tools, resources, prompts) for clear coverage and maintainability.

## Implementation Deliverables

1. **everything-server.yaml** (800+ lines): Complete test specification
2. **Test Documentation**: Inline YAML comments explaining each test scenario  
3. **Error Reference**: Comprehensive error response documentation
4. **Performance Benchmarks**: Expected response times and throughput metrics
5. **Integration Examples**: Common usage patterns and client integration examples

## Risk Mitigation

### LLM Sampling Dependency Risk
**Mitigation**: Include mock mode testing and graceful degradation for LLM unavailability

### Resource Update Timing Risk  
**Mitigation**: Implement flexible timing windows for subscription update testing (4-6 second range)

### Progress Notification Reliability Risk
**Mitigation**: Test progress notification robustness with network interruption simulation

## References

- Everything MCP Server: https://www.npmjs.com/package/@modelcontextprotocol/server-everything
- MCP Specification: https://spec.modelcontextprotocol.io/
- MCP Transport Protocols: https://spec.modelcontextprotocol.io/specification/basic/transports/
- Progress Notifications: https://spec.modelcontextprotocol.io/specification/server/progress/

---

**This design provides comprehensive validation of the everything MCP server's capabilities while establishing patterns for testing other MCP servers with similar complexity.** 