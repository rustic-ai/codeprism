//! CodePrism - Incremental Polyglot Parser and Graph Builder
//!
//! This crate provides the core functionality for parsing source code across
//! multiple languages, building a universal AST, and maintaining a graph of
//! code relationships.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod content;
pub mod error;
pub mod graph;
pub mod indexer;
pub mod linkers;
pub mod parser;
pub mod patch;
pub mod pipeline;
pub mod repository;
pub mod scanner;
pub mod watcher;

pub use ast::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
pub use content::search::{ContentSearchManager, SearchQueryBuilder};
pub use content::{
    CommentContext, ConfigFormat, ContentChunk, ContentNode, ContentStats, ContentType,
    DocumentFormat, SearchQuery, SearchResult,
};
pub use error::{Error, Result};
pub use graph::{
    DynamicAttribute, GraphQuery, GraphStore, InheritanceFilter, InheritanceInfo,
    InheritanceRelation, PathResult, SymbolInfo,
};
pub use indexer::{
    BulkIndexer, IndexingConfig, IndexingProgressReporter, IndexingResult, IndexingStats,
    MemoryStats,
};
pub use linkers::{Linker, RestLinker, SqlLinker, SymbolResolver};
pub use parser::{LanguageParser, LanguageRegistry, ParseContext, ParseResult, ParserEngine};
pub use patch::{AstPatch, PatchBuilder};
pub use pipeline::{
    LoggingEventHandler, MonitoringPipeline, NoOpEventHandler, PipelineConfig, PipelineEvent,
    PipelineEventHandler, PipelineStats,
};
pub use repository::{HealthStatus, RepositoryConfig, RepositoryInfo, RepositoryManager};
pub use scanner::{
    DependencyMode, DiscoveredFile, NoOpProgressReporter, ProgressReporter, RepositoryScanner,
    ScanResult,
};
pub use watcher::{ChangeEvent, ChangeKind, FileWatcher};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::ast::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
    pub use crate::content::search::{ContentSearchManager, SearchQueryBuilder};
    pub use crate::content::{
        CommentContext, ConfigFormat, ContentChunk, ContentNode, ContentStats, ContentType,
        DocumentFormat, SearchQuery, SearchResult,
    };
    pub use crate::error::{Error, Result};
    pub use crate::graph::{
        DynamicAttribute, GraphQuery, GraphStore, InheritanceFilter, InheritanceInfo,
        InheritanceRelation, PathResult, SymbolInfo,
    };
    pub use crate::indexer::{
        BulkIndexer, IndexingConfig, IndexingProgressReporter, IndexingResult, IndexingStats,
        MemoryStats,
    };
    pub use crate::linkers::{Linker, RestLinker, SqlLinker, SymbolResolver};
    pub use crate::parser::{
        LanguageParser, LanguageRegistry, ParseContext, ParseResult, ParserEngine,
    };
    pub use crate::patch::{AstPatch, PatchBuilder};
    pub use crate::pipeline::{
        LoggingEventHandler, MonitoringPipeline, NoOpEventHandler, PipelineConfig, PipelineEvent,
        PipelineEventHandler, PipelineStats,
    };
    pub use crate::repository::{
        HealthStatus, RepositoryConfig, RepositoryInfo, RepositoryManager,
    };
    pub use crate::scanner::{
        DependencyMode, DiscoveredFile, NoOpProgressReporter, ProgressReporter, RepositoryScanner,
        ScanResult,
    };
    pub use crate::watcher::{ChangeEvent, ChangeKind, FileWatcher};
}
