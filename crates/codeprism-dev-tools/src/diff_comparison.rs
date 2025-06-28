//! AST diff comparison utilities for parser development

use anyhow::Result;
use codeprism_core::ParseResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// AST diff analyzer for comparing parse results
#[derive(Debug, Clone)]
pub struct AstDiff {
    config: DiffConfig,
}

/// Configuration for diff analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    pub ignore_spans: bool,
    pub ignore_node_ids: bool,
    pub compare_edge_order: bool,
    pub max_differences: usize,
    pub similarity_threshold: f64,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            ignore_spans: false,
            ignore_node_ids: true,
            compare_edge_order: false,
            max_differences: 100,
            similarity_threshold: 0.8,
        }
    }
}

/// Type of difference found between ASTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    NodeAdded {
        node_name: String,
        node_type: String,
        location: Option<String>,
    },
    NodeRemoved {
        node_name: String,
        node_type: String,
        location: Option<String>,
    },
    NodeModified {
        node_name: String,
        old_type: String,
        new_type: String,
        changes: Vec<String>,
    },
    EdgeAdded {
        source: String,
        target: String,
        edge_type: String,
    },
    EdgeRemoved {
        source: String,
        target: String,
        edge_type: String,
    },
    EdgeModified {
        source: String,
        target: String,
        old_type: String,
        new_type: String,
    },
    StructuralChange {
        description: String,
        impact: StructuralImpact,
    },
}

/// Impact level of structural changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StructuralImpact {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive diff report
#[derive(Debug, Clone)]
pub struct DiffReport {
    pub differences: Vec<DiffType>,
    pub statistics: DiffStatistics,
    pub similarity_score: f64,
    pub is_significant_change: bool,
    pub summary: String,
}

/// Statistics about the differences found
#[derive(Debug, Clone, Default)]
pub struct DiffStatistics {
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub nodes_modified: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub edges_modified: usize,
    pub total_differences: usize,
    pub similarity_percentage: f64,
}

