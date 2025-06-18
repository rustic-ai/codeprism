//! Workflow context and guidance system
//! 
//! Provides intelligent tool suggestions and workflow guidance based on
//! current analysis state, session history, and detected patterns.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::session::{SessionState, WorkflowStage};

/// Confidence level for suggestions
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Suggestion for a specific tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuggestion {
    /// Name of the suggested tool
    pub tool_name: String,
    /// Suggested parameters
    pub suggested_parameters: HashMap<String, serde_json::Value>,
    /// Confidence in this suggestion
    pub confidence: ConfidenceLevel,
    /// Reason for the suggestion
    pub reasoning: String,
    /// Expected outcome
    pub expected_outcome: String,
    /// Priority (1 = highest)
    pub priority: u8,
}

impl ToolSuggestion {
    /// Create a new tool suggestion
    pub fn new(
        tool_name: String,
        confidence: ConfidenceLevel,
        reasoning: String,
        expected_outcome: String,
        priority: u8,
    ) -> Self {
        Self {
            tool_name,
            suggested_parameters: HashMap::new(),
            confidence,
            reasoning,
            expected_outcome,
            priority,
        }
    }

    /// Add a suggested parameter
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.suggested_parameters.insert(key, value);
        self
    }

    /// Add multiple suggested parameters
    pub fn with_parameters(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.suggested_parameters.extend(params);
        self
    }
}

/// Workflow guidance and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSuggestion {
    /// Current workflow stage
    pub current_stage: WorkflowStage,
    /// Suggested next stage
    pub next_stage: Option<WorkflowStage>,
    /// Immediate tool suggestions
    pub immediate_suggestions: Vec<ToolSuggestion>,
    /// Alternative approaches
    pub alternatives: Vec<ToolSuggestion>,
    /// Overall workflow guidance
    pub workflow_guidance: String,
    /// Progress assessment
    pub progress_assessment: String,
}

impl WorkflowSuggestion {
    /// Create workflow suggestion for a given stage
    pub fn for_stage(stage: WorkflowStage) -> Self {
        let next_stage = stage.next_stage();
        let workflow_guidance = Self::get_stage_guidance(&stage);
        let progress_assessment = Self::assess_stage_progress(&stage);

        Self {
            current_stage: stage,
            next_stage,
            immediate_suggestions: Vec::new(),
            alternatives: Vec::new(),
            workflow_guidance,
            progress_assessment,
        }
    }

    /// Get guidance text for a workflow stage
    fn get_stage_guidance(stage: &WorkflowStage) -> String {
        match stage {
            WorkflowStage::Discovery => {
                "Focus on understanding the overall structure and scope of the codebase. \
                Start with repository statistics and content exploration.".to_string()
            }
            WorkflowStage::Mapping => {
                "Map out the relationships between components. Use symbol search and \
                dependency analysis to understand the architecture.".to_string()
            }
            WorkflowStage::DeepDive => {
                "Dive deep into specific areas of interest. Explain individual symbols \
                and analyze complex patterns like inheritance hierarchies.".to_string()
            }
            WorkflowStage::Synthesis => {
                "Synthesize your findings with quality analysis. Look for complexity issues, \
                security concerns, and opportunities for improvement.".to_string()
            }
        }
    }

    /// Assess progress for a workflow stage
    fn assess_stage_progress(stage: &WorkflowStage) -> String {
        match stage {
            WorkflowStage::Discovery => {
                "Building initial understanding of the codebase structure.".to_string()
            }
            WorkflowStage::Mapping => {
                "Mapping relationships and understanding component interactions.".to_string()
            }
            WorkflowStage::DeepDive => {
                "Analyzing specific components and complex patterns in detail.".to_string()
            }
            WorkflowStage::Synthesis => {
                "Evaluating code quality and identifying improvement opportunities.".to_string()
            }
        }
    }

    /// Add an immediate suggestion
    pub fn add_suggestion(mut self, suggestion: ToolSuggestion) -> Self {
        self.immediate_suggestions.push(suggestion);
        self
    }

