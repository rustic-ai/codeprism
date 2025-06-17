//! GCore - Incremental Polyglot Parser and Graph Builder
//!
//! This crate provides the core functionality for parsing source code across
//! multiple languages, building a universal AST, and maintaining a graph of
//! code relationships.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod content;
pub mod error;
pub mod indexer;
pub mod linkers;
pub mod parser;
pub mod patch;
pub mod pipeline;
pub mod repository;
pub mod scanner;
pub mod watcher;
pub mod graph;

pub use ast::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
pub use error::{Error, Result};
pub use indexer::{BulkIndexer, IndexingConfig, IndexingProgressReporter, IndexingResult, IndexingStats, MemoryStats};
pub use linkers::{Linker, RestLinker, SqlLinker, SymbolResolver};
pub use parser::{LanguageParser, LanguageRegistry, ParseContext, ParseResult, ParserEngine};
pub use patch::{AstPatch, PatchBuilder};
pub use pipeline::{LoggingEventHandler, MonitoringPipeline, NoOpEventHandler, PipelineConfig, PipelineEvent, PipelineEventHandler, PipelineStats};
pub use repository::{HealthStatus, RepositoryConfig, RepositoryInfo, RepositoryManager};
pub use scanner::{DiscoveredFile, NoOpProgressReporter, ProgressReporter, RepositoryScanner, ScanResult, DependencyMode};
pub use watcher::{ChangeEvent, ChangeKind, FileWatcher};
pub use graph::{GraphStore, GraphQuery, PathResult, SymbolInfo};
pub use content::{
    ContentChunk, ContentNode, ContentStats, ContentType, 
    SearchQuery, SearchResult, DocumentFormat, ConfigFormat, CommentContext
};
pub use content::search::{ContentSearchManager, SearchQueryBuilder};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::ast::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
    pub use crate::error::{Error, Result};
    pub use crate::indexer::{BulkIndexer, IndexingConfig, IndexingProgressReporter, IndexingResult, IndexingStats, MemoryStats};
    pub use crate::linkers::{Linker, RestLinker, SqlLinker, SymbolResolver};
    pub use crate::parser::{
        LanguageParser, LanguageRegistry, ParseContext, ParseResult, ParserEngine,
    };
    pub use crate::patch::{AstPatch, PatchBuilder};
    pub use crate::pipeline::{LoggingEventHandler, MonitoringPipeline, NoOpEventHandler, PipelineConfig, PipelineEvent, PipelineEventHandler, PipelineStats};
    pub use crate::repository::{HealthStatus, RepositoryConfig, RepositoryInfo, RepositoryManager};
    pub use crate::scanner::{DiscoveredFile, NoOpProgressReporter, ProgressReporter, RepositoryScanner, ScanResult, DependencyMode};
    pub use crate::watcher::{ChangeEvent, ChangeKind, FileWatcher};
    pub use crate::graph::{GraphStore, GraphQuery, PathResult, SymbolInfo};
    pub use crate::content::{
        ContentChunk, ContentNode, ContentStats, ContentType, 
        SearchQuery, SearchResult, DocumentFormat, ConfigFormat, CommentContext
    };
    pub use crate::content::search::{ContentSearchManager, SearchQueryBuilder};
}
