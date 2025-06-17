use prism_lang_python::*;
use std::path::PathBuf;

#[test]
fn test_parse_simple_python_file() {
    let mut parser = PythonParser::new();
    let content = std::fs::read_to_string("tests/fixtures/simple.py").unwrap();
    
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("tests/fixtures/simple.py"),
        old_tree: None,
        content,
    };

    let result = parser.parse(&context).unwrap();
    
    // Check that we have nodes
    assert!(!result.nodes.is_empty());
    
    // Should have a module node
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Module)));
    
    // Should have function nodes
    let functions: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Function))
        .collect();
    assert!(functions.len() >= 2);
    
    // Check specific function names
    assert!(functions.iter().any(|n| n.name == "hello_world"));
    assert!(functions.iter().any(|n| n.name == "add_numbers"));
    
    // Should have variable nodes
    let variables: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Variable))
        .collect();
    assert!(!variables.is_empty());
    
    // Should have call nodes
    let calls: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    assert!(!calls.is_empty());
}

#[test]
fn test_parse_class_example() {
    let mut parser = PythonParser::new();
    let content = std::fs::read_to_string("tests/fixtures/class_example.py").unwrap();
    
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("tests/fixtures/class_example.py"),
        old_tree: None,
        content,
    };

    let result = parser.parse(&context).unwrap();
    
    // Should have a module node
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Module)));
    
    // Should have a class node
    let classes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Class))
        .collect();
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].name, "Calculator");
    
    // Should have method nodes
    let methods: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Method))
        .collect();
    assert!(methods.len() >= 4); // __init__, add, subtract, get_history, current_value, multiply_static
    
    // Check for specific methods
    assert!(methods.iter().any(|n| n.name == "__init__"));
    assert!(methods.iter().any(|n| n.name == "add"));
    assert!(methods.iter().any(|n| n.name == "subtract"));
    
    // Should have import nodes
    let imports: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Import))
        .collect();
    assert!(!imports.is_empty());
    
    // Should have some edges
    assert!(!result.edges.is_empty());
}

#[test]
fn test_language_detection() {
    use std::path::Path;
    
    assert_eq!(PythonParser::detect_language(Path::new("test.py")), Language::Python);
    assert_eq!(PythonParser::detect_language(Path::new("test.pyw")), Language::Python);
    assert_eq!(PythonParser::detect_language(Path::new("script")), Language::Python); // defaults to Python
}

#[test]
fn test_node_spans() {
    let mut parser = PythonParser::new();
    let content = "def test_function():\n    return 42";
    
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.py"),
        old_tree: None,
        content: content.to_string(),
    };

    let result = parser.parse(&context).unwrap();
    
    // Find the function node
    let func_node = result.nodes.iter()
        .find(|n| matches!(n.kind, NodeKind::Function) && n.name == "test_function")
        .unwrap();
    
    // Check that spans are reasonable
    assert!(func_node.span.start_byte < func_node.span.end_byte);
    assert!(func_node.span.start_line <= func_node.span.end_line);
    assert!(func_node.span.start_column > 0);
}

#[test]
fn test_edges_creation() {
    let mut parser = PythonParser::new();
    let content = "def caller():\n    callee()\n\ndef callee():\n    pass";
    
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.py"),
        old_tree: None,
        content: content.to_string(),
    };

    let result = parser.parse(&context).unwrap();
    
    // Should have some edges
    assert!(!result.edges.is_empty());
    
    // Check for CALLS edges
    let call_edges: Vec<_> = result.edges.iter()
        .filter(|e| matches!(e.kind, EdgeKind::Calls))
        .collect();
    assert!(!call_edges.is_empty());
}

#[test]
fn test_incremental_parsing() {
    let mut parser = PythonParser::new();
    
    // First parse
    let content1 = "def original_function():\n    return 1";
    let context1 = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.py"),
        old_tree: None,
        content: content1.to_string(),
    };
    let result1 = parser.parse(&context1).unwrap();
    
    // Second parse with small modification (using old tree)
    let content2 = "def original_function():\n    return 2";  // Changed return value
    let context2 = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.py"),
        old_tree: Some(result1.tree),
        content: content2.to_string(),
    };
    let result2 = parser.parse(&context2).unwrap();
    
    // Should still find the function
    let func1 = result1.nodes.iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();
    let func2 = result2.nodes.iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();
    
    assert_eq!(func1.name, "original_function");
    assert_eq!(func2.name, "original_function");
} 