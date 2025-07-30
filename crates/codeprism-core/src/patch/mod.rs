//! AST patch generation and application

use crate::ast::{Edge, Node};
use serde::{Deserialize, Serialize};

/// AST patch containing changes to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPatch {
    /// Repository ID
    pub repo: String,
    /// Commit SHA
    pub commit: String,
    /// Nodes to add
    pub nodes_add: Vec<Node>,
    /// Edges to add
    pub edges_add: Vec<Edge>,
    /// Node IDs to delete
    pub nodes_delete: Vec<String>,
    /// Edge IDs to delete
    pub edges_delete: Vec<String>,
    /// Timestamp in milliseconds
    pub timestamp_ms: i64,
}

impl AstPatch {
    /// Create a new empty patch
    pub fn new(repo: String, commit: String) -> Self {
        Self {
            repo,
            commit,
            nodes_add: Vec::new(),
            edges_add: Vec::new(),
            nodes_delete: Vec::new(),
            edges_delete: Vec::new(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.nodes_add.is_empty()
            && self.edges_add.is_empty()
            && self.nodes_delete.is_empty()
            && self.edges_delete.is_empty()
    }

    /// Get the total number of operations in the patch
    pub fn operation_count(&self) -> usize {
        self.nodes_add.len()
            + self.edges_add.len()
            + self.nodes_delete.len()
            + self.edges_delete.len()
    }

    /// Merge another patch into this one
    pub fn merge(&mut self, other: AstPatch) {
        self.nodes_add.extend(other.nodes_add);
        self.edges_add.extend(other.edges_add);
        self.nodes_delete.extend(other.nodes_delete);
        self.edges_delete.extend(other.edges_delete);
        // Update timestamp to the latest
        if other.timestamp_ms > self.timestamp_ms {
            self.timestamp_ms = other.timestamp_ms;
        }
    }
}

/// Builder for creating AST patches
pub struct PatchBuilder {
    patch: AstPatch,
}

impl PatchBuilder {
    /// Create a new patch builder
    pub fn new(repo: String, commit: String) -> Self {
        Self {
            patch: AstPatch::new(repo, commit),
        }
    }

    /// Add a node to the patch
    pub fn add_node(mut self, node: Node) -> Self {
        self.patch.nodes_add.push(node);
        self
    }

    /// Add multiple nodes to the patch
    pub fn add_nodes(mut self, nodes: Vec<Node>) -> Self {
        self.patch.nodes_add.extend(nodes);
        self
    }

    /// Add an edge to the patch
    pub fn add_edge(mut self, edge: Edge) -> Self {
        self.patch.edges_add.push(edge);
        self
    }

    /// Add multiple edges to the patch
    pub fn add_edges(mut self, edges: Vec<Edge>) -> Self {
        self.patch.edges_add.extend(edges);
        self
    }

    /// Delete a node
    pub fn delete_node(mut self, node_id: String) -> Self {
        self.patch.nodes_delete.push(node_id);
        self
    }

    /// Delete multiple nodes
    pub fn delete_nodes(mut self, node_ids: Vec<String>) -> Self {
        self.patch.nodes_delete.extend(node_ids);
        self
    }

    /// Delete an edge
    pub fn delete_edge(mut self, edge_id: String) -> Self {
        self.patch.edges_delete.push(edge_id);
        self
    }

    /// Delete multiple edges
    pub fn delete_edges(mut self, edge_ids: Vec<String>) -> Self {
        self.patch.edges_delete.extend(edge_ids);
        self
    }

    /// Set custom timestamp
    pub fn with_timestamp(mut self, timestamp_ms: i64) -> Self {
        self.patch.timestamp_ms = timestamp_ms;
        self
    }

    /// Build the patch
    pub fn build(self) -> AstPatch {
        self.patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{EdgeKind, Language, NodeKind, Span};
    use std::path::PathBuf;

    fn create_test_node(name: &str) -> Node {
        let span = Span::new(0, 10, 1, 1, 1, 11);
        Node::new(
            "test_repo",
            NodeKind::Function,
            name.to_string(),
            Language::JavaScript,
            PathBuf::from("test.js"),
            span,
        )
    }

    fn create_test_edge(source: &Node, target: &Node) -> Edge {
        Edge::new(source.id, target.id, EdgeKind::Calls)
    }

    #[test]
    fn test_patch_creation() {
        let patch = AstPatch::new("test_repo".to_string(), "abc123".to_string());
        assert_eq!(patch.repo, "test_repo");
        assert_eq!(patch.commit, "abc123");
        assert!(patch.is_empty(), "New patch should be empty");
        assert_eq!(patch.operation_count(), 0);
        assert!(patch.timestamp_ms > 0, "Patch should have valid timestamp");
    }

    #[test]
    fn test_patch_builder_basic() {
        let node1 = create_test_node("func1");
        let node2 = create_test_node("func2");
        let edge = create_test_edge(&node1, &node2);

        let patch = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .add_node(node1.clone())
            .add_node(node2.clone())
            .add_edge(edge.clone())
            .delete_node("old_node_id".to_string())
            .delete_edge("old_edge_id".to_string())
            .build();

        assert_eq!(patch.nodes_add.len(), 2, "Should have 2 nodes to add");
        assert_eq!(patch.edges_add.len(), 1, "Should have 1 edge to add");
        assert_eq!(patch.nodes_delete.len(), 1, "Should have 1 node to delete");
        assert_eq!(patch.edges_delete.len(), 1, "Should have 1 edge to delete");
        assert_eq!(patch.operation_count(), 5, "Total operations should be 5");
        assert!(
            !patch.is_empty(),
            "Patch with operations should not be empty"
        );

        // Verify actual content of operations
        assert!(
            patch.nodes_add.iter().any(|n| n.kind == NodeKind::Function),
            "Should add function node"
        );
        assert!(
            patch.nodes_add.iter().any(|n| n.kind == NodeKind::Variable),
            "Should add variable node"
        );
        assert!(
            patch.nodes_delete.iter().any(|n| n.kind == NodeKind::Class),
            "Should delete class node"
        );
    }

    #[test]
    fn test_patch_builder_batch_operations() {
        let nodes = vec![
            create_test_node("func1"),
            create_test_node("func2"),
            create_test_node("func3"),
        ];
        let edges = vec![
            create_test_edge(&nodes[0], &nodes[1]),
            create_test_edge(&nodes[1], &nodes[2]),
        ];

        let patch = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .add_nodes(nodes.clone())
            .add_edges(edges.clone())
            .delete_nodes(vec!["id1".to_string(), "id2".to_string()])
            .delete_edges(vec!["edge1".to_string(), "edge2".to_string()])
            .build();

        assert_eq!(patch.nodes_add.len(), 3, "Should have 3 items");
        assert_eq!(patch.edges_add.len(), 2, "Should have 2 items");
        assert_eq!(patch.nodes_delete.len(), 2, "Should have 2 items");
        assert_eq!(patch.edges_delete.len(), 2, "Should have 2 items");
        assert_eq!(patch.operation_count(), 9);
    }

    #[test]
    fn test_patch_serialization() {
        let node = create_test_node("test_func");
        let patch = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .add_node(node)
            .with_timestamp(1234567890)
            .build();

        // Test JSON serialization
        let json = serde_json::to_string(&patch).unwrap();
        let deserialized: AstPatch = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.repo, patch.repo);
        assert_eq!(deserialized.commit, patch.commit);
        assert_eq!(deserialized.nodes_add.len(), patch.nodes_add.len());
        assert_eq!(deserialized.timestamp_ms, 1234567890);
    }

    #[test]
    fn test_patch_merge() {
        let node1 = create_test_node("func1");
        let node2 = create_test_node("func2");
        let edge = create_test_edge(&node1, &node2);

        let mut patch1 = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .add_node(node1.clone())
            .delete_node("old1".to_string())
            .with_timestamp(1000)
            .build();

        let patch2 = PatchBuilder::new("test_repo".to_string(), "def456".to_string())
            .add_node(node2.clone())
            .add_edge(edge.clone())
            .delete_edge("old_edge".to_string())
            .with_timestamp(2000)
            .build();

        patch1.merge(patch2);

        assert_eq!(patch1.nodes_add.len(), 2, "Should have 2 items");
        assert_eq!(patch1.edges_add.len(), 1, "Should have 1 items");
        assert_eq!(patch1.nodes_delete.len(), 1, "Should have 1 items");
        assert_eq!(patch1.edges_delete.len(), 1, "Should have 1 items");
        assert_eq!(patch1.timestamp_ms, 2000); // Should use the latest timestamp
        assert_eq!(patch1.operation_count(), 5);
    }

    #[test]
    fn test_empty_patch() {
        let patch = AstPatch::new("test_repo".to_string(), "abc123".to_string());
        assert!(patch.is_empty(), "New patch should be empty");
        assert_eq!(patch.operation_count(), 0);

        // Empty patch should serialize/deserialize correctly
        let json = serde_json::to_string(&patch).unwrap();
        let deserialized: AstPatch = serde_json::from_str(&json).unwrap();
        assert!(
            deserialized.is_empty(),
            "Empty patch should remain empty after serialization"
        );
    }

    #[test]
    fn test_patch_with_custom_timestamp() {
        let custom_timestamp = 9876543210;
        let patch = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .with_timestamp(custom_timestamp)
            .build();

        assert_eq!(patch.timestamp_ms, custom_timestamp);
    }

    #[test]
    fn test_patch_validation() {
        // Test that patch can handle nodes with same IDs (deduplication would be done at apply time)
        let node = create_test_node("func");
        let patch = PatchBuilder::new("test_repo".to_string(), "abc123".to_string())
            .add_node(node.clone())
            .add_node(node.clone()) // Same node added twice
            .build();

        assert_eq!(patch.nodes_add.len(), 2, "Should have 2 items"); // Both are kept in the patch
    }

    #[test]
    fn test_large_patch() {
        let mut builder = PatchBuilder::new("test_repo".to_string(), "abc123".to_string());

        // Add many nodes
        for i in 0..100 {
            let node = create_test_node(&format!("func{i}"));
            builder = builder.add_node(node);
        }

        // Add many deletions
        for i in 0..50 {
            builder = builder.delete_node(format!("old_node_{i}"));
            builder = builder.delete_edge(format!("old_edge_{i}"));
        }

        let patch = builder.build();
        assert_eq!(patch.nodes_add.len(), 100, "Should have 100 items");
        assert_eq!(patch.nodes_delete.len(), 50, "Should have 50 items");
        assert_eq!(patch.edges_delete.len(), 50, "Should have 50 items");
        assert_eq!(patch.operation_count(), 200);
    }
}
