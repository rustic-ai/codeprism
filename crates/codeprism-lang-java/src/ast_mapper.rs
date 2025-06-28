//! AST mapping from tree-sitter Java CST to Universal AST

use crate::error::Result;
use crate::types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Node as TSNode, Tree};

/// Maps tree-sitter Java CST to Universal AST
pub struct AstMapper {
    /// Repository ID
    repo_id: String,
    /// File path
    file_path: PathBuf,
    /// Language
    language: Language,
    /// Source content
    content: String,
    /// Collected nodes
    nodes: Vec<Node>,
    /// Collected edges
    edges: Vec<Edge>,
    /// Node ID mappings (tree-sitter node ID -> Universal AST node ID)
    node_mappings: HashMap<usize, NodeId>,
}

impl AstMapper {
    /// Create a new AST mapper
    pub fn new(repo_id: &str, file_path: PathBuf, language: Language, content: &str) -> Self {
        Self {
            repo_id: repo_id.to_string(),
            file_path,
            language,
            content: content.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_mappings: HashMap::new(),
        }
    }

    /// Extract nodes and edges from the tree
    pub fn extract(mut self, tree: &Tree) -> Result<(Vec<Node>, Vec<Edge>)> {
        let root = tree.root_node();
        
        // Create module node for the file
        let module_span = Span::from_node(&root);
        let file_name = self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let module_node = Node::new(
            &self.repo_id,
            NodeKind::Module,
            file_name,
            self.language,
            self.file_path.clone(),
            module_span,
        ).with_metadata(json!({
            "type": "compilation_unit",
            "file_path": self.file_path.display().to_string()
        }));

        let module_id = module_node.id;
        self.nodes.push(module_node);
        self.node_mappings.insert(root.id(), module_id);

        // Process all child nodes
        self.process_node(&root, Some(module_id))?;

        Ok((self.nodes, self.edges))
    }

    /// Process a tree-sitter node recursively
    fn process_node(&mut self, ts_node: &TSNode, parent_id: Option<NodeId>) -> Result<Option<NodeId>> {
        let node_kind = ts_node.kind();
        
        let universal_node = match node_kind {
            "program" => {
                // Skip program node, already handled as module
                None
            }
            "package_declaration" => self.process_package_declaration(ts_node)?,
            "import_declaration" => self.process_import_declaration(ts_node)?,
            "class_declaration" => {
                // First process annotations in modifiers
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_class_declaration(ts_node)?
            }
            "interface_declaration" => {
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_interface_declaration(ts_node)?
            }
            "enum_declaration" => {
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_enum_declaration(ts_node)?
            }
            "method_declaration" => {
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_method_declaration(ts_node)?
            }
            "constructor_declaration" => {
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_constructor_declaration(ts_node)?
            }
            "field_declaration" => {
                self.process_modifiers_annotations(ts_node, parent_id)?;
                self.process_field_declaration(ts_node)?
            }
            "annotation" | "marker_annotation" | "normal_annotation" => self.process_annotation(ts_node)?,
            "method_invocation" => self.process_method_invocation(ts_node)?,
            _ => {
                // For unhandled node types, still process children
                None
            }
        };

        // Add edge from parent to this node
        if let (Some(parent), Some(node_id)) = (parent_id, &universal_node) {
            self.edges.push(Edge::new(parent, *node_id, EdgeKind::Contains));
        }

        // Process children
        let mut cursor = ts_node.walk();
        for child in ts_node.children(&mut cursor) {
            let child_parent = universal_node.or(parent_id);
            self.process_node(&child, child_parent)?;
        }

        Ok(universal_node)
    }

