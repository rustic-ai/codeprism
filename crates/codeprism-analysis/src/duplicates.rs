//! Code duplicate detection module with AST-based analysis and semantic understanding

use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// AST node structure for comparison
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    pub node_type: String,
    pub children: Vec<AstNode>,
    pub normalized_text: String,
    pub structural_hash: u64,
}

/// Duplicate type classification
#[derive(Debug, Clone, PartialEq)]
pub enum DuplicateType {
    ExactCopy,
    StructuralSimilar,
    SemanticSimilar,
    PatternDuplicate,
}

/// Refactoring suggestion for duplicates
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub estimated_effort: String,
    pub potential_savings: String,
    pub implementation_steps: Vec<String>,
}

/// Enhanced duplicate result with detailed analysis
#[derive(Debug, Clone)]
pub struct DuplicateResult {
    pub similarity_score: f64,
    pub duplicate_type: DuplicateType,
    pub files: Vec<DuplicateFile>,
    pub common_patterns: Vec<String>,
    pub refactoring_suggestions: Vec<RefactoringSuggestion>,
    pub confidence_level: f64,
    pub estimated_savings: DuplicateSavings,
}

#[derive(Debug, Clone)]
pub struct DuplicateFile {
    pub path: String,
    pub lines: usize,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone)]
pub struct DuplicateSavings {
    pub lines_of_code: usize,
    pub maintenance_effort: String,
    pub bug_risk_reduction: String,
}

