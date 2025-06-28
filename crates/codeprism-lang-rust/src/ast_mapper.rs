//! AST mapper for converting Tree-sitter CST to Universal AST for Rust

use crate::error::Result;
use crate::types::{Edge, EdgeKind, Language, Node, NodeKind, Span};

use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Tree, TreeCursor};

/// AST mapper for Rust
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
            "function_item" => self.handle_function(cursor)?,
            "struct_item" => self.handle_struct(cursor)?,
            "enum_item" => self.handle_enum(cursor)?,
            "trait_item" => self.handle_trait(cursor)?,
            "impl_item" => self.handle_impl(cursor)?,
            "use_declaration" => self.handle_use_declaration(cursor)?,
            "mod_item" => self.handle_mod_item(cursor)?,
            "const_item" => self.handle_const_item(cursor)?,
            "static_item" => self.handle_static_item(cursor)?,
            "call_expression" => self.handle_call_expression(cursor)?,
            "let_declaration" => self.handle_let_declaration(cursor)?,
            _ => {} // Skip other node types for now
        }

        Ok(())
    }

    /// Handle function definitions
    fn handle_function(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_function_name(&node)?;
        let kind = if self.is_inside_impl(&node) {
            NodeKind::Method
        } else {
            NodeKind::Function
        };

        let func_node = Node::new(
            &self.repo_id,
            kind,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), func_node.id);

        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, func_node.id, EdgeKind::Contains));
        }

        self.nodes.push(func_node);
        Ok(())
    }

    /// Handle struct definitions
    fn handle_struct(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let struct_node = Node::new(
            &self.repo_id,
            NodeKind::Struct,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), struct_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, struct_node.id, EdgeKind::Contains));
        }

        self.nodes.push(struct_node);
        Ok(())
    }

    /// Handle enum definitions
    fn handle_enum(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let enum_node = Node::new(
            &self.repo_id,
            NodeKind::Enum,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), enum_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, enum_node.id, EdgeKind::Contains));
        }

        self.nodes.push(enum_node);
        Ok(())
    }

    /// Handle trait definitions
    fn handle_trait(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let trait_node = Node::new(
            &self.repo_id,
            NodeKind::Trait,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), trait_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, trait_node.id, EdgeKind::Contains));
        }

        self.nodes.push(trait_node);
        Ok(())
    }

    /// Handle implementation blocks
    fn handle_impl(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let impl_name = self.extract_impl_info(&node)?;

        let impl_node = Node::new(
            &self.repo_id,
            NodeKind::Impl,
            impl_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), impl_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, impl_node.id, EdgeKind::Contains));
        }

        self.nodes.push(impl_node);
        Ok(())
    }

    /// Handle use declarations
    fn handle_use_declaration(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let use_path = self.extract_use_path(&node);

        let use_node = Node::new(
            &self.repo_id,
            NodeKind::Use,
            use_path,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), use_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, use_node.id, EdgeKind::Uses));
        }

        self.nodes.push(use_node);
        Ok(())
    }

    /// Handle module declarations
    fn handle_mod_item(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let mod_node = Node::new(
            &self.repo_id,
            NodeKind::Mod,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), mod_node.id);

        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, mod_node.id, EdgeKind::Contains));
        }

        self.nodes.push(mod_node);
        Ok(())
    }

    /// Handle const items
    fn handle_const_item(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let const_node = Node::new(
            &self.repo_id,
            NodeKind::Const,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), const_node.id);

        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, const_node.id, EdgeKind::Contains));
        }

        self.nodes.push(const_node);
        Ok(())
    }

    /// Handle static items
    fn handle_static_item(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let name = self.extract_identifier_name(&node, "name")?;

        let static_node = Node::new(
            &self.repo_id,
            NodeKind::Static,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), static_node.id);

        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, static_node.id, EdgeKind::Contains));
        }

        self.nodes.push(static_node);
        Ok(())
    }

    /// Handle function call expressions
    fn handle_call_expression(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let function_name = self.extract_call_target(&node);

        let call_node = Node::new(
            &self.repo_id,
            NodeKind::Call,
            function_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), call_node.id);

        if let Some(caller_id) = self.find_containing_function_id(&node) {
            self.edges
                .push(Edge::new(caller_id, call_node.id, EdgeKind::Calls));
        }

        self.nodes.push(call_node);
        Ok(())
    }

    /// Handle let declarations (variable assignments)
    fn handle_let_declaration(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        if let Some(pattern_node) = node.child_by_field_name("pattern") {
            if pattern_node.kind() == "identifier" {
                let var_name = self.get_node_text(&pattern_node);
                let var_node = Node::new(
                    &self.repo_id,
                    NodeKind::Variable,
                    var_name,
                    self.language,
                    self.file_path.clone(),
                    span,
                );

                if let Some(parent_id) = self.find_enclosing_scope_id(&node) {
                    self.edges
                        .push(Edge::new(parent_id, var_node.id, EdgeKind::Writes));
                }

                self.nodes.push(var_node);
            }
        }

        Ok(())
    }

    // Helper methods

    /// Extract function name from function_item node
    fn extract_function_name(&self, node: &tree_sitter::Node) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            Ok(self.get_node_text(&name_node))
        } else {
            Ok("anonymous".to_string())
        }
    }

    /// Extract identifier name by field name
    fn extract_identifier_name(&self, node: &tree_sitter::Node, field: &str) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name(field) {
            Ok(self.get_node_text(&name_node))
        } else {
            Ok("unnamed".to_string())
        }
    }

    /// Extract impl information
    fn extract_impl_info(&self, node: &tree_sitter::Node) -> Result<String> {
        if let Some(trait_node) = node.child_by_field_name("trait") {
            let trait_name = self.get_node_text(&trait_node);
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = self.get_node_text(&type_node);
                Ok(format!("{} for {}", trait_name, type_name))
            } else {
                Ok(trait_name)
            }
        } else if let Some(type_node) = node.child_by_field_name("type") {
            let type_name = self.get_node_text(&type_node);
            Ok(type_name)
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Extract use path
    fn extract_use_path(&self, node: &tree_sitter::Node) -> String {
        if let Some(argument_node) = node.child_by_field_name("argument") {
            self.get_node_text(&argument_node)
        } else {
            self.get_node_text(node)
        }
    }

    /// Extract call target (function being called)
    fn extract_call_target(&self, node: &tree_sitter::Node) -> String {
        if let Some(function_node) = node.child_by_field_name("function") {
            match function_node.kind() {
                "identifier" => self.get_node_text(&function_node),
                "field_expression" => {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        self.get_node_text(&field_node)
                    } else {
                        self.get_node_text(&function_node)
                    }
                }
                _ => self.get_node_text(&function_node),
            }
        } else {
            "unknown_call".to_string()
        }
    }

    /// Check if node is inside an impl block
    fn is_inside_impl(&self, node: &tree_sitter::Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "impl_item" {
                return true;
            }
            current = parent.parent();
        }
        false
    }

    /// Get text content of a node
    fn get_node_text(&self, node: &tree_sitter::Node) -> String {
        node.utf8_text(self.source.as_bytes())
            .unwrap_or("ERROR")
            .to_string()
    }

    /// Find module node ID (the root module)
    fn find_module_node_id(&self) -> Option<crate::types::NodeId> {
        self.nodes
            .first()
            .filter(|n| matches!(n.kind, NodeKind::Module))
            .map(|n| n.id)
    }

    /// Find parent scope ID for a node
    fn find_parent_scope_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if let Some(&node_id) = self.node_map.get(&parent.id()) {
                return Some(node_id);
            }
            current = parent.parent();
        }
        self.find_module_node_id()
    }

    /// Find enclosing scope ID for a node
    fn find_enclosing_scope_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if matches!(
                parent.kind(),
                "function_item" | "impl_item" | "mod_item" | "source_file"
            ) {
                if let Some(&node_id) = self.node_map.get(&parent.id()) {
                    return Some(node_id);
                }
            }
            current = parent.parent();
        }
        self.find_module_node_id()
    }

    /// Find containing function ID for a node
    fn find_containing_function_id(
        &self,
        node: &tree_sitter::Node,
    ) -> Option<crate::types::NodeId> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "function_item" {
                if let Some(&node_id) = self.node_map.get(&parent.id()) {
                    return Some(node_id);
                }
            }
            current = parent.parent();
        }
        None
    }
}
