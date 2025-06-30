//! Specialized analysis tools for inheritance and decorators

#![allow(clippy::too_many_arguments)]

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;
use anyhow::Result;
use serde_json::Value;

/// List specialized analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "trace_inheritance".to_string(),
            title: Some("Trace Inheritance Hierarchy".to_string()),
            description: "Analyze Python inheritance hierarchies including metaclasses, mixins, and method resolution order with comprehensive metaprogramming analysis".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "class_id": {
                        "type": "string",
                        "description": "Symbol ID of the class to analyze inheritance for"
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["up", "down", "both"],
                        "description": "Direction to trace inheritance (up=parents, down=children, both=complete tree)",
                        "default": "both"
                    },
                    "max_depth": {
                        "type": "number",
                        "description": "Maximum inheritance depth to traverse",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "include_metaclass_analysis": {
                        "type": "boolean",
                        "description": "Include detailed metaclass impact analysis",
                        "default": true
                    },
                    "include_mixin_analysis": {
                        "type": "boolean", 
                        "description": "Include mixin composition analysis",
                        "default": true
                    },
                    "include_mro_analysis": {
                        "type": "boolean",
                        "description": "Include Method Resolution Order analysis",
                        "default": true
                    },
                    "include_dynamic_attributes": {
                        "type": "boolean",
                        "description": "Include analysis of dynamically added attributes",
                        "default": true
                    },
                    "detect_diamond_inheritance": {
                        "type": "boolean",
                        "description": "Detect and analyze diamond inheritance patterns",
                        "default": true
                    },
                    "include_source_context": {
                        "type": "boolean",
                        "description": "Include source code context for inheritance relationships",
                        "default": false
                    }
                },
                "required": ["class_id"]
            }),
        },
        Tool {
            name: "analyze_decorators".to_string(),
            title: Some("Analyze Decorators".to_string()),
            description: "Comprehensive decorator analysis including patterns, effects, framework integration, and metaprogramming impacts".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "decorator_id": {
                        "type": "string",
                        "description": "Symbol ID of decorator to analyze (optional - analyzes all if not provided)"
                    },
                    "scope": {
                        "type": "string",
                        "enum": ["global", "file", "class", "specific"],
                        "description": "Scope of decorator analysis",
                        "default": "global"
                    },
                    "include_usage_analysis": {
                        "type": "boolean",
                        "description": "Include comprehensive usage pattern analysis",
                        "default": true
                    },
                    "include_effect_analysis": {
                        "type": "boolean",
                        "description": "Include analysis of decorator effects on target functions/classes",
                        "default": true
                    },
                    "include_factory_analysis": {
                        "type": "boolean",
                        "description": "Include decorator factory pattern analysis",
                        "default": true
                    },
                    "include_chain_analysis": {
                        "type": "boolean",
                        "description": "Include decorator chain composition analysis",
                        "default": true
                    },
                    "framework_detection": {
                        "type": "boolean",
                        "description": "Detect and analyze framework-specific decorators (Flask, Django, FastAPI, etc.)",
                        "default": true
                    },
                    "pattern_detection": {
                        "type": "boolean",
                        "description": "Detect common decorator patterns (caching, validation, authorization, etc.)",
                        "default": true
                    },
                    "confidence_threshold": {
                        "type": "number",
                        "description": "Confidence threshold for pattern detection (0.0 to 1.0)",
                        "default": 0.8,
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "include_recommendations": {
                        "type": "boolean",
                        "description": "Include best practice recommendations",
                        "default": true
                    }
                },
                "required": []
            }),
        }
    ]
}

/// Route specialized analysis tool calls
pub async fn call_tool(
    server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    match params.name.as_str() {
        "trace_inheritance" => trace_inheritance(server, params.arguments.as_ref()).await,
        "analyze_decorators" => analyze_decorators(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!(
            "Unknown specialized analysis tool: {}",
            params.name
        )),
    }
}