    /// Process annotations found in modifiers
    fn process_modifiers_annotations(&mut self, ts_node: &TSNode, parent_id: Option<NodeId>) -> Result<()> {
        let mut cursor = ts_node.walk();
        
        for child in ts_node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mut mod_cursor = child.walk();
                for modifier in child.children(&mut mod_cursor) {
                    if matches!(modifier.kind(), "annotation" | "marker_annotation" | "normal_annotation") {
                        if let Some(annotation_id) = self.process_annotation(&modifier)? {
                            // Add edge from parent to annotation
                            if let Some(parent) = parent_id {
                                self.edges.push(Edge::new(parent, annotation_id, EdgeKind::Annotates));
                            }
                        }
                    }
                }
                break;
            }
        }
        
        Ok(())
    }

    /// Get the text content of a node
    fn node_text(&self, node: &TSNode) -> String {
        node.utf8_text(self.content.as_bytes())
            .unwrap_or("")
            .to_string()
    }

    /// Process package declaration
    fn process_package_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let package_name = self.extract_package_name(ts_node);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Package,
            package_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "package_name": package_name,
            "type": "package_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process import declaration
    fn process_import_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let import_path = self.extract_import_path(ts_node);
        let is_static = self.node_text(ts_node).contains("static");
        let is_wildcard = import_path.ends_with("*");
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Import,
            import_path.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "import_path": import_path,
            "is_static": is_static,
            "is_wildcard": is_wildcard,
            "type": "import_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process class declaration
    fn process_class_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let class_name = self.extract_class_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let is_abstract = modifiers.contains(&"abstract".to_string());
        let is_final = modifiers.contains(&"final".to_string());
        let visibility = self.extract_visibility(&modifiers);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Class,
            class_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "class_name": class_name,
            "modifiers": modifiers,
            "is_abstract": is_abstract,
            "is_final": is_final,
            "visibility": visibility,
            "type": "class_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process interface declaration
    fn process_interface_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let interface_name = self.extract_interface_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let visibility = self.extract_visibility(&modifiers);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Interface,
            interface_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "interface_name": interface_name,
            "modifiers": modifiers,
            "visibility": visibility,
            "type": "interface_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process enum declaration
    fn process_enum_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let enum_name = self.extract_enum_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let visibility = self.extract_visibility(&modifiers);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Enum,
            enum_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "enum_name": enum_name,
            "modifiers": modifiers,
            "visibility": visibility,
            "type": "enum_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process method declaration
    fn process_method_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let method_name = self.extract_method_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let is_static = modifiers.contains(&"static".to_string());
        let is_abstract = modifiers.contains(&"abstract".to_string());
        let is_final = modifiers.contains(&"final".to_string());
        let is_synchronized = modifiers.contains(&"synchronized".to_string());
        let visibility = self.extract_visibility(&modifiers);
        let return_type = self.extract_return_type(ts_node);
        let parameters = self.extract_method_parameters(ts_node);
        let span = Span::from_node(ts_node);

        let signature = self.build_method_signature(&method_name, &parameters, &return_type);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Method,
            method_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_signature(signature)
         .with_metadata(json!({
            "method_name": method_name,
            "modifiers": modifiers,
            "is_static": is_static,
            "is_abstract": is_abstract,
            "is_final": is_final,
            "is_synchronized": is_synchronized,
            "visibility": visibility,
            "return_type": return_type,
            "parameters": parameters,
            "type": "method_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process constructor declaration
    fn process_constructor_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let constructor_name = self.extract_constructor_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let visibility = self.extract_visibility(&modifiers);
        let parameters = self.extract_method_parameters(ts_node);
        let span = Span::from_node(ts_node);

        let signature = self.build_constructor_signature(&constructor_name, &parameters);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Constructor,
            constructor_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_signature(signature)
         .with_metadata(json!({
            "constructor_name": constructor_name,
            "modifiers": modifiers,
            "visibility": visibility,
            "parameters": parameters,
            "type": "constructor_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process field declaration
    fn process_field_declaration(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let field_name = self.extract_field_name(ts_node);
        let modifiers = self.extract_modifiers(ts_node);
        let is_static = modifiers.contains(&"static".to_string());
        let is_final = modifiers.contains(&"final".to_string());
        let is_volatile = modifiers.contains(&"volatile".to_string());
        let is_transient = modifiers.contains(&"transient".to_string());
        let visibility = self.extract_visibility(&modifiers);
        let field_type = self.extract_field_type(ts_node);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Field,
            field_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "field_name": field_name,
            "modifiers": modifiers,
            "is_static": is_static,
            "is_final": is_final,
            "is_volatile": is_volatile,
            "is_transient": is_transient,
            "visibility": visibility,
            "field_type": field_type,
            "type": "field_declaration"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process annotation
    fn process_annotation(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let annotation_name = self.extract_annotation_name(ts_node);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Annotation,
            annotation_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "annotation_name": annotation_name,
            "type": "annotation"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    /// Process method invocation
    fn process_method_invocation(&mut self, ts_node: &TSNode) -> Result<Option<NodeId>> {
        let method_name = self.extract_invocation_method_name(ts_node);
        let span = Span::from_node(ts_node);

        let node = Node::new(
            &self.repo_id,
            NodeKind::Call,
            method_name.clone(),
            self.language,
            self.file_path.clone(),
            span,
        ).with_metadata(json!({
            "method_name": method_name,
            "type": "method_invocation"
        }));

        let node_id = node.id;
        self.nodes.push(node);
        self.node_mappings.insert(ts_node.id(), node_id);
        
        Ok(Some(node_id))
    }

    // Helper methods for extracting information from tree-sitter nodes

    /// Extract package name from package declaration
    fn extract_package_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "unknown".to_string()
    }

    /// Extract import path from import declaration
    fn extract_import_path(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                return self.node_text(&child);
            }
            if child.kind() == "asterisk" {
                if let Some(prev_sibling) = child.prev_sibling() {
                    return format!("{}.*", self.node_text(&prev_sibling));
                }
            }
        }
        "unknown".to_string()
    }

    /// Extract class name from class declaration
    fn extract_class_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "Unknown".to_string()
    }

    /// Extract interface name from interface declaration
    fn extract_interface_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "Unknown".to_string()
    }

    /// Extract enum name from enum declaration
    fn extract_enum_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "Unknown".to_string()
    }

    /// Extract modifiers from a declaration
    fn extract_modifiers(&self, node: &TSNode) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mut mod_cursor = child.walk();
                for modifier in child.children(&mut mod_cursor) {
                    modifiers.push(self.node_text(&modifier));
                }
                break;
            }
        }
        
        modifiers
    }

    /// Extract visibility from modifiers
    fn extract_visibility(&self, modifiers: &[String]) -> String {
        for modifier in modifiers {
            match modifier.as_str() {
                "public" | "private" | "protected" => return modifier.clone(),
                _ => {}
            }
        }
        "package-private".to_string()
    }

    /// Extract method name from method declaration
    fn extract_method_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "unknown".to_string()
    }

    /// Extract constructor name from constructor declaration
    fn extract_constructor_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "unknown".to_string()
    }

    /// Extract return type from method declaration
    fn extract_return_type(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "type_identifier" | "void_type" | "generic_type" | "array_type") {
                return self.node_text(&child);
            }
        }
        "void".to_string()
    }

    /// Extract method parameters
    fn extract_method_parameters(&self, node: &TSNode) -> Vec<Value> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "formal_parameters" {
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "formal_parameter" {
                        if let Some(param_info) = self.extract_parameter_info(&param) {
                            parameters.push(param_info);
                        }
                    }
                }
                break;
            }
        }
        
        parameters
    }

    /// Extract parameter information
    fn extract_parameter_info(&self, param_node: &TSNode) -> Option<Value> {
        let mut param_type = String::new();
        let mut param_name = String::new();
        
        let mut cursor = param_node.walk();
        for child in param_node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" | "generic_type" | "array_type" => {
                    param_type = self.node_text(&child);
                }
                "identifier" => {
                    param_name = self.node_text(&child);
                }
                _ => {}
            }
        }
        
        if !param_name.is_empty() {
            Some(json!({
                "name": param_name,
                "type": param_type
            }))
        } else {
            None
        }
    }

    /// Build method signature
    fn build_method_signature(&self, name: &str, params: &[Value], return_type: &str) -> String {
        let param_strs: Vec<String> = params.iter()
            .filter_map(|p| {
                if let (Some(name), Some(ptype)) = (p.get("name"), p.get("type")) {
                    Some(format!("{}: {}", name.as_str().unwrap_or(""), ptype.as_str().unwrap_or("")))
                } else {
                    None
                }
            })
            .collect();
        
        format!("{}({}) -> {}", name, param_strs.join(", "), return_type)
    }

    /// Build constructor signature
    fn build_constructor_signature(&self, name: &str, params: &[Value]) -> String {
        let param_strs: Vec<String> = params.iter()
            .filter_map(|p| {
                if let (Some(name), Some(ptype)) = (p.get("name"), p.get("type")) {
                    Some(format!("{}: {}", name.as_str().unwrap_or(""), ptype.as_str().unwrap_or("")))
                } else {
                    None
                }
            })
            .collect();
        
        format!("{}({})", name, param_strs.join(", "))
    }

    /// Extract field name from field declaration
    fn extract_field_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                let mut var_cursor = child.walk();
                for var_child in child.children(&mut var_cursor) {
                    if var_child.kind() == "identifier" {
                        return self.node_text(&var_child);
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Extract field type from field declaration
    fn extract_field_type(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "type_identifier" | "generic_type" | "array_type" | "primitive_type") {
                return self.node_text(&child);
            }
        }
        "Object".to_string()
    }

    /// Extract annotation name
    fn extract_annotation_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" || child.kind() == "identifier" {
                return self.node_text(&child);
            }
        }
        "unknown".to_string()
    }

    /// Extract method name from method invocation
    fn extract_invocation_method_name(&self, node: &TSNode) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child);
            }
            if child.kind() == "field_access" {
                // For chained method calls like obj.method()
                let mut field_cursor = child.walk();
                for field_child in child.children(&mut field_cursor) {
                    if field_child.kind() == "identifier" {
                        return self.node_text(&field_child);
                    }
                }
            }
        }
        "unknown".to_string()
    }
} 