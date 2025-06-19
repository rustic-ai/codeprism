//! Symbol resolver for creating cross-file edges
//!
//! This module resolves imports, function calls, and other references across files
//! to create a complete dependency graph after initial parsing.

use crate::ast::{Edge, EdgeKind, Node, NodeId, NodeKind};
use crate::error::Result;
use crate::graph::GraphStore;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Symbol resolver for cross-file linking
pub struct SymbolResolver {
    graph: Arc<GraphStore>,
    /// Index of importable symbols by module path
    module_symbols: HashMap<String, Vec<NodeId>>,
    /// Index of symbols by qualified name (module.symbol)
    qualified_symbols: HashMap<String, NodeId>,
    /// Import resolution cache
    #[allow(dead_code)]
    import_cache: HashMap<String, String>,
}

impl SymbolResolver {
    /// Create a new symbol resolver
    pub fn new(graph: Arc<GraphStore>) -> Self {
        Self {
            graph,
            module_symbols: HashMap::new(),
            qualified_symbols: HashMap::new(),
            import_cache: HashMap::new(),
        }
    }

    /// Resolve all cross-file relationships
    pub fn resolve_all(&mut self) -> Result<Vec<Edge>> {
        let mut new_edges = Vec::new();

        // Build symbol indices
        self.build_symbol_indices()?;

        // Resolve imports
        new_edges.extend(self.resolve_imports()?);

        // Resolve function calls
        new_edges.extend(self.resolve_function_calls()?);

        // Resolve class instantiations
        new_edges.extend(self.resolve_class_instantiations()?);

        // Resolve inheritance relationships
        new_edges.extend(self.resolve_inheritance()?);

        Ok(new_edges)
    }

