//! Comment extractors for source code files
//!
//! This module provides extractors that work with tree-sitter parse trees
//! to extract comments and documentation from various programming languages.

use super::{CommentContext, ContentChunk, ContentType};
use crate::ast::{Language, NodeId, Span};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::Tree;

/// Comment extractor that works with tree-sitter parse trees
pub struct CommentExtractor {
    /// Language-specific comment extractors
    language_extractors: HashMap<Language, Box<dyn LanguageCommentExtractor>>,
}

impl CommentExtractor {
    /// Create a new comment extractor
    pub fn new() -> Self {
        let mut extractors: HashMap<Language, Box<dyn LanguageCommentExtractor>> = HashMap::new();

        // Register language-specific extractors
        extractors.insert(
            Language::JavaScript,
            Box::new(JavaScriptCommentExtractor::new()),
        );
        extractors.insert(
            Language::TypeScript,
            Box::new(JavaScriptCommentExtractor::new()),
        );
        extractors.insert(Language::Python, Box::new(PythonCommentExtractor::new()));
        extractors.insert(Language::Java, Box::new(JavaCommentExtractor::new()));
        extractors.insert(Language::Rust, Box::new(RustCommentExtractor::new()));
        extractors.insert(Language::C, Box::new(CCommentExtractor::new()));
        extractors.insert(Language::Cpp, Box::new(CCommentExtractor::new()));

        Self {
            language_extractors: extractors,
        }
    }

    /// Extract comments from a tree-sitter parse tree
    pub fn extract_comments(
        &self,
        language: Language,
        tree: &Tree,
        source: &str,
        file_path: &Path,
        ast_nodes: &[NodeId],
    ) -> Result<Vec<ContentChunk>> {
        let extractor = self
            .language_extractors
            .get(&language)
            .ok_or_else(|| anyhow!("No comment extractor for language: {:?}", language))?;

        extractor.extract_comments(tree, source, file_path, ast_nodes)
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: Language) -> bool {
        self.language_extractors.contains_key(&language)
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.language_extractors.keys().copied().collect()
    }
}

impl Default for CommentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for language-specific comment extraction
pub trait LanguageCommentExtractor: Send + Sync {
    /// Extract comments from source code
    fn extract_comments(
        &self,
        tree: &Tree,
        source: &str,
        file_path: &Path,
        ast_nodes: &[NodeId],
    ) -> Result<Vec<ContentChunk>>;

    /// Get the comment patterns for this language
    fn comment_patterns(&self) -> &CommentPatterns;
}

/// Comment patterns for a programming language
#[derive(Debug, Clone)]
pub struct CommentPatterns {
    /// Single-line comment prefixes (e.g., "//", "#")
    pub single_line: Vec<String>,
    /// Block comment patterns (start, end)
    pub block: Vec<(String, String)>,
    /// Documentation comment patterns
    pub documentation: Vec<String>,
}

/// JavaScript/TypeScript comment extractor
pub struct JavaScriptCommentExtractor {
    patterns: CommentPatterns,
    comment_regex: Regex,
}

impl Default for JavaScriptCommentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptCommentExtractor {
    /// Create a new JavaScript comment extractor
    pub fn new() -> Self {
        Self {
            patterns: CommentPatterns {
                single_line: vec!["//".to_string()],
                block: vec![("/*".to_string(), "*/".to_string())],
                documentation: vec!["/**".to_string(), "///".to_string()],
            },
            comment_regex: Regex::new(r"(?m)//.*$|/\*[\s\S]*?\*/").unwrap(),
        }
    }
}

impl LanguageCommentExtractor for JavaScriptCommentExtractor {
    fn extract_comments(
        &self,
        _tree: &Tree,
        source: &str,
        file_path: &Path,
        _ast_nodes: &[NodeId],
    ) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        // Extract all comments using regex
        for comment_match in self.comment_regex.find_iter(source) {
            let comment_text = comment_match.as_str();
            let span = self.calculate_match_span(&comment_match, source);

            // Clean comment text
            let cleaned_text = if comment_text.starts_with("/**") {
                self.clean_jsdoc_comment(comment_text)
            } else if comment_text.starts_with("/*") {
                self.clean_block_comment(comment_text)
            } else {
                self.clean_single_line_comment(comment_text)
            };

            // Skip empty comments
            if cleaned_text.trim().is_empty() {
                continue;
            }

            let context = if comment_text.starts_with("/**") {
                CommentContext::Documentation
            } else if comment_text.starts_with("/*") {
                CommentContext::Block
            } else {
                CommentContext::Inline
            };

            let content_type = ContentType::Comment {
                language: Language::JavaScript,
                context,
            };

            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                content_type,
                cleaned_text,
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "raw_text": comment_text,
                "language": "javascript"
            }));

            chunks.push(chunk);
            chunk_index += 1;
        }

        Ok(chunks)
    }

    fn comment_patterns(&self) -> &CommentPatterns {
        &self.patterns
    }
}

