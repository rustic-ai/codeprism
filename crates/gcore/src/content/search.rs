//! High-level content search interface
//!
//! This module provides a unified interface for searching content across
//! all file types including documentation, configuration, comments, and source code.

use super::{
    ContentChunk, ContentNode, ContentStats, SearchQuery, SearchResult,
    ContentType, DocumentFormat, ConfigFormat, CommentContext,
    parsers::DocumentParser,
    extractors::CommentExtractor,
    index::{ContentIndex, ContentUpdateListener},
};
use crate::ast::{Language, NodeId};
use crate::graph::GraphStore;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
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
            let comment_chunks = self.comment_extractor.extract_comments(
                language, tree, content, file_path, ast_nodes
            )?;
            
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
    pub fn simple_search(&self, query: &str, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };
        
        self.search(&search_query)
    }
    
    /// Search only in documentation
    pub fn search_documentation(&self, query: &str, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            content_types: vec![
                ContentType::Documentation { format: DocumentFormat::Markdown },
                ContentType::Documentation { format: DocumentFormat::PlainText },
                ContentType::Documentation { format: DocumentFormat::RestructuredText },
                ContentType::Documentation { format: DocumentFormat::AsciiDoc },
                ContentType::Documentation { format: DocumentFormat::Html },
            ],
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };
        
        self.search(&search_query)
    }
    
    /// Search only in comments
    pub fn search_comments(&self, query: &str, language: Option<Language>, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let content_types = if let Some(lang) = language {
            vec![
                ContentType::Comment { language: lang, context: CommentContext::Block },
                ContentType::Comment { language: lang, context: CommentContext::Inline },
                ContentType::Comment { language: lang, context: CommentContext::Documentation },
            ]
        } else {
            vec![
                ContentType::Comment { language: Language::Unknown, context: CommentContext::Block },
                ContentType::Comment { language: Language::Unknown, context: CommentContext::Inline },
                ContentType::Comment { language: Language::Unknown, context: CommentContext::Documentation },
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
    pub fn search_configuration(&self, query: &str, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            content_types: vec![
                ContentType::Configuration { format: ConfigFormat::Json },
                ContentType::Configuration { format: ConfigFormat::Yaml },
                ContentType::Configuration { format: ConfigFormat::Toml },
                ContentType::Configuration { format: ConfigFormat::Ini },
                ContentType::Configuration { format: ConfigFormat::Properties },
                ContentType::Configuration { format: ConfigFormat::Env },
                ContentType::Configuration { format: ConfigFormat::Xml },
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
    pub fn regex_search(&self, pattern: &str, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: pattern.to_string(),
            use_regex: true,
            max_results: max_results.unwrap_or(50),
            ..Default::default()
        };
        
        self.search(&search_query)
    }
    
    /// Search within specific file types
    pub fn search_in_files(&self, query: &str, file_patterns: Vec<String>, max_results: Option<usize>) -> Result<Vec<SearchResult>> {
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
        !matches!(language, Language::Unknown)
    }
    
    /// Index a source code file (without tree-sitter integration)
    fn index_source_file(&self, file_path: &Path, content: &str, language: Language) -> Result<ContentNode> {
        // For now, create a simple code content node
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
        ).with_metadata(serde_json::json!({
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
        Self::new(query).content_types(vec![
            ContentType::Documentation { format: DocumentFormat::Markdown }
        ])
    }
    
    /// Search only in JavaScript/TypeScript comments
    pub fn js_comments(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Comment { language: Language::JavaScript, context: CommentContext::Block },
            ContentType::Comment { language: Language::JavaScript, context: CommentContext::Documentation },
            ContentType::Comment { language: Language::TypeScript, context: CommentContext::Block },
            ContentType::Comment { language: Language::TypeScript, context: CommentContext::Documentation },
        ])
    }
    
    /// Search only in Python docstrings and comments
    pub fn python_docs(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Comment { language: Language::Python, context: CommentContext::Documentation },
            ContentType::Comment { language: Language::Python, context: CommentContext::Inline },
        ])
    }
    
    /// Search only in JSON configuration files
    pub fn json_config(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Configuration { format: ConfigFormat::Json }
        ])
    }
    
    /// Search only in YAML configuration files
    pub fn yaml_config(query: impl Into<String>) -> Self {
        Self::new(query).content_types(vec![
            ContentType::Configuration { format: ConfigFormat::Yaml }
        ])
    }
} 