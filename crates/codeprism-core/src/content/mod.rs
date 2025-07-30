//! Content search and indexing infrastructure
//!
//! This module provides comprehensive content search capabilities for all file types
//! in a repository, including documentation, configuration files, code comments,
//! and source code content.

use crate::ast::{Language, NodeId, Span};
use blake3::Hasher;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub mod extractors;
pub mod index;
pub mod parsers;
pub mod search;

/// Unique identifier for content chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId([u8; 16]);

impl ChunkId {
    /// Create a new chunk ID
    pub fn new(file_path: &Path, chunk_index: usize, content_hash: &[u8; 32]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(file_path.to_string_lossy().as_bytes());
        hasher.update(&chunk_index.to_le_bytes());
        hasher.update(content_hash);

        let hash = hasher.finalize();
        let mut id = [0u8; 16];
        id.copy_from_slice(&hash.as_bytes()[..16]);
        Self(id)
    }

    /// Get the ID as a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Types of content that can be indexed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Source code with language context
    Code {
        /// Programming language of the source code
        language: Language,
    },
    /// Documentation files
    Documentation {
        /// Format of the documentation file
        format: DocumentFormat,
    },
    /// Configuration files
    Configuration {
        /// Format of the configuration file
        format: ConfigFormat,
    },
    /// Code comments
    Comment {
        /// Programming language containing the comment
        language: Language,
        /// Context where the comment appears
        context: CommentContext,
    },
    /// Plain text files
    PlainText,
}

/// Documentation formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    /// Markdown format (.md)
    Markdown,
    /// reStructuredText format (.rst)
    RestructuredText,
    /// AsciiDoc format (.adoc)
    AsciiDoc,
    /// Plain text format (.txt)
    PlainText,
    /// HTML format (.html)
    Html,
}

/// Configuration file formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFormat {
    /// JSON format (.json)
    Json,
    /// YAML format (.yml, .yaml)
    Yaml,
    /// TOML format (.toml)
    Toml,
    /// INI format (.ini)
    Ini,
    /// Properties format (.properties)
    Properties,
    /// Environment variable format (.env)
    Env,
    /// XML format (.xml)
    Xml,
}

/// Context for comments within code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentContext {
    /// Comment associated with a function
    Function {
        /// Name of the function this comment describes
        function_name: String,
    },
    /// Comment associated with a class
    Class {
        /// Name of the class this comment describes
        class_name: String,
    },
    /// Comment associated with a module/file
    Module,
    /// Inline comment
    Inline,
    /// Block comment
    Block,
    /// Documentation comment (e.g., JSDoc, Python docstring)
    Documentation,
}