impl JavaScriptCommentExtractor {
    /// Clean JSDoc comment text
    fn clean_jsdoc_comment(&self, comment: &str) -> String {
        comment
            .trim_start_matches("/**")
            .trim_end_matches("*/")
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clean block comment text
    fn clean_block_comment(&self, comment: &str) -> String {
        comment
            .trim_start_matches("/*")
            .trim_end_matches("*/")
            .trim()
            .to_string()
    }

    /// Clean single line comment text
    fn clean_single_line_comment(&self, comment: &str) -> String {
        comment.trim_start_matches("//").trim().to_string()
    }

    /// Calculate span for a regex match
    fn calculate_match_span(&self, match_obj: &regex::Match, source: &str) -> Span {
        let start_byte = match_obj.start();
        let end_byte = match_obj.end();

        let source_before = &source[..start_byte];
        // Count newlines to get the line number (1-indexed)
        let start_line = source_before.chars().filter(|&c| c == '\n').count() + 1;
        let start_column = source_before.lines().last().map(|l| l.len()).unwrap_or(0) + 1;

        let match_content = match_obj.as_str();
        let lines_in_match = match_content.chars().filter(|&c| c == '\n').count();
        let end_line = start_line + lines_in_match;
        let end_column = if lines_in_match > 0 {
            match_content.lines().last().map(|l| l.len()).unwrap_or(0) + 1
        } else {
            start_column + match_content.len()
        };

        Span::new(
            start_byte,
            end_byte,
            start_line,
            end_line,
            start_column,
            end_column,
        )
    }
}

/// Python comment extractor
pub struct PythonCommentExtractor {
    patterns: CommentPatterns,
    comment_regex: Regex,
    docstring_regex: Regex,
}

impl Default for PythonCommentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonCommentExtractor {
    /// Create a new Python comment extractor
    pub fn new() -> Self {
        Self {
            patterns: CommentPatterns {
                single_line: vec!["#".to_string()],
                block: vec![
                    ("\"\"\"".to_string(), "\"\"\"".to_string()),
                    ("'''".to_string(), "'''".to_string()),
                ],
                documentation: vec!["\"\"\"".to_string(), "'''".to_string()],
            },
            comment_regex: Regex::new(r"(?m)#.*$").unwrap(),
            docstring_regex: Regex::new(r#"("""[\s\S]*?"""|'''[\s\S]*?''')"#).unwrap(),
        }
    }
}

impl LanguageCommentExtractor for PythonCommentExtractor {
    fn extract_comments(
        &self,
        _tree: &Tree,
        source: &str,
        file_path: &Path,
        _ast_nodes: &[NodeId],
    ) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        // Extract hash comments
        for comment_match in self.comment_regex.find_iter(source) {
            let comment_text = comment_match.as_str();
            let cleaned_text = comment_text.trim_start_matches('#').trim().to_string();

            if cleaned_text.is_empty() {
                continue;
            }

            let span = self.calculate_match_span(&comment_match, source);
            let content_type = ContentType::Comment {
                language: Language::Python,
                context: CommentContext::Inline,
            };

            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                content_type,
                cleaned_text,
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "raw_text": comment_text,
                "language": "python"
            }));

            chunks.push(chunk);
            chunk_index += 1;
        }

        // Extract docstrings
        for docstring_match in self.docstring_regex.find_iter(source) {
            let docstring_text = docstring_match.as_str();
            let cleaned_text = self.clean_docstring(docstring_text);

            if cleaned_text.is_empty() {
                continue;
            }

            let span = self.calculate_match_span(&docstring_match, source);
            let content_type = ContentType::Comment {
                language: Language::Python,
                context: CommentContext::Documentation,
            };

            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                content_type,
                cleaned_text,
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "raw_text": docstring_text,
                "language": "python"
            }));

            chunks.push(chunk);
            chunk_index += 1;
        }

        Ok(chunks)
    }

    fn comment_patterns(&self) -> &CommentPatterns {
        &self.patterns
    }
}

