//! Workflow optimization and performance enhancement
//!
//! Provides intelligent optimization suggestions for analysis workflows
//! based on performance patterns, tool usage, and execution efficiency.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::context::session::SessionId;
use crate::tools::{CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;

/// Create the optimize_workflow tool
pub fn create_optimize_workflow_tool() -> Tool {
    Tool {
        name: "optimize_workflow".to_string(),
        title: Some("Optimize Workflow".to_string()),
        description: "Analyze and optimize analysis workflows for better performance and efficiency. Provides recommendations for tool sequencing, parallelization, and resource optimization.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "workflow_history": {
                    "type": "array",
                    "description": "History of tool calls to analyze",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool_name": {"type": "string"},
                            "execution_time_ms": {"type": "integer"},
                            "success": {"type": "boolean"},
                            "parameters": {"type": "object"},
                            "timestamp": {"type": "string"}
                        },
                        "required": ["tool_name", "execution_time_ms", "success"]
                    }
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID to analyze (optional)"
                },
                "optimization_goals": {
                    "type": "array",
                    "description": "Optimization objectives",
                    "items": {
                        "type": "string",
                        "enum": ["speed", "accuracy", "resource_usage", "parallelization", "user_experience"]
                    },
                    "default": ["speed", "user_experience"]
                },
                "target_performance": {
                    "type": "object",
                    "description": "Performance targets",
                    "properties": {
                        "max_total_time_minutes": {"type": "integer", "minimum": 1, "maximum": 60},
                        "max_parallel_tools": {"type": "integer", "minimum": 1, "maximum": 10},
                        "target_success_rate": {"type": "number", "minimum": 0.5, "maximum": 1.0}
                    }
                },
                "constraints": {
                    "type": "object",
                    "description": "Optimization constraints",
                    "properties": {
                        "preserve_accuracy": {"type": "boolean", "default": true},
                        "memory_limit_mb": {"type": "integer", "minimum": 100, "maximum": 2048},
                        "must_include_tools": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tools that must be included in optimized workflow"
                        },
                        "exclude_tools": {
                            "type": "array", 
                            "items": {"type": "string"},
                            "description": "Tools to exclude from optimization"
                        }
                    }
                }
            },
            "additionalProperties": false
        }),
    }
}

