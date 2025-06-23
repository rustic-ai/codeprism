//! Workflow guidance and analysis planning
//!
//! Provides intelligent workflow recommendations based on user goals,
//! current context, and optimal tool sequencing strategies.

use anyhow::Result;
use serde_json::{json, Value};

use crate::context::session::SessionId;
use crate::context::session::WorkflowStage;
use crate::tools::{CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;

/// Create the suggest_analysis_workflow tool
pub fn create_suggest_analysis_workflow_tool() -> Tool {
    Tool {
        name: "suggest_analysis_workflow".to_string(),
        title: Some("Suggest Analysis Workflow".to_string()),
        description: "Recommend optimal sequence of analysis tools based on user goals and current context. Provides systematic workflow guidance to achieve specific analysis objectives efficiently.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "goal": {
                    "type": "string",
                    "description": "Analysis objective: 'understand_codebase', 'find_security_issues', 'analyze_performance', 'trace_data_flow', 'analyze_architecture', 'debug_issue', 'refactor_preparation'"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID for context-aware recommendations (optional)"
                },
                "complexity_preference": {
                    "type": "string",
                    "enum": ["quick", "standard", "comprehensive"],
                    "description": "Analysis depth preference",
                    "default": "standard"
                },
                "time_constraints": {
                    "type": "integer",
                    "description": "Approximate time budget in minutes (optional)",
                    "minimum": 5,
                    "maximum": 240
                },
                "current_context": {
                    "type": "object",
                    "description": "Current analysis context (optional)",
                    "properties": {
                        "symbols_analyzed": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Symbols already analyzed"
                        },
                        "areas_of_interest": {
                            "type": "array", 
                            "items": {"type": "string"},
                            "description": "Specific areas, files, or patterns to focus on"
                        },
                        "known_issues": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Known issues or concerns to investigate"
                        }
                    }
                }
            },
            "required": ["goal"],
            "additionalProperties": false
        }),
    }
}

/// Generate workflow recommendations based on user goals
pub async fn suggest_analysis_workflow(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let goal = args
        .get("goal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid goal parameter"))?;

    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| SessionId(s.to_string()));

    let complexity = args
        .get("complexity_preference")
        .and_then(|v| v.as_str())
        .unwrap_or("standard");

    let time_budget = args
        .get("time_constraints")
        .and_then(|v| v.as_u64())
        .map(|t| t as u32);

    let current_context = args.get("current_context");

    // Generate workflow recommendation based on goal
    let workflow = generate_workflow_for_goal(
        goal,
        complexity,
        time_budget,
        current_context,
        session_id.as_ref(),
    )?;

    // Create comprehensive workflow plan
    let mut result = json!({
        "workflow_plan": {
            "goal": goal,
            "complexity_level": complexity,
            "estimated_duration_minutes": workflow.estimated_duration,
            "total_tools": workflow.tool_sequence.len(),
            "workflow_stages": workflow.stages,
            "success_criteria": workflow.success_criteria
        },
        "tool_sequence": workflow.tool_sequence,
        "execution_strategy": {
            "type": workflow.execution_type,
            "parallel_groups": workflow.parallel_groups,
            "dependencies": workflow.dependencies
        },
        "guidance": {
            "getting_started": workflow.getting_started_guidance,
            "optimization_tips": workflow.optimization_tips,
            "common_pitfalls": workflow.common_pitfalls,
            "alternative_approaches": workflow.alternative_approaches
        }
    });

    // Add session-specific recommendations if session provided
    if let Some(session_id) = session_id {
        if let Ok(session_context) = get_session_context(session_id) {
            result["session_context"] = json!({
                "current_stage": session_context.current_stage,
                "completed_tools": session_context.completed_tools,
                "recommendations": session_context.recommendations,
                "progress_assessment": session_context.progress_assessment
            });
        }
    }

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Workflow recommendation structure
#[derive(Debug, Clone)]
struct WorkflowRecommendation {
    goal: String,
    estimated_duration: u32,
    tool_sequence: Vec<ToolStep>,
    stages: Vec<WorkflowStageInfo>,
    execution_type: String,
    parallel_groups: Vec<Vec<String>>,
    dependencies: Vec<ToolDependency>,
    success_criteria: Vec<String>,
    getting_started_guidance: String,
    optimization_tips: Vec<String>,
    common_pitfalls: Vec<String>,
    alternative_approaches: Vec<String>,
}

