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

/// REST API linker - simplified implementation to avoid memory issues
pub struct RestLinker;

impl Linker for RestLinker {
    fn name(&self) -> &str {
        "REST"
    }

    fn find_edges(&self, nodes: &[Node]) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find routes and functions
        let mut routes = Vec::new();
        let mut functions = Vec::new();

        for node in nodes {
            match node.kind {
                crate::ast::NodeKind::Route => routes.push(node),
                crate::ast::NodeKind::Function | crate::ast::NodeKind::Method => {
                    functions.push(node)
                }
                _ => {}
            }
        }

        // Simple matching - just check if route name contains function name or vice versa
        for route in routes {
            for function in &functions {
                if self.simple_name_match(&route.name, &function.name) {
                    edges.push(Edge::new(
                        route.id,
                        function.id,
                        crate::ast::EdgeKind::RoutesTo,
                    ));
                    break; // Only link to first match
                }
            }
        }

        Ok(edges)
    }
}

impl RestLinker {
    /// Simple name matching to avoid complex string operations
    fn simple_name_match(&self, route_name: &str, func_name: &str) -> bool {
        let route_lower = route_name.to_lowercase();
        let func_lower = func_name.to_lowercase();

        // Basic containment check
        route_lower.contains(&func_lower) || func_lower.contains(&route_lower)
    }
}

/// SQL query linker - simplified implementation
pub struct SqlLinker;

impl Linker for SqlLinker {
    fn name(&self) -> &str {
        "SQL"
    }

    fn find_edges(&self, nodes: &[Node]) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find SQL queries and potential tables
        let mut sql_queries = Vec::new();
        let mut table_candidates = Vec::new();

        for node in nodes {
            match node.kind {
                crate::ast::NodeKind::SqlQuery => sql_queries.push(node),
                crate::ast::NodeKind::Class | crate::ast::NodeKind::Variable => {
                    table_candidates.push(node)
                }
                _ => {}
            }
        }

        // Simple matching - check if query contains table/model names
        for query in sql_queries {
            for candidate in &table_candidates {
                if self.simple_table_match(&query.name, &candidate.name) {
                    edges.push(Edge::new(
                        query.id,
                        candidate.id,
                        crate::ast::EdgeKind::Reads,
                    ));
                }
            }
        }

        Ok(edges)
    }
}

impl SqlLinker {
    /// Simple table name matching
    fn simple_table_match(&self, query_text: &str, table_name: &str) -> bool {
        let query_lower = query_text.to_lowercase();
        let table_lower = table_name.to_lowercase();

        // Basic containment check
        query_lower.contains(&table_lower)
    }
}
