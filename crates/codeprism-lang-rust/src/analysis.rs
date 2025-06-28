//! Rust-specific analysis capabilities

use crate::types::{Edge, Node, NodeKind, Span};
use std::collections::HashMap;

/// Analyzer for Rust-specific language features
pub struct RustAnalyzer {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    /// Map from function/scope ID to nodes within that scope
    scope_map: HashMap<crate::types::NodeId, Vec<crate::types::NodeId>>,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut analyzer = Self {
            nodes,
            edges,
            scope_map: HashMap::new(),
        };
        analyzer.build_scope_map();
        analyzer
    }

    /// Build a map of scopes to their contained nodes
    fn build_scope_map(&mut self) {
        for edge in &self.edges {
            if matches!(edge.kind, crate::types::EdgeKind::Contains) {
                self.scope_map
                    .entry(edge.source)
                    .or_default()
                    .push(edge.target);
            }
        }
    }

    /// Comprehensive analysis of all Rust-specific patterns
    pub fn analyze_all(&self) -> RustAnalysisResult {
        RustAnalysisResult {
            ownership_patterns: self.analyze_ownership_patterns(),
            performance_issues: self.analyze_performance_patterns(),
            safety_issues: self.analyze_safety_patterns(),
            concurrency_issues: self.analyze_concurrency_patterns(),
            trait_implementations: self.analyze_trait_implementations(),
            unsafe_usage: self.analyze_unsafe_usage(),
            lifetime_usage: self.analyze_lifetime_usage(),
            macro_usage: self.analyze_macro_usage(),
        }
    }

    /// Analyze ownership patterns in the code
    pub fn analyze_ownership_patterns(&self) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        for node in &self.nodes {
            match &node.kind {
                NodeKind::Function | NodeKind::Method => {
                    patterns.extend(self.detect_ownership_antipatterns(node));
                    patterns.extend(self.detect_unnecessary_clones(node));
                    patterns.extend(self.detect_inefficient_borrowing(node));
                    patterns.extend(self.detect_move_semantics_issues(node));
                }
                NodeKind::Variable => {
                    patterns.extend(self.analyze_variable_ownership(node));
                }
                _ => {}
            }
        }

        patterns
    }

    /// Analyze performance-related patterns
    pub fn analyze_performance_patterns(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        for node in &self.nodes {
            match &node.kind {
                NodeKind::Function | NodeKind::Method => {
                    issues.extend(self.detect_allocation_patterns(node));
                    issues.extend(self.detect_string_inefficiencies(node));
                    issues.extend(self.detect_iterator_inefficiencies(node));
                    issues.extend(self.detect_collection_inefficiencies(node));
                }
                NodeKind::Call => {
                    issues.extend(self.analyze_call_performance(node));
                }
                _ => {}
            }
        }

        issues
    }

    /// Analyze safety-related patterns
    pub fn analyze_safety_patterns(&self) -> Vec<SafetyIssue> {
        let mut issues = Vec::new();

        for node in &self.nodes {
            match &node.kind {
                NodeKind::Function | NodeKind::Method => {
                    if self.is_unsafe_function(node) {
                        issues.extend(self.analyze_unsafe_function(node));
                    }
                    issues.extend(self.detect_ffi_patterns(node));
                    issues.extend(self.detect_memory_safety_issues(node));
                }
                _ => {}
            }
        }

        issues
    }

    /// Analyze concurrency-related patterns
    pub fn analyze_concurrency_patterns(&self) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        for node in &self.nodes {
            match &node.kind {
                NodeKind::Function | NodeKind::Method => {
                    issues.extend(self.analyze_async_patterns(node));
                    issues.extend(self.detect_deadlock_potential(node));
                    issues.extend(self.analyze_thread_safety(node));
                    issues.extend(self.analyze_channel_usage(node));
                }
                NodeKind::Struct | NodeKind::Enum => {
                    issues.extend(self.analyze_send_sync_traits(node));
                }
                _ => {}
            }
        }

        issues
    }

    // Ownership Pattern Detection Methods

    /// Detect ownership antipatterns in a function
    fn detect_ownership_antipatterns(&self, function_node: &Node) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // Check function signature for ownership issues
        if let Some(signature) = &function_node.signature {
            // Detect unnecessary owned parameters
            if signature.contains("String") && !signature.contains("&str") {
                patterns.push(OwnershipPattern {
                    pattern_type: OwnershipPatternType::UnnecessaryOwned,
                    location: function_node.span.clone(),
                    description: "Function takes owned String when &str would suffice".to_string(),
                    suggestion: Some(
                        "Consider using &str parameter for read-only string access".to_string(),
                    ),
                    severity: Severity::Medium,
                });
            }

            // Detect multiple mutable borrows pattern
            let mut_borrow_count = signature.matches("&mut").count();
            if mut_borrow_count > 1 {
                patterns.push(OwnershipPattern {
                    pattern_type: OwnershipPatternType::MultipleMutableBorrows,
                    location: function_node.span.clone(),
                    description: format!(
                        "Function has {} mutable borrows, potential for borrowing conflicts",
                        mut_borrow_count
                    ),
                    suggestion: Some(
                        "Consider refactoring to reduce mutable borrows or use interior mutability"
                            .to_string(),
                    ),
                    severity: Severity::High,
                });
            }
        }

        patterns
    }

    /// Detect unnecessary clone operations
    fn detect_unnecessary_clones(&self, function_node: &Node) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // Look for calls within this function that might be unnecessary clones
        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) && call_node.name.contains("clone")
                    {
                        patterns.push(OwnershipPattern {
                            pattern_type: OwnershipPatternType::UnnecessaryClone,
                            location: call_node.span.clone(),
                            description: "Potential unnecessary clone() call".to_string(),
                            suggestion: Some(
                                "Check if borrowing would work instead of cloning".to_string(),
                            ),
                            severity: Severity::Medium,
                        });
                    }
                }
            }
        }

        patterns
    }

    /// Detect inefficient borrowing patterns
    fn detect_inefficient_borrowing(&self, function_node: &Node) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // Analyze function metadata for borrowing patterns
        if let Some(metadata) = function_node.metadata.as_object() {
            if let Some(params) = metadata.get("parameters").and_then(|p| p.as_array()) {
                for param in params {
                    if let Some(param_obj) = param.as_object() {
                        if let Some(ownership) = param_obj.get("ownership").and_then(|o| o.as_str())
                        {
                            if ownership == "borrowed" {
                                if let Some(usage) =
                                    param_obj.get("usage_pattern").and_then(|u| u.as_str())
                                {
                                    if usage == "stored" {
                                        patterns.push(OwnershipPattern {
                                            pattern_type: OwnershipPatternType::InefficientBorrowing,
                                            location: function_node.span.clone(),
                                            description: "Borrowed parameter is stored, consider taking ownership".to_string(),
                                            suggestion: Some("Take ownership of the parameter if it needs to be stored".to_string()),
                                            severity: Severity::Medium,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        patterns
    }

    /// Detect move semantics issues
    fn detect_move_semantics_issues(&self, function_node: &Node) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // Check for potential move after use patterns
        if let Some(signature) = &function_node.signature {
            if signature.contains("self")
                && !signature.contains("&self")
                && !signature.contains("&mut self")
            {
                patterns.push(OwnershipPattern {
                    pattern_type: OwnershipPatternType::PotentialMoveError,
                    location: function_node.span.clone(),
                    description: "Method takes self by value, object will be moved".to_string(),
                    suggestion: Some(
                        "Consider if borrowing (&self or &mut self) would be more appropriate"
                            .to_string(),
                    ),
                    severity: Severity::Low,
                });
            }
        }

        patterns
    }

    /// Analyze variable ownership patterns
    fn analyze_variable_ownership(&self, variable_node: &Node) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // Check if variable name suggests it should be a reference
        if variable_node.name.ends_with("_ref") || variable_node.name.starts_with("ref_") {
            patterns.push(OwnershipPattern {
                pattern_type: OwnershipPatternType::InconsistentNaming,
                location: variable_node.span.clone(),
                description: "Variable name suggests reference but type might be owned".to_string(),
                suggestion: Some(
                    "Ensure naming convention matches ownership semantics".to_string(),
                ),
                severity: Severity::Low,
            });
        }

        patterns
    }

    // Performance Analysis Methods

    /// Detect allocation patterns that might be inefficient
    fn detect_allocation_patterns(&self, function_node: &Node) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Detect Vec::new() in loops (should check actual loop context)
                        if call_node.name.contains("Vec::new") || call_node.name.contains("vec!") {
                            issues.push(PerformanceIssue {
                                issue_type: PerformanceIssueType::FrequentAllocations,
                                location: call_node.span.clone(),
                                description: "Vector allocation detected - ensure not in hot path"
                                    .to_string(),
                                suggestion: Some(
                                    "Consider pre-allocating with capacity or reusing allocations"
                                        .to_string(),
                                ),
                                impact: PerformanceImpact::Medium,
                            });
                        }

                        // Detect HashMap::new() without capacity
                        if call_node.name.contains("HashMap::new") {
                            issues.push(PerformanceIssue {
                                issue_type: PerformanceIssueType::UnoptimizedCollections,
                                location: call_node.span.clone(),
                                description: "HashMap created without initial capacity".to_string(),
                                suggestion: Some(
                                    "Use HashMap::with_capacity() if size is known".to_string(),
                                ),
                                impact: PerformanceImpact::Medium,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect string manipulation inefficiencies
    fn detect_string_inefficiencies(&self, function_node: &Node) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            let mut string_pushes = 0;

            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Count string push operations
                        if call_node.name.contains("push_str") || call_node.name.contains("push") {
                            string_pushes += 1;
                        }

                        // Detect string concatenation in loops
                        if call_node.name.contains("+") && string_pushes > 2 {
                            issues.push(PerformanceIssue {
                                issue_type: PerformanceIssueType::StringConcatenation,
                                location: call_node.span.clone(),
                                description: "Multiple string operations detected".to_string(),
                                suggestion: Some(
                                    "Consider using String::with_capacity() or format! macro"
                                        .to_string(),
                                ),
                                impact: PerformanceImpact::Medium,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect iterator chain inefficiencies
    fn detect_iterator_inefficiencies(&self, function_node: &Node) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Detect collect() followed by into_iter()
                        if call_node.name.contains("collect") {
                            issues.push(PerformanceIssue {
                                issue_type: PerformanceIssueType::IteratorChains,
                                location: call_node.span.clone(),
                                description:
                                    "collect() call may be unnecessary if chaining iterators"
                                        .to_string(),
                                suggestion: Some(
                                    "Consider avoiding intermediate collection if possible"
                                        .to_string(),
                                ),
                                impact: PerformanceImpact::Medium,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect collection usage inefficiencies
    fn detect_collection_inefficiencies(&self, function_node: &Node) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        if let Some(signature) = &function_node.signature {
            // Detect Vec<T> parameters when slice would work
            if signature.contains("Vec<") && !signature.contains("&mut") {
                issues.push(PerformanceIssue {
                    issue_type: PerformanceIssueType::UnoptimizedCollections,
                    location: function_node.span.clone(),
                    description: "Function takes Vec<T> when &[T] slice might suffice".to_string(),
                    suggestion: Some(
                        "Consider using slice (&[T]) for read-only access".to_string(),
                    ),
                    impact: PerformanceImpact::Low,
                });
            }
        }

        issues
    }

    /// Analyze performance implications of specific calls
    fn analyze_call_performance(&self, call_node: &Node) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // Detect expensive operations
        if call_node.name.contains("sort") && !call_node.name.contains("sort_unstable") {
            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::SuboptimalAlgorithms,
                location: call_node.span.clone(),
                description: "Using stable sort when unstable sort might be sufficient".to_string(),
                suggestion: Some(
                    "Consider sort_unstable() for better performance if stability not required"
                        .to_string(),
                ),
                impact: PerformanceImpact::Low,
            });
        }

        issues
    }

    // Safety Analysis Methods

    /// Check if a function is marked as unsafe
    fn is_unsafe_function(&self, function_node: &Node) -> bool {
        // Check function signature for unsafe keyword
        if let Some(signature) = &function_node.signature {
            if signature.contains("unsafe") {
                return true;
            }
        }

        // Check function name for unsafe
        if function_node.name.contains("unsafe") {
            return true;
        }

        // Check if it's a method with unsafe in metadata
        if matches!(function_node.kind, NodeKind::Method) {
            if let Some(metadata) = function_node.metadata.as_object() {
                if let Some(unsafe_flag) = metadata.get("unsafe").and_then(|u| u.as_bool()) {
                    if unsafe_flag {
                        return true;
                    }
                }
                if let Some(modifiers) = metadata.get("modifiers").and_then(|m| m.as_array()) {
                    for modifier in modifiers {
                        if let Some(mod_str) = modifier.as_str() {
                            if mod_str == "unsafe" {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Analyze unsafe function usage
    fn analyze_unsafe_function(&self, function_node: &Node) -> Vec<SafetyIssue> {
        let mut issues = Vec::new();

        issues.push(SafetyIssue {
            issue_type: SafetyIssueType::UnsafeFunction,
            location: function_node.span.clone(),
            description: "Unsafe function requires careful review".to_string(),
            rationale: None,
            mitigation: Some(
                "Document safety invariants and ensure all callers uphold them".to_string(),
            ),
            risk_level: RiskLevel::High,
        });

        // Check for common unsafe patterns in function scope
        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Raw pointer dereference
                        if call_node.name.contains("*") && call_node.name.contains("raw") {
                            issues.push(SafetyIssue {
                                issue_type: SafetyIssueType::RawPointerDereference,
                                location: call_node.span.clone(),
                                description: "Raw pointer dereference detected".to_string(),
                                rationale: None,
                                mitigation: Some(
                                    "Ensure pointer is valid and properly aligned".to_string(),
                                ),
                                risk_level: RiskLevel::Critical,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect FFI-related safety patterns
    fn detect_ffi_patterns(&self, function_node: &Node) -> Vec<SafetyIssue> {
        let mut issues = Vec::new();

        // Check signature for FFI patterns
        if let Some(signature) = &function_node.signature {
            // Detect extern "C" functions
            if signature.contains("extern") && signature.contains("\"C\"") {
                issues.push(SafetyIssue {
                    issue_type: SafetyIssueType::FFIBoundary,
                    location: function_node.span.clone(),
                    description: "FFI function boundary requires careful handling".to_string(),
                    rationale: None,
                    mitigation: Some(
                        "Validate all parameters and handle C-style errors properly".to_string(),
                    ),
                    risk_level: RiskLevel::High,
                });
            }

            // Detect CString usage (common in FFI)
            if signature.contains("CString") || signature.contains("CStr") {
                issues.push(SafetyIssue {
                    issue_type: SafetyIssueType::FFIBoundary,
                    location: function_node.span.clone(),
                    description: "C string handling requires null termination validation"
                        .to_string(),
                    rationale: None,
                    mitigation: Some(
                        "Ensure proper null termination and UTF-8 validation".to_string(),
                    ),
                    risk_level: RiskLevel::Medium,
                });
            }

            // Detect C-style raw pointers (common in FFI)
            if signature.contains("*const") || signature.contains("*mut") {
                // Look for typical FFI patterns
                if signature.contains("i8")
                    || signature.contains("c_char")
                    || signature.contains("c_void")
                {
                    issues.push(SafetyIssue {
                        issue_type: SafetyIssueType::FFIBoundary,
                        location: function_node.span.clone(),
                        description: "Function uses C-style raw pointers typical of FFI"
                            .to_string(),
                        rationale: None,
                        mitigation: Some(
                            "Validate pointer parameters and handle null pointers safely"
                                .to_string(),
                        ),
                        risk_level: RiskLevel::High,
                    });
                }
            }
        }

        // Check metadata for FFI information
        if let Some(metadata) = function_node.metadata.as_object() {
            if let Some(function_type) = metadata.get("function_type").and_then(|t| t.as_str()) {
                if function_type == "extern" {
                    issues.push(SafetyIssue {
                        issue_type: SafetyIssueType::FFIBoundary,
                        location: function_node.span.clone(),
                        description: "Extern function requires FFI safety considerations"
                            .to_string(),
                        rationale: None,
                        mitigation: Some(
                            "Ensure proper parameter validation and error handling".to_string(),
                        ),
                        risk_level: RiskLevel::High,
                    });
                }
            }
        }

        // Check for uses of CString/CStr in the function scope
        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(use_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(use_node.kind, NodeKind::Use)
                        && (use_node.name.contains("CString") || use_node.name.contains("CStr"))
                    {
                        issues.push(SafetyIssue {
                            issue_type: SafetyIssueType::FFIBoundary,
                            location: use_node.span.clone(),
                            description: "C string types used - typical of FFI code".to_string(),
                            rationale: None,
                            mitigation: Some(
                                "Ensure proper null termination and UTF-8 validation".to_string(),
                            ),
                            risk_level: RiskLevel::Medium,
                        });
                    }
                }
            }
        }

        issues
    }

    /// Detect memory safety issues
    fn detect_memory_safety_issues(&self, function_node: &Node) -> Vec<SafetyIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Memory allocation/deallocation
                        if call_node.name.contains("alloc") || call_node.name.contains("dealloc") {
                            issues.push(SafetyIssue {
                                issue_type: SafetyIssueType::ManualMemoryManagement,
                                location: call_node.span.clone(),
                                description: "Manual memory management detected".to_string(),
                                rationale: None,
                                mitigation: Some("Ensure matching allocation/deallocation and consider RAII patterns".to_string()),
                                risk_level: RiskLevel::High,
                            });
                        }

                        // Transmute usage
                        if call_node.name.contains("transmute") {
                            issues.push(SafetyIssue {
                                issue_type: SafetyIssueType::TypeTransmutation,
                                location: call_node.span.clone(),
                                description: "Type transmutation is extremely dangerous".to_string(),
                                rationale: None,
                                mitigation: Some("Validate size and alignment requirements, consider safer alternatives".to_string()),
                                risk_level: RiskLevel::Critical,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    // Concurrency Analysis Methods

    /// Analyze async/await usage patterns
    fn analyze_async_patterns(&self, function_node: &Node) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        if let Some(signature) = &function_node.signature {
            if signature.contains("async") {
                // Check for blocking operations in async functions
                if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
                    for &node_id in scope_nodes {
                        if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                            if matches!(call_node.kind, NodeKind::Call) {
                                // Blocking operations
                                if call_node.name.contains("block_on")
                                    || call_node.name.contains("wait")
                                {
                                    issues.push(ConcurrencyIssue {
                                        issue_type: ConcurrencyIssueType::AsyncAntipattern,
                                        location: call_node.span.clone(),
                                        description: "Blocking operation in async function".to_string(),
                                        suggestion: Some("Use async alternatives or spawn_blocking for CPU-bound work".to_string()),
                                        severity: ConcurrencySeverity::High,
                                    });
                                }

                                // Missing .await
                                if call_node.name.contains("async")
                                    && !call_node.name.contains(".await")
                                {
                                    issues.push(ConcurrencyIssue {
                                        issue_type: ConcurrencyIssueType::MissingAwait,
                                        location: call_node.span.clone(),
                                        description: "Async call without .await".to_string(),
                                        suggestion: Some(
                                            "Add .await to async function call".to_string(),
                                        ),
                                        severity: ConcurrencySeverity::High,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect potential deadlock situations
    fn detect_deadlock_potential(&self, function_node: &Node) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            let mut mutex_calls = Vec::new();

            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) && call_node.name.contains("lock") {
                        mutex_calls.push(call_node);
                    }
                }
            }

            // Multiple mutex locks in same function
            if mutex_calls.len() > 1 {
                issues.push(ConcurrencyIssue {
                    issue_type: ConcurrencyIssueType::DeadlockPotential,
                    location: function_node.span.clone(),
                    description: format!(
                        "Function acquires {} locks, potential deadlock risk",
                        mutex_calls.len()
                    ),
                    suggestion: Some(
                        "Ensure consistent lock ordering or use try_lock with timeouts".to_string(),
                    ),
                    severity: ConcurrencySeverity::High,
                });
            }
        }

        issues
    }

    /// Analyze thread safety patterns
    fn analyze_thread_safety(&self, function_node: &Node) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        // Check for non-Send/Sync types in function signature
        if let Some(signature) = &function_node.signature {
            // This is a simplified check - in practice, would need type resolution
            if signature.contains("Rc<") || signature.contains("RefCell<") {
                issues.push(ConcurrencyIssue {
                    issue_type: ConcurrencyIssueType::ThreadSafetyViolation,
                    location: function_node.span.clone(),
                    description: "Function uses non-thread-safe types (Rc, RefCell)".to_string(),
                    suggestion: Some(
                        "Consider Arc and Mutex for thread-safe alternatives".to_string(),
                    ),
                    severity: ConcurrencySeverity::Medium,
                });
            }
        }

        // Check for non-thread-safe types used within the function scope
        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Detect Rc::new or RefCell::new calls
                        if call_node.name.contains("Rc::new")
                            || call_node.name.contains("RefCell::new")
                        {
                            issues.push(ConcurrencyIssue {
                                issue_type: ConcurrencyIssueType::ThreadSafetyViolation,
                                location: call_node.span.clone(),
                                description: "Non-thread-safe type used (Rc/RefCell)".to_string(),
                                suggestion: Some(
                                    "Consider Arc and Mutex for thread-safe alternatives"
                                        .to_string(),
                                ),
                                severity: ConcurrencySeverity::Medium,
                            });
                        }
                    }
                }
            }
        }

        // Also check if function scope uses non-thread-safe types from use statements
        for use_node in &self.nodes {
            if matches!(use_node.kind, NodeKind::Use)
                && (use_node.name.contains("std::rc::Rc")
                    || use_node.name.contains("std::cell::RefCell"))
            {
                // If this use is in the same module as the function, it's a potential issue
                issues.push(ConcurrencyIssue {
                    issue_type: ConcurrencyIssueType::ThreadSafetyViolation,
                    location: function_node.span.clone(),
                    description: "Module imports non-thread-safe types (Rc, RefCell)".to_string(),
                    suggestion: Some(
                        "Consider Arc and Mutex for thread-safe alternatives".to_string(),
                    ),
                    severity: ConcurrencySeverity::Low,
                });
            }
        }

        issues
    }

    /// Analyze channel usage patterns
    fn analyze_channel_usage(&self, function_node: &Node) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        if let Some(scope_nodes) = self.scope_map.get(&function_node.id) {
            for &node_id in scope_nodes {
                if let Some(call_node) = self.nodes.iter().find(|n| n.id == node_id) {
                    if matches!(call_node.kind, NodeKind::Call) {
                        // Unbounded channel creation
                        if call_node.name.contains("unbounded") {
                            issues.push(ConcurrencyIssue {
                                issue_type: ConcurrencyIssueType::UnboundedChannel,
                                location: call_node.span.clone(),
                                description: "Unbounded channel can lead to memory growth"
                                    .to_string(),
                                suggestion: Some(
                                    "Consider bounded channels with appropriate capacity"
                                        .to_string(),
                                ),
                                severity: ConcurrencySeverity::Medium,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Analyze Send/Sync trait implementations for types
    fn analyze_send_sync_traits(&self, type_node: &Node) -> Vec<ConcurrencyIssue> {
        let mut issues = Vec::new();

        // Look for manual Send/Sync implementations
        let type_name = &type_node.name;
        for node in &self.nodes {
            if matches!(node.kind, NodeKind::Impl)
                && node.name.contains(type_name)
                && (node.name.contains("Send") || node.name.contains("Sync"))
            {
                issues.push(ConcurrencyIssue {
                    issue_type: ConcurrencyIssueType::ManualSendSync,
                    location: node.span.clone(),
                    description: "Manual Send/Sync implementation requires careful review"
                        .to_string(),
                    suggestion: Some("Ensure thread safety guarantees are maintained".to_string()),
                    severity: ConcurrencySeverity::High,
                });
            }
        }

        issues
    }

    /// Analyze trait implementations
    pub fn analyze_trait_implementations(&self) -> Vec<TraitImplementation> {
        let mut implementations = Vec::new();

        for node in &self.nodes {
            if matches!(node.kind, NodeKind::Impl) {
                // Parse impl name to extract trait and type information
                if node.name.contains(" for ") {
                    let parts: Vec<&str> = node.name.split(" for ").collect();
                    if parts.len() == 2 {
                        implementations.push(TraitImplementation {
                            trait_name: parts[0].to_string(),
                            type_name: parts[1].to_string(),
                            impl_node: node.clone(),
                        });
                    }
                }
            }
        }

        implementations
    }

    /// Analyze unsafe code usage
    pub fn analyze_unsafe_usage(&self) -> Vec<UnsafeUsage> {
        let mut unsafe_usages = Vec::new();

        for node in &self.nodes {
            if matches!(node.kind, NodeKind::Function | NodeKind::Method) {
                if let Some(signature) = &node.signature {
                    if signature.contains("unsafe") {
                        unsafe_usages.push(UnsafeUsage {
                            location: node.span.clone(),
                            usage_type: UnsafeType::Function,
                            description: format!("Unsafe function: {}", node.name),
                        });
                    }
                }
            }
        }

        unsafe_usages
    }

    /// Analyze lifetime usage
    pub fn analyze_lifetime_usage(&self) -> Vec<LifetimeUsage> {
        let mut lifetime_usages = Vec::new();

        for node in &self.nodes {
            if matches!(node.kind, NodeKind::Lifetime) {
                lifetime_usages.push(LifetimeUsage {
                    name: node.name.clone(),
                    location: node.span.clone(),
                    scope: self.find_lifetime_scope(node),
                });
            }
        }

        lifetime_usages
    }

    /// Find all macros used in the code
    pub fn analyze_macro_usage(&self) -> Vec<MacroUsage> {
        let mut macro_usages = Vec::new();

        for node in &self.nodes {
            if matches!(node.kind, NodeKind::Call) && node.name.ends_with('!') {
                macro_usages.push(MacroUsage {
                    name: node.name.clone(),
                    location: node.span.clone(),
                    macro_type: MacroType::Invocation,
                });
            }
        }

        macro_usages
    }

    /// Find the scope of a lifetime parameter
    fn find_lifetime_scope(&self, _lifetime_node: &Node) -> LifetimeScope {
        // Placeholder - would analyze where the lifetime is used
        LifetimeScope::Function
    }
}

// Result Types

/// Comprehensive Rust analysis result
#[derive(Debug, Clone)]
pub struct RustAnalysisResult {
    pub ownership_patterns: Vec<OwnershipPattern>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub safety_issues: Vec<SafetyIssue>,
    pub concurrency_issues: Vec<ConcurrencyIssue>,
    pub trait_implementations: Vec<TraitImplementation>,
    pub unsafe_usage: Vec<UnsafeUsage>,
    pub lifetime_usage: Vec<LifetimeUsage>,
    pub macro_usage: Vec<MacroUsage>,
}

/// Represents an ownership pattern found in the code
#[derive(Debug, Clone)]
pub struct OwnershipPattern {
    pub pattern_type: OwnershipPatternType,
    pub location: Span,
    pub description: String,
    pub suggestion: Option<String>,
    pub severity: Severity,
}

/// Types of ownership patterns
#[derive(Debug, Clone)]
pub enum OwnershipPatternType {
    UnnecessaryClone,
    InefficientBorrowing,
    PotentialMoveError,
    OptimalOwnership,
    UnnecessaryOwned,
    MultipleMutableBorrows,
    InconsistentNaming,
}

/// Performance issue found in the code
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub location: Span,
    pub description: String,
    pub suggestion: Option<String>,
    pub impact: PerformanceImpact,
}

/// Types of performance issues
#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    FrequentAllocations,
    StringConcatenation,
    IteratorChains,
    UnoptimizedCollections,
    SuboptimalAlgorithms,
}

/// Performance impact levels
#[derive(Debug, Clone)]
pub enum PerformanceImpact {
    Low,
    Medium,
    High,
    Critical,
}

/// Safety issue found in the code
#[derive(Debug, Clone)]
pub struct SafetyIssue {
    pub issue_type: SafetyIssueType,
    pub location: Span,
    pub description: String,
    pub rationale: Option<String>,
    pub mitigation: Option<String>,
    pub risk_level: RiskLevel,
}

/// Types of safety issues
#[derive(Debug, Clone)]
pub enum SafetyIssueType {
    UnsafeFunction,
    RawPointerDereference,
    FFIBoundary,
    ManualMemoryManagement,
    TypeTransmutation,
}

/// Risk levels for safety issues
#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Concurrency issue found in the code
#[derive(Debug, Clone)]
pub struct ConcurrencyIssue {
    pub issue_type: ConcurrencyIssueType,
    pub location: Span,
    pub description: String,
    pub suggestion: Option<String>,
    pub severity: ConcurrencySeverity,
}

/// Types of concurrency issues
#[derive(Debug, Clone)]
pub enum ConcurrencyIssueType {
    AsyncAntipattern,
    MissingAwait,
    DeadlockPotential,
    ThreadSafetyViolation,
    UnboundedChannel,
    ManualSendSync,
}

/// Severity levels for concurrency issues
#[derive(Debug, Clone)]
pub enum ConcurrencySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// General severity levels
#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Represents a trait implementation
#[derive(Debug, Clone)]
pub struct TraitImplementation {
    pub trait_name: String,
    pub type_name: String,
    pub impl_node: Node,
}

/// Represents unsafe code usage
#[derive(Debug, Clone)]
pub struct UnsafeUsage {
    pub location: Span,
    pub usage_type: UnsafeType,
    pub description: String,
}

/// Types of unsafe code usage
#[derive(Debug, Clone)]
pub enum UnsafeType {
    Function,
    Block,
    Trait,
    Impl,
}

/// Represents lifetime usage in the code
#[derive(Debug, Clone)]
pub struct LifetimeUsage {
    pub name: String,
    pub location: Span,
    pub scope: LifetimeScope,
}

/// Scope where a lifetime is used
#[derive(Debug, Clone)]
pub enum LifetimeScope {
    Function,
    Struct,
    Trait,
    Impl,
}

/// Represents macro usage
#[derive(Debug, Clone)]
pub struct MacroUsage {
    pub name: String,
    pub location: Span,
    pub macro_type: MacroType,
}

/// Types of macro usage
#[derive(Debug, Clone)]
pub enum MacroType {
    Definition,
    Invocation,
}
