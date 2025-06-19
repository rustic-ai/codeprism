//! Code duplicate detection module

use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// Duplicate analyzer for code analysis
pub struct DuplicateAnalyzer;

impl DuplicateAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Find code duplicates in the repository
    pub fn find_code_duplicates(
        &self,
        repo_path: &Path,
        similarity_threshold: f64,
        min_lines: usize,
        exclude_patterns: &[String],
    ) -> Result<Vec<Value>> {
        let mut duplicates = Vec::new();

        // Get all source files
        let mut file_contents = HashMap::new();

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
                    // Skip if matches exclude patterns
                    let path_str = path.to_string_lossy();
                    if exclude_patterns
                        .iter()
                        .any(|pattern| path_str.contains(pattern))
                    {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        file_contents.insert(path.to_path_buf(), content);
                    }
                }
            }
        }

        // Simple duplicate detection based on similar line patterns
        let mut analyzed_pairs = HashSet::new();

        for (file1, content1) in &file_contents {
            for (file2, content2) in &file_contents {
                if file1 >= file2 || analyzed_pairs.contains(&(file1.clone(), file2.clone())) {
                    continue;
                }
                analyzed_pairs.insert((file1.clone(), file2.clone()));

                let similarity = self.calculate_content_similarity(content1, content2);
                if similarity >= similarity_threshold {
                    let lines1 = content1.lines().count();
                    let lines2 = content2.lines().count();

                    if lines1 >= min_lines && lines2 >= min_lines {
                        duplicates.push(serde_json::json!({
                            "similarity": similarity,
                            "files": [
                                {
                                    "path": file1.display().to_string(),
                                    "lines": lines1
                                },
                                {
                                    "path": file2.display().to_string(),
                                    "lines": lines2
                                }
                            ],
                            "lines": lines1.min(lines2),
                            "type": "file_similarity"
                        }));
                    }
                }
            }
        }

        Ok(duplicates)
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
                // Replace identifiers with placeholders while keeping structure
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

        assert!(!duplicates.is_empty());
    }

    #[test]
    fn test_find_code_duplicates() {
        let analyzer = DuplicateAnalyzer::new();
        let temp_dir = tempdir().unwrap();

        // Create test files
        let file1_path = temp_dir.path().join("file1.py");
        let file2_path = temp_dir.path().join("file2.py");

        fs::write(&file1_path, "def test():\n    return 1").unwrap();
        fs::write(&file2_path, "def test():\n    return 1").unwrap();

        let duplicates = analyzer
            .find_code_duplicates(temp_dir.path(), 0.8, 1, &[])
            .unwrap();
        assert!(!duplicates.is_empty());
    }
}
