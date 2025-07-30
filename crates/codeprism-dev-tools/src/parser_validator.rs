//! Parser validation utilities for CodePrism development

use anyhow::Result;
use codeprism_core::{Edge, Node, ParseResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Parser validator for comprehensive validation checks
#[derive(Debug, Clone)]
pub struct ParserValidator {
    config: ValidationConfig,
}

/// Configuration for parser validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub check_span_overlaps: bool,
    pub check_edge_consistency: bool,
    pub check_unreachable_nodes: bool,
    pub check_text_coverage: bool,
    pub check_duplicate_nodes: bool,
    pub min_span_size: usize,
    pub max_parsing_time_ms: u64,
    pub check_syntax_tree_structure: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_span_overlaps: true,
            check_edge_consistency: true,
            check_unreachable_nodes: true,
            check_text_coverage: true,
            check_duplicate_nodes: true,
            min_span_size: 0,
            max_parsing_time_ms: 5000,
            check_syntax_tree_structure: true,
        }
    }
}

/// Validation report containing all validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub statistics: ValidationStatistics,
    pub is_valid: bool,
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    SpanOverlap {
        node1_id: String,
        node2_id: String,
        overlap_start: usize,
        overlap_end: usize,
    },
    InvalidEdge {
        edge_id: String,
        source_id: String,
        target_id: String,
        reason: String,
    },
    UnreachableNode {
        node_id: String,
        node_type: String,
    },
    TextCoverageGap {
        start_byte: usize,
        end_byte: usize,
        gap_size: usize,
    },
    InvalidSpan {
        node_id: String,
        start_byte: usize,
        end_byte: usize,
        reason: String,
    },
}

/// Validation warning types
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    SmallSpan {
        node_id: String,
        span_size: usize,
        min_expected: usize,
    },
    DeepNesting {
        node_id: String,
        depth: usize,
        max_recommended: usize,
    },
}

/// Statistics collected during validation
#[derive(Debug, Clone, Default)]
pub struct ValidationStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_depth: usize,
    pub text_coverage_percentage: f64,
    pub validation_time_ms: u64,
    pub node_type_distribution: HashMap<String, usize>,
}

impl ParserValidator {
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_complete(
        &self,
        parse_result: &ParseResult,
        _source: &str,
    ) -> Result<ValidationReport> {
        let start_time = std::time::Instant::now();

        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut statistics = ValidationStatistics {
            total_nodes: parse_result.nodes.len(),
            total_edges: parse_result.edges.len(),
            ..Default::default()
        };

        // Basic validation checks (simplified for initial implementation)
        if self.config.check_span_overlaps {
            self.validate_span_overlaps(&parse_result.nodes, &mut errors)?;
        }

        if self.config.check_edge_consistency {
            self.validate_edge_consistency(&parse_result.nodes, &parse_result.edges, &mut errors)?;
        }

        statistics.validation_time_ms = start_time.elapsed().as_millis() as u64;
        let is_valid = errors.is_empty();

        Ok(ValidationReport {
            errors,
            warnings,
            statistics,
            is_valid,
        })
    }

    fn validate_span_overlaps(
        &self,
        nodes: &[Node],
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        for (i, node1) in nodes.iter().enumerate() {
            for (_j, node2) in nodes.iter().enumerate().skip(i + 1) {
                let start1 = node1.span.start_byte;
                let end1 = node1.span.end_byte;
                let start2 = node2.span.start_byte;
                let end2 = node2.span.end_byte;

                let overlap_start = start1.max(start2);
                let overlap_end = end1.min(end2);

                if overlap_start < overlap_end {
                    let is_containment =
                        (start1 <= start2 && end1 >= end2) || (start2 <= start1 && end2 >= end1);

                    if !is_containment {
                        errors.push(ValidationError::SpanOverlap {
                            node1_id: node1.id.to_hex(),
                            node2_id: node2.id.to_hex(),
                            overlap_start,
                            overlap_end,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_edge_consistency(
        &self,
        nodes: &[Node],
        edges: &[Edge],
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        let node_ids: HashSet<_> = nodes.iter().map(|n| &n.id).collect();

        for edge in edges {
            if !node_ids.contains(&edge.source) {
                errors.push(ValidationError::InvalidEdge {
                    edge_id: format!("{}->{}", edge.source.to_hex(), edge.target.to_hex()),
                    source_id: edge.source.to_hex(),
                    target_id: edge.target.to_hex(),
                    reason: "Source node does not exist".to_string(),
                });
            }

            if !node_ids.contains(&edge.target) {
                errors.push(ValidationError::InvalidEdge {
                    edge_id: format!("{}->{}", edge.source.to_hex(), edge.target.to_hex()),
                    source_id: edge.source.to_hex(),
                    target_id: edge.target.to_hex(),
                    reason: "Target node does not exist".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl Default for ParserValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }

    pub fn statistics(&self) -> &ValidationStatistics {
        &self.statistics
    }

    pub fn summary(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Parser Validation Report ===\n\n");

        if self.is_valid {
            output.push_str("✅ Validation PASSED\n");
        } else {
            output.push_str("❌ Validation FAILED\n");
        }

        output.push_str(&format!("Errors: {}\n", self.errors.len()));
        output.push_str(&format!("Warnings: {}\n", self.warnings.len()));
        output.push_str(&format!("Total nodes: {}\n", self.statistics.total_nodes));
        output.push_str(&format!("Total edges: {}\n", self.statistics.total_edges));

        output
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::SpanOverlap {
                node1_id,
                node2_id,
                overlap_start,
                overlap_end,
            } => {
                write!(
                    f,
                    "Span overlap between nodes {node1_id} and {node2_id} at bytes {overlap_start}..{overlap_end}"
                )
            }
            ValidationError::InvalidEdge {
                edge_id, reason, ..
            } => {
                write!(f, "Invalid edge {edge_id}: {reason}")
            }
            ValidationError::UnreachableNode { node_id, node_type } => {
                write!(f, "Unreachable node {node_id} (type: {node_type})")
            }
            ValidationError::TextCoverageGap {
                start_byte,
                end_byte,
                gap_size,
            } => {
                write!(
                    f,
                    "Text coverage gap at bytes {start_byte}..{end_byte} (size: {gap_size})"
                )
            }
            ValidationError::InvalidSpan {
                node_id,
                start_byte,
                end_byte,
                reason,
            } => {
                write!(
                    f,
                    "Invalid span for node {node_id} ({start_byte}..{end_byte}): {reason}"
                )
            }
        }
    }
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationWarning::SmallSpan {
                node_id,
                span_size,
                min_expected,
            } => {
                write!(
                    f,
                    "Small span for node {node_id} (size: {span_size}, expected: ≥{min_expected})"
                )
            }
            ValidationWarning::DeepNesting {
                node_id,
                depth,
                max_recommended,
            } => {
                write!(
                    f,
                    "Deep nesting in node {node_id} (depth: {depth}, recommended: ≤{max_recommended})"
                )
            }
        }
    }
}
