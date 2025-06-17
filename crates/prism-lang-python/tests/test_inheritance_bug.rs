/// Test for inheritance parsing bug
/// 
/// This test checks if the Python parser correctly captures inheritance relationships
/// and creates the appropriate Call nodes and edges for base classes.

use prism_lang_python::*;
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
fn test_simple_inheritance() {
    let source = r#"
class Parent:
    def parent_method(self):
        pass

class Child(Parent):
    def child_method(self):
        pass
"#;
    
    let result = parse_python_code(source);
    
    // Find class nodes
    let class_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Class))
        .collect();
    
    println!("Class nodes found:");
    for node in &class_nodes {
        println!("  Class: '{}' (id: {})", node.name, node.id.to_hex());
    }
    
    // Find call nodes (should include inheritance references)
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Call nodes found:");
    for node in &call_nodes {
        println!("  Call: '{}' (id: {})", node.name, node.id.to_hex());
    }
    
    // Look for edges representing inheritance
    println!("Edges found:");
    for edge in &result.edges {
        if let (Some(source_node), Some(target_node)) = (
            result.nodes.iter().find(|n| n.id == edge.source),
            result.nodes.iter().find(|n| n.id == edge.target)
        ) {
            println!("  {} -> {} (kind: {:?})", 
                source_node.name, target_node.name, edge.kind);
        }
    }
    
    // Should have inheritance edge from Child to Parent
    let child_node = class_nodes.iter().find(|n| n.name == "Child").unwrap();
    let child_edges: Vec<_> = result.edges.iter()
        .filter(|e| e.source == child_node.id)
        .collect();
    
    println!("Child class outgoing edges:");
    for edge in &child_edges {
        if let Some(target_node) = result.nodes.iter().find(|n| n.id == edge.target) {
            println!("  Child -> {} (kind: {:?})", target_node.name, edge.kind);
        }
    }
    
    // Check if we have a Call node representing the Parent inheritance
    let parent_call_nodes: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == "Parent")
        .collect();
    
    if parent_call_nodes.is_empty() {
        println!("❌ No Call node found for Parent inheritance");
    } else {
        println!("✅ Found {} Call node(s) for Parent inheritance", parent_call_nodes.len());
    }
    
    // For now, just report what we found - don't assert until we understand the issue
    println!("Summary: {} classes, {} calls, {} edges", 
        class_nodes.len(), call_nodes.len(), result.edges.len());
}

#[test]
fn test_generic_inheritance() {
    let source = r#"
from typing import Generic, TypeVar

T = TypeVar('T')

class Agent(Generic[T]):
    def get_spec(self):
        pass

class GuildManagerAgent(Agent[GuildManagerAgentProps]):
    def process(self):
        pass
"#;
    
    let result = parse_python_code(source);
    
    // Find class nodes
    let class_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Class))
        .collect();
    
    println!("Generic inheritance test - Class nodes found:");
    for node in &class_nodes {
        println!("  Class: '{}' (id: {})", node.name, node.id.to_hex());
    }
    
    // Find call nodes (should include inheritance references)
    let call_nodes: Vec<_> = result.nodes.iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();
    
    println!("Call nodes found:");
    for node in &call_nodes {
        println!("  Call: '{}' (id: {})", node.name, node.id.to_hex());
    }
    
    // Look for GuildManagerAgent inheritance
    let guild_manager = class_nodes.iter().find(|n| n.name == "GuildManagerAgent").unwrap();
    let guild_manager_edges: Vec<_> = result.edges.iter()
        .filter(|e| e.source == guild_manager.id)
        .collect();
    
    println!("GuildManagerAgent outgoing edges:");
    for edge in &guild_manager_edges {
        if let Some(target_node) = result.nodes.iter().find(|n| n.id == edge.target) {
            println!("  GuildManagerAgent -> {} (kind: {:?})", target_node.name, edge.kind);
        }
    }
    
    // Check for Agent inheritance Call node
    let agent_call_nodes: Vec<_> = call_nodes.iter()
        .filter(|n| n.name == "Agent")
        .collect();
    
    if agent_call_nodes.is_empty() {
        println!("❌ No Call node found for Agent inheritance");
    } else {
        println!("✅ Found {} Call node(s) for Agent inheritance", agent_call_nodes.len());
    }
    
    println!("Summary: {} classes, {} calls, {} edges", 
        class_nodes.len(), call_nodes.len(), result.edges.len());
}

#[test]
fn test_debug_tree_structure() {
    // Simple case to debug tree-sitter parsing
    let source = "class Child(Parent): pass";
    
    let result = parse_python_code(source);
    
    println!("Debug tree structure for: {}", source);
    println!("Nodes found:");
    for node in &result.nodes {
        println!("  {}: '{}' (kind: {:?})", 
            node.id.to_hex(), node.name, node.kind);
    }
    
    println!("Edges found:");
    for edge in &result.edges {
        if let (Some(source_node), Some(target_node)) = (
            result.nodes.iter().find(|n| n.id == edge.source),
            result.nodes.iter().find(|n| n.id == edge.target)
        ) {
            println!("  {} -> {} (kind: {:?})", 
                source_node.name, target_node.name, edge.kind);
        }
    }
} 