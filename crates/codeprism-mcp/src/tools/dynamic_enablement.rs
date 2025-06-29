//! Dynamic Tool Enablement Manager (Phase 2.2)
//!
//! This module provides dynamic tool enablement based on repository characteristics,
//! client type, and configuration rules. Tools can be enabled or disabled based on
//! conditions like repository size, detected languages, and client capabilities.

use crate::config::{
    EnablementAction, EnablementCondition, EnablementRule, McpConfigProfile, ToolCategory,
    ToolConfiguration,
};
use crate::protocol::ClientType;
use crate::tools_legacy::Tool;
use crate::CodePrismMcpServer;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info, warn};

/// Repository analysis for tool enablement decisions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryAnalysis {
    /// Total repository size in MB
    pub size_mb: usize,
    /// Total number of files
    pub file_count: usize,
    /// Detected programming languages
    pub languages: Vec<String>,
    /// Repository type (monorepo, microservice, library, etc.)
    pub repo_type: String,
    /// Primary language (most files)
    pub primary_language: Option<String>,
    /// Framework detections
    pub frameworks: Vec<String>,
    /// Average file size in KB
    pub avg_file_size_kb: f64,
    /// Complexity score (0-100)
    pub complexity_score: u32,
}

/// Dynamic tool enablement manager
#[derive(Debug, Clone)]
pub struct DynamicToolManager {
    /// Base tool configuration
    #[allow(dead_code)] // TODO: Will be used for tool configuration management
    base_config: ToolConfiguration,
    /// Active enablement rules
    rules: Vec<EnablementRule>,
    /// Enabled tool set
    enabled_tools: HashSet<String>,
    /// Disabled tool set
    disabled_tools: HashSet<String>,
    /// Tool-specific configurations
    tool_configs: HashMap<String, crate::config::ToolConfig>,
    /// Repository analysis cache
    repo_analysis: Option<RepositoryAnalysis>,
    /// Client type
    client_type: Option<ClientType>,
}

impl DynamicToolManager {
    /// Create a new dynamic tool manager
    pub fn new(config: ToolConfiguration) -> Self {
        let mut enabled_tools = HashSet::new();

        // Enable tools from enabled categories
        for category in &config.enabled_categories {
            enabled_tools.extend(Self::get_tools_for_category(category));
        }

        // Remove explicitly disabled tools
        for tool in &config.disabled_tools {
            enabled_tools.remove(tool);
        }

        let disabled_tools = config.disabled_tools.iter().cloned().collect();

        Self {
            base_config: config.clone(),
            rules: config.enablement_rules.clone(),
            enabled_tools,
            disabled_tools,
            tool_configs: config.tool_configs.clone(),
            repo_analysis: None,
            client_type: None,
        }
    }

    /// Create from configuration profile
    pub fn from_profile(profile: &McpConfigProfile) -> Self {
        Self::new(profile.tool_config.clone())
    }

    /// Get tools for a specific category
    fn get_tools_for_category(category: &ToolCategory) -> Vec<String> {
        match category {
            ToolCategory::CoreNavigation => vec![
                "repository_stats".to_string(),
                "explain_symbol".to_string(),
                "trace_path".to_string(),
                "find_dependencies".to_string(),
            ],
            ToolCategory::SearchDiscovery => vec![
                "search_symbols".to_string(),
                "search_content".to_string(),
                "find_files".to_string(),
                "content_stats".to_string(),
            ],
            ToolCategory::Analysis => vec![
                "find_unused_code".to_string(),
                "analyze_security".to_string(),
                "analyze_performance".to_string(),
                "analyze_api_surface".to_string(),
                "analyze_complexity".to_string(),
                "trace_data_flow".to_string(),
                "analyze_transitive_dependencies".to_string(),
                "detect_patterns".to_string(),
                "trace_inheritance".to_string(),
                "analyze_decorators".to_string(),
                "find_duplicates".to_string(),
            ],
            ToolCategory::JavaScriptAnalysis => vec![
                "analyze_javascript_frameworks".to_string(),
                "analyze_react_components".to_string(),
                "analyze_nodejs_patterns".to_string(),
            ],
            ToolCategory::Workflow => vec![
                "orchestrate_workflow".to_string(),
                "batch_analyze".to_string(),
                "generate_guidance".to_string(),
                "optimize_workflow".to_string(),
            ],
            ToolCategory::Experimental => vec![
                // Future experimental tools
            ],
        }
    }

