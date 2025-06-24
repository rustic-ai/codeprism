//! Code complexity analysis module

use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

/// Complexity metrics for code analysis
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_effort: f64,
    pub maintainability_index: f64,
    pub lines_of_code: usize,
}

/// Complexity analyzer for code analysis
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze complexity for a given file
    pub fn analyze_file_complexity(
        &self,
        file_path: &Path,
        metrics: &[String],
        threshold_warnings: bool,
    ) -> Result<Value> {
        let content = std::fs::read_to_string(file_path)?;
        let lines_count = content.lines().count();

        let complexity_metrics = self.calculate_all_metrics(&content, lines_count);

        let mut result = serde_json::json!({
            "file": file_path.display().to_string(),
            "lines_of_code": lines_count,
            "metrics": {}
        });

        if metrics.contains(&"cyclomatic".to_string()) || metrics.contains(&"all".to_string()) {
            result["metrics"]["cyclomatic_complexity"] = complexity_metrics.cyclomatic.into();
            if threshold_warnings && complexity_metrics.cyclomatic > 10 {
                result["warnings"] = serde_json::json!([{
                    "type": "high_cyclomatic_complexity",
                    "message": format!("Cyclomatic complexity ({}) exceeds recommended threshold (10)", complexity_metrics.cyclomatic)
                }]);
            }
        }

        if metrics.contains(&"cognitive".to_string()) || metrics.contains(&"all".to_string()) {
            result["metrics"]["cognitive_complexity"] = complexity_metrics.cognitive.into();
            if threshold_warnings && complexity_metrics.cognitive > 15 {
                let warnings = result["warnings"].as_array().cloned().unwrap_or_default();
                let mut new_warnings = warnings;
                new_warnings.push(serde_json::json!({
                    "type": "high_cognitive_complexity",
                    "message": format!("Cognitive complexity ({}) exceeds recommended threshold (15)", complexity_metrics.cognitive)
                }));
                result["warnings"] = new_warnings.into();
            }
        }

        if metrics.contains(&"halstead".to_string()) || metrics.contains(&"all".to_string()) {
            result["metrics"]["halstead"] = serde_json::json!({
                "volume": complexity_metrics.halstead_volume,
                "difficulty": complexity_metrics.halstead_difficulty,
                "effort": complexity_metrics.halstead_effort
            });
        }

        if metrics.contains(&"maintainability".to_string()) || metrics.contains(&"all".to_string())
        {
            result["metrics"]["maintainability_index"] =
                complexity_metrics.maintainability_index.into();
            if threshold_warnings && complexity_metrics.maintainability_index < 50.0 {
                let warnings = result["warnings"].as_array().cloned().unwrap_or_default();
                let mut new_warnings = warnings;
                new_warnings.push(serde_json::json!({
                    "type": "low_maintainability",
                    "message": format!("Maintainability index ({:.1}) is below recommended threshold (50.0)", complexity_metrics.maintainability_index)
                }));
                result["warnings"] = new_warnings.into();
            }
        }

        Ok(result)
    }

    /// Calculate all complexity metrics for content
    pub fn calculate_all_metrics(&self, content: &str, lines_count: usize) -> ComplexityMetrics {
        let cyclomatic = self.calculate_cyclomatic_complexity(content);
        let cognitive = self.calculate_cognitive_complexity(content);
        let (halstead_volume, halstead_difficulty, halstead_effort) =
            self.calculate_halstead_metrics(content);
        let maintainability_index = self.calculate_maintainability_index(content, lines_count);

        ComplexityMetrics {
            cyclomatic,
            cognitive,
            halstead_volume,
            halstead_difficulty,
            halstead_effort,
            maintainability_index,
            lines_of_code: lines_count,
        }
    }

    /// Calculate cyclomatic complexity (simplified)
    pub fn calculate_cyclomatic_complexity(&self, content: &str) -> usize {
        let mut complexity = 1; // Base complexity

        // Count decision points (simplified heuristic)
        let decision_keywords = [
            "if", "else if", "elif", "while", "for", "foreach", "switch", "case", "catch",
            "except", "?", "&&", "||", "and", "or",
        ];

        for keyword in &decision_keywords {
            complexity += content.matches(keyword).count();
        }

        complexity
    }

    /// Calculate cognitive complexity (simplified)
    pub fn calculate_cognitive_complexity(&self, content: &str) -> usize {
        let mut complexity = 0;
        let mut nesting_level: usize = 0;

        let lines = content.lines();
        for line in lines {
            let trimmed = line.trim();

            // Increment nesting for certain constructs
            if trimmed.contains('{')
                || trimmed.starts_with("if ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("try ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("function ")
            {
                nesting_level += 1;
            }

            // Decrement nesting
            if trimmed.contains('}') {
                nesting_level = nesting_level.saturating_sub(1usize);
            }

            // Add complexity based on constructs
            if trimmed.contains("if ") || trimmed.contains("elif ") || trimmed.contains("else if") {
                complexity += 1 + nesting_level;
            }
            if trimmed.contains("while ") || trimmed.contains("for ") {
                complexity += 1 + nesting_level;
            }
            if trimmed.contains("catch ") || trimmed.contains("except ") {
                complexity += 1 + nesting_level;
            }
        }

        complexity
    }

    /// Calculate Halstead complexity metrics (simplified)
    pub fn calculate_halstead_metrics(&self, content: &str) -> (f64, f64, f64) {
        // Simplified Halstead calculation
        let operators = [
            "=", "+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=", "&&", "||",
        ];
        let mut unique_operators = HashSet::new();
        let mut total_operators = 0;

        for op in &operators {
            let count = content.matches(op).count();
            if count > 0 {
                unique_operators.insert(op);
                total_operators += count;
            }
        }

        // Rough operand estimation (identifiers, literals)
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut unique_operands = HashSet::new();
        let mut total_operands = 0;

        for word in words {
            if word.chars().any(|c| c.is_alphanumeric()) {
                unique_operands.insert(word);
                total_operands += 1;
            }
        }

        let n1 = unique_operators.len().max(1) as f64; // Minimum 1 operator
        let n2 = unique_operands.len().max(1) as f64; // Minimum 1 operand
        let big_n1 = total_operators.max(1) as f64; // Minimum 1 operator usage
        let big_n2 = total_operands.max(1) as f64; // Minimum 1 operand usage

        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;

        // Ensure vocabulary is at least 2 to avoid log2(1) = 0
        let safe_vocabulary = vocabulary.max(2.0);
        let volume = length * safe_vocabulary.log2();

        // Safe difficulty calculation
        let difficulty = (n1 / 2.0) * (big_n2 / n2);
        let effort = difficulty * volume;

        (volume, difficulty, effort)
    }

    /// Calculate maintainability index (simplified)
    pub fn calculate_maintainability_index(&self, content: &str, lines_count: usize) -> f64 {
        let (volume, difficulty, _effort) = self.calculate_halstead_metrics(content);
        let cyclomatic = self.calculate_cyclomatic_complexity(content) as f64;
        let loc = lines_count.max(1) as f64; // Minimum 1 line

        // Ensure volume is meaningful for logarithm
        let safe_volume = volume.max(1.0);
        let safe_loc = loc.max(1.0);

        // Adjusted maintainability index formula to be more sensitive
        // Based on the standard formula but with adjusted coefficients for this simplified implementation
        // Higher volume, complexity, and difficulty should decrease maintainability more significantly
        let volume_penalty = safe_volume.ln() * 8.0; // Increased from 5.2
        let complexity_penalty = cyclomatic * 5.0; // Increased from 0.23
        let loc_penalty = safe_loc.ln() * 20.0; // Increased from 16.2
        let difficulty_penalty = difficulty * 2.0; // Add difficulty factor

        let mi = 171.0 - volume_penalty - complexity_penalty - loc_penalty - difficulty_penalty;

        // Ensure result is in valid range
        mi.clamp(0.0, 100.0)
    }
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclomatic_complexity() {
        let analyzer = ComplexityAnalyzer::new();

        let simple_code = "function test() { return 1; }";
        assert_eq!(analyzer.calculate_cyclomatic_complexity(simple_code), 1);

        let complex_code = "if (x) { if (y) { while (z) { for (i in items) { } } } }";
        assert!(analyzer.calculate_cyclomatic_complexity(complex_code) > 1);
    }

    #[test]
    fn test_cognitive_complexity() {
        let analyzer = ComplexityAnalyzer::new();

        let simple_code = "function test() { return 1; }";
        assert_eq!(analyzer.calculate_cognitive_complexity(simple_code), 0);

        let nested_code = "if (x) {\n  if (y) {\n    while (z) {\n    }\n  }\n}";
        assert!(analyzer.calculate_cognitive_complexity(nested_code) > 0);
    }

    #[test]
    fn test_halstead_metrics() {
        let analyzer = ComplexityAnalyzer::new();

        let code = "x = a + b * c";
        let (volume, difficulty, effort) = analyzer.calculate_halstead_metrics(code);

        assert!(volume > 0.0);
        assert!(difficulty > 0.0);
        assert!(effort > 0.0);
    }

    #[test]
    fn test_maintainability_index() {
        let analyzer = ComplexityAnalyzer::new();

        let simple_code = "function test() { return 1; }";
        let mi = analyzer.calculate_maintainability_index(simple_code, 1);

        assert!((0.0..=100.0).contains(&mi));
    }
}
