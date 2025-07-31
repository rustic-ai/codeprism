//! Integration tests for Rust parser

use codeprism_lang_rust::*;
use std::path::PathBuf;

fn create_test_context(content: &str) -> ParseContext {
    ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: content.to_string(),
    }
}

#[test]
fn test_basic_function_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("fn hello() -> &'static str {\n    \"world\"\n}");

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty(), "Should not be empty");

    // Should have at least a module node and a function node
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Module)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Function)));
}

#[test]
fn test_struct_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("struct Point { x: i32, y: i32 }");

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty(), "Should not be empty");

    // Should have module and struct nodes
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Module)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Struct)));
}

#[test]
fn test_trait_and_impl_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Display { fn fmt(&self); }\nimpl Display for String { fn fmt(&self) {} }",
    );

    let result = parser.parse(&context).unwrap();
    assert!(!result.nodes.is_empty(), "Should not be empty");

    // Should have trait and impl nodes
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Trait)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Impl)));
}

#[test]
fn test_enum_parsing() {
    let mut parser = RustParser::new();
    let context = create_test_context("enum Color { Red, Green, Blue }");

    let result = parser.parse(&context).unwrap();

    // Should have enum node
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Enum)));
}

#[test]
fn test_use_statements() {
    let mut parser = RustParser::new();
    let context =
        create_test_context("use std::collections::HashMap;\nuse serde::{Serialize, Deserialize};");

    let result = parser.parse(&context).unwrap();

    let use_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Use))
        .collect();

    // Should have at least one use node
    assert!(!use_nodes.is_empty(), "Should not be empty");
}

#[test]
fn test_ownership_patterns() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "fn process_data(data: Vec<String>, buffer: &mut [u8], reference: &str) -> &str { reference }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have function and parameter nodes
    let func_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Function))
        .collect();
    assert_eq!(func_nodes.len(), 1, "Should have 1 items");

    let param_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Parameter))
        .collect();

    // Should have parameters with ownership information
    assert!(param_nodes.len() >= 3);

    // Check that at least one parameter has ownership metadata
    let has_ownership_metadata = param_nodes.iter().any(|node| {
        node.metadata
            .as_object()
            .is_some_and(|metadata| metadata.contains_key("ownership"))
    });
    assert!(has_ownership_metadata);
}

#[test]
fn test_lifetime_annotations() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have lifetime nodes
    let lifetime_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Lifetime))
        .collect();

    // Should have at least one lifetime node
    assert!(!lifetime_nodes.is_empty(), "Should not be empty");

    // Check for 'a lifetime
    let has_a_lifetime = lifetime_nodes.iter().any(|node| node.name.contains("'a"));
    assert!(has_a_lifetime);
}

#[test]
fn test_trait_bounds_and_impl() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Clone { fn clone(&self) -> Self; }\nstruct Point { x: i32, y: i32 }\nimpl Clone for Point { fn clone(&self) -> Self { *self } }"
    );

    let result = parser.parse(&context).unwrap();

    // Should have trait and impl nodes
    let trait_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Trait))
        .collect();
    assert_eq!(trait_nodes.len(), 1, "Should have 1 items");

    let impl_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Impl))
        .collect();
    assert_eq!(impl_nodes.len(), 1, "Should have 1 items");

    // Check impl metadata
    let impl_node = &impl_nodes[0];
    assert!(impl_node.metadata.as_object().is_some_and(|metadata| {
        metadata.get("impl_type") == Some(&serde_json::Value::String("trait_impl".to_string()))
    }));
}

#[test]
fn test_derive_attributes() {
    let mut parser = RustParser::new();
    let context =
        create_test_context("#[derive(Debug, Clone, PartialEq)]\nstruct Point { x: i32, y: i32 }");

    let result = parser.parse(&context).unwrap();

    // Should have attribute nodes
    let attr_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Attribute))
        .collect();

    assert!(!attr_nodes.is_empty(), "Should not be empty");

    // Check for derive attribute with traits
    let has_derive_attr = attr_nodes.iter().any(|node| {
        node.name.contains("derive")
            && node.metadata.as_object().is_some_and(|metadata| {
                metadata.get("attribute_type")
                    == Some(&serde_json::Value::String("derive".to_string()))
            })
    });
    assert!(has_derive_attr);
}

