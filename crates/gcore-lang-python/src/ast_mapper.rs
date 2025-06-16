//! AST mapper for converting Tree-sitter CST to Universal AST for Python

use crate::error::Result;
use crate::types::{Edge, EdgeKind, Language, Node, NodeKind, Span};
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Tree, TreeCursor};

/// AST mapper for Python
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
            // Function definitions
            "function_definition" => {
                self.handle_function(cursor)?;
            }

            // Class definitions
            "class_definition" => {
                self.handle_class(cursor)?;
            }

            // Assignment statements (variable declarations)
            "assignment" | "augmented_assignment" => {
                self.handle_assignment(cursor)?;
            }

            // Function calls
            "call" => {
                self.handle_call_expression(cursor)?;
            }

            // Import statements
            "import_statement" | "import_from_statement" => {
                self.handle_import(cursor)?;
            }

            // Method definitions (inside class)
            "decorated_definition" => {
                // Check if this is a method definition
                self.handle_decorated_definition(cursor)?;
            }

            _ => {
                // Skip other node types for now
            }
        }

        Ok(())
    }

    /// Handle function definitions
    fn handle_function(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract function name
        let name = self.extract_function_name(&node)?;

        // Determine if it's a method (inside a class) or function
        let kind = if self.is_inside_class(&node) {
            NodeKind::Method
        } else {
            NodeKind::Function
        };

        // Extract type hints if available
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

        // Add edge from parent (module or class) to function
        if let Some(parent_id) = self.find_parent_scope_id(&node) {
            self.edges
                .push(Edge::new(parent_id, func_node.id, EdgeKind::Calls));
        }

        self.nodes.push(func_node);
        Ok(())
    }

    /// Handle class definitions
    fn handle_class(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract class name
        let name = self.extract_class_name(&node)?;
        
        // Skip if name extraction failed
        if name == "anonymous" || name.is_empty() {
            return Ok(());
        }

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

        // Extract base classes and create inheritance edges
        self.extract_base_classes(&node, class_node.id)?;

        self.nodes.push(class_node);
        Ok(())
    }

    /// Handle assignments (variable declarations)
    fn handle_assignment(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract variable names from left side
        if let Some(left_node) = node.child_by_field_name("left") {
            self.extract_assignment_targets(&left_node, &span)?;
        }

        Ok(())
    }

    /// Extract assignment targets (variables being assigned to)
    fn extract_assignment_targets(&mut self, target_node: &tree_sitter::Node, span: &Span) -> Result<()> {
        match target_node.kind() {
            "identifier" => {
                let name = self.get_node_text(target_node);
                self.create_variable_node(name, span.clone())?;
            }
            "pattern_list" | "tuple_pattern" => {
                // Handle multiple assignment: a, b = 1, 2
                let mut cursor = target_node.walk();
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        if child.kind() == "identifier" {
                            let name = self.get_node_text(&child);
                            let child_span = Span::from_node(&child);
                            self.create_variable_node(name, child_span)?;
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
            _ => {
                // Other types of assignments (attribute access, subscript, etc.)
                // For now, skip these
            }
        }
        Ok(())
    }

    /// Create a variable node
    fn create_variable_node(&mut self, name: String, span: Span) -> Result<()> {
        let var_node = Node::new(
            &self.repo_id,
            NodeKind::Variable,
            name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Add edge from parent scope
        if let Some(parent_id) = self.find_enclosing_scope_id() {
            self.edges
                .push(Edge::new(parent_id, var_node.id, EdgeKind::Writes));
        }

        self.nodes.push(var_node);
        Ok(())
    }

    /// Handle function calls
    fn handle_call_expression(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        // Extract function being called
        if let Some(function_node) = node.child_by_field_name("function") {
            let function_name = self.extract_call_target(&function_node);

            let call_node = Node::new(
                &self.repo_id,
                NodeKind::Call,
                function_name,
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

    /// Extract the target of a call (function name)
    fn extract_call_target(&self, node: &tree_sitter::Node) -> String {
        match node.kind() {
            "identifier" => {
                self.get_node_text(node)
            }
            "attribute" => {
                // Handle method calls like obj.method()
                if let Some(attr_node) = node.child_by_field_name("attribute") {
                    self.get_node_text(&attr_node)
                } else {
                    // Fallback: extract from full text but validate
                    self.extract_safe_function_name(node)
                }
            }
            "subscript" => {
                // Handle subscript calls like func[key]()
                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_call_target(&value_node)
                } else {
                    self.extract_safe_function_name(node)
                }
            }
            "call" => {
                // Nested call like func()()
                if let Some(function_node) = node.child_by_field_name("function") {
                    self.extract_call_target(&function_node)
                } else {
                    self.extract_safe_function_name(node)
                }
            }
            _ => {
                // For any other complex expressions, try to extract safely
                self.extract_safe_function_name(node)
            }
        }
    }

    /// Safely extract a function name from a node, avoiding malformed names
    fn extract_safe_function_name(&self, node: &tree_sitter::Node) -> String {
        let raw_text = self.get_node_text(node);
        
        // Check for obviously invalid names
        if raw_text.is_empty() || 
           raw_text == ")" || 
           raw_text == "(" || 
           raw_text.trim().is_empty() ||
           raw_text.chars().all(|c| !c.is_alphanumeric() && c != '_' && c != '.') {
            return "anonymous_call".to_string();
        }
        
        // Try to extract a meaningful name from complex expressions
        if let Some(extracted) = self.extract_function_name_from_text(&raw_text) {
            extracted
        } else {
            // Fallback: use truncated text or anonymous
            if raw_text.len() > 50 {
                "complex_call".to_string()
            } else {
                raw_text
            }
        }
    }

    /// Extract function name from raw text using pattern matching
    fn extract_function_name_from_text(&self, text: &str) -> Option<String> {
        let text = text.trim();
        
        // Handle simple identifier
        if text.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(text.to_string());
        }
        
        // Handle attribute access: obj.method
        if let Some(dot_pos) = text.rfind('.') {
            let after_dot = &text[dot_pos + 1..];
            if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(after_dot.to_string());
            }
        }
        
        // Handle complex expressions by finding the last valid identifier
        let mut last_identifier = String::new();
        let mut current_identifier = String::new();
        
        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                current_identifier.push(ch);
            } else {
                if !current_identifier.is_empty() {
                    last_identifier = current_identifier.clone();
                    current_identifier.clear();
                }
            }
        }
        
        // Use the last identifier found
        if !current_identifier.is_empty() {
            Some(current_identifier)
        } else if !last_identifier.is_empty() {
            Some(last_identifier)
        } else {
            None
        }
    }

    /// Handle import statements
    fn handle_import(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        let span = Span::from_node(&node);

        match node.kind() {
            "import_statement" => {
                // Handle: import module_name
                self.handle_simple_import(&node, &span)?;
            }
            "import_from_statement" => {
                // Handle: from module import name
                self.handle_from_import(&node, &span)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle simple import: import module
    fn handle_simple_import(&mut self, node: &tree_sitter::Node, span: &Span) -> Result<()> {
        // Find import names
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "dotted_name" || child.kind() == "identifier" {
                    let module_name = self.get_node_text(&child);
                    self.create_import_node(module_name, span.clone())?;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Handle from import: from module import name
    fn handle_from_import(&mut self, node: &tree_sitter::Node, span: &Span) -> Result<()> {
        // Extract module name
        if let Some(module_node) = node.child_by_field_name("module_name") {
            let module_name = self.get_node_text(&module_node);
            self.create_import_node(module_name, span.clone())?;
        }
        Ok(())
    }

    /// Create an import node
    fn create_import_node(&mut self, module_name: String, span: Span) -> Result<()> {
        let import_node = Node::new(
            &self.repo_id,
            NodeKind::Import,
            module_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Add edge from module to import
        if let Some(module_id) = self.find_module_node_id() {
            self.edges
                .push(Edge::new(module_id, import_node.id, EdgeKind::Imports));
        }

        self.nodes.push(import_node);
        Ok(())
    }

    /// Handle decorated definitions (might be methods)
    fn handle_decorated_definition(&mut self, cursor: &TreeCursor) -> Result<()> {
        let node = cursor.node();
        
        // Look for function_definition inside decorated_definition
        let mut child_cursor = node.walk();
        if child_cursor.goto_first_child() {
            loop {
                if child_cursor.node().kind() == "function_definition" {
                    self.handle_function(&child_cursor)?;
                    break;
                }
                if !child_cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Extract function name from a function node
    fn extract_function_name(&self, node: &tree_sitter::Node) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            Ok(self.get_node_text(&name_node))
        } else {
            Ok("anonymous".to_string())
        }
    }

    /// Extract class name from a class node
    fn extract_class_name(&self, node: &tree_sitter::Node) -> Result<String> {
        // First try the standard "name" field
        if let Some(name_node) = node.child_by_field_name("name") {
            return Ok(self.get_node_text(&name_node));
        }
        
        // Fallback: search for identifier nodes in the class definition
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "identifier" {
                    // Found an identifier - this is likely the class name
                    let class_name = self.get_node_text(&child);
                    if !class_name.is_empty() && class_name != "class" {
                        return Ok(class_name);
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        // Last resort: extract from full node text
        let full_text = self.get_node_text(node);
        if let Some(name) = self.extract_class_name_from_text(&full_text) {
            return Ok(name);
        }
        
        Ok("anonymous".to_string())
    }
    
    /// Extract class name from raw text using regex-like parsing
    fn extract_class_name_from_text(&self, text: &str) -> Option<String> {
        // Look for pattern: class ClassName(...):
        let lines: Vec<&str> = text.lines().collect();
        if let Some(first_line) = lines.first() {
            let trimmed = first_line.trim();
            if trimmed.starts_with("class ") {
                // Extract everything between "class " and "(" or ":"
                let after_class = &trimmed[6..]; // Skip "class "
                
                // Find the class name (everything up to '(' or ':' or whitespace)
                for (i, ch) in after_class.char_indices() {
                    if ch == '(' || ch == ':' || ch.is_whitespace() {
                        let class_name = after_class[..i].trim();
                        if !class_name.is_empty() {
                            return Some(class_name.to_string());
                        }
                        break;
                    }
                }
                
                // If no special chars found, take the whole thing
                let class_name = after_class.trim();
                if !class_name.is_empty() {
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Extract function signature (type hints)
    fn extract_function_signature(&self, _node: &tree_sitter::Node) -> Option<String> {
        // TODO: Implement proper signature extraction for Python type hints
        // This would involve parsing parameters and return type annotations
        None
    }

    /// Check if a node is inside a class
    fn is_inside_class(&self, node: &tree_sitter::Node) -> bool {
        let mut parent = node.parent();
        while let Some(p) = parent {
            if p.kind() == "class_definition" {
                return true;
            }
            parent = p.parent();
        }
        false
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

    /// Find parent scope ID (function, method, class, or module)
    fn find_parent_scope_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut parent = node.parent();
        while let Some(p) = parent {
            if let Some(id) = self.node_map.get(&p.id()) {
                return Some(*id);
            }
            parent = p.parent();
        }
        self.find_module_node_id()
    }

    /// Find enclosing scope ID (for variable declarations)
    fn find_enclosing_scope_id(&self) -> Option<crate::types::NodeId> {
        // For now, just return module ID
        // TODO: Implement proper scope detection
        self.find_module_node_id()
    }

    /// Find containing function ID
    fn find_containing_function_id(&self, node: &tree_sitter::Node) -> Option<crate::types::NodeId> {
        let mut parent = node.parent();
        while let Some(p) = parent {
            match p.kind() {
                "function_definition" => {
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

    /// Extract base classes and create inheritance edges
    fn extract_base_classes(&mut self, node: &tree_sitter::Node, class_id: crate::types::NodeId) -> Result<()> {
        // Look for argument_list node which contains base classes
        if let Some(arg_list) = node.child_by_field_name("superclasses") {
            self.parse_base_class_list(&arg_list, class_id, node)?;
        } else {
            // Fallback: search for argument_list in children
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "argument_list" {
                        self.parse_base_class_list(&child, class_id, node)?;
                        break;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse base class list and create inheritance nodes
    fn parse_base_class_list(&mut self, arg_list: &tree_sitter::Node, class_id: crate::types::NodeId, class_node: &tree_sitter::Node) -> Result<()> {
        let mut cursor = arg_list.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "identifier" => {
                        // Simple base class: class Child(Parent):
                        let base_class = self.get_node_text(&child);
                        if !base_class.trim().is_empty() {
                            self.create_inheritance_edge(class_id, base_class, &child)?;
                        }
                    }
                    "attribute" => {
                        // Qualified base class: class Child(module.Parent):
                        // Extract just the class name from qualified name
                        if let Some(attr_node) = child.child_by_field_name("attribute") {
                            let base_class = self.get_node_text(&attr_node);
                            if !base_class.trim().is_empty() {
                                self.create_inheritance_edge(class_id, base_class, &child)?;
                            }
                        } else {
                            let base_class = self.get_node_text(&child);
                            if !base_class.trim().is_empty() {
                                self.create_inheritance_edge(class_id, base_class, &child)?;
                            }
                        }
                    }
                    "subscript" => {
                        // Generic base class: class Child(Generic[T]):
                        // Extract the base part before the subscript
                        if let Some(value_node) = child.child_by_field_name("value") {
                            let base_class = self.extract_call_target(&value_node);
                            if !base_class.trim().is_empty() && 
                               base_class != "anonymous_call" && 
                               base_class != "complex_call" {
                                self.create_inheritance_edge(class_id, base_class, &child)?;
                            }
                        }
                    }
                    _ => {
                        // Skip other types (like parentheses, commas, etc.)
                        // Don't try to create inheritance edges for these
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Create an inheritance edge (child extends parent)
    fn create_inheritance_edge(&mut self, child_class_id: crate::types::NodeId, parent_class_name: String, base_class_node: &tree_sitter::Node) -> Result<()> {
        // Use the actual span of the base class node to ensure unique IDs
        let span = Span::from_node(base_class_node);

        // Create a virtual node representing the inheritance relationship
        let inheritance_node = Node::new(
            &self.repo_id,
            NodeKind::Call, // Use Call kind to represent inheritance usage
            parent_class_name,
            self.language,
            self.file_path.clone(),
            span,
        );

        // Create edge from child class to parent class reference
        self.edges.push(Edge::new(child_class_id, inheritance_node.id, EdgeKind::Calls));

        self.nodes.push(inheritance_node);
        Ok(())
    }
} 