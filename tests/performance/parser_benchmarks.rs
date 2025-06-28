//! Parser performance benchmarks for all supported languages

use super::{PerformanceTestHarness, PerformanceThresholds, PerformanceResult};
use std::time::{Duration, Instant};
use std::path::PathBuf;

// Import actual parser modules
use codeprism_lang_python::parser::{PythonParser, ParseContext as PythonParseContext};
use codeprism_lang_js::parser::{JavaScriptParser, ParseContext as JsParseContext};
use codeprism_lang_rust::parser::{RustParser, ParseContext as RustParseContext};
use codeprism_lang_java::parser::{JavaParser, ParseContext as JavaParseContext};

/// Test sample code sizes
const SMALL_CODE_LINES: usize = 100;
const MEDIUM_CODE_LINES: usize = 1000;
const LARGE_CODE_LINES: usize = 10000;

/// Memory usage measurement result
#[derive(Debug)]
pub struct MemoryUsage {
    pub peak_memory_kb: usize,
    pub allocation_count: usize,
}

/// Performance metrics for parser benchmarks
#[derive(Debug)]
pub struct ParserPerformanceMetrics {
    pub parse_time: Duration,
    pub ast_node_count: usize,
    pub memory_usage: MemoryUsage,
    pub lines_per_second: f64,
}

/// Language parser benchmarks
pub struct ParserBenchmarks;

impl ParserBenchmarks {
    /// Run all parser benchmarks
    pub fn run_all_benchmarks() -> Vec<PerformanceResult> {
        let mut harness = PerformanceTestHarness::new();
        
        // Python parser benchmarks
        Self::benchmark_python_parser(&mut harness);
        
        // JavaScript parser benchmarks  
        Self::benchmark_javascript_parser(&mut harness);
        
        // Rust parser benchmarks
        Self::benchmark_rust_parser(&mut harness);
        
        // Java parser benchmarks
        Self::benchmark_java_parser(&mut harness);
        
        harness.results().to_vec()
    }

    fn benchmark_python_parser(harness: &mut PerformanceTestHarness) {
        harness.run_test("python_small_file", || {
            let python_code = Self::generate_python_code(SMALL_CODE_LINES);
            Self::parse_python_code(&python_code)
        });

        harness.run_test("python_medium_file", || {
            let python_code = Self::generate_python_code(MEDIUM_CODE_LINES);
            Self::parse_python_code(&python_code)
        });

        harness.run_test("python_large_file", || {
            let python_code = Self::generate_python_code(LARGE_CODE_LINES);
            Self::parse_python_code(&python_code)
        });
    }

    fn benchmark_javascript_parser(harness: &mut PerformanceTestHarness) {
        harness.run_test("javascript_small_file", || {
            let js_code = Self::generate_javascript_code(SMALL_CODE_LINES);
            Self::parse_javascript_code(&js_code)
        });

        harness.run_test("javascript_medium_file", || {
            let js_code = Self::generate_javascript_code(MEDIUM_CODE_LINES);
            Self::parse_javascript_code(&js_code)
        });
    }

    fn benchmark_rust_parser(harness: &mut PerformanceTestHarness) {
        harness.run_test("rust_small_file", || {
            let rust_code = Self::generate_rust_code(SMALL_CODE_LINES);
            Self::parse_rust_code(&rust_code)
        });
    }

    fn benchmark_java_parser(harness: &mut PerformanceTestHarness) {
        harness.run_test("java_small_file", || {
            let java_code = Self::generate_java_code(SMALL_CODE_LINES);
            Self::parse_java_code(&java_code)
        });
    }

