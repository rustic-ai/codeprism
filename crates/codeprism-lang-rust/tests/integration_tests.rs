//! Integration tests for Rust parser

use codeprism_lang_rust::*;
use std::path::PathBuf;

fn create_test_context(content: &str) -> ParseContext {
    ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: content.to_string(),
    }
}

#[test]
fn test_basic_function_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("fn hello() -> &'static str {\n    \"world\"\n}");

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty());

    // Should have at least a module node and a function node
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Module)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Function)));
}

#[test]
fn test_struct_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("struct Point { x: i32, y: i32 }");

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty());

    // Should have module and struct nodes
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Module)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Struct)));
}

#[test]
fn test_trait_and_impl_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Display { fn fmt(&self); }\nimpl Display for String { fn fmt(&self) {} }",
    );

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty());

    // Should have trait and impl nodes
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Trait)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Impl)));
}

#[test]
fn test_enum_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("enum Color { Red, Green, Blue }");

    let result = parser.parse(&context).unwrap();

    // Should have enum node
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Enum)));
}

#[test]
fn test_use_statements() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "use std::collections::HashMap;\nuse serde::{Serialize, Deserialize};",
    );

    let result = parser.parse(&context).unwrap();

    let use_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Use))
        .collect();

    // Should have at least one use node
    assert!(!use_nodes.is_empty());
}

#[test]
fn test_ownership_patterns() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "fn process_data(data: Vec<String>, buffer: &mut [u8], reference: &str) -> &str { reference }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have function and parameter nodes
    let func_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Function))
        .collect();
    assert_eq!(func_nodes.len(), 1);

    let param_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Parameter))
        .collect();

    // Should have parameters with ownership information
    assert!(param_nodes.len() >= 3);

    // Check that at least one parameter has ownership metadata
    let has_ownership_metadata = param_nodes.iter().any(|node| {
        node.metadata
            .as_object()
            .map_or(false, |metadata| metadata.contains_key("ownership"))
    });
    assert!(has_ownership_metadata);
}

#[test]
fn test_lifetime_annotations() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have lifetime nodes
    let lifetime_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Lifetime))
        .collect();

    // Should have at least one lifetime node
    assert!(lifetime_nodes.len() >= 1);

    // Check for 'a lifetime
    let has_a_lifetime = lifetime_nodes.iter().any(|node| node.name.contains("'a"));
    assert!(has_a_lifetime);
}

#[test]
fn test_trait_bounds_and_impl() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Clone { fn clone(&self) -> Self; }\nstruct Point { x: i32, y: i32 }\nimpl Clone for Point { fn clone(&self) -> Self { *self } }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have trait and impl nodes
    let trait_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Trait))
        .collect();
    assert_eq!(trait_nodes.len(), 1);

    let impl_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Impl))
        .collect();
    assert_eq!(impl_nodes.len(), 1);

    // Check impl metadata
    let impl_node = &impl_nodes[0];
    assert!(impl_node.metadata.as_object().map_or(false, |metadata| {
        metadata.get("impl_type") == Some(&serde_json::Value::String("trait_impl".to_string()))
    }));
}

#[test]
fn test_derive_attributes() {
    let mut parser = RustParser::new();
    let context = create_test_context("#[derive(Debug, Clone, PartialEq)]\nstruct Point { x: i32, y: i32 }");

    let result = parser.parse(&context).unwrap();

    // Should have attribute nodes
    let attr_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Attribute))
        .collect();

    assert!(attr_nodes.len() >= 1);

    // Check for derive attribute with traits
    let has_derive_attr = attr_nodes.iter().any(|node| {
        node.name.contains("derive")
            && node.metadata.as_object().map_or(false, |metadata| {
                metadata.get("attribute_type")
                    == Some(&serde_json::Value::String("derive".to_string()))
            })
    });
    assert!(has_derive_attr);
}

#[test]
fn test_macro_invocations() {
    let mut parser = RustParser::new();
    let context = create_test_context("fn main() { println!(\"Hello, world!\"); vec![1, 2, 3]; }");

    let result = parser.parse(&context).unwrap();

    // Should have call nodes for macro invocations
    let call_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();

    // Should have at least println! and vec! macro calls
    assert!(call_nodes.len() >= 2);

    // Check for macro call names
    let has_println = call_nodes.iter().any(|node| node.name.contains("println!"));
    let has_vec = call_nodes.iter().any(|node| node.name.contains("vec!"));

    assert!(has_println);
    assert!(has_vec);
}

#[test]
fn test_complex_rust_features() {
    let mut parser = RustParser::new();
    let complex_code = r#"
        use std::collections::HashMap;
        
        #[derive(Debug, Clone)]
        pub struct Config<'a> {
            name: &'a str,
            values: HashMap<String, i32>,
        }
        
        trait Configurable {
            fn configure(&mut self);
        }
        
        impl<'a> Configurable for Config<'a> {
            fn configure(&mut self) {
                self.values.insert("default".to_string(), 42);
            }
        }
        
        fn process<T: Configurable>(mut item: T) -> T {
            item.configure();
            item
        }
        
        const DEFAULT_SIZE: usize = 1024;
        static mut GLOBAL_COUNTER: i32 = 0;
    "#;
    
    let context = create_test_context(complex_code);
    let result = parser.parse(&context).unwrap();

    // Verify various node types are present
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Use)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Struct)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Trait)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Impl)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Function)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Const)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Static)));
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Attribute)));
}

#[test]
fn test_rust_analyzer_integration() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Display { fn fmt(&self); }\nimpl Display for String { fn fmt(&self) {} }",
    );

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);

    // Test trait implementation analysis
    let trait_impls = analyzer.analyze_trait_implementations();
    assert!(!trait_impls.is_empty());
    
    let display_impl = trait_impls.iter().find(|impl_| impl_.trait_name == "Display");
    assert!(display_impl.is_some());
    
    if let Some(impl_) = display_impl {
        assert_eq!(impl_.type_name, "String");
    }
}

#[test]
fn test_incremental_parsing() {
    let mut parser = RustParser::new();

    // First parse
    let context1 = create_test_context("fn foo() -> i32 {\n    1\n}");
    let result1 = parser.parse(&context1).unwrap();

    // Second parse with small edit
    let mut context2 = create_test_context("fn foo() -> i32 {\n    2\n}");
    context2.old_tree = Some(result1.tree);
    let result2 = parser.parse(&context2).unwrap();

    // Both should have the same structure
    assert_eq!(result1.nodes.len(), result2.nodes.len());

    // Function should still be found
    let func1 = result1
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();
    let func2 = result2
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();

    assert_eq!(func1.name, "foo");
    assert_eq!(func2.name, "foo");
} 