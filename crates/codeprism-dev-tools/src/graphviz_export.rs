//! GraphViz export utilities for AST visualization

use anyhow::Result;
use codeprism_core::{Edge, EdgeKind, Node, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// GraphViz exporter for generating DOT format graphs
#[derive(Debug, Clone)]
pub struct GraphVizExporter {
    config: GraphVizConfig,
}

/// Configuration for GraphViz export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVizConfig {
    pub graph_name: String,
    pub graph_type: GraphType,
    pub node_options: NodeStyle,
    pub edge_options: EdgeStyle,
    pub layout_engine: LayoutEngine,
    pub include_node_labels: bool,
    pub include_edge_labels: bool,
    pub max_label_length: usize,
    pub group_by_type: bool,
}

impl Default for GraphVizConfig {
    fn default() -> Self {
        Self {
            graph_name: "ast_graph".to_string(),
            graph_type: GraphType::Directed,
            node_options: NodeStyle::default(),
            edge_options: EdgeStyle::default(),
            layout_engine: LayoutEngine::Dot,
            include_node_labels: true,
            include_edge_labels: true,
            max_label_length: 20,
            group_by_type: false,
        }
    }
}

/// Graph type for GraphViz
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GraphType {
    Directed,
    Undirected,
}

/// Layout engine options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LayoutEngine {
    Dot,
    Neato,
    Circo,
    Fdp,
    Sfdp,
    Twopi,
}

/// Node styling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    pub shape: String,
    pub color: String,
    pub fillcolor: String,
    pub style: String,
    pub fontname: String,
    pub fontsize: u32,
    pub node_type_colors: HashMap<String, String>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        let mut node_type_colors = HashMap::new();
        node_type_colors.insert("Function".to_string(), "#e1f5fe".to_string());
        node_type_colors.insert("Class".to_string(), "#f3e5f5".to_string());
        node_type_colors.insert("Variable".to_string(), "#fff3e0".to_string());
        node_type_colors.insert("Import".to_string(), "#e8f5e8".to_string());

        Self {
            shape: "box".to_string(),
            color: "black".to_string(),
            fillcolor: "#f5f5f5".to_string(),
            style: "filled".to_string(),
            fontname: "Arial".to_string(),
            fontsize: 10,
            node_type_colors,
        }
    }
}

/// Edge styling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub color: String,
    pub style: String,
    pub arrowhead: String,
    pub fontname: String,
    pub fontsize: u32,
    pub edge_type_colors: HashMap<String, String>,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        let mut edge_type_colors = HashMap::new();
        edge_type_colors.insert("Calls".to_string(), "#2196f3".to_string());
        edge_type_colors.insert("Imports".to_string(), "#4caf50".to_string());
        edge_type_colors.insert("Reads".to_string(), "#ff9800".to_string());
        edge_type_colors.insert("Writes".to_string(), "#f44336".to_string());

        Self {
            color: "gray".to_string(),
            style: "solid".to_string(),
            arrowhead: "normal".to_string(),
            fontname: "Arial".to_string(),
            fontsize: 8,
            edge_type_colors,
        }
    }
}

/// GraphViz options for specific exports
#[derive(Debug, Clone, Default)]
pub struct GraphVizOptions {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub highlight_nodes: Vec<String>,
    pub highlight_edges: Vec<String>,
    pub filter_node_types: Option<Vec<NodeKind>>,
    pub filter_edge_types: Option<Vec<EdgeKind>>,
    pub cluster_by_file: bool,
    pub show_spans: bool,
}

impl GraphVizExporter {
    /// Create a new GraphViz exporter with default configuration
    pub fn new() -> Self {
        Self {
            config: GraphVizConfig::default(),
        }
    }

    /// Create a GraphViz exporter with custom configuration
    pub fn with_config(config: GraphVizConfig) -> Self {
        Self { config }
    }

    /// Export nodes and edges to GraphViz DOT format
    pub fn export_nodes_and_edges(&self, nodes: &[Node], edges: &[Edge]) -> Result<String> {
        self.export_with_options(nodes, edges, &GraphVizOptions::default())
    }

