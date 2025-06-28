//! JavaScript/TypeScript analysis tools for framework detection and code intelligence
//!
//! This module provides specialized tools for analyzing JavaScript and TypeScript code,
//! including framework detection, React component analysis, and performance assessment.

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;
use anyhow::Result;
use codeprism_lang_js::JavaScriptAnalyzer;
use serde_json::Value;
use std::path::Path;

/// List JavaScript analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_javascript_frameworks".to_string(),
            title: Some("Analyze JavaScript Frameworks".to_string()),
            description: "Detect and analyze JavaScript/TypeScript frameworks and libraries in use".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_confidence": {
                        "type": "boolean",
                        "description": "Include confidence scores for framework detection",
                        "default": true
                    },
                    "analyze_versions": {
                        "type": "boolean", 
                        "description": "Attempt to detect framework versions",
                        "default": true
                    },
                    "frameworks": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["react", "vue", "angular", "express", "nextjs", "nuxt", "all"]
                        },
                        "description": "Specific frameworks to analyze",
                        "default": ["all"]
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "analyze_react_components".to_string(),
            title: Some("Analyze React Components".to_string()),
            description: "Deep analysis of React components, hooks, and patterns".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_hooks": {
                        "type": "boolean",
                        "description": "Include React hooks analysis",
                        "default": true
                    },
                    "analyze_props": {
                        "type": "boolean",
                        "description": "Analyze component props and PropTypes",
                        "default": true
                    },
                    "detect_patterns": {
                        "type": "boolean",
                        "description": "Detect React patterns and anti-patterns",
                        "default": true
                    },
                    "include_context": {
                        "type": "boolean",
                        "description": "Analyze React Context usage",
                        "default": true
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "analyze_nodejs_patterns".to_string(),
            title: Some("Analyze Node.js Patterns".to_string()),
            description: "Analyze Node.js backend patterns, database integrations, and architecture".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_databases": {
                        "type": "boolean",
                        "description": "Include database integration analysis",
                        "default": true
                    },
                    "analyze_routing": {
                        "type": "boolean",
                        "description": "Analyze routing patterns and middleware",
                        "default": true
                    },
                    "detect_orms": {
                        "type": "boolean",
                        "description": "Detect ORM frameworks and patterns",
                        "default": true
                    },
                    "include_security": {
                        "type": "boolean",
                        "description": "Include security pattern analysis",
                        "default": true
                    }
                },
                "required": []
            }),
        }
    ]
}

/// Route JavaScript analysis tool calls
pub async fn call_tool(
    server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    match params.name.as_str() {
        "analyze_javascript_frameworks" => analyze_javascript_frameworks(server, params.arguments.as_ref()).await,
        "analyze_react_components" => analyze_react_components(server, params.arguments.as_ref()).await,
        "analyze_nodejs_patterns" => analyze_nodejs_patterns(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!(
            "Unknown JavaScript analysis tool: {}",
            params.name
        )),
    }
}

