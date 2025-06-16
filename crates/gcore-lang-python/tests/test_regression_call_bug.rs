/// Regression test for function call name extraction bug
/// 
/// This test ensures that the Python parser never creates Call nodes with invalid names
/// like ")" which was a bug where complex function call expressions were not properly parsed.

use gcore_lang_python::*;
use std::path::PathBuf;

fn parse_python_code(source: &str) -> ParseResult {
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
fn test_no_invalid_call_names_simple() {
    let source = r#"
def test():
    func()
    obj.method()
    nested.attr.call()
"#;
    
    let result = parse_python_code(source);
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    // Ensure no invalid call names
    for node in &call_nodes {
        assert!(!node.name.is_empty(), "Call node has empty name");
        assert!(node.name != ")", "Call node has invalid name: ')'");
        assert!(node.name != "(", "Call node has invalid name: '('");
        assert!(!node.name.trim().is_empty(), "Call node has whitespace-only name");
        
        // Should contain at least one alphanumeric character or underscore
        assert!(node.name.chars().any(|c| c.is_alphanumeric() || c == '_'), 
                "Call node '{}' has no valid identifier characters", node.name);
    }
    
    println!("✓ Simple call test passed: {} valid call nodes", call_nodes.len());
}

#[test]
fn test_no_invalid_call_names_complex() {
    // This was derived from patterns in the real Agent class that caused the bug
    let source = r#"
class Agent:
    def process(self):
        # Complex method chaining
        result = self._origin_message.routing_slip.model_copy(deep=True)
        
        # Nested function calls
        value = step.get_updated_state(
            self._origin_message,
            agent_state,
            guild_state,
            routed,
            routable,
        )
        
        # Attribute access with method calls
        tag = self._agent.get_agent_tag()
        name = self._agent.get_qualified_class_name()
        
        # Complex expressions
        handlers = {**format_handlers, **raw_handlers}
        
        # Conditional expressions with calls
        routing_slip = (
            self._origin_message.routing_slip.model_copy(deep=True) 
            if self._origin_message.routing_slip else None
        )
        
        # Dictionary comprehensions and complex calls
        processed = [func(item) for item in items if validate(item)]
        
        return result
"#;
    
    let result = parse_python_code(source);
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Complex test found {} call nodes:", call_nodes.len());
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. '{}' (kind: {:?})", i + 1, node.name, node.kind);
    }
    
    // Ensure no invalid call names
    let invalid_nodes: Vec<_> = call_nodes.iter()
        .filter(|n| {
            n.name.is_empty() || 
            n.name == ")" || 
            n.name == "(" || 
            n.name.trim().is_empty() ||
            n.name.chars().all(|c| !c.is_alphanumeric() && c != '_')
        })
        .collect();
    
    if !invalid_nodes.is_empty() {
        for node in &invalid_nodes {
            println!("❌ Invalid call node found: '{}'", node.name);
        }
        panic!("Found {} invalid call nodes", invalid_nodes.len());
    }
    
    // Should have found multiple valid call nodes
    assert!(call_nodes.len() >= 5, "Should have found at least 5 function calls in complex code");
    
    println!("✓ Complex call test passed: {} valid call nodes", call_nodes.len());
}

#[test]
fn test_edge_case_expressions() {
    // Test edge cases that might cause parsing issues
    let source = r#"
def edge_cases():
    # Nested parentheses
    result = func(other(nested()))
    
    # Multiple chained calls
    value = obj.method().another().final()
    
    # Subscript with calls
    item = array[index()]()
    
    # Complex lambda expressions
    mapped = map(lambda x: process(x), items)
    
    # Generator expressions
    gen = (func(x) for x in items if validate(x))
    
    # Decorator patterns
    @decorator(arg)
    def decorated():
        pass
        
    # Class instantiation with complex args
    instance = MyClass(
        param1=func(),
        param2=other.method(),
    )
"#;
    
    let result = parse_python_code(source);
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Edge case test found {} call nodes:", call_nodes.len());
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. '{}' (kind: {:?})", i + 1, node.name, node.kind);
    }
    
    // Ensure no invalid call names
    for node in &call_nodes {
        assert!(!node.name.is_empty(), "Call node has empty name");
        assert!(node.name != ")", "Call node has invalid name: ')'");
        assert!(node.name != "(", "Call node has invalid name: '('");
        assert!(!node.name.trim().is_empty(), "Call node has whitespace-only name");
        
        // The name should be a reasonable identifier or have meaningful content
        assert!(node.name.chars().any(|c| c.is_alphanumeric() || c == '_'), 
                "Call node '{}' has no valid identifier characters", node.name);
        
        // Names should not be just punctuation
        assert!(!node.name.chars().all(|c| !c.is_alphanumeric() && c != '_' && c != '.'), 
                "Call node '{}' is just punctuation", node.name);
    }
    
    println!("✓ Edge case test passed: {} valid call nodes", call_nodes.len());
}

#[test]
fn test_malformed_code_handling() {
    // Test how the parser handles slightly malformed or incomplete code
    let source = r#"
def test():
    # These might be tricky to parse correctly
    incomplete_call(
    another_call()
    
    # Missing parentheses (but still valid Python)
    obj.attr
    
    # Complex nesting
    deeply().nested().call().chain()
"#;
    
    let result = parse_python_code(source);
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Malformed code test found {} call nodes:", call_nodes.len());
    for (i, node) in call_nodes.iter().enumerate() {
        println!("  {}. '{}' (kind: {:?})", i + 1, node.name, node.kind);
    }
    
    // Even with malformed code, we should not create invalid call names
    for node in &call_nodes {
        assert!(!node.name.is_empty(), "Call node has empty name");
        assert!(node.name != ")", "Call node has invalid name: ')'");
        assert!(node.name != "(", "Call node has invalid name: '('");
        
        // Should have meaningful content
        assert!(node.name.chars().any(|c| c.is_alphanumeric() || c == '_'), 
                "Call node '{}' has no valid identifier characters", node.name);
    }
    
    println!("✓ Malformed code test passed: {} valid call nodes", call_nodes.len());
} 