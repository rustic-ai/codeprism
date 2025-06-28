//! Integration tests for Java parser

use codeprism_lang_java::{JavaParser, ParseContext};
use std::path::PathBuf;

#[test]
fn test_parse_simple_java_class() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

import java.util.List;
import java.util.ArrayList;

public class HelloWorld {
    private String message;
    
    public HelloWorld(String message) {
        this.message = message;
    }
    
    public void sayHello() {
        System.out.println(message);
    }
    
    public static void main(String[] args) {
        HelloWorld hello = new HelloWorld("Hello, World!");
        hello.sayHello();
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("HelloWorld.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse Java file");
    
    // Verify we got some nodes
    assert!(!result.nodes.is_empty());
    
    // Check for expected node types
    let node_kinds: Vec<_> = result.nodes.iter().map(|n| n.kind).collect();
    
    // Should have module, package, imports, class, constructor, method, field
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Module)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Package)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Import)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Class)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Constructor)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Method)));
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Field)));
}

#[test]
fn test_parse_interface() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

public interface Drawable {
    void draw();
    default void print() {
        System.out.println("Drawing...");
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("Drawable.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse Java interface");
    
    // Check for interface node
    let node_kinds: Vec<_> = result.nodes.iter().map(|n| n.kind).collect();
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Interface)));
}

#[test]
fn test_parse_enum() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

public enum Color {
    RED, GREEN, BLUE;
    
    public String getName() {
        return name().toLowerCase();
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("Color.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse Java enum");
    
    // Check for enum node
    let node_kinds: Vec<_> = result.nodes.iter().map(|n| n.kind).collect();
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Enum)));
}

#[test]
fn test_parse_annotations() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

import javax.annotation.Nullable;

public class AnnotatedClass {
    @Nullable
    private String value;
    
    @Override
    public String toString() {
        return value;
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("AnnotatedClass.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse annotated Java class");
    
    // Check for annotation nodes
    let node_kinds: Vec<_> = result.nodes.iter().map(|n| n.kind).collect();
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Annotation)));
}

#[test]
fn test_method_calls() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

public class MethodCalls {
    public void example() {
        System.out.println("Hello");
        String.valueOf(42);
        this.doSomething();
    }
    
    private void doSomething() {
        // Implementation
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("MethodCalls.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse method calls");
    
    // Check for call nodes
    let node_kinds: Vec<_> = result.nodes.iter().map(|n| n.kind).collect();
    assert!(node_kinds.iter().any(|k| matches!(k, codeprism_lang_java::NodeKind::Call)));
}

#[test]
fn test_parser_metadata() {
    let mut parser = JavaParser::new();
    
    let java_code = r#"
package com.example;

public final class FinalClass {
    private static final String CONSTANT = "value";
    protected volatile boolean flag;
    
    public synchronized void syncMethod() {
        // Implementation
    }
}
"#;

    let context = ParseContext {
        repo_id: "test-repo".to_string(),
        file_path: PathBuf::from("FinalClass.java"),
        old_tree: None,
        content: java_code.to_string(),
    };

    let result = parser.parse(&context).expect("Failed to parse Java class with modifiers");
    
    // Find the class node and check its metadata
    let class_node = result.nodes.iter()
        .find(|n| matches!(n.kind, codeprism_lang_java::NodeKind::Class))
        .expect("Should have a class node");
    
    let metadata = &class_node.metadata;
    assert!(metadata.get("is_final").and_then(|v| v.as_bool()).unwrap_or(false));
    assert_eq!(metadata.get("visibility").and_then(|v| v.as_str()).unwrap_or(""), "public");
} 