    /// Set client type for client-specific optimizations
    pub fn set_client_type(&mut self, client_type: ClientType) {
        self.client_type = Some(client_type.clone());
        self.apply_client_optimizations(&client_type);
    }

    /// Apply client-specific optimizations
    fn apply_client_optimizations(&mut self, client_type: &ClientType) {
        match client_type {
            ClientType::Claude => {
                // Claude supports streaming and larger responses
                // Enable comprehensive analysis tools
                self.enable_tool("analyze_transitive_dependencies");
                self.enable_tool("find_duplicates");
            }
            ClientType::Cursor => {
                // Cursor prefers faster responses
                // Disable heavy analysis tools
                self.disable_tool("analyze_transitive_dependencies");
                self.disable_tool("find_duplicates");

                // Limit results for content tools
                self.set_tool_config("search_content", "max_results", 50);
                self.set_tool_config("find_files", "max_results", 100);
            }
            ClientType::VSCode => {
                // VS Code balance between performance and features
                self.set_tool_config("search_content", "max_results", 75);
                self.set_tool_config("find_files", "max_results", 150);
            }
            ClientType::Unknown(_) => {
                // Conservative defaults for unknown clients
                self.disable_tool("analyze_transitive_dependencies");
                self.set_tool_config("search_content", "max_results", 50);
            }
        }

        info!("Applied client optimizations for {:?}", client_type);
    }

    /// Analyze repository and apply enablement rules
    pub async fn analyze_and_configure<P: AsRef<Path>>(
        &mut self,
        server: &CodePrismMcpServer,
        repo_path: P,
    ) -> Result<()> {
        let analysis = self.analyze_repository(server, repo_path).await?;
        self.repo_analysis = Some(analysis.clone());

        // Apply enablement rules based on analysis
        self.apply_enablement_rules(&analysis)?;

        info!(
            "Configured tools for repository: {} languages, {}MB, {} files",
            analysis.languages.join(", "),
            analysis.size_mb,
            analysis.file_count
        );

        Ok(())
    }

    /// Analyze repository characteristics
    async fn analyze_repository<P: AsRef<Path>>(
        &self,
        _server: &CodePrismMcpServer,
        repo_path: P,
    ) -> Result<RepositoryAnalysis> {
        let path = repo_path.as_ref();

        // Calculate repository size
        let size_mb = self.calculate_directory_size(path)? / (1024 * 1024);

        // Count files and detect languages
        let (file_count, languages) = self.analyze_files(path)?;

        // Determine repository type
        let repo_type = self.determine_repo_type(path)?;

        // Detect frameworks
        let frameworks = self.detect_frameworks(path)?;

        // Calculate average file size
        let avg_file_size_kb = if file_count > 0 {
            (size_mb * 1024) as f64 / file_count as f64
        } else {
            0.0
        };

        // Simple complexity scoring based on file count and size
        let complexity_score = std::cmp::min(
            ((file_count / 100) + (size_mb / 10) + (languages.len() * 5)) as u32,
            100,
        );

        let primary_language = self.determine_primary_language(&languages);

        Ok(RepositoryAnalysis {
            size_mb,
            file_count,
            languages,
            repo_type,
            primary_language,
            frameworks,
            avg_file_size_kb,
            complexity_score,
        })
    }

    /// Calculate directory size in bytes
    fn calculate_directory_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
        let mut total_size = 0;