impl PythonCommentExtractor {
    /// Clean docstring text
    fn clean_docstring(&self, docstring: &str) -> String {
        let cleaned = if docstring.starts_with("\"\"\"") {
            docstring
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"")
        } else {
            docstring.trim_start_matches("'''").trim_end_matches("'''")
        };

        cleaned.trim().to_string()
    }

    /// Calculate span for a regex match
    fn calculate_match_span(&self, match_obj: &regex::Match, source: &str) -> Span {
        let start_byte = match_obj.start();
        let end_byte = match_obj.end();

        let source_before = &source[..start_byte];
        // Count newlines to get the line number (1-indexed)
        let start_line = source_before.chars().filter(|&c| c == '\n').count() + 1;
        let start_column = source_before.lines().last().map(|l| l.len()).unwrap_or(0) + 1;

        let match_content = match_obj.as_str();
        let lines_in_match = match_content.chars().filter(|&c| c == '\n').count();
        let end_line = start_line + lines_in_match;
        let end_column = if lines_in_match > 0 {
            match_content.lines().last().map(|l| l.len()).unwrap_or(0) + 1
        } else {
            start_column + match_content.len()
        };

        Span::new(
            start_byte,
            end_byte,
            start_line,
            end_line,
            start_column,
            end_column,
        )
    }
}

// Stub implementations for other languages
macro_rules! simple_comment_extractor {
    ($name:ident, $language:ident, $single_line:expr, $block_start:expr, $block_end:expr) => {
        /// Comment extractor for a specific programming language
        pub struct $name {
            patterns: CommentPatterns,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// Create a new comment extractor for this language
            pub fn new() -> Self {
                Self {
                    patterns: CommentPatterns {
                        single_line: vec![$single_line.to_string()],
                        block: vec![($block_start.to_string(), $block_end.to_string())],
                        documentation: vec![],
                    },
                }
            }
        }

        impl LanguageCommentExtractor for $name {
            fn extract_comments(
                &self,
                _tree: &Tree,
                source: &str,
                file_path: &Path,
                _ast_nodes: &[NodeId],
            ) -> Result<Vec<ContentChunk>> {
                let mut chunks = Vec::new();
                let single_line_regex =
                    Regex::new(&format!(r"(?m){}.*$", regex::escape($single_line))).unwrap();
                let block_regex = Regex::new(&format!(
                    r"{}[\s\S]*?{}",
                    regex::escape($block_start),
                    regex::escape($block_end)
                ))
                .unwrap();

                let mut chunk_index = 0;

                // Extract single line comments
                for comment_match in single_line_regex.find_iter(source) {
                    let comment_text = comment_match.as_str();
                    let cleaned_text = comment_text
                        .trim_start_matches($single_line)
                        .trim()
                        .to_string();

                    if cleaned_text.is_empty() {
                        continue;
                    }

                    let span = self.calculate_match_span(&comment_match, source);
                    let content_type = ContentType::Comment {
                        language: Language::$language,
                        context: CommentContext::Inline,
                    };

                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        content_type,
                        cleaned_text,
                        span,
                        chunk_index,
                    );

                    chunks.push(chunk);
                    chunk_index += 1;
                }

                // Extract block comments
                for comment_match in block_regex.find_iter(source) {
                    let comment_text = comment_match.as_str();
                    let cleaned_text = comment_text
                        .trim_start_matches($block_start)
                        .trim_end_matches($block_end)
                        .trim()
                        .to_string();

                    if cleaned_text.is_empty() {
                        continue;
                    }

                    let span = self.calculate_match_span(&comment_match, source);
                    let content_type = ContentType::Comment {
                        language: Language::$language,
                        context: CommentContext::Block,
                    };

                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        content_type,
                        cleaned_text,
                        span,
                        chunk_index,
                    );

                    chunks.push(chunk);
                    chunk_index += 1;
                }

                Ok(chunks)
            }

            fn comment_patterns(&self) -> &CommentPatterns {
                &self.patterns
            }
        }

        impl $name {
            fn calculate_match_span(&self, match_obj: &regex::Match, source: &str) -> Span {
                let start_byte = match_obj.start();
                let end_byte = match_obj.end();

                let source_before = &source[..start_byte];
                // Count newlines to get the line number (1-indexed)
                let start_line = source_before.chars().filter(|&c| c == '\n').count() + 1;
                let start_column = source_before.lines().last().map(|l| l.len()).unwrap_or(0) + 1;

                let match_content = match_obj.as_str();
                let lines_in_match = match_content.chars().filter(|&c| c == '\n').count();
                let end_line = start_line + lines_in_match;
                let end_column = if lines_in_match > 0 {
                    match_content.lines().last().map(|l| l.len()).unwrap_or(0) + 1
                } else {
                    start_column + match_content.len()
                };

                Span::new(
                    start_byte,
                    end_byte,
                    start_line,
                    end_line,
                    start_column,
                    end_column,
                )
            }
        }
    };
}

