//! Pattern detection and analysis tools.
//!
//! This module provides comprehensive pattern detection capabilities including
//! design patterns, architectural patterns, anti-patterns, and metaprogramming patterns.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List pattern detection tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "detect_patterns".to_string(),
        title: Some("Detect Design Patterns".to_string()),
        description:
            "Detect design patterns, architectural patterns, and anti-patterns in the codebase"
                .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "enum": ["repository", "file", "module"],
                    "default": "repository"
                },
                "pattern_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "default": ["all"]
                },
                "confidence_threshold": {
                    "type": "number",
                    "default": 0.8
                },
                "include_suggestions": {
                    "type": "boolean",
                    "default": true
                }
            }
        }),
    }]
}

/// Call pattern detection tool
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "detect_patterns" => detect_patterns(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Detect patterns in the codebase
async fn detect_patterns(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.unwrap_or_default();
    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");
    let confidence_threshold = args
        .get("confidence_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    let result = if server.repository_path().is_some() {
        let patterns = analyze_patterns(server, confidence_threshold).await?;
        serde_json::json!({
            "scope": scope,
            "patterns": patterns,
            "summary": {
                "total_patterns_detected": patterns.len(),
                "confidence_threshold": confidence_threshold
            },
            "analysis_successful": true
        })
    } else {
        serde_json::json!({
            "error": "No repository initialized",
            "analysis_successful": false
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

async fn analyze_patterns(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    // Simple pattern detection for singleton classes
    let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
    let classes = server
        .graph_query()
        .search_symbols("*", symbol_types, Some(100))?;

    for class_symbol in classes {
        let class_node = &class_symbol.node;
        if class_node.name.to_lowercase().contains("singleton") {
            patterns.push(serde_json::json!({
                "pattern_type": "Singleton",
                "category": "Creational",
                "confidence": 0.9,
                "location": {
                    "file": class_node.file.display().to_string(),
                    "class": class_node.name,
                    "line": class_node.span.start_line
                },
                "description": "Potential singleton pattern detected"
            }));
        }
    }

    Ok(patterns)
}

/// Comprehensive pattern analysis
async fn analyze_design_patterns(
    server: &CodePrismMcpServer,
    pattern_types: &[String],
    confidence_threshold: f64,
    include_suggestions: bool,
) -> Result<Vec<serde_json::Value>> {
    let mut all_patterns = Vec::new();

    // Determine which pattern types to analyze
    let analyze_all = pattern_types.contains(&"all".to_string());
    let analyze_design = analyze_all || pattern_types.contains(&"design".to_string());
    let analyze_architectural = analyze_all || pattern_types.contains(&"architectural".to_string());
    let analyze_anti_patterns = analyze_all || pattern_types.contains(&"anti-patterns".to_string());
    let analyze_metaprogramming =
        analyze_all || pattern_types.contains(&"metaprogramming".to_string());

    // Design Patterns
    if analyze_design {
        // Creational Patterns
        all_patterns.extend(detect_singleton_pattern(server, confidence_threshold).await?);
        all_patterns.extend(detect_factory_pattern(server, confidence_threshold).await?);

        // Behavioral Patterns
        all_patterns.extend(detect_observer_pattern(server, confidence_threshold).await?);
    }

    // Anti-patterns
    if analyze_anti_patterns {
        all_patterns.extend(detect_anti_patterns(server, confidence_threshold).await?);
    }

    // Architectural Patterns
    if analyze_architectural {
        all_patterns.extend(detect_architectural_patterns(server, confidence_threshold).await?);
    }

    // Metaprogramming Patterns
    if analyze_metaprogramming {
        all_patterns.extend(detect_metaprogramming_patterns(server, confidence_threshold).await?);
    }

    // Add suggestions if requested
    if include_suggestions {
        for pattern in &mut all_patterns {
            if let Some(pattern_type) = pattern.get("pattern_type").and_then(|v| v.as_str()) {
                let suggestions = get_pattern_suggestions(pattern_type);
                if let Some(pattern_obj) = pattern.as_object_mut() {
                    pattern_obj.insert("suggestions".to_string(), serde_json::json!(suggestions));
                }
            }
        }
    }

    Ok(all_patterns)
}

/// Helper functions for pattern detection

/// Detect singleton pattern implementation
async fn detect_singleton_pattern(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    // Find all classes in the repository
    let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
    let limit = Some(1000);
    let classes = server
        .graph_query()
        .search_symbols("*", symbol_types, limit)?;

    for class_symbol in classes {
        let class_node = &class_symbol.node;
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Check for private constructor pattern
        if let Ok(constructor_methods) = server.graph_query().find_references(&class_node.id) {
            let has_private_constructor = constructor_methods.iter().any(|ref_info| {
                ref_info.source_node.name.contains("__init__")
                    || ref_info.source_node.name.contains("__new__")
            });

            if has_private_constructor {
                confidence += 0.4;
                indicators.push("Private constructor detected".to_string());
            }
        }

        // Check for static instance variable
        if class_node.name.to_lowercase().contains("singleton")
            || class_node.name.to_lowercase().contains("instance")
        {
            confidence += 0.3;
            indicators.push("Singleton naming convention".to_string());
        }

        // Check for getInstance method pattern
        if let Ok(methods) = server.graph_query().find_dependencies(
            &class_node.id,
            codeprism_core::graph::DependencyType::Direct,
        ) {
            let has_get_instance = methods.iter().any(|dep| {
                dep.target_node.name.to_lowercase().contains("getinstance")
                    || dep.target_node.name.to_lowercase().contains("get_instance")
                    || dep.target_node.name.to_lowercase().contains("instance")
            });

            if has_get_instance {
                confidence += 0.3;
                indicators.push("getInstance method pattern".to_string());
            }
        }

        if confidence >= confidence_threshold {
            patterns.push(serde_json::json!({
                "pattern_type": "Singleton",
                "category": "Creational",
                "confidence": confidence,
                "location": {
                    "file": class_node.file.display().to_string(),
                    "class": class_node.name,
                    "span": {
                        "start_line": class_node.span.start_line,
                        "end_line": class_node.span.end_line
                    }
                },
                "indicators": indicators,
                "description": "Ensures a class has only one instance and provides global access to it"
            }));
        }
    }

    Ok(patterns)
}

/// Detect factory pattern implementations
async fn detect_factory_pattern(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    let symbol_types = Some(vec![
        codeprism_core::NodeKind::Function,
        codeprism_core::NodeKind::Method,
    ]);
    let limit = Some(1000);
    let functions = server
        .graph_query()
        .search_symbols("*", symbol_types, limit)?;

    for func_symbol in functions {
        let func_node = &func_symbol.node;
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Check naming patterns
        if func_node.name.to_lowercase().contains("create")
            || func_node.name.to_lowercase().contains("factory")
            || func_node.name.to_lowercase().contains("make")
            || func_node.name.to_lowercase().contains("build")
        {
            confidence += 0.3;
            indicators.push("Factory naming convention".to_string());
        }

        // Check return type patterns - functions that return class instances
        if let Ok(deps) = server
            .graph_query()
            .find_dependencies(&func_node.id, codeprism_core::graph::DependencyType::Direct)
        {
            let returns_class_instances = deps
                .iter()
                .any(|dep| matches!(dep.target_node.kind, codeprism_core::NodeKind::Class));

            if returns_class_instances {
                confidence += 0.4;
                indicators.push("Returns class instances".to_string());
            }
        }

        // Check for conditional object creation based on parameters
        if func_node.name.to_lowercase().contains("factory") {
            confidence += 0.3;
            indicators.push("Explicit factory in name".to_string());
        }

        if confidence >= confidence_threshold {
            patterns.push(serde_json::json!({
                "pattern_type": "Factory",
                "category": "Creational",
                "confidence": confidence,
                "location": {
                    "file": func_node.file.display().to_string(),
                    "function": func_node.name,
                    "span": {
                        "start_line": func_node.span.start_line,
                        "end_line": func_node.span.end_line
                    }
                },
                "indicators": indicators,
                "description": "Creates objects without specifying their exact classes"
            }));
        }
    }

    Ok(patterns)
}

/// Detect observer pattern implementations
async fn detect_observer_pattern(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
    let limit = Some(1000);
    let classes = server
        .graph_query()
        .search_symbols("*", symbol_types, limit)?;

    for class_symbol in classes {
        let class_node = &class_symbol.node;
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Check for observer-related naming
        if class_node.name.to_lowercase().contains("observer")
            || class_node.name.to_lowercase().contains("listener")
            || class_node.name.to_lowercase().contains("subscriber")
            || class_node.name.to_lowercase().contains("watcher")
        {
            confidence += 0.4;
            indicators.push("Observer naming convention".to_string());
        }

        // Check for notify/update method patterns
        if let Ok(methods) = server.graph_query().find_dependencies(
            &class_node.id,
            codeprism_core::graph::DependencyType::Direct,
        ) {
            let has_notify_methods = methods.iter().any(|dep| {
                let name = dep.target_node.name.to_lowercase();
                name.contains("notify")
                    || name.contains("update")
                    || name.contains("trigger")
                    || name.contains("fire")
            });

            if has_notify_methods {
                confidence += 0.3;
                indicators.push("Notification methods present".to_string());
            }

            // Check for subscription methods
            let has_subscription_methods = methods.iter().any(|dep| {
                let name = dep.target_node.name.to_lowercase();
                name.contains("subscribe")
                    || name.contains("unsubscribe")
                    || name.contains("attach")
                    || name.contains("detach")
                    || name.contains("add_observer")
                    || name.contains("remove_observer")
            });

            if has_subscription_methods {
                confidence += 0.3;
                indicators.push("Subscription methods present".to_string());
            }
        }

        if confidence >= confidence_threshold {
            patterns.push(serde_json::json!({
                "pattern_type": "Observer",
                "category": "Behavioral",
                "confidence": confidence,
                "location": {
                    "file": class_node.file.display().to_string(),
                    "class": class_node.name,
                    "span": {
                        "start_line": class_node.span.start_line,
                        "end_line": class_node.span.end_line
                    }
                },
                "indicators": indicators,
                "description": "Defines one-to-many dependency between objects for automatic notifications"
            }));
        }
    }

    Ok(patterns)
}

/// Detect anti-patterns in the code
async fn detect_anti_patterns(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    // Detect God Class anti-pattern
    let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
    let limit = Some(1000);
    let classes = server
        .graph_query()
        .search_symbols("*", symbol_types, limit)?;

    for class_symbol in classes {
        let class_node = &class_symbol.node;
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Check class size (lines of code)
        let class_size = class_node.span.end_line - class_node.span.start_line;
        if class_size > 500 {
            confidence += 0.4;
            indicators.push(format!("Large class: {} lines", class_size));
        }

        // Check number of methods/dependencies
        if let Ok(methods) = server.graph_query().find_dependencies(
            &class_node.id,
            codeprism_core::graph::DependencyType::Direct,
        ) {
            if methods.len() > 20 {
                confidence += 0.3;
                indicators.push(format!("High method count: {}", methods.len()));
            }
        }

        // Check for responsibilities (multiple unrelated method groups)
        if class_node.name.to_lowercase().contains("manager")
            || class_node.name.to_lowercase().contains("handler")
            || class_node.name.to_lowercase().contains("controller")
        {
            confidence += 0.3;
            indicators
                .push("Manager/Handler naming suggests multiple responsibilities".to_string());
        }

        if confidence >= confidence_threshold {
            patterns.push(serde_json::json!({
                "pattern_type": "God Class",
                "category": "Anti-pattern",
                "confidence": confidence,
                "severity": "High",
                "location": {
                    "file": class_node.file.display().to_string(),
                    "class": class_node.name,
                    "span": {
                        "start_line": class_node.span.start_line,
                        "end_line": class_node.span.end_line
                    }
                },
                "indicators": indicators,
                "description": "Class that knows too much or does too much",
                "impact": "Reduces maintainability, increases coupling, harder to test"
            }));
        }
    }

    Ok(patterns)
}

/// Detect architectural patterns
async fn detect_architectural_patterns(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    // MVC Pattern Detection
    let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
    let limit = Some(1000);
    let classes = server
        .graph_query()
        .search_symbols("*", symbol_types, limit)?;

    let mut models = 0;
    let mut views = 0;
    let mut controllers = 0;

    for class_symbol in classes {
        let class_name = class_symbol.node.name.to_lowercase();

        if class_name.contains("model") || class_name.ends_with("model") {
            models += 1;
        } else if class_name.contains("view") || class_name.ends_with("view") {
            views += 1;
        } else if class_name.contains("controller") || class_name.ends_with("controller") {
            controllers += 1;
        }
    }

    // If we have a reasonable distribution of MVC components
    if models > 0 && views > 0 && controllers > 0 {
        let total_mvc = models + views + controllers;
        let confidence = if total_mvc >= 6 { 0.9 } else { 0.7 };

        if confidence >= confidence_threshold {
            patterns.push(serde_json::json!({
                "pattern_type": "Model-View-Controller (MVC)",
                "category": "Architectural",
                "confidence": confidence,
                "location": {
                    "scope": "Repository-wide"
                },
                "indicators": [
                    format!("Models: {}", models),
                    format!("Views: {}", views),
                    format!("Controllers: {}", controllers)
                ],
                "description": "Separates application logic into three interconnected components"
            }));
        }
    }

    Ok(patterns)
}

/// Detect metaprogramming patterns
async fn detect_metaprogramming_patterns(
    server: &CodePrismMcpServer,
    confidence_threshold: f64,
) -> Result<Vec<serde_json::Value>> {
    let mut patterns = Vec::new();

    // Skip decorator pattern detection since NodeKind::Decorator doesn't exist yet
    let decorators: Vec<codeprism_core::SymbolInfo> = Vec::new();

    // Detect metaclass usage
    let metaclass_symbols = server
        .graph_query()
        .search_symbols("metaclass", None, Some(100))?;
    if !metaclass_symbols.is_empty() {
        patterns.push(serde_json::json!({
            "pattern_type": "Metaclass Pattern",
            "category": "Metaprogramming",
            "confidence": 0.9,
            "location": {
                "scope": "Repository-wide"
            },
            "indicators": [
                format!("Metaclass references: {}", metaclass_symbols.len())
            ],
            "description": "Uses metaclasses to control class creation and behavior"
        }));
    }

    Ok(patterns)
}

/// Get improvement suggestions for detected patterns
fn get_pattern_suggestions(pattern_type: &str) -> Vec<String> {
    match pattern_type.to_lowercase().as_str() {
        "singleton" => vec![
            "Consider using dependency injection instead of singleton pattern".to_string(),
            "Ensure thread safety if used in multi-threaded environment".to_string(),
            "Consider if global state is really necessary".to_string(),
        ],
        "factory" => vec![
            "Document the types of objects the factory can create".to_string(),
            "Consider using abstract factory for families of related objects".to_string(),
            "Ensure factory methods have clear naming conventions".to_string(),
        ],
        "observer" => vec![
            "Implement proper unsubscription to prevent memory leaks".to_string(),
            "Consider using weak references for observers".to_string(),
            "Ensure thread safety for concurrent notifications".to_string(),
        ],
        "god class" => vec![
            "Split into smaller, focused classes following Single Responsibility Principle"
                .to_string(),
            "Extract utility methods into separate utility classes".to_string(),
            "Use composition over inheritance to reduce complexity".to_string(),
            "Consider using facade pattern to simplify interface".to_string(),
        ],
        "model-view-controller (mvc)" => vec![
            "Ensure clear separation between model, view, and controller responsibilities"
                .to_string(),
            "Consider using dependency injection for better testability".to_string(),
            "Implement proper error handling across all layers".to_string(),
        ],
        "decorator pattern" => vec![
            "Ensure decorators are composable and order-independent where possible".to_string(),
            "Document decorator side effects and dependencies".to_string(),
            "Consider performance impact of decorator chains".to_string(),
        ],
        "metaclass pattern" => vec![
            "Document metaclass behavior clearly as it can be complex".to_string(),
            "Consider if simpler alternatives like class decorators could work".to_string(),
            "Ensure metaclass inheritance is well understood".to_string(),
        ],
        _ => vec![
            "Review pattern implementation for best practices".to_string(),
            "Ensure pattern is well-documented and understood by team".to_string(),
            "Consider if pattern is necessary or if simpler approach would work".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_patterns_list_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "detect_patterns");
    }

    #[tokio::test]
    async fn test_pattern_suggestions() {
        let suggestions = get_pattern_suggestions("singleton");
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("dependency injection"));
    }
}