    /// Add an alternative suggestion
    pub fn add_alternative(mut self, suggestion: ToolSuggestion) -> Self {
        self.alternatives.push(suggestion);
        self
    }

    /// Sort suggestions by priority
    pub fn sort_by_priority(mut self) -> Self {
        self.immediate_suggestions.sort_by_key(|s| s.priority);
        self.alternatives.sort_by_key(|s| s.priority);
        self
    }
}

/// Workflow context that tracks current state and provides guidance
#[derive(Debug)]
pub struct WorkflowContext {
    /// Session state reference
    session_state: SessionState,
}

impl WorkflowContext {
    /// Create new workflow context from session state
    pub fn new(session_state: SessionState) -> Self {
        Self { session_state }
    }

    /// Generate workflow suggestions based on current state
    pub fn generate_suggestions(&self) -> Result<WorkflowSuggestion> {
        let stage = &self.session_state.current_stage;
        let workflow_guidance = match stage {
            WorkflowStage::Discovery => "Focus on understanding the overall structure and scope of the codebase.",
            WorkflowStage::Mapping => "Map relationships between components and understand architecture.",
            WorkflowStage::DeepDive => "Dive deep into specific areas of interest and analyze patterns.",
            WorkflowStage::Synthesis => "Synthesize findings with quality analysis and security review.",
        };

        // Generate basic suggestions based on the workflow stage
        let immediate_suggestions = match stage {
            WorkflowStage::Discovery => vec![
                ToolSuggestion::new(
                    "repository_stats".to_string(),
                    ConfidenceLevel::High,
                    "Get an overview of the repository structure".to_string(),
                    "Understanding of codebase size and organization".to_string(),
                    1,
                ),
            ],
            WorkflowStage::Mapping => vec![
                ToolSuggestion::new(
                    "search_symbols".to_string(),
                    ConfidenceLevel::High,
                    "Find key symbols in the codebase".to_string(),
                    "Discovery of main classes and functions".to_string(),
                    1,
                ),
            ],
            WorkflowStage::DeepDive => vec![
                ToolSuggestion::new(
                    "explain_symbol".to_string(),
                    ConfidenceLevel::High,
                    "Analyze specific symbols in detail".to_string(),
                    "Deep understanding of symbol implementation".to_string(),
                    1,
                ),
            ],
            WorkflowStage::Synthesis => vec![
                ToolSuggestion::new(
                    "analyze_complexity".to_string(),
                    ConfidenceLevel::High,
                    "Evaluate code complexity and quality".to_string(),
                    "Assessment of code maintainability".to_string(),
                    1,
                ),
            ],
        };

        Ok(WorkflowSuggestion {
            current_stage: stage.clone(),
            next_stage: stage.next_stage(),
            immediate_suggestions,
            alternatives: vec![],
            workflow_guidance: workflow_guidance.to_string(),
            progress_assessment: "Analysis in progress".to_string(),
        })
    }

    /// Get suggestions for a specific symbol
    pub fn suggest_for_symbol(&self, symbol_id: &str) -> Result<Vec<ToolSuggestion>> {
        let mut suggestions = Vec::new();

        // Always suggest explain_symbol for detailed understanding
        suggestions.push(
            ToolSuggestion::new(
                "explain_symbol".to_string(),
                ConfidenceLevel::High,
                format!("Get detailed explanation of symbol {}", symbol_id),
                "Complete understanding of symbol implementation and context".to_string(),
                1,
            ).with_parameter("symbol_id".to_string(), serde_json::Value::String(symbol_id.to_string()))
             .with_parameter("include_dependencies".to_string(), serde_json::Value::Bool(true))
             .with_parameter("include_usages".to_string(), serde_json::Value::Bool(true))
        );

        // Suggest finding references
        suggestions.push(
            ToolSuggestion::new(
                "find_references".to_string(),
                ConfidenceLevel::High,
                format!("Find all references to symbol {}", symbol_id),
                "Understanding of how and where the symbol is used".to_string(),
                2,
            ).with_parameter("symbol_id".to_string(), serde_json::Value::String(symbol_id.to_string()))
        );

        // Suggest dependency analysis
        suggestions.push(
            ToolSuggestion::new(
                "find_dependencies".to_string(),
                ConfidenceLevel::Medium,
                format!("Analyze dependencies for symbol {}", symbol_id),
                "Understanding of what the symbol depends on".to_string(),
                3,
            ).with_parameter("target".to_string(), serde_json::Value::String(symbol_id.to_string()))
        );

        Ok(suggestions)
    }