/// Trace inheritance hierarchy (full implementation)
async fn trace_inheritance(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Get target class - either by name or ID
    let target_classes = if let Some(class_name) = args.get("class_name").and_then(|v| v.as_str()) {
        // Search for classes by name
        let symbol_types = Some(vec![codeprism_core::NodeKind::Class]);
        let limit = Some(10);
        let search_results =
            server
                .graph_query()
                .search_symbols(class_name, symbol_types, limit)?;

        if search_results.is_empty() {
            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("No classes found matching pattern: {}", class_name),
                }],
                is_error: Some(true),
            });
        }

        // Convert SymbolInfo to classes
        search_results
            .into_iter()
            .filter_map(|symbol| server.graph_store().get_node(&symbol.node.id))
            .filter(|node| matches!(node.kind, codeprism_core::NodeKind::Class))
            .collect::<Vec<_>>()
    } else if let Some(class_id_str) = args.get("class_id").and_then(|v| v.as_str()) {
        // Use specific class ID
        let class_id = parse_node_id(class_id_str)?;
        if let Some(node) = server.graph_store().get_node(&class_id) {
            if matches!(node.kind, codeprism_core::NodeKind::Class) {
                vec![node]
            } else {
                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Node {} is not a class", class_id_str),
                    }],
                    is_error: Some(true),
                });
            }
        } else {
            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Class not found: {}", class_id_str),
                }],
                is_error: Some(true),
            });
        }
    } else {
        return Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: "Either class_name or class_id parameter is required".to_string(),
            }],
            is_error: Some(true),
        });
    };

    // Parse options
    let direction = args
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("both");

    let include_metaclasses = args
        .get("include_metaclasses")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_mixins = args
        .get("include_mixins")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_mro = args
        .get("include_mro")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_dynamic_attributes = args
        .get("include_dynamic_attributes")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(10);

    let include_source_context = args
        .get("include_source_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Analyze each target class
    let mut analysis_results = Vec::new();

    for target_class in &target_classes {
        let inheritance_info = server
            .graph_query()
            .get_inheritance_info(&target_class.id)?;

        // Build inheritance tree visualization
        let inheritance_tree = build_inheritance_tree(
            server,
            &target_class.id,
            direction,
            max_depth,
            include_source_context,
        )
        .await?;

        // Metaclass analysis
        let metaclass_analysis = if include_metaclasses && inheritance_info.metaclass.is_some() {
            Some(analyze_metaclass_impact(server, &inheritance_info).await?)
        } else {
            None
        };

        // Mixin analysis
        let mixin_analysis = if include_mixins && !inheritance_info.mixins.is_empty() {
            Some(analyze_mixin_relationships(server, &inheritance_info).await?)
        } else {
            None
        };

        // Method Resolution Order
        let mro_analysis = if include_mro && !inheritance_info.method_resolution_order.is_empty() {
            Some(analyze_method_resolution_order(server, &inheritance_info).await?)
        } else {
            None
        };

        // Dynamic attributes analysis
        let dynamic_attributes_analysis =
            if include_dynamic_attributes && !inheritance_info.dynamic_attributes.is_empty() {
                Some(analyze_dynamic_attributes(server, &inheritance_info).await?)
            } else {
                None
            };

        // Diamond inheritance detection
        let diamond_inheritance = detect_diamond_inheritance(server, &target_class.id).await?;

        let mut analysis = serde_json::json!({
            "target_class": {
                "id": target_class.id.to_hex(),
                "name": target_class.name,
                "file": target_class.file.display().to_string(),
                "span": {
                    "start_line": target_class.span.start_line,
                    "end_line": target_class.span.end_line,
                    "start_column": target_class.span.start_column,
                    "end_column": target_class.span.end_column
                }
            },
            "inheritance_tree": inheritance_tree,
            "diamond_inheritance": diamond_inheritance,
            "basic_inheritance_info": {
                "is_metaclass": inheritance_info.is_metaclass,
                "base_classes_count": inheritance_info.base_classes.len(),
                "subclasses_count": inheritance_info.subclasses.len(),
                "inheritance_depth": inheritance_info.inheritance_chain.len() - 1
            }
        });

        // Add optional analyses
        if let Some(metaclass) = metaclass_analysis {
            analysis["metaclass_analysis"] = metaclass;
        }

        if let Some(mixins) = mixin_analysis {
            analysis["mixin_analysis"] = mixins;
        }

        if let Some(mro) = mro_analysis {
            analysis["method_resolution_order"] = mro;
        }

        if let Some(dynamic_attrs) = dynamic_attributes_analysis {
            analysis["dynamic_attributes_analysis"] = dynamic_attrs;
        }

        analysis_results.push(analysis);
    }

    let result = serde_json::json!({
        "analysis_results": analysis_results,
        "summary": {
            "classes_analyzed": target_classes.len(),
            "direction": direction,
            "max_depth": max_depth,
            "options": {
                "include_metaclasses": include_metaclasses,
                "include_mixins": include_mixins,
                "include_mro": include_mro,
                "include_dynamic_attributes": include_dynamic_attributes,
                "include_source_context": include_source_context
            }
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Analyze decorators (full implementation)
async fn analyze_decorators(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Get target decorators - either by pattern or ID
    let target_decorators = if let Some(decorator_pattern) =
        args.get("decorator_pattern").and_then(|v| v.as_str())
    {
        // Search for decorators by pattern
        let symbol_types = Some(vec![
            codeprism_core::NodeKind::Function,
            codeprism_core::NodeKind::Call,
        ]);
        let limit = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(100);

        let search_results =
            server
                .graph_query()
                .search_symbols(decorator_pattern, symbol_types, Some(limit))?;

        if search_results.is_empty() {
            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!(
                        "No decorators found matching pattern: {}",
                        decorator_pattern
                    ),
                }],
                is_error: Some(true),
            });
        }

        // Filter for decorator-like symbols
        search_results
            .into_iter()
            .filter_map(|symbol| server.graph_store().get_node(&symbol.node.id))
            .filter(is_decorator_node)
            .collect::<Vec<_>>()
    } else if let Some(decorator_id_str) = args.get("decorator_id").and_then(|v| v.as_str()) {
        // Use specific decorator ID
        let decorator_id = parse_node_id(decorator_id_str)?;
        if let Some(node) = server.graph_store().get_node(&decorator_id) {
            if is_decorator_node(&node) {
                vec![node]
            } else {
                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Node {} is not a decorator", decorator_id_str),
                    }],
                    is_error: Some(true),
                });
            }
        } else {
            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Decorator not found: {}", decorator_id_str),
                }],
                is_error: Some(true),
            });
        }
    } else {
        return Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: "Either decorator_pattern or decorator_id parameter is required".to_string(),
            }],
            is_error: Some(true),
        });
    };

    // Parse options
    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    let include_factories = args
        .get("include_factories")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyze_effects = args
        .get("analyze_effects")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_chains = args
        .get("include_chains")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let detect_patterns = args
        .get("detect_patterns")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_framework_analysis = args
        .get("include_framework_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_source_context = args
        .get("include_source_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let confidence_threshold = args
        .get("confidence_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    // Analyze each target decorator
    let mut analysis_results = Vec::new();

    for target_decorator in &target_decorators {
        // Basic decorator analysis
        let decorator_usage = analyze_decorator_usage(server, &target_decorator.id, scope).await?;

        // Decorator effects analysis
        let effects_analysis = if analyze_effects {
            Some(analyze_decorator_effects(server, &target_decorator.id).await?)
        } else {
            None
        };

        // Decorator factory analysis
        let factory_analysis = if include_factories {
            Some(analyze_decorator_factory(server, &target_decorator.id).await?)
        } else {
            None
        };

        // Decorator chain analysis
        let chain_analysis = if include_chains {
            Some(analyze_decorator_chains(server, &target_decorator.id).await?)
        } else {
            None
        };

        // Framework-specific analysis
        let framework_analysis = if include_framework_analysis {
            Some(analyze_framework_decorators(server, &target_decorator.id).await?)
        } else {
            None
        };

        // Pattern detection
        let pattern_analysis = if detect_patterns {
            Some(
                detect_decorator_patterns(server, &target_decorator.id, confidence_threshold)
                    .await?,
            )
        } else {
            None
        };

        let mut analysis = serde_json::json!({
            "target_decorator": {
                "id": target_decorator.id.to_hex(),
                "name": target_decorator.name,
                "file": target_decorator.file.display().to_string(),
                "span": {
                    "start_line": target_decorator.span.start_line,
                    "end_line": target_decorator.span.end_line,
                    "start_column": target_decorator.span.start_column,
                    "end_column": target_decorator.span.end_column
                }
            },
            "usage_analysis": decorator_usage
        });

        // Add source context if requested
        if include_source_context {
            if let Some(context) =
                extract_source_context(&target_decorator.file, target_decorator.span.start_line, 3)
            {
                analysis["source_context"] = context;
            }
        }

        // Add optional analyses
        if let Some(effects) = effects_analysis {
            analysis["effects_analysis"] = effects;
        }

        if let Some(factory) = factory_analysis {
            analysis["factory_analysis"] = factory;
        }

        if let Some(chains) = chain_analysis {
            analysis["chain_analysis"] = chains;
        }

        if let Some(framework) = framework_analysis {
            analysis["framework_analysis"] = framework;
        }

        if let Some(patterns) = pattern_analysis {
            analysis["pattern_analysis"] = patterns;
        }

        analysis_results.push(analysis);
    }

    let result = serde_json::json!({
        "analysis_results": analysis_results,
        "summary": {
            "decorators_analyzed": target_decorators.len(),
            "scope": scope,
            "options": {
                "include_factories": include_factories,
                "analyze_effects": analyze_effects,
                "include_chains": include_chains,
                "detect_patterns": detect_patterns,
                "include_framework_analysis": include_framework_analysis,
                "include_source_context": include_source_context,
                "confidence_threshold": confidence_threshold
            }
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

// Helper function to parse node IDs from hex string
fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|e| anyhow::anyhow!("Invalid node ID '{}': {}", hex_str, e))
}

/// Build complete inheritance tree visualization
async fn build_inheritance_tree(
    server: &CodePrismMcpServer,
    class_id: &codeprism_core::NodeId,
    direction: &str,
    max_depth: usize,
    include_source_context: bool,
) -> Result<serde_json::Value> {
    let mut tree = serde_json::Map::new();
    let mut visited = std::collections::HashSet::new();

    // Build tree recursively
    build_tree_recursive(
        server,
        class_id,
        &mut tree,
        &mut visited,
        direction,
        0,
        max_depth,
        include_source_context,
    )
    .await?;

    Ok(serde_json::Value::Object(tree))
}

/// Recursive helper for building inheritance tree
fn build_tree_recursive<'a>(
    server: &'a CodePrismMcpServer,
    class_id: &'a codeprism_core::NodeId,
    tree: &'a mut serde_json::Map<String, serde_json::Value>,
    visited: &'a mut std::collections::HashSet<codeprism_core::NodeId>,
    direction: &'a str,
    current_depth: usize,
    max_depth: usize,
    include_source_context: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        if current_depth >= max_depth || visited.contains(class_id) {
            return Ok(());
        }

        visited.insert(*class_id);

        if let Some(class_node) = server.graph_store().get_node(class_id) {
            if let Ok(inheritance_info) = server.graph_query().get_inheritance_info(class_id) {
                let mut class_data = serde_json::Map::new();

                // Basic class information
                class_data.insert(
                    "id".to_string(),
                    serde_json::Value::String(class_id.to_hex()),
                );
                class_data.insert(
                    "name".to_string(),
                    serde_json::Value::String(class_node.name.clone()),
                );
                class_data.insert(
                    "file".to_string(),
                    serde_json::Value::String(class_node.file.display().to_string()),
                );
                class_data.insert(
                    "is_metaclass".to_string(),
                    serde_json::Value::Bool(inheritance_info.is_metaclass),
                );

                // Add source context if requested
                if include_source_context {
                    if let Some(context) =
                        extract_source_context(&class_node.file, class_node.span.start_line, 3)
                    {
                        class_data.insert("source_context".to_string(), context);
                    }
                }

                // Metaclass information
                if let Some(metaclass) = &inheritance_info.metaclass {
                    class_data.insert(
                        "metaclass".to_string(),
                        serde_json::json!({
                            "name": metaclass.class_name,
                            "file": metaclass.file.display().to_string()
                        }),
                    );
                }

                // Process parent classes (up direction)
                if direction == "up" || direction == "both" {
                    let mut parents = serde_json::Map::new();
                    for base_class in &inheritance_info.base_classes {
                        // Try to find the actual base class node
                        let base_classes = server
                            .graph_store()
                            .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                        if let Some(base_node) = base_classes
                            .iter()
                            .find(|node| node.name == base_class.class_name)
                        {
                            build_tree_recursive(
                                server,
                                &base_node.id,
                                &mut parents,
                                visited,
                                direction,
                                current_depth + 1,
                                max_depth,
                                include_source_context,
                            )
                            .await?;
                        } else {
                            // External class (not in our codebase)
                            parents.insert(
                                base_class.class_name.clone(),
                                serde_json::json!({
                                    "name": base_class.class_name,
                                    "external": true,
                                    "relationship_type": base_class.relationship_type
                                }),
                            );
                        }
                    }
                    if !parents.is_empty() {
                        class_data.insert(
                            "parent_classes".to_string(),
                            serde_json::Value::Object(parents),
                        );
                    }
                }

                // Process child classes (down direction)
                if direction == "down" || direction == "both" {
                    let mut children = serde_json::Map::new();
                    for subclass in &inheritance_info.subclasses {
                        // Try to find the actual subclass node
                        let subclasses = server
                            .graph_store()
                            .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                        if let Some(sub_node) = subclasses
                            .iter()
                            .find(|node| node.name == subclass.class_name)
                        {
                            build_tree_recursive(
                                server,
                                &sub_node.id,
                                &mut children,
                                visited,
                                direction,
                                current_depth + 1,
                                max_depth,
                                include_source_context,
                            )
                            .await?;
                        }
                    }
                    if !children.is_empty() {
                        class_data.insert(
                            "child_classes".to_string(),
                            serde_json::Value::Object(children),
                        );
                    }
                }

                // Add mixins if any
                if !inheritance_info.mixins.is_empty() {
                    let mixins: Vec<_> = inheritance_info
                        .mixins
                        .iter()
                        .map(|mixin| {
                            serde_json::json!({
                                "name": mixin.class_name,
                                "file": mixin.file.display().to_string()
                            })
                        })
                        .collect();
                    class_data.insert("mixins".to_string(), serde_json::Value::Array(mixins));
                }

                tree.insert(
                    class_node.name.clone(),
                    serde_json::Value::Object(class_data),
                );
            }
        }

        Ok(())
    })
}

fn extract_source_context(
    file_path: &std::path::Path,
    line_number: usize,
    context_lines: usize,
) -> Option<serde_json::Value> {
    std::fs::read_to_string(file_path).ok().and_then(|content| {
        let lines: Vec<&str> = content.lines().collect();

        if line_number == 0 || line_number > lines.len() {
            return None;
        }

        let start_line = line_number.saturating_sub(context_lines).max(1);
        let end_line = (line_number + context_lines).min(lines.len());

        let context_lines_vec: Vec<serde_json::Value> = (start_line..=end_line)
            .map(|i| {
                serde_json::json!({
                    "line_number": i,
                    "content": lines.get(i.saturating_sub(1)).unwrap_or(&""),
                    "is_target": i == line_number
                })
            })
            .collect();

        Some(serde_json::json!({
            "file": file_path.display().to_string(),
            "target_line": line_number,
            "context_lines": context_lines_vec
        }))
    })
}

/// Analyze metaclass impact on inheritance hierarchy
async fn analyze_metaclass_impact(
    server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    if let Some(metaclass) = &inheritance_info.metaclass {
        // Find all classes affected by this metaclass
        let all_classes = server
            .graph_store()
            .get_nodes_by_kind(codeprism_core::NodeKind::Class);
        let mut affected_classes = Vec::new();

        for class in all_classes {
            if let Ok(class_inheritance) = server.graph_query().get_inheritance_info(&class.id) {
                if let Some(class_metaclass) = &class_inheritance.metaclass {
                    if class_metaclass.class_name == metaclass.class_name {
                        affected_classes.push(serde_json::json!({
                            "name": class.name,
                            "file": class.file.display().to_string(),
                            "dynamic_attributes": class_inheritance.dynamic_attributes
                        }));
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "metaclass": {
                "name": metaclass.class_name,
                "file": metaclass.file.display().to_string()
            },
            "affected_classes_count": affected_classes.len(),
            "affected_classes": affected_classes,
            "creates_dynamic_attributes": !inheritance_info.dynamic_attributes.is_empty(),
            "dynamic_attributes": inheritance_info.dynamic_attributes,
            "behavior_modifications": [
                "class_creation",
                "attribute_access",
                "method_registration"
            ]
        }))
    } else {
        Ok(serde_json::json!(null))
    }
}

/// Analyze mixin relationships and their effects
async fn analyze_mixin_relationships(
    server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    let mut mixin_analysis = Vec::new();

    for mixin in &inheritance_info.mixins {
        // Find the mixin class and analyze its methods
        let mixin_classes = server
            .graph_store()
            .get_nodes_by_kind(codeprism_core::NodeKind::Class);
        if let Some(mixin_node) = mixin_classes
            .iter()
            .find(|node| node.name == mixin.class_name)
        {
            let mixin_methods = server
                .graph_store()
                .get_outgoing_edges(&mixin_node.id)
                .iter()
                .filter_map(|edge| server.graph_store().get_node(&edge.target))
                .filter(|node| matches!(node.kind, codeprism_core::NodeKind::Method))
                .map(|method| {
                    serde_json::json!({
                        "name": method.name,
                        "file": method.file.display().to_string()
                    })
                })
                .collect::<Vec<_>>();

            mixin_analysis.push(serde_json::json!({
                "name": mixin.class_name,
                "file": mixin.file.display().to_string(),
                "methods_provided": mixin_methods,
                "method_count": mixin_methods.len(),
                "mixin_type": if mixin.class_name.ends_with("Mixin") { "explicit" } else { "implicit" }
            }));
        }
    }

    Ok(serde_json::json!({
        "mixins": mixin_analysis,
        "total_mixins": mixin_analysis.len(),
        "mixin_pattern_usage": "multiple_inheritance"
    }))
}

/// Analyze Method Resolution Order in detail
async fn analyze_method_resolution_order(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    let mro = &inheritance_info.method_resolution_order;

    let mut mro_analysis = Vec::new();
    for (index, class_name) in mro.iter().enumerate() {
        mro_analysis.push(serde_json::json!({
            "order": index,
            "class_name": class_name,
            "is_root": class_name == "object",
            "is_base": index == mro.len() - 1 || class_name == "object",
            "is_target": index == 0
        }));
    }

    Ok(serde_json::json!({
        "method_resolution_order": mro_analysis,
        "mro_length": mro.len(),
        "linearization": mro,
        "complexity": if mro.len() > 5 { "complex" } else if mro.len() > 3 { "moderate" } else { "simple" },
        "has_diamond_pattern": mro.len() > 4 && mro.iter().any(|name| name.contains("Mixin"))
    }))
}

/// Analyze dynamic attributes created by metaclasses
async fn analyze_dynamic_attributes(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    let dynamic_attrs = &inheritance_info.dynamic_attributes;

    let mut attribute_analysis = Vec::new();
    let mut creation_sources = std::collections::HashMap::new();

    for attr in dynamic_attrs {
        attribute_analysis.push(serde_json::json!({
            "name": attr.name,
            "created_by": attr.created_by,
            "type": attr.attribute_type,
            "creation_source": if attr.created_by.starts_with("metaclass:") { "metaclass" } else { "decorator" }
        }));

        let source = if attr.created_by.starts_with("metaclass:") {
            "metaclass"
        } else {
            "decorator"
        };
        *creation_sources.entry(source).or_insert(0) += 1;
    }

    Ok(serde_json::json!({
        "dynamic_attributes": attribute_analysis,
        "total_dynamic_attributes": dynamic_attrs.len(),
        "creation_sources": creation_sources,
        "attribute_types": dynamic_attrs.iter().map(|attr| &attr.attribute_type).collect::<std::collections::HashSet<_>>(),
        "patterns": {
            "registry_pattern": dynamic_attrs.iter().any(|attr| attr.name.contains("registry") || attr.name.contains("_processors")),
            "injection_pattern": dynamic_attrs.iter().any(|attr| attr.created_by.starts_with("metaclass:")),
            "decorator_pattern": dynamic_attrs.iter().any(|attr| attr.created_by.starts_with("decorator:"))
        }
    }))
}

/// Detect diamond inheritance patterns
async fn detect_diamond_inheritance(
    server: &CodePrismMcpServer,
    class_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    let inheritance_info = server.graph_query().get_inheritance_info(class_id)?;

    // Diamond inheritance occurs when a class inherits from multiple classes
    // that share a common ancestor
    let mut diamond_patterns = Vec::new();

    if inheritance_info.base_classes.len() > 1 {
        // Check for shared ancestors among base classes
        for i in 0..inheritance_info.base_classes.len() {
            for j in i + 1..inheritance_info.base_classes.len() {
                let base_class_1 = &inheritance_info.base_classes[i];
                let base_class_2 = &inheritance_info.base_classes[j];

                // Try to find common ancestors
                let all_classes = server
                    .graph_store()
                    .get_nodes_by_kind(codeprism_core::NodeKind::Class);
                if let Some(base_node_1) = all_classes
                    .iter()
                    .find(|node| node.name == base_class_1.class_name)
                {
                    if let Some(base_node_2) = all_classes
                        .iter()
                        .find(|node| node.name == base_class_2.class_name)
                    {
                        if let Ok(inheritance_1) =
                            server.graph_query().get_inheritance_info(&base_node_1.id)
                        {
                            if let Ok(inheritance_2) =
                                server.graph_query().get_inheritance_info(&base_node_2.id)
                            {
                                // Look for common ancestors
                                let ancestors_1: std::collections::HashSet<_> =
                                    inheritance_1.inheritance_chain.iter().collect();
                                let ancestors_2: std::collections::HashSet<_> =
                                    inheritance_2.inheritance_chain.iter().collect();

                                let common_ancestors: Vec<_> =
                                    ancestors_1.intersection(&ancestors_2).collect();

                                if !common_ancestors.is_empty() {
                                    diamond_patterns.push(serde_json::json!({
                                        "base_class_1": base_class_1.class_name,
                                        "base_class_2": base_class_2.class_name,
                                        "common_ancestors": common_ancestors,
                                        "mro_impact": inheritance_info.method_resolution_order.len() > 4
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({
        "has_diamond_inheritance": !diamond_patterns.is_empty(),
        "diamond_patterns": diamond_patterns,
        "complexity_impact": match diamond_patterns.len() {
            0 => "none",
            1 => "low",
            2..=3 => "moderate",
            _ => "high"
        },
        "mro_complexity": inheritance_info.method_resolution_order.len()
    }))
}

/// Check if a node represents a decorator
fn is_decorator_node(node: &codeprism_core::Node) -> bool {
    // Check if it's a function that could be a decorator
    if matches!(node.kind, codeprism_core::NodeKind::Function) {
        // Common decorator naming patterns
        if node.name.starts_with("_") && node.name.len() > 1 {
            return false; // Likely private function
        }

        // Check for common decorator patterns
        let decorator_indicators = [
            "decorator",
            "wrap",
            "cache",
            "validate",
            "auth",
            "property",
            "classmethod",
            "staticmethod",
            "lru_cache",
            "route",
            "app",
            "requires",
            "check",
            "log",
            "retry",
            "timeout",
            "rate_limit",
        ];

        return decorator_indicators
            .iter()
            .any(|&indicator| node.name.to_lowercase().contains(indicator));
    }

    // Check if it's a call that could be a decorator usage
    if matches!(node.kind, codeprism_core::NodeKind::Call) {
        // Look for @decorator syntax patterns
        return node.name.starts_with("@")
            || node.name.contains("decorator")
            || node.name.contains("property")
            || node.name.contains("classmethod")
            || node.name.contains("staticmethod");
    }

    false
}

/// Analyze decorator usage patterns
async fn analyze_decorator_usage(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
    scope: &str,
) -> Result<serde_json::Value> {
    // Find all references to this decorator
    let references = server.graph_query().find_references(decorator_id)?;

    let mut usage_locations = Vec::new();
    let mut decorated_functions = Vec::new();
    let mut decorated_classes = Vec::new();
    let mut usage_files = std::collections::HashSet::new();

    for reference in &references {
        usage_files.insert(reference.location.file.clone());

        usage_locations.push(serde_json::json!({
            "file": reference.location.file.display().to_string(),
            "line": reference.location.span.start_line,
            "target_name": reference.source_node.name,
            "target_type": format!("{:?}", reference.source_node.kind)
        }));

        // Categorize what's being decorated
        match reference.source_node.kind {
            codeprism_core::NodeKind::Function | codeprism_core::NodeKind::Method => {
                decorated_functions.push(serde_json::json!({
                    "name": reference.source_node.name,
                    "file": reference.source_node.file.display().to_string(),
                    "type": format!("{:?}", reference.source_node.kind)
                }));
            }
            codeprism_core::NodeKind::Class => {
                decorated_classes.push(serde_json::json!({
                    "name": reference.source_node.name,
                    "file": reference.source_node.file.display().to_string()
                }));
            }
            _ => {}
        }
    }

    Ok(serde_json::json!({
        "usage_count": references.len(),
        "files_count": usage_files.len(),
        "decorated_functions": decorated_functions,
        "decorated_classes": decorated_classes,
        "usage_locations": usage_locations,
        "scope_coverage": match scope {
            "repository" => "full_repository",
            "module" => "single_module",
            "function" => "function_level",
            "class" => "class_level",
            _ => "unknown"
        }
    }))
}

/// Analyze decorator effects on targets
async fn analyze_decorator_effects(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    let decorator_node = server
        .graph_store()
        .get_node(decorator_id)
        .ok_or_else(|| anyhow::anyhow!("Decorator node not found"))?;

    // Analyze what the decorator function does
    let mut effects = Vec::new();
    let mut modifies_signature = false;
    let adds_metadata = false;
    let mut creates_wrapper = false;
    let mut registers_function = false;

    // Look for common decorator patterns in the function body
    // This is a simplified analysis - a more comprehensive implementation could parse the AST
    let decorator_name = &decorator_node.name.to_lowercase();

    if decorator_name.contains("wrapper") || decorator_name.contains("wrap") {
        creates_wrapper = true;
        effects.push("Creates wrapper function");
    }

    if decorator_name.contains("property") {
        modifies_signature = true;
        effects.push("Converts method to property");
    }

    if decorator_name.contains("cache") || decorator_name.contains("lru") {
        effects.push("Adds caching behavior");
    }

    if decorator_name.contains("validate") {
        effects.push("Adds input validation");
    }

    if decorator_name.contains("auth") || decorator_name.contains("require") {
        effects.push("Adds authorization checks");
    }

    if decorator_name.contains("route") || decorator_name.contains("endpoint") {
        registers_function = true;
        effects.push("Registers as web endpoint");
    }

    if decorator_name.contains("log") {
        effects.push("Adds logging functionality");
    }

    if decorator_name.contains("retry") {
        effects.push("Adds retry mechanism");
    }

    if decorator_name.contains("timeout") {
        effects.push("Adds timeout handling");
    }

    if decorator_name.contains("classmethod") || decorator_name.contains("staticmethod") {
        modifies_signature = true;
        effects.push("Changes method binding");
    }

    Ok(serde_json::json!({
        "effects": effects,
        "modifies_signature": modifies_signature,
        "adds_metadata": adds_metadata,
        "creates_wrapper": creates_wrapper,
        "registers_function": registers_function,
        "effect_categories": {
            "behavioral": effects.iter().any(|e| e.contains("behavior") || e.contains("mechanism")),
            "structural": modifies_signature || creates_wrapper,
            "registration": registers_function,
            "validation": effects.iter().any(|e| e.contains("validation") || e.contains("auth")),
            "performance": effects.iter().any(|e| e.contains("cache") || e.contains("timeout"))
        }
    }))
}

/// Analyze if decorator is a factory pattern
async fn analyze_decorator_factory(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    let decorator_node = server
        .graph_store()
        .get_node(decorator_id)
        .ok_or_else(|| anyhow::anyhow!("Decorator node not found"))?;

    // Check if this function returns another function (decorator factory pattern)
    let outgoing_edges = server.graph_store().get_outgoing_edges(decorator_id);
    let has_inner_function = outgoing_edges.iter().any(|edge| {
        if let Some(target_node) = server.graph_store().get_node(&edge.target) {
            matches!(target_node.kind, codeprism_core::NodeKind::Function)
        } else {
            false
        }
    });

    let is_factory = has_inner_function
        || decorator_node.name.to_lowercase().contains("factory")
        || decorator_node.name.ends_with("_decorator")
        || decorator_node.name.starts_with("make_");

    let mut factory_parameters = Vec::new();
    if is_factory {
        // In a real implementation, you'd parse the function signature
        // Currently using naming heuristics
        if decorator_node.name.to_lowercase().contains("cache") {
            factory_parameters.push("maxsize");
        }
        if decorator_node.name.to_lowercase().contains("retry") {
            factory_parameters.push("attempts");
            factory_parameters.push("delay");
        }
        if decorator_node.name.to_lowercase().contains("timeout") {
            factory_parameters.push("seconds");
        }
    }

    Ok(serde_json::json!({
        "is_factory": is_factory,
        "has_inner_function": has_inner_function,
        "factory_parameters": factory_parameters,
        "factory_type": if is_factory {
            if decorator_node.name.to_lowercase().contains("param") { "parameterized" }
            else if has_inner_function { "closure_based" }
            else { "configuration_based" }
        } else { "simple_decorator" }
    }))
}

/// Analyze decorator chains
async fn analyze_decorator_chains(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    // Find functions/classes that use this decorator and see if they have other decorators
    let references = server.graph_query().find_references(decorator_id)?;
    let mut chain_analysis = Vec::new();

    for reference in &references {
        // Look for other decorators on the same target
        let target_dependencies = server.graph_query().find_dependencies(
            &reference.source_node.id,
            codeprism_core::graph::DependencyType::Direct,
        )?;

        let other_decorators: Vec<_> = target_dependencies
            .iter()
            .filter(|dep| is_decorator_node(&dep.target_node))
            .filter(|dep| dep.target_node.id != *decorator_id)
            .map(|dep| {
                serde_json::json!({
                    "name": dep.target_node.name,
                    "id": dep.target_node.id.to_hex(),
                    "file": dep.target_node.file.display().to_string()
                })
            })
            .collect();

        if !other_decorators.is_empty() {
            chain_analysis.push(serde_json::json!({
                "target": {
                    "name": reference.source_node.name,
                    "type": format!("{:?}", reference.source_node.kind),
                    "file": reference.source_node.file.display().to_string()
                },
                "decorators_in_chain": other_decorators,
                "chain_length": other_decorators.len() + 1
            }));
        }
    }

    Ok(serde_json::json!({
        "chains_found": chain_analysis.len(),
        "decorator_chains": chain_analysis,
        "has_complex_chains": chain_analysis.iter().any(|chain|
            chain["chain_length"].as_u64().unwrap_or(0) > 2
        )
    }))
}

/// Analyze framework-specific decorator patterns
async fn analyze_framework_decorators(
    _server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    let decorator_node = _server
        .graph_store()
        .get_node(decorator_id)
        .ok_or_else(|| anyhow::anyhow!("Decorator node not found"))?;

    let decorator_name = &decorator_node.name.to_lowercase();
    let mut framework_info = serde_json::Map::new();

    // Flask framework patterns
    if decorator_name.contains("route") || decorator_name.contains("app.") {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("Flask".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("routing".to_string()),
        );
        framework_info.insert(
            "creates_endpoint".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // Django framework patterns
    else if decorator_name.contains("csrf")
        || decorator_name.contains("login_required")
        || decorator_name.contains("permission_required")
    {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("Django".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("security".to_string()),
        );
        framework_info.insert(
            "creates_middleware".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // FastAPI framework patterns
    else if decorator_name.contains("get")
        || decorator_name.contains("post")
        || decorator_name.contains("put")
        || decorator_name.contains("delete")
    {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("FastAPI".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("routing".to_string()),
        );
        framework_info.insert(
            "creates_endpoint".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // pytest patterns
    else if decorator_name.contains("fixture")
        || decorator_name.contains("mark")
        || decorator_name.contains("parametrize")
    {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("pytest".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("testing".to_string()),
        );
        framework_info.insert(
            "test_infrastructure".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // SQLAlchemy patterns
    else if decorator_name.contains("validates")
        || decorator_name.contains("hybrid")
        || decorator_name.contains("relationship")
    {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("SQLAlchemy".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("orm".to_string()),
        );
        framework_info.insert(
            "database_integration".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // Celery patterns
    else if decorator_name.contains("task")
        || decorator_name.contains("periodic")
        || decorator_name.contains("crontab")
    {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("Celery".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("task_queue".to_string()),
        );
        framework_info.insert(
            "async_processing".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    // Generic patterns
    else {
        framework_info.insert(
            "framework".to_string(),
            serde_json::Value::String("generic".to_string()),
        );
        framework_info.insert(
            "pattern_type".to_string(),
            serde_json::Value::String("custom".to_string()),
        );
    }

    Ok(serde_json::Value::Object(framework_info))
}

/// Detect common decorator patterns
async fn detect_decorator_patterns(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
    confidence_threshold: f64,
) -> Result<serde_json::Value> {
    let decorator_node = server
        .graph_store()
        .get_node(decorator_id)
        .ok_or_else(|| anyhow::anyhow!("Decorator node not found"))?;

    let decorator_name = &decorator_node.name.to_lowercase();
    let mut detected_patterns = Vec::new();

    // Caching pattern
    if decorator_name.contains("cache")
        || decorator_name.contains("lru")
        || decorator_name.contains("memoize")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "caching",
            "confidence": 0.95,
            "description": "Caches function results to improve performance",
            "common_use_cases": ["expensive computations", "database queries", "API calls"],
            "performance_impact": "positive"
        }));
    }

    // Validation pattern
    if decorator_name.contains("validate")
        || decorator_name.contains("check")
        || decorator_name.contains("verify")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "validation",
            "confidence": 0.90,
            "description": "Validates input parameters or preconditions",
            "common_use_cases": ["input sanitization", "type checking", "business rules"],
            "security_impact": "positive"
        }));
    }

    // Authorization pattern
    if decorator_name.contains("auth")
        || decorator_name.contains("require")
        || decorator_name.contains("permission")
        || decorator_name.contains("login")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "authorization",
            "confidence": 0.92,
            "description": "Enforces access control and permissions",
            "common_use_cases": ["user authentication", "role-based access", "API security"],
            "security_impact": "critical"
        }));
    }

    // Logging pattern
    if decorator_name.contains("log")
        || decorator_name.contains("trace")
        || decorator_name.contains("audit")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "logging",
            "confidence": 0.88,
            "description": "Adds logging or auditing functionality",
            "common_use_cases": ["debugging", "monitoring", "compliance"],
            "observability_impact": "positive"
        }));
    }

    // Retry pattern
    if decorator_name.contains("retry")
        || decorator_name.contains("attempt")
        || decorator_name.contains("backoff")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "retry",
            "confidence": 0.91,
            "description": "Implements retry logic for failed operations",
            "common_use_cases": ["network calls", "external services", "transient failures"],
            "reliability_impact": "positive"
        }));
    }

    // Rate limiting pattern
    if decorator_name.contains("rate")
        || decorator_name.contains("limit")
        || decorator_name.contains("throttle")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "rate_limiting",
            "confidence": 0.89,
            "description": "Controls the rate of function execution",
            "common_use_cases": ["API rate limiting", "resource protection", "load management"],
            "performance_impact": "protective"
        }));
    }

    // Timeout pattern
    if decorator_name.contains("timeout") || decorator_name.contains("deadline") {
        detected_patterns.push(serde_json::json!({
            "pattern": "timeout",
            "confidence": 0.87,
            "description": "Enforces execution time limits",
            "common_use_cases": ["long-running operations", "external service calls", "resource management"],
            "reliability_impact": "protective"
        }));
    }

    // Property pattern
    if decorator_name.contains("property")
        || decorator_name.contains("getter")
        || decorator_name.contains("setter")
    {
        detected_patterns.push(serde_json::json!({
            "pattern": "property_accessor",
            "confidence": 0.95,
            "description": "Converts methods to property-like accessors",
            "common_use_cases": ["data encapsulation", "computed properties", "lazy evaluation"],
            "design_impact": "encapsulation"
        }));
    }

    // Filter patterns by confidence threshold
    let filtered_patterns: Vec<_> = detected_patterns
        .into_iter()
        .filter(|pattern| pattern["confidence"].as_f64().unwrap_or(0.0) >= confidence_threshold)
        .collect();

    Ok(serde_json::json!({
        "detected_patterns": filtered_patterns,
        "pattern_count": filtered_patterns.len(),
        "confidence_threshold": confidence_threshold,
        "recommendations": get_decorator_recommendations(&filtered_patterns)
    }))
}

