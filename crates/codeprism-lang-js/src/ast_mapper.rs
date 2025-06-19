//! AST mapper for converting Tree-sitter CST to Universal AST

use crate::error::Result;
use crate::types::{Edge, EdgeKind, Language, Node, NodeKind, Span};
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Tree, TreeCursor};

/// AST mapper for JavaScript/TypeScript
pub struct AstMapper {
    repo_id: String,
    file_path: PathBuf,
    language: Language,
    source: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    /// Map from tree-sitter node ID to our NodeId for edge creation
    node_map: HashMap<usize, crate::types::NodeId>,
}

impl AstMapper {
    /// Create a new AST mapper
    pub fn new(repo_id: &str, file_path: PathBuf, language: Language, source: &str) -> Self {
        Self {
            repo_id: repo_id.to_string(),
            file_path,
            language,
            source: source.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    /// Extract nodes and edges from the tree
    pub fn extract(mut self, tree: &Tree) -> Result<(Vec<Node>, Vec<Edge>)> {
        let mut cursor = tree.walk();

        // Create module node for the file
        let module_node = self.create_module_node(&cursor)?;
        self.nodes.push(module_node);

        // Walk the tree and extract nodes
        self.walk_tree(&mut cursor)?;

        Ok((self.nodes, self.edges))
    }

    /// Create a module node for the file
    fn create_module_node(&mut self, cursor: &TreeCursor) -> Result<Node> {
        let root = cursor.node();
        let span = Span::from_node(&root);

        let module_name = self
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module")
            .to_string();

        let node = Node::new(
            &self.repo_id,
            NodeKind::Module,
            module_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(root.id(), node.id);
        Ok(node)
    }

    /// Walk the tree and extract nodes
    fn walk_tree(&mut self, cursor: &mut TreeCursor) -> Result<()> {
        self.visit_node(cursor)?;

        if cursor.goto_first_child() {
            loop {
                self.walk_tree(cursor)?;
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        Ok(())
    }

    /// Visit a single node and extract information
    fn visit_node(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            // Function declarations
            "function_declaration" | "arrow_function" | "method_definition" => {
                self.handle_function(cursor)?;
            }

            // Skip "function" as it's part of function_declaration
            "function" => {
                // This is the function keyword, not a function declaration
            }

            // Class declarations
            "class_declaration" | "class" => {
                self.handle_class(cursor)?;
            }

            // Variable declarations
            "variable_declaration" | "lexical_declaration" => {
                self.handle_variable_declaration(cursor)?;
            }

            // Function calls
            "call_expression" => {
                self.handle_call_expression(cursor)?;
            }

            // Imports
            "import_statement" | "import_declaration" => {
                self.handle_import(cursor)?;
            }

            // Exports
            "export_statement" | "export_declaration" => {
                self.handle_export(cursor)?;
            }

            _ => {
                // Skip other node types for now
            }
        }

        Ok(())
    }

    /// Handle function declarations
    fn handle_function(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract function name
        let name = self.extract_function_name(&node)?;

        // Determine if it's a method or function
        let kind = if node.kind() == "method_definition" {
            NodeKind::Method
        } else {
            NodeKind::Function
        };

        // Extract signature if available
        let signature = self.extract_function_signature(&node);

        let mut func_node = Node::new(
            &self.repo_id,
            kind,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        if let Some(sig) = signature {
            func_node.signature = Some(sig);
        }

        // Store the mapping
        self.node_map.insert(node.id(), func_node.id);

        // Add edge from module/class to function
        if let Some(parent_id) = self.find_parent_node_id(&node) {
            self.edges
                .push(Edge::new(parent_id, func_node.id, EdgeKind::Calls));
        }

        self.nodes.push(func_node);
        Ok(())
    }

    /// Handle class declarations
    fn handle_class(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract class name
        let name = self.extract_class_name(&node)?;

        let class_node = Node::new(
            &self.repo_id,
            NodeKind::Class,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Store the mapping
        self.node_map.insert(node.id(), class_node.id);

        // Add edge from module to class
        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, class_node.id, EdgeKind::Calls));
        }

        self.nodes.push(class_node);
        Ok(())
    }

    /// Handle variable declarations
    fn handle_variable_declaration(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();

        // Find all variable declarators
        let mut child_cursor = node.walk();
        if child_cursor.goto_first_child() {
            loop {
                if child_cursor.node().kind() == "variable_declarator" {
                    self.handle_variable_declarator(&child_cursor)?;
                }
                if !child_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a single variable declarator
    fn handle_variable_declarator(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract variable name
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node);

            let var_node = Node::new(
                &self.repo_id,
                NodeKind::Variable,
                name,
                self.language,
                self.file_path.clone(),
                span,
            );

            // Store the mapping
            self.node_map.insert(node.id(), var_node.id);

            // Add edge from parent scope
            if let Some(parent_id) = self.find_parent_scope_id(&node) {
                self.edges
                    .push(Edge::new(parent_id, var_node.id, EdgeKind::Writes));
            }

            self.nodes.push(var_node);
        }

        Ok(())
    }

    /// Handle function calls
    fn handle_call_expression(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract function being called
        if let Some(function_node) = node.child_by_field_name("function") {
            let function_name = self.get_node_text(&function_node);

            let call_node = Node::new(
                &self.repo_id,
                NodeKind::Call,
                function_name.clone(),
                self.language,
                self.file_path.clone(),
                span,
            );

            // Store the mapping
            self.node_map.insert(node.id(), call_node.id);

            // Add edge from caller to call
            if let Some(caller_id) = self.find_containing_function_id(&node) {
                self.edges
                    .push(Edge::new(caller_id, call_node.id, EdgeKind::Calls));
            }

            self.nodes.push(call_node);
        }

        Ok(())
    }

    /// Handle import statements
    fn handle_import(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract import source
        if let Some(source_node) = node.child_by_field_name("source") {
            let source_text = self.get_node_text(&source_node);
            let source = source_text.trim_matches(|c| c == '"' || c == '\'');

            let import_node = Node::new(
                &self.repo_id,
                NodeKind::Import,
                source.to_string(),
                self.language,
                self.file_path.clone(),
                span,
            );

            // Store the mapping
            self.node_map.insert(node.id(), import_node.id);

            // Add edge from module to import
            if let Some(module_id) = self.find_module_node_id() {
                self.edges
                    .push(Edge::new(module_id, import_node.id, EdgeKind::Imports));
            }

            self.nodes.push(import_node);
        }

        Ok(())
    }

    /// Handle export statements
    fn handle_export(&mut self, _cursor: &TreeCursor) -> Result<()> {
        // TODO: Implement export handling
        Ok(())
    }

    /// Extract function name from a function node
    fn extract_function_name(&self, node: &tree_sitter::Node) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            Ok(self.get_node_text(&name_node))
        } else {
            // Anonymous function
            Ok("<anonymous>".to_string())
        }
    }

    /// Extract class name from a class node
    fn extract_class_name(&self, node: &tree_sitter::Node) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            Ok(self.get_node_text(&name_node))
        } else {
            // Anonymous class
            Ok("<anonymous>".to_string())
        }
    }

