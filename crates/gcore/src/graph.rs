//! Graph storage and query engine for code intelligence
//!
//! This module provides in-memory graph storage with efficient querying capabilities
//! for supporting advanced MCP tools like trace_path, find_references, etc.

use crate::ast::{Edge, EdgeKind, Node, NodeId, NodeKind};
use crate::error::{Error, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

/// In-memory graph store for code intelligence
#[derive(Debug)]
pub struct GraphStore {
    /// All nodes indexed by their ID
    nodes: Arc<DashMap<NodeId, Node>>,
    /// Outgoing edges from each node
    outgoing_edges: Arc<DashMap<NodeId, Vec<Edge>>>,
    /// Incoming edges to each node
    incoming_edges: Arc<DashMap<NodeId, Vec<Edge>>>,
    /// Index of nodes by file path
    file_index: Arc<DashMap<PathBuf, Vec<NodeId>>>,
    /// Index of nodes by symbol name
    symbol_index: Arc<DashMap<String, Vec<NodeId>>>,
    /// Index of nodes by kind
    kind_index: Arc<DashMap<NodeKind, Vec<NodeId>>>,
}

impl GraphStore {
    /// Create a new empty graph store
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(DashMap::new()),
            outgoing_edges: Arc::new(DashMap::new()),
            incoming_edges: Arc::new(DashMap::new()),
            file_index: Arc::new(DashMap::new()),
            symbol_index: Arc::new(DashMap::new()),
            kind_index: Arc::new(DashMap::new()),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&self, node: Node) {
        let node_id = node.id;
        
        // Add to file index
        self.file_index
            .entry(node.file.clone())
            .or_insert_with(Vec::new)
            .push(node_id);
        
        // Add to symbol index
        self.symbol_index
            .entry(node.name.clone())
            .or_insert_with(Vec::new)
            .push(node_id);
        
        // Add to kind index
        self.kind_index
            .entry(node.kind)
            .or_insert_with(Vec::new)
            .push(node_id);
        
        // Add the node
        self.nodes.insert(node_id, node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&self, edge: Edge) {
        // Add to outgoing edges
        self.outgoing_edges
            .entry(edge.source)
            .or_insert_with(Vec::new)
            .push(edge.clone());
        
        // Add to incoming edges
        self.incoming_edges
            .entry(edge.target)
            .or_insert_with(Vec::new)
            .push(edge);
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        self.nodes.get(node_id).map(|entry| entry.clone())
    }

    /// Get all nodes in a file
    pub fn get_nodes_in_file(&self, file_path: &PathBuf) -> Vec<Node> {
        if let Some(node_ids) = self.file_index.get(file_path) {
            node_ids
                .iter()
                .filter_map(|id| self.get_node(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get nodes by symbol name
    pub fn get_nodes_by_name(&self, name: &str) -> Vec<Node> {
        if let Some(node_ids) = self.symbol_index.get(name) {
            node_ids
                .iter()
                .filter_map(|id| self.get_node(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get nodes by kind
    pub fn get_nodes_by_kind(&self, kind: NodeKind) -> Vec<Node> {
        if let Some(node_ids) = self.kind_index.get(&kind) {
            node_ids
                .iter()
                .filter_map(|id| self.get_node(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<Edge> {
        self.outgoing_edges
            .get(node_id)
            .map(|edges| edges.clone())
            .unwrap_or_default()
    }

    /// Get incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<Edge> {
        self.incoming_edges
            .get(node_id)
            .map(|edges| edges.clone())
            .unwrap_or_default()
    }

    /// Get graph statistics
    pub fn get_stats(&self) -> GraphStats {
        GraphStats {
            total_nodes: self.nodes.len(),
            total_edges: self.outgoing_edges.iter().map(|entry| entry.len()).sum(),
            total_files: self.file_index.len(),
            nodes_by_kind: self.kind_index
                .iter()
                .map(|entry| (*entry.key(), entry.len()))
                .collect(),
        }
    }

    /// Clear all data from the graph
    pub fn clear(&self) {
        self.nodes.clear();
        self.outgoing_edges.clear();
        self.incoming_edges.clear();
        self.file_index.clear();
        self.symbol_index.clear();
        self.kind_index.clear();
    }

    /// Remove a node and all its edges
    pub fn remove_node(&self, node_id: &NodeId) -> Option<Node> {
        if let Some((_, node)) = self.nodes.remove(node_id) {
            // Remove from indices
            if let Some(mut file_nodes) = self.file_index.get_mut(&node.file) {
                file_nodes.retain(|id| id != node_id);
            }
            
            if let Some(mut symbol_nodes) = self.symbol_index.get_mut(&node.name) {
                symbol_nodes.retain(|id| id != node_id);
            }
            
            if let Some(mut kind_nodes) = self.kind_index.get_mut(&node.kind) {
                kind_nodes.retain(|id| id != node_id);
            }
            
            // Remove edges
            self.outgoing_edges.remove(node_id);
            self.incoming_edges.remove(node_id);
            
            // Remove edges that reference this node
            for mut edges in self.outgoing_edges.iter_mut() {
                edges.retain(|edge| edge.target != *node_id);
            }
            
            for mut edges in self.incoming_edges.iter_mut() {
                edges.retain(|edge| edge.source != *node_id);
            }
            
            Some(node)
        } else {
            None
        }
    }

    /// Get all file paths in the index
    pub fn get_all_files(&self) -> Vec<PathBuf> {
        self.file_index.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Iterate over file index entries (file path -> node IDs)
    pub fn iter_file_index(&self) -> impl Iterator<Item = (PathBuf, Vec<NodeId>)> + '_ {
        self.file_index.iter().map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    /// Iterate over symbol index entries (symbol name -> node IDs)
    pub fn iter_symbol_index(&self) -> impl Iterator<Item = (String, Vec<NodeId>)> + '_ {
        self.symbol_index.iter().map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    /// Get nodes by file path
    pub fn get_nodes_by_file(&self, file_path: &PathBuf) -> Vec<NodeId> {
        self.file_index.get(file_path).map(|ids| ids.clone()).unwrap_or_default()
    }

    /// Get nodes by symbol name
    pub fn get_node_ids_by_name(&self, name: &str) -> Vec<NodeId> {
        self.symbol_index.get(name).map(|ids| ids.clone()).unwrap_or_default()
    }
}

impl Default for GraphStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Total number of files
    pub total_files: usize,
    /// Nodes grouped by kind
    pub nodes_by_kind: HashMap<NodeKind, usize>,
}

/// Graph query engine for advanced operations
pub struct GraphQuery {
    graph: Arc<GraphStore>,
}

impl GraphQuery {
    /// Create a new graph query engine
    pub fn new(graph: Arc<GraphStore>) -> Self {
        Self { graph }
    }

    /// Find the shortest path between two nodes
    pub fn find_path(&self, source: &NodeId, target: &NodeId, max_depth: Option<usize>) -> Result<Option<PathResult>> {
        let max_depth = max_depth.unwrap_or(10);
        
        if source == target {
            return Ok(Some(PathResult {
                source: *source,
                target: *target,
                path: vec![*source],
                distance: 0,
                edges: Vec::new(),
            }));
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();
        let mut edge_map = HashMap::new();

        queue.push_back((*source, 0));
        visited.insert(*source);

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for edge in self.graph.get_outgoing_edges(&current) {
                if !visited.contains(&edge.target) {
                    visited.insert(edge.target);
                    parent.insert(edge.target, current);
                    edge_map.insert(edge.target, edge.clone());
                    queue.push_back((edge.target, depth + 1));

                    if edge.target == *target {
                        // Reconstruct path
                        let mut path = Vec::new();
                        let mut edges = Vec::new();
                        let mut current_node = *target;

                        // Build path from target back to source
                        path.push(current_node);
                        while let Some(&prev) = parent.get(&current_node) {
                            if let Some(edge) = edge_map.get(&current_node) {
                                edges.push(edge.clone());
                            }
                            current_node = prev;
                            path.push(current_node);
                        }
                        
                        // Reverse to get path from source to target
                        path.reverse();
                        edges.reverse();

                        let distance = path.len() - 1; // Number of edges, not nodes

                        return Ok(Some(PathResult {
                            source: *source,
                            target: *target,
                            path,
                            distance,
                            edges,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find all references to a symbol (incoming edges)
    pub fn find_references(&self, node_id: &NodeId) -> Result<Vec<SymbolReference>> {
        let mut references = Vec::new();
        
        for edge in self.graph.get_incoming_edges(node_id) {
            if let Some(source_node) = self.graph.get_node(&edge.source) {
                references.push(SymbolReference {
                    location: ReferenceLocation {
                        file: source_node.file.clone(),
                        span: source_node.span.clone(),
                    },
                    source_node,
                    edge_kind: edge.kind,
                });
            }
        }
        
        Ok(references)
    }

    /// Find all dependencies of a node (outgoing edges)
    pub fn find_dependencies(&self, node_id: &NodeId, dependency_type: DependencyType) -> Result<Vec<SymbolDependency>> {
        let mut dependencies = Vec::new();
        
        for edge in self.graph.get_outgoing_edges(node_id) {
            let include_edge = match dependency_type {
                DependencyType::Direct => true,
                DependencyType::Calls => matches!(edge.kind, EdgeKind::Calls),
                DependencyType::Imports => matches!(edge.kind, EdgeKind::Imports),
                DependencyType::Reads => matches!(edge.kind, EdgeKind::Reads),
                DependencyType::Writes => matches!(edge.kind, EdgeKind::Writes),
            };
            
            if include_edge {
                if let Some(target_node) = self.graph.get_node(&edge.target) {
                    dependencies.push(SymbolDependency {
                        target_node,
                        edge_kind: edge.kind,
                        dependency_type: dependency_type.clone(),
                    });
                }
            }
        }
        
        Ok(dependencies)
    }

    /// Search symbols by name pattern (regex or fuzzy)
    pub fn search_symbols(&self, pattern: &str, symbol_types: Option<Vec<NodeKind>>, limit: Option<usize>) -> Result<Vec<SymbolInfo>> {
        let limit = limit.unwrap_or(50);
        let mut results = Vec::new();
        
        // Simple substring search for now (can be enhanced with regex later)
        for entry in self.graph.symbol_index.iter() {
            if entry.key().to_lowercase().contains(&pattern.to_lowercase()) {
                for node_id in entry.value() {
                    if let Some(node) = self.graph.get_node(node_id) {
                        // Filter by symbol types if specified
                        if let Some(ref types) = symbol_types {
                            if !types.contains(&node.kind) {
                                continue;
                            }
                        }
                        
                        results.push(SymbolInfo {
                            node,
                            references_count: self.graph.get_incoming_edges(node_id).len(),
                            dependencies_count: self.graph.get_outgoing_edges(node_id).len(),
                        });
                        
                        if results.len() >= limit {
                            break;
                        }
                    }
                }
                
                if results.len() >= limit {
                    break;
                }
            }
        }
        
        Ok(results)
    }
}

/// Result of a path finding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    /// Source node ID
    pub source: NodeId,
    /// Target node ID
    pub target: NodeId,
    /// Path of node IDs from source to target
    pub path: Vec<NodeId>,
    /// Distance (number of hops)
    pub distance: usize,
    /// Edges in the path
    pub edges: Vec<Edge>,
}

/// Information about a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// The node representing the symbol
    pub node: Node,
    /// Number of references to this symbol
    pub references_count: usize,
    /// Number of dependencies from this symbol
    pub dependencies_count: usize,
}

/// A reference to a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    /// The node that references the symbol
    pub source_node: Node,
    /// Type of reference (edge kind)
    pub edge_kind: EdgeKind,
    /// Location of the reference
    pub location: ReferenceLocation,
}

/// Location of a reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLocation {
    /// File containing the reference
    pub file: PathBuf,
    /// Span of the reference
    pub span: crate::ast::Span,
}

/// A dependency of a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDependency {
    /// The target node that is depended upon
    pub target_node: Node,
    /// Type of dependency (edge kind)
    pub edge_kind: EdgeKind,
    /// Dependency classification
    pub dependency_type: DependencyType,
}

/// Type of dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// All direct dependencies
    Direct,
    /// Only function calls
    Calls,
    /// Only imports
    Imports,
    /// Only reads
    Reads,
    /// Only writes
    Writes,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Language, Span};
    use std::path::Path;

    fn create_test_node(name: &str, kind: NodeKind, file: &str) -> Node {
        Node::new(
            "test_repo",
            kind,
            name.to_string(),
            Language::Python,
            PathBuf::from(file),
            Span::new(0, 10, 1, 1, 1, 11),
        )
    }

    fn create_test_node_with_span(name: &str, kind: NodeKind, file: &str, start_byte: usize, end_byte: usize) -> Node {
        Node::new(
            "test_repo",
            kind,
            name.to_string(),
            Language::Python,
            PathBuf::from(file),
            Span::new(start_byte, end_byte, 1, 1, 1, 11),
        )
    }

    #[test]
    fn test_graph_store_basic_operations() {
        let graph = GraphStore::new();
        
        let node1 = create_test_node("function1", NodeKind::Function, "test.py");
        let node2 = create_test_node("function2", NodeKind::Function, "test.py");
        
        graph.add_node(node1.clone());
        graph.add_node(node2.clone());
        
        assert_eq!(graph.get_node(&node1.id).unwrap().id, node1.id);
        assert_eq!(graph.get_node(&node2.id).unwrap().id, node2.id);
        
        let edge = Edge::new(node1.id, node2.id, EdgeKind::Calls);
        graph.add_edge(edge.clone());
        
        let outgoing = graph.get_outgoing_edges(&node1.id);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0], edge);
        
        let incoming = graph.get_incoming_edges(&node2.id);
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0], edge);
    }

    #[test]
    fn test_graph_query_path_finding() {
        let graph = Arc::new(GraphStore::new());
        let query = GraphQuery::new(graph.clone());
        
        let node1 = create_test_node_with_span("function1", NodeKind::Function, "test.py", 0, 10);
        let node2 = create_test_node_with_span("function2", NodeKind::Function, "test.py", 20, 30);
        let node3 = create_test_node_with_span("function3", NodeKind::Function, "test.py", 40, 50);
        
        graph.add_node(node1.clone());
        graph.add_node(node2.clone());
        graph.add_node(node3.clone());
        
        graph.add_edge(Edge::new(node1.id, node2.id, EdgeKind::Calls));
        graph.add_edge(Edge::new(node2.id, node3.id, EdgeKind::Calls));
        
        let path = query.find_path(&node1.id, &node3.id, None).unwrap();
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.distance, 2);
        assert_eq!(path.path, vec![node1.id, node2.id, node3.id]);
    }

    #[test]
    fn test_symbol_search() {
        let graph = Arc::new(GraphStore::new());
        let query = GraphQuery::new(graph.clone());
        
        let node1 = create_test_node("test_function", NodeKind::Function, "test.py");
        let node2 = create_test_node("another_function", NodeKind::Function, "test.py");
        let node3 = create_test_node("test_class", NodeKind::Class, "test.py");
        
        graph.add_node(node1.clone());
        graph.add_node(node2.clone());
        graph.add_node(node3.clone());
        
        let results = query.search_symbols("test", None, None).unwrap();
        assert_eq!(results.len(), 2); // test_function and test_class
        
        let results = query.search_symbols("test", Some(vec![NodeKind::Function]), None).unwrap();
        assert_eq!(results.len(), 1); // only test_function
    }
} 