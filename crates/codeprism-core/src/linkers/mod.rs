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

    fn find_edges(&self, nodes: &[Node]) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find all route nodes
        let routes: Vec<&Node> = nodes
            .iter()
            .filter(|node| matches!(node.kind, crate::ast::NodeKind::Route))
            .collect();

        // Find all function/method nodes
        let functions: Vec<&Node> = nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.kind,
                    crate::ast::NodeKind::Function | crate::ast::NodeKind::Method
                )
            })
            .collect();

        // Try to link routes to controllers/functions
        for route in routes {
            if let Some(controller) = self.find_matching_controller(route, &functions) {
                edges.push(Edge::new(
                    route.id,
                    controller,
                    crate::ast::EdgeKind::RoutesTo,
                ));
            }
        }

        Ok(edges)
    }
}

impl RestLinker {
    /// Find controller/function that matches a REST route
    fn find_matching_controller(
        &self,
        route: &Node,
        functions: &[&Node],
    ) -> Option<crate::ast::NodeId> {
        let route_name = &route.name;

        // Extract HTTP method and path from route name
        let (method, path) = if let Some((m, p)) = route_name.split_once(' ') {
            (m.to_lowercase(), p)
        } else {
            ("get".to_string(), route_name.as_str())
        };

        // Look for functions with names that match the route pattern
        for function in functions {
            let func_name = function.name.to_lowercase();

            // Check for common REST controller naming patterns
            if self.matches_rest_pattern(&method, path, &func_name) {
                return Some(function.id);
            }
        }

        None
    }

    /// Check if a function name matches REST patterns
    fn matches_rest_pattern(&self, method: &str, path: &str, func_name: &str) -> bool {
        // Remove leading slash and convert to lowercase
        let clean_path = path.trim_start_matches('/').to_lowercase();
        let path_segments: Vec<&str> = clean_path.split('/').collect();

        // Pattern 1: methodResourceName (e.g., getUsers, postUser)
        if func_name.starts_with(method) {
            let resource_part = &func_name[method.len()..];
            if !resource_part.is_empty() {
                let resource_lower = resource_part.to_lowercase();
                return path_segments.iter().any(|segment| {
                    segment.contains(&resource_lower) || resource_lower.contains(segment)
                });
            }
        }

        // Pattern 2: resourceMethod (e.g., usersGet, userPost)
        for segment in &path_segments {
            if func_name.contains(segment) && func_name.contains(method) {
                return true;
            }
        }

        // Pattern 3: exact path segment match
        if path_segments
            .iter()
            .any(|segment| func_name.contains(segment))
        {
            return true;
        }

        // Pattern 4: controller class + method pattern
        if func_name.contains("controller") || func_name.contains("handler") {
            return path_segments
                .iter()
                .any(|segment| func_name.contains(segment));
        }

        false
    }
}

/// SQL query linker
pub struct SqlLinker;

impl Linker for SqlLinker {
    fn name(&self) -> &str {
        "SQL"
    }

    fn find_edges(&self, nodes: &[Node]) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();

        // Find all SQL query nodes
        let sql_queries: Vec<&Node> = nodes
            .iter()
            .filter(|node| matches!(node.kind, crate::ast::NodeKind::SqlQuery))
            .collect();

        // Find all potential table/class nodes that might represent database tables
        let table_candidates: Vec<&Node> = nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.kind,
                    crate::ast::NodeKind::Class | crate::ast::NodeKind::Variable
                )
            })
            .collect();

        // Try to link SQL queries to table references
        for query in sql_queries {
            let referenced_tables = self.extract_table_references(&query.name);

            for table_name in referenced_tables {
                if let Some(table_node_id) =
                    self.find_matching_table(&table_name, &table_candidates)
                {
                    edges.push(Edge::new(
                        query.id,
                        table_node_id,
                        crate::ast::EdgeKind::Reads, // SQL queries typically read from tables
                    ));
                }
            }
        }

        Ok(edges)
    }
}

impl SqlLinker {
    /// Extract table names from SQL query text
    fn extract_table_references(&self, query_text: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let query_lower = query_text.to_lowercase();

        // Look for common SQL patterns to extract table names
        self.extract_from_clause(&query_lower, &mut tables);
        self.extract_join_clause(&query_lower, &mut tables);
        self.extract_insert_into(&query_lower, &mut tables);
        self.extract_update_table(&query_lower, &mut tables);
        self.extract_delete_from(&query_lower, &mut tables);

        // Remove duplicates and empty strings
        tables.sort();
        tables.dedup();
        tables.retain(|t| !t.is_empty());

        tables
    }

