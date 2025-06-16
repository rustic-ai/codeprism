use gcore_lang_python::*;
use std::path::PathBuf;

fn test_function_calls_with_parser(source: &str) -> ParseResult {
    let mut parser = PythonParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.py"),
        old_tree: None,
        content: source.to_string(),
    };
    
    parser.parse(&context).unwrap()
}

#[test]
fn test_function_call_parsing_simple() {
    let source = r#"
def test_function():
    func()
    method.call()
    obj.attr.nested()
"#;
    
    let result = test_function_calls_with_parser(source);
    
    // Find all call nodes
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Found {} call nodes:", call_nodes.len());
    for node in &call_nodes {
        println!("  Call: '{}' (kind: {:?})", node.name, node.kind);
    }
    
    // Verify no invalid call names
    let invalid_calls: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == ")" || n.name.is_empty())
        .collect();
    
    assert_eq!(invalid_calls.len(), 0, 
        "Found {} invalid call nodes with name ')'", invalid_calls.len());
    
    // Should have at least 3 valid calls
    assert!(call_nodes.len() >= 3, "Should have at least 3 function calls");
}

#[test]
fn test_complex_function_calls() {
    let source = r#"
class Agent:
    def __init__(self):
        self.id = generate_id()
        self.client = Client()
        
    def process(self):
        result = self.helper.method()
        data = transform(result)
        return validate(data)
"#;
    
    let result = test_function_calls_with_parser(source);
    
    // Find all call nodes
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Found {} call nodes in complex example:", call_nodes.len());
    for node in &call_nodes {
        println!("  Call: '{}' (kind: {:?})", node.name, node.kind);
    }
    
    // Check for invalid names - this should fail and show us the bug
    let invalid_calls: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == ")" || n.name.trim().is_empty())
        .collect();
    
    if !invalid_calls.is_empty() {
        println!("BUG REPRODUCED! Invalid call nodes found:");
        for invalid in &invalid_calls {
            println!("  Invalid call: '{}'", invalid.name);
        }
    }
    
    // For now, let's see what we get instead of asserting
    println!("Total calls: {}, Invalid calls: {}", call_nodes.len(), invalid_calls.len());
}

#[test]
fn test_agent_class_reproduction() {
    // Simplified version of the Agent class that's causing issues
    let source = r#"
class Agent:
    def __init__(self):
        self.spec = agent_spec
        self.id = agent_spec.id
        
    def get_spec(self):
        return self._agent_spec
        
    def _set_client(self, client):
        self._client = client
        return self._client
"#;
    
    let result = test_function_calls_with_parser(source);
    
    // Find all call nodes
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Agent class call analysis:");
    println!("Found {} call nodes:", call_nodes.len());
    
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. Call: '{}' (kind: {:?})", i+1, node.name, node.kind);
    }
    
    // Look specifically for invalid calls
    let invalid_calls: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == ")")
        .collect();
    
    if !invalid_calls.is_empty() {
        println!("FOUND THE BUG! {} invalid calls with name ')'", invalid_calls.len());
        for invalid in &invalid_calls {
            println!("  File: {}", invalid.file.display());
            println!("  Span: {:?}", invalid.span);
        }
    }
}

#[test]
fn test_problematic_call_patterns() {
    // Test patterns that might be causing the ")" bug
    let source = r#"
class TestClass:
    def test_method(self):
        # These might be problematic patterns:
        result = some_func(arg1, arg2)
        nested = outer(inner())
        chained = obj.method().another()
        complex = self.get_something(param).process()
        
        # Attribute access without calls
        value = self.property
        
        # Method calls with complex arguments
        processed = self.process(
            complex_arg(nested_call()),
            another_arg
        )
"#;
    
    let result = test_function_calls_with_parser(source);
    
    // Find all call nodes
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Problematic patterns call analysis:");
    println!("Found {} call nodes:", call_nodes.len());
    
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. Call: '{}' (kind: {:?})", i+1, node.name, node.kind);
        if node.name == ")" || node.name.trim().is_empty() {
            println!("      ⚠️  INVALID CALL DETECTED!");
        }
    }
    
    // Count invalid calls
    let invalid_calls: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == ")" || n.name.trim().is_empty())
        .collect();
    
    println!("Summary: {} valid, {} invalid calls", 
        call_nodes.len() - invalid_calls.len(), 
        invalid_calls.len());
}

#[test]
fn test_real_agent_patterns() {
    // Patterns from the actual Agent class that might cause issues
    let source = r#"
class Agent:
    def process_message(self):
        # Complex chained method calls
        routing_slip = (
            self._origin_message.routing_slip.model_copy(deep=True) 
            if self._origin_message.routing_slip else None
        )
        
        # Multiple attribute access patterns
        agent_tag = self._agent.get_agent_tag()
        class_name = self._agent.get_qualified_class_name()
        
        # Complex nested calls
        state_update = step.get_updated_state(
            self._origin_message,
            agent_state,
            guild_state,
            routed,
            routable,
        )
        
        # Method chaining with conditionals
        result = obj.method().another() if condition else default
        
        # Complex function calls in list comprehensions
        handlers = {**format_handlers, **raw_handlers}
        
        # Deeply nested attribute access
        value = self._origin_message.routing_slip.get_next_steps()
"#;
    
    let result = test_function_calls_with_parser(source);
    
    // Find all call nodes
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Real Agent patterns analysis:");
    println!("Found {} call nodes:", call_nodes.len());
    
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. Call: '{}' (kind: {:?})", i+1, node.name, node.kind);
        if node.name == ")" || node.name.trim().is_empty() {
            println!("      🚨 BUG FOUND: Invalid call name!");
        }
    }
    
    // Count invalid calls
    let invalid_calls: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == ")" || n.name.trim().is_empty())
        .collect();
    
    if invalid_calls.len() > 0 {
        println!("🎯 SUCCESS: Reproduced the bug with {} invalid calls!", invalid_calls.len());
    } else {
        println!("❌ Bug not reproduced in this test");
    }
} 