impl AstDiff {
    /// Create a new AST diff analyzer
    pub fn new() -> Self {
        Self {
            config: DiffConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: DiffConfig) -> Self {
        Self { config }
    }

    /// Compare two parse results and generate a diff report
    pub fn compare(
        &self,
        old_result: &ParseResult,
        new_result: &ParseResult,
        _source: &str,
    ) -> Result<DiffReport> {
        let mut differences = Vec::new();
        let mut statistics = DiffStatistics::default();

        // Compare nodes
        self.compare_nodes(
            &old_result.nodes,
            &new_result.nodes,
            &mut differences,
            &mut statistics,
        )?;

        // Compare edges
        self.compare_edges(
            &old_result.edges,
            &new_result.edges,
            &mut differences,
            &mut statistics,
        )?;

        // Compare tree structures
        self.compare_tree_structures(&old_result.tree, &new_result.tree, &mut differences)?;

        // Calculate similarity score
        let similarity_score = self.calculate_similarity_score(&statistics, old_result, new_result);

        // Determine if this is a significant change
        let is_significant_change = similarity_score < self.config.similarity_threshold;

        // Generate summary
        let summary = self.generate_summary(&statistics, similarity_score);

        statistics.total_differences = differences.len();
        statistics.similarity_percentage = similarity_score * 100.0;

        Ok(DiffReport {
            differences,
            statistics,
            similarity_score,
            is_significant_change,
            summary,
        })
    }

    /// Compare node lists
    fn compare_nodes(
        &self,
        old_nodes: &[codeprism_core::Node],
        new_nodes: &[codeprism_core::Node],
        differences: &mut Vec<DiffType>,
        statistics: &mut DiffStatistics,
    ) -> Result<()> {
        // Create maps for easier comparison
        let old_node_map: HashMap<String, &codeprism_core::Node> = old_nodes
            .iter()
            .map(|n| (self.create_node_key(n), n))
            .collect();

        let new_node_map: HashMap<String, &codeprism_core::Node> = new_nodes
            .iter()
            .map(|n| (self.create_node_key(n), n))
            .collect();

        let old_keys: HashSet<_> = old_node_map.keys().collect();
        let new_keys: HashSet<_> = new_node_map.keys().collect();

        // Find added nodes
        for key in new_keys.difference(&old_keys) {
            if let Some(node) = new_node_map.get(*key) {
                differences.push(DiffType::NodeAdded {
                    node_name: node.name.clone(),
                    node_type: format!("{:?}", node.kind),
                    location: Some(format!("{}:{}", node.span.start_byte, node.span.end_byte)),
                });
                statistics.nodes_added += 1;
            }
        }

        // Find removed nodes
        for key in old_keys.difference(&new_keys) {
            if let Some(node) = old_node_map.get(*key) {
                differences.push(DiffType::NodeRemoved {
                    node_name: node.name.clone(),
                    node_type: format!("{:?}", node.kind),
                    location: Some(format!("{}:{}", node.span.start_byte, node.span.end_byte)),
                });
                statistics.nodes_removed += 1;
            }
        }

        // Find modified nodes (same key but different properties)
        for key in old_keys.intersection(&new_keys) {
            if let (Some(old_node), Some(new_node)) =
                (old_node_map.get(*key), new_node_map.get(*key))
            {
                let changes = self.compare_node_properties(old_node, new_node);
                if !changes.is_empty() {
                    differences.push(DiffType::NodeModified {
                        node_name: old_node.name.clone(),
                        old_type: format!("{:?}", old_node.kind),
                        new_type: format!("{:?}", new_node.kind),
                        changes,
                    });
                    statistics.nodes_modified += 1;
                }
            }
        }

        Ok(())
    }

    /// Compare edge lists
    fn compare_edges(
        &self,
        old_edges: &[codeprism_core::Edge],
        new_edges: &[codeprism_core::Edge],
        differences: &mut Vec<DiffType>,
        statistics: &mut DiffStatistics,
    ) -> Result<()> {
        let old_edge_map: HashMap<String, &codeprism_core::Edge> = old_edges
            .iter()
            .map(|e| (self.create_edge_key(e), e))
            .collect();

        let new_edge_map: HashMap<String, &codeprism_core::Edge> = new_edges
            .iter()
            .map(|e| (self.create_edge_key(e), e))
            .collect();

        let old_keys: HashSet<_> = old_edge_map.keys().collect();
        let new_keys: HashSet<_> = new_edge_map.keys().collect();

        // Find added edges
        for key in new_keys.difference(&old_keys) {
            if let Some(edge) = new_edge_map.get(*key) {
                differences.push(DiffType::EdgeAdded {
                    source: edge.source.to_hex(),
                    target: edge.target.to_hex(),
                    edge_type: format!("{:?}", edge.kind),
                });
                statistics.edges_added += 1;
            }
        }

        // Find removed edges
        for key in old_keys.difference(&new_keys) {
            if let Some(edge) = old_edge_map.get(*key) {
                differences.push(DiffType::EdgeRemoved {
                    source: edge.source.to_hex(),
                    target: edge.target.to_hex(),
                    edge_type: format!("{:?}", edge.kind),
                });
                statistics.edges_removed += 1;
            }
        }

        Ok(())
    }

    /// Compare tree structures
    fn compare_tree_structures(
        &self,
        old_tree: &tree_sitter::Tree,
        new_tree: &tree_sitter::Tree,
        differences: &mut Vec<DiffType>,
    ) -> Result<()> {
        let old_root = old_tree.root_node();
        let new_root = new_tree.root_node();

        // Compare root node types
        if old_root.kind() != new_root.kind() {
            differences.push(DiffType::StructuralChange {
                description: format!(
                    "Root node type changed from '{}' to '{}'",
                    old_root.kind(),
                    new_root.kind()
                ),
                impact: StructuralImpact::Critical,
            });
        }

        // Compare tree depths
        let old_depth = self.calculate_tree_depth(&old_root);
        let new_depth = self.calculate_tree_depth(&new_root);

        if (old_depth as i32 - new_depth as i32).abs() > 2 {
            differences.push(DiffType::StructuralChange {
                description: format!(
                    "Significant tree depth change: {} -> {}",
                    old_depth, new_depth
                ),
                impact: if (old_depth as i32 - new_depth as i32).abs() > 5 {
                    StructuralImpact::High
                } else {
                    StructuralImpact::Medium
                },
            });
        }

        Ok(())
    }

    /// Create a unique key for a node (ignoring IDs if configured)
    fn create_node_key(&self, node: &codeprism_core::Node) -> String {
        if self.config.ignore_node_ids {
            if self.config.ignore_spans {
                format!("{}:{:?}", node.name, node.kind)
            } else {
                format!(
                    "{}:{:?}:{}:{}",
                    node.name, node.kind, node.span.start_byte, node.span.end_byte
                )
            }
        } else {
            node.id.to_hex()
        }
    }

    /// Create a unique key for an edge
    fn create_edge_key(&self, edge: &codeprism_core::Edge) -> String {
        format!(
            "{}->{}:{:?}",
            edge.source.to_hex(),
            edge.target.to_hex(),
            edge.kind
        )
    }

    /// Compare properties of two nodes
    fn compare_node_properties(
        &self,
        old_node: &codeprism_core::Node,
        new_node: &codeprism_core::Node,
    ) -> Vec<String> {
        let mut changes = Vec::new();

        if old_node.kind != new_node.kind {
            changes.push(format!(
                "Type changed: {:?} -> {:?}",
                old_node.kind, new_node.kind
            ));
        }

        if old_node.name != new_node.name {
            changes.push(format!(
                "Name changed: '{}' -> '{}'",
                old_node.name, new_node.name
            ));
        }

        if !self.config.ignore_spans
            && (old_node.span.start_byte != new_node.span.start_byte
                || old_node.span.end_byte != new_node.span.end_byte)
        {
            changes.push(format!(
                "Span changed: {}..{} -> {}..{}",
                old_node.span.start_byte,
                old_node.span.end_byte,
                new_node.span.start_byte,
                new_node.span.end_byte
            ));
        }

        changes
    }

    /// Calculate tree depth
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_tree_depth(&self, node: &tree_sitter::Node) -> usize {
        let mut max_depth = 0;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let child_depth = self.calculate_tree_depth(&child);
                max_depth = max_depth.max(child_depth);
            }
        }
        max_depth + 1
    }

    /// Calculate similarity score between two parse results
    fn calculate_similarity_score(
        &self,
        statistics: &DiffStatistics,
        old_result: &ParseResult,
        new_result: &ParseResult,
    ) -> f64 {
        let total_old_items = old_result.nodes.len() + old_result.edges.len();
        let total_new_items = new_result.nodes.len() + new_result.edges.len();
        let max_items = total_old_items.max(total_new_items) as f64;

        if max_items == 0.0 {
            return 1.0; // Both empty, perfect similarity
        }

        let total_changes = statistics.nodes_added
            + statistics.nodes_removed
            + statistics.nodes_modified
            + statistics.edges_added
            + statistics.edges_removed
            + statistics.edges_modified;

        let similarity = 1.0 - (total_changes as f64 / max_items);
        similarity.max(0.0)
    }

    /// Generate a human-readable summary
    fn generate_summary(&self, statistics: &DiffStatistics, similarity_score: f64) -> String {
        if statistics.total_differences == 0 {
            return "No differences found - ASTs are identical".to_string();
        }

        let mut summary = format!(
            "Found {} differences (similarity: {:.1}%)",
            statistics.total_differences,
            similarity_score * 100.0
        );

        let mut parts = Vec::new();

        if statistics.nodes_added > 0 {
            parts.push(format!("{} nodes added", statistics.nodes_added));
        }
        if statistics.nodes_removed > 0 {
            parts.push(format!("{} nodes removed", statistics.nodes_removed));
        }
        if statistics.nodes_modified > 0 {
            parts.push(format!("{} nodes modified", statistics.nodes_modified));
        }
        if statistics.edges_added > 0 {
            parts.push(format!("{} edges added", statistics.edges_added));
        }
        if statistics.edges_removed > 0 {
            parts.push(format!("{} edges removed", statistics.edges_removed));
        }
        if statistics.edges_modified > 0 {
            parts.push(format!("{} edges modified", statistics.edges_modified));
        }

        if !parts.is_empty() {
            summary.push_str(": ");
            summary.push_str(&parts.join(", "));
        }

        summary
    }
}