/// Get recommendations based on detected patterns
fn get_decorator_recommendations(patterns: &[serde_json::Value]) -> Vec<String> {
    let mut recommendations = Vec::new();

    for pattern in patterns {
        if let Some(pattern_name) = pattern["pattern"].as_str() {
            match pattern_name {
                "caching" => {
                    recommendations.push("Consider cache invalidation strategy".to_string());
                    recommendations.push("Monitor cache hit rates".to_string());
                }
                "validation" => {
                    recommendations.push("Ensure comprehensive error handling".to_string());
                    recommendations.push("Document validation rules".to_string());
                }
                "authorization" => {
                    recommendations.push("Regular security audits recommended".to_string());
                    recommendations.push("Implement principle of least privilege".to_string());
                }
                "logging" => {
                    recommendations.push("Avoid logging sensitive information".to_string());
                    recommendations.push("Configure appropriate log levels".to_string());
                }
                "retry" => {
                    recommendations.push("Use exponential backoff".to_string());
                    recommendations.push("Set maximum retry limits".to_string());
                }
                "rate_limiting" => {
                    recommendations.push("Monitor rate limit effectiveness".to_string());
                    recommendations.push("Provide clear error messages".to_string());
                }
                "timeout" => {
                    recommendations.push("Set reasonable timeout values".to_string());
                    recommendations.push("Handle timeout exceptions gracefully".to_string());
                }
                _ => {}
            }
        }
    }

    if recommendations.is_empty() {
        recommendations.push("Consider adding documentation for custom decorator".to_string());
    }

    recommendations
}