    /// Export with custom options
    pub fn export_with_options(
        &self,
        nodes: &[Node],
        edges: &[Edge],
        options: &GraphVizOptions,
    ) -> Result<String> {
        let mut dot = String::new();

        // Graph header
        let graph_keyword = match self.config.graph_type {
            GraphType::Directed => "digraph",
            GraphType::Undirected => "graph",
        };

        dot.push_str(&format!(
            "{} {} {{\n",
            graph_keyword, self.config.graph_name
        ));

        // Graph attributes
        self.write_graph_attributes(&mut dot, options);

        // Filter nodes if requested
        let filtered_nodes: Vec<_> = if let Some(ref filter_types) = options.filter_node_types {
            nodes
                .iter()
                .filter(|n| filter_types.contains(&n.kind))
                .collect()
        } else {
            nodes.iter().collect()
        };

        // Group nodes by file if clustering is enabled
        if options.cluster_by_file {
            self.write_clustered_nodes(&mut dot, &filtered_nodes, options)?;
        } else {
            self.write_nodes(&mut dot, &filtered_nodes, options)?;
        }

        // Filter and write edges
        let filtered_edges: Vec<_> = if let Some(ref filter_types) = options.filter_edge_types {
            edges
                .iter()
                .filter(|e| filter_types.contains(&e.kind))
                .collect()
        } else {
            edges.iter().collect()
        };

        self.write_edges(&mut dot, &filtered_edges, &filtered_nodes, options)?;

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Write graph-level attributes
    fn write_graph_attributes(&self, dot: &mut String, options: &GraphVizOptions) {
        dot.push_str("  // Graph attributes\n");
        dot.push_str(&format!("  layout=\"{:?}\";\n", self.config.layout_engine).to_lowercase());
        dot.push_str("  rankdir=\"TB\";\n");
        dot.push_str("  splines=\"ortho\";\n");
        dot.push_str("  nodesep=\"0.5\";\n");
        dot.push_str("  ranksep=\"1.0\";\n");

        // Add title if provided
        if let Some(ref title) = options.title {
            dot.push_str(&format!("  label=\"{}\";\n", self.escape_label(title)));
            dot.push_str("  fontsize=\"16\";\n");
            dot.push_str("  fontname=\"Arial Bold\";\n");
        }

        // Default node and edge attributes
        dot.push_str("  // Default node attributes\n");
        dot.push_str(&format!("  node [shape=\"{}\", style=\"{}\", fillcolor=\"{}\", color=\"{}\", fontname=\"{}\", fontsize=\"{}\"];\n",
            self.config.node_options.shape,
            self.config.node_options.style,
            self.config.node_options.fillcolor,
            self.config.node_options.color,
            self.config.node_options.fontname,
            self.config.node_options.fontsize
        ));

        dot.push_str("  // Default edge attributes\n");
        dot.push_str(&format!("  edge [color=\"{}\", style=\"{}\", arrowhead=\"{}\", fontname=\"{}\", fontsize=\"{}\"];\n",
            self.config.edge_options.color,
            self.config.edge_options.style,
            self.config.edge_options.arrowhead,
            self.config.edge_options.fontname,
            self.config.edge_options.fontsize
        ));

        dot.push('\n');
    }

    /// Write nodes to DOT format
    fn write_nodes(
        &self,
        dot: &mut String,
        nodes: &[&Node],
        options: &GraphVizOptions,
    ) -> Result<()> {
        dot.push_str("  // Nodes\n");

        for node in nodes {
            let node_id = self.sanitize_id(&node.id.to_hex());
            let mut attributes = Vec::new();

            // Node label
            if self.config.include_node_labels {
                let label = self.create_node_label(node, options);
                attributes.push(format!("label=\"{}\"", self.escape_label(&label)));
            }

            // Node color based on type
            let node_type_str = format!("{:?}", node.kind);
            if let Some(color) = self
                .config
                .node_options
                .node_type_colors
                .get(&node_type_str)
            {
                attributes.push(format!("fillcolor=\"{}\"", color));
            }

            // Highlight if requested
            if options.highlight_nodes.contains(&node.id.to_hex()) {
                attributes.push("penwidth=\"3\"".to_string());
                attributes.push("color=\"red\"".to_string());
            }

            // Write node
            if attributes.is_empty() {
                dot.push_str(&format!("  {};\n", node_id));
            } else {
                dot.push_str(&format!("  {} [{}];\n", node_id, attributes.join(", ")));
            }
        }

        dot.push('\n');
        Ok(())
    }

    /// Write nodes grouped by file (clusters)
    fn write_clustered_nodes(
        &self,
        dot: &mut String,
        nodes: &[&Node],
        options: &GraphVizOptions,
    ) -> Result<()> {
        let mut file_groups: HashMap<_, Vec<_>> = HashMap::new();

        for node in nodes {
            let file_path = node.file.to_string_lossy();
            file_groups
                .entry(file_path.to_string())
                .or_default()
                .push(*node);
        }

        dot.push_str("  // Clustered nodes by file\n");

        for (i, (file_path, file_nodes)) in file_groups.iter().enumerate() {
            dot.push_str(&format!("  subgraph cluster_{} {{\n", i));
            dot.push_str(&format!(
                "    label=\"{}\";\n",
                self.escape_label(file_path)
            ));
            dot.push_str("    style=\"filled\";\n");
            dot.push_str("    fillcolor=\"#f0f0f0\";\n");
            dot.push_str("    color=\"gray\";\n");

            for node in file_nodes {
                let node_id = self.sanitize_id(&node.id.to_hex());
                let label = if self.config.include_node_labels {
                    self.create_node_label(node, options)
                } else {
                    node.name.clone()
                };

                dot.push_str(&format!(
                    "    {} [label=\"{}\"];\n",
                    node_id,
                    self.escape_label(&label)
                ));
            }

            dot.push_str("  }\n");
        }

        dot.push('\n');
        Ok(())
    }

    /// Write edges to DOT format
    fn write_edges(
        &self,
        dot: &mut String,
        edges: &[&Edge],
        nodes: &[&Node],
        options: &GraphVizOptions,
    ) -> Result<()> {
        let node_ids: std::collections::HashSet<_> = nodes.iter().map(|n| &n.id).collect();

        dot.push_str("  // Edges\n");

        let edge_connector = match self.config.graph_type {
            GraphType::Directed => "->",
            GraphType::Undirected => "--",
        };

        for edge in edges {
            // Only include edges between filtered nodes
            if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                continue;
            }

            let source_id = self.sanitize_id(&edge.source.to_hex());
            let target_id = self.sanitize_id(&edge.target.to_hex());

            let mut attributes = Vec::new();

            // Edge label
            if self.config.include_edge_labels {
                let label = format!("{:?}", edge.kind);
                attributes.push(format!("label=\"{}\"", self.escape_label(&label)));
            }

            // Edge color based on type
            let edge_type_str = format!("{:?}", edge.kind);
            if let Some(color) = self
                .config
                .edge_options
                .edge_type_colors
                .get(&edge_type_str)
            {
                attributes.push(format!("color=\"{}\"", color));
            }

            // Highlight if requested
            let edge_id = format!("{}->{}", edge.source.to_hex(), edge.target.to_hex());
            if options.highlight_edges.contains(&edge_id) {
                attributes.push("penwidth=\"3\"".to_string());
                attributes.push("color=\"red\"".to_string());
            }

            // Write edge
            if attributes.is_empty() {
                dot.push_str(&format!(
                    "  {} {} {};\n",
                    source_id, edge_connector, target_id
                ));
            } else {
                dot.push_str(&format!(
                    "  {} {} {} [{}];\n",
                    source_id,
                    edge_connector,
                    target_id,
                    attributes.join(", ")
                ));
            }
        }

        dot.push('\n');
        Ok(())
    }

