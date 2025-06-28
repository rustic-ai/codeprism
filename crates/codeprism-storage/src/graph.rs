//! Graph serialization types for storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Serializable representation of a code graph for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGraph {
    pub repo_id: String,
    pub nodes: Vec<SerializableNode>,
    pub edges: Vec<SerializableEdge>,
    pub metadata: GraphMetadata,
}

/// Serializable representation of a graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file: PathBuf,
    pub span: SerializableSpan,
    pub attributes: HashMap<String, String>,
}

/// Serializable representation of a graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub attributes: HashMap<String, String>,
}

/// Serializable representation of a source code span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

/// Graph metadata for version tracking and incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub repo_id: String,
    pub last_updated: SystemTime,
    pub version: u64,
    pub file_hashes: HashMap<PathBuf, String>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub schema_version: String,
}

impl SerializableGraph {
    /// Create a new empty serializable graph
    pub fn new(repo_id: String) -> Self {
        Self {
            repo_id: repo_id.clone(),
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata {
                repo_id,
                last_updated: SystemTime::now(),
                version: 1,
                file_hashes: HashMap::new(),
                total_nodes: 0,
                total_edges: 0,
                schema_version: "1.0".to_string(),
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: SerializableNode) {
        self.nodes.push(node);
        self.metadata.total_nodes = self.nodes.len();
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: SerializableEdge) {
        self.edges.push(edge);
        self.metadata.total_edges = self.edges.len();
    }

    /// Update metadata
    pub fn update_metadata(&mut self) {
        self.metadata.last_updated = SystemTime::now();
        self.metadata.total_nodes = self.nodes.len();
        self.metadata.total_edges = self.edges.len();
        self.metadata.version += 1;
    }
}

impl SerializableNode {
    /// Create a new serializable node
    pub fn new(
        id: String,
        name: String,
        kind: String,
        file: PathBuf,
        span: SerializableSpan,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            file,
            span,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the node
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
}

impl SerializableEdge {
    /// Create a new serializable edge
    pub fn new(source: String, target: String, kind: String) -> Self {
        Self {
            source,
            target,
            kind,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the edge
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
}
