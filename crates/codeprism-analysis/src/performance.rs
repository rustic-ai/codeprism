//! Performance analysis module

use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// Performance issue information
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
    pub complexity_estimate: Option<String>,
    pub impact_score: Option<f64>,
    pub optimization_effort: Option<String>,
}

/// Recursive complexity information
#[derive(Debug, Clone, Serialize)]
pub struct RecursiveComplexity {
    pub function_name: String,
    pub depth_estimate: String,
    pub complexity: String,
    pub optimization_potential: String,
}

/// Memory allocation pattern
#[derive(Debug, Clone, Serialize)]
pub struct MemoryPattern {
    pub pattern_type: String,
    pub allocation_frequency: String,
    pub impact: String,
    pub recommendation: String,
}

/// Performance analyzer for code analysis
pub struct PerformanceAnalyzer {
    patterns: HashMap<String, Vec<PerformancePattern>>,
    language_specific_patterns: HashMap<String, Vec<PerformancePattern>>,
}

#[derive(Debug, Clone)]
struct PerformancePattern {
    name: String,
    pattern: Regex,
    severity: String,
    description: String,
    recommendation: String,
    complexity: String,
    impact_score: f64,
    optimization_effort: String,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
            language_specific_patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer.initialize_language_specific_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Enhanced time complexity patterns
        let time_patterns = vec![
            PerformancePattern {
                name: "Quadruple Nested Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+.*?for\s+.*?for\s+").unwrap(),
                severity: "critical".to_string(),
                description: "Quadruple nested loop detected - O(n⁴) complexity".to_string(),
                recommendation: "Critical: Consider algorithmic redesign, data preprocessing, or caching strategies".to_string(),
                complexity: "O(n⁴)".to_string(),
                impact_score: 10.0,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "Triple Nested Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+.*?for\s+").unwrap(),
                severity: "critical".to_string(),
                description: "Triple nested loop detected - O(n³) complexity".to_string(),
                recommendation: "Consider algorithmic optimization, matrix operations, or divide-and-conquer approaches".to_string(),
                complexity: "O(n³)".to_string(),
                impact_score: 8.5,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "Double Nested Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+").unwrap(),
                severity: "high".to_string(),
                description: "Double nested loop detected - O(n²) complexity".to_string(),
                recommendation: "Consider if this can be optimized to O(n log n) using sorting or O(n) using hash tables".to_string(),
                complexity: "O(n²)".to_string(),
                impact_score: 6.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "Exponential Recursion".to_string(),
                pattern: Regex::new(r"(?s)(def|function)\s+\w+.*?(\w+\([^)]*\)\s*\+\s*\w+\([^)]*\))").unwrap(),
                severity: "critical".to_string(),
                description: "Potential exponential recursion pattern - O(2^n) complexity".to_string(),
                recommendation: "Implement memoization, dynamic programming, or iterative solution".to_string(),
                complexity: "O(2^n)".to_string(),
                impact_score: 10.0,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "Factorial Complexity".to_string(),
                pattern: Regex::new(r"(?i)(factorial|permutation|all.*combinations)").unwrap(),
                severity: "critical".to_string(), 
                description: "Potential factorial complexity detected - O(n!) complexity".to_string(),
                recommendation: "Use pruning, branch-and-bound, or approximation algorithms".to_string(),
                complexity: "O(n!)".to_string(),
                impact_score: 10.0,
                optimization_effort: "Very High".to_string(),
            },
            PerformancePattern {
                name: "Inefficient String Concatenation".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?[\+=]\s*(str|string|\w+)").unwrap(),
                severity: "high".to_string(),
                description: "String concatenation in loop - O(n²) complexity due to immutable strings".to_string(),
                recommendation: "Use StringBuilder, StringBuffer, list.join(), or similar efficient methods".to_string(),
                complexity: "O(n²)".to_string(),
                impact_score: 5.0,
                optimization_effort: "Low".to_string(),
            },
            PerformancePattern {
                name: "Inefficient Collection Search".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(list|array)\.contains\s*\(").unwrap(),
                severity: "medium".to_string(),
                description: "Linear search in loop creates O(n²) complexity".to_string(),
                recommendation: "Convert to Set or HashMap for O(1) lookups, reducing to O(n) overall".to_string(),
                complexity: "O(n²)".to_string(),
                impact_score: 4.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.patterns
            .insert("time_complexity".to_string(), time_patterns);

        // Enhanced memory usage patterns
        let memory_patterns = vec![
            PerformancePattern {
                name: "Large Object Creation in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?new\s+\w+\s*\(").unwrap(),
                severity: "high".to_string(),
                description:
                    "Frequent large object allocation causing GC pressure and memory fragmentation"
                        .to_string(),
                recommendation:
                    "Use object pooling, factory patterns, or move allocation outside loop"
                        .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 6.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "Memory Leak Pattern".to_string(),
                pattern: Regex::new(r"(?i)(global|static)\s+\w+\s*=\s*\[\]").unwrap(),
                severity: "critical".to_string(),
                description: "Global/static collection may grow indefinitely causing memory leaks"
                    .to_string(),
                recommendation:
                    "Implement proper cleanup, use bounded collections, or WeakReference patterns"
                        .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 9.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "Excessive Buffer Allocation".to_string(),
                pattern: Regex::new(
                    r"(?i)(buffer|byte\[\]|char\[\])\s*=\s*new\s+.*?\[\s*\d{4,}\s*\]",
                )
                .unwrap(),
                severity: "medium".to_string(),
                description: "Large buffer allocation may cause memory pressure".to_string(),
                recommendation:
                    "Use streaming, chunked processing, or memory-mapped files for large data"
                        .to_string(),
                complexity: "O(1)".to_string(),
                impact_score: 4.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "String Interning Issues".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?new\s+String\s*\(").unwrap(),
                severity: "medium".to_string(),
                description: "Excessive string object creation in loop".to_string(),
                recommendation: "Use string interning, StringBuilder, or string constants"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 3.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.patterns
            .insert("memory_usage".to_string(), memory_patterns);

        // Enhanced hot spots patterns
        let hotspot_patterns = vec![
            PerformancePattern {
                name: "Database Query in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(query|execute|select|insert|update|delete)\s*\(").unwrap(),
                severity: "critical".to_string(),
                description: "Database query inside loop - classic N+1 query problem".to_string(),
                recommendation: "Use batch operations, joins, or implement query result caching".to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 9.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "File I/O in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(open|read|write|close|file)\s*\(").unwrap(),
                severity: "high".to_string(),
                description: "File I/O operations inside loop causing excessive disk access".to_string(),
                recommendation: "Batch file operations, use streaming, or implement file caching".to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 7.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "Network Call in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(http|fetch|request|get|post|ajax)\s*\(").unwrap(),
                severity: "critical".to_string(),
                description: "Network calls inside loop causing severe latency issues".to_string(),
                recommendation: "Use batch APIs, parallel processing with connection pooling, or async/await patterns".to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 9.5,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "Synchronous I/O Blocking".to_string(),
                pattern: Regex::new(r"(?i)(sync|synchronous|blocking)\s*(read|write|call|request)").unwrap(),
                severity: "high".to_string(),
                description: "Synchronous I/O operations blocking thread execution".to_string(),
                recommendation: "Implement async/await patterns, non-blocking I/O, or worker thread pools".to_string(),
                complexity: "O(1)".to_string(),
                impact_score: 6.0,
                optimization_effort: "High".to_string(),
            },
        ];
        self.patterns
            .insert("hot_spots".to_string(), hotspot_patterns);

        // Concurrency bottleneck patterns
        let concurrency_patterns = vec![
            PerformancePattern {
                name: "Thread Contention".to_string(),
                pattern: Regex::new(r"(?i)(synchronized|lock|mutex|semaphore)\s*\([^)]*\)\s*\{[^}]*for").unwrap(),
                severity: "high".to_string(),
                description: "Lock contention in loop causing thread blocking and reduced parallelism".to_string(),
                recommendation: "Use lock-free algorithms, reduce critical section size, or implement fine-grained locking".to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 7.0,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "False Sharing".to_string(),
                pattern: Regex::new(r"(?i)(shared|volatile)\s+\w+\s*\[\s*\]").unwrap(),
                severity: "medium".to_string(),
                description: "Potential false sharing causing cache line invalidation".to_string(),
                recommendation: "Use cache line padding, thread-local storage, or redesign data structures".to_string(),
                complexity: "O(1)".to_string(),
                impact_score: 5.0,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "Thread Pool Exhaustion".to_string(),
                pattern: Regex::new(r"(?i)thread\s*\.\s*start\s*\(\)").unwrap(),
                severity: "medium".to_string(),
                description: "Manual thread creation may lead to thread exhaustion".to_string(),
                recommendation: "Use managed thread pools, async/await, or reactive patterns".to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 4.0,
                optimization_effort: "Medium".to_string(),
            },
        ];
        self.patterns
            .insert("concurrency_bottlenecks".to_string(), concurrency_patterns);

        // Algorithm-specific patterns
        let algorithm_patterns = vec![
            PerformancePattern {
                name: "Inefficient Sorting".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?for\s+.*?(swap|exchange|compare)").unwrap(),
                severity: "medium".to_string(),
                description: "Bubble sort or similar O(n²) sorting algorithm detected".to_string(),
                recommendation:
                    "Use built-in sort functions, quicksort, mergesort, or heapsort for O(n log n)"
                        .to_string(),
                complexity: "O(n²)".to_string(),
                impact_score: 5.0,
                optimization_effort: "Low".to_string(),
            },
            PerformancePattern {
                name: "Linear Search in Sorted Data".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(sorted|ordered).*?==").unwrap(),
                severity: "medium".to_string(),
                description: "Linear search in sorted data structure".to_string(),
                recommendation: "Use binary search for O(log n) complexity instead of O(n)"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 4.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.patterns
            .insert("algorithm_patterns".to_string(), algorithm_patterns);

        // Performance regression patterns
        let regression_patterns = vec![
            PerformancePattern {
                name: "Caching Disabled".to_string(),
                pattern: Regex::new(r"(?i)(cache\s*=\s*false|no.?cache|disable.*cache)").unwrap(),
                severity: "medium".to_string(),
                description: "Caching appears to be disabled or bypassed".to_string(),
                recommendation: "Review caching strategy and ensure proper cache utilization"
                    .to_string(),
                complexity: "O(1)".to_string(),
                impact_score: 6.0,
                optimization_effort: "Low".to_string(),
            },
            PerformancePattern {
                name: "Debug Code in Production".to_string(),
                pattern: Regex::new(r"(?i)(debug|trace|verbose)\s*(=\s*true|logging|print)")
                    .unwrap(),
                severity: "low".to_string(),
                description: "Debug code may impact production performance".to_string(),
                recommendation:
                    "Remove debug statements or use conditional compilation for production builds"
                        .to_string(),
                complexity: "O(1)".to_string(),
                impact_score: 2.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.patterns
            .insert("regression_patterns".to_string(), regression_patterns);
    }

    fn initialize_language_specific_patterns(&mut self) {
        // Python-specific patterns
        let python_patterns = vec![
            PerformancePattern {
                name: "Global Interpreter Lock Contention".to_string(),
                pattern: Regex::new(r"(?i)threading.*for\s+.*?in").unwrap(),
                severity: "high".to_string(),
                description: "Threading in CPU-bound loop affected by Python GIL".to_string(),
                recommendation:
                    "Use multiprocessing, asyncio, or consider Cython/PyPy for CPU-bound tasks"
                        .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 7.0,
                optimization_effort: "High".to_string(),
            },
            PerformancePattern {
                name: "List Comprehension Opportunity".to_string(),
                pattern: Regex::new(r"(?s)for\s+\w+\s+in\s+.*?\.append\s*\(").unwrap(),
                severity: "low".to_string(),
                description: "Loop with append can be replaced with list comprehension".to_string(),
                recommendation: "Use list comprehension for better performance and readability"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 2.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.language_specific_patterns
            .insert("python".to_string(), python_patterns);

        // JavaScript-specific patterns
        let js_patterns = vec![
            PerformancePattern {
                name: "DOM Manipulation in Loop".to_string(),
                pattern: Regex::new(
                    r"(?s)for\s+.*?(appendChild|removeChild|innerHTML|createElement)",
                )
                .unwrap(),
                severity: "high".to_string(),
                description: "DOM manipulation inside loop causing layout thrashing".to_string(),
                recommendation: "Batch DOM changes using DocumentFragment or virtual DOM patterns"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 8.0,
                optimization_effort: "Medium".to_string(),
            },
            PerformancePattern {
                name: "Prototype Chain Lookup".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?hasOwnProperty").unwrap(),
                severity: "low".to_string(),
                description: "Prototype chain traversal in loop".to_string(),
                recommendation: "Cache property lookups or use Map/Set for better performance"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 2.0,
                optimization_effort: "Low".to_string(),
            },
        ];
        self.language_specific_patterns
            .insert("javascript".to_string(), js_patterns);

        // Java-specific patterns
        let java_patterns = vec![
            PerformancePattern {
                name: "String Concatenation in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?String\s+.*?\+\s*").unwrap(),
                severity: "high".to_string(),
                description: "String concatenation in loop creating many intermediate objects"
                    .to_string(),
                recommendation: "Use StringBuilder or StringBuffer for efficient string building"
                    .to_string(),
                complexity: "O(n²)".to_string(),
                impact_score: 6.0,
                optimization_effort: "Low".to_string(),
            },
            PerformancePattern {
                name: "Boxing in Loop".to_string(),
                pattern: Regex::new(r"(?s)for\s+.*?(Integer|Double|Boolean|Float)\s*\(").unwrap(),
                severity: "medium".to_string(),
                description: "Autoboxing/unboxing in loop creating wrapper objects".to_string(),
                recommendation: "Use primitive collections or avoid autoboxing in hot code paths"
                    .to_string(),
                complexity: "O(n)".to_string(),
                impact_score: 4.0,
                optimization_effort: "Medium".to_string(),
            },
        ];
        self.language_specific_patterns
            .insert("java".to_string(), java_patterns);
    }

    /// Analyze content for performance issues with enhanced capabilities
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
                        // Find all matches, not just the first one
                        for mat in pattern.pattern.find_iter(content) {
                            issues.push(PerformanceIssue {
                                issue_type: pattern.name.clone(),
                                severity: pattern.severity.clone(),
                                description: pattern.description.clone(),
                                location: Some(self.get_line_info(content, mat.start())),
                                recommendation: pattern.recommendation.clone(),
                                complexity_estimate: Some(pattern.complexity.clone()),
                                impact_score: Some(pattern.impact_score),
                                optimization_effort: Some(pattern.optimization_effort.clone()),
                            });
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyze recursive function complexity
    pub fn analyze_recursive_complexity(&self, content: &str) -> Result<Vec<RecursiveComplexity>> {
        let mut recursive_functions = Vec::new();

        // Pattern for recursive functions (simplified since Rust regex doesn't support backreferences)
        let recursive_pattern = Regex::new(r"(?s)(def|function)\s+(\w+)").unwrap();

        for captures in recursive_pattern.captures_iter(content) {
            if let Some(func_name) = captures.get(2) {
                let function_name = func_name.as_str().to_string();

                // Check if the function actually calls itself by searching for the function name in the content
                if content.contains(&format!("{function_name}(")) {
                    // Count occurrences to determine if it's likely recursive
                    let call_count = content.matches(&format!("{function_name}(")).count();
                    if call_count > 1 {
                        // Function definition + at least one call
                        // Analyze recursion depth and complexity
                        let (depth_estimate, complexity, optimization_potential) =
                            self.estimate_recursive_complexity(content, &function_name);

                        recursive_functions.push(RecursiveComplexity {
                            function_name,
                            depth_estimate,
                            complexity,
                            optimization_potential,
                        });
                    }
                }
            }
        }

        Ok(recursive_functions)
    }

    /// Analyze memory allocation patterns
    pub fn analyze_memory_patterns(&self, content: &str) -> Result<Vec<MemoryPattern>> {
        let mut patterns = Vec::new();

        // Analyze various memory allocation patterns
        let allocation_patterns = vec![
            (
                r"(?s)for\s+.*?new\s+",
                "Loop Allocation",
                "High",
                "High GC pressure",
            ),
            (
                r"(?s)new\s+.*?\[\s*\d{4,}\s*\]",
                "Large Array",
                "Medium",
                "Memory fragmentation",
            ),
            (
                r"(?i)(arraylist|vector|list)\s*\(\s*\)",
                "Default Capacity",
                "Low",
                "Potential resizing overhead",
            ),
        ];

        for (pattern_str, pattern_type, frequency, impact) in allocation_patterns {
            let pattern = Regex::new(pattern_str).unwrap();
            if pattern.is_match(content) {
                patterns.push(MemoryPattern {
                    pattern_type: pattern_type.to_string(),
                    allocation_frequency: frequency.to_string(),
                    impact: impact.to_string(),
                    recommendation: self.get_memory_recommendation(pattern_type),
                });
            }
        }

        Ok(patterns)
    }

    /// Get architectural performance recommendations
    pub fn get_architectural_recommendations(&self, issues: &[PerformanceIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let _total_impact: f64 = issues.iter().filter_map(|i| i.impact_score).sum();

        let critical_issues: usize = issues.iter().filter(|i| i.severity == "critical").count();

        let high_issues: usize = issues.iter().filter(|i| i.severity == "high").count();

        // Architectural recommendations based on issue patterns
        if critical_issues > 0 {
            recommendations.push("CRITICAL: Immediate architectural review required".to_string());
            recommendations
                .push("Consider implementing performance monitoring and alerting".to_string());
        }

        if _total_impact > 30.0 {
            recommendations.push(
                "High performance impact detected - consider performance testing".to_string(),
            );
        }

        if high_issues > 3 {
            recommendations.push(
                "Multiple high-impact issues - implement staged optimization approach".to_string(),
            );
        }

        // Specific architectural patterns
        if issues.iter().any(|i| i.issue_type.contains("Database")) {
            recommendations.push("Database Architecture: Implement connection pooling, read replicas, and query optimization".to_string());
        }

        if issues.iter().any(|i| i.issue_type.contains("Network")) {
            recommendations.push(
                "Network Architecture: Consider CDN, caching layers, and circuit breaker patterns"
                    .to_string(),
            );
        }

        if issues.iter().any(|i| i.issue_type.contains("Memory")) {
            recommendations.push("Memory Architecture: Implement memory profiling, garbage collection tuning, and memory-efficient data structures".to_string());
        }

        if issues
            .iter()
            .any(|i| i.issue_type.contains("Concurrency") || i.issue_type.contains("Thread"))
        {
            recommendations.push("Concurrency Architecture: Consider actor model, reactive streams, or lock-free algorithms".to_string());
        }

        recommendations.push(
            "Implement comprehensive performance benchmarking and continuous monitoring"
                .to_string(),
        );

        recommendations
    }

    fn estimate_recursive_complexity(
        &self,
        _content: &str,
        function_name: &str,
    ) -> (String, String, String) {
        // Simplified complexity estimation - in practice would analyze call patterns
        if function_name.contains("fib") || function_name.contains("factorial") {
            (
                "Deep".to_string(),
                "O(2^n)".to_string(),
                "High - implement memoization".to_string(),
            )
        } else {
            (
                "Moderate".to_string(),
                "O(n)".to_string(),
                "Medium - consider iterative approach".to_string(),
            )
        }
    }

    fn get_memory_recommendation(&self, pattern_type: &str) -> String {
        match pattern_type {
            "Loop Allocation" => "Move allocation outside loop or use object pooling".to_string(),
            "Large Array" => "Consider streaming or chunked processing".to_string(),
            "Default Capacity" => "Initialize collections with expected capacity".to_string(),
            _ => "Review memory allocation patterns".to_string(),
        }
    }

    fn get_line_info(&self, content: &str, position: usize) -> String {
        let line_num = content[..position].matches('\n').count() + 1;
        format!("Line: {line_num}, Position: {position}")
    }

    /// Enhanced complexity threshold checking
    fn meets_complexity_threshold(&self, complexity: &str, threshold: &str) -> bool {
        match threshold {
            "low" => true, // Include all complexities
            "medium" => !complexity.contains("O(1)") && !matches!(complexity, "O(log n)"),
            "high" => {
                complexity.contains("O(n²)")
                    || complexity.contains("O(n³)")
                    || complexity.contains("O(n⁴)")
                    || complexity.contains("O(2^n)")
                    || complexity.contains("O(n!)")
            }
            _ => true,
        }
    }

    /// Enhanced performance recommendations with architectural guidance
    pub fn get_performance_recommendations(&self, issues: &[PerformanceIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if issues.is_empty() {
            recommendations.push(
                "No obvious performance issues detected. Continue monitoring with profiling tools."
                    .to_string(),
            );
            return recommendations;
        }

        // Group by issue type and calculate priorities
        let mut issue_counts = HashMap::new();
        let mut _total_impact = 0.0;
        for issue in issues {
            *issue_counts.entry(issue.issue_type.clone()).or_insert(0) += 1;
            if let Some(impact) = issue.impact_score {
                _total_impact += impact;
            }
        }

        // Prioritized recommendations based on impact
        let mut priority_issues: Vec<_> = issue_counts.iter().collect();
        priority_issues.sort_by(|a, b| b.1.cmp(a.1));

        // Critical issue recommendations
        if issue_counts.contains_key("Database Query in Loop") {
            recommendations.push("HIGH PRIORITY: Eliminate N+1 query problems with batch operations and proper ORM usage".to_string());
        }

        if issue_counts.contains_key("Exponential Recursion")
            || issue_counts.contains_key("Factorial Complexity")
        {
            recommendations.push("CRITICAL: Implement dynamic programming or iterative solutions for exponential algorithms".to_string());
        }

        if issue_counts.contains_key("Network Call in Loop") {
            recommendations.push(
                "CRITICAL: Implement async/batch processing for network operations".to_string(),
            );
        }

        // Algorithmic recommendations
        if issue_counts.contains_key("Triple Nested Loop")
            || issue_counts.contains_key("Quadruple Nested Loop")
        {
            recommendations.push("ALGORITHM: Review data structures and consider preprocessing or caching strategies".to_string());
        }

        if issue_counts.contains_key("Double Nested Loop") {
            recommendations.push(
                "ALGORITHM: Consider hash-based lookups or sorting-based optimizations".to_string(),
            );
        }

        // Memory optimization recommendations
        if issue_counts.contains_key("Large Object Creation in Loop") {
            recommendations.push(
                "MEMORY: Implement object pooling or move allocations outside hot paths"
                    .to_string(),
            );
        }

        if issue_counts.contains_key("Memory Leak Pattern") {
            recommendations.push(
                "MEMORY: Implement proper resource cleanup and bounded collection strategies"
                    .to_string(),
            );
        }

        // Concurrency recommendations
        if issue_counts.contains_key("Thread Contention") {
            recommendations.push("CONCURRENCY: Reduce lock contention with fine-grained locking or lock-free algorithms".to_string());
        }

        // General architectural recommendations
        recommendations.extend(self.get_architectural_recommendations(issues));

        // Tool recommendations
        recommendations.push(
            "MONITORING: Implement APM tools for continuous performance monitoring".to_string(),
        );
        recommendations.push(
            "PROFILING: Use language-specific profilers to validate optimizations".to_string(),
        );
        recommendations
            .push("TESTING: Implement performance regression tests in CI/CD pipeline".to_string());

        recommendations
    }

    /// Analyze time complexity issues
    pub fn analyze_time_complexity(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["time_complexity".to_string()], "low")?;

        Ok(issues
            .into_iter()
            .map(|i| {
                serde_json::json!({
                    "type": i.issue_type,
                    "severity": i.severity,
                    "description": i.description,
                    "location": i.location,
                    "recommendation": i.recommendation,
                    "complexity": i.complexity_estimate,
                    "impact_score": i.impact_score,
                    "optimization_effort": i.optimization_effort
                })
            })
            .collect())
    }

    /// Analyze memory usage issues
    pub fn analyze_memory_usage(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["memory_usage".to_string()], "low")?;

        Ok(issues
            .into_iter()
            .map(|i| {
                serde_json::json!({
                    "type": i.issue_type,
                    "severity": i.severity,
                    "description": i.description,
                    "location": i.location,
                    "recommendation": i.recommendation,
                    "complexity": i.complexity_estimate,
                    "impact_score": i.impact_score,
                    "optimization_effort": i.optimization_effort
                })
            })
            .collect())
    }

    /// Detect performance hot spots
    pub fn detect_performance_hot_spots(&self, content: &str) -> Result<Vec<Value>> {
        let issues = self.analyze_content(content, &["hot_spots".to_string()], "low")?;

        Ok(issues
            .into_iter()
            .map(|i| {
                serde_json::json!({
                    "type": i.issue_type,
                    "severity": i.severity,
                    "description": i.description,
                    "location": i.location,
                    "recommendation": i.recommendation,
                    "complexity": i.complexity_estimate,
                    "impact_score": i.impact_score,
                    "optimization_effort": i.optimization_effort
                })
            })
            .collect())
    }

    /// Detect concurrency bottlenecks
    pub fn detect_concurrency_bottlenecks(&self, content: &str) -> Result<Vec<Value>> {
        let issues =
            self.analyze_content(content, &["concurrency_bottlenecks".to_string()], "low")?;

        Ok(issues
            .into_iter()
            .map(|i| {
                serde_json::json!({
                    "type": i.issue_type,
                    "severity": i.severity,
                    "description": i.description,
                    "location": i.location,
                    "recommendation": i.recommendation,
                    "complexity": i.complexity_estimate,
                    "impact_score": i.impact_score,
                    "optimization_effort": i.optimization_effort
                })
            })
            .collect())
    }

    /// Comprehensive performance analysis
    pub fn comprehensive_analysis(&self, content: &str, language: Option<&str>) -> Result<Value> {
        let mut all_issues = Vec::new();

        // Run all analysis types
        let analysis_types = vec![
            "time_complexity".to_string(),
            "memory_usage".to_string(),
            "hot_spots".to_string(),
            "concurrency_bottlenecks".to_string(),
            "algorithm_patterns".to_string(),
            "regression_patterns".to_string(),
        ];

        for analysis_type in analysis_types {
            let issues = self.analyze_content(content, &[analysis_type], "low")?;
            all_issues.extend(issues);
        }

        // Add language-specific analysis if specified
        if let Some(lang) = language {
            if let Some(lang_patterns) = self.language_specific_patterns.get(lang) {
                for pattern in lang_patterns {
                    for mat in pattern.pattern.find_iter(content) {
                        all_issues.push(PerformanceIssue {
                            issue_type: pattern.name.clone(),
                            severity: pattern.severity.clone(),
                            description: pattern.description.clone(),
                            location: Some(self.get_line_info(content, mat.start())),
                            recommendation: pattern.recommendation.clone(),
                            complexity_estimate: Some(pattern.complexity.clone()),
                            impact_score: Some(pattern.impact_score),
                            optimization_effort: Some(pattern.optimization_effort.clone()),
                        });
                    }
                }
            }
        }

        // Analyze recursive complexity
        let recursive_analysis = self.analyze_recursive_complexity(content)?;

        // Analyze memory patterns
        let memory_patterns = self.analyze_memory_patterns(content)?;

        // Generate comprehensive recommendations
        let recommendations = self.get_performance_recommendations(&all_issues);
        let architectural_recommendations = self.get_architectural_recommendations(&all_issues);

        // Calculate summary statistics
        let total_issues = all_issues.len();
        let critical_issues = all_issues
            .iter()
            .filter(|i| i.severity == "critical")
            .count();
        let high_issues = all_issues.iter().filter(|i| i.severity == "high").count();
        let _total_impact: f64 = all_issues.iter().filter_map(|i| i.impact_score).sum();

        Ok(serde_json::json!({
            "summary": {
                "total_issues": total_issues,
                "critical_issues": critical_issues,
                "high_issues": high_issues,
                "total_impact_score": _total_impact,
                "performance_grade": self.calculate_performance_grade(_total_impact, total_issues)
            },
            "issues": all_issues.iter().map(|i| serde_json::json!({
                "type": i.issue_type,
                "severity": i.severity,
                "description": i.description,
                "location": i.location,
                "recommendation": i.recommendation,
                "complexity": i.complexity_estimate,
                "impact_score": i.impact_score,
                "optimization_effort": i.optimization_effort
            })).collect::<Vec<_>>(),
            "recursive_analysis": recursive_analysis,
            "memory_patterns": memory_patterns,
            "recommendations": recommendations,
            "architectural_recommendations": architectural_recommendations,
            "language_specific": language.unwrap_or("generic")
        }))
    }

    fn calculate_performance_grade(&self, total_impact: f64, issue_count: usize) -> String {
        let average_impact = if issue_count > 0 {
            total_impact / issue_count as f64
        } else {
            0.0
        };

        match average_impact {
            x if x >= 8.0 => "F - Critical Performance Issues".to_string(),
            x if x >= 6.0 => "D - Significant Performance Issues".to_string(),
            x if x >= 4.0 => "C - Moderate Performance Issues".to_string(),
            x if x >= 2.0 => "B - Minor Performance Issues".to_string(),
            _ => "A - Good Performance".to_string(),
        }
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
    fn test_enhanced_nested_loop_detection() {
        let analyzer = PerformanceAnalyzer::new();

        let code =
            "for i in range(n): for j in range(n): for k in range(n): for l in range(n): pass";
        let issues = analyzer
            .analyze_content(code, &["time_complexity".to_string()], "low")
            .unwrap();

        assert!(
            !issues.is_empty(),
            "Should detect performance issues in quadruple nested loop"
        );

        // Validate specific detection of quadruple nested loop
        let nested_loop_issue = issues
            .iter()
            .find(|i| i.issue_type == "Quadruple Nested Loop")
            .expect("Should detect quadruple nested loop issue");

        assert!(
            nested_loop_issue.severity == "high" || nested_loop_issue.severity == "critical",
            "Quadruple nested loop should be high/critical severity, got: {}",
            nested_loop_issue.severity
        );
        assert!(
            !nested_loop_issue.description.is_empty(),
            "Issue should have meaningful description"
        );
    }

    #[test]
    fn test_exponential_recursion_detection() {
        let analyzer = PerformanceAnalyzer::new();

        let code = "def fibonacci(n): return fibonacci(n-1) + fibonacci(n-2)";
        let issues = analyzer
            .analyze_content(code, &["time_complexity".to_string()], "low")
            .unwrap();

        assert!(!issues.is_empty(), "Should detect exponential recursion");

        // Validate we detected the actual exponential recursion pattern
        let recursion_issue = issues
            .iter()
            .find(|i| i.issue_type.contains("Recursion") || i.issue_type.contains("Exponential"))
            .expect("Should detect recursion-related performance issue");

        assert!(
            recursion_issue.severity == "high" || recursion_issue.severity == "critical",
            "Exponential recursion should be high severity"
        );
        assert!(
            recursion_issue.description.contains("fibonacci")
                || recursion_issue
                    .description
                    .to_lowercase()
                    .contains("recursion"),
            "Description should reference the actual issue"
        );
    }

    #[test]
    fn test_concurrency_bottleneck_detection() {
        let analyzer = PerformanceAnalyzer::new();

        let code = "synchronized(lock) { for(int i = 0; i < n; i++) { process(i); } }";
        let issues = analyzer
            .analyze_content(code, &["concurrency_bottlenecks".to_string()], "low")
            .unwrap();

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == "Thread Contention"));
    }

    #[test]
    fn test_comprehensive_analysis() {
        let analyzer = PerformanceAnalyzer::new();

        let code = "for i in range(n): for j in range(n): query('SELECT * FROM table')";
        let result = analyzer
            .comprehensive_analysis(code, Some("python"))
            .unwrap();

        assert!(result.get("summary").is_some());
        assert!(result.get("issues").is_some());
        assert!(result.get("recommendations").is_some());
    }

    #[test]
    fn test_memory_pattern_analysis() {
        let analyzer = PerformanceAnalyzer::new();

        let code = "for i in range(n): obj = new LargeObject()";
        let patterns = analyzer.analyze_memory_patterns(code).unwrap();

        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_performance_grade_calculation() {
        let analyzer = PerformanceAnalyzer::new();

        assert_eq!(
            analyzer.calculate_performance_grade(10.0, 1),
            "F - Critical Performance Issues"
        );
        assert_eq!(
            analyzer.calculate_performance_grade(1.0, 1),
            "A - Good Performance"
        );
    }

    #[test]
    fn test_database_query_in_loop() {
        let analyzer = PerformanceAnalyzer::new();

        let code =
            "for user in users:\n    query(\"SELECT * FROM orders WHERE user_id = ?\", user.id)";
        let issues = analyzer
            .analyze_content(code, &["hot_spots".to_string()], "low")
            .unwrap();

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "Database Query in Loop"));
    }

    #[test]
    fn test_string_concatenation() {
        let analyzer = PerformanceAnalyzer::new();

        let code = "for item in items: result += str(item)";
        let issues = analyzer
            .analyze_content(code, &["time_complexity".to_string()], "low")
            .unwrap();

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "Inefficient String Concatenation"));
    }

    #[test]
    fn test_enhanced_complexity_threshold() {
        let analyzer = PerformanceAnalyzer::new();

        assert!(analyzer.meets_complexity_threshold("O(n²)", "medium"));
        assert!(!analyzer.meets_complexity_threshold("O(1)", "medium"));
        assert!(analyzer.meets_complexity_threshold("O(n⁴)", "high"));
        assert!(analyzer.meets_complexity_threshold("O(2^n)", "high"));
    }

    #[test]
    fn test_enhanced_performance_recommendations() {
        let analyzer = PerformanceAnalyzer::new();

        let issues = vec![PerformanceIssue {
            issue_type: "Database Query in Loop".to_string(),
            severity: "critical".to_string(),
            description: "Test".to_string(),
            location: None,
            recommendation: "Test".to_string(),
            complexity_estimate: Some("O(n)".to_string()),
            impact_score: Some(9.0),
            optimization_effort: Some("Medium".to_string()),
        }];

        let recommendations = analyzer.get_performance_recommendations(&issues);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("HIGH PRIORITY")));
    }
}
