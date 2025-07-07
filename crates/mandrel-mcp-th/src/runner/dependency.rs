//! Dependency resolution for test case execution ordering

use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// Resolves test case dependencies and determines execution order
#[derive(Debug, Default)]
pub struct DependencyResolver {
    dependency_graph: HashMap<String, Vec<String>>,
    execution_order: Vec<String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve dependencies and return execution order
    ///
    /// Performs topological sort to determine the correct execution order
    /// for test cases based on their dependencies.
    ///
    /// # Arguments
    /// * `test_cases` - Map of test case names to their dependencies
    ///
    /// # Returns
    /// Ordered list of test case names for execution
    ///
    /// # Errors
    /// Returns `Error::Dependency` if circular dependencies are detected
    pub fn resolve_dependencies(
        &mut self,
        test_cases: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>> {
        // Phase 1: Validate input dependencies
        self.validate_dependencies(test_cases)?;

        self.dependency_graph = test_cases.clone();

        // Phase 2: Check for circular dependencies
        if let Some(cycle) = self.detect_circular_dependencies() {
            return Err(Error::dependency(format!(
                "Circular dependency detected: {}",
                cycle.join(" -> ")
            )));
        }

        // Phase 3: Perform topological sort
        self.execution_order = self.topological_sort()?;
        Ok(self.execution_order.clone())
    }

    /// Validates all dependency specifications for correctness
    ///
    /// # Validation Rules
    /// - No empty dependency names
    /// - No self-dependencies  
    /// - All dependency references must exist in test suite
    /// - No null or malformed dependency entries
    ///
    /// # Errors
    /// - `Error::Dependency` with specific validation failure details
    fn validate_dependencies(&self, test_cases: &HashMap<String, Vec<String>>) -> Result<()> {
        for (test_name, dependencies) in test_cases {
            for dependency in dependencies {
                // Rule 1: No empty names
                if dependency.is_empty() {
                    return Err(Error::dependency(format!(
                        "Empty dependency name for test '{}'",
                        test_name
                    )));
                }

                // Rule 2: No self-dependencies
                if dependency == test_name {
                    return Err(Error::dependency(format!(
                        "Self-dependency detected for test '{}'",
                        test_name
                    )));
                }

                // Rule 3: Dependency must exist
                if !test_cases.contains_key(dependency) {
                    return Err(Error::dependency(format!(
                        "Unknown dependency '{}' for test '{}'",
                        dependency, test_name
                    )));
                }
            }
        }
        Ok(())
    }

    /// Detect circular dependencies in the dependency graph
    ///
    /// Uses depth-first search with a recursion stack to detect cycles
    ///
    /// # Returns
    /// Some(cycle) if a circular dependency is found, None otherwise
    pub fn detect_circular_dependencies(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.dependency_graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) =
                    self.dfs_cycle_detection(node, &mut visited, &mut recursion_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    /// Depth-first search for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(dependencies) = self.dependency_graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if let Some(cycle) =
                        self.dfs_cycle_detection(dep, visited, recursion_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(dep) {
                    // Found a cycle - return the cycle path
                    let cycle_start = path.iter().position(|x| x == dep).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(dep.to_string()); // Complete the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        recursion_stack.remove(node);
        None
    }

    /// Perform topological sort using Kahn's algorithm
    fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Initialize in-degree count for all nodes
        for node in self.dependency_graph.keys() {
            in_degree.insert(node.clone(), 0);
        }

        // Calculate in-degrees correctly
        // If test_cases has "test2" -> ["test1"], it means test2 depends on test1
        // So test2 should have in-degree 1, not test1
        for (node, dependencies) in &self.dependency_graph {
            let dependency_count = dependencies.len();
            if let Some(degree) = in_degree.get_mut(node) {
                *degree = dependency_count;
            }
        }

        // Add nodes with no dependencies to queue
        for (node, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process nodes in topological order
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            // For each node that depends on the current node, reduce its in-degree
            // Find all nodes that have the current node as a dependency
            for (dependent_node, dependencies) in &self.dependency_graph {
                if dependencies.contains(&node) {
                    if let Some(degree) = in_degree.get_mut(dependent_node) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent_node.clone());
                        }
                    }
                }
            }
        }

        // Check if all nodes were processed (no cycles)
        if result.len() != self.dependency_graph.len() {
            return Err(Error::dependency(
                "Failed to resolve dependencies - circular dependencies remain after validation"
                    .to_string(),
            ));
        }