#[test]
fn test_macro_invocations() {
    let mut parser = RustParser::new();
    let context = create_test_context("fn main() { println!(\"Hello, world!\"); vec![1, 2, 3]; }");

    let result = parser.parse(&context).unwrap();

    // Should have call nodes for macro invocations
    let call_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Call))
        .collect();

    // Should have at least println! and vec! macro calls
    assert!(call_nodes.len() >= 2);

    // Check for macro call names
    let has_println = call_nodes.iter().any(|node| node.name.contains("println!"));
    let has_vec = call_nodes.iter().any(|node| node.name.contains("vec!"));

    assert!(has_println);
    assert!(has_vec);
}

#[test]
fn test_complex_rust_features() {
    let mut parser = RustParser::new();
    let complex_code = r#"
        use std::collections::HashMap;
        
        #[derive(Debug, Clone)]
        pub struct Config<'a> {
            name: &'a str,
            values: HashMap<String, i32>,
        }
        
        trait Configurable {
            fn configure(&mut self);
        }
        
        impl<'a> Configurable for Config<'a> {
            fn configure(&mut self) {
                self.values.insert("default".to_string(), 42);
            }
        }
        
        fn process<T: Configurable>(mut item: T) -> T {
            item.configure();
            item
        }
        
        const DEFAULT_SIZE: usize = 1024;
        static mut GLOBAL_COUNTER: i32 = 0;
    "#;

    let context = create_test_context(complex_code);
    let result = parser.parse(&context).unwrap();

    // Verify various node types are present
    assert!(result.nodes.iter().any(|n| matches!(n.kind, NodeKind::Use)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Struct)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Trait)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Impl)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Function)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Const)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Static)));
    assert!(result
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Attribute)));
}

#[test]
fn test_rust_analyzer_integration() {
    let mut parser = RustParser::new();
    let context = create_test_context(
        "trait Display { fn fmt(&self); }\nimpl Display for String { fn fmt(&self) {} }",
    );

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);

    // Test trait implementation analysis
    let trait_impls = analyzer.analyze_trait_implementations();
    assert!(!trait_impls.is_empty(), "Should not be empty");

    let display_impl = trait_impls
        .iter()
        .find(|impl_| impl_.trait_name == "Display");
    assert!(display_impl.is_some(), "Should have value");

    if let Some(impl_) = display_impl {
        assert_eq!(impl_.type_name, "String");
    }
}

#[test]
fn test_incremental_parsing() {
    let mut parser = RustParser::new();

    // First parse
    let context1 = create_test_context("fn foo() -> i32 {\n    1\n}");
    let result1 = parser.parse(&context1).unwrap();

    // Second parse with small edit
    let mut context2 = create_test_context("fn foo() -> i32 {\n    2\n}");
    context2.old_tree = Some(result1.tree);
    let result2 = parser.parse(&context2).unwrap();

    // Both should have the same structure
    assert_eq!(result1.nodes.len(), result2.nodes.len());

    // Function should still be found
    let func1 = result1
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();
    let func2 = result2
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Function))
        .unwrap();

    assert_eq!(func1.name, "foo");
    assert_eq!(func2.name, "foo");
}

#[tokio::test]
async fn test_enhanced_rust_analysis_ownership_patterns() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: r#"
fn process_data(data: String, numbers: Vec<i32>) -> String {
    let cloned_data = data.clone();
    let another_clone = cloned_data.clone();
    another_clone
}

fn inefficient_borrowing(text: &str) -> String {
    // This borrowed parameter is used to create an owned value
    text.to_string()
}

fn multiple_mut_borrows(a: &mut Vec<i32>, b: &mut Vec<i32>, c: &mut HashMap<String, i32>) {
    a.push(1);
    b.push(2);
    c.insert("key".to_string(), 3);
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Should detect ownership patterns
    assert!(
        !analysis.ownership_patterns.is_empty(),
        "Should not be empty"
    );

    // Should detect unnecessary owned parameters
    let has_unnecessary_owned = analysis
        .ownership_patterns
        .iter()
        .any(|pattern| matches!(pattern.pattern_type, OwnershipPatternType::UnnecessaryOwned));
    assert!(has_unnecessary_owned);

    // Should detect multiple mutable borrows
    let has_multiple_mut_borrows = analysis.ownership_patterns.iter().any(|pattern| {
        matches!(
            pattern.pattern_type,
            OwnershipPatternType::MultipleMutableBorrows
        )
    });
    assert!(has_multiple_mut_borrows);
}

#[tokio::test]
async fn test_enhanced_rust_analysis_performance_issues() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: r#"
use std::collections::HashMap;

