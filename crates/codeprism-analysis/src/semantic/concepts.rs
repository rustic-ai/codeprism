//! Code concept mapping and understanding
//!
//! Provides mapping from high-level concepts to code elements
//! and understanding of architectural patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A high-level code concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeConcept {
    /// Concept name
    pub name: String,
    /// Concept description
    pub description: String,
    /// Associated keywords
    pub keywords: Vec<String>,
    /// Concept category
    pub category: ConceptCategory,
}

/// Categories of code concepts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConceptCategory {
    /// Architectural patterns
    Architecture,
    /// Design patterns
    DesignPattern,
    /// Data handling
    DataProcessing,
    /// Security-related
    Security,
    /// User interface
    UserInterface,
    /// System integration
    Integration,
    /// Performance-related
    Performance,
}

/// Relationship between concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelationship {
    /// Source concept
    pub from: String,
    /// Target concept
    pub to: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship strength (0.0 to 1.0)
    pub strength: f64,
}

/// Types of concept relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// One concept is part of another
    PartOf,
    /// Concepts are similar
    Similar,
    /// One concept depends on another
    DependsOn,
    /// Concepts are commonly used together
    ComplementaryTo,
    /// One concept is an implementation of another
    ImplementationOf,
}

/// Maps text and code to high-level concepts
pub struct ConceptMapper {
    /// Known concepts
    concepts: HashMap<String, CodeConcept>,
    /// Concept relationships
    relationships: Vec<ConceptRelationship>,
}

impl ConceptMapper {
    /// Create a new concept mapper
    pub fn new() -> Self {
        let mut concepts = HashMap::new();

        // Authentication concept
        concepts.insert(
            "authentication".to_string(),
            CodeConcept {
                name: "authentication".to_string(),
                description: "User identity verification and access control".to_string(),
                keywords: vec![
                    "login".to_string(),
                    "auth".to_string(),
                    "authenticate".to_string(),
                    "verify".to_string(),
                    "credential".to_string(),
                    "token".to_string(),
                    "session".to_string(),
                    "password".to_string(),
                    "oauth".to_string(),
                    "jwt".to_string(),
                ],
                category: ConceptCategory::Security,
            },
        );

        // Database concept
        concepts.insert(
            "database".to_string(),
            CodeConcept {
                name: "database".to_string(),
                description: "Data storage and retrieval operations".to_string(),
                keywords: vec![
                    "query".to_string(),
                    "sql".to_string(),
                    "database".to_string(),
                    "db".to_string(),
                    "connection".to_string(),
                    "transaction".to_string(),
                    "repository".to_string(),
                    "model".to_string(),
                    "entity".to_string(),
                    "orm".to_string(),
                ],
                category: ConceptCategory::DataProcessing,
            },
        );

        // API concept
        concepts.insert(
            "api".to_string(),
            CodeConcept {
                name: "api".to_string(),
                description: "Application programming interface and web services".to_string(),
                keywords: vec![
                    "endpoint".to_string(),
                    "route".to_string(),
                    "controller".to_string(),
                    "handler".to_string(),
                    "request".to_string(),
                    "response".to_string(),
                    "middleware".to_string(),
                    "validation".to_string(),
                    "rest".to_string(),
                    "graphql".to_string(),
                ],
                category: ConceptCategory::Integration,
            },
        );

        // Message processing concept
        concepts.insert(
            "message_processing".to_string(),
            CodeConcept {
                name: "message_processing".to_string(),
                description: "Event-driven messaging and queue processing".to_string(),
                keywords: vec![
                    "message".to_string(),
                    "queue".to_string(),
                    "event".to_string(),
                    "handler".to_string(),
                    "processor".to_string(),
                    "publish".to_string(),
                    "subscribe".to_string(),
                    "broker".to_string(),
                    "dispatch".to_string(),
                    "stream".to_string(),
                ],
                category: ConceptCategory::Architecture,
            },
        );

        // Error handling concept
        concepts.insert(
            "error_handling".to_string(),
            CodeConcept {
                name: "error_handling".to_string(),
                description: "Exception management and error recovery".to_string(),
                keywords: vec![
                    "error".to_string(),
                    "exception".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                    "handle".to_string(),
                    "raise".to_string(),
                    "throw".to_string(),
                    "fail".to_string(),
                    "recover".to_string(),
                    "retry".to_string(),
                ],
                category: ConceptCategory::Architecture,
            },
        );

        let relationships = vec![
            ConceptRelationship {
                from: "authentication".to_string(),
                to: "api".to_string(),
                relationship_type: RelationshipType::ComplementaryTo,
                strength: 0.8,
            },
            ConceptRelationship {
                from: "database".to_string(),
                to: "api".to_string(),
                relationship_type: RelationshipType::ComplementaryTo,
                strength: 0.7,
            },
            ConceptRelationship {
                from: "error_handling".to_string(),
                to: "api".to_string(),
                relationship_type: RelationshipType::PartOf,
                strength: 0.6,
            },
        ];

        Self {
            concepts,
            relationships,
        }
    }