impl Default for AstDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffReport {
    /// Format the diff report for display
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("=== AST Diff Report ===\n\n");
        output.push_str(&format!("Summary: {}\n", self.summary));
        output.push_str(&format!(
            "Similarity Score: {:.1}%\n",
            self.similarity_score * 100.0
        ));

        if self.is_significant_change {
            output.push_str("⚠️  Significant changes detected!\n");
        } else {
            output.push_str("✅ Minor changes only\n");
        }

        output.push('\n');

        if !self.differences.is_empty() {
            output.push_str("## Detailed Changes:\n");
            for (i, diff) in self.differences.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, self.format_diff(diff)));
            }
        }

        output.push_str("\n## Statistics:\n");
        output.push_str(&format!("- Nodes added: {}\n", self.statistics.nodes_added));
        output.push_str(&format!(
            "- Nodes removed: {}\n",
            self.statistics.nodes_removed
        ));
        output.push_str(&format!(
            "- Nodes modified: {}\n",
            self.statistics.nodes_modified
        ));
        output.push_str(&format!("- Edges added: {}\n", self.statistics.edges_added));
        output.push_str(&format!(
            "- Edges removed: {}\n",
            self.statistics.edges_removed
        ));
        output.push_str(&format!(
            "- Edges modified: {}\n",
            self.statistics.edges_modified
        ));
        output.push_str(&format!(
            "- Total differences: {}\n",
            self.statistics.total_differences
        ));

        output
    }

    /// Format a single difference
    fn format_diff(&self, diff: &DiffType) -> String {
        match diff {
            DiffType::NodeAdded {
                node_name,
                node_type,
                location,
            } => {
                format!(
                    "Added node '{}' ({}) at {}",
                    node_name,
                    node_type,
                    location.as_deref().unwrap_or("unknown")
                )
            }
            DiffType::NodeRemoved {
                node_name,
                node_type,
                location,
            } => {
                format!(
                    "Removed node '{}' ({}) from {}",
                    node_name,
                    node_type,
                    location.as_deref().unwrap_or("unknown")
                )
            }
            DiffType::NodeModified {
                node_name,
                old_type,
                new_type,
                changes,
            } => {
                format!(
                    "Modified node '{}' ({} -> {}): {}",
                    node_name,
                    old_type,
                    new_type,
                    changes.join(", ")
                )
            }
            DiffType::EdgeAdded {
                source,
                target,
                edge_type,
            } => {
                format!("Added edge {} -> {} ({})", source, target, edge_type)
            }
            DiffType::EdgeRemoved {
                source,
                target,
                edge_type,
            } => {
                format!("Removed edge {} -> {} ({})", source, target, edge_type)
            }
            DiffType::EdgeModified {
                source,
                target,
                old_type,
                new_type,
            } => {
                format!(
                    "Modified edge {} -> {} ({} -> {})",
                    source, target, old_type, new_type
                )
            }
            DiffType::StructuralChange {
                description,
                impact,
            } => {
                format!("Structural change ({:?}): {}", impact, description)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeprism_core::{NodeKind, Span};
    use std::path::PathBuf;

    fn create_test_node(_id: u64, name: &str, kind: NodeKind) -> codeprism_core::Node {
        let path = PathBuf::from("test.rs");
        let span = Span::new(0, 10, 1, 1, 1, 10);
        let repo_id = "test_repo";

        codeprism_core::Node {
            id: codeprism_core::NodeId::new(repo_id, &path, &span, &kind),
            kind,
            name: name.to_string(),
            file: path,
            span,
            lang: codeprism_core::Language::Rust,
            metadata: Default::default(),
            signature: Default::default(),
        }
    }

    #[test]
    fn test_ast_diff_creation() {
        let diff = AstDiff::new();
        assert!(diff.config.ignore_node_ids);
        assert!(!diff.config.ignore_spans);
    }

    #[test]
    fn test_create_node_key() {
        let diff = AstDiff::new();
        let node = create_test_node(1, "test", NodeKind::Function);
        let key = diff.create_node_key(&node);
        assert!(key.contains("test"));
        assert!(key.contains("Function"));
    }

    #[test]
    fn test_node_key_ignoring_ids() {
        let config = DiffConfig {
            ignore_node_ids: true,
            ..Default::default()
        };
        let diff = AstDiff::with_config(config);

        let node1 = create_test_node(1, "test", NodeKind::Function);
        let node2 = create_test_node(2, "test", NodeKind::Function);

        let key1 = diff.create_node_key(&node1);
        let key2 = diff.create_node_key(&node2);

        assert_eq!(key1, key2); // Should be same since IDs are ignored
    }
}
