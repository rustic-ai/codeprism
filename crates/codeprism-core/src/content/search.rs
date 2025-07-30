//! High-level content search interface
//!
//! This module provides a unified interface for searching content across
//! all file types including documentation, configuration, comments, and source code.

use super::{
    extractors::CommentExtractor,
    index::{ContentIndex, ContentUpdateListener},
    parsers::DocumentParser,
    CommentContext, ConfigFormat, ContentChunk, ContentNode, ContentStats, ContentType,
    DocumentFormat, SearchQuery, SearchResult,
};
use crate::ast::{Language, NodeId};
use crate::graph::GraphStore;
use anyhow::Result;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tree_sitter::Tree;

/// High-level content search manager
pub struct ContentSearchManager {
    /// Content index for fast search
    index: Arc<ContentIndex>,
    /// Document parser for non-code files
    document_parser: DocumentParser,
    /// Comment extractor for source files
    comment_extractor: CommentExtractor,
    /// Graph store reference for AST integration
    graph_store: Option<Arc<GraphStore>>,
}

impl ContentSearchManager {
    /// Create a new content search manager
    pub fn new() -> Self {
        Self {
            index: Arc::new(ContentIndex::new()),
            document_parser: DocumentParser::new(),
            comment_extractor: CommentExtractor::new(),
            graph_store: None,
        }
    }

    /// Create with graph store integration
    pub fn with_graph_store(graph_store: Arc<GraphStore>) -> Self {
        let mut manager = Self::new();
        manager.graph_store = Some(graph_store);
        manager
    }

    /// Index a file's content
    pub fn index_file(&self, file_path: &Path, content: &str) -> Result<()> {
        let language = self.detect_language(file_path);

        let content_node = match language {
            Some(lang) if self.is_source_code_language(lang) => {
                self.index_source_file(file_path, content, lang)?
            }
            _ => {
                // Handle as document/config file
                self.document_parser.parse_file(file_path, content)?
            }
        };

        self.index.add_node(content_node)?;
        Ok(())
    }

    /// Index a source code file with comments
    pub fn index_source_file_with_tree(
        &self,
        file_path: &Path,
        content: &str,
        tree: &Tree,
        language: Language,
        ast_nodes: &[NodeId],
    ) -> Result<()> {
        let mut content_node = self.index_source_file(file_path, content, language)?;

        // Extract comments from the parse tree
        if self.comment_extractor.supports_language(language) {
            let comment_chunks = self
                .comment_extractor
                .extract_comments(language, tree, content, file_path, ast_nodes)?;

            for chunk in comment_chunks {
                content_node.add_chunk(chunk);
            }
        }

        // Link AST nodes
        for node_id in ast_nodes {
            content_node.add_ast_node(*node_id);
        }

        self.index.add_node(content_node)?;
        Ok(())
    }

    /// Remove a file from the index
    pub fn remove_file(&self, file_path: &Path) -> Result<()> {
        self.index.remove_node(file_path)
    }