    /// Create a label for a node
    fn create_node_label(&self, node: &Node, options: &GraphVizOptions) -> String {
        let mut label = node.name.clone();

        if label.len() > self.config.max_label_length {
            label.truncate(self.config.max_label_length - 3);
            label.push_str("...");
        }

        // Add type information
        if self.config.group_by_type {
            label = format!("{}\n({:?})", label, node.kind);
        }

        // Add span information if requested
        if options.show_spans {
            label = format!(
                "{}\n[{}..{}]",
                label, node.span.start_byte, node.span.end_byte
            );
        }

        label
    }

    /// Sanitize an ID for GraphViz
    fn sanitize_id(&self, id: &str) -> String {
        format!("node_{}", id.replace('-', "_"))
    }

    /// Escape a label for GraphViz
    fn escape_label(&self, label: &str) -> String {
        label
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Export tree-sitter syntax tree to GraphViz
    pub fn export_syntax_tree(&self, tree: &tree_sitter::Tree, source: &str) -> Result<String> {
        let root_node = tree.root_node();
        let mut dot = String::new();

        dot.push_str(&format!("digraph {} {{\n", self.config.graph_name));
        dot.push_str("  rankdir=\"TB\";\n");
        dot.push_str("  node [shape=\"box\", style=\"filled\", fillcolor=\"lightblue\"];\n");

        self.export_syntax_node_recursive(&mut dot, &root_node, source, 0)?;

        dot.push_str("}\n");
        Ok(dot)
    }

    /// Recursively export syntax tree nodes
    fn export_syntax_node_recursive(
        &self,
        dot: &mut String,
        node: &tree_sitter::Node,
        source: &str,
        depth: usize,
    ) -> Result<()> {
        let node_id = format!("syntax_{}_{}", depth, node.start_byte());
        let mut label = node.kind().to_string();

        // Add text content for leaf nodes
        if node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                if text.len() <= 20 {
                    label = format!("{}\\n\"{}\"", label, self.escape_label(text));
                } else {
                    label = format!("{}\\n\"{}...\"", label, self.escape_label(&text[..17]));
                }
            }
        }