/// A chunk of content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChunk {
    /// Unique identifier for this chunk
    pub id: ChunkId,
    /// Type of content
    pub content_type: ContentType,
    /// The actual text content
    pub content: String,
    /// Source location in the file
    pub span: Span,
    /// File path
    pub file_path: PathBuf,
    /// Extracted tokens for search
    pub tokens: Vec<String>,
    /// Related AST nodes (if any)
    pub related_nodes: Vec<NodeId>,
    /// When this chunk was last updated
    pub last_modified: SystemTime,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl ContentChunk {
    /// Create a new content chunk
    pub fn new(
        file_path: PathBuf,
        content_type: ContentType,
        content: String,
        span: Span,
        chunk_index: usize,
    ) -> Self {
        let content_bytes = blake3::hash(content.as_bytes());
        let id = ChunkId::new(&file_path, chunk_index, content_bytes.as_bytes());

        Self {
            id,
            content_type,
            content: content.clone(),
            span,
            file_path,
            tokens: Self::tokenize_content(&content),
            related_nodes: Vec::new(),
            last_modified: SystemTime::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Extract tokens from content for search indexing
    fn tokenize_content(content: &str) -> Vec<String> {
        // Simple tokenization - split on whitespace and common delimiters
        let re = Regex::new(r"[^\w]+").unwrap();
        re.split(content)
            .filter(|s| !s.is_empty() && s.len() > 1) // Filter out empty and single chars
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Add related AST node
    pub fn add_related_node(&mut self, node_id: NodeId) {
        if !self.related_nodes.contains(&node_id) {
            self.related_nodes.push(node_id);
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Content node representing an entire file's content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentNode {
    /// File path
    pub file_path: PathBuf,
    /// Content type of the file
    pub content_type: ContentType,
    /// All content chunks in this file
    pub chunks: Vec<ContentChunk>,
    /// AST nodes associated with this file
    pub ast_nodes: Vec<NodeId>,
    /// When this file was last indexed
    pub last_indexed: SystemTime,
    /// File size in bytes
    pub file_size: usize,
    /// Whether this file is actively monitored for changes
    pub is_monitored: bool,
}

impl ContentNode {
    /// Create a new content node
    pub fn new(file_path: PathBuf, content_type: ContentType) -> Self {
        Self {
            file_path,
            content_type,
            chunks: Vec::new(),
            ast_nodes: Vec::new(),
            last_indexed: SystemTime::now(),
            file_size: 0,
            is_monitored: true,
        }
    }

    /// Add a content chunk
    pub fn add_chunk(&mut self, chunk: ContentChunk) {
        self.chunks.push(chunk);
    }

    /// Add related AST node
    pub fn add_ast_node(&mut self, node_id: NodeId) {
        if !self.ast_nodes.contains(&node_id) {
            self.ast_nodes.push(node_id);
        }
    }

    /// Get all tokens from all chunks
    pub fn get_all_tokens(&self) -> Vec<String> {
        let mut all_tokens = Vec::new();
        for chunk in &self.chunks {
            all_tokens.extend(chunk.tokens.clone());
        }
        all_tokens.sort();
        all_tokens.dedup();
        all_tokens
    }

    /// Search for content within this node
    pub fn search(&self, query: &str, case_sensitive: bool) -> Vec<&ContentChunk> {
        let search_query = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        self.chunks
            .iter()
            .filter(|chunk| {
                let content = if case_sensitive {
                    &chunk.content
                } else {
                    &chunk.content.to_lowercase()
                };
                content.contains(&search_query)
            })
            .collect()
    }
}

/// Statistics about indexed content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStats {
    /// Total number of indexed files
    pub total_files: usize,
    /// Total number of content chunks
    pub total_chunks: usize,
    /// Total number of unique tokens
    pub total_tokens: usize,
    /// Content by type
    pub content_by_type: HashMap<String, usize>,
    /// File size distribution
    pub size_distribution: HashMap<String, usize>,
    /// When stats were last computed
    pub computed_at: SystemTime,
}

impl ContentStats {
    /// Create empty stats
    pub fn new() -> Self {
        Self {
            total_files: 0,
            total_chunks: 0,
            total_tokens: 0,
            content_by_type: HashMap::new(),
            size_distribution: HashMap::new(),
            computed_at: SystemTime::now(),
        }
    }
}

impl Default for ContentStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Content search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search text or pattern
    pub query: String,
    /// Content types to search in
    pub content_types: Vec<ContentType>,
    /// File patterns to include
    pub file_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum number of results
    pub max_results: usize,
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Use regex pattern matching
    pub use_regex: bool,
    /// Include context around matches
    pub include_context: bool,
    /// Context lines before and after match
    pub context_lines: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            content_types: vec![
                ContentType::Code {
                    language: Language::Unknown,
                },
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                ContentType::Comment {
                    language: Language::Unknown,
                    context: CommentContext::Block,
                },
            ],
            file_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            max_results: 100,
            case_sensitive: false,
            use_regex: false,
            include_context: true,
            context_lines: 2,
        }
    }
}

/// Search result for a content match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching content chunk
    pub chunk: ContentChunk,
    /// Relevance score (0.0 to 1.0)
    pub score: f32,
    /// Matched text snippets with highlighting
    pub matches: Vec<SearchMatch>,
    /// Related AST nodes
    pub related_nodes: Vec<NodeId>,
}

/// Individual match within content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Matched text
    pub text: String,
    /// Position in the content
    pub position: usize,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Column number (1-indexed)
    pub column_number: usize,
    /// Context before the match
    pub context_before: Option<String>,
    /// Context after the match
    pub context_after: Option<String>,
}