/// Individual tool step in workflow
#[derive(Debug, Clone, serde::Serialize)]
struct ToolStep {
    step: u32,
    tool_name: String,
    parameters: serde_json::Value,
    reasoning: String,
    expected_outcome: String,
    estimated_time_minutes: u32,
    priority: String,
    optional: bool,
}

/// Workflow stage information
#[derive(Debug, Clone, serde::Serialize)]
struct WorkflowStageInfo {
    stage: String,
    description: String,
    tools: Vec<String>,
    success_indicators: Vec<String>,
}

/// Tool dependency relationship
#[derive(Debug, Clone, serde::Serialize)]
struct ToolDependency {
    tool: String,
    depends_on: Vec<String>,
    dependency_type: String,
}

/// Session context for recommendations
#[derive(Debug, Clone)]
struct SessionContext {
    current_stage: WorkflowStage,
    completed_tools: Vec<String>,
    recommendations: Vec<String>,
    progress_assessment: String,
}

/// Generate workflow recommendation based on analysis goal
fn generate_workflow_for_goal(
    goal: &str,
    complexity: &str,
    time_budget: Option<u32>,
    _current_context: Option<&Value>,
    _session_id: Option<&SessionId>,
) -> Result<WorkflowRecommendation> {
    match goal {
        "understand_codebase" => generate_codebase_understanding_workflow(complexity, time_budget),
        "find_security_issues" => generate_security_analysis_workflow(complexity, time_budget),
        "analyze_performance" => generate_performance_analysis_workflow(complexity, time_budget),
        "trace_data_flow" => generate_data_flow_analysis_workflow(complexity, time_budget),
        "analyze_architecture" => generate_architecture_analysis_workflow(complexity, time_budget),
        "debug_issue" => generate_debugging_workflow(complexity, time_budget),
        "refactor_preparation" => generate_refactoring_workflow(complexity, time_budget),
        _ => Err(anyhow::anyhow!("Unknown analysis goal: {}", goal)),
    }
}

