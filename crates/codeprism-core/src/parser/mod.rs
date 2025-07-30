//! Parser engine for incremental parsing

use crate::ast::{Language, Node};
use crate::error::{Error, Result};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tree_sitter::Tree;

/// Parser context for incremental parsing
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Repository ID
    pub repo_id: String,
    /// File path being parsed
    pub file_path: PathBuf,
    /// Previous tree for incremental parsing
    pub old_tree: Option<Tree>,
    /// File content
    pub content: String,
}

impl ParseContext {
    /// Create a new parse context
    pub fn new(repo_id: String, file_path: PathBuf, content: String) -> Self {
        Self {
            repo_id,
            file_path,
            content,
            old_tree: None,
        }
    }

    /// Set the old tree for incremental parsing
    pub fn with_old_tree(mut self, tree: Tree) -> Self {
        self.old_tree = Some(tree);
        self
    }
}

/// Language parser trait
pub trait LanguageParser: Send + Sync {
    /// Get the language this parser handles
    fn language(&self) -> Language;

    /// Parse a file and extract nodes and edges
    fn parse(&self, context: &ParseContext) -> Result<ParseResult>;
}

/// Result of parsing a file
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed tree
    pub tree: Tree,
    /// Extracted nodes
    pub nodes: Vec<Node>,
    /// Extracted edges
    pub edges: Vec<crate::ast::Edge>,
}

/// Registry for language parsers
pub struct LanguageRegistry {
    parsers: DashMap<Language, Arc<dyn LanguageParser>>,
}

impl LanguageRegistry {
    /// Create a new language registry
    pub fn new() -> Self {
        Self {
            parsers: DashMap::new(),
        }
    }

    /// Register a language parser
    pub fn register(&self, parser: Arc<dyn LanguageParser>) {
        let lang = parser.language();
        self.parsers.insert(lang, parser);
    }

    /// Get a parser for a language
    pub fn get(&self, language: Language) -> Option<Arc<dyn LanguageParser>> {
        self.parsers.get(&language).map(|p| Arc::clone(&*p))
    }

