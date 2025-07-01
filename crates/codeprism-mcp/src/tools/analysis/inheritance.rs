//! Inheritance analysis tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List inheritance analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "trace_inheritance".to_string(),
        title: Some("Trace Inheritance Hierarchy".to_string()),
        description:
            "Trace complete inheritance hierarchy with metaclasses, mixins, and MRO analysis"
                .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "class_name": {
                    "type": "string",
                    "description": "Name of the class to analyze"
                },
                "class_id": {
                    "type": "string",
                    "description": "ID of the class to analyze"
                },
                "direction": {
                    "type": "string",
                    "enum": ["up", "down", "both"],
                    "default": "both",
                    "description": "Direction to trace inheritance"
                },
                "include_metaclasses": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include metaclass analysis"
                },
                "include_mixins": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include mixin analysis"
                },
                "include_mro": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include method resolution order"
                },
                "include_dynamic_attributes": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include dynamic attribute analysis"
                },
                "max_depth": {
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum depth for inheritance traversal"
                },
                "include_source_context": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include source code context"
                }
            }
        }),
    }]
}

/// Call inheritance analysis tool
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "trace_inheritance" => trace_inheritance(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Trace complete inheritance hierarchy for a class
async fn trace_inheritance(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
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
            .get_inheritance_info(&target_class.id)
            .unwrap_or_else(|_| create_empty_inheritance_info());

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
                "inheritance_depth": inheritance_info.inheritance_chain.len().saturating_sub(1)
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
#[async_recursion::async_recursion]
async fn build_tree_recursive(
    server: &CodePrismMcpServer,
    class_id: &codeprism_core::NodeId,
    tree: &mut serde_json::Map<String, serde_json::Value>,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    direction: &str,
    current_depth: usize,
    max_depth: usize,
    include_source_context: bool,
) -> Result<()> {
    // Prevent infinite recursion and excessive depth
    if current_depth >= max_depth || visited.contains(class_id) {
        return Ok(());
    }

    visited.insert(*class_id);

    if let Some(class_node) = server.graph_store().get_node(class_id) {
        let inheritance_info = server
            .graph_query()
            .get_inheritance_info(class_id)
            .unwrap_or_else(|_| create_empty_inheritance_info());

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
            "line".to_string(),
            serde_json::Value::Number(serde_json::Number::from(class_node.span.start_line)),
        );

        if include_source_context {
            // Add source context if requested
            if let Some(context) =
                extract_source_context(&class_node.file, class_node.span.start_line, 3)
            {
                class_data.insert("source_context".to_string(), context);
            }
        }

        // Handle different directions
        match direction {
            "up" | "both" => {
                // Add base classes
                if !inheritance_info.base_classes.is_empty() {
                    let mut base_classes = Vec::new();
                    for base_class_rel in &inheritance_info.base_classes {
                        if let Some(base_node) =
                            server.graph_store().get_node(&base_class_rel.node_id)
                        {
                            base_classes.push(serde_json::json!({
                                "id": base_class_rel.node_id.to_hex(),
                                "name": base_node.name
                            }));

                            // Recurse up the hierarchy
                            build_tree_recursive(
                                server,
                                &base_class_rel.node_id,
                                tree,
                                visited,
                                direction,
                                current_depth + 1,
                                max_depth,
                                include_source_context,
                            )
                            .await?;
                        }
                    }
                    class_data.insert(
                        "base_classes".to_string(),
                        serde_json::Value::Array(base_classes),
                    );
                }
            }
            _ => {}
        }

        match direction {
            "down" | "both" => {
                // Add subclasses
                if !inheritance_info.subclasses.is_empty() {
                    let mut subclasses = Vec::new();
                    for subclass_rel in &inheritance_info.subclasses {
                        if let Some(sub_node) = server.graph_store().get_node(&subclass_rel.node_id)
                        {
                            subclasses.push(serde_json::json!({
                                "id": subclass_rel.node_id.to_hex(),
                                "name": sub_node.name
                            }));

                            // Recurse down the hierarchy
                            build_tree_recursive(
                                server,
                                &subclass_rel.node_id,
                                tree,
                                visited,
                                direction,
                                current_depth + 1,
                                max_depth,
                                include_source_context,
                            )
                            .await?;
                        }
                    }
                    class_data.insert(
                        "subclasses".to_string(),
                        serde_json::Value::Array(subclasses),
                    );
                }
            }
            _ => {}
        }

        tree.insert(class_id.to_hex(), serde_json::Value::Object(class_data));
    }

    Ok(())
}

// Simplified analysis functions (these would be more complex in real implementation)
async fn analyze_metaclass_impact(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "has_metaclass": inheritance_info.metaclass.is_some(),
        "metaclass_id": inheritance_info.metaclass.as_ref().map(|rel| rel.node_id.to_hex()),
        "is_metaclass": inheritance_info.is_metaclass,
        "impact": "Custom metaclass detected - may affect class creation and behavior"
    }))
}