/// Generate codebase understanding workflow
fn generate_codebase_understanding_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let (tools, duration) = match complexity {
        "quick" => (
            vec![
                ToolStep {
                    step: 1,
                    tool_name: "repository_stats".to_string(),
                    parameters: json!({}),
                    reasoning: "Get high-level overview of repository structure and metrics"
                        .to_string(),
                    expected_outcome: "Understanding of codebase size, languages, and organization"
                        .to_string(),
                    estimated_time_minutes: 2,
                    priority: "high".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 2,
                    tool_name: "search_symbols".to_string(),
                    parameters: json!({"pattern": ".*", "limit": 20}),
                    reasoning: "Discover main classes and functions in the codebase".to_string(),
                    expected_outcome: "List of key symbols and entry points".to_string(),
                    estimated_time_minutes: 3,
                    priority: "high".to_string(),
                    optional: false,
                },
            ],
            10,
        ),
        "comprehensive" => (
            vec![
                ToolStep {
                    step: 1,
                    tool_name: "repository_stats".to_string(),
                    parameters: json!({}),
                    reasoning: "Get comprehensive repository metrics and structure analysis"
                        .to_string(),
                    expected_outcome: "Detailed understanding of codebase organization".to_string(),
                    estimated_time_minutes: 3,
                    priority: "high".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 2,
                    tool_name: "search_symbols".to_string(),
                    parameters: json!({"pattern": ".*", "limit": 50}),
                    reasoning: "Comprehensive symbol discovery and classification".to_string(),
                    expected_outcome: "Complete catalog of main symbols".to_string(),
                    estimated_time_minutes: 5,
                    priority: "high".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 3,
                    tool_name: "detect_patterns".to_string(),
                    parameters: json!({"pattern_types": ["architectural", "design"]}),
                    reasoning: "Identify architectural and design patterns in use".to_string(),
                    expected_outcome:
                        "Understanding of architectural patterns and design principles".to_string(),
                    estimated_time_minutes: 8,
                    priority: "medium".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 4,
                    tool_name: "analyze_transitive_dependencies".to_string(),
                    parameters: json!({"max_depth": 3}),
                    reasoning: "Analyze module dependencies and coupling".to_string(),
                    expected_outcome: "Dependency graph and coupling analysis".to_string(),
                    estimated_time_minutes: 10,
                    priority: "medium".to_string(),
                    optional: true,
                },
            ],
            30,
        ),
        _ => (
            // standard
            vec![
                ToolStep {
                    step: 1,
                    tool_name: "repository_stats".to_string(),
                    parameters: json!({}),
                    reasoning: "Get overview of repository structure and basic metrics".to_string(),
                    expected_outcome: "Understanding of codebase scope and organization"
                        .to_string(),
                    estimated_time_minutes: 2,
                    priority: "high".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 2,
                    tool_name: "search_symbols".to_string(),
                    parameters: json!({"pattern": ".*", "limit": 30}),
                    reasoning: "Discover key symbols and main components".to_string(),
                    expected_outcome: "Catalog of important classes and functions".to_string(),
                    estimated_time_minutes: 4,
                    priority: "high".to_string(),
                    optional: false,
                },
                ToolStep {
                    step: 3,
                    tool_name: "detect_patterns".to_string(),
                    parameters: json!({"pattern_types": ["architectural"]}),
                    reasoning: "Identify main architectural patterns".to_string(),
                    expected_outcome: "Understanding of architectural approach".to_string(),
                    estimated_time_minutes: 6,
                    priority: "medium".to_string(),
                    optional: false,
                },
            ],
            20,
        ),
    };

    let adjusted_duration = time_budget.unwrap_or(duration).min(duration);

    Ok(WorkflowRecommendation {
        goal: "understand_codebase".to_string(),
        estimated_duration: adjusted_duration,
        tool_sequence: tools,
        stages: vec![
            WorkflowStageInfo {
                stage: "Discovery".to_string(),
                description: "Initial exploration and high-level understanding".to_string(),
                tools: vec!["repository_stats".to_string()],
                success_indicators: vec!["Repository metrics obtained".to_string()],
            },
            WorkflowStageInfo {
                stage: "Mapping".to_string(),
                description: "Symbol discovery and component identification".to_string(),
                tools: vec!["search_symbols".to_string(), "detect_patterns".to_string()],
                success_indicators: vec!["Key symbols identified".to_string(), "Patterns recognized".to_string()],
            },
        ],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![
            ToolDependency {
                tool: "search_symbols".to_string(),
                depends_on: vec!["repository_stats".to_string()],
                dependency_type: "context".to_string(),
            }
        ],
        success_criteria: vec![
            "Repository structure and metrics understood".to_string(),
            "Key symbols and components identified".to_string(),
            "Architectural patterns recognized".to_string(),
        ],
        getting_started_guidance: "Start with repository_stats to get an overview, then use search_symbols to discover key components. Focus on understanding the overall structure before diving into details.".to_string(),
        optimization_tips: vec![
            "Use symbol search results to guide deeper analysis".to_string(),
            "Pattern detection helps understand design principles".to_string(),
            "Repository stats provide context for subsequent analysis".to_string(),
        ],
        common_pitfalls: vec![
            "Don't get lost in details too early - maintain high-level perspective".to_string(),
            "Repository stats should guide symbol search scope".to_string(),
        ],
        alternative_approaches: vec![
            "Start with specific areas of interest if known".to_string(),
            "Use search_content for text-based exploration".to_string(),
        ],
    })
}

