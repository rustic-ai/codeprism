//! Semantic analysis and search capabilities
//!
//! Provides concept-based code search and high-level understanding
//! of architectural patterns and semantic relationships.

pub mod concepts;
pub mod search;

// Re-export main types
pub use concepts::{CodeConcept, ConceptMapper, ConceptRelationship};
pub use search::{SearchQuery, SemanticSearchEngine, SemanticSearchResult};