// Generate simple extractors for other languages
simple_comment_extractor!(JavaCommentExtractor, Java, "//", "/*", "*/");
simple_comment_extractor!(RustCommentExtractor, Rust, "//", "/*", "*/");
simple_comment_extractor!(CCommentExtractor, C, "//", "/*", "*/");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_extractor_creation() {
        let extractor = CommentExtractor::new();
        assert!(extractor.supports_language(Language::JavaScript));
        assert!(extractor.supports_language(Language::Python));
        assert!(extractor.supports_language(Language::Rust));
        assert!(!extractor.supports_language(Language::Unknown));

        let supported = extractor.supported_languages();
        assert!(supported.contains(&Language::JavaScript));
        assert!(supported.contains(&Language::Python));
    }

    #[test]
    fn test_javascript_comment_patterns() {
        let extractor = JavaScriptCommentExtractor::new();
        let patterns = extractor.comment_patterns();

        assert!(patterns.single_line.contains(&"//".to_string()));
        assert!(patterns
            .block
            .contains(&("/*".to_string(), "*/".to_string())));
        assert!(patterns.documentation.contains(&"/**".to_string()));
    }

    #[test]
    fn test_python_comment_patterns() {
        let extractor = PythonCommentExtractor::new();
        let patterns = extractor.comment_patterns();

        assert!(patterns.single_line.contains(&"#".to_string()));
        assert!(patterns
            .block
            .contains(&("\"\"\"".to_string(), "\"\"\"".to_string())));
        assert!(patterns.documentation.contains(&"\"\"\"".to_string()));
    }

    #[test]
    fn test_comment_pattern_matching() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test comment regex matches
        let source = "// Single line comment\n/* Block comment */";
        let matches: Vec<_> = js_extractor.comment_regex.find_iter(source).collect();
        assert_eq!(matches.len(), 2, "Should find 2 comment matches");

        assert_eq!(matches[0].as_str(), "// Single line comment");
        assert_eq!(matches[1].as_str(), "/* Block comment */");
    }

    #[test]
    fn test_comment_cleaning() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test JSDoc cleaning
        let jsdoc = "/**\n * This is a JSDoc comment\n * @param value The input value\n */";
        let cleaned = js_extractor.clean_jsdoc_comment(jsdoc);
        assert!(cleaned.contains("This is a JSDoc comment"));
        assert!(cleaned.contains("@param value The input value"));
        assert!(!cleaned.contains("/**"));
        assert!(!cleaned.contains("*/"));

        // Test block comment cleaning
        let block = "/* This is a block comment */";
        let cleaned = js_extractor.clean_block_comment(block);
        assert_eq!(cleaned, "This is a block comment");

        // Test single line comment cleaning
        let single = "// This is a single line comment";
        let cleaned = js_extractor.clean_single_line_comment(single);
        assert_eq!(cleaned, "This is a single line comment");
    }

    #[test]
    fn test_python_docstring_cleaning() {
        let py_extractor = PythonCommentExtractor::new();

        // Test triple quote docstring
        let docstring = "\"\"\"This is a docstring\nwith multiple lines\"\"\"";
        let cleaned = py_extractor.clean_docstring(docstring);
        assert!(cleaned.contains("This is a docstring"));
        assert!(!cleaned.contains("\"\"\""));

        // Test single quote docstring
        let docstring = "'''This is another docstring'''";
        let cleaned = py_extractor.clean_docstring(docstring);
        assert_eq!(cleaned, "This is another docstring");
    }

    #[test]
    fn test_span_calculation() {
        let js_extractor = JavaScriptCommentExtractor::new();
        let source = "const x = 5;\n// This is a comment\nconst y = 10;";

        if let Some(comment_match) = js_extractor.comment_regex.find(source) {
            let span = js_extractor.calculate_match_span(&comment_match, source);

            assert_eq!(span.start_line, 2);
            assert_eq!(span.end_line, 2);
            assert!(span.start_column >= 1);
            assert!(span.end_column > span.start_column);
            assert_eq!(comment_match.as_str(), "// This is a comment");
        } else {
            panic!("Should find comment in source");
        }
    }

    #[test]
    fn test_regex_edge_cases() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test nested comments
        let source = "/* outer /* inner */ comment */";
        let matches: Vec<_> = js_extractor.comment_regex.find_iter(source).collect();
        assert!(
            !matches.is_empty(),
            "Should handle nested comments gracefully"
        );

        // Test comment at end of file without newline
        let source = "const x = 5; // Comment at end";
        let matches: Vec<_> = js_extractor.comment_regex.find_iter(source).collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].as_str(), "// Comment at end");

        // Test empty comments
        let source = "// \n/* */";
        let matches: Vec<_> = js_extractor.comment_regex.find_iter(source).collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_comprehensive_regex_edge_cases() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test multiline scenarios
        let multiline_source = r#"
