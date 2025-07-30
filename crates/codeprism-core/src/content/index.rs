//! Content indexing for fast search and retrieval
//!
//! This module provides efficient indexing of content chunks with support for
//! full-text search, pattern matching, and content type filtering.

use super::{
    ChunkId, ContentChunk, ContentNode, ContentStats, ContentType, ContentUpdate,
    ContentUpdateKind, SearchMatch, SearchQuery, SearchResult,
};

use anyhow::Result;
use dashmap::DashMap;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Content index for fast search and retrieval
pub struct ContentIndex {
    /// Content nodes indexed by file path
    nodes: DashMap<PathBuf, ContentNode>,
    /// Content chunks indexed by chunk ID
    chunks: DashMap<ChunkId, ContentChunk>,
    /// Token index for full-text search
    token_index: DashMap<String, HashSet<ChunkId>>,
    /// File pattern index for file discovery
    file_index: DashMap<String, HashSet<PathBuf>>,
    /// Content type index for filtering
    type_index: DashMap<String, HashSet<ChunkId>>,
    /// Statistics cache
    stats_cache: Arc<RwLock<Option<ContentStats>>>,
    /// Update listeners
    update_listeners: Arc<RwLock<Vec<Box<dyn ContentUpdateListener>>>>,
}

impl ContentIndex {
    /// Create a new content index
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
            chunks: DashMap::new(),
            token_index: DashMap::new(),
            file_index: DashMap::new(),
            type_index: DashMap::new(),
            stats_cache: Arc::new(RwLock::new(None)),
            update_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a content node to the index
    pub fn add_node(&self, node: ContentNode) -> Result<()> {
        let file_path = node.file_path.clone();

        // Remove existing node and its chunks
        if let Some(old_node) = self.nodes.get(&file_path) {
            for chunk in &old_node.chunks {
                self.remove_chunk_from_indexes(&chunk.id);
            }
        }

        // Index all chunks in the node
        for chunk in &node.chunks {
            self.add_chunk_to_indexes(chunk.clone())?;
        }

        // Index the file pattern
        self.index_file_pattern(&file_path);

        // Store the node
        self.nodes.insert(file_path.clone(), node);

        // Invalidate stats cache
        *self.stats_cache.write().unwrap() = None;

        // Notify listeners
        self.notify_update(ContentUpdate {
            file_path,
            update_kind: ContentUpdateKind::Modified,
            timestamp: SystemTime::now(),
        });

        Ok(())
    }

    /// Remove a content node from the index
    pub fn remove_node(&self, file_path: &Path) -> Result<()> {
        if let Some((_, node)) = self.nodes.remove(file_path) {
            // Remove all chunks from indexes
            for chunk in &node.chunks {
                self.remove_chunk_from_indexes(&chunk.id);
            }

            // Remove file pattern
            self.remove_file_pattern(file_path);

            // Invalidate stats cache
            *self.stats_cache.write().unwrap() = None;

            // Notify listeners
            self.notify_update(ContentUpdate {
                file_path: file_path.to_path_buf(),
                update_kind: ContentUpdateKind::Deleted,
                timestamp: SystemTime::now(),
            });
        }

        Ok(())
    }

    /// Get a content node by file path
    pub fn get_node(&self, file_path: &Path) -> Option<ContentNode> {
        self.nodes.get(file_path).map(|entry| entry.value().clone())
    }

    /// Get a content chunk by ID
    pub fn get_chunk(&self, chunk_id: &ChunkId) -> Option<ContentChunk> {
        self.chunks.get(chunk_id).map(|entry| entry.value().clone())
    }

    /// Search for content
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let mut seen_chunks = HashSet::new();

        // Prepare search regex if needed
        let search_regex = if query.use_regex {
            Some(Regex::new(&query.query)?)
        } else {
            None
        };

        // Get candidate chunks based on search strategy
        let candidate_chunks = if query.use_regex {
            self.search_by_regex(search_regex.as_ref().unwrap(), query)?
        } else {
            self.search_by_tokens(&query.query, query)?
        };

        // Process candidates and create results
        for chunk_id in candidate_chunks {
            if seen_chunks.contains(&chunk_id) {
                continue;
            }
            seen_chunks.insert(chunk_id);

            if let Some(chunk) = self.get_chunk(&chunk_id) {
                // Filter by content type
                if !query.content_types.is_empty()
                    && !self.matches_content_type(&chunk.content_type, &query.content_types)
                {
                    continue;
                }

                // Filter by file patterns
                if !self.matches_file_patterns(
                    &chunk.file_path,
                    &query.file_patterns,
                    &query.exclude_patterns,
                )? {
                    continue;
                }

                // Find matches within the chunk
                let matches = self.find_matches_in_chunk(&chunk, query, &search_regex)?;
                if !matches.is_empty() {
                    let score = self.calculate_relevance_score(&chunk, &matches, query);
                    results.push(SearchResult {
                        chunk: chunk.clone(),
                        score,
                        matches,
                        related_nodes: chunk.related_nodes.clone(),
                    });
                }
            }

            if results.len() >= query.max_results {
                break;
            }
        }