    /// Extract function signature (for TypeScript)
    fn extract_function_signature(&self, _node: &tree_sitter::Node) -> Option<String> {
        // TODO: Implement proper signature extraction for TypeScript
        None
    }

    /// Get text content of a node
    fn get_node_text(&self, node: &tree_sitter::Node) -> String {
        node.utf8_text(self.source.as_bytes())
            .unwrap_or("<error>")
            .to_string()
    }

    /// Find the module node ID
    fn find_module_node_id(&self) -> Option<crate::types::NodeId> {
        self.nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Module))
            .map(|n| n.id)
    }

    /// Find parent node ID (module or class)
    fn find_parent_node_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut parent = node.parent();
        while let Some(p) = parent {
            if let Some(id) = self.node_map.get(&p.id()) {
                return Some(*id);
            }
            parent = p.parent();
        }
        self.find_module_node_id()
    }

    /// Find parent scope ID (function, method, or module)
    fn find_parent_scope_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut parent = node.parent();
        while let Some(p) = parent {
            match p.kind() {
                "function_declaration" | "function" | "arrow_function" | "method_definition" => {
                    if let Some(id) = self.node_map.get(&p.id()) {
                        return Some(*id);
                    }
                }
                _ => {}
            }
            parent = p.parent();
        }
        self.find_module_node_id()
    }

    /// Find containing function ID
    fn find_containing_function_id(
        &self,
        node: &tree_sitter::Node,
    ) -> Option<crate::types::NodeId> {
        let mut parent = node.parent();
        while let Some(p) = parent {
            match p.kind() {
                "function_declaration" | "function" | "arrow_function" | "method_definition" => {
                    if let Some(id) = self.node_map.get(&p.id()) {
                        return Some(*id);
                    }
                }
                _ => {}
            }
            parent = p.parent();
        }
        self.find_module_node_id()
    }
}