    /// Build indices of available symbols for resolution
    fn build_symbol_indices(&mut self) -> Result<()> {
        // Get all nodes and organize by module
        for (file_path, node_ids) in self.graph.iter_file_index() {
            let module_name = self.file_path_to_module_name(&file_path);

            for node_id in node_ids {
                if let Some(node) = self.graph.get_node(&node_id) {
                    match node.kind {
                        NodeKind::Class | NodeKind::Function | NodeKind::Variable => {
                            // Add to module symbols
                            self.module_symbols
                                .entry(module_name.clone())
                                .or_default()
                                .push(node_id);

                            // Add to qualified symbols
                            let qualified_name = format!("{}.{}", module_name, node.name);
                            self.qualified_symbols.insert(qualified_name, node_id);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolve import statements to create edges to imported symbols
    fn resolve_imports(&mut self) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find all import nodes
        let import_nodes = self.graph.get_nodes_by_kind(NodeKind::Import);

        for import_node in import_nodes {
            edges.extend(self.resolve_single_import(&import_node)?);
        }

        Ok(edges)
    }

    /// Resolve a single import node
    fn resolve_single_import(&mut self, import_node: &Node) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Parse import statement
        let import_parts = self.parse_import_statement(&import_node.name);

        for (module_path, symbol_name) in import_parts {
            // Find the target symbol
            if let Some(target_id) = self.find_symbol_in_module(&module_path, &symbol_name) {
                // Create import edge
                edges.push(Edge::new(import_node.id, target_id, EdgeKind::Imports));
            }
        }

        Ok(edges)
    }

    /// Resolve function calls to actual function definitions
    fn resolve_function_calls(&mut self) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find all call nodes
        let call_nodes = self.graph.get_nodes_by_kind(NodeKind::Call);

        for call_node in call_nodes {
            if let Some(target_id) = self.resolve_call_target(&call_node)? {
                edges.push(Edge::new(call_node.id, target_id, EdgeKind::Calls));
            }
        }

        Ok(edges)
    }

    /// Resolve class instantiations (calls to __init__)
    fn resolve_class_instantiations(&mut self) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find call nodes that might be class instantiations
        let call_nodes = self.graph.get_nodes_by_kind(NodeKind::Call);

        for call_node in call_nodes {
            // Check if this is a class name (first letter uppercase)
            if call_node
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
            {
                if let Some(class_id) = self.find_class_by_name(&call_node.name) {
                    // Find the __init__ method of this class
                    if let Some(init_id) = self.find_method_in_class(class_id, "__init__") {
                        edges.push(Edge::new(call_node.id, init_id, EdgeKind::Calls));
                    }
                }
            }
        }

        Ok(edges)
    }

    /// Parse import statement to extract module and symbol names
    fn parse_import_statement(&self, import_name: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();

        // Handle different import patterns
        if import_name.contains('.') {
            // Module.symbol or complex import
            let parts: Vec<&str> = import_name.split('.').collect();
            if parts.len() >= 2 {
                let module = parts[..parts.len() - 1].join(".");
                let symbol = parts.last().unwrap().to_string();
                results.push((module, symbol));
            }
        } else {
            // Simple module import - all exportable symbols
            if let Some(symbols) = self.module_symbols.get(import_name) {
                for symbol_id in symbols {
                    if let Some(node) = self.graph.get_node(symbol_id) {
                        results.push((import_name.to_string(), node.name.clone()));
                    }
                }
            }
        }

        results
    }

    /// Find a symbol in a specific module
    fn find_symbol_in_module(&self, module_path: &str, symbol_name: &str) -> Option<NodeId> {
        // Try qualified name first
        let qualified_name = format!("{}.{}", module_path, symbol_name);
        if let Some(node_id) = self.qualified_symbols.get(&qualified_name) {
            return Some(*node_id);
        }

        // Try by module and name
        if let Some(symbol_ids) = self.module_symbols.get(module_path) {
            for symbol_id in symbol_ids {
                if let Some(node) = self.graph.get_node(symbol_id) {
                    if node.name == symbol_name {
                        return Some(*symbol_id);
                    }
                }
            }
        }

        None
    }

    /// Resolve the target of a function call
    fn resolve_call_target(&self, call_node: &Node) -> Result<Option<NodeId>> {
        // Get the file where this call is made
        let calling_file = &call_node.file;

        // First check for local functions in the same file
        let file_nodes = self.graph.get_nodes_in_file(calling_file);
        for node in &file_nodes {
            if matches!(node.kind, NodeKind::Function | NodeKind::Method)
                && node.name == call_node.name
            {
                return Ok(Some(node.id));
            }
        }

        // Check imported functions
        // Find import nodes in the same file
        for node in &file_nodes {
            if node.kind == NodeKind::Import {
                let import_parts = self.parse_import_statement(&node.name);
                for (module_path, symbol_name) in import_parts {
                    if symbol_name == call_node.name {
                        if let Some(target_id) =
                            self.find_symbol_in_module(&module_path, &symbol_name)
                        {
                            return Ok(Some(target_id));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find a class by name (could be local or imported)
    fn find_class_by_name(&self, class_name: &str) -> Option<NodeId> {
        // Search all class nodes for matching name
        let class_nodes = self.graph.get_nodes_by_kind(NodeKind::Class);
        for node in class_nodes {
            if node.name == class_name {
                return Some(node.id);
            }
        }
        None
    }

    /// Find a method within a specific class
    fn find_method_in_class(&self, class_id: NodeId, method_name: &str) -> Option<NodeId> {
        // Get the class node to find its file
        if let Some(class_node) = self.graph.get_node(&class_id) {
            let file_nodes = self.graph.get_nodes_in_file(&class_node.file);

            for node in file_nodes {
                if node.kind == NodeKind::Method && node.name == method_name {
                    // Check if this method is within the class span
                    if node.span.start_line >= class_node.span.start_line
                        && node.span.end_line <= class_node.span.end_line
                    {
                        return Some(node.id);
                    }
                }
            }
        }
        None
    }

    /// Convert file path to module name
    fn file_path_to_module_name(&self, file_path: &Path) -> String {
        // Convert file path to Python module name
        if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            if stem == "__init__" {
                // For __init__.py, use parent directory name
                if let Some(parent) = file_path.parent() {
                    if let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) {
                        return parent_name.to_string();
                    }
                }
            }

            // Convert path separators to dots for module name
            let path_str = file_path.to_string_lossy();
            let module_path = path_str
                .replace(['/', '\\'], ".")
                .replace(".py", "")
                .replace(".__init__", "");

            return module_path;
        }

        "unknown".to_string()
    }

    /// Resolve inheritance relationships (class extends parent class)
    fn resolve_inheritance(&mut self) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find all class nodes
        let class_nodes = self.graph.get_nodes_by_kind(NodeKind::Class);

        for class_node in class_nodes {
            // Get all outgoing Call edges from this class (inheritance is represented as Call)
            let outgoing_edges = self.graph.get_outgoing_edges(&class_node.id);

            for edge in outgoing_edges {
                if edge.kind == EdgeKind::Calls {
                    // Check if the target node represents a base class reference
                    if let Some(call_node) = self.graph.get_node(&edge.target) {
                        if call_node.kind == NodeKind::Call {
                            // Try to resolve this call to an actual class
                            if let Some(target_class_id) =
                                self.resolve_base_class_name(&call_node.name, &class_node.file)
                            {
                                // Create inheritance edge: child class -> parent class
                                edges.push(Edge::new(
                                    class_node.id,
                                    target_class_id,
                                    EdgeKind::Calls,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(edges)
    }

    /// Resolve a base class name to its actual class node
    fn resolve_base_class_name(
        &self,
        class_name: &str,
        calling_file: &std::path::PathBuf,
    ) -> Option<NodeId> {
        // First check for local classes in the same file
        let file_nodes = self.graph.get_nodes_in_file(calling_file);
        for node in &file_nodes {
            if node.kind == NodeKind::Class && node.name == class_name {
                return Some(node.id);
            }
        }

        // Check imported classes
        // Find import nodes in the same file and see if they import this class
        for node in &file_nodes {
            if node.kind == NodeKind::Import {
                let import_parts = self.parse_import_statement(&node.name);
                for (module_path, symbol_name) in import_parts {
                    if symbol_name == class_name {
                        if let Some(target_id) =
                            self.find_symbol_in_module(&module_path, &symbol_name)
                        {
                            // Verify it's actually a class
                            if let Some(target_node) = self.graph.get_node(&target_id) {
                                if target_node.kind == NodeKind::Class {
                                    return Some(target_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: search all classes by name
        let all_class_nodes = self.graph.get_nodes_by_kind(NodeKind::Class);
        for node in all_class_nodes {
            if node.name == class_name {
                return Some(node.id);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name_conversion() {
        let resolver = SymbolResolver::new(Arc::new(GraphStore::new()));

        let path1 = PathBuf::from("src/rustic_ai/core/guild/agent.py");
        assert_eq!(
            resolver.file_path_to_module_name(&path1),
            "src.rustic_ai.core.guild.agent"
        );

        let path2 = PathBuf::from("src/utils/__init__.py");
        assert_eq!(resolver.file_path_to_module_name(&path2), "utils");
    }

    #[test]
    fn test_import_parsing() {
        let resolver = SymbolResolver::new(Arc::new(GraphStore::new()));

        let parts = resolver.parse_import_statement("rustic_ai.core.guild.Agent");
        assert_eq!(parts.len(), 1);
        assert_eq!(
            parts[0],
            ("rustic_ai.core.guild".to_string(), "Agent".to_string())
        );
    }
}
