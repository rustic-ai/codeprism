//! MCP Prompts implementation
//!
//! Prompts allow servers to provide structured messages and instructions for
//! interacting with language models. Clients can discover available prompts,
//! retrieve their contents, and provide arguments to customize them.

use crate::CodePrismMcpServer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Prompt capabilities as defined by MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    /// Whether the server will emit notifications when the list of available prompts changes
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub name: String,
    /// Optional human-readable title for display purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description of the prompt
    pub description: String,
    /// Optional arguments schema for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether the argument is required
    pub required: bool,
}

/// Prompt message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Role of the message (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: PromptContent,
}

/// Prompt content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type of the image
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Parameters for getting a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// Name of the prompt to get
    pub name: String,
    /// Optional arguments for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Map<String, Value>>,
}

/// Result of getting a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Messages that make up the prompt
    pub messages: Vec<PromptMessage>,
}

/// Parameters for listing prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Result of listing prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// List of available prompts
    pub prompts: Vec<Prompt>,
    /// Optional cursor for pagination
    #[serde(rename = "nextCursor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Prompt manager for MCP server
pub struct PromptManager {
    server: std::sync::Arc<tokio::sync::RwLock<CodePrismMcpServer>>,
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<CodePrismMcpServer>>) -> Self {
        Self { server }
    }

    /// List available prompts
    pub async fn list_prompts(&self, _params: ListPromptsParams) -> Result<ListPromptsResult> {
        let prompts = vec![
            Prompt {
                name: "repository_overview".to_string(),
                title: Some("Repository Overview".to_string()),
                description: "Generate a comprehensive overview of the repository structure and contents".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "focus_area".to_string(),
                        description: "Optional area to focus on (architecture, dependencies, entry_points, etc.)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "code_analysis".to_string(),
                title: Some("Code Analysis".to_string()),
                description: "Analyze code quality, patterns, and potential improvements".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "file_pattern".to_string(),
                        description: "Optional file pattern to focus analysis on".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "analysis_type".to_string(),
                        description: "Type of analysis (quality, security, performance, architecture)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "debug_assistance".to_string(),
                title: Some("Debug Assistance".to_string()),
                description: "Help debug issues in the codebase with contextual information".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "issue_description".to_string(),
                        description: "Description of the issue or error".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "affected_files".to_string(),
                        description: "Files related to the issue".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "debug_issue".to_string(),
                title: Some("Debug Issue".to_string()),
                description: "Analyze potential bug sources and dependencies for debugging".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "error_location".to_string(),
                        description: "File and line where error occurs".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "error_message".to_string(),
                        description: "Error message or description".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "refactoring_guidance".to_string(),
                title: Some("Refactoring Guidance".to_string()),
                description: "Provide guidance for refactoring code with repository context".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "target_area".to_string(),
                        description: "Area of code to refactor".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "refactoring_goal".to_string(),
                        description: "Goal of the refactoring (performance, maintainability, etc.)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "architectural_analysis".to_string(),
                title: Some("Architectural Analysis".to_string()),
                description: "Analyze the overall architecture, patterns, and structural design of the codebase".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "analysis_focus".to_string(),
                        description: "Focus area for analysis (layers, patterns, dependencies, coupling)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "architectural_concerns".to_string(),
                        description: "Specific architectural concerns or questions".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "pattern_assessment".to_string(),
                title: Some("Design Pattern Assessment".to_string()),
                description: "Assess design patterns usage and suggest architectural improvements".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "pattern_types".to_string(),
                        description: "Types of patterns to focus on (design_patterns, anti_patterns, architectural_patterns)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "improvement_scope".to_string(),
                        description: "Scope of improvements to suggest (immediate, long-term, strategic)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "dependency_analysis".to_string(),
                title: Some("Dependency Analysis".to_string()),
                description: "Analyze dependencies, coupling, and suggest decoupling strategies".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "analysis_target".to_string(),
                        description: "Target for analysis (file, class, module, or repository-wide)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "dependency_concerns".to_string(),
                        description: "Specific dependency concerns (cycles, coupling, cohesion)".to_string(),
                        required: false,
                    },
                ]),
            },

            // New prompt templates for large codebase understanding
            Prompt {
                name: "new_developer_onboarding".to_string(),
                title: Some("New Developer Onboarding".to_string()),
                description: "Create a comprehensive onboarding guide for new developers joining the project".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "developer_experience".to_string(),
                        description: "Developer's experience level (junior, mid, senior)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "focus_areas".to_string(),
                        description: "Areas the developer should focus on first (frontend, backend, infrastructure, etc.)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "timeline".to_string(),
                        description: "Onboarding timeline (1-week, 2-week, 1-month)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "feature_exploration".to_string(),
                title: Some("Feature Exploration".to_string()),
                description: "Deep dive into understanding a specific feature or module in the codebase".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "feature_name".to_string(),
                        description: "Name or description of the feature to explore".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "exploration_depth".to_string(),
                        description: "Depth of exploration (overview, detailed, implementation)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "include_tests".to_string(),
                        description: "Whether to include test coverage analysis".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "learning_path_generator".to_string(),
                title: Some("Learning Path Generator".to_string()),
                description: "Generate a guided learning path through the codebase for specific goals".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "learning_goal".to_string(),
                        description: "What the user wants to learn or achieve".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "current_knowledge".to_string(),
                        description: "User's current knowledge level with the codebase".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "time_constraint".to_string(),
                        description: "Available time for learning (1-day, 1-week, 1-month)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "technology_stack_analysis".to_string(),
                title: Some("Technology Stack Analysis".to_string()),
                description: "Analyze the technology stack, frameworks, and architectural decisions".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "analysis_focus".to_string(),
                        description: "Focus area (languages, frameworks, tools, infrastructure)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "include_alternatives".to_string(),
                        description: "Whether to suggest alternative technologies".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "codebase_health_check".to_string(),
                title: Some("Codebase Health Check".to_string()),
                description: "Comprehensive health assessment of the codebase including quality, maintainability, and technical debt".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "health_aspects".to_string(),
                        description: "Aspects to focus on (code_quality, test_coverage, security, performance, maintainability)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "severity_threshold".to_string(),
                        description: "Minimum severity level for issues to report (low, medium, high, critical)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "documentation_generator".to_string(),
                title: Some("Documentation Generator".to_string()),
                description: "Generate comprehensive documentation for the codebase or specific components".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "documentation_type".to_string(),
                        description: "Type of documentation (API, architecture, developer_guide, user_manual)".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "target_audience".to_string(),
                        description: "Target audience (developers, users, architects, managers)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "scope".to_string(),
                        description: "Scope of documentation (full_repository, specific_module, specific_feature)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "testing_strategy_analysis".to_string(),
                title: Some("Testing Strategy Analysis".to_string()),
                description: "Analyze current testing approach and suggest improvements for test coverage and quality".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "test_types".to_string(),
                        description: "Types of tests to analyze (unit, integration, e2e, performance)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "coverage_threshold".to_string(),
                        description: "Minimum coverage threshold to aim for (percentage)".to_string(),
                        required: false,
                    },
                ]),
            },

            Prompt {
                name: "migration_planning".to_string(),
                title: Some("Migration Planning".to_string()),
                description: "Plan migration strategies for legacy code, frameworks, or architectural changes".to_string(),
                arguments: Some(vec![
                    PromptArgument {
                        name: "migration_type".to_string(),
                        description: "Type of migration (framework_upgrade, language_migration, architecture_change)".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "target_technology".to_string(),
                        description: "Target technology or framework to migrate to".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "risk_tolerance".to_string(),
                        description: "Risk tolerance level (low, medium, high)".to_string(),
                        required: false,
                    },
                ]),
            },
        ];

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
        })
    }

    /// Get a specific prompt
    pub async fn get_prompt(&self, params: GetPromptParams) -> Result<GetPromptResult> {
        let server = self.server.read().await;

        match params.name.as_str() {
            "repository_overview" => {
                self.repository_overview_prompt(&server, params.arguments)
                    .await
            }
            "code_analysis" => self.code_analysis_prompt(&server, params.arguments).await,
            "debug_assistance" => {
                self.debug_assistance_prompt(&server, params.arguments)
                    .await
            }
            "debug_issue" => self.debug_issue_prompt(&server, params.arguments).await,
            "refactoring_guidance" => {
                self.refactoring_guidance_prompt(&server, params.arguments)
                    .await
            }
            "architectural_analysis" => {
                self.architectural_analysis_prompt(&server, params.arguments)
                    .await
            }
            "pattern_assessment" => {
                self.pattern_assessment_prompt(&server, params.arguments)
                    .await
            }
            "dependency_analysis" => {
                self.dependency_analysis_prompt(&server, params.arguments)
                    .await
            }
            "new_developer_onboarding" => {
                self.new_developer_onboarding_prompt(&server, params.arguments)
                    .await
            }
            "feature_exploration" => {
                self.feature_exploration_prompt(&server, params.arguments)
                    .await
            }
            "learning_path_generator" => {
                self.learning_path_generator_prompt(&server, params.arguments)
                    .await
            }
            "technology_stack_analysis" => {
                self.technology_stack_analysis_prompt(&server, params.arguments)
                    .await
            }
            "codebase_health_check" => {
                self.codebase_health_check_prompt(&server, params.arguments)
                    .await
            }
            "documentation_generator" => {
                self.documentation_generator_prompt(&server, params.arguments)
                    .await
            }
            "testing_strategy_analysis" => {
                self.testing_strategy_analysis_prompt(&server, params.arguments)
                    .await
            }
            "migration_planning" => {
                self.migration_planning_prompt(&server, params.arguments)
                    .await
            }
            _ => Err(anyhow::anyhow!("Unknown prompt: {}", params.name)),
        }
    }

    /// Generate repository overview prompt
    async fn repository_overview_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let focus_area = arguments
            .as_ref()
            .and_then(|args| args.get("focus_area"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            let file_count = server
                .scanner()
                .discover_files(repo_path)
                .map(|files| files.len())
                .unwrap_or(0);

            format!(
                "Repository: {}\nTotal files: {}\nFocus area: {}",
                repo_path.display(),
                file_count,
                focus_area
            )
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please provide a comprehensive overview of this repository with the following context:

{}

Please analyze and provide:
1. Repository structure and organization
2. Main technologies and frameworks used
3. Key entry points and important files
4. Dependencies and external libraries
5. Code patterns and architectural decisions
6. Areas for potential improvement

Focus particularly on: {}

Use the repository resources and tools available to gather detailed information about the codebase."#,
            repo_context, focus_area
        );

        Ok(GetPromptResult {
            description: Some("Repository overview and analysis prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate code analysis prompt
    async fn code_analysis_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let file_pattern = arguments
            .as_ref()
            .and_then(|args| args.get("file_pattern"))
            .and_then(|v| v.as_str())
            .unwrap_or("*");

        let analysis_type = arguments
            .as_ref()
            .and_then(|args| args.get("analysis_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("quality");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please perform a {} analysis of the codebase with the following context:

{}

File pattern: {}
Analysis focus: {}

Please analyze and provide:
1. Code quality assessment
2. Potential issues and vulnerabilities
3. Best practices compliance
4. Performance considerations
5. Maintainability factors
6. Specific recommendations for improvement

Use the available tools to gather detailed information about the code structure and patterns."#,
            analysis_type, repo_context, file_pattern, analysis_type
        );

        Ok(GetPromptResult {
            description: Some("Code analysis and quality assessment prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate debug assistance prompt
    async fn debug_assistance_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let issue_description = arguments
            .as_ref()
            .and_then(|args| args.get("issue_description"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("issue_description argument is required"))?;

        let affected_files = arguments
            .as_ref()
            .and_then(|args| args.get("affected_files"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let prompt_text = format!(
            r#"I need help debugging the following issue:

Issue Description: {}
Affected Files: {}

Please help me:
1. Understand the root cause of this issue
2. Identify related code that might be contributing to the problem
3. Suggest debugging steps and approaches
4. Provide potential solutions or workarounds
5. Recommend best practices to prevent similar issues

Use the repository tools to examine the relevant code and dependencies."#,
            issue_description, affected_files
        );

        Ok(GetPromptResult {
            description: Some("Debug assistance with repository context".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate debug issue prompt
    async fn debug_issue_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let error_location = arguments
            .as_ref()
            .and_then(|args| args.get("error_location"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("error_location argument is required"))?;

        let error_message = arguments
            .as_ref()
            .and_then(|args| args.get("error_message"))
            .and_then(|v| v.as_str())
            .unwrap_or("No error message provided");

        let prompt_text = format!(
            r#"Please analyze the following error:

Error Location: {}
Error Message: {}

Please provide:
1. Analysis of the error's source and potential impact
2. Suggestions for debugging and resolving the issue
3. Recommendations for preventing similar errors in the future

Use the repository tools to understand the code and dependencies related to this error."#,
            error_location, error_message
        );

        Ok(GetPromptResult {
            description: Some("Debug issue analysis prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate refactoring guidance prompt
    async fn refactoring_guidance_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let target_area = arguments
            .as_ref()
            .and_then(|args| args.get("target_area"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("target_area argument is required"))?;

        let refactoring_goal = arguments
            .as_ref()
            .and_then(|args| args.get("refactoring_goal"))
            .and_then(|v| v.as_str())
            .unwrap_or("improve maintainability");

        let prompt_text = format!(
            r#"I need guidance for refactoring the following area of the codebase:

Target Area: {}
Refactoring Goal: {}

Please provide:
1. Analysis of the current code structure and patterns
2. Identification of code smells or areas for improvement
3. Recommended refactoring approach and strategy
4. Step-by-step refactoring plan
5. Potential risks and mitigation strategies
6. Testing considerations during refactoring

Use the repository tools to understand the current implementation and its dependencies."#,
            target_area, refactoring_goal
        );

        Ok(GetPromptResult {
            description: Some("Refactoring guidance with repository analysis".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate architectural analysis prompt
    async fn architectural_analysis_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let analysis_focus = arguments
            .as_ref()
            .and_then(|args| args.get("analysis_focus"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt_text = format!(
            r#"Please analyze the overall architecture, patterns, and structural design of the codebase with the following focus:

{}

Please provide:
1. Analysis of the current architecture
2. Identification of architectural concerns or questions
3. Suggestions for architectural improvements
4. Step-by-step plan for implementing improvements
5. Potential risks and mitigation strategies
6. Testing considerations during architectural changes

Use the repository tools to understand the current implementation and its dependencies."#,
            analysis_focus
        );

        Ok(GetPromptResult {
            description: Some("Architectural analysis prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate pattern assessment prompt
    async fn pattern_assessment_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let pattern_types = arguments
            .as_ref()
            .and_then(|args| args.get("pattern_types"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let _improvement_scope = arguments
            .as_ref()
            .and_then(|args| args.get("improvement_scope"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt_text = format!(
            r#"Please assess the usage of design patterns in the codebase with the following focus:

{}

Please provide:
1. Analysis of the current pattern usage
2. Identification of patterns that could be improved
3. Suggestions for architectural improvements based on pattern usage
4. Step-by-step plan for implementing improvements
5. Potential risks and mitigation strategies
6. Testing considerations during architectural changes

Use the repository tools to understand the current implementation and its dependencies."#,
            pattern_types
        );

        Ok(GetPromptResult {
            description: Some("Design pattern assessment prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate dependency analysis prompt
    async fn dependency_analysis_prompt(
        &self,
        _server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let _analysis_target = arguments
            .as_ref()
            .and_then(|args| args.get("analysis_target"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let dependency_concerns = arguments
            .as_ref()
            .and_then(|args| args.get("dependency_concerns"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt_text = format!(
            r#"Please analyze dependencies, coupling, and suggest decoupling strategies with the following focus:

{}

Please provide:
1. Analysis of the current dependency structure
2. Identification of dependency concerns
3. Suggestions for decoupling strategies
4. Step-by-step plan for implementing improvements
5. Potential risks and mitigation strategies
6. Testing considerations during architectural changes

Use the repository tools to understand the current implementation and its dependencies."#,
            dependency_concerns
        );

        Ok(GetPromptResult {
            description: Some("Dependency analysis prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate new developer onboarding prompt
    async fn new_developer_onboarding_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let developer_experience = arguments
            .as_ref()
            .and_then(|args| args.get("developer_experience"))
            .and_then(|v| v.as_str())
            .unwrap_or("mid");

        let focus_areas = arguments
            .as_ref()
            .and_then(|args| args.get("focus_areas"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let timeline = arguments
            .as_ref()
            .and_then(|args| args.get("timeline"))
            .and_then(|v| v.as_str())
            .unwrap_or("2-week");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please create a comprehensive onboarding guide for a new {} developer joining this project:

{}

Developer experience level: {}
Focus areas: {}
Onboarding timeline: {}

Please provide:
1. **Repository Overview**: Structure, key directories, and important files
2. **Technology Stack**: Languages, frameworks, tools, and dependencies
3. **Development Environment Setup**: Step-by-step setup instructions
4. **Architecture Understanding**: High-level architecture and design patterns
5. **Key Features Walkthrough**: Main features and how they work
6. **Development Workflow**: Coding standards, testing, and deployment processes
7. **First Tasks**: Suggested starter tasks to get familiar with the codebase
8. **Learning Resources**: Documentation, wiki, and helpful references
9. **Team Contacts**: Who to contact for different areas/questions
10. **Success Metrics**: How to measure onboarding progress

Use repository analysis tools to gather detailed information about:
- File structure and organization
- Dependencies and external libraries
- Code patterns and conventions
- Test coverage and quality
- Documentation availability

Tailor the guide to the developer's experience level and focus areas."#,
            developer_experience, repo_context, developer_experience, focus_areas, timeline
        );

        Ok(GetPromptResult {
            description: Some("Comprehensive new developer onboarding guide".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate feature exploration prompt
    async fn feature_exploration_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let feature_name = arguments
            .as_ref()
            .and_then(|args| args.get("feature_name"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("feature_name argument is required"))?;

        let exploration_depth = arguments
            .as_ref()
            .and_then(|args| args.get("exploration_depth"))
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        let include_tests = arguments
            .as_ref()
            .and_then(|args| args.get("include_tests"))
            .and_then(|v| v.as_str())
            .unwrap_or("true");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please provide a comprehensive exploration of the '{}' feature in this codebase:

{}

Exploration depth: {}
Include test analysis: {}

Please analyze and provide:

1. **Feature Overview**:
   - Purpose and functionality of the feature
   - Business value and user impact
   - High-level architecture

2. **Code Structure**:
   - Main files and directories related to the feature
   - Key classes, functions, and modules
   - Entry points and public APIs

3. **Implementation Details**:
   - Core algorithms and logic
   - Data models and structures
   - External dependencies and integrations

4. **Dependencies and Relationships**:
   - Dependencies on other features/modules
   - Features that depend on this one
   - External library dependencies

5. **Data Flow**:
   - How data flows through the feature
   - Input/output interfaces
   - State management

6. **Configuration and Settings**:
   - Configuration options
   - Environment variables
   - Feature flags or toggles

{}

7. **Documentation and Comments**:
   - Existing documentation
   - Code comments and explanations
   - Missing documentation areas

8. **Potential Issues and Improvements**:
   - Code quality concerns
   - Performance considerations
   - Security implications
   - Refactoring opportunities

Use repository analysis tools to:
- Search for relevant files and symbols
- Analyze dependencies and references
- Examine code patterns and complexity
- Trace execution paths

Provide specific file paths, function names, and code examples where relevant."#,
            feature_name,
            repo_context,
            exploration_depth,
            include_tests,
            if include_tests == "true" {
                "\n7. **Test Coverage**:\n   - Existing test files and test cases\n   - Test coverage analysis\n   - Test quality and completeness\n   - Missing test scenarios"
            } else {
                ""
            }
        );

        Ok(GetPromptResult {
            description: Some("Comprehensive feature exploration and analysis".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate learning path generator prompt
    async fn learning_path_generator_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let learning_goal = arguments
            .as_ref()
            .and_then(|args| args.get("learning_goal"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("learning_goal argument is required"))?;

        let current_knowledge = arguments
            .as_ref()
            .and_then(|args| args.get("current_knowledge"))
            .and_then(|v| v.as_str())
            .unwrap_or("beginner");

        let time_constraint = arguments
            .as_ref()
            .and_then(|args| args.get("time_constraint"))
            .and_then(|v| v.as_str())
            .unwrap_or("1-week");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please create a personalized learning path for the following goal:

**Learning Goal**: {}
**Current Knowledge Level**: {}
**Time Available**: {}
**Repository Context**: {}

Please create a structured learning path that includes:

1. **Learning Objectives**:
   - Specific, measurable goals
   - Prerequisites and assumptions
   - Success criteria

2. **Phased Learning Plan**:
   - Phase 1: Foundation (understanding basics)
   - Phase 2: Core Concepts (deeper understanding)
   - Phase 3: Advanced Topics (mastery and application)
   - Phase 4: Practical Application (hands-on work)

3. **Daily/Weekly Schedule**:
   - Time allocation for each phase
   - Daily learning tasks
   - Milestone checkpoints

4. **Resource Identification**:
   - Key files to study (prioritized list)
   - Important functions/classes to understand
   - Relevant documentation and comments
   - External resources if needed

5. **Hands-on Exercises**:
   - Specific code reading exercises
   - Small modification tasks
   - Debugging exercises
   - Code tracing activities

6. **Assessment Methods**:
   - Self-assessment questions
   - Practical challenges
   - Code review criteria

7. **Common Pitfalls and Tips**:
   - Areas that are typically confusing
   - Best practices to follow
   - Debugging strategies

Use repository analysis tools to:
- Identify the most relevant files and functions for the learning goal
- Map dependencies and relationships
- Assess code complexity to sequence learning appropriately
- Find examples and test cases

Adjust the complexity and pace based on the current knowledge level and time constraints."#,
            learning_goal, current_knowledge, time_constraint, repo_context
        );

        Ok(GetPromptResult {
            description: Some("Personalized learning path through the codebase".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate technology stack analysis prompt
    async fn technology_stack_analysis_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let analysis_focus = arguments
            .as_ref()
            .and_then(|args| args.get("analysis_focus"))
            .and_then(|v| v.as_str())
            .unwrap_or("comprehensive");

        let include_alternatives = arguments
            .as_ref()
            .and_then(|args| args.get("include_alternatives"))
            .and_then(|v| v.as_str())
            .unwrap_or("false");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please analyze the technology stack and architectural decisions in this codebase:

{}

Analysis focus: {}
Include alternatives: {}

Please provide a comprehensive analysis covering:

1. **Programming Languages**:
   - Primary and secondary languages used
   - Language versions and compatibility
   - Language-specific patterns and idioms

2. **Frameworks and Libraries**:
   - Web frameworks (if applicable)
   - Testing frameworks
   - Utility libraries and their purposes
   - Version information and update status

3. **Development Tools**:
   - Build systems and task runners
   - Package managers and dependency management
   - Code quality tools (linters, formatters)
   - Development environment setup

4. **Infrastructure and Deployment**:
   - Deployment strategies and tools
   - Containerization (Docker, etc.)
   - CI/CD pipeline tools
   - Cloud services and platforms

5. **Data Storage and Management**:
   - Database technologies
   - Data modeling approaches
   - Caching strategies
   - Data migration tools

6. **Architecture Patterns**:
   - Architectural styles (MVC, microservices, etc.)
   - Design patterns in use
   - Communication patterns (REST, GraphQL, etc.)
   - Event handling and messaging

7. **Security Technologies**:
   - Authentication and authorization frameworks
   - Security libraries and tools
   - Encryption and hashing methods

8. **Performance and Monitoring**:
   - Performance monitoring tools
   - Logging frameworks
   - Metrics and analytics tools

{}

9. **Technology Assessment**:
   - Strengths of current technology choices
   - Potential limitations or technical debt
   - Compatibility and integration concerns
   - Maintenance and support considerations

10. **Recommendations**:
    - Technology upgrade suggestions
    - Missing tools or frameworks
    - Best practices alignment
    - Future-proofing considerations

Use repository analysis tools to:
- Examine configuration files (package.json, requirements.txt, etc.)
- Analyze import statements and dependencies
- Review build and deployment scripts
- Assess code patterns and conventions

Provide specific examples and file references where relevant."#,
            repo_context,
            analysis_focus,
            include_alternatives,
            if include_alternatives == "true" {
                "\n8. **Alternative Technology Considerations**:\n   - Alternative frameworks or libraries\n   - Trade-offs and migration considerations\n   - Emerging technologies relevant to the project\n   - Cost-benefit analysis of alternatives"
            } else {
                ""
            }
        );

        Ok(GetPromptResult {
            description: Some("Comprehensive technology stack analysis".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate codebase health check prompt
    async fn codebase_health_check_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let health_aspects = arguments
            .as_ref()
            .and_then(|args| args.get("health_aspects"))
            .and_then(|v| v.as_str())
            .unwrap_or("comprehensive");

        let severity_threshold = arguments
            .as_ref()
            .and_then(|args| args.get("severity_threshold"))
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please perform a comprehensive health assessment of this codebase:

{}

Health aspects to focus on: {}
Minimum severity threshold: {}

Please analyze and provide:

1. **Code Quality Metrics**:
   - Code complexity analysis (cyclomatic, cognitive)
   - Code duplication detection
   - Code style and consistency
   - Documentation coverage

2. **Maintainability Assessment**:
   - Code organization and structure
   - Naming conventions and clarity
   - Function and class size appropriateness
   - Coupling and cohesion analysis

3. **Technical Debt Analysis**:
   - Identified code smells and anti-patterns
   - Legacy code sections requiring attention
   - Architecture inconsistencies
   - Performance bottlenecks

4. **Security Assessment**:
   - Security vulnerability scan
   - Authentication and authorization review
   - Data handling and privacy concerns
   - Dependency security analysis

5. **Test Coverage and Quality**:
   - Test coverage percentage and gaps
   - Test quality and effectiveness
   - Test maintenance issues
   - Testing strategy evaluation

6. **Dependency Management**:
   - Outdated dependencies and security risks
   - Dependency conflicts and compatibility
   - Unused dependencies
   - License compliance review

7. **Performance Indicators**:
   - Performance hot spots and bottlenecks
   - Memory usage patterns
   - Algorithmic complexity issues
   - Scalability concerns

8. **Documentation Health**:
   - API documentation completeness
   - Code comment quality and coverage
   - Architecture documentation
   - User and developer guides

9. **Development Process Health**:
   - Build system reliability
   - CI/CD pipeline effectiveness
   - Code review practices
   - Version control hygiene

10. **Overall Health Score**:
    - Weighted health score (0-100)
    - Critical issues requiring immediate attention
    - Priority improvement recommendations
    - Long-term health strategy

Use repository analysis tools to:
- Calculate complexity metrics
- Detect code duplicates and patterns
- Analyze dependencies and vulnerabilities
- Assess test coverage and quality
- Examine security patterns

Provide specific examples, file paths, and actionable recommendations.
Focus on issues at or above the '{}' severity threshold."#,
            repo_context, health_aspects, severity_threshold, severity_threshold
        );

        Ok(GetPromptResult {
            description: Some("Comprehensive codebase health assessment".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate documentation generator prompt
    async fn documentation_generator_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let documentation_type = arguments
            .as_ref()
            .and_then(|args| args.get("documentation_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("documentation_type argument is required"))?;

        let target_audience = arguments
            .as_ref()
            .and_then(|args| args.get("target_audience"))
            .and_then(|v| v.as_str())
            .unwrap_or("developers");

        let scope = arguments
            .as_ref()
            .and_then(|args| args.get("scope"))
            .and_then(|v| v.as_str())
            .unwrap_or("full_repository");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let (doc_sections, specific_guidance) = match documentation_type {
            "API" => (
                vec![
                    "API Overview and Purpose",
                    "Authentication and Authorization",
                    "Endpoint Documentation",
                    "Request/Response Schemas",
                    "Error Handling",
                    "Rate Limiting and Pagination",
                    "Code Examples and SDKs",
                    "Versioning and Compatibility"
                ],
                "Focus on public APIs, endpoints, data models, and integration examples. Include OpenAPI/Swagger specs if applicable."
            ),
            "architecture" => (
                vec![
                    "System Overview and Context",
                    "Architecture Diagrams",
                    "Component Descriptions",
                    "Data Flow and Interactions",
                    "Design Patterns and Principles",
                    "Technology Stack",
                    "Deployment Architecture",
                    "Security Architecture"
                ],
                "Focus on high-level system design, component interactions, and architectural decisions."
            ),
            "developer_guide" => (
                vec![
                    "Getting Started Guide",
                    "Development Environment Setup",
                    "Project Structure",
                    "Coding Standards and Guidelines",
                    "Build and Deployment Process",
                    "Testing Guidelines",
                    "Debugging and Troubleshooting",
                    "Contributing Guidelines"
                ],
                "Focus on practical information for developers working on the codebase."
            ),
            "user_manual" => (
                vec![
                    "Introduction and Overview",
                    "Installation Instructions",
                    "Basic Usage and Tutorials",
                    "Feature Documentation",
                    "Configuration Options",
                    "Troubleshooting Guide",
                    "FAQ and Common Issues",
                    "Support and Resources"
                ],
                "Focus on end-user perspective, practical usage, and problem-solving."
            ),
            _ => (
                vec![
                    "Project Overview",
                    "Architecture and Design",
                    "API Documentation",
                    "Development Guide",
                    "User Instructions"
                ],
                "Provide comprehensive documentation covering all aspects."
            )
        };

        let sections_text = doc_sections
            .iter()
            .enumerate()
            .map(|(i, section)| format!("{}. **{}**", i + 1, section))
            .collect::<Vec<_>>()
            .join("\n   ");

        let prompt_text = format!(
            r#"Please generate comprehensive {} documentation for this codebase:

{}

Documentation type: {}
Target audience: {}
Scope: {}

Please create documentation with the following structure:

   {}

**Specific Guidance**: {}

**Documentation Requirements**:
- Clear, concise, and well-structured content
- Appropriate level of technical detail for the target audience
- Code examples and practical usage scenarios
- Visual aids where helpful (diagrams, flowcharts)
- Cross-references and navigation aids
- Up-to-date and accurate information

**Content Guidelines**:
- Use clear headings and subheadings
- Include table of contents for longer documents
- Provide code examples with explanations
- Include troubleshooting sections where relevant
- Add links to related resources
- Use consistent formatting and style

Use repository analysis tools to:
- Examine code structure and organization
- Extract API definitions and schemas
- Analyze dependencies and integrations
- Review existing documentation and comments
- Understand data models and interfaces
- Identify key features and functionalities

Ensure the documentation is:
- Accurate and reflects the current codebase
- Complete within the specified scope
- Appropriately detailed for the target audience
- Professionally formatted and easy to navigate
- Include specific file paths and code references where helpful"#,
            documentation_type,
            repo_context,
            documentation_type,
            target_audience,
            scope,
            sections_text,
            specific_guidance
        );

        Ok(GetPromptResult {
            description: Some(format!(
                "Generate {} documentation for {}",
                documentation_type, target_audience
            )),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate testing strategy analysis prompt
    async fn testing_strategy_analysis_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let test_types = arguments
            .as_ref()
            .and_then(|args| args.get("test_types"))
            .and_then(|v| v.as_str())
            .unwrap_or("comprehensive");

        let coverage_threshold = arguments
            .as_ref()
            .and_then(|args| args.get("coverage_threshold"))
            .and_then(|v| v.as_str())
            .unwrap_or("80");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let prompt_text = format!(
            r#"Please analyze the current testing strategy and provide recommendations for improvement:

{}

Test types to analyze: {}
Target coverage threshold: {}%

Please provide a comprehensive testing analysis:

1. **Current Testing Overview**:
   - Existing test files and structure
   - Testing frameworks and tools in use
   - Test types currently implemented
   - Overall test coverage statistics

2. **Test Coverage Analysis**:
   - Line coverage, branch coverage, function coverage
   - Coverage gaps and untested areas
   - Critical paths without adequate testing
   - High-risk areas requiring more tests

3. **Test Quality Assessment**:
   - Test case effectiveness and completeness
   - Test maintenance and readability
   - Test data management and setup
   - Assertion quality and specificity

4. **Testing Strategy Evaluation**:
   - Unit testing strategy and coverage
   - Integration testing approach
   - End-to-end testing implementation
   - Performance and load testing
   - Security testing considerations

5. **Test Organization and Structure**:
   - Test file organization and naming
   - Test helper utilities and shared code
   - Test configuration and environment setup
   - Test data management strategies

6. **Testing Tools and Infrastructure**:
   - Testing framework effectiveness
   - CI/CD integration and automation
   - Test reporting and metrics
   - Test environment management

7. **Testing Best Practices Compliance**:
   - Test isolation and independence
   - Test naming and documentation
   - Mock and stub usage
   - Test pyramid adherence

8. **Identified Issues and Gaps**:
   - Missing test scenarios
   - Flaky or unreliable tests
   - Slow or inefficient tests
   - Maintenance bottlenecks

9. **Improvement Recommendations**:
   - Prioritized list of testing improvements
   - New test types to implement
   - Testing tool upgrades or changes
   - Process and workflow improvements

10. **Implementation Roadmap**:
    - Short-term quick wins
    - Medium-term strategic improvements
    - Long-term testing vision
    - Resource and time estimates

Use repository analysis tools to:
- Identify all test files and patterns
- Analyze test coverage and gaps
- Examine testing frameworks and dependencies
- Review test organization and structure
- Assess test complexity and maintainability

Provide specific examples, file paths, and actionable recommendations to reach the {}% coverage threshold."#,
            repo_context, test_types, coverage_threshold, coverage_threshold
        );

        Ok(GetPromptResult {
            description: Some(
                "Comprehensive testing strategy analysis and recommendations".to_string(),
            ),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }

    /// Generate migration planning prompt
    async fn migration_planning_prompt(
        &self,
        server: &CodePrismMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let migration_type = arguments
            .as_ref()
            .and_then(|args| args.get("migration_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("migration_type argument is required"))?;

        let target_technology = arguments
            .as_ref()
            .and_then(|args| args.get("target_technology"))
            .and_then(|v| v.as_str())
            .unwrap_or("not specified");

        let risk_tolerance = arguments
            .as_ref()
            .and_then(|args| args.get("risk_tolerance"))
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            format!("Repository: {}", repo_path.display())
        } else {
            "No repository currently loaded".to_string()
        };

        let (migration_focus, specific_considerations) = match migration_type {
            "framework_upgrade" => (
                "Framework Version Upgrade",
                "Focus on breaking changes, deprecated features, and compatibility issues.",
            ),
            "language_migration" => (
                "Programming Language Migration",
                "Focus on language differences, ecosystem changes, and tooling requirements.",
            ),
            "architecture_change" => (
                "Architectural Transformation",
                "Focus on structural changes, component redesign, and system interactions.",
            ),
            _ => (
                "General Migration",
                "Focus on identifying migration requirements and planning approach.",
            ),
        };

        let prompt_text = format!(
            r#"Please create a comprehensive migration plan for this {} project:

{}

Migration type: {}
Target technology: {}
Risk tolerance: {}

**Migration Planning Overview**:
{}

Please provide a detailed migration strategy:

1. **Current State Analysis**:
   - Current technology stack and versions
   - Dependencies and external integrations
   - Code structure and architectural patterns
   - Technical debt and legacy components

2. **Target State Definition**:
   - Target technology specifications
   - Expected benefits and improvements
   - New capabilities and features
   - Performance and scalability goals

3. **Gap Analysis**:
   - Compatibility issues and breaking changes
   - Features that need redesign or rewriting
   - Dependencies requiring updates or replacements
   - Infrastructure and tooling changes needed

4. **Migration Strategy**:
   - Recommended migration approach (big bang, incremental, parallel)
   - Migration phases and milestones
   - Rollback and contingency plans
   - Success criteria and validation methods

5. **Risk Assessment and Mitigation**:
   - Technical risks and challenges
   - Business continuity considerations
   - Resource and timeline risks
   - Mitigation strategies for each risk

6. **Implementation Roadmap**:
   - Phase-by-phase implementation plan
   - Timeline estimates and dependencies
   - Resource requirements and team structure
   - Key decision points and checkpoints

7. **Code Migration Planning**:
   - Files and modules requiring changes
   - Automated migration tools and scripts
   - Manual migration requirements
   - Code review and testing strategies

8. **Testing and Validation**:
   - Migration testing strategy
   - Regression testing approach
   - Performance testing requirements
   - User acceptance testing plan

9. **Deployment and Rollout**:
   - Deployment strategy and environments
   - Feature flags and gradual rollout
   - Monitoring and observability
   - Communication and training plans

10. **Post-Migration Considerations**:
    - Cleanup and optimization tasks
    - Documentation updates
    - Team training and knowledge transfer
    - Long-term maintenance considerations

Use repository analysis tools to:
- Examine current codebase structure and dependencies
- Identify migration complexity and effort
- Assess code quality and technical debt
- Analyze testing coverage and quality
- Review architecture and design patterns

Tailor recommendations to the {} risk tolerance level.
Provide specific file references, code examples, and actionable steps."#,
            migration_focus,
            repo_context,
            migration_type,
            target_technology,
            risk_tolerance,
            specific_considerations,
            risk_tolerance
        );

        Ok(GetPromptResult {
            description: Some(format!(
                "Comprehensive migration planning for {}",
                migration_type
            )),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text: prompt_text },
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_capabilities() {
        let capabilities = PromptCapabilities {
            list_changed: Some(false),
        };

        assert_eq!(capabilities.list_changed, Some(false));
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            name: "test_prompt".to_string(),
            title: Some("Test Prompt".to_string()),
            description: "A test prompt".to_string(),
            arguments: Some(vec![PromptArgument {
                name: "test_arg".to_string(),
                description: "A test argument".to_string(),
                required: true,
            }]),
        };

        let json = serde_json::to_string(&prompt).unwrap();
        let deserialized: Prompt = serde_json::from_str(&json).unwrap();

        assert_eq!(prompt.name, deserialized.name);
        assert_eq!(prompt.title, deserialized.title);
        assert_eq!(prompt.description, deserialized.description);
    }

    #[test]
    fn test_prompt_content_text() {
        let content = PromptContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: PromptContent = serde_json::from_str(&json).unwrap();

        match deserialized {
            PromptContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
    }

    async fn create_test_server() -> crate::CodePrismMcpServer {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Create test files for prompt testing
        fs::write(
            repo_path.join("main.py"),
            r#"
class UserService:
    """Service for managing users."""
    
    def __init__(self, database):
        self.database = database
    
    def authenticate_user(self, username: str, password: str) -> bool:
        """Authenticate a user with username and password."""
        user = self.database.get_user(username)
        if not user:
            return False
        return user.verify_password(password)
    
    def create_user(self, username: str, email: str, password: str) -> 'User':
        """Create a new user account."""
        if self.database.user_exists(username):
            raise ValueError("User already exists")
        
        user = User(username, email, password)
        self.database.save_user(user)
        return user

class User:
    """User model representing a system user."""
    
    def __init__(self, username: str, email: str, password: str):
        self.username = username
        self.email = email
        self.password_hash = self._hash_password(password)
    
    def verify_password(self, password: str) -> bool:
        """Verify the user's password."""
        return self._hash_password(password) == self.password_hash
    
    def _hash_password(self, password: str) -> str:
        """Hash a password for storage."""
        import hashlib
        return hashlib.sha256(password.encode()).hexdigest()
"#,
        )
        .unwrap();

        fs::write(
            repo_path.join("database.py"),
            r#"
"""Database interface for the application."""

from typing import Optional, List
import sqlite3

class Database:
    """Simple SQLite database interface."""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.connection = sqlite3.connect(db_path)
        self._create_tables()
    
    def _create_tables(self) -> None:
        """Create necessary database tables."""
        cursor = self.connection.cursor()
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                email TEXT NOT NULL,
                password_hash TEXT NOT NULL
            )
        ''')
        self.connection.commit()
    
    def get_user(self, username: str) -> Optional['User']:
        """Get a user by username."""
        cursor = self.connection.cursor()
        cursor.execute('SELECT * FROM users WHERE username = ?', (username,))
        row = cursor.fetchone()
        
        if row:
            from main import User
            return User(row[1], row[2], "")  # Password already hashed
        return None
    
    def user_exists(self, username: str) -> bool:
        """Check if a user exists."""
        return self.get_user(username) is not None
    
    def save_user(self, user: 'User') -> None:
        """Save a user to the database."""
        cursor = self.connection.cursor()
        cursor.execute(
            'INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)',
            (user.username, user.email, user.password_hash)
        )
        self.connection.commit()
"#,
        )
        .unwrap();

        let mut server = crate::CodePrismMcpServer::new().expect("Failed to create server");
        server
            .initialize_with_repository(repo_path)
            .await
            .expect("Failed to initialize repository");

        // Keep temp_dir alive
        std::mem::forget(temp_dir);

        server
    }

    #[tokio::test]
    async fn test_prompt_manager_creation() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        // Prompt manager should be created successfully
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let result = prompt_manager
            .list_prompts(ListPromptsParams { cursor: None })
            .await;
        assert!(result.is_ok());

        let prompts_result = result.unwrap();
        assert_eq!(prompts_result.prompts.len(), 16); // Original 8 + 8 new prompts for large codebase understanding
        assert!(prompts_result.next_cursor.is_none());

        // Verify all expected prompts are present
        let prompt_names: Vec<String> = prompts_result
            .prompts
            .iter()
            .map(|p| p.name.clone())
            .collect();
        assert!(prompt_names.contains(&"repository_overview".to_string()));
        assert!(prompt_names.contains(&"code_analysis".to_string()));
        assert!(prompt_names.contains(&"debug_assistance".to_string()));
        assert!(prompt_names.contains(&"debug_issue".to_string()));
        assert!(prompt_names.contains(&"refactoring_guidance".to_string()));

        // Verify prompt structure
        for prompt in prompts_result.prompts {
            assert!(!prompt.name.is_empty());
            assert!(!prompt.description.is_empty());

            // Check arguments structure
            if let Some(args) = prompt.arguments {
                for arg in args {
                    assert!(!arg.name.is_empty());
                    assert!(!arg.description.is_empty());
                }
            }
        }
    }

    #[tokio::test]
    async fn test_repository_overview_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "repository_overview".to_string(),
            arguments: Some(serde_json::Map::from_iter([(
                "focus_area".to_string(),
                serde_json::Value::String("architecture".to_string()),
            )])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert!(prompt_result.description.is_some());
        assert_eq!(prompt_result.messages.len(), 1);

        let message = &prompt_result.messages[0];
        assert_eq!(message.role, "user");

        if let PromptContent::Text { text } = &message.content {
            assert!(text.contains("architecture"));
            assert!(text.contains("repository"));
            assert!(text.contains("analyze"));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_repository_overview_prompt_default_focus() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "repository_overview".to_string(),
            arguments: None, // Test default focus area
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("general")); // Default focus area
        }
    }

    #[tokio::test]
    async fn test_code_analysis_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "code_analysis".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "file_pattern".to_string(),
                    serde_json::Value::String("*.py".to_string()),
                ),
                (
                    "analysis_type".to_string(),
                    serde_json::Value::String("security".to_string()),
                ),
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("security"));
            assert!(text.contains("*.py"));
            assert!(text.contains("analysis"));
        }
    }

    #[tokio::test]
    async fn test_debug_assistance_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "debug_assistance".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "issue_description".to_string(),
                    serde_json::Value::String(
                        "Authentication is failing for some users".to_string(),
                    ),
                ),
                (
                    "affected_files".to_string(),
                    serde_json::Value::String("main.py, database.py".to_string()),
                ),
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("Authentication is failing"));
            assert!(text.contains("main.py, database.py"));
            assert!(text.contains("debugging"));
        }
    }

    #[tokio::test]
    async fn test_debug_assistance_prompt_missing_required_arg() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "debug_assistance".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "affected_files".to_string(),
                    serde_json::Value::String("main.py".to_string()),
                ),
                // Missing required issue_description
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("issue_description"));
        assert!(error.to_string().contains("required"));
    }

    #[tokio::test]
    async fn test_debug_issue_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "debug_issue".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "error_location".to_string(),
                    serde_json::Value::String("main.py:25".to_string()),
                ),
                (
                    "error_message".to_string(),
                    serde_json::Value::String(
                        "AttributeError: 'NoneType' object has no attribute 'verify_password'"
                            .to_string(),
                    ),
                ),
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("main.py:25"));
            assert!(text.contains("AttributeError"));
            assert!(text.contains("verify_password"));
        }
    }

    #[tokio::test]
    async fn test_debug_issue_prompt_missing_required_arg() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "debug_issue".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "error_message".to_string(),
                    serde_json::Value::String("Some error".to_string()),
                ),
                // Missing required error_location
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("error_location"));
        assert!(error.to_string().contains("required"));
    }

    #[tokio::test]
    async fn test_refactoring_guidance_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "refactoring_guidance".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "target_area".to_string(),
                    serde_json::Value::String("UserService class".to_string()),
                ),
                (
                    "refactoring_goal".to_string(),
                    serde_json::Value::String("improve testability".to_string()),
                ),
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("UserService class"));
            assert!(text.contains("improve testability"));
            assert!(text.contains("refactoring"));
        }
    }

    #[tokio::test]
    async fn test_refactoring_guidance_prompt_missing_required_arg() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "refactoring_guidance".to_string(),
            arguments: Some(serde_json::Map::from_iter([
                (
                    "refactoring_goal".to_string(),
                    serde_json::Value::String("improve performance".to_string()),
                ),
                // Missing required target_area
            ])),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("target_area"));
        assert!(error.to_string().contains("required"));
    }

    #[tokio::test]
    async fn test_unknown_prompt() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);

        let params = GetPromptParams {
            name: "unknown_prompt".to_string(),
            arguments: None,
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unknown prompt"));
        assert!(error.to_string().contains("unknown_prompt"));
    }

    #[test]
    fn test_prompt_message_serialization() {
        let message = PromptMessage {
            role: "user".to_string(),
            content: PromptContent::Text {
                text: "Test message".to_string(),
            },
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: PromptMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(message.role, deserialized.role);
        if let (PromptContent::Text { text: orig }, PromptContent::Text { text: deser }) =
            (&message.content, &deserialized.content)
        {
            assert_eq!(orig, deser);
        } else {
            panic!("Content type mismatch");
        }
    }

    #[test]
    fn test_prompt_content_image() {
        let content = PromptContent::Image {
            data: "base64encodeddata".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: PromptContent = serde_json::from_str(&json).unwrap();

        if let PromptContent::Image { data, mime_type } = deserialized {
            assert_eq!(data, "base64encodeddata");
            assert_eq!(mime_type, "image/png");
        } else {
            panic!("Expected image content");
        }
    }

    #[test]
    fn test_get_prompt_params_serialization() {
        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        let params = GetPromptParams {
            name: "test_prompt".to_string(),
            arguments: Some(arguments),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: GetPromptParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.name, deserialized.name);
        assert_eq!(params.arguments, deserialized.arguments);
    }

    #[test]
    fn test_get_prompt_result_serialization() {
        let result = GetPromptResult {
            description: Some("Test prompt result".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text {
                    text: "Test content".to_string(),
                },
            }],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: GetPromptResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.description, deserialized.description);
        assert_eq!(result.messages.len(), deserialized.messages.len());
    }

    #[test]
    fn test_list_prompts_params_serialization() {
        let params = ListPromptsParams {
            cursor: Some("test_cursor".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ListPromptsParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.cursor, deserialized.cursor);
    }

    #[test]
    fn test_prompt_argument_serialization() {
        let arg = PromptArgument {
            name: "test_arg".to_string(),
            description: "Test argument".to_string(),
            required: true,
        };

        let json = serde_json::to_value(&arg).unwrap();
        assert_eq!(json["name"], "test_arg");
        assert_eq!(json["description"], "Test argument");
        assert_eq!(json["required"], true);

        let deserialized: PromptArgument = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name, "test_arg");
        assert_eq!(deserialized.description, "Test argument");
        assert!(deserialized.required);
    }

    #[tokio::test]
    async fn test_architectural_analysis_prompt() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = GetPromptParams {
            name: "architectural_analysis".to_string(),
            arguments: Some({
                let mut args = serde_json::Map::new();
                args.insert("analysis_focus".to_string(), "layers".into());
                args.insert("architectural_concerns".to_string(), "coupling".into());
                args
            }),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert!(prompt_result.description.is_some());
        assert_eq!(prompt_result.messages.len(), 1);
        assert_eq!(prompt_result.messages[0].role, "user");

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("architectural"));
            assert!(text.contains("layers"));
        }
    }

    #[tokio::test]
    async fn test_pattern_assessment_prompt() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = GetPromptParams {
            name: "pattern_assessment".to_string(),
            arguments: Some({
                let mut args = serde_json::Map::new();
                args.insert("pattern_types".to_string(), "design_patterns".into());
                args.insert("improvement_scope".to_string(), "immediate".into());
                args
            }),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("pattern"));
            assert!(text.contains("design_patterns"));
        }
    }

    #[tokio::test]
    async fn test_dependency_analysis_prompt() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = GetPromptParams {
            name: "dependency_analysis".to_string(),
            arguments: Some({
                let mut args = serde_json::Map::new();
                args.insert("analysis_target".to_string(), "repository-wide".into());
                args.insert("dependency_concerns".to_string(), "cycles".into());
                args
            }),
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        assert_eq!(prompt_result.messages.len(), 1);

        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("dependency"));
            assert!(text.contains("cycles"));
        }
    }

    #[tokio::test]
    async fn test_new_prompts_in_list() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = ListPromptsParams { cursor: None };
        let result = prompt_manager.list_prompts(params).await;
        assert!(result.is_ok());

        let prompts_result = result.unwrap();
        let prompt_names: Vec<&String> = prompts_result.prompts.iter().map(|p| &p.name).collect();

        // Check that our new architectural prompts are included
        assert!(prompt_names.contains(&&"architectural_analysis".to_string()));
        assert!(prompt_names.contains(&&"pattern_assessment".to_string()));
        assert!(prompt_names.contains(&&"dependency_analysis".to_string()));

        // Should have all prompts including new ones
        assert!(prompts_result.prompts.len() >= 16); // All 16 prompts should be available
    }

    #[tokio::test]
    async fn test_architectural_prompts_with_default_args() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        // Test architectural_analysis with no arguments (should use defaults)
        let params = GetPromptParams {
            name: "architectural_analysis".to_string(),
            arguments: None,
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_ok());

        let prompt_result = result.unwrap();
        if let PromptContent::Text { text } = &prompt_result.messages[0].content {
            assert!(text.contains("general")); // Should use default focus
        }
    }

    #[tokio::test]
    async fn test_unknown_architectural_prompt() {
        let server = create_test_server().await;
        let prompt_manager =
            PromptManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));

        let params = GetPromptParams {
            name: "unknown_architectural_prompt".to_string(),
            arguments: None,
        };

        let result = prompt_manager.get_prompt(params).await;
        assert!(result.is_err()); // Should return error for unknown prompt
    }
}