/// Generate security analysis workflow (simplified implementation)
fn generate_security_analysis_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 15,
        "comprehensive" => 45,
        _ => 25,
    };

    Ok(WorkflowRecommendation {
        goal: "find_security_issues".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "analyze_security".to_string(),
            parameters: json!({"include_all_severities": true}),
            reasoning: "Comprehensive security vulnerability scan".to_string(),
            expected_outcome: "Identification of potential security issues".to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Security Analysis".to_string(),
            description: "Comprehensive security vulnerability assessment".to_string(),
            tools: vec!["analyze_security".to_string()],
            success_indicators: vec!["Security issues identified and prioritized".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Security vulnerabilities identified and assessed".to_string()],
        getting_started_guidance:
            "Run comprehensive security analysis to identify potential vulnerabilities".to_string(),
        optimization_tips: vec!["Focus on high-severity issues first".to_string()],
        common_pitfalls: vec!["Don't ignore low-severity issues in production code".to_string()],
        alternative_approaches: vec![
            "Combine with code pattern analysis for better context".to_string()
        ],
    })
}

/// Generate performance analysis workflow (simplified implementation)
fn generate_performance_analysis_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 10,
        "comprehensive" => 30,
        _ => 20,
    };

    Ok(WorkflowRecommendation {
        goal: "analyze_performance".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "analyze_performance".to_string(),
            parameters: json!({}),
            reasoning: "Analyze performance characteristics and bottlenecks".to_string(),
            expected_outcome: "Performance issues and optimization opportunities identified"
                .to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Performance Analysis".to_string(),
            description: "Performance bottleneck identification and optimization".to_string(),
            tools: vec!["analyze_performance".to_string()],
            success_indicators: vec!["Performance issues identified".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Performance bottlenecks identified and prioritized".to_string()],
        getting_started_guidance:
            "Analyze performance characteristics to identify optimization opportunities".to_string(),
        optimization_tips: vec!["Focus on high-impact performance improvements".to_string()],
        common_pitfalls: vec!["Don't optimize without measuring impact".to_string()],
        alternative_approaches: vec![
            "Combine with complexity analysis for comprehensive assessment".to_string(),
        ],
    })
}

/// Generate data flow analysis workflow (simplified implementation)
fn generate_data_flow_analysis_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 12,
        "comprehensive" => 35,
        _ => 22,
    };

    Ok(WorkflowRecommendation {
        goal: "trace_data_flow".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "trace_data_flow".to_string(),
            parameters: json!({"include_transformations": true}),
            reasoning: "Trace data flow and transformations through the system".to_string(),
            expected_outcome: "Understanding of data flow patterns and transformations".to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Data Flow Analysis".to_string(),
            description: "Comprehensive data flow tracing and analysis".to_string(),
            tools: vec!["trace_data_flow".to_string()],
            success_indicators: vec!["Data flow patterns identified".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Data flow and transformations mapped".to_string()],
        getting_started_guidance: "Trace data flow to understand system data processing"
            .to_string(),
        optimization_tips: vec!["Focus on critical data paths first".to_string()],
        common_pitfalls: vec!["Don't ignore error handling in data flow".to_string()],
        alternative_approaches: vec![
            "Use symbol analysis to understand data structures first".to_string()
        ],
    })
}