        fn visit_dir(dir: &Path, total: &mut usize) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Skip common large directories
                    if !matches!(
                        dir_name,
                        "node_modules" | "target" | ".git" | "vendor" | "dist" | "build"
                    ) {
                        visit_dir(&path, total)?;
                    }
                } else {
                    *total += entry.metadata()?.len() as usize;
                }
            }
            Ok(())
        }

        visit_dir(path.as_ref(), &mut total_size)?;
        Ok(total_size)
    }

    /// Analyze files and detect languages
    fn analyze_files<P: AsRef<Path>>(&self, path: P) -> Result<(usize, Vec<String>)> {
        let mut file_count = 0;
        let mut language_extensions = HashSet::new();

        fn visit_dir(dir: &Path, count: &mut usize, exts: &mut HashSet<String>) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if !matches!(
                        dir_name,
                        "node_modules" | "target" | ".git" | "vendor" | "dist" | "build"
                    ) {
                        visit_dir(&path, count, exts)?;
                    }
                } else {
                    *count += 1;
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        exts.insert(ext.to_lowercase());
                    }
                }
            }
            Ok(())
        }

        visit_dir(path.as_ref(), &mut file_count, &mut language_extensions)?;

        // Map extensions to languages
        let languages = language_extensions
            .iter()
            .filter_map(|ext| self.extension_to_language(ext))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Ok((file_count, languages))
    }

    /// Map file extension to programming language
    fn extension_to_language(&self, ext: &str) -> Option<String> {
        match ext {
            "js" | "jsx" | "mjs" | "cjs" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "py" | "pyw" => Some("python".to_string()),
            "rs" => Some("rust".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "cxx" | "cc" | "C" => Some("cpp".to_string()),
            "c" | "h" => Some("c".to_string()),
            "go" => Some("go".to_string()),
            "php" => Some("php".to_string()),
            "rb" => Some("ruby".to_string()),
            "kt" | "kts" => Some("kotlin".to_string()),
            "swift" => Some("swift".to_string()),
            "cs" => Some("csharp".to_string()),
            "scala" | "sc" => Some("scala".to_string()),
            "clj" | "cljs" => Some("clojure".to_string()),
            "hs" => Some("haskell".to_string()),
            "sh" | "bash" | "zsh" => Some("shell".to_string()),
            _ => None,
        }
    }

    /// Determine repository type
    fn determine_repo_type<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();

        // Check for specific files that indicate repository type
        if path.join("package.json").exists() {
            if path.join("lerna.json").exists() || path.join("nx.json").exists() {
                return Ok("monorepo".to_string());
            }
            return Ok("nodejs".to_string());
        }

        if path.join("Cargo.toml").exists() {
            if path.join("Cargo.lock").exists() {
                return Ok("rust_binary".to_string());
            }
            return Ok("rust_library".to_string());
        }

        if path.join("requirements.txt").exists()
            || path.join("setup.py").exists()
            || path.join("pyproject.toml").exists()
        {
            return Ok("python".to_string());
        }

        if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            return Ok("java".to_string());
        }

        if path.join("go.mod").exists() {
            return Ok("go".to_string());
        }

        Ok("generic".to_string())
    }

    /// Detect frameworks used in the repository
    fn detect_frameworks<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let mut frameworks = Vec::new();
        let path = path.as_ref();

        // Check package.json for JavaScript frameworks
        if let Ok(package_json) = std::fs::read_to_string(path.join("package.json")) {
            if package_json.contains("\"react\"") {
                frameworks.push("react".to_string());
            }
            if package_json.contains("\"vue\"") {
                frameworks.push("vue".to_string());
            }
            if package_json.contains("\"angular\"") {
                frameworks.push("angular".to_string());
            }
            if package_json.contains("\"express\"") {
                frameworks.push("express".to_string());
            }
            if package_json.contains("\"next\"") {
                frameworks.push("nextjs".to_string());
            }
        }

        // Check for framework-specific files
        if path.join("angular.json").exists() {
            frameworks.push("angular".to_string());
        }
        if path.join("next.config.js").exists() {
            frameworks.push("nextjs".to_string());
        }
        if path.join("vue.config.js").exists() {
            frameworks.push("vue".to_string());
        }

        Ok(frameworks)
    }

    /// Determine primary language based on file count
    fn determine_primary_language(&self, languages: &[String]) -> Option<String> {
        // For now, simple heuristic - could be enhanced with actual file counting
        languages.first().cloned()
    }

    /// Apply enablement rules based on repository analysis
    fn apply_enablement_rules(&mut self, analysis: &RepositoryAnalysis) -> Result<()> {
        for rule in &self.rules.clone() {
            if self.evaluate_condition(&rule.condition, analysis) {
                debug!("Applying enablement rule: {}", rule.name);

                for action in &rule.actions {
                    self.apply_enablement_action(action)?;
                }
            }
        }

        Ok(())
    }

    /// Evaluate an enablement condition
    fn evaluate_condition(
        &self,
        condition: &EnablementCondition,
        analysis: &RepositoryAnalysis,
    ) -> bool {
        match condition {
            EnablementCondition::RepositorySize { max_size_mb } => analysis.size_mb <= *max_size_mb,
            EnablementCondition::FileCount { max_files } => analysis.file_count <= *max_files,
            EnablementCondition::HasLanguages { languages } => languages
                .iter()
                .any(|lang| analysis.languages.contains(lang)),
            EnablementCondition::ClientType { client_types } => {
                if let Some(ref client) = self.client_type {
                    let client_name = match client {
                        ClientType::Claude => "claude",
                        ClientType::Cursor => "cursor",
                        ClientType::VSCode => "vscode",
                        ClientType::Unknown(name) => name,
                    };
                    client_types.iter().any(|ct| ct == client_name)
                } else {
                    false
                }
            }
            EnablementCondition::RepositoryType { repo_types } => {
                repo_types.contains(&analysis.repo_type)
            }
            EnablementCondition::Custom { expression: _ } => {
                // For now, custom expressions are not implemented
                warn!("Custom enablement conditions not yet implemented");
                false
            }
        }
    }

    /// Apply an enablement action
    fn apply_enablement_action(&mut self, action: &EnablementAction) -> Result<()> {
        match action {
            EnablementAction::Enable { tools } => {
                for tool in tools {
                    self.enable_tool(tool);
                }
            }
            EnablementAction::Disable { tools } => {
                for tool in tools {
                    self.disable_tool(tool);
                }
            }
            EnablementAction::EnableCategory { category } => {
                let tools = Self::get_tools_for_category(category);
                for tool in tools {
                    self.enable_tool(&tool);
                }
            }
            EnablementAction::DisableCategory { category } => {
                let tools = Self::get_tools_for_category(category);
                for tool in tools {
                    self.disable_tool(&tool);
                }
            }
            EnablementAction::Configure { tool, config } => {
                self.tool_configs.insert(tool.clone(), config.clone());
            }
        }

        Ok(())
    }

    /// Enable a specific tool
    pub fn enable_tool(&mut self, tool_name: &str) {
        self.enabled_tools.insert(tool_name.to_string());
        self.disabled_tools.remove(tool_name);
        debug!("Enabled tool: {}", tool_name);
    }

    /// Disable a specific tool
    pub fn disable_tool(&mut self, tool_name: &str) {
        self.enabled_tools.remove(tool_name);
        self.disabled_tools.insert(tool_name.to_string());
        debug!("Disabled tool: {}", tool_name);
    }

    /// Set tool-specific configuration
    pub fn set_tool_config(
        &mut self,
        tool_name: &str,
        key: &str,
        value: impl Into<serde_json::Value>,
    ) {
        let config = self
            .tool_configs
            .entry(tool_name.to_string())
            .or_insert_with(|| crate::config::ToolConfig {
                timeout: None,
                max_results: None,
                memory_limit_mb: None,
                custom_params: HashMap::new(),
            });

        config.custom_params.insert(key.to_string(), value.into());
    }

    /// Check if a tool is enabled
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.enabled_tools.contains(tool_name) && !self.disabled_tools.contains(tool_name)
    }

    /// Get enabled tools
    pub fn get_enabled_tools(&self) -> Vec<String> {
        self.enabled_tools.iter().cloned().collect()
    }

    /// Get disabled tools
    pub fn get_disabled_tools(&self) -> Vec<String> {
        self.disabled_tools.iter().cloned().collect()
    }

    /// Get tool configuration
    pub fn get_tool_config(&self, tool_name: &str) -> Option<&crate::config::ToolConfig> {
        self.tool_configs.get(tool_name)
    }

    /// Filter available tools based on enabled state
    pub fn filter_tools(&self, all_tools: Vec<Tool>) -> Vec<Tool> {
        all_tools
            .into_iter()
            .filter(|tool| self.is_tool_enabled(&tool.name))
            .collect()
    }

    /// Get configuration summary
    pub fn get_summary(&self) -> ToolEnablementSummary {
        ToolEnablementSummary {
            enabled_tools: self.get_enabled_tools(),
            disabled_tools: self.get_disabled_tools(),
            total_rules: self.rules.len(),
            repo_analysis: self.repo_analysis.clone(),
            client_type: self.client_type.clone(),
        }
    }
}

