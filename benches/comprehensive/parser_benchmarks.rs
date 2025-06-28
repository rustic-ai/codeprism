//! Comprehensive Parser Performance Benchmarks
//!
//! Benchmarks for all language parsers to establish performance baselines
//! and detect performance regressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::path::PathBuf;
use std::time::Duration;

// Import all parser crates
use codeprism_lang_python::{RustParser as PythonParser, ParseContext as PythonParseContext};
use codeprism_lang_js::{RustParser as JavaScriptParser, ParseContext as JavaScriptParseContext};
use codeprism_lang_rust::{RustParser, ParseContext as RustParseContext};

/// Benchmark data for different file sizes
const SMALL_FILE_SIZE: usize = 1_000;      // 1KB
const MEDIUM_FILE_SIZE: usize = 10_000;    // 10KB  
const LARGE_FILE_SIZE: usize = 100_000;    // 100KB
const XLARGE_FILE_SIZE: usize = 1_000_000; // 1MB

/// Python parser benchmarks
fn benchmark_python_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("python_parser");
    
    // Test different file sizes
    for size in [SMALL_FILE_SIZE, MEDIUM_FILE_SIZE, LARGE_FILE_SIZE] {
        group.throughput(Throughput::Bytes(size as u64));
        
        let content = generate_python_code(size);
        let context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("benchmark.py"),
            old_tree: None,
            content: content.clone(),
        };

        group.bench_with_input(
            BenchmarkId::new("parse", size),
            &(content, context),
            |b, (_, ctx)| {
                let mut parser = PythonParser::new();
                b.iter(|| {
                    let result = parser.parse(black_box(ctx));
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// JavaScript parser benchmarks
fn benchmark_javascript_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("javascript_parser");
    
    for size in [SMALL_FILE_SIZE, MEDIUM_FILE_SIZE, LARGE_FILE_SIZE] {
        group.throughput(Throughput::Bytes(size as u64));
        
        let content = generate_javascript_code(size);
        let context = JavaScriptParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("benchmark.js"),
            old_tree: None,
            content: content.clone(),
        };

        group.bench_with_input(
            BenchmarkId::new("parse", size),
            &(content, context),
            |b, (_, ctx)| {
                let mut parser = JavaScriptParser::new();
                b.iter(|| {
                    let result = parser.parse(black_box(ctx));
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Rust parser benchmarks
fn benchmark_rust_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_parser");
    
    for size in [SMALL_FILE_SIZE, MEDIUM_FILE_SIZE, LARGE_FILE_SIZE] {
        group.throughput(Throughput::Bytes(size as u64));
        
        let content = generate_rust_code(size);
        let context = RustParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("benchmark.rs"),
            old_tree: None,
            content: content.clone(),
        };

        group.bench_with_input(
            BenchmarkId::new("parse", size),
            &(content, context),
            |b, (_, ctx)| {
                let mut parser = RustParser::new();
                b.iter(|| {
                    let result = parser.parse(black_box(ctx));
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Incremental parsing benchmarks
fn benchmark_incremental_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_parsing");
    
    // Test incremental parsing performance
    let base_content = generate_python_code(MEDIUM_FILE_SIZE);
    let modified_content = format!("{}\n# New comment", base_content);
    
    group.bench_function("python_incremental", |b| {
        let mut parser = PythonParser::new();
        
        // Parse initial content
        let initial_context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("benchmark.py"),
            old_tree: None,
            content: base_content.clone(),
        };
        
        let initial_result = parser.parse(&initial_context).unwrap();
        
        // Benchmark incremental parse
        let incremental_context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("benchmark.py"),
            old_tree: Some(initial_result.tree),
            content: modified_content.clone(),
        };
        
        b.iter(|| {
            let result = parser.parse(black_box(&incremental_context));
            black_box(result)
        });
    });
    
    group.finish();
}

/// Memory usage benchmarks
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(30));
    
    // Test memory efficiency with large files
    group.bench_function("large_python_file", |b| {
        let content = generate_python_code(XLARGE_FILE_SIZE);
        let context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("large.py"),
            old_tree: None,
            content,
        };
        
        b.iter(|| {
            let mut parser = PythonParser::new();
            let result = parser.parse(black_box(&context));
            black_box(result)
        });
    });
    
    group.finish();
}

/// Batch parsing benchmarks
fn benchmark_batch_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parsing");
    
    // Test parsing multiple files
    let file_counts = [10, 50, 100];
    
    for count in file_counts {
        group.bench_with_input(
            BenchmarkId::new("python_files", count),
            &count,
            |b, &count| {
                let files: Vec<_> = (0..count)
                    .map(|i| {
                        PythonParseContext {
                            repo_id: "benchmark".to_string(),
                            file_path: PathBuf::from(format!("file_{}.py", i)),
                            old_tree: None,
                            content: generate_python_code(SMALL_FILE_SIZE),
                        }
                    })
                    .collect();
                
                b.iter(|| {
                    let mut parser = PythonParser::new();
                    for file in &files {
                        let result = parser.parse(black_box(file));
                        black_box(result);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Complex code structure benchmarks
fn benchmark_complex_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_structures");
    
    // Test deeply nested code
    group.bench_function("deep_nesting", |b| {
        let content = generate_deeply_nested_python(20); // 20 levels deep
        let context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("deep.py"),
            old_tree: None,
            content,
        };
        
        b.iter(|| {
            let mut parser = PythonParser::new();
            let result = parser.parse(black_box(&context));
            black_box(result)
        });
    });
    
    // Test many small functions
    group.bench_function("many_functions", |b| {
        let content = generate_many_python_functions(100); // 100 functions
        let context = PythonParseContext {
            repo_id: "benchmark".to_string(),
            file_path: PathBuf::from("many_funcs.py"),
            old_tree: None,
            content,
        };
        
        b.iter(|| {
            let mut parser = PythonParser::new();
            let result = parser.parse(black_box(&context));
            black_box(result)
        });
    });
    
    group.finish();
}

// Code generation functions

fn generate_python_code(target_size: usize) -> String {
    let mut code = String::new();
    
    code.push_str("#!/usr/bin/env python3\n");
    code.push_str("import os\nimport sys\nimport json\nfrom typing import List, Dict, Optional\n\n");
    
    let mut current_size = code.len();
    let mut func_count = 0;
    
    while current_size < target_size {
        let func = format!(
            r#"
def function_{}(param1: str, param2: int = 42) -> Dict[str, any]:
    """
    This is function number {}.
    It does some complex processing and returns a dictionary.
    """
    result = {{}}
    for i in range(param2):
        if i % 2 == 0:
            result[f"even_{{i}}"] = param1 + str(i)
        else:
            result[f"odd_{{i}}"] = param1.upper() + str(i)
    
    # Some conditional logic
    if len(result) > 10:
        result["large"] = True
        result["size"] = len(result)
    else:
        result["small"] = True
    
    return result

class Class{}:
    def __init__(self, value: int):
        self.value = value
        self.data = function_{}("prefix", value)
    
    def process(self) -> str:
        return json.dumps(self.data, indent=2)

"#,
            func_count, func_count, func_count, func_count
        );
        
        code.push_str(&func);
        current_size = code.len();
        func_count += 1;
    }
    
    code
}

fn generate_javascript_code(target_size: usize) -> String {
    let mut code = String::new();
    
    code.push_str("// Generated JavaScript code for benchmarking\n");
    code.push_str("const fs = require('fs');\nconst path = require('path');\n\n");
    
    let mut current_size = code.len();
    let mut func_count = 0;
    
    while current_size < target_size {
        let func = format!(
            r#"
function function{}(param1, param2 = 42) {{
    /**
     * Function number {}
     * Performs complex processing and returns an object
     */
    const result = {{}};
    
    for (let i = 0; i < param2; i++) {{
        if (i % 2 === 0) {{
            result[`even_${{i}}`] = param1 + i.toString();
        }} else {{
            result[`odd_${{i}}`] = param1.toUpperCase() + i.toString();
        }}
    }}
    
    // Conditional processing
    if (Object.keys(result).length > 10) {{
        result.large = true;
        result.size = Object.keys(result).length;
    }} else {{
        result.small = true;
    }}
    
    return result;
}}

class Class{} {{
    constructor(value) {{
        this.value = value;
        this.data = function{}("prefix", value);
    }}
    
    process() {{
        return JSON.stringify(this.data, null, 2);
    }}
}}

"#,
            func_count, func_count, func_count, func_count
        );
        
        code.push_str(&func);
        current_size = code.len();
        func_count += 1;
    }
    
    code
}

fn generate_rust_code(target_size: usize) -> String {
    let mut code = String::new();
    
    code.push_str("//! Generated Rust code for benchmarking\n\n");
    code.push_str("use std::collections::HashMap;\nuse serde::{Serialize, Deserialize};\n\n");
    
    let mut current_size = code.len();
    let mut func_count = 0;
    
    while current_size < target_size {
        let func = format!(
            r#"
/// Function number {}
/// Performs complex processing and returns a HashMap
pub fn function_{}(param1: &str, param2: i32) -> HashMap<String, String> {{
    let mut result = HashMap::new();
    
    for i in 0..param2 {{
        if i % 2 == 0 {{
            result.insert(format!("even_{{}}", i), format!("{{}}{{}}", param1, i));
        }} else {{
            result.insert(format!("odd_{{}}", i), format!("{{}}{{}}", param1.to_uppercase(), i));
        }}
    }}
    
    // Conditional logic
    if result.len() > 10 {{
        result.insert("large".to_string(), "true".to_string());
        result.insert("size".to_string(), result.len().to_string());
    }} else {{
        result.insert("small".to_string(), "true".to_string());
    }}
    
    result
}}

#[derive(Debug, Serialize, Deserialize)]
pub struct Struct{} {{
    value: i32,
    data: HashMap<String, String>,
}}

impl Struct{} {{
    pub fn new(value: i32) -> Self {{
        Self {{
            value,
            data: function_{}("prefix", value),
        }}
    }}
    
    pub fn process(&self) -> String {{
        serde_json::to_string_pretty(&self.data).unwrap_or_default()
    }}
}}

"#,
            func_count, func_count, func_count, func_count, func_count
        );
        
        code.push_str(&func);
        current_size = code.len();
        func_count += 1;
    }
    
    code
}

fn generate_deeply_nested_python(depth: usize) -> String {
    let mut code = String::new();
    
    // Create deeply nested function calls
    for i in 0..depth {
        code.push_str(&format!("def nested_function_{}():\n", i));
        code.push_str(&format!("{}if True:\n", "    ".repeat(i + 1)));
        code.push_str(&format!("{}try:\n", "    ".repeat(i + 2)));
        code.push_str(&format!("{}for j in range(10):\n", "    ".repeat(i + 3)));
        code.push_str(&format!("{}if j % 2 == 0:\n", "    ".repeat(i + 4)));
    }
    
    // Close all the nesting
    for i in (0..depth).rev() {
        code.push_str(&format!("{}return {}\n", "    ".repeat(i + 5), i));
        code.push_str(&format!("{}except Exception:\n", "    ".repeat(i + 3)));
        code.push_str(&format!("{}pass\n", "    ".repeat(i + 4)));
    }
    
    code
}

fn generate_many_python_functions(count: usize) -> String {
    let mut code = String::new();
    
    for i in 0..count {
        code.push_str(&format!(
            r#"
def simple_function_{}():
    """Simple function number {}"""
    x = {} * 2
    y = x + 1
    return x * y

"#,
            i, i, i
        ));
    }
    
    code
}

criterion_group!(
    parser_benches,
    benchmark_python_parser,
    benchmark_javascript_parser,
    benchmark_rust_parser,
    benchmark_incremental_parsing,
    benchmark_memory_usage,
    benchmark_batch_parsing,
    benchmark_complex_structures
);

criterion_main!(parser_benches); 