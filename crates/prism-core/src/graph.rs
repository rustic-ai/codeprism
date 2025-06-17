//! Graph storage and query engine for code intelligence
//!
//! This module provides in-memory graph storage with efficient querying capabilities
//! for supporting advanced MCP tools like trace_path, find_references, etc.

use crate::ast::{Edge, EdgeKind, Node, NodeId, NodeKind};
use crate::error::Result;
use dashmap::DashMap;
use regex;
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
        
        // Try to compile as regex first, fall back to substring search if invalid
        let regex_result = regex::Regex::new(pattern);
        let use_regex = regex_result.is_ok();
        let regex = regex_result.ok();
        
        for entry in self.graph.symbol_index.iter() {
            let symbol_name = entry.key();
            
            let matches = if use_regex {
                // Use regex matching
                regex.as_ref().unwrap().is_match(symbol_name)
            } else {
                // Fall back to case-insensitive substring search
                symbol_name.to_lowercase().contains(&pattern.to_lowercase())
            };
            
            if matches {
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

    /// Search symbols by name pattern with inheritance filters
    pub fn search_symbols_with_inheritance(&self, pattern: &str, symbol_types: Option<Vec<NodeKind>>, inheritance_filters: Option<Vec<InheritanceFilter>>, limit: Option<usize>) -> Result<Vec<SymbolInfo>> {
        let mut results = self.search_symbols(pattern, symbol_types, limit)?;
        
        if let Some(filters) = inheritance_filters {
            results.retain(|symbol_info| {
                filters.iter().any(|filter| self.matches_inheritance_filter(&symbol_info.node, filter))
            });
        }
        
        Ok(results)
    }

    /// Check if a node matches an inheritance filter
    fn matches_inheritance_filter(&self, node: &Node, filter: &InheritanceFilter) -> bool {
        match filter {
            InheritanceFilter::InheritsFrom(base_class) => {
                self.inherits_from(&node.id, base_class).unwrap_or(false)
            }
            InheritanceFilter::HasMetaclass(metaclass) => {
                self.has_metaclass(&node.id, metaclass).unwrap_or(false)
            }
            InheritanceFilter::UsesMixin(mixin) => {
                self.uses_mixin(&node.id, mixin).unwrap_or(false)
            }
        }
    }

    /// Get comprehensive inheritance information for a class
    pub fn get_inheritance_info(&self, node_id: &NodeId) -> Result<InheritanceInfo> {
        let node = self.graph.get_node(node_id)
            .ok_or_else(|| crate::error::Error::NodeNotFound(node_id.to_hex()))?;

        if !matches!(node.kind, NodeKind::Class) {
            return Ok(InheritanceInfo::default());
        }

        let base_classes = self.get_base_classes(node_id)?;
        let subclasses = self.get_subclasses(node_id)?;
        let metaclass = self.get_metaclass(node_id)?;
        let mixins = self.get_mixins(node_id)?;
        let mro = self.calculate_method_resolution_order(node_id)?;
        let dynamic_attributes = self.get_dynamic_attributes(node_id)?;

        Ok(InheritanceInfo {
            class_name: node.name.clone(),
            base_classes,
            subclasses,
            metaclass,
            mixins,
            method_resolution_order: mro,
            dynamic_attributes,
            is_metaclass: self.is_metaclass(node_id)?,
            inheritance_chain: self.get_full_inheritance_chain(node_id)?,
        })
    }

    /// Get direct base classes of a class
    pub fn get_base_classes(&self, node_id: &NodeId) -> Result<Vec<InheritanceRelation>> {
        let mut base_classes = Vec::new();
        
        for edge in self.graph.get_outgoing_edges(node_id) {
            if matches!(edge.kind, EdgeKind::Extends) {
                if let Some(parent_node) = self.graph.get_node(&edge.target) {
                    let is_metaclass = parent_node.metadata
                        .get("is_metaclass")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    
                    base_classes.push(InheritanceRelation {
                        class_name: parent_node.name.clone(),
                        node_id: parent_node.id,
                        relationship_type: if is_metaclass { "metaclass".to_string() } else { "extends".to_string() },
                        file: parent_node.file.clone(),
                        span: parent_node.span.clone(),
                    });
                }
            }
        }
        
        Ok(base_classes)
    }

    /// Get direct subclasses of a class
    pub fn get_subclasses(&self, node_id: &NodeId) -> Result<Vec<InheritanceRelation>> {
        let mut subclasses = Vec::new();
        
        for edge in self.graph.get_incoming_edges(node_id) {
            if matches!(edge.kind, EdgeKind::Extends) {
                if let Some(child_node) = self.graph.get_node(&edge.source) {
                    subclasses.push(InheritanceRelation {
                        class_name: child_node.name.clone(),
                        node_id: child_node.id,
                        relationship_type: "extends".to_string(),
                        file: child_node.file.clone(),
                        span: child_node.span.clone(),
                    });
                }
            }
        }
        
        Ok(subclasses)
    }

    /// Get the metaclass of a class (if any)
    pub fn get_metaclass(&self, node_id: &NodeId) -> Result<Option<InheritanceRelation>> {
        for edge in self.graph.get_outgoing_edges(node_id) {
            if matches!(edge.kind, EdgeKind::Extends) {
                if let Some(parent_node) = self.graph.get_node(&edge.target) {
                    let is_metaclass = parent_node.metadata
                        .get("is_metaclass")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    
                    if is_metaclass {
                        return Ok(Some(InheritanceRelation {
                            class_name: parent_node.name.clone(),
                            node_id: parent_node.id,
                            relationship_type: "metaclass".to_string(),
                            file: parent_node.file.clone(),
                            span: parent_node.span.clone(),
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Get mixins used by a class
    pub fn get_mixins(&self, node_id: &NodeId) -> Result<Vec<InheritanceRelation>> {
        let mut mixins = Vec::new();
        
        for edge in self.graph.get_outgoing_edges(node_id) {
            if matches!(edge.kind, EdgeKind::Extends) {
                if let Some(parent_node) = self.graph.get_node(&edge.target) {
                    // Heuristic: classes ending with "Mixin" or containing "mixin" are considered mixins
                    if parent_node.name.ends_with("Mixin") || 
                       parent_node.name.to_lowercase().contains("mixin") {
                        mixins.push(InheritanceRelation {
                            class_name: parent_node.name.clone(),
                            node_id: parent_node.id,
                            relationship_type: "mixin".to_string(),
                            file: parent_node.file.clone(),
                            span: parent_node.span.clone(),
                        });
                    }
                }
            }
        }
        
        Ok(mixins)
    }

    /// Calculate method resolution order (simplified)
    pub fn calculate_method_resolution_order(&self, node_id: &NodeId) -> Result<Vec<String>> {
        let mut mro = Vec::new();
        let mut visited = HashSet::new();
        
        self.collect_mro_recursive(node_id, &mut mro, &mut visited)?;
        
        Ok(mro)
    }

    /// Recursively collect method resolution order
    fn collect_mro_recursive(&self, node_id: &NodeId, mro: &mut Vec<String>, visited: &mut HashSet<NodeId>) -> Result<()> {
        if visited.contains(node_id) {
            return Ok(());
        }
        
        visited.insert(*node_id);
        
        if let Some(node) = self.graph.get_node(node_id) {
            mro.push(node.name.clone());
            
            // Add parent classes to MRO
            for edge in self.graph.get_outgoing_edges(node_id) {
                if matches!(edge.kind, EdgeKind::Extends) {
                    self.collect_mro_recursive(&edge.target, mro, visited)?;
                }
            }
        }
        
        Ok(())
    }

    /// Get dynamic attributes potentially created by metaclasses or decorators
    pub fn get_dynamic_attributes(&self, node_id: &NodeId) -> Result<Vec<DynamicAttribute>> {
        let mut attributes = Vec::new();
        
        // Look for common patterns that create dynamic attributes
        if let Some(metaclass) = self.get_metaclass(node_id)? {
            // Common metaclass-created attributes
            let common_metaclass_attrs = vec![
                "_registry", "_instances", "_subclasses", "_handlers", "_processors",
                "_mixins", "_plugins", "_decorators", "_metadata"
            ];
            
            for attr_name in common_metaclass_attrs {
                attributes.push(DynamicAttribute {
                    name: attr_name.to_string(),
                    created_by: format!("metaclass:{}", metaclass.class_name),
                    attribute_type: "dynamic".to_string(),
                });
            }
        }
        
        Ok(attributes)
    }

    /// Check if a class is a metaclass
    pub fn is_metaclass(&self, node_id: &NodeId) -> Result<bool> {
        if let Some(node) = self.graph.get_node(node_id) {
            // Check metadata first
            if let Some(is_meta) = node.metadata.get("is_metaclass").and_then(|v| v.as_bool()) {
                return Ok(is_meta);
            }
            
            // Heuristic: inherits from 'type' or name contains 'Meta'
            let inherits_from_type = self.inherits_from(node_id, "type")?;
            let name_suggests_metaclass = node.name.contains("Meta") || node.name.ends_with("Metaclass");
            
            Ok(inherits_from_type || name_suggests_metaclass)
        } else {
            Ok(false)
        }
    }

    /// Get the full inheritance chain up to the root
    pub fn get_full_inheritance_chain(&self, node_id: &NodeId) -> Result<Vec<String>> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        
        self.collect_inheritance_chain_recursive(node_id, &mut chain, &mut visited)?;
        
        Ok(chain)
    }

    /// Recursively collect inheritance chain
    fn collect_inheritance_chain_recursive(&self, node_id: &NodeId, chain: &mut Vec<String>, visited: &mut HashSet<NodeId>) -> Result<()> {
        if visited.contains(node_id) {
            return Ok(());
        }
        
        visited.insert(*node_id);
        
        if let Some(node) = self.graph.get_node(node_id) {
            chain.push(node.name.clone());
            
            // Follow parent classes
            for edge in self.graph.get_outgoing_edges(node_id) {
                if matches!(edge.kind, EdgeKind::Extends) {
                    self.collect_inheritance_chain_recursive(&edge.target, chain, visited)?;
                }
            }
        }
        
        Ok(())
    }

    /// Check if a class inherits from a specific base class
    pub fn inherits_from(&self, node_id: &NodeId, base_class_name: &str) -> Result<bool> {
        let mut visited = HashSet::new();
        self.inherits_from_recursive(node_id, base_class_name, &mut visited)
    }

    /// Recursively check inheritance
    fn inherits_from_recursive(&self, node_id: &NodeId, base_class_name: &str, visited: &mut HashSet<NodeId>) -> Result<bool> {
        if visited.contains(node_id) {
            return Ok(false);
        }
        
        visited.insert(*node_id);
        
        for edge in self.graph.get_outgoing_edges(node_id) {
            if matches!(edge.kind, EdgeKind::Extends) {
                if let Some(parent_node) = self.graph.get_node(&edge.target) {
                    if parent_node.name == base_class_name {
                        return Ok(true);
                    }
                    
                    if self.inherits_from_recursive(&edge.target, base_class_name, visited)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    /// Check if a class has a specific metaclass
    pub fn has_metaclass(&self, node_id: &NodeId, metaclass_name: &str) -> Result<bool> {
        if let Some(metaclass) = self.get_metaclass(node_id)? {
            Ok(metaclass.class_name == metaclass_name)
        } else {
            Ok(false)
        }
    }

    /// Check if a class uses a specific mixin
    pub fn uses_mixin(&self, node_id: &NodeId, mixin_name: &str) -> Result<bool> {
        let mixins = self.get_mixins(node_id)?;
        Ok(mixins.iter().any(|m| m.class_name == mixin_name))
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

/// Inheritance filter types for advanced symbol search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InheritanceFilter {
    /// Filter by classes that inherit from a specific base class
    InheritsFrom(String),
    /// Filter by classes that have a specific metaclass
    HasMetaclass(String),
    /// Filter by classes that use a specific mixin
    UsesMixin(String),
}

/// Comprehensive inheritance information for a class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceInfo {
    /// The class name
    pub class_name: String,
    /// Direct base classes
    pub base_classes: Vec<InheritanceRelation>,
    /// Direct subclasses
    pub subclasses: Vec<InheritanceRelation>,
    /// Metaclass (if any)
    pub metaclass: Option<InheritanceRelation>,
    /// Mixins used by this class
    pub mixins: Vec<InheritanceRelation>,
    /// Method resolution order
    pub method_resolution_order: Vec<String>,
    /// Dynamic attributes created by metaclasses/decorators
    pub dynamic_attributes: Vec<DynamicAttribute>,
    /// Whether this class is a metaclass
    pub is_metaclass: bool,
    /// Full inheritance chain
    pub inheritance_chain: Vec<String>,
}

impl Default for InheritanceInfo {
    fn default() -> Self {
        Self {
            class_name: String::new(),
            base_classes: Vec::new(),
            subclasses: Vec::new(),
            metaclass: None,
            mixins: Vec::new(),
            method_resolution_order: Vec::new(),
            dynamic_attributes: Vec::new(),
            is_metaclass: false,
            inheritance_chain: Vec::new(),
        }
    }
}

/// Represents an inheritance relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceRelation {
    /// Name of the related class
    pub class_name: String,
    /// Node ID of the related class
    pub node_id: NodeId,
    /// Type of relationship (extends, metaclass, mixin)
    pub relationship_type: String,
    /// File where the class is defined
    pub file: PathBuf,
    /// Location in the file
    pub span: crate::ast::Span,
}

/// Represents a dynamic attribute created by metaclasses or decorators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAttribute {
    /// Name of the attribute
    pub name: String,
    /// What created this attribute (metaclass, decorator, etc.)
    pub created_by: String,
    /// Type of attribute (dynamic, static, etc.)
    pub attribute_type: String,
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

    #[test]
    fn test_symbol_search_regex() {
        let graph = Arc::new(GraphStore::new());
        let query = GraphQuery::new(graph.clone());
        
        let agent_node = create_test_node("Agent", NodeKind::Class, "agent.py");
        let user_agent_node = create_test_node("UserAgent", NodeKind::Class, "user_agent.py");
        let guild_manager_agent_node = create_test_node("GuildManagerAgent", NodeKind::Class, "guild_manager_agent.py");
        let other_node = create_test_node("ProcessAgent", NodeKind::Function, "process.py");
        
        graph.add_node(agent_node.clone());
        graph.add_node(user_agent_node.clone());
        graph.add_node(guild_manager_agent_node.clone());
        graph.add_node(other_node.clone());
        
        // Test exact match using regex
        let results = query.search_symbols("^Agent$", None, None).unwrap();
        assert_eq!(results.len(), 1); // only exact "Agent"
        assert_eq!(results[0].node.name, "Agent");
        
        // Test suffix match
        let results = query.search_symbols("Agent$", None, None).unwrap();
        assert_eq!(results.len(), 4); // Agent, UserAgent, GuildManagerAgent, ProcessAgent
        
        // Test case-sensitive prefix match
        let results = query.search_symbols("^Guild", None, None).unwrap();
        assert_eq!(results.len(), 1); // only GuildManagerAgent
        assert_eq!(results[0].node.name, "GuildManagerAgent");
        
        // Test fallback to substring search with invalid regex
        let results = query.search_symbols("[invalid", None, None).unwrap();
        assert_eq!(results.len(), 0); // no matches for invalid pattern (falls back to substring)
        
        // Test normal substring search still works
        let results = query.search_symbols("Agent", None, None).unwrap();
        assert_eq!(results.len(), 4); // All nodes containing "Agent"
    }
} 