fn performance_issues() {
    // Allocation without capacity
    let mut map = HashMap::new();
    let mut vec = Vec::new();
    
    // String concatenation
    let mut result = String::new();
    result.push_str("hello");
    result.push_str("world");
    result.push_str("test");
    result.push_str("more");
    
    // Inefficient sorting
    let mut numbers = vec![3, 1, 4, 1, 5];
    numbers.sort();
    
    // Iterator collect
    let collected: Vec<i32> = vec.iter().map(|x| x * 2).collect();
}

fn takes_vec_unnecessarily(data: Vec<String>) -> usize {
    data.len()
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Should detect performance issues
    assert!(
        !analysis.performance_issues.is_empty(),
        "Should not be empty"
    );

    // Should detect unoptimized collections
    let has_unoptimized_collections = analysis.performance_issues.iter().any(|issue| {
        matches!(
            issue.issue_type,
            PerformanceIssueType::UnoptimizedCollections
        )
    });
    assert!(has_unoptimized_collections);
}

#[tokio::test]
async fn test_enhanced_rust_analysis_safety_issues() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: r#"
use std::ffi::{CString, CStr};

unsafe fn unsafe_function() -> *mut i32 {
    std::ptr::null_mut()
}

extern "C" fn ffi_function(ptr: *const i8) -> i32 {
    unsafe {
        if ptr.is_null() {
            return -1;
        }
        *ptr as i32
    }
}

fn uses_c_strings() {
    let c_string = CString::new("hello").unwrap();
    let c_str = unsafe { CStr::from_ptr(c_string.as_ptr()) };
}

unsafe fn dangerous_transmute() {
    let value = 42u32;
    let bytes: [u8; 4] = std::mem::transmute(value);
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Should detect safety issues
    assert!(!analysis.safety_issues.is_empty(), "Should not be empty");

    // Should detect unsafe functions
    let has_unsafe_function = analysis
        .safety_issues
        .iter()
        .any(|issue| matches!(issue.issue_type, SafetyIssueType::UnsafeFunction));
    assert!(has_unsafe_function);

    // Should detect FFI boundaries
    let has_ffi_boundary = analysis
        .safety_issues
        .iter()
        .any(|issue| matches!(issue.issue_type, SafetyIssueType::FFIBoundary));
    assert!(has_ffi_boundary);
}

#[tokio::test]
async fn test_enhanced_rust_analysis_concurrency_issues() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("test.rs"),
        old_tree: None,
        content: r#"
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use tokio::sync::mpsc;

async fn async_function_with_blocking() {
    // Blocking operation in async context
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let result = some_async_operation();
    // Missing .await
}

async fn some_async_operation() -> i32 {
    42
}

fn multiple_locks() {
    let mutex1 = Arc::new(Mutex::new(0));
    let mutex2 = Arc::new(Mutex::new(0));
    
    let _guard1 = mutex1.lock().unwrap();
    let _guard2 = mutex2.lock().unwrap();
}

fn deadlock_potential_example() {
    use std::sync::{Arc, Mutex};
    let m1 = Arc::new(Mutex::new(0));
    let m2 = Arc::new(Mutex::new(0));
    
    {
        let _lock1 = m1.lock().unwrap();
        let _lock2 = m2.lock().unwrap(); // Multiple locks in sequence
    }
}

fn non_thread_safe_types() {
    let rc_data = Rc::new(RefCell::new(42));
    // This won't be thread-safe
}

fn unbounded_channels() {
    let (tx, rx) = mpsc::unbounded_channel::<i32>();
}

#[derive(Debug)]
struct MyStruct {
    data: i32,
}

unsafe impl Send for MyStruct {}
unsafe impl Sync for MyStruct {}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Should detect concurrency issues
    assert!(
        !analysis.concurrency_issues.is_empty(),
        "Should not be empty"
    );

    // Should detect thread safety violations
    let has_thread_safety_violation = analysis.concurrency_issues.iter().any(|issue| {
        matches!(
            issue.issue_type,
            ConcurrencyIssueType::ThreadSafetyViolation
        )
    });
    assert!(has_thread_safety_violation);

    // Should detect deadlock potential
    let has_deadlock_potential = analysis
        .concurrency_issues
        .iter()
        .any(|issue| matches!(issue.issue_type, ConcurrencyIssueType::DeadlockPotential));
    assert!(has_deadlock_potential);
}

