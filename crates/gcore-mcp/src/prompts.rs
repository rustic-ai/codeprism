//! MCP Prompts implementation
//! 
//! Prompts allow servers to provide structured messages and instructions for
//! interacting with language models. Clients can discover available prompts,
//! retrieve their contents, and provide arguments to customize them.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::GCoreMcpServer;

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
    server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>,
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>) -> Self {
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
            "repository_overview" => self.repository_overview_prompt(&server, params.arguments).await,
            "code_analysis" => self.code_analysis_prompt(&server, params.arguments).await,
            "debug_assistance" => self.debug_assistance_prompt(&server, params.arguments).await,
            "debug_issue" => self.debug_issue_prompt(&server, params.arguments).await,
            "refactoring_guidance" => self.refactoring_guidance_prompt(&server, params.arguments).await,
            _ => Err(anyhow::anyhow!("Unknown prompt: {}", params.name)),
        }
    }

    /// Generate repository overview prompt
    async fn repository_overview_prompt(
        &self,
        server: &GCoreMcpServer,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<GetPromptResult> {
        let focus_area = arguments
            .as_ref()
            .and_then(|args| args.get("focus_area"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let repo_context = if let Some(repo_path) = server.repository_path() {
            let file_count = server.scanner().discover_files(repo_path)
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: prompt_text,
                    },
                },
            ],
        })
    }

    /// Generate code analysis prompt
    async fn code_analysis_prompt(
        &self,
        server: &GCoreMcpServer,
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: prompt_text,
                    },
                },
            ],
        })
    }

    /// Generate debug assistance prompt
    async fn debug_assistance_prompt(
        &self,
        _server: &GCoreMcpServer,
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: prompt_text,
                    },
                },
            ],
        })
    }

    /// Generate debug issue prompt
    async fn debug_issue_prompt(
        &self,
        _server: &GCoreMcpServer,
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: prompt_text,
                    },
                },
            ],
        })
    }

    /// Generate refactoring guidance prompt
    async fn refactoring_guidance_prompt(
        &self,
        _server: &GCoreMcpServer,
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: prompt_text,
                    },
                },
            ],
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
            arguments: Some(vec![
                PromptArgument {
                    name: "test_arg".to_string(),
                    description: "A test argument".to_string(),
                    required: true,
                },
            ]),
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

    async fn create_test_server() -> crate::GCoreMcpServer {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        
        // Create test files for prompt testing
        fs::write(repo_path.join("main.py"), r#"
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
"#).unwrap();

        fs::write(repo_path.join("database.py"), r#"
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
"#).unwrap();

        let mut server = crate::GCoreMcpServer::new().expect("Failed to create server");
        server.initialize_with_repository(repo_path).await
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
        assert!(true); // Just testing creation doesn't panic
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let server = create_test_server().await;
        let server_arc = std::sync::Arc::new(tokio::sync::RwLock::new(server));
        let prompt_manager = PromptManager::new(server_arc);
        
        let result = prompt_manager.list_prompts(ListPromptsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let prompts_result = result.unwrap();
        assert_eq!(prompts_result.prompts.len(), 5);
        assert!(prompts_result.next_cursor.is_none());
        
        // Verify all expected prompts are present
        let prompt_names: Vec<String> = prompts_result.prompts.iter().map(|p| p.name.clone()).collect();
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
            arguments: Some(serde_json::Map::from_iter([
                ("focus_area".to_string(), serde_json::Value::String("architecture".to_string())),
            ])),
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
                ("file_pattern".to_string(), serde_json::Value::String("*.py".to_string())),
                ("analysis_type".to_string(), serde_json::Value::String("security".to_string())),
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
                ("issue_description".to_string(), serde_json::Value::String("Authentication is failing for some users".to_string())),
                ("affected_files".to_string(), serde_json::Value::String("main.py, database.py".to_string())),
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
                ("affected_files".to_string(), serde_json::Value::String("main.py".to_string())),
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
                ("error_location".to_string(), serde_json::Value::String("main.py:25".to_string())),
                ("error_message".to_string(), serde_json::Value::String("AttributeError: 'NoneType' object has no attribute 'verify_password'".to_string())),
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
                ("error_message".to_string(), serde_json::Value::String("Some error".to_string())),
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
                ("target_area".to_string(), serde_json::Value::String("UserService class".to_string())),
                ("refactoring_goal".to_string(), serde_json::Value::String("improve testability".to_string())),
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
                ("refactoring_goal".to_string(), serde_json::Value::String("improve performance".to_string())),
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
            (&message.content, &deserialized.content) {
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
        arguments.insert("key".to_string(), serde_json::Value::String("value".to_string()));
        
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
            messages: vec![
                PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text {
                        text: "Test content".to_string(),
                    },
                },
            ],
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
            description: "A test argument".to_string(),
            required: true,
        };
        
        let json = serde_json::to_string(&arg).unwrap();
        let deserialized: PromptArgument = serde_json::from_str(&json).unwrap();
        
        assert_eq!(arg.name, deserialized.name);
        assert_eq!(arg.description, deserialized.description);
        assert_eq!(arg.required, deserialized.required);
    }
} 