/// Advanced duplicate analyzer with AST and semantic analysis
pub struct DuplicateAnalyzer {
    /// Cache for parsed AST structures
    #[allow(dead_code)] // Will be used for AST caching optimization
    ast_cache: HashMap<String, AstNode>,
    /// Language-specific analyzers for different programming languages
    language_analyzers: HashMap<String, LanguageAnalyzer>,
    /// Semantic patterns for identifying functional similarities
    semantic_patterns: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct LanguageAnalyzer {
    keywords: Vec<String>,
    #[allow(dead_code)] // Will be used for operator-aware analysis
    operators: Vec<String>,
    control_structures: Vec<String>,
    comment_patterns: Vec<String>,
}

impl DuplicateAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            ast_cache: HashMap::new(),
            semantic_patterns: HashMap::new(),
            language_analyzers: HashMap::new(),
        };
        analyzer.initialize_language_analyzers();
        analyzer.initialize_semantic_patterns();
        analyzer
    }

    fn initialize_language_analyzers(&mut self) {
        // Python analyzer
        self.language_analyzers.insert(
            "py".to_string(),
            LanguageAnalyzer {
                keywords: vec![
                    "def".to_string(),
                    "class".to_string(),
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "try".to_string(),
                    "except".to_string(),
                    "with".to_string(),
                    "import".to_string(),
                    "from".to_string(),
                    "return".to_string(),
                    "yield".to_string(),
                ],
                operators: vec![
                    "+".to_string(),
                    "-".to_string(),
                    "*".to_string(),
                    "/".to_string(),
                    "==".to_string(),
                    "!=".to_string(),
                    ">=".to_string(),
                    "<=".to_string(),
                    "and".to_string(),
                    "or".to_string(),
                    "not".to_string(),
                    "in".to_string(),
                ],
                control_structures: vec![
                    "if".to_string(),
                    "elif".to_string(),
                    "else".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "try".to_string(),
                    "except".to_string(),
                    "finally".to_string(),
                ],
                comment_patterns: vec!["#".to_string(), "\"\"\"".to_string(), "'''".to_string()],
            },
        );

        // JavaScript/TypeScript analyzer
        self.language_analyzers.insert(
            "js".to_string(),
            LanguageAnalyzer {
                keywords: vec![
                    "function".to_string(),
                    "class".to_string(),
                    "if".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                    "const".to_string(),
                    "let".to_string(),
                    "var".to_string(),
                    "return".to_string(),
                    "async".to_string(),
                    "await".to_string(),
                    "import".to_string(),
                    "export".to_string(),
                ],
                operators: vec![
                    "+".to_string(),
                    "-".to_string(),
                    "*".to_string(),
                    "/".to_string(),
                    "==".to_string(),
                    "===".to_string(),
                    "!=".to_string(),
                    "!==".to_string(),
                    "&&".to_string(),
                    "||".to_string(),
                    "!".to_string(),
                ],
                control_structures: vec![
                    "if".to_string(),
                    "else".to_string(),
                    "switch".to_string(),
                    "case".to_string(),
                    "for".to_string(),
                    "while".to_string(),
                    "do".to_string(),
                    "try".to_string(),
                    "catch".to_string(),
                    "finally".to_string(),
                ],
                comment_patterns: vec!["//".to_string(), "/*".to_string(), "*/".to_string()],
            },
        );

        // Copy JS analyzer for TS files
        self.language_analyzers
            .insert("ts".to_string(), self.language_analyzers["js"].clone());
    }

    fn initialize_semantic_patterns(&mut self) {
        // Common programming patterns
        self.semantic_patterns.insert(
            "data_validation".to_string(),
            vec![
                "validate".to_string(),
                "check".to_string(),
                "verify".to_string(),
                "assert".to_string(),
                "ensure".to_string(),
                "require".to_string(),
            ],
        );

        self.semantic_patterns.insert(
            "error_handling".to_string(),
            vec![
                "try".to_string(),
                "catch".to_string(),
                "except".to_string(),
                "error".to_string(),
                "exception".to_string(),
                "throw".to_string(),
                "raise".to_string(),
                "handle".to_string(),
            ],
        );

        self.semantic_patterns.insert(
            "database_operations".to_string(),
            vec![
                "select".to_string(),
                "insert".to_string(),
                "update".to_string(),
                "delete".to_string(),
                "query".to_string(),
                "execute".to_string(),
                "commit".to_string(),
                "rollback".to_string(),
            ],
        );

        self.semantic_patterns.insert(
            "api_patterns".to_string(),
            vec![
                "get".to_string(),
                "post".to_string(),
                "put".to_string(),
                "delete".to_string(),
                "patch".to_string(),
                "request".to_string(),
                "response".to_string(),
                "endpoint".to_string(),
                "route".to_string(),
            ],
        );
    }

    /// Find code duplicates with advanced AST and semantic analysis
    pub fn find_code_duplicates_advanced(
        &mut self,
        repo_path: &Path,
        similarity_threshold: f64,
        min_lines: usize,
        exclude_patterns: &[String],
    ) -> Result<Vec<DuplicateResult>> {
        let mut duplicates = Vec::new();
        let mut file_contents = HashMap::new();
        let mut file_asts = HashMap::new();

        // Collect all source files and parse them
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if [
                    "js", "ts", "py", "java", "rs", "c", "cpp", "go", "rb", "php",
                ]
                .contains(&ext)
                {
                    let path_str = path.to_string_lossy();
                    if exclude_patterns
                        .iter()
                        .any(|pattern| path_str.contains(pattern))
                    {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        let lines = content.lines().count();
                        if lines >= min_lines {
                            // Parse AST for the file
                            let ast = self.parse_file_ast(&content, ext)?;
                            file_asts.insert(path.to_path_buf(), ast);
                            file_contents.insert(path.to_path_buf(), content);
                        }
                    }
                }
            }
        }

        // Advanced duplicate detection using multiple techniques
        duplicates.extend(self.find_exact_duplicates(&file_contents, min_lines)?);
        duplicates.extend(self.find_structural_duplicates(
            &file_asts,
            similarity_threshold,
            min_lines,
        )?);
        duplicates.extend(self.find_semantic_duplicates(
            &file_contents,
            similarity_threshold,
            min_lines,
        )?);
        duplicates.extend(self.find_pattern_duplicates(&file_contents, similarity_threshold)?);

        // Remove overlapping duplicates and enhance with refactoring suggestions
        let deduplicated = self.deduplicate_results(duplicates);
        let enhanced_results = deduplicated
            .into_iter()
            .map(|dup| self.enhance_with_refactoring_suggestions(dup))
            .collect();

        Ok(enhanced_results)
    }

    /// Parse file content into AST representation
    fn parse_file_ast(&mut self, content: &str, language: &str) -> Result<AstNode> {
        // Simple AST parsing - in production this would use tree-sitter or similar
        let lines: Vec<&str> = content.lines().collect();
        let mut root_children = Vec::new();

        for line in lines.iter() {
            let trimmed = line.trim();
            if trimmed.is_empty() || self.is_comment_line(trimmed, language) {
                continue;
            }

            let node_type = self.classify_line_type(trimmed, language);
            let normalized = self.normalize_line_for_ast(trimmed, language);
            let hash = self.calculate_structural_hash(&normalized);

            root_children.push(AstNode {
                node_type: node_type.clone(),
                children: Vec::new(),
                normalized_text: normalized,
                structural_hash: hash,
            });
        }

        Ok(AstNode {
            node_type: "file".to_string(),
            children: root_children,
            normalized_text: "".to_string(),
            structural_hash: self.calculate_structural_hash(content),
        })
    }

    fn is_comment_line(&self, line: &str, language: &str) -> bool {
        if let Some(analyzer) = self.language_analyzers.get(language) {
            analyzer
                .comment_patterns
                .iter()
                .any(|pattern| line.starts_with(pattern))
        } else {
            line.starts_with("//") || line.starts_with("#") || line.starts_with("/*")
        }
    }

    fn classify_line_type(&self, line: &str, language: &str) -> String {
        if let Some(analyzer) = self.language_analyzers.get(language) {
            for keyword in &analyzer.keywords {
                if line.contains(keyword) {
                    return keyword.clone();
                }
            }
            for control in &analyzer.control_structures {
                if line.contains(control) {
                    return "control_structure".to_string();
                }
            }
        }
        "statement".to_string()
    }

    fn normalize_line_for_ast(&self, line: &str, language: &str) -> String {
        let mut normalized = line.to_string();

        // Remove language-specific syntax while preserving structure
        if let Some(_analyzer) = self.language_analyzers.get(language) {
            // Replace identifiers with normalized tokens
            normalized = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b")
                .unwrap()
                .replace_all(&normalized, "IDENTIFIER")
                .to_string();

            // Replace numbers with normalized tokens
            normalized = regex::Regex::new(r"\b\d+\b")
                .unwrap()
                .replace_all(&normalized, "NUMBER")
                .to_string();

            // Replace strings with normalized tokens
            normalized = regex::Regex::new(r#""[^"]*""#)
                .unwrap()
                .replace_all(&normalized, "STRING")
                .to_string();
        }

        normalized
    }

    fn calculate_structural_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Find exact duplicates (copy-paste)
    fn find_exact_duplicates(
        &self,
        file_contents: &HashMap<std::path::PathBuf, String>,
        min_lines: usize,
    ) -> Result<Vec<DuplicateResult>> {
        let mut duplicates = Vec::new();
        let mut analyzed_pairs = HashSet::new();

        for (file1, content1) in file_contents {
            for (file2, content2) in file_contents {
                if file1 >= file2 || analyzed_pairs.contains(&(file1.clone(), file2.clone())) {
                    continue;
                }
                analyzed_pairs.insert((file1.clone(), file2.clone()));

                let similarity = self.calculate_exact_similarity(content1, content2);
                if similarity >= 0.95 {
                    // Very high threshold for exact duplicates
                    let lines1 = content1.lines().count();
                    let lines2 = content2.lines().count();

                    if lines1 >= min_lines && lines2 >= min_lines {
                        duplicates.push(DuplicateResult {
                            similarity_score: similarity,
                            duplicate_type: DuplicateType::ExactCopy,
                            files: vec![
                                DuplicateFile {
                                    path: file1.display().to_string(),
                                    lines: lines1,
                                    start_line: None,
                                    end_line: None,
                                    complexity_score: self.calculate_complexity_score(content1),
                                },
                                DuplicateFile {
                                    path: file2.display().to_string(),
                                    lines: lines2,
                                    start_line: None,
                                    end_line: None,
                                    complexity_score: self.calculate_complexity_score(content2),
                                },
                            ],
                            common_patterns: self.identify_common_patterns(content1, content2),
                            refactoring_suggestions: Vec::new(), // Will be filled later
                            confidence_level: 0.95,
                            estimated_savings: DuplicateSavings {
                                lines_of_code: lines1.min(lines2),
                                maintenance_effort: "High".to_string(),
                                bug_risk_reduction: "Significant".to_string(),
                            },
                        });
                    }
                }
            }
        }

        Ok(duplicates)
    }

    /// Find structural duplicates using AST comparison
    fn find_structural_duplicates(
        &self,
        file_asts: &HashMap<std::path::PathBuf, AstNode>,
        similarity_threshold: f64,
        min_lines: usize,
    ) -> Result<Vec<DuplicateResult>> {
        let mut duplicates = Vec::new();
        let mut analyzed_pairs = HashSet::new();

        for (file1, ast1) in file_asts {
            for (file2, ast2) in file_asts {
                if file1 >= file2 || analyzed_pairs.contains(&(file1.clone(), file2.clone())) {
                    continue;
                }
                analyzed_pairs.insert((file1.clone(), file2.clone()));

                let similarity = self.calculate_structural_similarity_ast(ast1, ast2);
                if similarity >= similarity_threshold
                    && ast1.children.len() >= min_lines
                    && ast2.children.len() >= min_lines
                {
                    duplicates.push(DuplicateResult {
                        similarity_score: similarity,
                        duplicate_type: DuplicateType::StructuralSimilar,
                        files: vec![
                            DuplicateFile {
                                path: file1.display().to_string(),
                                lines: ast1.children.len(),
                                start_line: None,
                                end_line: None,
                                complexity_score: self.calculate_ast_complexity(ast1),
                            },
                            DuplicateFile {
                                path: file2.display().to_string(),
                                lines: ast2.children.len(),
                                start_line: None,
                                end_line: None,
                                complexity_score: self.calculate_ast_complexity(ast2),
                            },
                        ],
                        common_patterns: self.identify_structural_patterns(ast1, ast2),
                        refactoring_suggestions: Vec::new(),
                        confidence_level: similarity * 0.9, // Slightly lower confidence for structural
                        estimated_savings: DuplicateSavings {
                            lines_of_code: ast1.children.len().min(ast2.children.len()),
                            maintenance_effort: "Medium".to_string(),
                            bug_risk_reduction: "Moderate".to_string(),
                        },
                    });
                }
            }
        }

        Ok(duplicates)
    }

    /// Find semantic duplicates based on functionality
    fn find_semantic_duplicates(
        &self,
        file_contents: &HashMap<std::path::PathBuf, String>,
        similarity_threshold: f64,
        min_lines: usize,
    ) -> Result<Vec<DuplicateResult>> {
        let mut duplicates = Vec::new();
        let mut analyzed_pairs = HashSet::new();

        for (file1, content1) in file_contents {
            for (file2, content2) in file_contents {
                if file1 >= file2 || analyzed_pairs.contains(&(file1.clone(), file2.clone())) {
                    continue;
                }
                analyzed_pairs.insert((file1.clone(), file2.clone()));

                let similarity = self.calculate_semantic_similarity(content1, content2);
                if similarity >= similarity_threshold {
                    let lines1 = content1.lines().count();
                    let lines2 = content2.lines().count();

                    if lines1 >= min_lines && lines2 >= min_lines {
                        duplicates.push(DuplicateResult {
                            similarity_score: similarity,
                            duplicate_type: DuplicateType::SemanticSimilar,
                            files: vec![
                                DuplicateFile {
                                    path: file1.display().to_string(),
                                    lines: lines1,
                                    start_line: None,
                                    end_line: None,
                                    complexity_score: self.calculate_complexity_score(content1),
                                },
                                DuplicateFile {
                                    path: file2.display().to_string(),
                                    lines: lines2,
                                    start_line: None,
                                    end_line: None,
                                    complexity_score: self.calculate_complexity_score(content2),
                                },
                            ],
                            common_patterns: self.identify_semantic_patterns(content1, content2),
                            refactoring_suggestions: Vec::new(),
                            confidence_level: similarity * 0.8, // Lower confidence for semantic
                            estimated_savings: DuplicateSavings {
                                lines_of_code: lines1.min(lines2) / 2, // Conservative estimate
                                maintenance_effort: "Medium".to_string(),
                                bug_risk_reduction: "Low".to_string(),
                            },
                        });
                    }
                }
            }
        }

        Ok(duplicates)
    }

    /// Find pattern-based duplicates (design patterns, common code structures)
    fn find_pattern_duplicates(
        &self,
        _file_contents: &HashMap<std::path::PathBuf, String>,
        _similarity_threshold: f64,
    ) -> Result<Vec<DuplicateResult>> {
        let duplicates = Vec::new();
        // Pattern-based detection would analyze for common design patterns
        // This is a simplified implementation
        Ok(duplicates)
    }

    fn calculate_exact_similarity(&self, content1: &str, content2: &str) -> f64 {
        self.calculate_content_similarity(content1, content2)
    }

    fn calculate_structural_similarity_ast(&self, ast1: &AstNode, ast2: &AstNode) -> f64 {
        if ast1.children.is_empty() && ast2.children.is_empty() {
            return if ast1.normalized_text == ast2.normalized_text {
                1.0
            } else {
                0.0
            };
        }

        let mut matches = 0;
        let total = ast1.children.len().max(ast2.children.len());

        for child1 in &ast1.children {
            for child2 in &ast2.children {
                if child1.node_type == child2.node_type
                    && child1.structural_hash == child2.structural_hash
                {
                    matches += 1;
                    break;
                }
            }
        }

        if total == 0 {
            0.0
        } else {
            matches as f64 / total as f64
        }
    }

    fn calculate_semantic_similarity(&self, content1: &str, content2: &str) -> f64 {
        let mut similarity_score = 0.0;
        let mut _pattern_matches = 0;
        let mut total_patterns = 0;

        for patterns in self.semantic_patterns.values() {
            total_patterns += patterns.len();

            let count1 = patterns
                .iter()
                .map(|p| content1.matches(p).count())
                .sum::<usize>();
            let count2 = patterns
                .iter()
                .map(|p| content2.matches(p).count())
                .sum::<usize>();

            if count1 > 0 && count2 > 0 {
                _pattern_matches += patterns.len();
                similarity_score += (count1.min(count2) as f64) / (count1.max(count2) as f64);
            }
        }

        if total_patterns == 0 {
            0.0
        } else {
            similarity_score / total_patterns as f64
        }
    }

    /// Legacy method for backward compatibility
    pub fn find_code_duplicates(
        &mut self,
        repo_path: &Path,
        similarity_threshold: f64,
        min_lines: usize,
        exclude_patterns: &[String],
    ) -> Result<Vec<Value>> {
        let advanced_results = self.find_code_duplicates_advanced(
            repo_path,
            similarity_threshold,
            min_lines,
            exclude_patterns,
        )?;

        // Convert advanced results to legacy format
        let legacy_results = advanced_results
            .into_iter()
            .map(|result| {
                serde_json::json!({
                    "similarity": result.similarity_score,
                    "files": result.files.iter().map(|f| serde_json::json!({
                        "path": f.path,
                        "lines": f.lines
                    })).collect::<Vec<_>>(),
                    "lines": result.files.iter().map(|f| f.lines).min().unwrap_or(0),
                    "type": match result.duplicate_type {
                        DuplicateType::ExactCopy => "exact_copy",
                        DuplicateType::StructuralSimilar => "structural_similar",
                        DuplicateType::SemanticSimilar => "semantic_similar",
                        DuplicateType::PatternDuplicate => "pattern_duplicate",
                    },
                    "confidence_level": result.confidence_level,
                    "refactoring_suggestions": result.refactoring_suggestions.len()
                })
            })
            .collect();

        Ok(legacy_results)
    }

    fn calculate_complexity_score(&self, content: &str) -> f64 {
        let lines = content.lines().count();
        let non_empty_lines = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        // Simple complexity metric based on line ratios and control structures
        let control_structures = content.matches("if").count()
            + content.matches("for").count()
            + content.matches("while").count()
            + content.matches("try").count()
            + content.matches("catch").count()
            + content.matches("switch").count();
        let functions = content.matches("def ").count()
            + content.matches("function ").count()
            + content.matches("class ").count();

        let base_complexity = non_empty_lines as f64 / lines.max(1) as f64;
        let control_complexity = control_structures as f64 / non_empty_lines.max(1) as f64;
        let function_complexity = functions as f64 / non_empty_lines.max(1) as f64;

        (base_complexity + control_complexity + function_complexity) * 100.0
    }

    fn calculate_ast_complexity(&self, ast: &AstNode) -> f64 {
        let mut complexity = 0.0;
        complexity += ast.children.len() as f64;

        for child in &ast.children {
            if child.node_type.contains("if")
                || child.node_type.contains("for")
                || child.node_type.contains("while")
            {
                complexity += 2.0; // Control structures add more complexity
            } else {
                complexity += 1.0;
            }
        }

        complexity
    }

    fn identify_common_patterns(&self, content1: &str, content2: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for common semantic patterns
        for (pattern_type, pattern_keywords) in &self.semantic_patterns {
            let matches1 = pattern_keywords
                .iter()
                .filter(|&keyword| content1.contains(keyword))
                .count();
            let matches2 = pattern_keywords
                .iter()
                .filter(|&keyword| content2.contains(keyword))
                .count();

            if matches1 > 0 && matches2 > 0 {
                patterns.push(pattern_type.clone());
            }
        }

        // Check for structural patterns
        let common_keywords = ["function", "class", "if", "for", "while", "try", "catch"];
        for keyword in &common_keywords {
            if content1.contains(keyword) && content2.contains(keyword) {
                patterns.push(keyword.to_string());
            }
        }

        patterns
    }

    fn identify_structural_patterns(&self, ast1: &AstNode, ast2: &AstNode) -> Vec<String> {
        let mut patterns = Vec::new();

        // Find common node types
        let types1: HashSet<_> = ast1.children.iter().map(|child| &child.node_type).collect();
        let types2: HashSet<_> = ast2.children.iter().map(|child| &child.node_type).collect();

        for common_type in types1.intersection(&types2) {
            patterns.push(format!("structural_{common_type}"));
        }

        patterns
    }

    fn identify_semantic_patterns(&self, content1: &str, content2: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        for (pattern_type, keywords) in &self.semantic_patterns {
            let score1 = keywords
                .iter()
                .map(|k| content1.matches(k).count())
                .sum::<usize>();
            let score2 = keywords
                .iter()
                .map(|k| content2.matches(k).count())
                .sum::<usize>();

            if score1 > 0 && score2 > 0 {
                let similarity = (score1.min(score2) as f64) / (score1.max(score2) as f64);
                if similarity > 0.5 {
                    patterns.push(format!("semantic_{pattern_type}"));
                }
            }
        }

        patterns
    }

    fn deduplicate_results(&self, mut duplicates: Vec<DuplicateResult>) -> Vec<DuplicateResult> {
        // Remove overlapping duplicates - keep the one with highest confidence
        duplicates.sort_by(|a, b| b.confidence_level.partial_cmp(&a.confidence_level).unwrap());

        let mut result = Vec::new();
        let mut seen_files = HashSet::new();

        for duplicate in duplicates {
            let file_paths: Vec<String> = duplicate.files.iter().map(|f| f.path.clone()).collect();

            if !file_paths.iter().any(|path| seen_files.contains(path)) {
                for path in &file_paths {
                    seen_files.insert(path.clone());
                }
                result.push(duplicate);
            }
        }

        result
    }

    fn enhance_with_refactoring_suggestions(
        &self,
        mut duplicate: DuplicateResult,
    ) -> DuplicateResult {
        let mut suggestions = Vec::new();

        match duplicate.duplicate_type {
            DuplicateType::ExactCopy => {
                suggestions.push(RefactoringSuggestion {
                    suggestion_type: "Extract Common Function".to_string(),
                    description: "Extract the duplicated code into a common function or module"
                        .to_string(),
                    estimated_effort: "Low".to_string(),
                    potential_savings: format!(
                        "{} lines of code",
                        duplicate.estimated_savings.lines_of_code
                    ),
                    implementation_steps: vec![
                        "1. Create a new function with the common code".to_string(),
                        "2. Replace duplicate instances with function calls".to_string(),
                        "3. Test to ensure functionality is preserved".to_string(),
                    ],
                });
            }
            DuplicateType::StructuralSimilar => {
                suggestions.push(RefactoringSuggestion {
                    suggestion_type: "Create Abstract Base Class".to_string(),
                    description: "Create a common base class or interface for similar structures"
                        .to_string(),
                    estimated_effort: "Medium".to_string(),
                    potential_savings: format!(
                        "{} lines of code reduction",
                        duplicate.estimated_savings.lines_of_code / 2
                    ),
                    implementation_steps: vec![
                        "1. Identify common structural elements".to_string(),
                        "2. Create base class or interface".to_string(),
                        "3. Refactor duplicate classes to inherit/implement".to_string(),
                        "4. Test inheritance hierarchy".to_string(),
                    ],
                });
            }
            DuplicateType::SemanticSimilar => {
                suggestions.push(RefactoringSuggestion {
                    suggestion_type: "Strategy Pattern".to_string(),
                    description: "Use strategy pattern to handle similar functionality".to_string(),
                    estimated_effort: "High".to_string(),
                    potential_savings: "Improved maintainability and reduced complexity"
                        .to_string(),
                    implementation_steps: vec![
                        "1. Define common interface for similar behaviors".to_string(),
                        "2. Implement concrete strategies".to_string(),
                        "3. Refactor to use strategy pattern".to_string(),
                        "4. Add configuration for strategy selection".to_string(),
                    ],
                });
            }
            DuplicateType::PatternDuplicate => {
                suggestions.push(RefactoringSuggestion {
                    suggestion_type: "Template Method".to_string(),
                    description: "Use template method pattern for common algorithmic structure"
                        .to_string(),
                    estimated_effort: "Medium".to_string(),
                    potential_savings: "Reduced code duplication and improved consistency"
                        .to_string(),
                    implementation_steps: vec![
                        "1. Identify common algorithm structure".to_string(),
                        "2. Create template method in base class".to_string(),
                        "3. Implement varying steps in subclasses".to_string(),
                    ],
                });
            }
        }

        // Add complexity-based suggestions
        let avg_complexity = duplicate
            .files
            .iter()
            .map(|f| f.complexity_score)
            .sum::<f64>()
            / duplicate.files.len() as f64;

        if avg_complexity > 50.0 {
            suggestions.push(RefactoringSuggestion {
                suggestion_type: "Simplify Complex Code".to_string(),
                description:
                    "Break down complex duplicated code into smaller, more manageable pieces"
                        .to_string(),
                estimated_effort: "High".to_string(),
                potential_savings: "Improved readability and maintainability".to_string(),
                implementation_steps: vec![
                    "1. Identify complex sections within duplicates".to_string(),
                    "2. Extract helper functions".to_string(),
                    "3. Simplify conditional logic".to_string(),
                    "4. Add comprehensive tests".to_string(),
                ],
            });
        }

        duplicate.refactoring_suggestions = suggestions;
        duplicate
    }

    /// Calculate content similarity between two text blocks using Jaccard coefficient
    pub fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f64 {
        let lines1: Vec<String> = content1
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with("//") && !s.starts_with("#"))
            .collect();

        let lines2: Vec<String> = content2
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with("//") && !s.starts_with("#"))
            .collect();

        if lines1.is_empty() || lines2.is_empty() {
            return 0.0;
        }

        // Jaccard coefficient: intersection / union
        let set1: HashSet<String> = lines1.into_iter().collect();
        let set2: HashSet<String> = lines2.into_iter().collect();

        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Find duplicate code blocks within files
    pub fn find_duplicate_blocks(
        &self,
        content: &str,
        min_lines: usize,
        similarity_threshold: f64,
    ) -> Result<Vec<Value>> {
        let mut duplicates = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() < min_lines * 2 {
            return Ok(duplicates);
        }

        // Compare all possible blocks of minimum size
        for i in 0..=lines.len().saturating_sub(min_lines) {
            for j in (i + min_lines)..=lines.len().saturating_sub(min_lines) {
                let block1 = &lines[i..i + min_lines];
                let block2 = &lines[j..j + min_lines];

                let block1_text = block1.join("\n");
                let block2_text = block2.join("\n");

                let similarity = self.calculate_content_similarity(&block1_text, &block2_text);

                if similarity >= similarity_threshold {
                    duplicates.push(serde_json::json!({
                        "similarity": similarity,
                        "blocks": [
                            {
                                "start_line": i + 1,
                                "end_line": i + min_lines,
                                "content": block1_text
                            },
                            {
                                "start_line": j + 1,
                                "end_line": j + min_lines,
                                "content": block2_text
                            }
                        ],
                        "type": "block_similarity"
                    }));
                }
            }
        }

        Ok(duplicates)
    }

    /// Calculate structural similarity (ignoring variable names)
    pub fn calculate_structural_similarity(&self, content1: &str, content2: &str) -> f64 {
        // Normalize content by removing variable names and keeping structure
        let normalized1 = self.normalize_for_structure(content1);
        let normalized2 = self.normalize_for_structure(content2);

        self.calculate_content_similarity(&normalized1, &normalized2)
    }

    /// Normalize content for structural comparison
    fn normalize_for_structure(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                // Replace identifiers with normalized tokens while keeping structure
                let mut normalized = trimmed.to_string();

                // Replace common patterns
                normalized = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b")
                    .unwrap()
                    .replace_all(&normalized, "IDENTIFIER")
                    .to_string();

                normalized = regex::Regex::new(r"\b\d+\b")
                    .unwrap()
                    .replace_all(&normalized, "NUMBER")
                    .to_string();

                normalized = regex::Regex::new(r#""[^"]*""#)
                    .unwrap()
                    .replace_all(&normalized, "STRING")
                    .to_string();

                normalized
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for DuplicateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_content_similarity() {
        let analyzer = DuplicateAnalyzer::new();

        let content1 = "line1\nline2\nline3";
        let content2 = "line1\nline2\nline4";

        let similarity = analyzer.calculate_content_similarity(content1, content2);
        assert!(similarity > 0.0 && similarity < 1.0);

        let identical = analyzer.calculate_content_similarity(content1, content1);
        assert_eq!(identical, 1.0);
    }

    #[test]
    fn test_structural_similarity() {
        let analyzer = DuplicateAnalyzer::new();

        let content1 = "def func1(x, y):\n    return x + y";
        let content2 = "def func2(a, b):\n    return a + b";

        let similarity = analyzer.calculate_structural_similarity(content1, content2);
        assert!(similarity > 0.8); // Should be very similar structurally
    }

    #[test]
    fn test_find_duplicate_blocks() {
        let analyzer = DuplicateAnalyzer::new();

        let content = "line1\nline2\nline3\nline4\nline1\nline2\nline3\nline5";
        let duplicates = analyzer.find_duplicate_blocks(content, 3, 0.8).unwrap();

        assert!(!duplicates.is_empty(), "Should find duplicate code");
    }

    #[test]
    fn test_find_code_duplicates() {
        let mut analyzer = DuplicateAnalyzer::new();
        let temp_dir = tempdir().unwrap();

        // Create test files
        let file1_path = temp_dir.path().join("file1.py");
        let file2_path = temp_dir.path().join("file2.py");

        fs::write(&file1_path, "def test():\n    return 1").unwrap();
        fs::write(&file2_path, "def test():\n    return 1").unwrap();

        let duplicates = analyzer
            .find_code_duplicates(temp_dir.path(), 0.8, 1, &[])
            .unwrap();
        assert!(!duplicates.is_empty(), "Should find duplicate code");
    }
}