/// Generate architecture analysis workflow (simplified implementation)
fn generate_architecture_analysis_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 15,
        "comprehensive" => 40,
        _ => 25,
    };

    Ok(WorkflowRecommendation {
        goal: "analyze_architecture".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "detect_patterns".to_string(),
            parameters: json!({"pattern_types": ["architectural", "design"]}),
            reasoning: "Identify architectural and design patterns".to_string(),
            expected_outcome: "Architecture patterns and design principles understood".to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Architecture Analysis".to_string(),
            description: "Architectural pattern identification and analysis".to_string(),
            tools: vec!["detect_patterns".to_string()],
            success_indicators: vec!["Architecture patterns identified".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Architectural patterns and principles identified".to_string()],
        getting_started_guidance: "Analyze architectural patterns to understand system design"
            .to_string(),
        optimization_tips: vec!["Focus on main architectural patterns first".to_string()],
        common_pitfalls: vec![
            "Don't confuse implementation patterns with architectural patterns".to_string(),
        ],
        alternative_approaches: vec![
            "Use dependency analysis to understand component relationships".to_string(),
        ],
    })
}

/// Generate debugging workflow (simplified implementation)
fn generate_debugging_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 18,
        "comprehensive" => 50,
        _ => 30,
    };

    Ok(WorkflowRecommendation {
        goal: "debug_issue".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "search_symbols".to_string(),
            parameters: json!({"pattern": ".*"}),
            reasoning: "Find relevant symbols related to the issue".to_string(),
            expected_outcome: "Relevant code components identified".to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Issue Investigation".to_string(),
            description: "Systematic debugging and issue analysis".to_string(),
            tools: vec!["search_symbols".to_string()],
            success_indicators: vec!["Issue-related code identified".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Issue location and cause identified".to_string()],
        getting_started_guidance: "Start by identifying symbols related to the issue".to_string(),
        optimization_tips: vec!["Use specific search terms related to the issue".to_string()],
        common_pitfalls: vec!["Don't assume the issue is where it manifests".to_string()],
        alternative_approaches: vec!["Use trace_data_flow if issue is data-related".to_string()],
    })
}

/// Generate refactoring workflow (simplified implementation)
fn generate_refactoring_workflow(
    complexity: &str,
    time_budget: Option<u32>,
) -> Result<WorkflowRecommendation> {
    let base_duration = match complexity {
        "quick" => 20,
        "comprehensive" => 60,
        _ => 35,
    };

    Ok(WorkflowRecommendation {
        goal: "refactor_preparation".to_string(),
        estimated_duration: time_budget.unwrap_or(base_duration),
        tool_sequence: vec![ToolStep {
            step: 1,
            tool_name: "analyze_complexity".to_string(),
            parameters: json!({}),
            reasoning: "Identify complex code that needs refactoring".to_string(),
            expected_outcome: "Complex code areas identified for refactoring".to_string(),
            estimated_time_minutes: base_duration,
            priority: "high".to_string(),
            optional: false,
        }],
        stages: vec![WorkflowStageInfo {
            stage: "Refactoring Analysis".to_string(),
            description: "Code complexity analysis for refactoring planning".to_string(),
            tools: vec!["analyze_complexity".to_string()],
            success_indicators: vec!["Refactoring targets identified".to_string()],
        }],
        execution_type: "sequential".to_string(),
        parallel_groups: vec![],
        dependencies: vec![],
        success_criteria: vec!["Refactoring opportunities identified and prioritized".to_string()],
        getting_started_guidance: "Analyze code complexity to identify refactoring opportunities"
            .to_string(),
        optimization_tips: vec!["Focus on high-complexity, high-impact areas".to_string()],
        common_pitfalls: vec!["Don't refactor without understanding current behavior".to_string()],
        alternative_approaches: vec![
            "Use duplicate detection to find refactoring opportunities".to_string()
        ],
    })
}

/// Get session context for recommendations (placeholder implementation)
fn get_session_context(_session_id: SessionId) -> Result<SessionContext> {
    // In a real implementation, this would fetch from session manager
    Ok(SessionContext {
        current_stage: WorkflowStage::Discovery,
        completed_tools: vec![],
        recommendations: vec!["Start with repository overview".to_string()],
        progress_assessment: "Beginning analysis".to_string(),
    })
}