    /// Extract tables from FROM clause
    fn extract_from_clause(&self, query: &str, tables: &mut Vec<String>) {
        if let Some(from_pos) = query.find(" from ") {
            let after_from = &query[from_pos + 6..];
            if let Some(table_name) = self.extract_first_word(after_from) {
                tables.push(table_name);
            }
        }
    }

    /// Extract tables from JOIN clauses
    fn extract_join_clause(&self, query: &str, tables: &mut Vec<String>) {
        let join_keywords = [
            " join ",
            " inner join ",
            " left join ",
            " right join ",
            " outer join ",
        ];

        for join_keyword in &join_keywords {
            let mut start = 0;
            while let Some(join_pos) = query[start..].find(join_keyword) {
                let absolute_pos = start + join_pos + join_keyword.len();
                if let Some(table_name) = self.extract_first_word(&query[absolute_pos..]) {
                    tables.push(table_name);
                }
                start = absolute_pos;
            }
        }
    }

    /// Extract table from INSERT INTO clause
    fn extract_insert_into(&self, query: &str, tables: &mut Vec<String>) {
        if let Some(insert_pos) = query.find("insert into ") {
            let after_insert = &query[insert_pos + 12..];
            if let Some(table_name) = self.extract_first_word(after_insert) {
                tables.push(table_name);
            }
        }
    }

    /// Extract table from UPDATE clause
    fn extract_update_table(&self, query: &str, tables: &mut Vec<String>) {
        if let Some(update_pos) = query.find("update ") {
            let after_update = &query[update_pos + 7..];
            if let Some(table_name) = self.extract_first_word(after_update) {
                tables.push(table_name);
            }
        }
    }

    /// Extract table from DELETE FROM clause
    fn extract_delete_from(&self, query: &str, tables: &mut Vec<String>) {
        if let Some(delete_pos) = query.find("delete from ") {
            let after_delete = &query[delete_pos + 12..];
            if let Some(table_name) = self.extract_first_word(after_delete) {
                tables.push(table_name);
            }
        }
    }

    /// Extract the first word (table name) from a string, stopping at SQL keywords or punctuation
    fn extract_first_word(&self, text: &str) -> Option<String> {
        let stop_words = [
            "where", "order", "group", "having", "limit", "on", "as", "set",
        ];
        let text = text.trim();

        if text.is_empty() {
            return None;
        }

        // Find the end of the first word
        let mut end_pos = 0;
        for (i, ch) in text.char_indices() {
            if ch.is_whitespace() || ch == '(' || ch == ',' || ch == ';' {
                end_pos = i;
                break;
            }
            end_pos = i + ch.len_utf8();
        }

        if end_pos == 0 {
            return None;
        }

        let word = &text[..end_pos];

        // Don't return SQL keywords as table names
        if stop_words.contains(&word) {
            return None;
        }

        Some(word.to_string())
    }

    /// Find a table node that matches the given table name
    fn find_matching_table(
        &self,
        table_name: &str,
        candidates: &[&Node],
    ) -> Option<crate::ast::NodeId> {
        let table_lower = table_name.to_lowercase();

        for candidate in candidates {
            let candidate_name = candidate.name.to_lowercase();

            // Exact match
            if candidate_name == table_lower {
                return Some(candidate.id);
            }

            // Check for common ORM/model naming patterns
            // Table: users -> Model: User or UserModel
            if self.matches_table_pattern(&table_lower, &candidate_name) {
                return Some(candidate.id);
            }
        }

        None
    }

    /// Check if a candidate name matches table naming patterns
    fn matches_table_pattern(&self, table_name: &str, candidate_name: &str) -> bool {
        // Pattern 1: Pluralized table -> Singular model (users -> user)
        if table_name.ends_with('s') && !table_name.ends_with("ss") {
            let singular = &table_name[..table_name.len() - 1];
            if candidate_name == singular || candidate_name == format!("{}model", singular) {
                return true;
            }
        }

        // Pattern 2: Snake_case table -> CamelCase model (user_profiles -> userprofile, userprofilemodel)
        let snake_removed = table_name.replace('_', "");
        if candidate_name.contains(&snake_removed) {
            return true;
        }

        // Pattern 3: Table contains candidate name or vice versa
        if table_name.contains(candidate_name) || candidate_name.contains(table_name) {
            return true;
        }

        // Pattern 4: Model/Entity suffix patterns
        if candidate_name.ends_with("model")
            || candidate_name.ends_with("entity")
            || candidate_name.ends_with("table")
        {
            let base_name = candidate_name
                .trim_end_matches("model")
                .trim_end_matches("entity")
                .trim_end_matches("table");
            if table_name.contains(base_name) || base_name.contains(table_name) {
                return true;
            }
        }

        false
    }
}
