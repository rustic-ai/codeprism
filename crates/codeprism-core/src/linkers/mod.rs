//! Cross-language linkers for detecting relationships between different languages

use crate::ast::{Edge, Node};
use crate::error::Result;

pub mod symbol_resolver;

pub use symbol_resolver::SymbolResolver;

/// Trait for cross-language linkers
pub trait Linker: Send + Sync {
    /// Name of the linker
    fn name(&self) -> &str;

    /// Find cross-language edges
    fn find_edges(&self, nodes: &[Node]) -> Result<Vec<Edge>>;
}

/// REST API linker
pub struct RestLinker;

impl Linker for RestLinker {
    fn name(&self) -> &str {
        "REST"
    }

    fn find_edges(&self, _nodes: &[Node]) -> Result<Vec<Edge>> {
        // TODO: Implement REST path to controller linking
        Ok(Vec::new())
    }
}

/// SQL query linker
pub struct SqlLinker;

impl Linker for SqlLinker {
    fn name(&self) -> &str {
        "SQL"
    }

    fn find_edges(&self, _nodes: &[Node]) -> Result<Vec<Edge>> {
        // TODO: Implement SQL query to table linking
        Ok(Vec::new())
    }
}
