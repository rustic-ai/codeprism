//! Content indexing for fast search and retrieval
//!
//! This module provides efficient indexing of content chunks with support for
//! full-text search, pattern matching, and content type filtering.

use super::{
    ContentChunk, ContentNode, ContentStats, ContentType, ContentUpdate, ContentUpdateKind,
    ChunkId, SearchQuery, SearchResult, SearchMatch,
};
use crate::ast::{Language, NodeId};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use regex::Regex;
use std::collections::{HashMap, HashSet, BTreeMap};
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
            self.search_by_regex(&search_regex.as_ref().unwrap(), query)?
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
                if !query.content_types.is_empty() && 
                   !self.matches_content_type(&chunk.content_type, &query.content_types) {
                    continue;
                }
                
                // Filter by file patterns
                if !self.matches_file_patterns(&chunk.file_path, &query.file_patterns, &query.exclude_patterns)? {
                    continue;
                }
                
                // Find matches within the chunk
                let matches = self.find_matches_in_chunk(&chunk, query, &search_regex)?;
                if !matches.is_empty() {
                    let score = self.calculate_relevance_score(&chunk, &matches, query);
                    results.push(SearchResult {
                        chunk,
                        score,
                        matches,
                        related_nodes: Vec::new(), // TODO: Populate from chunk.related_nodes
                    });
                }
            }
            
            if results.len() >= query.max_results {
                break;
            }
        }
        
        // Sort by relevance score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
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
            self.token_index.entry(token.clone())
                .or_insert_with(HashSet::new)
                .insert(chunk_id);
        }
        
        // Add to content type index
        let type_key = self.content_type_to_string(&chunk.content_type);
        self.type_index.entry(type_key)
            .or_insert_with(HashSet::new)
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
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // Index by filename
        self.file_index.entry(file_name.to_lowercase())
            .or_insert_with(HashSet::new)
            .insert(file_path.to_path_buf());
        
        // Index by extension
        if !extension.is_empty() {
            self.file_index.entry(format!("*.{}", extension.to_lowercase()))
                .or_insert_with(HashSet::new)
                .insert(file_path.to_path_buf());
        }
        
        // Index by full path components
        for component in file_path.components() {
            if let Some(component_str) = component.as_os_str().to_str() {
                self.file_index.entry(component_str.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(file_path.to_path_buf());
            }
        }
    }
    
    /// Remove file pattern from index
    fn remove_file_pattern(&self, file_path: &Path) {
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        let extension = file_path.extension()
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
    fn search_by_tokens(&self, query: &str, search_query: &SearchQuery) -> Result<Vec<ChunkId>> {
        let query_tokens: Vec<String> = query.to_lowercase()
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
                        self.get_context_after(&content, absolute_pos + search_term.len(), query.context_lines)
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
        let column_number = before_position.lines().last()
            .map(|line| line.len() + 1)
            .unwrap_or(1);
        (line_number, column_number)
    }
    
    /// Get context lines before a position
    fn get_context_before(&self, content: &str, position: usize, context_lines: usize) -> Option<String> {
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
    fn get_context_after(&self, content: &str, position: usize, context_lines: usize) -> Option<String> {
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
        query: &SearchQuery,
    ) -> f32 {
        if matches.is_empty() {
            return 0.0;
        }
        
        let mut score = 0.0;
        
        // Base score from number of matches
        score += matches.len() as f32 * 0.1;
        
        // Boost score based on content type relevance
        score += match &chunk.content_type {
            ContentType::Documentation { .. } => 1.0,
            ContentType::Comment { context, .. } => match context {
                super::CommentContext::Documentation => 0.9,
                super::CommentContext::Function { .. } => 0.8,
                super::CommentContext::Class { .. } => 0.8,
                _ => 0.5,
            },
            ContentType::Code { .. } => 0.7,
            ContentType::Configuration { .. } => 0.6,
            ContentType::PlainText => 0.4,
        };
        
        // Normalize score to 0.0-1.0 range
        score.min(1.0)
    }
    
    /// Check if content type matches query filters
    fn matches_content_type(&self, content_type: &ContentType, allowed_types: &[ContentType]) -> bool {
        allowed_types.iter().any(|allowed| {
            std::mem::discriminant(content_type) == std::mem::discriminant(allowed)
        })
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
            let regex = Regex::new(pattern)?;
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
            let regex = Regex::new(pattern)?;
            if regex.is_match(&path_str) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Convert content type to string for indexing
    fn content_type_to_string(&self, content_type: &ContentType) -> String {
        match content_type {
            ContentType::Code { language } => format!("code:{:?}", language),
            ContentType::Documentation { format } => format!("doc:{:?}", format),
            ContentType::Configuration { format } => format!("config:{:?}", format),
            ContentType::Comment { language, context } => format!("comment:{:?}:{:?}", language, context),
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
            *stats.size_distribution.entry(size_bucket.to_string()).or_insert(0) += 1;
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
        tracing::debug!(
            "Content update: {:?} - {}",
            update.update_kind,
            update.file_path.display()
        );
    }
} 