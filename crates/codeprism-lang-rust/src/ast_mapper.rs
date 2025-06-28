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
            "attribute_item" | "inner_attribute_item" => self.handle_attribute(cursor)?,
            "macro_invocation" => self.handle_macro_invocation(cursor)?,
            "lifetime_parameter" | "lifetime" => self.handle_lifetime_node(cursor)?,
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

        // Extract detailed function signature including lifetimes and ownership info
        let signature = self.extract_detailed_function_signature(&node);
        let metadata = self.extract_function_metadata(&node);

        let func_node = Node::new(
            &self.repo_id,
            kind,
            name,
            self.language,
            self.file_path.clone(),
            span,
        )
        .with_signature(signature.unwrap_or_default())
        .with_metadata(metadata);

        self.node_map.insert(node.id(), func_node.id);

        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, func_node.id, EdgeKind::Contains));
        }

        // Extract ownership and borrowing patterns from function parameters
        self.extract_function_ownership_patterns(&node, func_node.id)?;

        // Extract lifetime annotations
        self.extract_lifetime_annotations(&node, func_node.id)?;

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
        let signature = self.extract_trait_signature(&node);
        let metadata = self.extract_trait_metadata(&node);

        let trait_node = Node::new(
            &self.repo_id,
            NodeKind::Trait,
            name,
            self.language,
            self.file_path.clone(),
            span,
        )
        .with_signature(signature.unwrap_or_default())
        .with_metadata(metadata);

        self.node_map.insert(node.id(), trait_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, trait_node.id, EdgeKind::Contains));
        }

        // Extract trait bounds (supertraits)
        self.extract_trait_bounds(&node, trait_node.id)?;

        // Extract associated types and constants
        self.extract_associated_items(&node, trait_node.id)?;

        self.nodes.push(trait_node);
        Ok(())
    }

    /// Handle implementation blocks
    fn handle_impl(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let (impl_name, trait_name, type_name) = self.extract_detailed_impl_info(&node)?;
        let metadata = self.extract_impl_metadata(&node, &trait_name, &type_name);

        let impl_node = Node::new(
            &self.repo_id,
            NodeKind::Impl,
            impl_name,
            self.language,
            self.file_path.clone(),
            span,
        )
        .with_metadata(metadata);

        self.node_map.insert(node.id(), impl_node.id);

        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, impl_node.id, EdgeKind::Contains));
        }

        // Create trait implementation edge if this is a trait impl
        if let Some(trait_name) = trait_name {
            self.create_trait_implementation_edge(impl_node.id, &trait_name, &type_name);
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

    /// Handle attributes (especially derive attributes)
    fn handle_attribute(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let attr_name = self.extract_attribute_name(&node);
        let metadata = self.extract_attribute_metadata(&node);

        let attr_node = Node::new(
            &self.repo_id,
            NodeKind::Attribute,
            attr_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        )
        .with_metadata(metadata);

        self.node_map.insert(node.id(), attr_node.id);

        // Handle derive attributes specially
        if attr_name.starts_with("derive") {
            self.handle_derive_attribute(&node, attr_node.id)?;
        }

        self.nodes.push(attr_node);
        Ok(())
    }

    /// Handle macro invocations
    fn handle_macro_invocation(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let macro_name = self.extract_macro_invocation_name(&node);

        let macro_call_node = Node::new(
            &self.repo_id,
            NodeKind::Call,
            format!("{}!", macro_name),
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), macro_call_node.id);

        // Create edge from containing function/scope
        if let Some(caller_id) = self.find_containing_function_id(&node) {
            self.edges
                .push(Edge::new(caller_id, macro_call_node.id, EdgeKind::Expands));
        }

        self.nodes.push(macro_call_node);
        Ok(())
    }

    /// Handle lifetime nodes (lifetime parameters and lifetime annotations)
    fn handle_lifetime_node(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        let lifetime_name = self.get_node_text(&node);

        let lifetime_node = Node::new(
            &self.repo_id,
            NodeKind::Lifetime,
            lifetime_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        self.node_map.insert(node.id(), lifetime_node.id);

        // Create edge from the containing scope (function, struct, etc.)
        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, lifetime_node.id, EdgeKind::Contains));
        }

        self.nodes.push(lifetime_node);
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

    /// Extract detailed impl information
    fn extract_detailed_impl_info(
        &self,
        node: &tree_sitter::Node,
    ) -> Result<(String, Option<String>, String)> {
        let trait_name = if let Some(trait_node) = node.child_by_field_name("trait") {
            Some(self.get_node_text(&trait_node))
        } else {
            None
        };

        let type_name = if let Some(type_node) = node.child_by_field_name("type") {
            self.get_node_text(&type_node)
        } else {
            "unknown".to_string()
        };

        let impl_name = if let Some(trait_name) = &trait_name {
            format!("{} for {}", trait_name, type_name)
        } else {
            type_name.clone()
        };

        Ok((impl_name, trait_name, type_name))
    }

    /// Extract impl metadata
    fn extract_impl_metadata(
        &self,
        node: &tree_sitter::Node,
        trait_name: &Option<String>,
        type_name: &str,
    ) -> serde_json::Value {
        let mut metadata = serde_json::Map::new();

        // Add impl type information
        metadata.insert(
            "type_name".to_string(),
            serde_json::Value::String(type_name.to_string()),
        );

        if let Some(trait_name) = trait_name {
            metadata.insert(
                "trait_name".to_string(),
                serde_json::Value::String(trait_name.clone()),
            );
            metadata.insert(
                "impl_type".to_string(),
                serde_json::Value::String("trait_impl".to_string()),
            );
        } else {
            metadata.insert(
                "impl_type".to_string(),
                serde_json::Value::String("inherent_impl".to_string()),
            );
        }

        // Check for unsafe impl
        let text = self.get_node_text(node);
        if text.contains("unsafe impl") {
            metadata.insert("unsafe".to_string(), serde_json::Value::Bool(true));
        }

        serde_json::Value::Object(metadata)
    }

    /// Create trait implementation edge
    fn create_trait_implementation_edge(
        &mut self,
        _impl_id: crate::types::NodeId,
        trait_name: &str,
        type_name: &str,
    ) {
        // In a more sophisticated implementation, we would resolve the actual trait and type nodes
        // For now, we'll create a conceptual edge with metadata
        let _edge_metadata = serde_json::json!({
            "edge_type": "trait_implementation",
            "trait_name": trait_name,
            "type_name": type_name
        });

        // This could be enhanced to create actual edges to trait and type nodes
        // For now, we store the relationship as metadata on the impl node
        // The edge would be: Type --ImplementsTrait--> Trait
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

    /// Extract detailed function signature including lifetimes and ownership info
    fn extract_detailed_function_signature(&self, node: &tree_sitter::Node) -> Option<String> {
        let mut signature = String::new();

        // Add function name
        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.get_node_text(&name_node));
        }

        // Add generic parameters (including lifetimes)
        if let Some(generics_node) = node.child_by_field_name("type_parameters") {
            signature.push_str(&self.get_node_text(&generics_node));
        }

        // Add parameters with detailed type information
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let detailed_params = self.extract_detailed_parameters(&params_node);
            signature.push_str(&detailed_params);
        }

        // Add return type
        if let Some(return_type_node) = node.child_by_field_name("return_type") {
            signature.push_str(" ");
            signature.push_str(&self.get_node_text(&return_type_node));
        }

        if signature.is_empty() {
            None
        } else {
            Some(signature)
        }
    }

    /// Extract function metadata including visibility, async, unsafe, etc.
    fn extract_function_metadata(&self, node: &tree_sitter::Node) -> serde_json::Value {
        let mut metadata = serde_json::Map::new();

        // Check for visibility modifier
        if let Some(vis_node) = node.child_by_field_name("visibility_modifier") {
            metadata.insert(
                "visibility".to_string(),
                serde_json::Value::String(self.get_node_text(&vis_node)),
            );
        }

        // Check for async modifier
        let text = self.get_node_text(node);
        if text.contains("async") {
            metadata.insert("async".to_string(), serde_json::Value::Bool(true));
        }

        // Check for unsafe modifier
        if text.contains("unsafe") {
            metadata.insert("unsafe".to_string(), serde_json::Value::Bool(true));
        }

        // Check for const modifier
        if text.contains("const") {
            metadata.insert("const".to_string(), serde_json::Value::Bool(true));
        }

        serde_json::Value::Object(metadata)
    }

    /// Extract ownership and borrowing patterns from function parameters
    fn extract_function_ownership_patterns(
        &mut self,
        function_node: &tree_sitter::Node,
        function_id: crate::types::NodeId,
    ) -> Result<()> {
        if let Some(params_node) = function_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "parameter" {
                        self.analyze_parameter_ownership(&child, function_id)?;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract lifetime annotations from function
    fn extract_lifetime_annotations(
        &mut self,
        function_node: &tree_sitter::Node,
        function_id: crate::types::NodeId,
    ) -> Result<()> {
        // Extract lifetimes from generic parameters
        if let Some(generics_node) = function_node.child_by_field_name("type_parameters") {
            self.extract_lifetime_parameters(&generics_node, function_id)?;
        }

        // Extract lifetimes from parameters
        if let Some(params_node) = function_node.child_by_field_name("parameters") {
            self.extract_parameter_lifetimes(&params_node, function_id)?;
        }

        // Extract lifetimes from return type
        if let Some(return_type_node) = function_node.child_by_field_name("return_type") {
            self.extract_return_type_lifetimes(&return_type_node, function_id)?;
        }

        Ok(())
    }

    /// Extract detailed parameters with ownership information
    fn extract_detailed_parameters(&self, params_node: &tree_sitter::Node) -> String {
        let mut params = Vec::new();
        let mut cursor = params_node.walk();

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "parameter" {
                    let param_text = self.get_node_text(&child);
                    params.push(param_text);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        format!("({})", params.join(", "))
    }

    /// Analyze ownership patterns in a parameter
    fn analyze_parameter_ownership(
        &mut self,
        param_node: &tree_sitter::Node,
        function_id: crate::types::NodeId,
    ) -> Result<()> {
        let param_text = self.get_node_text(param_node);

        // Create parameter node
        if let Some(pattern_node) = param_node.child_by_field_name("pattern") {
            if pattern_node.kind() == "identifier" {
                let param_name = self.get_node_text(&pattern_node);
                let param_span = Span::from_node(param_node);

                // Determine ownership type
                let ownership_info = if param_text.contains("&mut") {
                    serde_json::json!({ "ownership": "mutable_borrow" })
                } else if param_text.contains("&") {
                    serde_json::json!({ "ownership": "immutable_borrow" })
                } else {
                    serde_json::json!({ "ownership": "owned" })
                };

                let param_node = Node::new(
                    &self.repo_id,
                    NodeKind::Parameter,
                    param_name,
                    self.language,
                    self.file_path.clone(),
                    param_span,
                )
                .with_metadata(ownership_info);

                // Create edge from function to parameter
                self.edges
                    .push(Edge::new(function_id, param_node.id, EdgeKind::Contains));

                self.nodes.push(param_node);
            }
        }

        Ok(())
    }

    /// Extract lifetime parameters from generics
    fn extract_lifetime_parameters(
        &mut self,
        generics_node: &tree_sitter::Node,
        function_id: crate::types::NodeId,
    ) -> Result<()> {
        let mut cursor = generics_node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "lifetime" {
                    let lifetime_name = self.get_node_text(&child);
                    let lifetime_span = Span::from_node(&child);

                    let lifetime_node = Node::new(
                        &self.repo_id,
                        NodeKind::Lifetime,
                        lifetime_name,
                        self.language,
                        self.file_path.clone(),
                        lifetime_span,
                    );

                    // Create edge from function to lifetime
                    self.edges
                        .push(Edge::new(function_id, lifetime_node.id, EdgeKind::Contains));

                    self.nodes.push(lifetime_node);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Extract lifetime annotations from parameters
    fn extract_parameter_lifetimes(
        &mut self,
        params_node: &tree_sitter::Node,
        _function_id: crate::types::NodeId,
    ) -> Result<()> {
        // This would involve parsing lifetime annotations in parameter types
        // For now, we'll implement a basic version
        let _params_text = self.get_node_text(params_node);
        // TODO: Parse lifetime annotations in parameter types
        Ok(())
    }

    /// Extract lifetime annotations from return type
    fn extract_return_type_lifetimes(
        &mut self,
        return_type_node: &tree_sitter::Node,
        _function_id: crate::types::NodeId,
    ) -> Result<()> {
        // This would involve parsing lifetime annotations in return types
        let _return_type_text = self.get_node_text(return_type_node);
        // TODO: Parse lifetime annotations in return types
        Ok(())
    }

    /// Extract trait signature including generics and bounds
    fn extract_trait_signature(&self, node: &tree_sitter::Node) -> Option<String> {
        let mut signature = String::new();

        // Add trait name
        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.get_node_text(&name_node));
        }

        // Add generic parameters
        if let Some(generics_node) = node.child_by_field_name("type_parameters") {
            signature.push_str(&self.get_node_text(&generics_node));
        }

        // Add trait bounds
        if let Some(bounds_node) = node.child_by_field_name("trait_bounds") {
            signature.push_str(": ");
            signature.push_str(&self.get_node_text(&bounds_node));
        }

        if signature.is_empty() {
            None
        } else {
            Some(signature)
        }
    }

    /// Extract trait metadata including visibility and safety
    fn extract_trait_metadata(&self, node: &tree_sitter::Node) -> serde_json::Value {
        let mut metadata = serde_json::Map::new();

        // Check for visibility modifier
        if let Some(vis_node) = node.child_by_field_name("visibility_modifier") {
            metadata.insert(
                "visibility".to_string(),
                serde_json::Value::String(self.get_node_text(&vis_node)),
            );
        }

        // Check for unsafe trait
        let text = self.get_node_text(node);
        if text.contains("unsafe trait") {
            metadata.insert("unsafe".to_string(), serde_json::Value::Bool(true));
        }

        serde_json::Value::Object(metadata)
    }

    /// Extract trait bounds (supertraits)
    fn extract_trait_bounds(
        &mut self,
        trait_node: &tree_sitter::Node,
        _trait_id: crate::types::NodeId,
    ) -> Result<()> {
        // Look for trait bounds in the trait definition
        if let Some(bounds_node) = trait_node.child_by_field_name("trait_bounds") {
            let bounds_text = self.get_node_text(&bounds_node);

            // Create edges for each supertrait
            // This is a simplified approach - in practice, we'd parse the bounds more carefully
            for bound in bounds_text.split('+') {
                let bound_name = bound.trim();
                if !bound_name.is_empty() {
                    // For now, we'll store this as metadata since we may not have the target trait node
                    // In a more sophisticated implementation, we'd resolve trait references
                    let edge_metadata = serde_json::json!({
                        "bound_type": "supertrait",
                        "target_trait": bound_name
                    });

                    // Create a conceptual edge - in practice this would link to actual trait nodes
                    let mut metadata = serde_json::Map::new();
                    metadata.insert("trait_bounds".to_string(), edge_metadata);
                }
            }
        }
        Ok(())
    }

    /// Extract associated items (types, constants, functions) from trait
    fn extract_associated_items(
        &mut self,
        trait_node: &tree_sitter::Node,
        trait_id: crate::types::NodeId,
    ) -> Result<()> {
        // Look for the trait body
        if let Some(body_node) = trait_node.child_by_field_name("body") {
            let mut cursor = body_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "associated_type" => {
                            self.handle_associated_type(&child, trait_id)?;
                        }
                        "const_item" => {
                            self.handle_associated_const(&child, trait_id)?;
                        }
                        "function_item" => {
                            // This is a trait method - already handled by regular function handler
                            // but we could add trait-specific metadata here
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle associated type declarations
    fn handle_associated_type(
        &mut self,
        assoc_type_node: &tree_sitter::Node,
        trait_id: crate::types::NodeId,
    ) -> Result<()> {
        let span = Span::from_node(assoc_type_node);
        let name = self.extract_identifier_name(assoc_type_node, "name")?;

        let assoc_type_node = Node::new(
            &self.repo_id,
            NodeKind::AssociatedType,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Create edge from trait to associated type
        self.edges
            .push(Edge::new(trait_id, assoc_type_node.id, EdgeKind::Contains));

        self.nodes.push(assoc_type_node);
        Ok(())
    }

    /// Handle associated constant declarations
    fn handle_associated_const(
        &mut self,
        assoc_const_node: &tree_sitter::Node,
        trait_id: crate::types::NodeId,
    ) -> Result<()> {
        let span = Span::from_node(assoc_const_node);
        let name = self.extract_identifier_name(assoc_const_node, "name")?;

        let assoc_const_node = Node::new(
            &self.repo_id,
            NodeKind::AssociatedConst,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Create edge from trait to associated const
        self.edges
            .push(Edge::new(trait_id, assoc_const_node.id, EdgeKind::Contains));

        self.nodes.push(assoc_const_node);
        Ok(())
    }

    /// Extract attribute name
    fn extract_attribute_name(&self, node: &tree_sitter::Node) -> String {
        let text = self.get_node_text(node);

        // Extract attribute name from #[attr] or #[attr(...)]
        if let Some(start) = text.find('[') {
            if let Some(end) = text[start..].find(']') {
                let attr_content = &text[start + 1..start + end];

                // Handle derive attributes specially
                if attr_content.starts_with("derive") {
                    return attr_content.to_string();
                }

                // For other attributes, extract just the name part
                if let Some(paren_pos) = attr_content.find('(') {
                    attr_content[..paren_pos].to_string()
                } else {
                    attr_content.to_string()
                }
            } else {
                "malformed_attribute".to_string()
            }
        } else {
            "unknown_attribute".to_string()
        }
    }

    /// Extract attribute metadata
    fn extract_attribute_metadata(&self, node: &tree_sitter::Node) -> serde_json::Value {
        let mut metadata = serde_json::Map::new();
        let text = self.get_node_text(node);

        // Determine attribute type
        if text.contains("derive") {
            metadata.insert(
                "attribute_type".to_string(),
                serde_json::Value::String("derive".to_string()),
            );

            // Extract derived traits
            if let Some(start) = text.find('(') {
                if let Some(end) = text.rfind(')') {
                    let traits_text = &text[start + 1..end];
                    let traits: Vec<String> = traits_text
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    metadata.insert("derived_traits".to_string(), serde_json::json!(traits));
                }
            }
        } else {
            metadata.insert(
                "attribute_type".to_string(),
                serde_json::Value::String("regular".to_string()),
            );
        }

        serde_json::Value::Object(metadata)
    }

    /// Handle derive attributes specially
    fn handle_derive_attribute(
        &mut self,
        attr_node: &tree_sitter::Node,
        _attr_id: crate::types::NodeId,
    ) -> Result<()> {
        let text = self.get_node_text(attr_node);

        // Extract derived traits and create edges
        if let Some(start) = text.find('(') {
            if let Some(end) = text.rfind(')') {
                let traits_text = &text[start + 1..end];
                for trait_name in traits_text.split(',') {
                    let trait_name = trait_name.trim();
                    if !trait_name.is_empty() {
                        // Create a derive edge - in practice this would link to the actual trait
                        // For now, we'll store this as metadata indicating the derive relationship
                        let _edge_metadata = serde_json::json!({
                            "derive_trait": trait_name
                        });

                        // This could be enhanced to create edges: Type --Derives--> Trait
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract macro invocation name
    fn extract_macro_invocation_name(&self, node: &tree_sitter::Node) -> String {
        if let Some(macro_node) = node.child_by_field_name("macro") {
            self.get_node_text(&macro_node)
        } else {
            // Fallback: try to extract from the full text
            let text = self.get_node_text(node);
            if let Some(exclamation_pos) = text.find('!') {
                text[..exclamation_pos].trim().to_string()
            } else {
                "unknown_macro".to_string()
            }
        }
    }
}