        // Sort by relevance score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Find files by pattern
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let pattern_regex = Regex::new(pattern)?;
        let mut matching_files = Vec::new();

        for entry in self.nodes.iter() {
            let file_path = entry.key();
            if pattern_regex.is_match(&file_path.to_string_lossy()) {
                matching_files.push(file_path.clone());
            }
        }

        Ok(matching_files)
    }

    /// Get content statistics
    pub fn get_stats(&self) -> ContentStats {
        // Try to use cached stats
        if let Ok(cache) = self.stats_cache.read() {
            if let Some(stats) = cache.as_ref() {
                return stats.clone();
            }
        }

        // Compute fresh stats
        let stats = self.compute_stats();

        // Cache the stats
        if let Ok(mut cache) = self.stats_cache.write() {
            *cache = Some(stats.clone());
        }

        stats
    }

    /// Add content update listener
    pub fn add_update_listener(&self, listener: Box<dyn ContentUpdateListener>) {
        if let Ok(mut listeners) = self.update_listeners.write() {
            listeners.push(listener);
        }
    }

    /// Clear all content from the index
    pub fn clear(&self) {
        self.nodes.clear();
        self.chunks.clear();
        self.token_index.clear();
        self.file_index.clear();
        self.type_index.clear();
        *self.stats_cache.write().unwrap() = None;
    }

    // Private helper methods

    /// Add a chunk to all relevant indexes
    fn add_chunk_to_indexes(&self, chunk: ContentChunk) -> Result<()> {
        let chunk_id = chunk.id;

        // Add to token index
        for token in &chunk.tokens {
            self.token_index
                .entry(token.clone())
                .or_default()
                .insert(chunk_id);
        }

        // Add to content type index
        let type_key = self.content_type_to_string(&chunk.content_type);
        self.type_index
            .entry(type_key)
            .or_default()
            .insert(chunk_id);

        // Store the chunk
        self.chunks.insert(chunk_id, chunk);

        Ok(())
    }

    /// Remove a chunk from all indexes
    fn remove_chunk_from_indexes(&self, chunk_id: &ChunkId) {
        // Remove from chunk storage
        if let Some((_, chunk)) = self.chunks.remove(chunk_id) {
            // Remove from token index
            for token in &chunk.tokens {
                if let Some(mut token_set) = self.token_index.get_mut(token) {
                    token_set.remove(chunk_id);
                    if token_set.is_empty() {
                        drop(token_set);
                        self.token_index.remove(token);
                    }
                }
            }

            // Remove from content type index
            let type_key = self.content_type_to_string(&chunk.content_type);
            if let Some(mut type_set) = self.type_index.get_mut(&type_key) {
                type_set.remove(chunk_id);
                if type_set.is_empty() {
                    drop(type_set);
                    self.type_index.remove(&type_key);
                }
            }
        }
    }

    /// Index file pattern for discovery
    fn index_file_pattern(&self, file_path: &Path) {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Index by filename
        self.file_index
            .entry(file_name.to_lowercase())
            .or_default()
            .insert(file_path.to_path_buf());

        // Index by extension
        if !extension.is_empty() {
            self.file_index
                .entry(format!("*.{}", extension.to_lowercase()))
                .or_default()
                .insert(file_path.to_path_buf());
        }

        // Index by full path components
        for component in file_path.components() {
            if let Some(component_str) = component.as_os_str().to_str() {
                self.file_index
                    .entry(component_str.to_lowercase())
                    .or_default()
                    .insert(file_path.to_path_buf());
            }
        }
    }

    /// Remove file pattern from index
    fn remove_file_pattern(&self, file_path: &Path) {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Remove from filename index
        if let Some(mut file_set) = self.file_index.get_mut(&file_name.to_lowercase()) {
            file_set.remove(file_path);
            if file_set.is_empty() {
                drop(file_set);
                self.file_index.remove(&file_name.to_lowercase());
            }
        }

        // Remove from extension index
        if !extension.is_empty() {
            let ext_key = format!("*.{}", extension.to_lowercase());
            if let Some(mut ext_set) = self.file_index.get_mut(&ext_key) {
                ext_set.remove(file_path);
                if ext_set.is_empty() {
                    drop(ext_set);
                    self.file_index.remove(&ext_key);
                }
            }
        }
    }

    /// Search by token matching
    fn search_by_tokens(&self, query: &str, _search_query: &SearchQuery) -> Result<Vec<ChunkId>> {
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if query_tokens.is_empty() {
            return Ok(Vec::new());
        }

        let mut result_chunks: Option<HashSet<ChunkId>> = None;

        // Find intersection of chunks containing all query tokens
        for token in &query_tokens {
            if let Some(chunk_set) = self.token_index.get(token) {
                let chunk_ids: HashSet<ChunkId> = chunk_set.iter().copied().collect();
                result_chunks = Some(match result_chunks {
                    None => chunk_ids,
                    Some(existing) => existing.intersection(&chunk_ids).copied().collect(),
                });
            } else {
                // If any token is not found, no results
                return Ok(Vec::new());
            }
        }

        Ok(result_chunks.unwrap_or_default().into_iter().collect())
    }

    /// Search by regex pattern
    fn search_by_regex(&self, regex: &Regex, search_query: &SearchQuery) -> Result<Vec<ChunkId>> {
        let mut matching_chunks = Vec::new();

        for entry in self.chunks.iter() {
            let chunk = entry.value();
            let content = if search_query.case_sensitive {
                &chunk.content
            } else {
                &chunk.content.to_lowercase()
            };

            if regex.is_match(content) {
                matching_chunks.push(chunk.id);
            }
        }

        Ok(matching_chunks)
    }

    /// Find matches within a chunk
    fn find_matches_in_chunk(
        &self,
        chunk: &ContentChunk,
        query: &SearchQuery,
        regex: &Option<Regex>,
    ) -> Result<Vec<SearchMatch>> {
        let mut matches = Vec::new();
        let content = if query.case_sensitive {
            chunk.content.clone()
        } else {
            chunk.content.to_lowercase()
        };

        let search_term = if query.case_sensitive {
            query.query.clone()
        } else {
            query.query.to_lowercase()
        };

        if let Some(regex) = regex {
            // Regex search
            for regex_match in regex.find_iter(&content) {
                let line_info = self.calculate_line_info(&content, regex_match.start());
                let search_match = SearchMatch {
                    text: regex_match.as_str().to_string(),
                    position: regex_match.start(),
                    line_number: line_info.0,
                    column_number: line_info.1,
                    context_before: if query.include_context {
                        self.get_context_before(&content, regex_match.start(), query.context_lines)
                    } else {
                        None
                    },
                    context_after: if query.include_context {
                        self.get_context_after(&content, regex_match.end(), query.context_lines)
                    } else {
                        None
                    },
                };
                matches.push(search_match);
            }
        } else {
            // Simple text search
            let mut start = 0;
            while let Some(pos) = content[start..].find(&search_term) {
                let absolute_pos = start + pos;
                let line_info = self.calculate_line_info(&content, absolute_pos);
                let search_match = SearchMatch {
                    text: search_term.clone(),
                    position: absolute_pos,
                    line_number: line_info.0,
                    column_number: line_info.1,
                    context_before: if query.include_context {
                        self.get_context_before(&content, absolute_pos, query.context_lines)
                    } else {
                        None
                    },
                    context_after: if query.include_context {
                        self.get_context_after(
                            &content,
                            absolute_pos + search_term.len(),
                            query.context_lines,
                        )
                    } else {
                        None
                    },
                };
                matches.push(search_match);
                start = absolute_pos + 1;
            }
        }

        Ok(matches)
    }

    /// Calculate line and column information for a position
    fn calculate_line_info(&self, content: &str, position: usize) -> (usize, usize) {
        let before_position = &content[..position.min(content.len())];
        let line_number = before_position.lines().count();
        let column_number = before_position
            .lines()
            .last()
            .map(|line| line.len() + 1)
            .unwrap_or(1);
        (line_number, column_number)
    }

    /// Get context lines before a position
    fn get_context_before(
        &self,
        content: &str,
        position: usize,
        context_lines: usize,
    ) -> Option<String> {
        if context_lines == 0 {
            return None;
        }

        let lines: Vec<&str> = content.lines().collect();
        let (line_number, _) = self.calculate_line_info(content, position);

        if line_number == 0 {
            return None;
        }

        let start_line = line_number.saturating_sub(context_lines + 1);
        let end_line = line_number.saturating_sub(1);

        if start_line >= lines.len() || end_line >= lines.len() || start_line > end_line {
            return None;
        }

        Some(lines[start_line..=end_line].join("\n"))
    }

    /// Get context lines after a position
    fn get_context_after(
        &self,
        content: &str,
        position: usize,
        context_lines: usize,
    ) -> Option<String> {
        if context_lines == 0 {
            return None;
        }

        let lines: Vec<&str> = content.lines().collect();
        let (line_number, _) = self.calculate_line_info(content, position);

        let start_line = line_number;
        let end_line = (start_line + context_lines).min(lines.len().saturating_sub(1));

        if start_line >= lines.len() || start_line > end_line {
            return None;
        }

        Some(lines[start_line..=end_line].join("\n"))
    }

    /// Calculate relevance score for a search result
    fn calculate_relevance_score(
        &self,
        chunk: &ContentChunk,
        matches: &[SearchMatch],
        _query: &SearchQuery,
    ) -> f32 {
        if matches.is_empty() {
            return 0.0;
        }

        // Base score from content type relevance (0.2-0.8)
        let type_score = match &chunk.content_type {
            ContentType::Documentation { .. } => 0.8,
            ContentType::Comment { context, .. } => match context {
                super::CommentContext::Documentation => 0.7,
                super::CommentContext::Function { .. } => 0.6,
                super::CommentContext::Class { .. } => 0.6,
                _ => 0.4,
            },
            ContentType::Code { .. } => 0.5,
            ContentType::Configuration { .. } => 0.4,
            ContentType::PlainText => 0.2,
        };

        // Match frequency bonus (0.1 per match)
        let match_bonus = matches.len() as f32 * 0.1;

        // Calculate final score and normalize to 0.0-1.0 range
        (type_score + match_bonus).min(1.0)
    }

    /// Check if content type matches query filters
    fn matches_content_type(
        &self,
        content_type: &ContentType,
        allowed_types: &[ContentType],
    ) -> bool {
        allowed_types
            .iter()
            .any(|allowed| std::mem::discriminant(content_type) == std::mem::discriminant(allowed))
    }

    /// Check if file path matches include/exclude patterns
    fn matches_file_patterns(
        &self,
        file_path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<bool> {
        let path_str = file_path.to_string_lossy();

        // Check exclude patterns first
        for pattern in exclude_patterns {
            let regex_pattern = self.glob_to_regex(pattern);
            let regex = Regex::new(&regex_pattern)?;
            if regex.is_match(&path_str) {
                return Ok(false);
            }
        }

        // If no include patterns, include by default
        if include_patterns.is_empty() {
            return Ok(true);
        }

        // Check include patterns
        for pattern in include_patterns {
            let regex_pattern = self.glob_to_regex(pattern);
            let regex = Regex::new(&regex_pattern)?;
            if regex.is_match(&path_str) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Convert glob pattern to regex pattern
    fn glob_to_regex(&self, glob: &str) -> String {
        let mut regex = String::new();
        regex.push('^');

        for ch in glob.chars() {
            match ch {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                '.' => regex.push_str("\\."),
                '+' => regex.push_str("\\+"),
                '^' => regex.push_str("\\^"),
                '$' => regex.push_str("\\$"),
                '(' => regex.push_str("\\("),
                ')' => regex.push_str("\\)"),
                '[' => regex.push_str("\\["),
                ']' => regex.push_str("\\]"),
                '{' => regex.push_str("\\{"),
                '}' => regex.push_str("\\}"),
                '|' => regex.push_str("\\|"),
                '\\' => regex.push_str("\\\\"),
                c => regex.push(c),
            }
        }

        regex.push('$');
        regex
    }

    /// Convert content type to string for indexing
    fn content_type_to_string(&self, content_type: &ContentType) -> String {
        match content_type {
            ContentType::Code { language } => format!("code:{language:?}"),
            ContentType::Documentation { format } => format!("doc:{format:?}"),
            ContentType::Configuration { format } => format!("config:{format:?}"),
            ContentType::Comment { language, context } => {
                format!("comment:{language:?}:{context:?}")
            }
            ContentType::PlainText => "text".to_string(),
        }
    }

    /// Compute fresh statistics
    fn compute_stats(&self) -> ContentStats {
        let mut stats = ContentStats::new();

        stats.total_files = self.nodes.len();
        stats.total_chunks = self.chunks.len();

        // Count unique tokens
        stats.total_tokens = self.token_index.len();

        // Count content by type
        for entry in self.type_index.iter() {
            let type_name = entry.key().clone();
            let chunk_count = entry.value().len();
            stats.content_by_type.insert(type_name, chunk_count);
        }

        // File size distribution
        for entry in self.nodes.iter() {
            let node = entry.value();
            let size_bucket = match node.file_size {
                0..=1024 => "small (0-1KB)",
                1025..=10240 => "medium (1-10KB)",
                10241..=102400 => "large (10-100KB)",
                _ => "very_large (>100KB)",
            };
            *stats
                .size_distribution
                .entry(size_bucket.to_string())
                .or_insert(0) += 1;
        }

        stats.computed_at = SystemTime::now();
        stats
    }

    /// Notify update listeners
    fn notify_update(&self, update: ContentUpdate) {
        if let Ok(listeners) = self.update_listeners.read() {
            for listener in listeners.iter() {
                listener.on_content_update(&update);
            }
        }
    }
}

impl Default for ContentIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for content update listeners
pub trait ContentUpdateListener: Send + Sync {
    /// Called when content is updated
    fn on_content_update(&self, update: &ContentUpdate);
}

/// Simple logging update listener
pub struct LoggingUpdateListener;

impl ContentUpdateListener for LoggingUpdateListener {
    fn on_content_update(&self, update: &ContentUpdate) {
        eprintln!(
            "Content updated: {:?} at {:?}",
            update.file_path, update.timestamp
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;
    use crate::content::ChunkId;
    use crate::{ConfigFormat, DocumentFormat};
    use std::path::Path;

    fn create_test_chunk(
        file_path: &Path,
        content: &str,
        content_type: ContentType,
        chunk_index: usize,
    ) -> ContentChunk {
        let span = Span::new(0, content.len(), 1, 1, 1, content.len());
        ContentChunk::new(
            file_path.to_path_buf(),
            content_type,
            content.to_string(),
            span,
            chunk_index,
        )
    }

    fn create_test_node(file_path: &Path, chunks: Vec<ContentChunk>) -> ContentNode {
        let mut node = ContentNode::new(file_path.to_path_buf(), chunks[0].content_type.clone());
        for chunk in chunks {
            node.add_chunk(chunk);
        }
        node.file_size = 1000; // Dummy size
        node
    }

    #[test]
    fn test_content_index_creation() {
        let index = ContentIndex::new();

        // Test default implementation
        let _index_default = ContentIndex::default();

        // Initially empty
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);
    }

    #[test]
    fn test_add_and_get_node() {
        let index = ContentIndex::new();
        let file_path = Path::new("test.md");

        // Create test content
        let chunk = create_test_chunk(
            file_path,
            "# Test Document\n\nThis is a test.",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            0,
        );
        let node = create_test_node(file_path, vec![chunk]);

        // Add node to index
        let result = index.add_node(node.clone());
        assert!(result.is_ok(), "Adding valid content node should succeed");

        // Retrieve the node and verify its content
        let retrieved_node = index.get_node(file_path);
        assert!(
            retrieved_node.is_some(),
            "Should be able to retrieve added node"
        );
        let retrieved_node = retrieved_node.unwrap();
        assert_eq!(
            retrieved_node.file_path, file_path,
            "Retrieved node should have correct file path"
        );
        assert_eq!(
            retrieved_node.chunks.len(),
            1,
            "Retrieved node should have 1 chunk"
        );

        // Verify chunk content was preserved
        assert_eq!(
            retrieved_node.chunks[0].content, "# Test Document\n\nThis is a test.",
            "Chunk content should be preserved"
        );
        assert!(
            matches!(
                retrieved_node.chunks[0].content_type,
                ContentType::Documentation { .. }
            ),
            "Content type should be preserved"
        );

        // Verify index statistics updated
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 1, "Stats should show 1 file");
        assert_eq!(stats.total_chunks, 1, "Stats should show 1 chunk");
    }

    #[test]
    fn test_add_node_replaces_existing() {
        let index = ContentIndex::new();
        let file_path = Path::new("test.md");

        // Add first version
        let chunk1 = create_test_chunk(
            file_path,
            "Original content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            0,
        );
        let node1 = create_test_node(file_path, vec![chunk1]);
        let _ = index.add_node(node1);

        // Add updated version
        let chunk2 = create_test_chunk(
            file_path,
            "Updated content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node2 = create_test_node(file_path, vec![chunk2]);
        let _ = index.add_node(node2);

        // Should have the updated content
        let retrieved_node = index.get_node(file_path).unwrap();
        assert_eq!(retrieved_node.chunks[0].content, "Updated content");
    }

    #[test]
    fn test_remove_node() {
        let index = ContentIndex::new();
        let file_path = Path::new("test.md");

        // Add a node
        let chunk = create_test_chunk(
            file_path,
            "Test content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            0,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Verify it exists
        assert!(
            index.get_node(file_path).is_some(),
            "Node should exist after adding"
        );
        let retrieved_node = index.get_node(file_path).unwrap();
        assert_eq!(
            retrieved_node.file_path, file_path,
            "Retrieved node should have correct path"
        );
        assert!(
            !retrieved_node.chunks.is_empty(),
            "Retrieved node should have chunks"
        );

        // Remove it
        let result = index.remove_node(file_path);
        assert!(result.is_ok(), "Operation should succeed");

        // Verify it's gone
        assert!(index.get_node(file_path).is_none());
    }

    #[test]
    fn test_get_chunk() {
        let index = ContentIndex::new();
        let file_path = Path::new("test.md");

        let chunk = create_test_chunk(
            file_path,
            "Test content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            42,
        );
        let chunk_id = chunk.id;
        let node = create_test_node(file_path, vec![chunk]);

        let _ = index.add_node(node);

        // Should be able to retrieve chunk by ID
        let retrieved_chunk = index.get_chunk(&chunk_id);
        assert!(retrieved_chunk.is_some(), "Should have value");
        assert_eq!(retrieved_chunk.unwrap().content, "Test content");

        // Non-existent chunk should return None
        let fake_chunk_id = ChunkId::new(Path::new("nonexistent.md"), 9999, &[0u8; 32]);
        let non_existent = index.get_chunk(&fake_chunk_id);
        assert!(non_existent.is_none(), "Should be none");
    }

    #[test]
    fn test_simple_text_search() {
        let index = ContentIndex::new();

        // Add some test content
        let file1 = Path::new("doc1.md");
        let chunk1 = create_test_chunk(
            file1,
            "This is a test document about programming",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node1 = create_test_node(file1, vec![chunk1]);
        let _ = index.add_node(node1);

        let file2 = Path::new("doc2.md");
        let chunk2 = create_test_chunk(
            file2,
            "Another document for testing purposes",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            2,
        );
        let node2 = create_test_node(file2, vec![chunk2]);
        let _ = index.add_node(node2);

        // Search for "document" (which should be in both)
        let search_query = SearchQuery {
            query: "document".to_string(),
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert!(!!results.is_empty(), "Should not be empty");

        // Should find matches in both documents
        let result_contents: Vec<_> = results.iter().map(|r| &r.chunk.content).collect();
        assert!(result_contents
            .iter()
            .any(|content| content.contains("programming")));
        assert!(result_contents
            .iter()
            .any(|content| content.contains("testing")));
    }

    #[test]
    fn test_regex_search() {
        let index = ContentIndex::new();

        // Add content with email addresses
        let file_path = Path::new("contacts.md");
        let chunk = create_test_chunk(
            file_path,
            "Contact John at john@example.com or Mary at mary@test.org",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Search with regex pattern
        let search_query = SearchQuery {
            query: r"\b\w+@\w+\.\w+\b".to_string(),
            use_regex: true,
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert!(!!results.is_empty(), "Should not be empty");

        // Should find email matches
        let result = &results[0];
        assert!(!!result.matches.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_search_with_content_type_filter() {
        let index = ContentIndex::new();

        // Add different content types
        let md_file = Path::new("doc.md");
        let md_chunk = create_test_chunk(
            md_file,
            "Documentation content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let md_node = create_test_node(md_file, vec![md_chunk]);
        let _ = index.add_node(md_node);

        let json_file = Path::new("config.json");
        let json_chunk = create_test_chunk(
            json_file,
            r#"{"config": "content"}"#,
            ContentType::Configuration {
                format: ConfigFormat::Json,
            },
            2,
        );
        let json_node = create_test_node(json_file, vec![json_chunk]);
        let _ = index.add_node(json_node);

        // Search only in documentation
        let search_query = SearchQuery {
            query: "content".to_string(),
            content_types: vec![ContentType::Documentation {
                format: DocumentFormat::Markdown,
            }],
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 1, "Should have 1 items");
        assert!(results[0].chunk.content.contains("Documentation"));
    }

    #[test]
    fn test_search_with_file_patterns() {
        let index = ContentIndex::new();

        // Add files with different extensions
        let md_file = Path::new("test.md");
        let md_chunk = create_test_chunk(
            md_file,
            "Markdown content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let md_node = create_test_node(md_file, vec![md_chunk]);
        let _ = index.add_node(md_node);

        let txt_file = Path::new("test.txt");
        let txt_chunk = create_test_chunk(
            txt_file,
            "Text content",
            ContentType::Documentation {
                format: DocumentFormat::PlainText,
            },
            2,
        );
        let txt_node = create_test_node(txt_file, vec![txt_chunk]);
        let _ = index.add_node(txt_node);

        // Search only in .md files
        let search_query = SearchQuery {
            query: "content".to_string(),
            file_patterns: vec!["*.md".to_string()],
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 1, "Should have 1 items");
        assert!(results[0].chunk.content.contains("Markdown"));
    }

    #[test]
    fn test_search_with_exclude_patterns() {
        let index = ContentIndex::new();

        // Add test files
        let md_file = Path::new("test.md");
        let md_chunk = create_test_chunk(
            md_file,
            "Markdown content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let md_node = create_test_node(md_file, vec![md_chunk]);
        let _ = index.add_node(md_node);

        let tmp_file = Path::new("temp.tmp");
        let tmp_chunk = create_test_chunk(
            tmp_file,
            "Temporary content",
            ContentType::Documentation {
                format: DocumentFormat::PlainText,
            },
            2,
        );
        let tmp_node = create_test_node(tmp_file, vec![tmp_chunk]);
        let _ = index.add_node(tmp_node);

        // Search excluding .tmp files
        let search_query = SearchQuery {
            query: "content".to_string(),
            exclude_patterns: vec!["*.tmp".to_string()],
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 1, "Should have 1 items");
        assert!(results[0].chunk.content.contains("Markdown"));
    }

    #[test]
    fn test_search_with_context() {
        let index = ContentIndex::new();

        let file_path = Path::new("test.md");
        let content = "Line 1\nLine 2 with target\nLine 3\nLine 4";
        let chunk = create_test_chunk(
            file_path,
            content,
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Search with context
        let search_query = SearchQuery {
            query: "target".to_string(),
            include_context: true,
            context_lines: 1,
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert!(!!results.is_empty(), "Should not be empty");

        let result = &results[0];
        assert!(!!result.matches.is_empty(), "Should not be empty");

        // Should have context before and after
        let search_match = &result.matches[0];
        assert!(search_match.context_before.is_some(), "Should have value");
        assert!(search_match.context_after.is_some(), "Should have value");
    }

    #[test]
    fn test_search_case_sensitive() {
        let index = ContentIndex::new();

        let file_path = Path::new("test.md");
        let chunk = create_test_chunk(
            file_path,
            "Test with UPPERCASE and lowercase",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Case sensitive search
        let search_query = SearchQuery {
            query: "UPPERCASE".to_string(),
            case_sensitive: true,
            max_results: 10,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert!(!!results.is_empty(), "Should not be empty");

        // Should not match lowercase
        let search_query_lower = SearchQuery {
            query: "uppercase".to_string(),
            case_sensitive: true,
            max_results: 10,
            ..Default::default()
        };

        let results_lower = index.search(&search_query_lower).unwrap();
        assert!(!results_lower.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_search_max_results() {
        let index = ContentIndex::new();

        // Add multiple documents with the same term
        for i in 0..10 {
            let file_path = PathBuf::from(format!("doc{i}.md"));
            let chunk = create_test_chunk(
                &file_path,
                &format!("Document {i} contains the search term"),
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                i,
            );
            let node = create_test_node(&file_path, vec![chunk]);
            let _ = index.add_node(node);
        }

        // Search with max results limit
        let search_query = SearchQuery {
            query: "search".to_string(),
            max_results: 3,
            ..Default::default()
        };

        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 3, "Should have 3 items");
    }

    #[test]
    fn test_find_files() {
        let index = ContentIndex::new();

        // Add files with different patterns
        let files = ["test_one.md", "test_two.md", "other.txt", "config.json"];
        for (i, file_name) in files.iter().enumerate() {
            let file_path = Path::new(file_name);
            let chunk = create_test_chunk(
                file_path,
                &format!("Content {i}"),
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                i,
            );
            let node = create_test_node(file_path, vec![chunk]);
            let _ = index.add_node(node);
        }

        // Find markdown files
        let md_files = index.find_files(r"\.md$").unwrap();
        assert_eq!(md_files.len(), 2, "Should have 2 items");

        // Find test files
        let test_files = index.find_files(r"test_").unwrap();
        assert_eq!(test_files.len(), 2, "Should have 2 items");

        // Find all files
        let all_files = index.find_files(r".*").unwrap();
        assert_eq!(all_files.len(), 4, "Should have 4 items");
    }

    #[test]
    fn test_content_stats() {
        let index = ContentIndex::new();

        // Initially empty
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);

        // Add some content
        let file1 = Path::new("doc1.md");
        let chunk1 = create_test_chunk(
            file1,
            "First document",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node1 = create_test_node(file1, vec![chunk1]);
        let _ = index.add_node(node1);

        let file2 = Path::new("doc2.md");
        let chunk2a = create_test_chunk(
            file2,
            "Second document first chunk",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            2,
        );
        let chunk2b = create_test_chunk(
            file2,
            "Second document second chunk",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            3,
        );
        let node2 = create_test_node(file2, vec![chunk2a, chunk2b]);
        let _ = index.add_node(node2);

        // Check updated stats
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_chunks, 3);
    }

    #[test]
    fn test_content_update_listeners() {
        struct TestListener {
            updates: Arc<std::sync::Mutex<Vec<ContentUpdate>>>,
        }

        impl ContentUpdateListener for TestListener {
            fn on_content_update(&self, update: &ContentUpdate) {
                self.updates.lock().unwrap().push(update.clone());
            }
        }

        let index = ContentIndex::new();
        let updates = Arc::new(std::sync::Mutex::new(Vec::new()));
        let listener = TestListener {
            updates: updates.clone(),
        };

        index.add_update_listener(Box::new(listener));

        // Add a node
        let file_path = Path::new("test.md");
        let chunk = create_test_chunk(
            file_path,
            "Test content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Should have received update notification
        let updates = updates.lock().unwrap();
        assert_eq!(updates.len(), 1, "Should have 1 items");
        assert_eq!(updates[0].file_path, file_path);
        assert!(matches!(
            updates[0].update_kind,
            ContentUpdateKind::Modified
        ));
    }

    #[test]
    fn test_clear() {
        let index = ContentIndex::new();

        // Add some content
        let file_path = Path::new("test.md");
        let chunk = create_test_chunk(
            file_path,
            "Test content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Verify content exists and validate its properties
        assert!(
            index.get_node(file_path).is_some(),
            "Node should exist after adding"
        );
        let retrieved_node = index.get_node(file_path).unwrap();
        assert_eq!(
            retrieved_node.file_path, file_path,
            "Retrieved node should have correct file path"
        );
        assert!(
            !retrieved_node.chunks.is_empty(),
            "Retrieved node should have chunks"
        );
        assert_eq!(
            retrieved_node.chunks[0].content, "Test content for clear",
            "Chunk should have correct content"
        );

        let stats = index.get_stats();
        assert!(
            stats.total_files > 0,
            "Stats should show files after adding content"
        );
        assert_eq!(stats.total_files, 1, "Should have exactly 1 file");

        // Clear all content
        index.clear();

        // Verify content is gone
        assert!(index.get_node(file_path).is_none());
        let stats = index.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);
    }

    #[test]
    fn test_invalid_regex_search() {
        let index = ContentIndex::new();

        // Add some content
        let file_path = Path::new("test.md");
        let chunk = create_test_chunk(
            file_path,
            "Test content",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );
        let node = create_test_node(file_path, vec![chunk]);
        let _ = index.add_node(node);

        // Try search with invalid regex
        let search_query = SearchQuery {
            query: "[invalid".to_string(),
            use_regex: true,
            max_results: 10,
            ..Default::default()
        };

        let result = index.search(&search_query);
        assert!(result.is_err());
    }

    #[test]
    fn test_logging_update_listener() {
        let listener = LoggingUpdateListener;
        let update = ContentUpdate {
            file_path: PathBuf::from("test.md"),
            update_kind: ContentUpdateKind::Modified,
            timestamp: SystemTime::now(),
        };

        // Should not panic
        listener.on_content_update(&update);
    }

    #[test]
    fn test_line_info_calculation() {
        let index = ContentIndex::new();

        let content = "Line 1\nLine 2\nLine 3 with text\nLine 4";
        let position = content.find("text").unwrap();

        let (line, column) = index.calculate_line_info(content, position);
        assert_eq!(line, 3); // Line number (1-indexed)
        assert!(column > 1); // Column position
    }

    #[test]
    fn test_context_extraction() {
        let index = ContentIndex::new();

        let content = "Line 1\nLine 2\nLine 3 target\nLine 4\nLine 5";
        let position = content.find("target").unwrap();

        // Test context before
        let context_before = index.get_context_before(content, position, 1);
        assert!(context_before.is_some(), "Should have value");
        assert!(context_before.unwrap().contains("Line 2"));

        // Test context after
        let context_after = index.get_context_after(content, position + 6, 1);
        assert!(context_after.is_some(), "Should have value");
        assert!(context_after.unwrap().contains("Line 4"));

        // Test with zero context lines
        let no_context = index.get_context_before(content, position, 0);
        assert!(no_context.is_none(), "Should be none");
    }

    #[test]
    fn test_relevance_score_calculation() {
        let index = ContentIndex::new();

        let file_path = Path::new("test.md");
        let chunk = create_test_chunk(
            file_path,
            "Test document with multiple test occurrences",
            ContentType::Documentation {
                format: DocumentFormat::Markdown,
            },
            1,
        );

        let matches = vec![
            SearchMatch {
                text: "test".to_string(),
                position: 0,
                line_number: 1,
                column_number: 1,
                context_before: None,
                context_after: None,
            },
            SearchMatch {
                text: "test".to_string(),
                position: 30,
                line_number: 1,
                column_number: 31,
                context_before: None,
                context_after: None,
            },
        ];

        let query = SearchQuery {
            query: "test".to_string(),
            ..Default::default()
        };

        let score = index.calculate_relevance_score(&chunk, &matches, &query);
        assert!(score > 0.0);

        // More matches should give higher score
        let single_match = vec![matches[0].clone()];
        let single_score = index.calculate_relevance_score(&chunk, &single_match, &query);
        assert!(score > single_score);
    }
}