    /// Get a parser for a file extension
    pub fn get_by_extension(&self, ext: &str) -> Option<Arc<dyn LanguageParser>> {
        let lang = Language::from_extension(ext);
        self.get(lang)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Main parser engine
pub struct ParserEngine {
    /// Language registry
    registry: Arc<LanguageRegistry>,
    /// Cache of parsed trees
    tree_cache: DashMap<PathBuf, Tree>,
}

impl ParserEngine {
    /// Create a new parser engine
    pub fn new(registry: Arc<LanguageRegistry>) -> Self {
        Self {
            registry,
            tree_cache: DashMap::new(),
        }
    }

    /// Parse a file
    pub fn parse_file(&self, context: ParseContext) -> Result<ParseResult> {
        // Detect language from file extension
        let ext = context
            .file_path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::parse(&context.file_path, "No file extension"))?;

        // Get the appropriate parser
        let parser = self
            .registry
            .get_by_extension(ext)
            .ok_or_else(|| Error::unsupported_language(ext.to_string()))?;

        // Parse the file
        let result = parser.parse(&context)?;

        // Cache the tree
        self.tree_cache
            .insert(context.file_path.clone(), result.tree.clone());

        Ok(result)
    }

    /// Parse a file incrementally
    pub fn parse_incremental(&self, mut context: ParseContext) -> Result<ParseResult> {
        // Try to get the old tree from cache
        if context.old_tree.is_none() {
            if let Some(old_tree) = self.tree_cache.get(&context.file_path) {
                context.old_tree = Some(old_tree.clone());
            }
        }

        self.parse_file(context)
    }

    /// Clear the tree cache
    pub fn clear_cache(&self) {
        self.tree_cache.clear();
    }

    /// Remove a specific file from the cache
    pub fn remove_from_cache(&self, path: &Path) {
        self.tree_cache.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Edge, EdgeKind, NodeKind, Span};
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock parser for testing
    struct MockParser {
        language: Language,
        parse_count: Arc<AtomicUsize>,
    }

    impl MockParser {
        fn new(language: Language) -> Self {
            Self {
                language,
                parse_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn parse_count(&self) -> usize {
            self.parse_count.load(Ordering::SeqCst)
        }
    }

    impl LanguageParser for MockParser {
        fn language(&self) -> Language {
            self.language
        }

        fn parse(&self, context: &ParseContext) -> Result<ParseResult> {
            self.parse_count.fetch_add(1, Ordering::SeqCst);

            // Create a real tree using tree-sitter
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .unwrap();
            let tree = parser.parse(&context.content, None).unwrap();

            // Create mock nodes based on content
            let mut nodes = Vec::new();
            let mut edges = Vec::new();

            // Simple mock: create a module node and a function node if "function" is in content
            let module_span = Span::new(0, context.content.len(), 1, 1, 1, 1);
            let module_node = crate::ast::Node::new(
                &context.repo_id,
                NodeKind::Module,
                context.file_path.to_string_lossy().to_string(),
                self.language,
                context.file_path.clone(),
                module_span,
            );
            nodes.push(module_node.clone());

            if context.content.contains("function") {
                let func_span = Span::new(0, 8, 1, 1, 1, 9);
                let func_node = crate::ast::Node::new(
                    &context.repo_id,
                    NodeKind::Function,
                    "testFunction".to_string(),
                    self.language,
                    context.file_path.clone(),
                    func_span,
                );
                nodes.push(func_node.clone());

                // Add an edge from module to function
                edges.push(Edge::new(module_node.id, func_node.id, EdgeKind::Calls));
            }

            Ok(ParseResult { tree, nodes, edges })
        }
    }

    #[test]
    fn test_language_registry() {
        let registry = LanguageRegistry::new();
        assert!(registry.get(Language::JavaScript).is_none());

        // Register a mock parser
        let parser = Arc::new(MockParser::new(Language::JavaScript));
        registry.register(parser.clone());

        // Test direct language lookup with functionality validation
        assert!(
            registry.get(Language::JavaScript).is_some(),
            "JavaScript parser should be registered"
        );
        let js_parser = registry.get(Language::JavaScript).unwrap();
        assert_eq!(
            js_parser.language(),
            Language::JavaScript,
            "Parser should return correct language"
        );
        // Verify we get the same language (ptr_eq doesn't work with trait objects)
        assert_eq!(
            js_parser.language(),
            parser.language(),
            "Should return parser with same language"
        );

        // Test extension lookup with functionality validation
        assert!(
            registry.get_by_extension("js").is_some(),
            "Should find parser by .js extension"
        );
        let js_parser_by_ext = registry.get_by_extension("js").unwrap();
        assert_eq!(
            js_parser_by_ext.language(),
            Language::JavaScript,
            "Extension lookup should return JavaScript parser"
        );
        assert!(
            registry.get_by_extension("ts").is_none(),
            "Should not find parser for unregistered .ts extension"
        );
    }

    #[test]
    fn test_parse_context() {
        let context = ParseContext::new(
            "test_repo".to_string(),
            PathBuf::from("test.js"),
            "console.log('hello');".to_string(),
        );

        assert_eq!(context.repo_id, "test_repo");
        assert_eq!(context.file_path, PathBuf::from("test.js"));
        assert!(context.old_tree.is_none(), "Should be none");
    }

    #[test]
    fn test_parser_engine_basic() {
        let registry = Arc::new(LanguageRegistry::new());
        let parser = Arc::new(MockParser::new(Language::JavaScript));
        registry.register(parser.clone());

        let engine = ParserEngine::new(registry);
        let context = ParseContext::new(
            "test_repo".to_string(),
            PathBuf::from("test.js"),
            "function hello() {}".to_string(),
        );

        let result = engine.parse_file(context).unwrap();
        assert_eq!(result.nodes.len(), 2, "Should have 2 items"); // Module + Function
        assert_eq!(result.edges.len(), 1, "Should have 1 items"); // Module -> Function
        assert_eq!(parser.parse_count(), 1);
    }

    #[test]
    fn test_parser_engine_unsupported_language() {
        let registry = Arc::new(LanguageRegistry::new());
        let engine = ParserEngine::new(registry);

        let context = ParseContext::new(
            "test_repo".to_string(),
            PathBuf::from("test.unknown"),
            "some content".to_string(),
        );

        let result = engine.parse_file(context);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Validation { field, message, .. } => {
                assert_eq!(field, "language");
                assert!(message.contains("unknown"));
            }
            _ => panic!("Expected Validation error for unsupported language"),
        }
    }

    #[test]
    fn test_parser_engine_no_extension() {
        let registry = Arc::new(LanguageRegistry::new());
        let engine = ParserEngine::new(registry);

        let context = ParseContext::new(
            "test_repo".to_string(),
            PathBuf::from("README"),
            "some content".to_string(),
        );

        let result = engine.parse_file(context);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Parse { file, message, .. } => {
                assert_eq!(file, PathBuf::from("README"));
                assert!(message.contains("No file extension"));
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_parser_engine_caching() {
        let registry = Arc::new(LanguageRegistry::new());
        let parser = Arc::new(MockParser::new(Language::JavaScript));
        registry.register(parser.clone());

        let engine = ParserEngine::new(registry);
        let file_path = PathBuf::from("test.js");

        // First parse
        let context1 = ParseContext::new(
            "test_repo".to_string(),
            file_path.clone(),
            "function one() {}".to_string(),
        );
        let _result1 = engine.parse_file(context1).unwrap();

        // Second parse - should use cached tree for incremental
        let context2 = ParseContext::new(
            "test_repo".to_string(),
            file_path.clone(),
            "function two() {}".to_string(),
        );
        let result2 = engine.parse_incremental(context2).unwrap();

        assert_eq!(result2.nodes.len(), 2, "Should have 2 items");
        assert_eq!(parser.parse_count(), 2); // Both parses executed
    }

    #[test]
    fn test_parser_engine_cache_management() {
        let registry = Arc::new(LanguageRegistry::new());
        registry.register(Arc::new(MockParser::new(Language::JavaScript)));

        let engine = ParserEngine::new(registry);
        let file_path = PathBuf::from("test.js");

        // Parse a file
        let context = ParseContext::new(
            "test_repo".to_string(),
            file_path.clone(),
            "function test() {}".to_string(),
        );
        let _result = engine.parse_file(context).unwrap();

        // Remove from cache
        engine.remove_from_cache(&file_path);

        // Clear entire cache
        engine.clear_cache();

        // Test passes if no panic
    }

    #[test]
    fn test_parse_result_validation() {
        let registry = Arc::new(LanguageRegistry::new());
        registry.register(Arc::new(MockParser::new(Language::JavaScript)));

        let engine = ParserEngine::new(registry);
        let context = ParseContext::new(
            "test_repo".to_string(),
            PathBuf::from("test.js"),
            "const x = 42;".to_string(),
        );

        let result = engine.parse_file(context).unwrap();

        // Validate nodes
        assert!(!!result.nodes.is_empty(), "Should not be empty");
        for node in &result.nodes {
            assert!(!!node.name.is_empty(), "Should not be empty");
            assert_eq!(node.lang, Language::JavaScript);
        }

        // Validate edges
        for edge in &result.edges {
            // Ensure edge endpoints exist in nodes
            let source_exists = result.nodes.iter().any(|n| n.id == edge.source);
            let target_exists = result.nodes.iter().any(|n| n.id == edge.target);
            assert!(source_exists || target_exists); // At least one should exist in our mock
        }
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let registry = Arc::new(LanguageRegistry::new());
        registry.register(Arc::new(MockParser::new(Language::JavaScript)));
        registry.register(Arc::new(MockParser::new(Language::Python)));

        let engine = Arc::new(ParserEngine::new(registry));

        let mut handles = vec![];

        // Spawn multiple threads parsing different files
        for i in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                let ext = if i % 2 == 0 { "js" } else { "py" };
                let context = ParseContext::new(
                    "test_repo".to_string(),
                    PathBuf::from(format!("test{i}.{ext}")),
                    format!("function test{i}() {{}}"),
                );
                engine_clone.parse_file(context).unwrap()
            });
            handles.push(handle);
        }

        // Wait for all threads and verify results
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(!!result.nodes.is_empty(), "Should not be empty");
        }
    }
}