/// Summary of tool enablement configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolEnablementSummary {
    pub enabled_tools: Vec<String>,
    pub disabled_tools: Vec<String>,
    pub total_rules: usize,
    pub repo_analysis: Option<RepositoryAnalysis>,
    pub client_type: Option<ClientType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CachingConfig, McpConfig, MonitoringConfig, SecurityConfig};

    fn create_test_profile() -> McpConfigProfile {
        McpConfigProfile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            settings: McpConfig::default(),
            tool_config: ToolConfiguration {
                enabled_categories: vec![
                    ToolCategory::CoreNavigation,
                    ToolCategory::SearchDiscovery,
                ],
                disabled_tools: vec!["find_duplicates".to_string()],
                tool_configs: HashMap::new(),
                enablement_rules: vec![EnablementRule {
                    name: "large_repo_rule".to_string(),
                    condition: EnablementCondition::RepositorySize { max_size_mb: 100 },
                    actions: vec![EnablementAction::Disable {
                        tools: vec!["analyze_transitive_dependencies".to_string()],
                    }],
                }],
            },
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
            caching: CachingConfig::default(),
        }
    }

    #[test]
    fn test_tool_category_mapping() {
        let core_tools = DynamicToolManager::get_tools_for_category(&ToolCategory::CoreNavigation);
        assert!(core_tools.contains(&"repository_stats".to_string()));
        assert!(core_tools.contains(&"explain_symbol".to_string()));

        let js_tools =
            DynamicToolManager::get_tools_for_category(&ToolCategory::JavaScriptAnalysis);
        assert!(js_tools.contains(&"analyze_javascript_frameworks".to_string()));
        assert!(js_tools.contains(&"analyze_react_components".to_string()));
    }

    #[test]
    fn test_dynamic_tool_manager_creation() {
        let profile = create_test_profile();
        let manager = DynamicToolManager::from_profile(&profile);

        // Should enable core navigation tools
        assert!(manager.is_tool_enabled("repository_stats"));
        assert!(manager.is_tool_enabled("explain_symbol"));

        // Should disable explicitly disabled tools
        assert!(!manager.is_tool_enabled("find_duplicates"));
    }

    #[test]
    fn test_client_optimizations() {
        let profile = create_test_profile();
        let mut manager = DynamicToolManager::from_profile(&profile);

        // Test Cursor optimizations (disable heavy tools)
        manager.set_client_type(ClientType::Cursor);
        assert!(!manager.is_tool_enabled("analyze_transitive_dependencies"));

        // Test Claude optimizations (enable comprehensive tools)
        manager.set_client_type(ClientType::Claude);
        assert!(manager.is_tool_enabled("analyze_transitive_dependencies"));
    }

    #[test]
    fn test_extension_to_language_mapping() {
        let manager = DynamicToolManager::new(ToolConfiguration {
            enabled_categories: vec![],
            disabled_tools: vec![],
            tool_configs: HashMap::new(),
            enablement_rules: vec![],
        });

        assert_eq!(
            manager.extension_to_language("js"),
            Some("javascript".to_string())
        );
        assert_eq!(
            manager.extension_to_language("ts"),
            Some("typescript".to_string())
        );
        assert_eq!(
            manager.extension_to_language("py"),
            Some("python".to_string())
        );
        assert_eq!(
            manager.extension_to_language("rs"),
            Some("rust".to_string())
        );
        assert_eq!(manager.extension_to_language("unknown"), None);
    }

    #[test]
    fn test_enablement_condition_evaluation() {
        let manager = DynamicToolManager::new(ToolConfiguration {
            enabled_categories: vec![],
            disabled_tools: vec![],
            tool_configs: HashMap::new(),
            enablement_rules: vec![],
        });

        let analysis = RepositoryAnalysis {
            size_mb: 50,
            file_count: 200,
            languages: vec!["javascript".to_string(), "typescript".to_string()],
            repo_type: "nodejs".to_string(),
            primary_language: Some("javascript".to_string()),
            frameworks: vec!["react".to_string()],
            avg_file_size_kb: 10.0,
            complexity_score: 45,
        };

        // Test size condition
        let size_condition = EnablementCondition::RepositorySize { max_size_mb: 100 };
        assert!(manager.evaluate_condition(&size_condition, &analysis));

        let size_condition_fail = EnablementCondition::RepositorySize { max_size_mb: 30 };
        assert!(!manager.evaluate_condition(&size_condition_fail, &analysis));

        // Test language condition
        let lang_condition = EnablementCondition::HasLanguages {
            languages: vec!["javascript".to_string()],
        };
        assert!(manager.evaluate_condition(&lang_condition, &analysis));

        let lang_condition_fail = EnablementCondition::HasLanguages {
            languages: vec!["python".to_string()],
        };
        assert!(!manager.evaluate_condition(&lang_condition_fail, &analysis));
    }
}
