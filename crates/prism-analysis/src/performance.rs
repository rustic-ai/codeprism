//! Performance analysis module

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

/// Performance issue information
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
    pub complexity_estimate: Option<String>,
}

/// Performance analyzer for code analysis
pub struct PerformanceAnalyzer {
    patterns: HashMap<String, Vec<PerformancePattern>>,
}

#[derive(Debug, Clone)]
struct PerformancePattern {
    name: String,
    pattern: Regex,
    severity: String,
    description: String,
    recommendation: String,
    complexity: String,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Time complexity patterns
        let time_patterns = vec![
            PerformancePattern {
                name: "Nested Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+.*?for\s+").unwrap(),
                severity: "high".to_string(),
                description: "Triple nested loop detected - O(n³) complexity".to_string(),
                recommendation: "Consider algorithmic optimization or data structure changes".to_string(),
                complexity: "O(n³)".to_string(),
            },
            PerformancePattern {
                name: "Double Nested Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+").unwrap(),
                severity: "medium".to_string(),
                description: "Double nested loop detected - O(n²) complexity".to_string(),
                recommendation: "Consider if this can be optimized to O(n log n) or O(n)".to_string(),
                complexity: "O(n²)".to_string(),
            },
            PerformancePattern {
                name: "Inefficient String Concatenation".to_string(),
                pattern: Regex::new(r"(?i)(str|string)\s*\+=\s*").unwrap(),
                severity: "medium".to_string(),
                description: "String concatenation in loop can be inefficient".to_string(),
                recommendation: "Use StringBuilder, list.join(), or similar efficient methods".to_string(),
                complexity: "O(n²)".to_string(),
            },
        ];
        self.patterns.insert("time_complexity".to_string(), time_patterns);

        // Memory usage patterns
        let memory_patterns = vec![
            PerformancePattern {
                name: "Large Object Creation in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?new\s+\w+\s*\(").unwrap(),
                severity: "medium".to_string(),
                description: "Object creation inside loop may cause memory pressure".to_string(),
                recommendation: "Consider object pooling or moving creation outside loop".to_string(),
                complexity: "O(n)".to_string(),
            },
            PerformancePattern {
                name: "Potential Memory Leak".to_string(),
                pattern: Regex::new(r"(?i)(global|static)\s+\w+\s*=\s*\[\]").unwrap(),
                severity: "high".to_string(),
                description: "Global/static collection may grow indefinitely".to_string(),
                recommendation: "Implement proper cleanup or use bounded collections".to_string(),
                complexity: "O(n)".to_string(),
            },
        ];
        self.patterns.insert("memory_usage".to_string(), memory_patterns);

        // Hot spots patterns
        let hotspot_patterns = vec![
            PerformancePattern {
                name: "Database Query in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(query|execute|select|insert|update|delete)\s*\(").unwrap(),
                severity: "critical".to_string(),
                description: "Database query inside loop - N+1 query problem".to_string(),
                recommendation: "Use batch operations or optimize with joins".to_string(),
                complexity: "O(n)".to_string(),
            },
            PerformancePattern {
                name: "File I/O in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(open|read|write|close)\s*\(").unwrap(),
                severity: "high".to_string(),
                description: "File I/O operations inside loop".to_string(),
                recommendation: "Batch file operations or use streaming approaches".to_string(),
                complexity: "O(n)".to_string(),
            },
            PerformancePattern {
                name: "Network Call in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(http|fetch|request|get|post)\s*\(").unwrap(),
                severity: "critical".to_string(),
                description: "Network calls inside loop".to_string(),
                recommendation: "Use batch APIs or parallel processing".to_string(),
                complexity: "O(n)".to_string(),
            },
        ];
        self.patterns.insert("hot_spots".to_string(), hotspot_patterns);

        // Anti-patterns
        let antipattern_patterns = vec![
            PerformancePattern {
                name: "Premature Optimization".to_string(),
                pattern: Regex::new(r"(?i)(micro.?optimization|premature.?optimization)").unwrap(),
                severity: "low".to_string(),
                description: "Potential premature optimization detected".to_string(),
                recommendation: "Profile first, then optimize based on actual bottlenecks".to_string(),
                complexity: "Variable".to_string(),
            },
            PerformancePattern {
                name: "Inefficient Collection Usage".to_string(),
                pattern: Regex::new(r"(?i)(list|array)\.contains\s*\(.*?\)").unwrap(),
                severity: "medium".to_string(),
                description: "Linear search in collection".to_string(),
                recommendation: "Consider using Set or HashMap for O(1) lookups".to_string(),
                complexity: "O(n)".to_string(),
            },
        ];
        self.patterns.insert("anti_patterns".to_string(), antipattern_patterns);

        // Scalability patterns
        let scalability_patterns = vec![
            PerformancePattern {
                name: "Synchronous Processing".to_string(),
                pattern: Regex::new(r"(?i)(synchronous|blocking|wait|sleep)\s*\(").unwrap(),
                severity: "medium".to_string(),
                description: "Synchronous operations may not scale well".to_string(),
                recommendation: "Consider asynchronous processing for better scalability".to_string(),
                complexity: "O(1)".to_string(),
            },
            PerformancePattern {
                name: "Single-threaded Processing".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(process|compute|calculate)\s*\([^)]*\)").unwrap(),
                severity: "low".to_string(),
                description: "Sequential processing of independent tasks".to_string(),
                recommendation: "Consider parallel processing for CPU-intensive tasks".to_string(),
                complexity: "O(n)".to_string(),
            },
        ];
        self.patterns.insert("scalability".to_string(), scalability_patterns);
    }

    /// Analyze content for performance issues
    pub fn analyze_content(
        &self,
        content: &str,
        analysis_types: &[String],
        complexity_threshold: &str,
    ) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();
        
        let target_types = if analysis_types.contains(&"all".to_string()) {
            self.patterns.keys().cloned().collect::<Vec<_>>()
        } else {
            analysis_types.to_vec()
        };

        for analysis_type in target_types {
            if let Some(patterns) = self.patterns.get(&analysis_type) {
                for pattern in patterns {
                    if self.meets_complexity_threshold(&pattern.complexity, complexity_threshold) {
                        if let Some(captures) = pattern.pattern.find(content) {
                            issues.push(PerformanceIssue {
                                issue_type: pattern.name.clone(),
                                severity: pattern.severity.clone(),
                                description: pattern.description.clone(),
                                location: Some(format!("Position: {}", captures.start())),
                                recommendation: pattern.recommendation.clone(),
                                complexity_estimate: Some(pattern.complexity.clone()),
                            });
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Check if complexity meets threshold
    fn meets_complexity_threshold(&self, complexity: &str, threshold: &str) -> bool {
        // Simple complexity comparison - in practice, this would be more sophisticated
        match threshold {
            "low" => true, // Include all complexities
            "medium" => !complexity.contains("O(1)"),
            "high" => complexity.contains("O(n²)") || complexity.contains("O(n³)") || complexity.contains("O(2^n)"),
            _ => true,
        }
    }

    /// Get performance recommendations
    pub fn get_performance_recommendations(&self, issues: &[PerformanceIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if issues.is_empty() {
            recommendations.push("No obvious performance issues detected. Continue monitoring with profiling tools.".to_string());
            return recommendations;
        }

        // Group by issue type
        let mut issue_counts = HashMap::new();
        for issue in issues {
            *issue_counts.entry(issue.issue_type.clone()).or_insert(0) += 1;
        }

        // General recommendations based on found issues
        if issue_counts.contains_key("Database Query in Loop") {
            recommendations.push("Optimize database access patterns to avoid N+1 query problems.".to_string());
        }

        if issue_counts.contains_key("Nested Loop") || issue_counts.contains_key("Double Nested Loop") {
            recommendations.push("Review algorithmic complexity and consider more efficient algorithms.".to_string());
        }

        if issue_counts.contains_key("Network Call in Loop") {
            recommendations.push("Implement batch processing or async patterns for network operations.".to_string());
        }

        if issue_counts.contains_key("Large Object Creation in Loop") {
            recommendations.push("Consider object pooling or factory patterns to reduce allocation overhead.".to_string());
        }

        recommendations.push("Use profiling tools to identify actual bottlenecks in production.".to_string());
        recommendations.push("Implement performance monitoring and alerting.".to_string());
        recommendations.push("Consider caching strategies for frequently accessed data.".to_string());
        
        recommendations
    }

    /// Analyze time complexity issues
    pub fn analyze_time_complexity(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["time_complexity".to_string()], "low")?;
        
        Ok(issues.into_iter().map(|i| serde_json::json!({
            "type": i.issue_type,
            "severity": i.severity,
            "description": i.description,
            "location": i.location,
            "recommendation": i.recommendation,
            "complexity": i.complexity_estimate
        })).collect())
    }

    /// Analyze memory usage issues
    pub fn analyze_memory_usage(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["memory_usage".to_string()], "low")?;
        
        Ok(issues.into_iter().map(|i| serde_json::json!({
            "type": i.issue_type,
            "severity": i.severity,
            "description": i.description,
            "location": i.location,
            "recommendation": i.recommendation,
            "complexity": i.complexity_estimate
        })).collect())
    }

    /// Detect performance hot spots
    pub fn detect_performance_hot_spots(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["hot_spots".to_string()], "low")?;
        
        Ok(issues.into_iter().map(|i| serde_json::json!({
            "type": i.issue_type,
            "severity": i.severity,
            "description": i.description,
            "location": i.location,
            "recommendation": i.recommendation,
            "complexity": i.complexity_estimate
        })).collect())
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_loop_detection() {
        let analyzer = PerformanceAnalyzer::new();
        
        let code = "for i in range(n): for j in range(n): for k in range(n): pass";
        let issues = analyzer.analyze_content(code, &["time_complexity".to_string()], "low").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == "Nested Loop"));
    }

    #[test]
    fn test_database_query_in_loop() {
        let analyzer = PerformanceAnalyzer::new();
        
        let code = "for user in users:\n    query(\"SELECT * FROM orders WHERE user_id = ?\", user.id)";
        let issues = analyzer.analyze_content(code, &["hot_spots".to_string()], "low").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == "Database Query in Loop"));
    }

    #[test]
    fn test_string_concatenation() {
        let analyzer = PerformanceAnalyzer::new();
        
        let code = "result = \"\"; str += item";
        let issues = analyzer.analyze_content(code, &["time_complexity".to_string()], "low").unwrap();
        
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == "Inefficient String Concatenation"));
    }

    #[test]
    fn test_complexity_threshold() {
        let analyzer = PerformanceAnalyzer::new();
        
        assert!(analyzer.meets_complexity_threshold("O(n²)", "medium"));
        assert!(!analyzer.meets_complexity_threshold("O(1)", "medium"));
        assert!(analyzer.meets_complexity_threshold("O(n³)", "high"));
    }

    #[test]
    fn test_performance_recommendations() {
        let analyzer = PerformanceAnalyzer::new();
        
        let issues = vec![
            PerformanceIssue {
                issue_type: "Database Query in Loop".to_string(),
                severity: "critical".to_string(),
                description: "Test".to_string(),
                location: None,
                recommendation: "Test".to_string(),
                complexity_estimate: Some("O(n)".to_string()),
            }
        ];
        
        let recommendations = analyzer.get_performance_recommendations(&issues);
        assert!(!recommendations.is_empty());
    }
} 