    // Code generators for benchmarking
    fn generate_python_code(lines: usize) -> String {
        let mut code = String::from("import sys\nimport os\nfrom typing import Dict, List, Optional\n\n");
        
        // Add classes and functions for more realistic parsing
        code.push_str("class DataProcessor:\n");
        code.push_str("    def __init__(self, config: Dict[str, str]):\n");
        code.push_str("        self.config = config\n\n");
        
        for i in 0..lines {
            if i % 10 == 0 {
                code.push_str(&format!("class Class{}:\n", i));
                code.push_str(&format!("    def method_{}(self, param: int) -> str:\n", i));
                code.push_str(&format!("        return f'result_{{param}}'\n\n"));
            } else {
                code.push_str(&format!("def function_{}(arg1: str, arg2: Optional[int] = None) -> Dict[str, int]:\n", i));
                code.push_str(&format!("    result = {{'value': {}, 'arg1': len(arg1)}}\n", i));
                code.push_str(&format!("    if arg2 is not None:\n"));
                code.push_str(&format!("        result['arg2'] = arg2\n"));
                code.push_str(&format!("    return result\n\n"));
            }
        }
        code
    }

    fn generate_javascript_code(lines: usize) -> String {
        let mut code = String::from("const fs = require('fs');\nconst path = require('path');\n\n");
        
        // Add classes and modern JavaScript features
        code.push_str("class DataManager {\n");
        code.push_str("    constructor(config) {\n");
        code.push_str("        this.config = config;\n");
        code.push_str("    }\n\n");
        
        for i in 0..lines {
            if i % 10 == 0 {
                code.push_str(&format!("    async method{}(data) {{\n", i));
                code.push_str(&format!("        return await this.processData{}(data);\n", i));
                code.push_str("    }\n\n");
            } else {
                code.push_str(&format!("const func{} = (param1, param2 = {}) => {{\n", i, i));
                code.push_str(&format!("    const result = {{ value: {}, param1, param2 }};\n", i));
                code.push_str("    return Object.freeze(result);\n");
                code.push_str("};\n\n");
            }
        }
        code.push_str("}\n\n");
        code.push_str("module.exports = { DataManager };\n");
        code
    }

    fn generate_rust_code(lines: usize) -> String {
        let mut code = String::from("use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\nuse serde::{Serialize, Deserialize};\n\n");
        
        // Add structs and traits for realistic Rust code
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        code.push_str("pub struct Config {\n");
        code.push_str("    pub name: String,\n");
        code.push_str("    pub value: i32,\n");
        code.push_str("}\n\n");
        
        code.push_str("pub trait DataProcessor<T> {\n");
        code.push_str("    fn process(&self, data: T) -> Result<T, Box<dyn std::error::Error>>;\n");
        code.push_str("}\n\n");
        
        for i in 0..lines {
            if i % 15 == 0 {
                code.push_str(&format!("pub struct Processor{} {{\n", i));
                code.push_str("    config: Arc<Mutex<Config>>,\n");
                code.push_str("}\n\n");
                code.push_str(&format!("impl DataProcessor<i32> for Processor{} {{\n", i));
                code.push_str("    fn process(&self, data: i32) -> Result<i32, Box<dyn std::error::Error>> {\n");
                code.push_str(&format!("        Ok(data + {})\n", i));
                code.push_str("    }\n");
                code.push_str("}\n\n");
            } else {
                code.push_str(&format!("pub fn func_{}(param: &str) -> Result<String, Box<dyn std::error::Error>> {{\n", i));
                code.push_str(&format!("    let result = format!(\"processed_{{}}_{{}}\", param, {});\n", i));
                code.push_str("    Ok(result)\n");
                code.push_str("}\n\n");
            }
        }
        code
    }

