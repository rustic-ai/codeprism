//! Semantic analysis and search capabilities
//! 
//! Provides concept-based code search and high-level understanding
//! of architectural patterns and semantic relationships.

pub mod search;
pub mod concepts;

// Re-export main types
pub use search::{SemanticSearchEngine, SemanticSearchResult, SearchQuery};
pub use concepts::{CodeConcept, ConceptMapper, ConceptRelationship}; 