    /// Map text to relevant concepts
    pub fn map_text_to_concepts(&self, text: &str) -> Vec<String> {
        let mut matched_concepts = Vec::new();
        let text_lower = text.to_lowercase();

        for (concept_name, concept) in &self.concepts {
            let mut score = 0.0;

            // Check direct name match
            if text_lower.contains(concept_name) {
                score += 1.0;
            }

            // Check keyword matches
            for keyword in &concept.keywords {
                if text_lower.contains(keyword) {
                    score += 0.5;
                }
            }

            // If we have a good match, include the concept
            if score >= 0.5 {
                matched_concepts.push(concept_name.clone());
            }
        }

        matched_concepts
    }

    /// Get concept by name
    pub fn get_concept(&self, name: &str) -> Option<&CodeConcept> {
        self.concepts.get(name)
    }

    /// Get all concepts in a category
    pub fn get_concepts_by_category(&self, category: &ConceptCategory) -> Vec<&CodeConcept> {
        self.concepts
            .values()
            .filter(|concept| &concept.category == category)
            .collect()
    }

    /// Find related concepts
    pub fn find_related_concepts(&self, concept_name: &str) -> Vec<String> {
        let mut related = Vec::new();

        for relationship in &self.relationships {
            if relationship.from == concept_name && relationship.strength > 0.5 {
                related.push(relationship.to.clone());
            } else if relationship.to == concept_name && relationship.strength > 0.5 {
                related.push(relationship.from.clone());
            }
        }

        related
    }

    /// Add a new concept
    pub fn add_concept(&mut self, concept: CodeConcept) {
        self.concepts.insert(concept.name.clone(), concept);
    }

    /// Add a concept relationship
    pub fn add_relationship(&mut self, relationship: ConceptRelationship) {
        self.relationships.push(relationship);
    }

    /// Get all concept names
    pub fn get_all_concept_names(&self) -> Vec<String> {
        self.concepts.keys().cloned().collect()
    }

    /// Calculate concept similarity
    pub fn calculate_similarity(&self, concept1: &str, concept2: &str) -> f64 {
        if concept1 == concept2 {
            return 1.0;
        }

        // Check direct relationships
        for relationship in &self.relationships {
            if (relationship.from == concept1 && relationship.to == concept2)
                || (relationship.from == concept2 && relationship.to == concept1)
            {
                return relationship.strength;
            }
        }

        // Check keyword overlap
        if let (Some(c1), Some(c2)) = (self.concepts.get(concept1), self.concepts.get(concept2)) {
            let common_keywords: Vec<_> = c1
                .keywords
                .iter()
                .filter(|k| c2.keywords.contains(k))
                .collect();

            let total_keywords = c1.keywords.len() + c2.keywords.len();
            if total_keywords > 0 {
                return (common_keywords.len() * 2) as f64 / total_keywords as f64;
            }
        }

        0.0
    }
}

impl Default for ConceptMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_creation() {
        let concept = CodeConcept {
            name: "test".to_string(),
            description: "Test concept".to_string(),
            keywords: vec!["test".to_string(), "testing".to_string()],
            category: ConceptCategory::DataProcessing,
        };

        assert_eq!(concept.name, "test");
        assert_eq!(concept.keywords.len(), 2, "Should have 2 items");
    }

    #[test]
    fn test_concept_mapper() {
        let mapper = ConceptMapper::new();

        // Test text mapping
        let concepts = mapper.map_text_to_concepts("user authentication system");
        assert!(concepts.contains(&"authentication".to_string()));

        // Test concept retrieval
        let auth_concept = mapper.get_concept("authentication");
        assert!(auth_concept.is_some(), "Should have value");

        // Test related concepts
        let related = mapper.find_related_concepts("authentication");
        assert!(!!related.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_concept_similarity() {
        let mapper = ConceptMapper::new();

        // Same concept should have similarity 1.0
        let sim = mapper.calculate_similarity("authentication", "authentication");
        assert_eq!(sim, 1.0);

        // Related concepts should have some similarity
        let sim = mapper.calculate_similarity("authentication", "api");
        assert!(sim > 0.0);
    }

    #[test]
    fn test_category_filtering() {
        let mapper = ConceptMapper::new();
        let security_concepts = mapper.get_concepts_by_category(&ConceptCategory::Security);

        assert!(!!security_concepts.is_empty(), "Should not be empty");
        assert!(security_concepts.iter().any(|c| c.name == "authentication"));
    }
}