/// Optimize workflow based on history and goals
pub async fn optimize_workflow(
    _server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let default_args = json!({});
    let args = arguments.unwrap_or(&default_args);

    let workflow_history = args.get("workflow_history").and_then(|v| v.as_array());

    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| SessionId(s.to_string()));

    let optimization_goals = args
        .get("optimization_goals")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["speed", "user_experience"]);

    let target_performance = args.get("target_performance");
    let constraints = args.get("constraints");

    // Analyze workflow data
    let analysis_data = if let Some(history) = workflow_history {
        analyze_workflow_history(history)?
    } else if let Some(session_id) = session_id {
        analyze_session_workflow(session_id)?
    } else {
        return Err(anyhow::anyhow!(
            "Either workflow_history or session_id must be provided"
        ));
    };

    // Generate optimization recommendations
    let optimizations = generate_optimization_recommendations(
        &analysis_data,
        &optimization_goals,
        target_performance,
        constraints,
    )?;

    // Create optimization report
    let mut result = json!({
        "optimization_analysis": {
            "current_performance": analysis_data.performance_metrics,
            "identified_issues": analysis_data.issues,
            "optimization_potential": analysis_data.optimization_potential,
            "bottlenecks": analysis_data.bottlenecks
        },
        "optimization_recommendations": optimizations.recommendations,
        "optimized_workflow": {
            "tool_sequence": optimizations.optimized_sequence,
            "parallel_groups": optimizations.parallel_groups,
            "estimated_improvement": optimizations.estimated_improvement,
            "execution_strategy": optimizations.execution_strategy
        },
        "implementation_guide": {
            "quick_wins": optimizations.quick_wins,
            "advanced_optimizations": optimizations.advanced_optimizations,
            "migration_steps": optimizations.migration_steps,
            "testing_recommendations": optimizations.testing_recommendations
        }
    });

    // Add performance projections
    result["performance_projections"] = json!({
        "current_metrics": analysis_data.performance_metrics,
        "optimized_metrics": optimizations.projected_metrics,
        "improvement_summary": {
            "time_reduction_percent": optimizations.estimated_improvement.time_reduction,
            "efficiency_gain": optimizations.estimated_improvement.efficiency_gain,
            "resource_savings": optimizations.estimated_improvement.resource_savings
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Workflow analysis data
#[derive(Debug, Clone)]
struct WorkflowAnalysis {
    performance_metrics: PerformanceMetrics,
    issues: Vec<WorkflowIssue>,
    optimization_potential: f64,
    bottlenecks: Vec<Bottleneck>,
    tool_usage_patterns: HashMap<String, ToolUsagePattern>,
}

/// Performance metrics
#[derive(Debug, Clone, serde::Serialize)]
struct PerformanceMetrics {
    total_execution_time_ms: u64,
    average_tool_time_ms: f64,
    success_rate: f64,
    parallelization_efficiency: f64,
    resource_utilization: f64,
    tool_count: usize,
}

/// Workflow issue
#[derive(Debug, Clone, serde::Serialize)]
struct WorkflowIssue {
    issue_type: String,
    severity: String,
    description: String,
    affected_tools: Vec<String>,
    impact: String,
}

/// Performance bottleneck
#[derive(Debug, Clone, serde::Serialize)]
struct Bottleneck {
    bottleneck_type: String,
    location: String,
    impact_ms: u64,
    recommendation: String,
}

/// Tool usage pattern
#[derive(Debug, Clone, serde::Serialize)]
struct ToolUsagePattern {
    frequency: u32,
    average_execution_time_ms: f64,
    success_rate: f64,
    common_parameters: HashMap<String, Value>,
    dependencies: Vec<String>,
}

/// Optimization recommendations
#[derive(Debug, Clone)]
struct OptimizationRecommendations {
    recommendations: Vec<OptimizationRecommendation>,
    optimized_sequence: Vec<OptimizedToolStep>,
    parallel_groups: Vec<Vec<String>>,
    estimated_improvement: ImprovementEstimate,
    execution_strategy: String,
    quick_wins: Vec<String>,
    advanced_optimizations: Vec<String>,
    migration_steps: Vec<String>,
    testing_recommendations: Vec<String>,
    projected_metrics: PerformanceMetrics,
}

/// Individual optimization recommendation
#[derive(Debug, Clone, serde::Serialize)]
struct OptimizationRecommendation {
    optimization_type: String,
    priority: String,
    description: String,
    implementation: String,
    expected_benefit: String,
    effort_level: String,
    tools_affected: Vec<String>,
}

/// Optimized tool step
#[derive(Debug, Clone, serde::Serialize)]
struct OptimizedToolStep {
    tool_name: String,
    execution_order: u32,
    parallel_group: Option<u32>,
    optimized_parameters: Value,
    rationale: String,
    expected_time_ms: u64,
}

/// Improvement estimate
#[derive(Debug, Clone, serde::Serialize)]
struct ImprovementEstimate {
    time_reduction: f64,
    efficiency_gain: f64,
    resource_savings: f64,
    confidence: f64,
}

/// Analyze workflow history from provided data
fn analyze_workflow_history(history: &[Value]) -> Result<WorkflowAnalysis> {
    let mut total_time = 0u64;
    let mut successful_tools = 0;
    let mut total_tools = 0;
    let mut tool_times = HashMap::new();
    let mut tool_success = HashMap::new();

    for entry in history {
        let tool_name = entry
            .get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let execution_time = entry
            .get("execution_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let success = entry
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        total_time += execution_time;
        total_tools += 1;

        if success {
            successful_tools += 1;
        }

        // Track tool-specific metrics
        let times = tool_times
            .entry(tool_name.to_string())
            .or_insert_with(Vec::new);
        times.push(execution_time);

        let successes = tool_success
            .entry(tool_name.to_string())
            .or_insert_with(|| (0, 0));
        successes.1 += 1; // total count
        if success {
            successes.0 += 1; // success count
        }
    }

    let average_time = if total_tools > 0 {
        total_time as f64 / total_tools as f64
    } else {
        0.0
    };
    let success_rate = if total_tools > 0 {
        successful_tools as f64 / total_tools as f64
    } else {
        0.0
    };

    // Identify issues and bottlenecks
    let issues = identify_workflow_issues(&tool_times, &tool_success, success_rate);
    let bottlenecks = identify_bottlenecks(&tool_times);

    // Calculate optimization potential
    let optimization_potential = calculate_optimization_potential(&tool_times, success_rate);

    // Create tool usage patterns
    let mut tool_usage_patterns = HashMap::new();
    for (tool_name, times) in &tool_times {
        let avg_time = times.iter().sum::<u64>() as f64 / times.len() as f64;
        let (successes, total) = tool_success.get(tool_name).unwrap_or(&(0, 0));
        let tool_success_rate = if *total > 0 {
            *successes as f64 / *total as f64
        } else {
            0.0
        };

        tool_usage_patterns.insert(
            tool_name.clone(),
            ToolUsagePattern {
                frequency: times.len() as u32,
                average_execution_time_ms: avg_time,
                success_rate: tool_success_rate,
                common_parameters: HashMap::new(),
                dependencies: vec![],
            },
        );
    }

    Ok(WorkflowAnalysis {
        performance_metrics: PerformanceMetrics {
            total_execution_time_ms: total_time,
            average_tool_time_ms: average_time,
            success_rate,
            parallelization_efficiency: estimate_parallelization_efficiency(&tool_times),
            resource_utilization: 0.8, // Placeholder
            tool_count: total_tools,
        },
        issues,
        optimization_potential,
        bottlenecks,
        tool_usage_patterns,
    })
}

/// Analyze session workflow
fn analyze_session_workflow(_session_id: SessionId) -> Result<WorkflowAnalysis> {
    // In real implementation, would fetch session data
    Ok(WorkflowAnalysis {
        performance_metrics: PerformanceMetrics {
            total_execution_time_ms: 30000,
            average_tool_time_ms: 5000.0,
            success_rate: 0.85,
            parallelization_efficiency: 0.6,
            resource_utilization: 0.7,
            tool_count: 6,
        },
        issues: vec![],
        optimization_potential: 0.7,
        bottlenecks: vec![],
        tool_usage_patterns: HashMap::new(),
    })
}

/// Identify workflow issues
fn identify_workflow_issues(
    tool_times: &HashMap<String, Vec<u64>>,
    tool_success: &HashMap<String, (u32, u32)>,
    _overall_success_rate: f64,
) -> Vec<WorkflowIssue> {
    let mut issues = Vec::new();

    // Check for slow tools
    for (tool_name, times) in tool_times {
        let avg_time = times.iter().sum::<u64>() as f64 / times.len() as f64;
        if avg_time > 15000.0 {
            // > 15 seconds
            issues.push(WorkflowIssue {
                issue_type: "performance".to_string(),
                severity: "medium".to_string(),
                description: format!("{} has high average execution time", tool_name),
                affected_tools: vec![tool_name.clone()],
                impact: "Significantly increases total workflow time".to_string(),
            });
        }
    }

    // Check for tools with low success rates
    for (tool_name, (successes, total)) in tool_success {
        let success_rate = *successes as f64 / *total as f64;
        if success_rate < 0.8 {
            issues.push(WorkflowIssue {
                issue_type: "reliability".to_string(),
                severity: "high".to_string(),
                description: format!(
                    "{} has low success rate: {:.1}%",
                    tool_name,
                    success_rate * 100.0
                ),
                affected_tools: vec![tool_name.clone()],
                impact: "May cause workflow failures and require retries".to_string(),
            });
        }
    }

    // Check for sequential execution inefficiency
    if tool_times.len() > 2 {
        issues.push(WorkflowIssue {
            issue_type: "parallelization".to_string(),
            severity: "low".to_string(),
            description: "Multiple tools could potentially run in parallel".to_string(),
            affected_tools: tool_times.keys().cloned().collect(),
            impact: "Sequential execution may be unnecessarily slow".to_string(),
        });
    }

    issues
}

/// Identify performance bottlenecks
fn identify_bottlenecks(tool_times: &HashMap<String, Vec<u64>>) -> Vec<Bottleneck> {
    let mut bottlenecks = Vec::new();

    for (tool_name, times) in tool_times {
        let max_time = times.iter().max().unwrap_or(&0);
        let avg_time = times.iter().sum::<u64>() as f64 / times.len() as f64;

        if *max_time > avg_time as u64 * 2 {
            bottlenecks.push(Bottleneck {
                bottleneck_type: "execution_variance".to_string(),
                location: tool_name.clone(),
                impact_ms: *max_time - avg_time as u64,
                recommendation: "Investigate inconsistent execution times".to_string(),
            });
        }

        if avg_time > 10000.0 {
            bottlenecks.push(Bottleneck {
                bottleneck_type: "slow_tool".to_string(),
                location: tool_name.clone(),
                impact_ms: avg_time as u64,
                recommendation: "Consider optimizing parameters or breaking into smaller operations".to_string(),
            });
        }
    }

    bottlenecks
}

/// Calculate optimization potential
fn calculate_optimization_potential(
    tool_times: &HashMap<String, Vec<u64>>,
    success_rate: f64,
) -> f64 {
    let mut potential = 0.0;

    // Parallelization potential
    if tool_times.len() > 1 {
        potential += 0.3; // 30% improvement from parallelization
    }

    // Success rate improvement potential
    if success_rate < 0.9 {
        potential += (0.9 - success_rate) * 0.5; // Potential from improving reliability
    }

    // Time optimization potential
    let total_time: u64 = tool_times.values().flatten().sum();
    let tool_count = tool_times.values().map(|times| times.len()).sum::<usize>();
    if tool_count > 0 {
        let avg_time = total_time as f64 / tool_count as f64;
        if avg_time > 5000.0 {
            potential += 0.2; // 20% improvement from time optimization
        }
    }

    potential.min(1.0)
}

/// Estimate parallelization efficiency
fn estimate_parallelization_efficiency(tool_times: &HashMap<String, Vec<u64>>) -> f64 {
    if tool_times.len() <= 1 {
        return 1.0;
    }

    // Simple estimation based on tool count and average times
    let avg_times: Vec<f64> = tool_times
        .values()
        .map(|times| times.iter().sum::<u64>() as f64 / times.len() as f64)
        .collect();

    let max_time = avg_times.iter().fold(0.0f64, |acc, &x| acc.max(x));
    let total_time: f64 = avg_times.iter().sum();

    if max_time > 0.0 {
        max_time / total_time
    } else {
        1.0
    }
}

/// Generate optimization recommendations
fn generate_optimization_recommendations(
    analysis: &WorkflowAnalysis,
    goals: &[&str],
    _target_performance: Option<&Value>,
    _constraints: Option<&Value>,
) -> Result<OptimizationRecommendations> {
    let mut recommendations = Vec::new();
    let mut quick_wins = Vec::new();
    let mut advanced_optimizations = Vec::new();

    // Speed optimization recommendations
    if goals.contains(&"speed") {
        if analysis.performance_metrics.parallelization_efficiency < 0.8 {
            recommendations.push(OptimizationRecommendation {
                optimization_type: "parallelization".to_string(),
                priority: "high".to_string(),
                description: "Execute compatible tools in parallel to reduce total time".to_string(),
                implementation: "Group analysis tools (complexity, security, performance) for parallel execution".to_string(),
                expected_benefit: "30-50% reduction in total execution time".to_string(),
                effort_level: "medium".to_string(),
                tools_affected: analysis.tool_usage_patterns.keys().cloned().collect(),
            });
            quick_wins.push("Enable parallel execution for analysis tools".to_string());
        }

        // Tool-specific optimizations
        for (tool_name, pattern) in &analysis.tool_usage_patterns {
            if pattern.average_execution_time_ms > 10000.0 {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: "parameter_optimization".to_string(),
                    priority: "medium".to_string(),
                    description: format!(
                        "Optimize {} parameters to reduce execution time",
                        tool_name
                    ),
                    implementation: "Reduce scope or adjust analysis depth".to_string(),
                    expected_benefit: "20-40% reduction in tool execution time".to_string(),
                    effort_level: "low".to_string(),
                    tools_affected: vec![tool_name.clone()],
                });
                quick_wins.push(format!(
                    "Optimize {} parameters for faster execution",
                    tool_name
                ));
            }
        }
    }

    // Resource optimization
    if goals.contains(&"resource_usage") {
        advanced_optimizations
            .push("Implement intelligent caching for expensive operations".to_string());
        advanced_optimizations.push("Use lazy loading for large result sets".to_string());
    }

    // Create optimized sequence
    let optimized_sequence = create_optimized_sequence(&analysis.tool_usage_patterns)?;
    let parallel_groups = identify_parallel_groups(&analysis.tool_usage_patterns);

    // Estimate improvements
    let time_reduction = if analysis.performance_metrics.parallelization_efficiency < 0.8 {
        0.4
    } else {
        0.2
    };
    let estimated_improvement = ImprovementEstimate {
        time_reduction,
        efficiency_gain: 0.25,
        resource_savings: 0.15,
        confidence: 0.8,
    };

    // Project optimized metrics
    let projected_metrics = PerformanceMetrics {
        total_execution_time_ms: (analysis.performance_metrics.total_execution_time_ms as f64
            * (1.0 - time_reduction)) as u64,
        average_tool_time_ms: analysis.performance_metrics.average_tool_time_ms * 0.8,
        success_rate: (analysis.performance_metrics.success_rate + 0.1).min(1.0),
        parallelization_efficiency: (analysis.performance_metrics.parallelization_efficiency + 0.3)
            .min(1.0),
        resource_utilization: analysis.performance_metrics.resource_utilization * 0.85,
        tool_count: analysis.performance_metrics.tool_count,
    };

    Ok(OptimizationRecommendations {
        recommendations,
        optimized_sequence,
        parallel_groups,
        estimated_improvement,
        execution_strategy: "optimized_parallel".to_string(),
        quick_wins,
        advanced_optimizations,
        migration_steps: vec![
            "1. Test parallel execution with non-critical tools".to_string(),
            "2. Gradually increase parallelization based on results".to_string(),
            "3. Implement caching for frequently used operations".to_string(),
            "4. Monitor performance metrics and adjust as needed".to_string(),
        ],
        testing_recommendations: vec![
            "A/B test optimized vs original workflow".to_string(),
            "Monitor success rates during optimization rollout".to_string(),
            "Validate result quality remains consistent".to_string(),
        ],
        projected_metrics,
    })
}

/// Create optimized tool sequence
fn create_optimized_sequence(
    tool_patterns: &HashMap<String, ToolUsagePattern>,
) -> Result<Vec<OptimizedToolStep>> {
    let mut sequence = Vec::new();
    let mut order = 1;

    // Repository stats should typically run first
    if tool_patterns.contains_key("repository_stats") {
        sequence.push(OptimizedToolStep {
            tool_name: "repository_stats".to_string(),
            execution_order: order,
            parallel_group: None,
            optimized_parameters: json!({}),
            rationale: "Provides context for subsequent analysis".to_string(),
            expected_time_ms: tool_patterns
                .get("repository_stats")
                .map(|p| p.average_execution_time_ms as u64)
                .unwrap_or(2000),
        });
        order += 1;
    }

    // Group analysis tools for parallel execution
    let analysis_tools = [
        "analyze_complexity",
        "analyze_security",
        "analyze_performance",
    ];
    let parallel_group_id = 1;

    for tool_name in &analysis_tools {
        if let Some(pattern) = tool_patterns.get(*tool_name) {
            sequence.push(OptimizedToolStep {
                tool_name: tool_name.to_string(),
                execution_order: order,
                parallel_group: Some(parallel_group_id),
                optimized_parameters: json!({}),
                rationale: "Can run in parallel with other analysis tools".to_string(),
                expected_time_ms: (pattern.average_execution_time_ms * 0.8) as u64,
            });
        }
    }

    if sequence
        .iter()
        .any(|s| s.parallel_group == Some(parallel_group_id))
    {
        order += 1;
        // parallel_group_id would be incremented here for future groups
    }

    // Add remaining tools
    for (tool_name, pattern) in tool_patterns {
        if !sequence.iter().any(|s| s.tool_name == *tool_name) {
            sequence.push(OptimizedToolStep {
                tool_name: tool_name.clone(),
                execution_order: order,
                parallel_group: None,
                optimized_parameters: json!({}),
                rationale: "Sequential execution based on dependencies".to_string(),
                expected_time_ms: pattern.average_execution_time_ms as u64,
            });
            order += 1;
        }
    }

    Ok(sequence)
}

/// Identify tools that can run in parallel
fn identify_parallel_groups(tool_patterns: &HashMap<String, ToolUsagePattern>) -> Vec<Vec<String>> {
    let mut groups = Vec::new();

    // Analysis tools group
    let analysis_tools: Vec<String> = [
        "analyze_complexity",
        "analyze_security",
        "analyze_performance",
    ]
    .iter()
    .filter(|tool| tool_patterns.contains_key(**tool))
    .map(|tool| tool.to_string())
    .collect();

    if analysis_tools.len() > 1 {
        groups.push(analysis_tools);
    }

    // Search tools group
    let search_tools: Vec<String> = ["search_symbols", "search_content", "find_files"]
        .iter()
        .filter(|tool| tool_patterns.contains_key(**tool))
        .map(|tool| tool.to_string())
        .collect();

    if search_tools.len() > 1 {
        groups.push(search_tools);
    }

    groups
}
