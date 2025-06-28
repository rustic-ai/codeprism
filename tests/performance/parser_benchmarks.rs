//! Parser performance benchmarks for all supported languages

use super::{PerformanceTestHarness, PerformanceThresholds, PerformanceResult};
use std::time::Duration;

/// Test sample code sizes
const SMALL_CODE_LINES: usize = 100;
const MEDIUM_CODE_LINES: usize = 1000;
const LARGE_CODE_LINES: usize = 10000;

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
        let mut code = String::from("import sys\nimport os\n\n");
        for i in 0..lines {
            code.push_str(&format!("def function_{}():\n    return {}\n\n", i, i));
        }
        code
    }

    fn generate_javascript_code(lines: usize) -> String {
        let mut code = String::from("const fs = require('fs');\n\n");
        for i in 0..lines {
            code.push_str(&format!("function func{}() {{ return {}; }}\n", i, i));
        }
        code
    }

    fn generate_rust_code(lines: usize) -> String {
        let mut code = String::from("use std::collections::HashMap;\n\n");
        for i in 0..lines {
            code.push_str(&format!("fn func_{}() -> i32 {{ {} }}\n", i, i));
        }
        code
    }

    fn generate_java_code(lines: usize) -> String {
        let mut code = String::from("import java.util.*;\n\npublic class Test {\n");
        for i in 0..lines {
            code.push_str(&format!("    public int method{}() {{ return {}; }}\n", i, i));
        }
        code.push_str("}\n");
        code
    }

    // Parser implementations (stubs for now - would integrate with actual parsers)
    fn parse_python_code(_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Integrate with actual Python parser
        std::thread::sleep(Duration::from_micros(100));
        Ok(())
    }

    fn parse_javascript_code(_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Integrate with actual JavaScript parser
        std::thread::sleep(Duration::from_micros(80));
        Ok(())
    }

    fn parse_rust_code(_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Integrate with actual Rust parser
        std::thread::sleep(Duration::from_micros(120));
        Ok(())
    }

    fn parse_java_code(_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Integrate with actual Java parser
        std::thread::sleep(Duration::from_micros(90));
        Ok(())
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
        assert!(python_code.contains("def function_0"));
        assert!(python_code.contains("def function_4"));
        
        let js_code = ParserBenchmarks::generate_javascript_code(3);
        assert!(js_code.contains("function func0"));
        assert!(js_code.contains("function func2"));
    }
} 