    /// Search for content
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        self.index.search(query)
    }

    /// Search with simple text query
    pub fn simple_search(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Search only in documentation
    pub fn search_documentation(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            content_types: vec![
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                ContentType::Documentation {
                    format: DocumentFormat::PlainText,
                },
                ContentType::Documentation {
                    format: DocumentFormat::RestructuredText,
                },
                ContentType::Documentation {
                    format: DocumentFormat::AsciiDoc,
                },
                ContentType::Documentation {
                    format: DocumentFormat::Html,
                },
            ],
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Search only in comments
    pub fn search_comments(
        &self,
        query: &str,
        language: Option<Language>,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let content_types = if let Some(lang) = language {
            vec![
                ContentType::Comment {
                    language: lang,
                    context: CommentContext::Block,
                },
                ContentType::Comment {
                    language: lang,
                    context: CommentContext::Inline,
                },
                ContentType::Comment {
                    language: lang,
                    context: CommentContext::Documentation,
                },
            ]
        } else {
            vec![
                ContentType::Comment {
                    language: Language::Unknown,
                    context: CommentContext::Block,
                },
                ContentType::Comment {
                    language: Language::Unknown,
                    context: CommentContext::Inline,
                },
                ContentType::Comment {
                    language: Language::Unknown,
                    context: CommentContext::Documentation,
                },
            ]
        };

        let search_query = SearchQuery {
            query: query.to_string(),
            content_types,
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Search only in configuration files
    pub fn search_configuration(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            content_types: vec![
                ContentType::Configuration {
                    format: ConfigFormat::Json,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Yaml,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Toml,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Ini,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Properties,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Env,
                },
                ContentType::Configuration {
                    format: ConfigFormat::Xml,
                },
            ],
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Find files by pattern
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        self.index.find_files(pattern)
    }

    /// Get content statistics
    pub fn get_stats(&self) -> ContentStats {
        self.index.get_stats()
    }

    /// Get a specific content node
    pub fn get_node(&self, file_path: &Path) -> Option<ContentNode> {
        self.index.get_node(file_path)
    }

    /// Add an update listener
    pub fn add_update_listener(&self, listener: Box<dyn ContentUpdateListener>) {
        self.index.add_update_listener(listener);
    }

    /// Clear all indexed content
    pub fn clear(&self) {
        self.index.clear();
    }

    /// Search with regex pattern
    pub fn regex_search(
        &self,
        pattern: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: pattern.to_string(),
            use_regex: true,
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Search within specific file types
    pub fn search_in_files(
        &self,
        query: &str,
        file_patterns: Vec<String>,
        max_results: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            file_patterns,
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };

        self.search(&search_query)
    }

    /// Get supported languages for comment extraction
    pub fn supported_comment_languages(&self) -> Vec<Language> {
        self.comment_extractor.supported_languages()
    }

    /// Check if a language is supported for comment extraction
    pub fn supports_comment_extraction(&self, language: Language) -> bool {
        self.comment_extractor.supports_language(language)
    }

    // Private helper methods

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Option<Language> {
        let extension = file_path.extension()?.to_str()?;
        let lang = Language::from_extension(extension);
        if matches!(lang, Language::Unknown) {
            None
        } else {
            Some(lang)
        }
    }

    /// Check if a language is a source code language
    fn is_source_code_language(&self, language: Language) -> bool {
        matches!(
            language,
            Language::JavaScript
                | Language::TypeScript
                | Language::Python
                | Language::Rust
                | Language::Java
                | Language::Cpp
                | Language::C
                | Language::Go
        )
    }

    /// Index a source code file (without tree-sitter integration)
    fn index_source_file(
        &self,
        file_path: &Path,
        content: &str,
        language: Language,
    ) -> Result<ContentNode> {
        // Currently creating a simple code content node
        // In the future, this could be enhanced with basic syntax highlighting
        let content_type = ContentType::Code { language };
        let mut node = ContentNode::new(file_path.to_path_buf(), content_type.clone());

        // Create a single chunk for the entire file
        let span = crate::ast::Span::new(
            0,
            content.len(),
            1,
            content.lines().count(),
            1,
            content.lines().last().map(|l| l.len()).unwrap_or(0),
        );

        let chunk = ContentChunk::new(
            file_path.to_path_buf(),
            content_type,
            content.to_string(),
            span,
            0,
        )
        .with_metadata(serde_json::json!({
            "language": format!("{:?}", language),
            "content_type": "source_code"
        }));

        node.add_chunk(chunk);
        node.file_size = content.len();

        Ok(node)
    }
}

impl Default for ContentSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating search queries
#[derive(Debug, Clone)]
pub struct SearchQueryBuilder {
    query: SearchQuery,
}

impl SearchQueryBuilder {
    /// Create a new search query builder
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: SearchQuery {
                query: query.into(),
                ..Default::default()
            },
        }
    }

    /// Set content types to search in
    pub fn content_types(mut self, types: Vec<ContentType>) -> Self {
        self.query.content_types = types;
        self
    }

    /// Add file patterns to include
    pub fn include_files(mut self, patterns: Vec<String>) -> Self {
        self.query.file_patterns = patterns;
        self
    }

    /// Add file patterns to exclude
    pub fn exclude_files(mut self, patterns: Vec<String>) -> Self {
        self.query.exclude_patterns = patterns;
        self
    }

    /// Set maximum number of results
    pub fn max_results(mut self, max: usize) -> Self {
        self.query.max_results = max;
        self
    }

    /// Enable case sensitive search
    pub fn case_sensitive(mut self) -> Self {
        self.query.case_sensitive = true;
        self
    }

    /// Enable regex pattern matching
    pub fn use_regex(mut self) -> Self {
        self.query.use_regex = true;
        self
    }

    /// Include context around matches
    pub fn with_context(mut self, lines: usize) -> Self {
        self.query.include_context = true;
        self.query.context_lines = lines;
        self
    }

    /// Disable context around matches
    pub fn without_context(mut self) -> Self {
        self.query.include_context = false;
        self
    }

    /// Build the search query
    pub fn build(self) -> SearchQuery {
        self.query
    }
}

/// Convenience functions for common search patterns
impl SearchQueryBuilder {
    /// Search only in markdown documentation
    pub fn markdown_docs(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![ContentType::Documentation {
            format: DocumentFormat::Markdown,
        }])
    }

    /// Search only in JavaScript/TypeScript comments
    pub fn js_comments(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Comment {
                language: Language::JavaScript,
                context: CommentContext::Block,
            },
            ContentType::Comment {
                language: Language::JavaScript,
                context: CommentContext::Documentation,
            },
            ContentType::Comment {
                language: Language::TypeScript,
                context: CommentContext::Block,
            },
            ContentType::Comment {
                language: Language::TypeScript,
                context: CommentContext::Documentation,
            },
        ])
    }

    /// Search only in Python docstrings and comments
    pub fn python_docs(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Comment {
                language: Language::Python,
                context: CommentContext::Documentation,
            },
            ContentType::Comment {
                language: Language::Python,
                context: CommentContext::Inline,
            },
        ])
    }

    /// Search only in JSON configuration files
    pub fn json_config(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![ContentType::Configuration {
            format: ConfigFormat::Json,
        }])
    }

    /// Search only in YAML configuration files
    pub fn yaml_config(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![ContentType::Configuration {
            format: ConfigFormat::Yaml,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_content_search_manager_creation() {
        let manager = ContentSearchManager::new();
        assert!(
            manager.graph_store.is_none(),
            "Default search manager should not have graph store initialized"
        );

        // Test default implementation
        let manager_default = ContentSearchManager::default();
        assert!(
            manager_default.graph_store.is_none(),
            "Default-created search manager should not have graph store"
        );

        // Verify managers are functional
        assert!(
            manager.content_index.nodes.is_empty(),
            "New manager should have empty content index"
        );
        assert!(
            manager_default.content_index.nodes.is_empty(),
            "Default manager should have empty content index"
        );
    }

    #[test]
    fn test_with_graph_store() {
        let graph_store = Arc::new(GraphStore::new());
        let manager = ContentSearchManager::with_graph_store(graph_store.clone());
        assert!(
            manager.graph_store.is_some(),
            "Search manager should have graph store after enabling"
        );

        // Verify the graph store is functional
        let graph_store = manager.graph_store.as_ref().unwrap();
        assert!(
            graph_store.nodes.is_empty(),
            "New graph store should start empty"
        );
    }

    #[test]
    fn test_language_detection() {
        let manager = ContentSearchManager::new();

        // Test various file extensions
        assert_eq!(
            manager.detect_language(Path::new("test.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            manager.detect_language(Path::new("test.py")),
            Some(Language::Python)
        );
        assert_eq!(
            manager.detect_language(Path::new("test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            manager.detect_language(Path::new("test.java")),
            Some(Language::Java)
        );
        assert_eq!(
            manager.detect_language(Path::new("test.ts")),
            Some(Language::TypeScript)
        );

        // Test unknown extensions
        assert_eq!(manager.detect_language(Path::new("test.unknown")), None);
        assert_eq!(manager.detect_language(Path::new("README")), None);
    }

    #[test]
    fn test_is_source_code_language() {
        let manager = ContentSearchManager::new();

        assert!(manager.is_source_code_language(Language::JavaScript));
        assert!(manager.is_source_code_language(Language::Python));
        assert!(manager.is_source_code_language(Language::Rust));
        assert!(!manager.is_source_code_language(Language::Unknown));
    }

    #[test]
    fn test_index_markdown_file() {
        let manager = ContentSearchManager::new();
        let file_path = Path::new("test.md");
        let content = "# Title\n\nThis is a test document.";

        let result = manager.index_file(file_path, content);
        assert!(result.is_ok(), "Search operation should succeed");

        // Verify the file was indexed
        let node = manager.get_node(file_path);
        assert!(node.is_some(), "Should find content node");
        let node = node.unwrap();
        assert_eq!(node.file_path, file_path);
        assert!(!node.chunks.is_empty(), "Node should have content chunks");
    }

    #[test]
    fn test_index_javascript_file() {
        let manager = ContentSearchManager::new();
        let file_path = Path::new("test.js");
        let content = "// Comment\nfunction test() { return 42; }";

        let result = manager.index_file(file_path, content);
        assert!(result.is_ok(), "Search operation should succeed");

        // Verify the file was indexed as source code
        let node = manager.get_node(file_path);
        assert!(node.is_some(), "Should find content node");
        let node = node.unwrap();
        assert_eq!(node.file_path, file_path);
        assert!(!node.chunks.is_empty(), "Node should have content chunks");

        // Should have one chunk with the entire source code
        assert_eq!(node.chunks.len(), 1, "Should have 1 items");
        if let ContentType::Code { language } = &node.chunks[0].content_type {
            assert_eq!(*language, Language::JavaScript);
        } else {
            panic!("Expected code content type");
        }
    }

    #[test]
    fn test_simple_search() {
        let manager = ContentSearchManager::new();

        // Index some test content
        let _ = manager.index_file(
            Path::new("test1.md"),
            "# Hello World\n\nThis is a test document about programming.",
        );
        let _ = manager.index_file(
            Path::new("test2.md"),
            "# Testing\n\nAnother document for testing purposes.",
        );

        // Search for content
        let results = manager.simple_search("test", Some(10)).unwrap();
        assert!(!results.is_empty(), "Should not be empty");

        // Search with max results
        let results = manager.simple_search("test", Some(1)).unwrap();
        assert!(results.len() <= 1);

        // Search for non-existent content
        let results = manager.simple_search("nonexistent", Some(10)).unwrap();
        assert!(
            results.is_empty(),
            "Should be empty for non-existent content"
        );
    }

    #[test]
    fn test_search_documentation() {
        let manager = ContentSearchManager::new();

        // Index documentation
        let _ = manager.index_file(
            Path::new("doc.md"),
            "# API Documentation\n\nThis describes the API.",
        );
        let _ = manager.index_file(Path::new("readme.txt"), "README file with API information.");
        let _ = manager.index_file(
            Path::new("code.js"),
            "// This is not documentation\nfunction api() {}",
        );

        let results = manager.search_documentation("API", Some(10)).unwrap();

        // Should only find documentation files, not source code
        assert!(!results.is_empty(), "Should not be empty");
        for result in &results {
            match &result.chunk.content_type {
                ContentType::Documentation { .. } => {} // Expected
                _ => panic!("Found non-documentation content in documentation search"),
            }
        }
    }

    #[test]
    fn test_search_configuration() {
        let manager = ContentSearchManager::new();

        // Index configuration files
        let _ = manager.index_file(Path::new("config.json"), r#"{"database": "localhost"}"#);
        let _ = manager.index_file(Path::new("settings.yaml"), "database:\n  host: localhost");
        let _ = manager.index_file(Path::new("readme.md"), "Database configuration info");

        let results = manager.search_configuration("database", Some(10)).unwrap();

        // Should only find configuration files
        assert!(!results.is_empty(), "Should not be empty");
        for result in &results {
            match &result.chunk.content_type {
                ContentType::Configuration { .. } => {} // Expected
                _ => panic!("Found non-configuration content in configuration search"),
            }
        }
    }

    #[test]
    fn test_regex_search() {
        let manager = ContentSearchManager::new();

        // Index content with patterns
        let _ = manager.index_file(
            Path::new("test.md"),
            "Email: user@example.com\nAnother: admin@test.org",
        );

        // Search with regex pattern
        let results = manager.regex_search(r"\b\w+@\w+\.\w+\b", Some(10)).unwrap();
        assert!(!results.is_empty(), "Should not be empty");

        // Invalid regex should return error
        let invalid_result = manager.regex_search("[invalid", Some(10));
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_search_in_files() {
        let manager = ContentSearchManager::new();

        // Index different file types
        let _ = manager.index_file(Path::new("test.md"), "markdown content");
        let _ = manager.index_file(Path::new("test.txt"), "text content");
        let _ = manager.index_file(Path::new("config.json"), r#"{"content": "json"}"#);

        // Search only in markdown files
        let results = manager
            .search_in_files("content", vec!["*.md".to_string()], Some(10))
            .unwrap();
        assert!(!results.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_file_removal() {
        let manager = ContentSearchManager::new();
        let file_path = Path::new("temp.md");

        // Index a file
        let _ = manager.index_file(file_path, "# Temporary\n\nThis will be removed.");
        assert!(manager.get_node(file_path).is_some());

        // Remove the file
        let result = manager.remove_file(file_path);
        assert!(result.is_ok(), "Search operation should succeed");
        assert!(manager.get_node(file_path).is_none());
    }

    #[test]
    fn test_clear() {
        let manager = ContentSearchManager::new();

        // Index some files
        let _ = manager.index_file(Path::new("test1.md"), "Content 1");
        let _ = manager.index_file(Path::new("test2.md"), "Content 2");

        // Verify files are indexed
        assert!(manager.get_node(Path::new("test1.md")).is_some());
        assert!(manager.get_node(Path::new("test2.md")).is_some());

        // Clear all content
        manager.clear();

        // Verify files are removed
        assert!(manager.get_node(Path::new("test1.md")).is_none());
        assert!(manager.get_node(Path::new("test2.md")).is_none());
    }

    #[test]
    fn test_get_stats() {
        let manager = ContentSearchManager::new();

        // Initially should have empty stats
        let stats = manager.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);

        // Index some content
        let _ = manager.index_file(Path::new("test.md"), "# Title\n\nContent");

        // Stats should be updated
        let stats = manager.get_stats();
        assert!(stats.total_files > 0);
        assert!(stats.total_chunks > 0);
    }

    #[test]
    fn test_find_files() {
        let manager = ContentSearchManager::new();

        // Index files with different names
        let _ = manager.index_file(Path::new("test_one.md"), "Content 1");
        let _ = manager.index_file(Path::new("test_two.md"), "Content 2");
        let _ = manager.index_file(Path::new("other.txt"), "Content 3");

        // Find markdown files
        let md_files = manager.find_files(r"\.md$").unwrap();
        assert_eq!(md_files.len(), 2, "Should have 2 items");

        // Find all test files
        let test_files = manager.find_files(r"test_").unwrap();
        assert_eq!(test_files.len(), 2, "Should have 2 items");
    }

    #[test]
    fn test_supported_comment_languages() {
        let manager = ContentSearchManager::new();

        let supported = manager.supported_comment_languages();
        assert!(supported.contains(&Language::JavaScript));
        assert!(supported.contains(&Language::Python));

        assert!(manager.supports_comment_extraction(Language::JavaScript));
        assert!(manager.supports_comment_extraction(Language::Python));
        assert!(!manager.supports_comment_extraction(Language::Unknown));
    }

    #[test]
    fn test_search_query_builder() {
        // Test basic builder
        let query = SearchQueryBuilder::new("test query")
            .max_results(10)
            .case_sensitive()
            .build();

        assert_eq!(query.query, "test query");
        assert_eq!(query.max_results, 10);
        assert!(query.case_sensitive);

        // Test with content types
        let query = SearchQueryBuilder::new("search")
            .content_types(vec![ContentType::Documentation {
                format: DocumentFormat::Markdown,
            }])
            .build();

        assert_eq!(query.content_types.len(), 1, "Should have 1 items");

        // Test with file patterns
        let query = SearchQueryBuilder::new("search")
            .include_files(vec!["*.md".to_string()])
            .exclude_files(vec!["*.tmp".to_string()])
            .build();

        assert_eq!(query.file_patterns.len(), 1, "Should have 1 items");
        assert_eq!(query.exclude_patterns.len(), 1, "Should have 1 items");

        // Test with regex and context
        let query = SearchQueryBuilder::new("pattern")
            .use_regex()
            .with_context(3)
            .build();

        assert!(query.use_regex);
        assert!(query.include_context);
        assert_eq!(query.context_lines, 3);

        // Test without context
        let query = SearchQueryBuilder::new("pattern").without_context().build();

        assert!(!query.include_context);
    }

    #[test]
    fn test_search_query_builder_convenience_methods() {
        // Test markdown docs builder
        let query = SearchQueryBuilder::markdown_docs("test").build();
        assert_eq!(query.content_types.len(), 1, "Should have 1 items");
        match &query.content_types[0] {
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            } => {}
            _ => panic!("Expected markdown documentation type"),
        }

        // Test JS comments builder
        let query = SearchQueryBuilder::js_comments("test").build();
        assert_eq!(query.content_types.len(), 4, "Should have 4 items"); // JS + TS, Block + Documentation

        // Test Python docs builder
        let query = SearchQueryBuilder::python_docs("test").build();
        assert_eq!(query.content_types.len(), 2, "Should have 2 items"); // Documentation + Inline

        // Test JSON config builder
        let query = SearchQueryBuilder::json_config("test").build();
        assert_eq!(query.content_types.len(), 1, "Should have 1 items");
        match &query.content_types[0] {
            ContentType::Configuration {
                format: ConfigFormat::Json,
            } => {}
            _ => panic!("Expected JSON configuration type"),
        }

        // Test YAML config builder
        let query = SearchQueryBuilder::yaml_config("test").build();
        assert_eq!(query.content_types.len(), 1, "Should have 1 items");
        match &query.content_types[0] {
            ContentType::Configuration {
                format: ConfigFormat::Yaml,
            } => {}
            _ => panic!("Expected YAML configuration type"),
        }
    }
}