        dot.push_str(&format!("  {} [label=\"{}\"];\n", node_id, label));

        // Add edges to children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let child_id = format!("syntax_{}_{}", depth + 1, child.start_byte());
                dot.push_str(&format!("  {} -> {};\n", node_id, child_id));
                self.export_syntax_node_recursive(dot, &child, source, depth + 1)?;
            }
        }

        Ok(())
    }
}

impl Default for GraphVizExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LayoutEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutEngine::Dot => write!(f, "dot"),
            LayoutEngine::Neato => write!(f, "neato"),
            LayoutEngine::Circo => write!(f, "circo"),
            LayoutEngine::Fdp => write!(f, "fdp"),
            LayoutEngine::Sfdp => write!(f, "sfdp"),
            LayoutEngine::Twopi => write!(f, "twopi"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeprism_core::{EdgeKind, Language, NodeKind, Span};
    use std::path::PathBuf;

    fn create_test_node(_id: u64, name: &str, kind: NodeKind) -> Node {
        let path = PathBuf::from("test.rs");
        let span = Span::new(0, 10, 1, 1, 1, 10);
        let repo_id = "test_repo";

        Node {
            id: codeprism_core::NodeId::new(repo_id, &path, &span, &kind),
            kind,
            name: name.to_string(),
            file: path,
            span,
            lang: Language::Rust,
            metadata: Default::default(),
            signature: Default::default(),
        }
    }

    fn create_test_edge(source_id: &str, target_id: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: codeprism_core::NodeId::from_hex(source_id).unwrap(),
            target: codeprism_core::NodeId::from_hex(target_id).unwrap(),
            kind,
        }
    }

    #[test]
    fn test_graphviz_exporter_creation() {
        let exporter = GraphVizExporter::new();
        assert_eq!(exporter.config.graph_name, "ast_graph");
        assert!(matches!(exporter.config.graph_type, GraphType::Directed));
    }

    #[test]
    fn test_sanitize_id() {
        let exporter = GraphVizExporter::new();
        let sanitized = exporter.sanitize_id("abc-def-123");
        assert_eq!(sanitized, "node_abc_def_123");
    }

    #[test]
    fn test_escape_label() {
        let exporter = GraphVizExporter::new();
        let escaped = exporter.escape_label("test\n\"quoted\"");
        assert_eq!(escaped, "test\\n\\\"quoted\\\"");
    }

    #[test]
    fn test_export_simple_graph() {
        let exporter = GraphVizExporter::new();
        let nodes = vec![
            create_test_node(1, "main", NodeKind::Function),
            create_test_node(2, "helper", NodeKind::Function),
        ];

        let main_id = nodes[0].id.to_hex();
        let helper_id = nodes[1].id.to_hex();

        let edges = vec![create_test_edge(&main_id, &helper_id, EdgeKind::Calls)];

        let dot = exporter.export_nodes_and_edges(&nodes, &edges).unwrap();
        assert!(dot.contains("digraph ast_graph"));
        assert!(dot.contains("node_"));
        assert!(dot.contains("->"));
    }
}