    /// Get alternative workflow approaches
    pub fn suggest_alternatives(&self) -> Result<Vec<ToolSuggestion>> {
        let current_stage = &self.session_state.current_stage;
        
        match current_stage {
            WorkflowStage::Discovery => Ok(vec![
                ToolSuggestion::new(
                    "search_content".to_string(),
                    ConfidenceLevel::Medium,
                    "Search for specific terms or concepts in the codebase".to_string(),
                    "Targeted discovery of relevant code sections".to_string(),
                    1,
                ),
            ]),
            WorkflowStage::Mapping => Ok(vec![
                ToolSuggestion::new(
                    "trace_path".to_string(),
                    ConfidenceLevel::Medium,
                    "Find connections between discovered symbols".to_string(),
                    "Understanding of execution paths and symbol relationships".to_string(),
                    1,
                ),
            ]),
            WorkflowStage::DeepDive => Ok(vec![
                ToolSuggestion::new(
                    "analyze_transitive_dependencies".to_string(),
                    ConfidenceLevel::Medium,
                    "Analyze complex dependency chains".to_string(),
                    "Understanding of indirect dependencies and potential cycles".to_string(),
                    1,
                ),
            ]),
            WorkflowStage::Synthesis => Ok(vec![
                ToolSuggestion::new(
                    "analyze_performance".to_string(),
                    ConfidenceLevel::Medium,
                    "Evaluate performance characteristics of the code".to_string(),
                    "Performance assessment and optimization opportunities".to_string(),
                    1,
                ),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::session::SessionState;

    #[test]
    fn test_tool_suggestion_creation() {
        let suggestion = ToolSuggestion::new(
            "test_tool".to_string(),
            ConfidenceLevel::High,
            "Test reasoning".to_string(),
            "Test outcome".to_string(),
            1,
        ).with_parameter("param1".to_string(), serde_json::Value::String("value1".to_string()));

        assert_eq!(suggestion.tool_name, "test_tool");
        assert_eq!(suggestion.confidence, ConfidenceLevel::High);
        assert_eq!(suggestion.priority, 1);
        assert!(suggestion.suggested_parameters.contains_key("param1"));
    }

    #[test]
    fn test_workflow_suggestion_creation() {
        let suggestion = WorkflowSuggestion::for_stage(WorkflowStage::Discovery);
        assert_eq!(suggestion.current_stage, WorkflowStage::Discovery);
        assert_eq!(suggestion.next_stage, Some(WorkflowStage::Mapping));
        assert!(!suggestion.workflow_guidance.is_empty());
    }

    #[test]
    fn test_workflow_context() {
        let session = SessionState::new();
        let context = WorkflowContext::new(session);
        
        let suggestions = context.generate_suggestions().unwrap();
        assert_eq!(suggestions.current_stage, WorkflowStage::Discovery);
        assert!(!suggestions.immediate_suggestions.is_empty());
    }

    #[test]
    fn test_symbol_suggestions() {
        let session = SessionState::new();
        let context = WorkflowContext::new(session);
        
        let suggestions = context.suggest_for_symbol("test_symbol").unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.tool_name == "explain_symbol"));
    }

    #[test]
    fn test_confidence_ordering() {
        assert!(ConfidenceLevel::High > ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium > ConfidenceLevel::Low);
    }
} 