/// Analyze JavaScript/TypeScript frameworks
async fn analyze_javascript_frameworks(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let default_args = serde_json::json!({});
    let args = arguments.unwrap_or(&default_args);

    let include_confidence = args.get("include_confidence")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyze_versions = args.get("analyze_versions")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let frameworks = args.get("frameworks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let analyzer = JavaScriptAnalyzer::new();
    let mut all_frameworks = Vec::new();
    let mut files_analyzed = 0;

    // Get repository path and analyze files
    if let Some(repo_path) = server.repository_path() {
        match server.scanner().discover_files(repo_path) {
            Ok(files) => {
                for file_path in files {
                    if is_javascript_file(&file_path) {
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                files_analyzed += 1;
                                
                                // Analyze content using our Phase 1.3 capabilities
                                match analyzer.detect_frameworks(&content) {
                                    Ok(frameworks_detected) => {
                                        // Extract framework information
                                        for framework in frameworks_detected {
                                            if frameworks.contains(&"all".to_string()) || 
                                               frameworks.contains(&framework.name.to_lowercase()) {
                                                all_frameworks.push(framework);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error analyzing {}: {}", file_path.display(), e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to discover files: {}", e));
            }
        }
    }

    // Aggregate and analyze results
    let mut framework_summary = std::collections::HashMap::new();
    for framework in &all_frameworks {
        let entry = framework_summary.entry(framework.name.clone()).or_insert_with(|| {
            serde_json::json!({
                "name": framework.name,
                "confidence": framework.confidence,
                "files_count": 0,
                "versions": Vec::<String>::new(),
                "features": Vec::<String>::new(),
                "best_practices": Vec::<String>::new()
            })
        });

        // Update counts and features
        entry["files_count"] = serde_json::json!(entry["files_count"].as_u64().unwrap_or(0) + 1);
        
        if include_confidence {
            let current_confidence = entry["confidence"].as_f64().unwrap_or(0.0);
            let new_confidence = (current_confidence + framework.confidence as f64) / 2.0;
            entry["confidence"] = serde_json::json!(new_confidence);
        }

        // Merge features and best practices
        for feature in &framework.features_used {
            if !entry["features"].as_array().unwrap().iter().any(|f| f.as_str() == Some(feature)) {
                entry["features"].as_array_mut().unwrap().push(serde_json::json!(feature));
            }
        }

        for practice in &framework.best_practices {
            if !entry["best_practices"].as_array().unwrap().iter().any(|p| p.as_str() == Some(practice)) {
                entry["best_practices"].as_array_mut().unwrap().push(serde_json::json!(practice));
            }
        }
    }

    let result = serde_json::json!({
        "analysis_summary": {
            "files_analyzed": files_analyzed,
            "frameworks_detected": framework_summary.len(),
            "total_framework_instances": all_frameworks.len()
        },
        "frameworks": framework_summary.values().collect::<Vec<_>>(),
        "analysis_metadata": {
            "include_confidence": include_confidence,
            "analyze_versions": analyze_versions,
            "requested_frameworks": frameworks,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "analyzer_version": "2.1.0"
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Analyze React components
async fn analyze_react_components(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let default_args = serde_json::json!({});
    let args = arguments.unwrap_or(&default_args);

    let include_hooks = args.get("include_hooks")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyze_props = args.get("analyze_props")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let detect_patterns = args.get("detect_patterns")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_context = args.get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyzer = JavaScriptAnalyzer::new();
    let mut all_components = Vec::new();
    let mut files_analyzed = 0;

    // Get repository path and analyze React files
    if let Some(repo_path) = server.repository_path() {
        match server.scanner().discover_files(repo_path) {
            Ok(files) => {
                for file_path in files {
                    if is_react_file(&file_path) {
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                files_analyzed += 1;
                                
                                // Analyze content using our Phase 1.3 capabilities
                                match analyzer.analyze_react_patterns(&content) {
                                    Ok(components) => {
                                        // Extract React component information
                                        for component in components {
                                            let hooks_data = if include_hooks {
                                                component.hooks_used.iter().map(|h| serde_json::json!({
                                                    "name": h.name,
                                                    "hook_type": h.hook_type,
                                                    "dependencies": h.dependencies,
                                                    "custom_hook": h.custom_hook
                                                })).collect::<Vec<_>>()
                                            } else {
                                                Vec::new()
                                            };

                                            let props_data = if analyze_props {
                                                Some(serde_json::json!({
                                                    "prop_names": component.props_analysis.prop_names,
                                                    "has_prop_types": component.props_analysis.has_prop_types,
                                                    "has_default_props": component.props_analysis.has_default_props,
                                                    "destructured": component.props_analysis.destructured,
                                                    "typescript_props": component.props_analysis.typescript_props
                                                }))
                                            } else {
                                                None
                                            };

                                            let context_data = if include_context {
                                                component.context_usage.iter().map(|c| serde_json::json!({
                                                    "context_name": c.context_name,
                                                    "usage_type": c.usage_type,
                                                    "values_consumed": c.values_consumed
                                                })).collect::<Vec<_>>()
                                            } else {
                                                Vec::new()
                                            };

                                            all_components.push(serde_json::json!({
                                                "name": component.name,
                                                "type": format!("{:?}", component.component_type),
                                                "file": file_path.display().to_string(),
                                                "hooks": if include_hooks { Some(hooks_data) } else { None },
                                                "props": props_data,
                                                "jsx_elements": component.jsx_elements,
                                                "lifecycle_methods": component.lifecycle_methods,
                                                "context_usage": if include_context { Some(context_data) } else { None }
                                            }));
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error analyzing {}: {}", file_path.display(), e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to discover files: {}", e));
            }
        }
    }

    // Generate component analysis summary
    let hooks_summary = if include_hooks {
        let mut hook_counts = std::collections::HashMap::new();
        for component in &all_components {
            if let Some(hooks) = component.get("hooks").and_then(|h| h.as_array()) {
                for hook in hooks {
                    if let Some(hook_name) = hook.as_str() {
                        *hook_counts.entry(hook_name.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        Some(hook_counts)
    } else {
        None
    };

    let result = serde_json::json!({
        "analysis_summary": {
            "files_analyzed": files_analyzed,
            "components_found": all_components.len(),
            "hooks_analysis_included": include_hooks,
            "props_analysis_included": analyze_props,
            "context_analysis_included": include_context
        },
        "components": all_components,
        "hooks_summary": hooks_summary,
        "analysis_metadata": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "analyzer_version": "2.1.0"
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Analyze Node.js patterns
async fn analyze_nodejs_patterns(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let default_args = serde_json::json!({});
    let args = arguments.unwrap_or(&default_args);

    let include_databases = args.get("include_databases")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyze_routing = args.get("analyze_routing")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let detect_orms = args.get("detect_orms")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_security = args.get("include_security")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let analyzer = JavaScriptAnalyzer::new();
    let mut nodejs_patterns = Vec::new();
    let mut files_analyzed = 0;

    // Get repository path and analyze Node.js files
    if let Some(repo_path) = server.repository_path() {
        match server.scanner().discover_files(repo_path) {
            Ok(files) => {
                for file_path in files {
                    if is_nodejs_file(&file_path) {
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                files_analyzed += 1;
                                
                                // Analyze content using our Phase 1.3 capabilities
                                match analyzer.analyze_nodejs_patterns(&content) {
                                    Ok(patterns) => {
                                        // Extract Node.js pattern information
                                        for pattern in patterns {
                                            let route_data = pattern.route_info.map(|r| serde_json::json!({
                                                "path": r.path,
                                                "method": r.method,
                                                "parameters": r.parameters,
                                                "query_params": r.query_params,
                                                "middleware_used": r.middleware_used
                                            }));

                                            let db_patterns_data = if include_databases {
                                                pattern.database_patterns.iter().map(|db| serde_json::json!({
                                                    "db_type": db.db_type,
                                                    "operations": db.operations,
                                                    "orm_framework": db.orm_framework
                                                })).collect::<Vec<_>>()
                                            } else {
                                                Vec::new()
                                            };

                                            nodejs_patterns.push(serde_json::json!({
                                                "pattern_type": format!("{:?}", pattern.pattern_type),
                                                "file": file_path.display().to_string(),
                                                "framework": pattern.framework,
                                                "route_info": route_data,
                                                "middleware_chain": pattern.middleware_chain,
                                                "http_methods": pattern.http_methods,
                                                "database_patterns": if include_databases { Some(db_patterns_data) } else { None }
                                            }));
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error analyzing {}: {}", file_path.display(), e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to discover files: {}", e));
            }
        }
    }

    let result = serde_json::json!({
        "analysis_summary": {
            "files_analyzed": files_analyzed,
            "patterns_found": nodejs_patterns.len(),
            "database_analysis_included": include_databases,
            "routing_analysis_included": analyze_routing,
            "orm_analysis_included": detect_orms,
            "security_analysis_included": include_security
        },
        "nodejs_patterns": nodejs_patterns,
        "analysis_metadata": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "analyzer_version": "2.1.0"
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Helper function to determine if a file is a JavaScript/TypeScript file
fn is_javascript_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(extension.to_str(), Some("js") | Some("jsx") | Some("ts") | Some("tsx") | Some("mjs") | Some("cjs"))
    } else {
        false
    }
}

/// Helper function to determine if a file is a React file
fn is_react_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(extension.to_str(), Some("jsx") | Some("tsx"))
    } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        name.contains("component") || name.contains("Component") || name.starts_with("use")
    } else {
        false
    }
}

/// Helper function to determine if a file is a Node.js file
fn is_nodejs_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(extension.to_str(), Some("js") | Some("ts") | Some("mjs") | Some("cjs"))
    } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(name, "server.js" | "app.js" | "index.js" | "main.js" | "api.js" | "routes.js" | "middleware.js")
    } else {
        false
    }
} 