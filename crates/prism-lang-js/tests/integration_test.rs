//! Integration tests for JavaScript/TypeScript parser

use prism_lang_js::{JavaScriptParser, ParseContext};
use std::fs;
use std::path::PathBuf;

fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_parse_simple_javascript() {
    let mut parser = JavaScriptParser::new();
    let file_path = get_fixture_path("simple.js");
    let content = fs::read_to_string(&file_path).expect("Failed to read fixture");

    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: file_path.clone(),
        old_tree: None,
        content,
    };

    let result = parser.parse(&context).expect("Failed to parse");

    // Verify we found the expected nodes
    let node_names: Vec<_> = result.nodes.iter().map(|n| &n.name).collect();

    // Should find module, function, class, method, variables, and calls
    assert!(node_names.contains(&&"simple".to_string()));
    assert!(node_names.contains(&&"greet".to_string()));
    assert!(node_names.contains(&&"Person".to_string()));
    assert!(node_names.contains(&&"sayHello".to_string()));
    assert!(node_names.contains(&&"message".to_string()));

    // Verify edges exist
    assert!(!result.edges.is_empty());
}

#[test]
fn test_parse_imports() {
    let mut parser = JavaScriptParser::new();
    let file_path = get_fixture_path("imports.js");
    let content = fs::read_to_string(&file_path).expect("Failed to read fixture");

    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: file_path.clone(),
        old_tree: None,
        content,
    };

    let result = parser.parse(&context).expect("Failed to parse");

    // Find import nodes
    let imports: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, prism_lang_js::NodeKind::Import))
        .map(|n| &n.name)
        .collect();

    // Should find all imports
    assert!(imports.contains(&&"react".to_string()));
    assert!(imports.contains(&&"./utils".to_string()));
    assert!(imports.contains(&&"./styles.css".to_string()));

    // Should find the App function
    assert!(result
        .nodes
        .iter()
        .any(|n| n.name == "App" && matches!(n.kind, prism_lang_js::NodeKind::Function)));
}

#[test]
fn test_parse_typescript() {
    let mut parser = JavaScriptParser::new();
    let file_path = get_fixture_path("typescript.ts");
    let content = fs::read_to_string(&file_path).expect("Failed to read fixture");

    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: file_path.clone(),
        old_tree: None,
        content,
    };

    let result = parser.parse(&context).expect("Failed to parse");

    // Verify TypeScript detection
    let lang = result
        .nodes
        .iter()
        .find(|n| matches!(n.kind, prism_lang_js::NodeKind::Module))
        .map(|n| n.lang)
        .expect("Should have module node");

    assert_eq!(lang, prism_lang_js::Language::TypeScript);

    // Should find the UserService class and its methods
    assert!(result
        .nodes
        .iter()
        .any(|n| n.name == "UserService" && matches!(n.kind, prism_lang_js::NodeKind::Class)));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.name == "getUser" && matches!(n.kind, prism_lang_js::NodeKind::Method)));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.name == "createUser" && matches!(n.kind, prism_lang_js::NodeKind::Method)));

    // Should find function calls
    let calls: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, prism_lang_js::NodeKind::Call))
        .map(|n| &n.name)
        .collect();

    assert!(calls.iter().any(|name| name.contains("fetch")));
}

#[test]
fn test_edge_extraction() {
    let mut parser = JavaScriptParser::new();
    let content = r#"
        function a() {
            b();
        }
        
        function b() {
            console.log("Hello");
        }
        
        a();
    "#;

    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.js"),
        old_tree: None,
        content: content.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse");

    // Should have edges for function calls
    let call_edges: Vec<_> = result
        .edges
        .iter()
        .filter(|e| matches!(e.kind, prism_lang_js::EdgeKind::Calls))
        .collect();

    assert!(!call_edges.is_empty(), "Should have call edges");
}