async fn analyze_mixin_relationships(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "mixin_count": inheritance_info.mixins.len(),
        "mixins": inheritance_info.mixins.iter().map(|rel| rel.node_id.to_hex()).collect::<Vec<_>>(),
        "analysis": "Multiple inheritance through mixins detected"
    }))
}

async fn analyze_method_resolution_order(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "mro_length": inheritance_info.method_resolution_order.len(),
        "mro": inheritance_info.method_resolution_order.clone(),
        "complexity": if inheritance_info.method_resolution_order.len() > 5 { "High" } else { "Normal" }
    }))
}

async fn analyze_dynamic_attributes(
    _server: &CodePrismMcpServer,
    inheritance_info: &codeprism_core::InheritanceInfo,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "dynamic_attributes_count": inheritance_info.dynamic_attributes.len(),
        "attributes": inheritance_info.dynamic_attributes,
        "risk": if inheritance_info.dynamic_attributes.len() > 3 { "High" } else { "Low" }
    }))
}

async fn detect_diamond_inheritance(
    server: &CodePrismMcpServer,
    class_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    // Simple diamond inheritance detection
    let inheritance_info = server
        .graph_query()
        .get_inheritance_info(class_id)
        .unwrap_or_else(|_| create_empty_inheritance_info());

    let has_diamond = inheritance_info.base_classes.len() > 1;

    Ok(serde_json::json!({
        "has_diamond_inheritance": has_diamond,
        "base_classes_count": inheritance_info.base_classes.len(),
        "risk_level": if has_diamond { "Medium" } else { "None" },
        "explanation": if has_diamond {
            "Multiple inheritance detected - potential for diamond inheritance pattern"
        } else {
            "No diamond inheritance detected"
        }
    }))
}

// Helper functions
fn create_empty_inheritance_info() -> codeprism_core::InheritanceInfo {
    codeprism_core::InheritanceInfo {
        class_name: String::new(),
        is_metaclass: false,
        metaclass: None,
        base_classes: Vec::new(),
        subclasses: Vec::new(),
        mixins: Vec::new(),
        inheritance_chain: Vec::new(),
        method_resolution_order: Vec::new(),
        dynamic_attributes: Vec::new(),
    }
}

fn extract_source_context(
    file_path: &std::path::Path,
    line_number: usize,
    context_lines: usize,
) -> Option<serde_json::Value> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().ok()?;

    let start = line_number.saturating_sub(context_lines + 1);
    let end = std::cmp::min(line_number + context_lines, lines.len());

    if start < lines.len() {
        let context_lines: Vec<serde_json::Value> = (start..end)
            .map(|i| {
                serde_json::json!({
                    "line_number": i + 1,
                    "content": lines.get(i).unwrap_or(&String::new()),
                    "is_target": i + 1 == line_number
                })
            })
            .collect();

        Some(serde_json::json!({
            "lines": context_lines,
            "start_line": start + 1,
            "end_line": end,
            "target_line": line_number
        }))
    } else {
        None
    }
}

fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|_| anyhow::anyhow!("Invalid node ID format: {}", hex_str))
}