/// Update event for content changes
#[derive(Debug, Clone)]
pub struct ContentUpdate {
    /// File that changed
    pub file_path: PathBuf,
    /// Type of update
    pub update_kind: ContentUpdateKind,
    /// When the update occurred
    pub timestamp: SystemTime,
}

/// Types of content updates
#[derive(Debug, Clone)]
pub enum ContentUpdateKind {
    /// File was created
    Created,
    /// File content was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed {
        /// The previous path before the file was renamed
        old_path: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeKind;

    #[test]
    fn test_chunk_id_generation() {
        let file_path = PathBuf::from("test.md");
        let content_hash = [0u8; 32];

        let id1 = ChunkId::new(&file_path, 0, &content_hash);
        let id2 = ChunkId::new(&file_path, 0, &content_hash);
        let id3 = ChunkId::new(&file_path, 1, &content_hash);

        assert_eq!(id1, id2, "Same inputs should generate same ID");
        assert_ne!(
            id1, id3,
            "Different chunk index should generate different ID"
        );

        let hex = id1.to_hex();
        assert_eq!(hex.len(), 32, "Hex string should be 32 characters");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "Should be valid hex"
        );
    }

    #[test]
    fn test_content_types_serialization() {
        let test_cases = vec![
            ContentType::Code {
                language: Language::Python,
            },
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            ContentType::Configuration {
                format: ConfigFormat::Json,
            },
            ContentType::Comment {
                language: Language::JavaScript,
                context: CommentContext::Function {
                    function_name: "test".to_string(),
                },
            },
            ContentType::PlainText,
        ];

        for content_type in test_cases {
            let json = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&json).unwrap();
            assert_eq!(
                std::mem::discriminant(&content_type),
                std::mem::discriminant(&deserialized),
                "Serialization roundtrip failed for: {content_type:?}"
            );
        }
    }

    #[test]
    fn test_content_chunk_creation() {
        let file_path = PathBuf::from("test.md");
        let content_type = ContentType::Documentation {
            format: DocumentFormat::Markdown,
        };
        let content = "# Test Header\nSome content here.".to_string();
        let span = Span::new(0, content.len(), 1, 2, 1, 19);

        let chunk = ContentChunk::new(
            file_path.clone(),
            content_type.clone(),
            content.clone(),
            span,
            0,
        );

        assert_eq!(chunk.file_path, file_path);
        assert_eq!(chunk.content, content);
        assert!(
            !chunk.tokens.is_empty(),
            "Should extract tokens from content"
        );
        assert!(
            chunk.tokens.contains(&"test".to_string()),
            "Should extract 'test' token"
        );
        assert!(
            chunk.tokens.contains(&"header".to_string()),
            "Should extract 'header' token"
        );
        assert!(
            chunk.tokens.contains(&"content".to_string()),
            "Should extract 'content' token"
        );
    }

    #[test]
    fn test_content_chunk_tokenization() {
        let file_path = PathBuf::from("test.py");
        let content_type = ContentType::Code {
            language: Language::Python,
        };
        let content = "def hello_world():\n    print('Hello, World!')".to_string();
        let span = Span::new(0, content.len(), 1, 2, 1, 26);

        let chunk = ContentChunk::new(file_path, content_type, content, span, 0);

        assert!(chunk.tokens.contains(&"def".to_string()));
        assert!(chunk.tokens.contains(&"hello".to_string()));
        assert!(chunk.tokens.contains(&"world".to_string()));
        assert!(chunk.tokens.contains(&"print".to_string()));
        assert!(
            !chunk.tokens.contains(&"(".to_string()),
            "Should filter out single chars"
        );
    }

    #[test]
    fn test_content_node_operations() {
        let file_path = PathBuf::from("test.md");
        let content_type = ContentType::Documentation {
            format: DocumentFormat::Markdown,
        };
        let mut node = ContentNode::new(file_path.clone(), content_type.clone());

        assert_eq!(node.file_path, file_path);
        assert_eq!(node.chunks.len(), 0);
        assert_eq!(node.ast_nodes.len(), 0);

        // Add a chunk
        let chunk = ContentChunk::new(
            file_path.clone(),
            content_type,
            "Test content".to_string(),
            Span::new(0, 12, 1, 1, 1, 13),
            0,
        );
        node.add_chunk(chunk);

        assert_eq!(node.chunks.len(), 1);

        // Add AST node
        let node_id = NodeId::new(
            "test",
            &file_path,
            &Span::new(0, 5, 1, 1, 1, 6),
            &NodeKind::Function,
        );
        node.add_ast_node(node_id);

        assert_eq!(node.ast_nodes.len(), 1);
        assert_eq!(node.ast_nodes[0], node_id);

        // Test token aggregation
        let tokens = node.get_all_tokens();
        assert!(tokens.contains(&"test".to_string()));
        assert!(tokens.contains(&"content".to_string()));
    }

    #[test]
    fn test_content_node_search() {
        let file_path = PathBuf::from("test.md");
        let content_type = ContentType::Documentation {
            format: DocumentFormat::Markdown,
        };
        let mut node = ContentNode::new(file_path.clone(), content_type.clone());

        // Add chunks with different content
        let chunk1 = ContentChunk::new(
            file_path.clone(),
            content_type.clone(),
            "First test content".to_string(),
            Span::new(0, 18, 1, 1, 1, 19),
            0,
        );
        let chunk2 = ContentChunk::new(
            file_path.clone(),
            content_type.clone(),
            "Second example content".to_string(),
            Span::new(19, 41, 2, 2, 1, 23),
            1,
        );
        node.add_chunk(chunk1);
        node.add_chunk(chunk2);

        // Case insensitive search
        let results = node.search("TEST", false);
        assert_eq!(results.len(), 1, "Should find 'test' case-insensitively");

        let results = node.search("content", false);
        assert_eq!(results.len(), 2, "Should find 'content' in both chunks");

        // Case sensitive search
        let results = node.search("TEST", true);
        assert_eq!(results.len(), 0, "Should not find 'TEST' case-sensitively");

        let results = node.search("First", true);
        assert_eq!(results.len(), 1, "Should find exact case match");
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();

        assert_eq!(query.query, "");
        assert_eq!(query.max_results, 100);
        assert!(!query.case_sensitive);
        assert!(!query.use_regex);
        assert!(query.include_context);
        assert_eq!(query.context_lines, 2);
        assert_eq!(query.content_types.len(), 3);
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery {
            query: "test query".to_string(),
            content_types: vec![ContentType::Code {
                language: Language::Python,
            }],
            file_patterns: vec!["*.py".to_string()],
            exclude_patterns: vec!["test_*.py".to_string()],
            max_results: 25,
            case_sensitive: true,
            use_regex: true,
            include_context: false,
            context_lines: 5,
        };

        // Test all fields are set correctly
        assert_eq!(query.query, "test query");
        assert_eq!(query.max_results, 25);
        assert!(query.case_sensitive);
        assert!(query.use_regex);
        assert!(!query.include_context);
        assert_eq!(query.context_lines, 5);
        assert_eq!(query.file_patterns, vec!["*.py"]);
        assert_eq!(query.exclude_patterns, vec!["test_*.py"]);
    }

    #[test]
    fn test_content_stats_creation() {
        let mut stats = ContentStats::new();

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_tokens, 0);
        assert!(stats.content_by_type.is_empty());
        assert!(stats.size_distribution.is_empty());

        // Test updating stats
        stats.total_files = 10;
        stats.total_chunks = 50;
        stats.total_tokens = 1000;
        stats.content_by_type.insert("code:python".to_string(), 15);
        stats.size_distribution.insert("small".to_string(), 8);

        assert_eq!(stats.total_files, 10);
        assert_eq!(stats.total_chunks, 50);
        assert_eq!(stats.total_tokens, 1000);
    }

    #[test]
    fn test_search_result_structure() {
        let file_path = PathBuf::from("test.md");
        let chunk = ContentChunk::new(
            file_path,
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            "Test content with query match".to_string(),
            Span::new(0, 29, 1, 1, 1, 30),
            0,
        );

        let search_match = SearchMatch {
            text: "query".to_string(),
            position: 18,
            line_number: 1,
            column_number: 19,
            context_before: Some("Test content with ".to_string()),
            context_after: Some(" match".to_string()),
        };

        let result = SearchResult {
            chunk: chunk.clone(),
            score: 0.85,
            matches: vec![search_match.clone()],
            related_nodes: vec![],
        };

        assert_eq!(result.score, 0.85);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].text, "query");
        assert_eq!(result.matches[0].position, 18);
        assert_eq!(result.chunk.content, chunk.content);
    }

    #[test]
    fn test_comment_context_variants() {
        let contexts = vec![
            CommentContext::Function {
                function_name: "test_func".to_string(),
            },
            CommentContext::Class {
                class_name: "TestClass".to_string(),
            },
            CommentContext::Module,
            CommentContext::Inline,
            CommentContext::Block,
            CommentContext::Documentation,
        ];

        for context in contexts {
            let content_type = ContentType::Comment {
                language: Language::Python,
                context: context.clone(),
            };

            // Test serialization
            let json = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&json).unwrap();

            if let ContentType::Comment {
                context: deserialized_context,
                ..
            } = deserialized
            {
                assert_eq!(
                    std::mem::discriminant(&context),
                    std::mem::discriminant(&deserialized_context),
                    "Context variant should match after serialization"
                );
            } else {
                panic!("Expected Comment content type");
            }
        }
    }

    #[test]
    fn test_document_format_variants() {
        let formats = vec![
            DocumentFormat::Markdown,
            DocumentFormat::RestructuredText,
            DocumentFormat::AsciiDoc,
            DocumentFormat::PlainText,
            DocumentFormat::Html,
        ];

        for format in formats {
            let content_type = ContentType::Documentation {
                format: format.clone(),
            };

            // Test serialization
            let json = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&json).unwrap();

            if let ContentType::Documentation {
                format: deserialized_format,
            } = deserialized
            {
                assert_eq!(
                    format, deserialized_format,
                    "Format should match after serialization"
                );
            } else {
                panic!("Expected Documentation content type");
            }
        }
    }

    #[test]
    fn test_config_format_variants() {
        let formats = vec![
            ConfigFormat::Json,
            ConfigFormat::Yaml,
            ConfigFormat::Toml,
            ConfigFormat::Ini,
            ConfigFormat::Properties,
            ConfigFormat::Env,
            ConfigFormat::Xml,
        ];

        for format in formats {
            let content_type = ContentType::Configuration {
                format: format.clone(),
            };

            // Test serialization
            let json = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&json).unwrap();

            if let ContentType::Configuration {
                format: deserialized_format,
            } = deserialized
            {
                assert_eq!(
                    format, deserialized_format,
                    "Format should match after serialization"
                );
            } else {
                panic!("Expected Configuration content type");
            }
        }
    }

    #[test]
    fn test_content_update_kinds() {
        let file_path = PathBuf::from("test.md");
        let old_path = PathBuf::from("old_test.md");

        let updates = vec![
            ContentUpdate {
                file_path: file_path.clone(),
                update_kind: ContentUpdateKind::Created,
                timestamp: SystemTime::now(),
            },
            ContentUpdate {
                file_path: file_path.clone(),
                update_kind: ContentUpdateKind::Modified,
                timestamp: SystemTime::now(),
            },
            ContentUpdate {
                file_path: file_path.clone(),
                update_kind: ContentUpdateKind::Deleted,
                timestamp: SystemTime::now(),
            },
            ContentUpdate {
                file_path: file_path.clone(),
                update_kind: ContentUpdateKind::Renamed {
                    old_path: old_path.clone(),
                },
                timestamp: SystemTime::now(),
            },
        ];

        for update in updates {
            // Test that all update kinds can be created and used
            assert_eq!(update.file_path, file_path);
            assert!(update.timestamp <= SystemTime::now());

            // Test the specific update kind
            match &update.update_kind {
                ContentUpdateKind::Created => { /* Content creation handled */ }
                ContentUpdateKind::Modified => { /* Content modification handled */ }
                ContentUpdateKind::Deleted => { /* Content deletion handled */ }
                ContentUpdateKind::Renamed {
                    old_path: renamed_old_path,
                } => {
                    assert_eq!(renamed_old_path, &old_path);
                }
            }
        }
    }
}