    fn generate_java_code(lines: usize) -> String {
        let mut code = String::from("import java.util.*;\nimport java.util.concurrent.*;\nimport java.util.stream.*;\n\n");
        code.push_str("public class GeneratedClass {\n");
        
        // Add fields and constructor
        code.push_str("    private final Map<String, Integer> config;\n");
        code.push_str("    private final ExecutorService executor;\n\n");
        
        code.push_str("    public GeneratedClass(Map<String, Integer> config) {\n");
        code.push_str("        this.config = new HashMap<>(config);\n");
        code.push_str("        this.executor = Executors.newFixedThreadPool(4);\n");
        code.push_str("    }\n\n");
        
        for i in 0..lines {
            if i % 10 == 0 {
                code.push_str(&format!("    public CompletableFuture<String> processAsync{}(List<String> data) {{\n", i));
                code.push_str("        return CompletableFuture.supplyAsync(() -> {\n");
                code.push_str(&format!("            return data.stream().map(s -> s + \"_{}\").collect(Collectors.joining(\",\"));\n", i));
                code.push_str("        }, executor);\n");
                code.push_str("    }\n\n");
            } else {
                code.push_str(&format!("    public Optional<Integer> method{}(String input, int defaultValue) {{\n", i));
                code.push_str("        try {\n");
                code.push_str(&format!("            int result = Integer.parseInt(input) + {} + defaultValue;\n", i));
                code.push_str("            return Optional.of(result);\n");
                code.push_str("        } catch (NumberFormatException e) {\n");
                code.push_str("            return Optional.empty();\n");
                code.push_str("        }\n");
                code.push_str("    }\n\n");
            }
        }
        code.push_str("}\n");
        code
    }

    // Real parser implementations with performance metrics
    fn parse_python_code(code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let mut parser = PythonParser::new();
        let context = PythonParseContext {
            repo_id: "benchmark_repo".to_string(),
            file_path: PathBuf::from("benchmark.py"),
            old_tree: None,
            content: code.to_string(),
        };

        let result = parser.parse(&context)?;
        let parse_time = start_time.elapsed();
        
        // Calculate metrics
        let lines_per_second = (code.lines().count() as f64) / parse_time.as_secs_f64();
        
        // Log performance metrics for analysis
        eprintln!("Python parser metrics: {} nodes, {} edges, {:.2} lines/sec", 
                  result.nodes.len(), result.edges.len(), lines_per_second);
        
        Ok(())
    }

    fn parse_javascript_code(code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let mut parser = JavaScriptParser::new();
        let context = JsParseContext {
            repo_id: "benchmark_repo".to_string(),
            file_path: PathBuf::from("benchmark.js"),
            old_tree: None,
            content: code.to_string(),
        };

        let result = parser.parse(&context)?;
        let parse_time = start_time.elapsed();
        
        // Calculate metrics
        let lines_per_second = (code.lines().count() as f64) / parse_time.as_secs_f64();
        
        // Log performance metrics for analysis
        eprintln!("JavaScript parser metrics: {} nodes, {} edges, {:.2} lines/sec", 
                  result.nodes.len(), result.edges.len(), lines_per_second);
        
        Ok(())
    }

    fn parse_rust_code(code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let mut parser = RustParser::new();
        let context = RustParseContext {
            repo_id: "benchmark_repo".to_string(),
            file_path: PathBuf::from("benchmark.rs"),
            old_tree: None,
            content: code.to_string(),
        };

        let result = parser.parse(&context)?;
        let parse_time = start_time.elapsed();
        
        // Calculate metrics
        let lines_per_second = (code.lines().count() as f64) / parse_time.as_secs_f64();
        
        // Log performance metrics for analysis
        eprintln!("Rust parser metrics: {} nodes, {} edges, {:.2} lines/sec", 
                  result.nodes.len(), result.edges.len(), lines_per_second);
        
        Ok(())
    }

    fn parse_java_code(code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let mut parser = JavaParser::new();
        let context = JavaParseContext {
            repo_id: "benchmark_repo".to_string(),
            file_path: PathBuf::from("benchmark.java"),
            old_tree: None,
            content: code.to_string(),
        };

        let result = parser.parse(&context)?;
        let parse_time = start_time.elapsed();
        
        // Calculate metrics
        let lines_per_second = (code.lines().count() as f64) / parse_time.as_secs_f64();
        
        // Log performance metrics for analysis
        eprintln!("Java parser metrics: {} nodes, {} edges, {:.2} lines/sec", 
                  result.nodes.len(), result.edges.len(), lines_per_second);
        
        Ok(())
    }