#[tokio::test]
async fn test_comprehensive_rust_analysis() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("comprehensive_test.rs"),
        old_tree: None,
        content: r#"
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct DataProcessor {
    cache: HashMap<String, String>,
    counters: Arc<Mutex<Vec<i32>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(), // Should suggest with_capacity
            counters: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn process_string_data(&self, input: String) -> String {
        // Unnecessary owned parameter
        let cloned = input.clone(); // Unnecessary clone
        cloned.to_uppercase()
    }
    
    pub fn move_semantics_issue(self) -> DataProcessor {
        // Takes self by value - move semantics
        self
    }
    
    pub fn borrowing_issue(&self, data: &str) {
        // Function takes borrowed but stores it (simulated)
        let _stored = data.to_string(); // Could suggest taking ownership
    }
    
    pub fn multiple_mutable_refs(&mut self, a: &mut Vec<i32>, b: &mut HashMap<String, i32>) {
        // Multiple mutable borrows
        a.push(1);
        b.insert("key".to_string(), 1);
    }
    
    pub fn inefficient_string_building(&self) -> String {
        let mut result = String::new();
        result.push_str("part1");
        result.push_str("part2");
        result.push_str("part3");
        result.push_str("part4"); // Multiple string operations
        result
    }
    
    pub fn performance_issues(&self) {
        // Unoptimized collections
        let mut vec = Vec::new(); // Should suggest with_capacity
        let mut map = HashMap::new(); // Should suggest with_capacity
        
        // Suboptimal algorithms
        let mut numbers = vec![3, 1, 4, 1, 5];
        numbers.sort(); // Should suggest sort_unstable
        
        // Iterator chains
        let data = vec![1, 2, 3, 4, 5];
        let collected: Vec<i32> = data.iter().map(|x| x * 2).collect(); // Potential optimization
        
        // Frequent allocations in loop
        for _i in 0..10 {
            let _v = Vec::new(); // Allocation in loop
        }
    }
    
    pub unsafe fn dangerous_operation(&self, ptr: *mut i32) {
        if !ptr.is_null() {
            *ptr = 42; // Raw pointer dereference
        }
    }
    
    pub async fn async_with_blocking(&self) {
        // Blocking in async context
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let future = std::future::ready(42);
        // Missing .await
        let _result = future;
    }
    
    pub fn concurrency_issues(&self) {
        // Multiple mutex locks - deadlock potential
        let m1 = Arc::new(Mutex::new(0));
        let m2 = Arc::new(Mutex::new(0));
        
        let _g1 = m1.lock().unwrap();
        let _g2 = m2.lock().unwrap();
        
        // Non-thread-safe types
        use std::rc::Rc;
        use std::cell::RefCell;
        let rc_data = Rc::new(RefCell::new(42));
    }
}

impl Clone for DataProcessor {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            counters: self.counters.clone(),
        }
    }
}

fn takes_vec_when_slice_would_work(data: Vec<String>) -> usize {
    data.len() // Could use &[String] instead
}

pub fn multiple_mutex_locks() {
    let m1 = Arc::new(Mutex::new(0));
    let m2 = Arc::new(Mutex::new(0));
    
    let _g1 = m1.lock().unwrap();
    let _g2 = m2.lock().unwrap(); // Potential deadlock
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Verify we get comprehensive analysis results
    assert!(
        !analysis.ownership_patterns.is_empty(),
        "Should detect ownership patterns"
    );
    assert!(
        !analysis.performance_issues.is_empty(),
        "Should detect performance issues"
    );
    assert!(
        !analysis.safety_issues.is_empty(),
        "Should detect safety issues"
    );
    assert!(
        !analysis.concurrency_issues.is_empty(),
        "Should detect concurrency issues"
    );

    // Verify trait implementations are analyzed
    assert!(
        !analysis.trait_implementations.is_empty(),
        "Should detect trait implementations"
    );

    // Check specific pattern types exist
    let ownership_types: std::collections::HashSet<_> = analysis
        .ownership_patterns
        .iter()
        .map(|p| std::mem::discriminant(&p.pattern_type))
        .collect();
    assert!(
        ownership_types.len() > 1,
        "Should detect multiple ownership pattern types"
    );

    let performance_types: std::collections::HashSet<_> = analysis
        .performance_issues
        .iter()
        .map(|p| std::mem::discriminant(&p.issue_type))
        .collect();
    assert!(
        performance_types.len() > 1,
        "Should detect multiple performance issue types"
    );
}

#[tokio::test]
async fn test_severity_and_impact_levels() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("severity_test.rs"),
        old_tree: None,
        content: r#"