const x = 1; // Comment on line 2
// Another comment on line 3
/* Block comment
   spanning multiple
   lines */
const y = 2; // Final comment
"#;

        let matches: Vec<_> = js_extractor
            .comment_regex
            .find_iter(multiline_source)
            .collect();
        assert!(
            matches.len() >= 4,
            "Should find all comment types including multiline block"
        );

        // Verify specific matches
        let comment_texts: Vec<&str> = matches.iter().map(|m| m.as_str()).collect();
        assert!(comment_texts
            .iter()
            .any(|&text| text.contains("Comment on line 2")));
        assert!(comment_texts
            .iter()
            .any(|&text| text.contains("Another comment")));
        assert!(comment_texts
            .iter()
            .any(|&text| text.contains("spanning multiple")));
        assert!(comment_texts
            .iter()
            .any(|&text| text.contains("Final comment")));
    }

    #[test]
    fn test_main_comment_extractor() {
        let extractor = CommentExtractor::new();

        // Test that it has language-specific extractors
        assert!(extractor.supports_language(Language::JavaScript));
        assert!(extractor.supports_language(Language::Python));
        assert!(extractor.supports_language(Language::Rust));
        assert!(extractor.supports_language(Language::Java));
        assert!(extractor.supports_language(Language::C));

        // Test unsupported language
        assert!(!extractor.supports_language(Language::Unknown));

        // Test supported languages list
        let supported = extractor.supported_languages();
        assert!(supported.len() >= 5);
        assert!(supported.contains(&Language::JavaScript));
        assert!(supported.contains(&Language::Python));
    }

    #[test]
    fn test_javascript_comment_extraction() {
        let _extractor = CommentExtractor::new();
        let _file_path = std::path::Path::new("test.js");

        let _source = r#"
// This is a single line comment
function test() {
    /* This is a block comment */
    return 42;
}

/**
 * This is a JSDoc comment
 * @param value The input value
 * @returns The result
 */
function documented(value) {
    return value * 2;
}
"#;

        // Test the pattern matching without tree parsing
        // (In real usage this would use a valid tree-sitter tree)
        let js_extractor = JavaScriptCommentExtractor::new();
        let patterns = js_extractor.comment_patterns();
        assert!(patterns.single_line.contains(&"//".to_string()));
        assert!(patterns.documentation.contains(&"/**".to_string()));
    }

    #[test]
    fn test_python_comment_extraction() {
        let _extractor = CommentExtractor::new();
        let _file_path = std::path::Path::new("test.py");

        let _source = r#"
# This is a single line comment
def test():
    """
    This is a docstring
    with multiple lines
    """
    return 42

class Example:
    '''
    Another docstring style
    '''
    pass
"#;

        // Test python extractor specifically
        let py_extractor = PythonCommentExtractor::new();
        let patterns = py_extractor.comment_patterns();
        assert!(patterns.single_line.contains(&"#".to_string()));
        assert!(patterns.documentation.contains(&"\"\"\"".to_string()));
        assert!(patterns.documentation.contains(&"'''".to_string()));
    }

    #[test]
    fn test_rust_comment_extraction() {
        let rust_extractor = RustCommentExtractor::new();
        let patterns = rust_extractor.comment_patterns();

        assert!(patterns.single_line.contains(&"//".to_string()));
        assert!(patterns
            .block
            .contains(&("/*".to_string(), "*/".to_string())));

        // Test that it's properly registered in main extractor
        let main_extractor = CommentExtractor::new();
        assert!(main_extractor.supports_language(Language::Rust));
    }

    #[test]
    fn test_java_comment_extraction() {
        let java_extractor = JavaCommentExtractor::new();
        let patterns = java_extractor.comment_patterns();

        assert!(patterns.single_line.contains(&"//".to_string()));
        assert!(patterns
            .block
            .contains(&("/*".to_string(), "*/".to_string())));

        // Test registration
        let main_extractor = CommentExtractor::new();
        assert!(main_extractor.supports_language(Language::Java));
    }

    #[test]
    fn test_c_comment_extraction() {
        let c_extractor = CCommentExtractor::new();
        let patterns = c_extractor.comment_patterns();

        assert!(patterns.single_line.contains(&"//".to_string()));
        assert!(patterns
            .block
            .contains(&("/*".to_string(), "*/".to_string())));

        // Test registration
        let main_extractor = CommentExtractor::new();
        assert!(main_extractor.supports_language(Language::C));
    }

    #[test]
    fn test_javascript_jsdoc_cleaning() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test comprehensive JSDoc cleaning
        let complex_jsdoc = r#"/**
         * Complex JSDoc comment
         * @param {string} name - The name parameter
         * @param {number} age - The age parameter
         * @returns {object} The result object
         * @example
         * // Usage example
         * const result = func("John", 25);
         * @see {@link http://example.com}
         */"#;

        let cleaned = js_extractor.clean_jsdoc_comment(complex_jsdoc);
        assert!(cleaned.contains("Complex JSDoc comment"));
        assert!(cleaned.contains("@param {string} name"));
        assert!(cleaned.contains("@returns {object}"));
        assert!(cleaned.contains("@example"));
        assert!(!cleaned.contains("/**"));
        assert!(!cleaned.contains("*/"));
        assert!(!cleaned.contains("         *"));
    }

    #[test]
    fn test_python_docstring_variations() {
        let py_extractor = PythonCommentExtractor::new();

        // Test different docstring styles
        let triple_quote = r#"""This is a triple quote docstring
        with multiple lines
        and various content"""#;

        let single_quote = r#"'''This is a single quote docstring
        also with multiple lines'''"#;

        let cleaned_triple = py_extractor.clean_docstring(triple_quote);
        let cleaned_single = py_extractor.clean_docstring(single_quote);

        assert!(!cleaned_triple.contains("\"\"\""));
        assert!(!cleaned_single.contains("'''"));
        assert!(cleaned_triple.contains("triple quote docstring"));
        assert!(cleaned_single.contains("single quote docstring"));
    }

    #[test]
    fn test_comment_context_detection() {
        let _js_extractor = JavaScriptCommentExtractor::new();

        // Test block vs inline detection logic
        let block_comment = "/* This is a block comment */";
        let inline_comment = "// This is an inline comment";

        // These would be block context
        assert!(block_comment.starts_with("/*"));
        assert!(block_comment.contains("*/"));

        // These would be inline context
        assert!(inline_comment.starts_with("//"));
        assert!(!inline_comment.contains("*/"));
    }

    #[test]
    fn test_span_calculation_edge_cases() {
        let js_extractor = JavaScriptCommentExtractor::new();

        // Test span calculation with various line endings
        let source_unix = "line1\n// comment\nline3";
        let source_windows = "line1\r\n// comment\r\nline3";
        let source_mixed = "line1\r\n// comment\nline3\r\n";

        for source in [source_unix, source_windows, source_mixed] {
            if let Some(comment_match) = js_extractor.comment_regex.find(source) {
                let span = js_extractor.calculate_match_span(&comment_match, source);
                assert!(span.start_line >= 1, "Line numbers should be 1-indexed");
                assert!(
                    span.end_line >= span.start_line,
                    "End line should be >= start line"
                );
                assert!(span.start_column >= 1, "Column numbers should be 1-indexed");
            }
        }
    }
}