    /// Measure detailed performance metrics for a parser
    pub fn measure_parser_performance<F>(parser_fn: F, code: &str, language: &str) -> ParserPerformanceMetrics 
    where
        F: FnOnce(&str) -> Result<(usize, usize), Box<dyn std::error::Error>>,
    {
        let start_time = Instant::now();
        
        // Run the parser and get node/edge counts
        let (node_count, edge_count) = parser_fn(code).unwrap_or((0, 0));
        
        let parse_time = start_time.elapsed();
        let line_count = code.lines().count();
        let lines_per_second = if parse_time.as_secs_f64() > 0.0 {
            line_count as f64 / parse_time.as_secs_f64()
        } else {
            0.0
        };

        ParserPerformanceMetrics {
            parse_time,
            ast_node_count: node_count,
            memory_usage: MemoryUsage {
                peak_memory_kb: Self::estimate_memory_usage(code.len(), node_count),
                allocation_count: node_count + edge_count,
            },
            lines_per_second,
        }
    }

    /// Estimate memory usage based on code size and AST node count
    fn estimate_memory_usage(code_size: usize, node_count: usize) -> usize {
        // Rough estimation: 
        // - Code storage: 1x the code size
        // - AST nodes: ~200 bytes per node on average  
        // - Parser overhead: ~50% of node memory
        let code_memory_kb = code_size / 1024;
        let node_memory_kb = (node_count * 200) / 1024;
        let overhead_kb = node_memory_kb / 2;
        
        code_memory_kb + node_memory_kb + overhead_kb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_benchmarks() {
        let results = ParserBenchmarks::run_all_benchmarks();
        assert!(!results.is_empty());
        
        // Verify all benchmarks pass
        for result in &results {
            assert!(result.success, "Benchmark {} failed", result.test_name);
        }
    }

    #[test]
    fn test_code_generation() {
        let python_code = ParserBenchmarks::generate_python_code(5);
        assert!(python_code.contains("def function_"));
        assert!(python_code.contains("class Class"));
        
        let js_code = ParserBenchmarks::generate_javascript_code(3);
        assert!(js_code.contains("const func"));
        assert!(js_code.contains("class DataManager"));
        
        let rust_code = ParserBenchmarks::generate_rust_code(3);
        assert!(rust_code.contains("pub fn func_"));
        assert!(rust_code.contains("pub struct"));
        
        let java_code = ParserBenchmarks::generate_java_code(3);
        assert!(java_code.contains("public"));
        assert!(java_code.contains("method"));
    }

    #[test]
    fn test_real_python_parsing() {
        let code = "def hello():\n    return 'world'";
        let result = ParserBenchmarks::parse_python_code(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_real_javascript_parsing() {
        let code = "function hello() { return 'world'; }";
        let result = ParserBenchmarks::parse_javascript_code(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_real_rust_parsing() {
        let code = "fn hello() -> &'static str { \"world\" }";
        let result = ParserBenchmarks::parse_rust_code(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_real_java_parsing() {
        let code = "public class Test { public void hello() {} }";
        let result = ParserBenchmarks::parse_java_code(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_metrics() {
        let code = "def simple_function():\n    return 42";
        
        let metrics = ParserBenchmarks::measure_parser_performance(
            |code| {
                ParserBenchmarks::parse_python_code(code)?;
                Ok((5, 3)) // Mock node and edge counts for test
            },
            code,
            "Python"
        );
        
        assert!(metrics.parse_time.as_nanos() > 0);
        assert_eq!(metrics.ast_node_count, 5);
        assert!(metrics.lines_per_second > 0.0);
        assert!(metrics.memory_usage.peak_memory_kb > 0);
    }
} 