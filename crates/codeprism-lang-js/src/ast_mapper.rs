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
                // Skip other node types in this implementation
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
    fn handle_export(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        match node.kind() {
            "export_statement" | "export_declaration" => {
                // Handle different types of exports
                let mut child_cursor = node.walk();
                if child_cursor.goto_first_child() {
                    loop {
                        let child = child_cursor.node();
                        match child.kind() {
                            "default" => {
                                // export default ...
                                if child_cursor.goto_next_sibling() {
                                    let export_target = child_cursor.node();
                                    self.handle_default_export(&export_target, &span)?;
                                }
                                break;
                            }
                            "function_declaration" | "class_declaration" => {
                                // export function foo() {} or export class Foo {}
                                self.handle_named_export(&child, &span)?;
                            }
                            "variable_declaration" | "lexical_declaration" => {
                                // export const/let/var foo = ...
                                self.handle_variable_export(&child, &span)?;
                            }
                            "export_clause" => {
                                // export { foo, bar }
                                self.handle_export_clause(&child, &span)?;
                            }
                            "string" => {
                                // export ... from "module"
                                let source = self.get_node_text(&child);
                                let module_name = source.trim_matches(|c| c == '"' || c == '\'');
                                self.create_re_export_node(module_name.to_string(), span.clone())?;
                            }
                            _ => {}
                        }
                        if !child_cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle default export
    fn handle_default_export(
        &mut self,
        target_node: &tree_sitter::Node,
        span: &Span,
    ) -> Result<()> {
        let export_name = match target_node.kind() {
            "function_declaration" | "function" => {
                if let Some(name_node) = target_node.child_by_field_name("name") {
                    self.get_node_text(&name_node)
                } else {
                    "default".to_string()
                }
            }
            "class_declaration" | "class" => {
                if let Some(name_node) = target_node.child_by_field_name("name") {
                    self.get_node_text(&name_node)
                } else {
                    "default".to_string()
                }
            }
            "identifier" => self.get_node_text(target_node),
            _ => "default".to_string(),
        };

        self.create_export_node(export_name, span.clone(), true)?;
        Ok(())
    }

    /// Handle named export
    fn handle_named_export(&mut self, target_node: &tree_sitter::Node, span: &Span) -> Result<()> {
        let export_name = if let Some(name_node) = target_node.child_by_field_name("name") {
            self.get_node_text(&name_node)
        } else {
            "unnamed".to_string()
        };

        self.create_export_node(export_name, span.clone(), false)?;
        Ok(())
    }

    /// Handle variable export
    fn handle_variable_export(
        &mut self,
        target_node: &tree_sitter::Node,
        span: &Span,
    ) -> Result<()> {
        // Find all variable declarators
        let mut cursor = target_node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "variable_declarator" {
                    if let Some(name_node) = cursor.node().child_by_field_name("name") {
                        let export_name = self.get_node_text(&name_node);
                        self.create_export_node(export_name, span.clone(), false)?;
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Handle export clause (export { foo, bar })
    fn handle_export_clause(&mut self, clause_node: &tree_sitter::Node, span: &Span) -> Result<()> {
        let mut cursor = clause_node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "export_specifier" => {
                        // Handle { foo as bar } or just { foo }
                        let mut export_name = String::new();

                        if let Some(name_node) = child.child_by_field_name("name") {
                            export_name = self.get_node_text(&name_node);
                        } else if let Some(local_node) = child.child_by_field_name("local") {
                            export_name = self.get_node_text(&local_node);
                        }

                        if !export_name.is_empty() {
                            self.create_export_node(export_name, span.clone(), false)?;
                        }
                    }
                    "identifier" => {
                        // Simple export name
                        let export_name = self.get_node_text(&child);
                        self.create_export_node(export_name, span.clone(), false)?;
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Create an export node
    fn create_export_node(&mut self, name: String, span: Span, is_default: bool) -> Result<()> {
        let export_node = Node::new(
            &self.repo_id,
            NodeKind::Variable, // Use Variable for exports since they're essentially exported variables/functions
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Add metadata for default exports and to indicate this is an export
        let mut metadata = serde_json::Map::new();
        metadata.insert("is_export".to_string(), serde_json::Value::Bool(true));
        if is_default {
            metadata.insert("is_default".to_string(), serde_json::Value::Bool(true));
        }
        let export_with_metadata = Node {
            metadata: serde_json::Value::Object(metadata),
            ..export_node
        };

        // Add edge from module to export
        if let Some(module_id) = self.find_module_node_id() {
            self.edges.push(Edge::new(
                module_id,
                export_with_metadata.id,
                EdgeKind::Writes,
            ));
        }

        self.nodes.push(export_with_metadata);

        Ok(())
    }

    /// Create a re-export node (export ... from "module")
    fn create_re_export_node(&mut self, module_name: String, span: Span) -> Result<()> {
        let reexport_node = Node::new(
            &self.repo_id,
            NodeKind::Import, // Re-exports are essentially imports that are also exported
            module_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Add metadata to indicate this is a re-export
        let mut metadata = serde_json::Map::new();
        metadata.insert("is_reexport".to_string(), serde_json::Value::Bool(true));
        let reexport_with_metadata = Node {
            metadata: serde_json::Value::Object(metadata),
            ..reexport_node
        };

        // Add edge from module to re-export
        if let Some(module_id) = self.find_module_node_id() {
            self.edges.push(Edge::new(
                module_id,
                reexport_with_metadata.id,
                EdgeKind::Imports,
            ));
        }

        self.nodes.push(reexport_with_metadata);
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
    fn extract_function_signature(&self, node: &tree_sitter::Node) -> Option<String> {
        let mut signature_parts = Vec::new();

        // Get function name
        if let Some(name_node) = node.child_by_field_name("name") {
            signature_parts.push(self.get_node_text(&name_node));
        } else {
            signature_parts.push("<anonymous>".to_string());
        }

        // Parse parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let params_str = self.extract_parameters_with_types(&params_node);
            signature_parts.push(format!("({})", params_str));
        } else {
            signature_parts.push("()".to_string());
        }

        // Parse return type annotation (TypeScript specific)
        if let Some(return_type_node) = node.child_by_field_name("return_type") {
            let return_type = self.get_node_text(&return_type_node);
            signature_parts.push(format!(": {}", return_type));
        }

        if signature_parts.len() > 1 {
            Some(signature_parts.join(""))
        } else {
            None
        }
    }

    /// Extract parameters with their type annotations
    fn extract_parameters_with_types(&self, params_node: &tree_sitter::Node) -> String {
        let mut params = Vec::new();
        let mut cursor = params_node.walk();

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "identifier" => {
                        // Simple parameter without type annotation
                        params.push(self.get_node_text(&child));
                    }
                    "required_parameter" | "optional_parameter" => {
                        // TypeScript parameter with optional type annotation
                        let mut param_parts = Vec::new();

                        if let Some(pattern_node) = child.child_by_field_name("pattern") {
                            param_parts.push(self.get_node_text(&pattern_node));
                        }

                        // Check if parameter is optional
                        if child.kind() == "optional_parameter" {
                            param_parts.push("?".to_string());
                        }

                        if let Some(type_node) = child.child_by_field_name("type") {
                            param_parts.push(format!(": {}", self.get_node_text(&type_node)));
                        }

                        if !param_parts.is_empty() {
                            params.push(param_parts.join(""));
                        }
                    }
                    "assignment_pattern" => {
                        // Parameter with default value: param = default
                        if let Some(left_node) = child.child_by_field_name("left") {
                            let mut param_parts = vec![self.get_node_text(&left_node)];

                            if let Some(right_node) = child.child_by_field_name("right") {
                                param_parts.push(format!(" = {}", self.get_node_text(&right_node)));
                            }

                            params.push(param_parts.join(""));
                        }
                    }
                    "rest_pattern" => {
                        // Rest parameter: ...args
                        if let Some(argument_node) = child.child_by_field_name("argument") {
                            params.push(format!("...{}", self.get_node_text(&argument_node)));
                        } else {
                            params.push(format!(
                                "...{}",
                                self.get_node_text(&child).trim_start_matches("...")
                            ));
                        }
                    }
                    "formal_parameters" => {
                        // Nested formal parameters - recurse
                        let nested_params = self.extract_parameters_with_types(&child);
                        if !nested_params.is_empty() {
                            params.push(nested_params);
                        }
                    }
                    _ => {
                        // Handle other parameter types or skip non-parameter nodes like commas
                        let text = self.get_node_text(&child);
                        if !text.trim().is_empty() && text != "," && text != "(" && text != ")" {
                            // Check if this looks like a parameter
                            if text.chars().any(|c| c.is_alphanumeric() || c == '_') {
                                params.push(text);
                            }
                        }
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        params.join(", ")
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
