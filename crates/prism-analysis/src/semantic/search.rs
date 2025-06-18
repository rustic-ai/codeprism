//! Semantic search engine for concept-based code discovery

use anyhow::Result;
use prism_core::{GraphStore, GraphQuery, Node, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The concept or pattern to search for
    pub concept: String,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(concept: String) -> Self {
        Self {
            concept,
            limit: Some(20),
        }
    }
}

/// Search result with semantic understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    /// Matching nodes
    pub nodes: Vec<SemanticMatch>,
    /// Search statistics
    pub search_stats: SearchStats,
}

/// A semantically matched node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMatch {
    /// The matched node
    pub node: Node,
    /// Semantic relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Matched concepts
    pub matched_concepts: Vec<String>,
    /// Context explanation
    pub context: String,
}

/// Search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// Total nodes examined
    pub nodes_examined: usize,
    /// Search time in milliseconds
    pub search_time_ms: u64,
}

/// Semantic search engine
pub struct SemanticSearchEngine {
    /// Known concept patterns
    concept_patterns: HashMap<String, Vec<String>>,
}

impl SemanticSearchEngine {
    /// Create a new semantic search engine
    pub fn new() -> Self {
        let mut concept_patterns = HashMap::new();
        
        // Authentication patterns
        concept_patterns.insert("authentication".to_string(), vec![
            "login".to_string(),
            "auth".to_string(),
            "authenticate".to_string(),
            "credential".to_string(),
            "token".to_string(),
            "session".to_string(),
        ]);
        
        // Database patterns
        concept_patterns.insert("database".to_string(), vec![
            "query".to_string(),
            "sql".to_string(),
            "database".to_string(),
            "connection".to_string(),
            "repository".to_string(),
            "model".to_string(),
        ]);

        Self { concept_patterns }
    }

    /// Perform semantic search
    pub fn search(
        &self,
        query: &SearchQuery,
        graph_store: &GraphStore,
        _graph_query: &GraphQuery,
    ) -> Result<SemanticSearchResult> {
        let start_time = std::time::Instant::now();
        
        // Extract concepts from query
        let concepts = self.extract_concepts(&query.concept);
        
        // Find matching nodes
        let matches = self.find_semantic_matches(&concepts, graph_store)?;
        
        let search_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Apply limit
        let limited_matches = if let Some(limit) = query.limit {
            matches.into_iter().take(limit).collect()
        } else {
            matches
        };
        
        let search_stats = SearchStats {
            nodes_examined: graph_store.get_stats().total_nodes,
            search_time_ms,
        };

        Ok(SemanticSearchResult {
            nodes: limited_matches,
            search_stats,
        })
    }

    /// Extract concepts from query string
    fn extract_concepts(&self, query: &str) -> Vec<String> {
        let mut concepts = Vec::new();
        let query_lower = query.to_lowercase();
        
        // Check for direct concept matches
        for (concept, patterns) in &self.concept_patterns {
            if query_lower.contains(concept) {
                concepts.push(concept.clone());
                continue;
            }
            
            // Check for pattern matches
            for pattern in patterns {
                if query_lower.contains(pattern) {
                    concepts.push(concept.clone());
                    break;
                }
            }
        }
        
        // Always include the original query as a concept
        if !concepts.contains(&query_lower) {
            concepts.push(query_lower);
        }
        
        concepts
    }

    /// Find nodes that semantically match the concepts
    fn find_semantic_matches(
        &self,
        concepts: &[String],
        graph_store: &GraphStore,
    ) -> Result<Vec<SemanticMatch>> {
        let mut matches = Vec::new();
        
        // Search through all symbols to get all nodes
        for symbol_entry in graph_store.iter_symbol_index() {
            for node_id in symbol_entry.1 {
                if let Some(node) = graph_store.get_node(&node_id) {
                    let relevance_score = self.calculate_relevance_score(&node, concepts);
                    
                    if relevance_score > 0.1 { // Minimum relevance threshold
                        let matched_concepts = self.get_matched_concepts(&node, concepts);
                        let context = self.generate_context_explanation(&node, &matched_concepts);
                        
                        matches.push(SemanticMatch {
                            node,
                            relevance_score,
                            matched_concepts,
                            context,
                        });
                    }
                }
            }
        }
        
        // Sort by relevance score (descending)
        matches.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(matches)
    }

    /// Calculate relevance score for a node
    fn calculate_relevance_score(&self, node: &Node, concepts: &[String]) -> f64 {
        let mut score = 0.0;
        let node_text = format!("{} {}", node.name, format!("{:?}", node.kind)).to_lowercase();
        
        for concept in concepts {
            // Direct name match
            if node.name.to_lowercase().contains(concept) {
                score += 0.8;
            }
            
            // Pattern-based matching
            if let Some(patterns) = self.concept_patterns.get(concept) {
                for pattern in patterns {
                    if node_text.contains(pattern) {
                        score += 0.5;
                    }
                }
            }
        }
        
        // Normalize score
        (score / concepts.len() as f64).min(1.0)
    }

    /// Get concepts that matched for a node
    fn get_matched_concepts(&self, node: &Node, concepts: &[String]) -> Vec<String> {
        let mut matched = Vec::new();
        let node_text = format!("{} {}", node.name, format!("{:?}", node.kind)).to_lowercase();
        
        for concept in concepts {
            if node.name.to_lowercase().contains(concept) {
                matched.push(concept.clone());
                continue;
            }
            
            if let Some(patterns) = self.concept_patterns.get(concept) {
                for pattern in patterns {
                    if node_text.contains(pattern) {
                        matched.push(concept.clone());
                        break;
                    }
                }
            }
        }
        
        matched
    }

    /// Generate context explanation
    fn generate_context_explanation(&self, node: &Node, matched_concepts: &[String]) -> String {
        let concept_text = if matched_concepts.is_empty() {
            "general purpose".to_string()
        } else {
            matched_concepts.join(", ")
        };
        
        format!(
            "{} '{}' appears to be related to {} based on its name and type",
            format!("{:?}", node.kind),
            node.name,
            concept_text
        )
    }
}

impl Default for SemanticSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