fn multiple_mutable_refs(a: &mut Vec<i32>, b: &mut HashMap<String, i32>, c: &mut String) {
    // High severity - multiple mutable borrows
    a.push(1);
    b.insert("key".to_string(), 1);
    c.push_str("test");
}

unsafe fn critical_unsafe_operation() {
    let value = 42u32;
    let bytes: [u8; 4] = std::mem::transmute(value); // Critical risk
}

fn medium_performance_issue() -> String {
    let mut result = String::new();
    result.push_str("a");
    result.push_str("b");
    result.push_str("c");
    result.push_str("d"); // Medium impact string concatenation
    result.push_str("e"); // More string operations to trigger medium impact
    result
}

fn unoptimized_collections() {
    let mut map = HashMap::new(); // Should be medium impact
    map.insert("key", 1);
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();
    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    // Check that severity levels are assigned appropriately
    let high_severity_ownership = analysis
        .ownership_patterns
        .iter()
        .any(|p| matches!(p.severity, Severity::High));
    assert!(
        high_severity_ownership,
        "Should have high severity ownership issues"
    );

    let medium_impact_performance = analysis
        .performance_issues
        .iter()
        .any(|p| matches!(p.impact, PerformanceImpact::Medium));
    assert!(
        medium_impact_performance,
        "Should have medium impact performance issues"
    );

    let critical_risk_safety = analysis
        .safety_issues
        .iter()
        .any(|s| matches!(s.risk_level, RiskLevel::Critical));
    assert!(
        critical_risk_safety,
        "Should have critical risk safety issues"
    );
}

#[tokio::test]
async fn debug_analysis_output() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("debug.rs"),
        old_tree: None,
        content: r#"
use std::ffi::{CString, CStr};

unsafe fn unsafe_function() -> *mut i32 {
    std::ptr::null_mut()
}

extern "C" fn ffi_function(ptr: *const i8) -> i32 {
    unsafe {
        if ptr.is_null() {
            return -1;
        }
        *ptr as i32
    }
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();

    println!("=== NODES ===");
    for node in &result.nodes {
        println!(
            "Node: {:?} - {} (signature: {:?})",
            node.kind, node.name, node.signature
        );
    }

    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    println!("=== DEBUG ANALYSIS OUTPUT ===");
    println!("Ownership patterns: {}", analysis.ownership_patterns.len());
    for pattern in &analysis.ownership_patterns {
        println!("  - {:?}: {}", pattern.pattern_type, pattern.description);
    }

    println!("Performance issues: {}", analysis.performance_issues.len());
    for issue in &analysis.performance_issues {
        println!("  - {:?}: {}", issue.issue_type, issue.description);
    }

    println!("Safety issues: {}", analysis.safety_issues.len());
    for issue in &analysis.safety_issues {
        println!("  - {:?}: {}", issue.issue_type, issue.description);
    }

    println!("Concurrency issues: {}", analysis.concurrency_issues.len());
    for issue in &analysis.concurrency_issues {
        println!("  - {:?}: {}", issue.issue_type, issue.description);
    }
}

#[tokio::test]
async fn debug_concurrency_analysis() {
    let mut parser = RustParser::new();
    let context = ParseContext {
        repo_id: "test_repo".to_string(),
        file_path: PathBuf::from("concurrency_debug.rs"),
        old_tree: None,
        content: r#"
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

fn non_thread_safe_types() {
    let rc_data = Rc::new(RefCell::new(42));
}

fn multiple_locks() {
    let mutex1 = Arc::new(Mutex::new(0));
    let mutex2 = Arc::new(Mutex::new(0));
    
    let _guard1 = mutex1.lock().unwrap();
    let _guard2 = mutex2.lock().unwrap();
}
"#
        .to_string(),
    };

    let result = parser.parse(&context).unwrap();

    println!("=== CONCURRENCY DEBUG NODES ===");
    for node in &result.nodes {
        println!(
            "Node: {:?} - {} (signature: {:?})",
            node.kind, node.name, node.signature
        );
    }

    let analyzer = RustAnalyzer::new(result.nodes, result.edges);
    let analysis = analyzer.analyze_all();

    println!("=== CONCURRENCY ANALYSIS ===");
    println!("Concurrency issues: {}", analysis.concurrency_issues.len());
    for issue in &analysis.concurrency_issues {
        println!("  - {:?}: {}", issue.issue_type, issue.description);
    }
}