        Ok(result)
    }

    /// Get the current execution order
    pub fn get_execution_order(&self) -> &[String] {
        &self.execution_order
    }

    /// Group test cases by dependency level for parallel execution
    ///
    /// Groups tests that can be executed in parallel (same dependency level)
    pub fn group_by_dependency_level(&self) -> Vec<Vec<String>> {
        let mut levels = Vec::new();
        let mut remaining: HashSet<String> = self.dependency_graph.keys().cloned().collect();
        let mut satisfied_dependencies = HashSet::new();

        while !remaining.is_empty() {
            let mut current_level = Vec::new();

            // Find all nodes whose dependencies are satisfied
            for node in &remaining {
                let dependencies = self.dependency_graph.get(node).unwrap();
                if dependencies
                    .iter()
                    .all(|dep| satisfied_dependencies.contains(dep))
                {
                    current_level.push(node.clone());
                }
            }

            if current_level.is_empty() {
                // This shouldn't happen if topological sort succeeded
                break;
            }

            // Remove processed nodes and mark their dependencies as satisfied
            for node in &current_level {
                remaining.remove(node);
                satisfied_dependencies.insert(node.clone());
            }

            levels.push(current_level);
        }

        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // PHASE 1: ENHANCED VALIDATION TESTS (RED PHASE - FAILING TESTS FIRST)
    // ========================================================================

    #[test]
    fn test_validate_empty_dependency_name() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec!["".to_string()]); // Empty dependency name

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(
            result.is_err(),
            "Empty dependency name should cause validation error"
        );

        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        assert!(
            error.to_string().contains("Empty dependency name"),
            "Error should mention empty dependency: {}",
            error
        );
    }

    #[test]
    fn test_validate_self_dependency() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec!["test1".to_string()]); // Self-dependency

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(
            result.is_err(),
            "Self-dependency should cause validation error"
        );

        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        assert!(
            error.to_string().contains("Self-dependency"),
            "Error should mention self-dependency: {}",
            error
        );
    }

    #[test]
    fn test_validate_non_existent_dependency() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec!["non_existent_test".to_string()]);

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(
            result.is_err(),
            "Non-existent dependency should cause validation error"
        );

        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        assert!(
            error.to_string().contains("Unknown dependency"),
            "Error should mention unknown dependency: {}",
            error
        );
    }

    #[test]
    fn test_validate_multiple_validation_errors() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert(
            "test1".to_string(),
            vec!["".to_string(), "test1".to_string()],
        ); // Both empty and self-dependency

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(
            result.is_err(),
            "Multiple validation errors should cause error"
        );

        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        // Should fail on first validation error (empty dependency name)
        assert!(
            error.to_string().contains("Empty dependency name"),
            "Should fail on first validation error: {}",
            error
        );
    }

    #[test]
    fn test_validate_complex_dependency_mix() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec!["test2".to_string()]);
        test_cases.insert("test2".to_string(), vec!["missing_test".to_string()]); // Non-existent dependency

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(
            result.is_err(),
            "Non-existent dependency in chain should cause validation error"
        );

        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        assert!(
            error
                .to_string()
                .contains("Unknown dependency 'missing_test'"),
            "Error should identify the missing dependency: {}",
            error
        );
    }

    // ========================================================================
    // EXISTING TESTS (These should still work after validation is added)
    // ========================================================================

    #[test]
    fn test_simple_dependency_resolution() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec![]);
        test_cases.insert("test2".to_string(), vec!["test1".to_string()]);
        test_cases.insert("test3".to_string(), vec!["test2".to_string()]);

        let order = resolver.resolve_dependencies(&test_cases).unwrap();
        assert_eq!(order, vec!["test1", "test2", "test3"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec!["test2".to_string()]);
        test_cases.insert("test2".to_string(), vec!["test1".to_string()]);

        let result = resolver.resolve_dependencies(&test_cases);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_parallel_grouping() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec![]);
        test_cases.insert("test2".to_string(), vec![]);
        test_cases.insert("test3".to_string(), vec!["test1".to_string()]);
        test_cases.insert("test4".to_string(), vec!["test2".to_string()]);

        resolver.resolve_dependencies(&test_cases).unwrap();
        let groups = resolver.group_by_dependency_level();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 2); // test1 and test2 can run in parallel
        assert_eq!(groups[1].len(), 2); // test3 and test4 can run in parallel
    }

    #[test]
    fn test_no_dependencies() {
        let mut resolver = DependencyResolver::new();
        let mut test_cases = HashMap::new();

        test_cases.insert("test1".to_string(), vec![]);
        test_cases.insert("test2".to_string(), vec![]);
        test_cases.insert("test3".to_string(), vec![]);

        let order = resolver.resolve_dependencies(&test_cases).unwrap();
        assert_eq!(order.len(), 3);
        // Order can vary for independent tests
        assert!(order.contains(&"test1".to_string()));
        assert!(order.contains(&"test2".to_string()));
        assert!(order.contains(&"test3".to_string()));
    }
}
