//! Advanced Configuration System for CodePrism MCP Server
//!
//! This module provides a comprehensive configuration system with predefined profiles,
//! dynamic tool enablement, performance monitoring, and production-ready features.
//! Ported from legacy codeprism-mcp and adapted for rust-sdk architecture.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

/// Configuration profile for different deployment scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePrismProfile {
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: String,
    /// Base configuration settings
    pub settings: ServerSettings,
    /// Tool enablement rules
    pub tools: ToolsConfig,
    /// Performance monitoring settings
    pub monitoring: MonitoringConfig,
    /// Security and access control
    pub security: SecurityConfig,
    /// Caching configuration
    pub caching: CachingConfig,
}

/// Main server configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Server name for MCP identification
    pub name: String,
    /// Server version
    pub version: String,
    /// Memory limit in MB
    pub memory_limit_mb: usize,
    /// Batch size for parallel processing
    pub batch_size: usize,
    /// Maximum file size to process in MB
    pub max_file_size_mb: usize,
    /// Disable memory limit checking
    pub disable_memory_limit: bool,
    /// Directories to exclude from analysis
    pub exclude_dirs: Vec<String>,
    /// File extensions to include
    pub include_extensions: Option<Vec<String>>,
    /// Dependency scanning mode
    pub dependency_mode: DependencyMode,
    /// Default timeout for operations
    pub default_timeout: Duration,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Enable streaming responses
    pub enable_streaming: bool,
    /// Maximum response size in bytes
    pub max_response_size: usize,
}

/// Dependency scanning modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyMode {
    /// Exclude all dependency directories
    Exclude,
    /// Smart scanning - include only public APIs
    Smart,
    /// Include all dependencies
    IncludeAll,
}

/// Tool configuration and enablement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Enabled tool categories
    pub enabled_categories: Vec<ToolCategory>,
    /// Disabled specific tools
    pub disabled_tools: Vec<String>,
    /// Tool-specific configurations
    pub tool_configs: HashMap<String, ToolConfig>,
    /// Conditional enablement rules
    pub enablement_rules: Vec<EnablementRule>,
}

/// Tool categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolCategory {
    /// Core navigation and understanding
    CoreNavigation,
    /// Search and discovery
    SearchDiscovery,
    /// Code analysis and quality
    Analysis,
    /// Workflow orchestration
    Workflow,
    /// Experimental features
    Experimental,
}

/// Individual tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool-specific timeout
    pub timeout: Option<Duration>,
    /// Maximum results to return
    pub max_results: Option<usize>,
    /// Tool-specific memory limit
    pub memory_limit_mb: Option<usize>,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// Tool enablement rules based on conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnablementRule {
    /// Rule name
    pub name: String,
    /// Condition for enablement
    pub condition: EnablementCondition,
    /// Tools to enable/disable
    pub actions: Vec<EnablementAction>,
}

/// Conditions for tool enablement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnablementCondition {
    /// Based on repository size
    RepositorySize { max_size_mb: usize },
    /// Based on file count
    FileCount { max_files: usize },
    /// Based on detected languages
    HasLanguages { languages: Vec<String> },
    /// Based on client type
    ClientType { client_types: Vec<String> },
    /// Based on repository type
    RepositoryType { repo_types: Vec<String> },
    /// Custom condition
    Custom { expression: String },
}

/// Actions for tool enablement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnablementAction {
    /// Enable specific tools
    Enable { tools: Vec<String> },
    /// Disable specific tools
    Disable { tools: Vec<String> },
    /// Enable tool category
    EnableCategory { category: ToolCategory },
    /// Disable tool category
    DisableCategory { category: ToolCategory },
    /// Modify tool configuration
    Configure { tool: String, config: ToolConfig },
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Memory usage monitoring
    pub monitor_memory: bool,
    /// Response time monitoring
    pub monitor_response_times: bool,
    /// Error rate monitoring
    pub monitor_errors: bool,
    /// Export metrics to file
    pub export_metrics: bool,
    /// Metrics export path
    pub metrics_export_path: Option<PathBuf>,
    /// Performance alerting thresholds
    pub alert_thresholds: AlertThresholds,
}

/// Performance alerting thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Maximum error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Minimum success rate (0.0 to 1.0)
    pub min_success_rate: f64,
}

/// Security and access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable audit logging
    pub enable_audit_log: bool,
    /// Audit log path
    pub audit_log_path: Option<PathBuf>,
    /// Allowed repository paths
    pub allowed_paths: Vec<PathBuf>,
    /// Denied repository paths
    pub denied_paths: Vec<PathBuf>,
    /// Maximum analysis depth
    pub max_analysis_depth: usize,
    /// Enable path validation
    pub validate_paths: bool,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Maximum requests per minute
    pub requests_per_minute: usize,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
    /// Burst allowance
    pub burst_size: usize,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Cache size limit in MB
    pub max_cache_size_mb: usize,
    /// Cache TTL for analysis results
    pub analysis_ttl: Duration,
    /// Cache TTL for file content
    pub content_ttl: Duration,
    /// Cache compression
    pub enable_compression: bool,
    /// Cache cleanup interval
    pub cleanup_interval: Duration,
}

/// Main configuration structure for the CodePrism MCP Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Active configuration profile
    pub profile: CodePrismProfile,
    /// Configuration profile manager
    #[serde(skip)]
    pub manager: ConfigProfileManager,
}

/// Configuration profile manager
#[derive(Debug, Clone)]
pub struct ConfigProfileManager {
    profiles: HashMap<String, CodePrismProfile>,
    active_profile: Option<String>,
}

impl ConfigProfileManager {
    /// Create a new configuration profile manager
    pub fn new() -> Self {
        let mut manager = Self {
            profiles: HashMap::new(),
            active_profile: None,
        };

        // Register built-in profiles
        manager.register_builtin_profiles();
        manager
    }

    /// Register built-in configuration profiles
    fn register_builtin_profiles(&mut self) {
        // Development profile
        self.profiles.insert(
            "development".to_string(),
            CodePrismProfile {
                name: "development".to_string(),
                description: "Fast development with minimal resource usage".to_string(),
                settings: ServerSettings {
                    name: crate::SERVER_NAME.to_string(),
                    version: crate::VERSION.to_string(),
                    memory_limit_mb: 1024,
                    batch_size: 10,
                    max_file_size_mb: 5,
                    disable_memory_limit: false,
                    exclude_dirs: vec![
                        "node_modules".to_string(),
                        "target".to_string(),
                        ".git".to_string(),
                    ],
                    include_extensions: Some(vec![
                        "py".to_string(),
                        "js".to_string(),
                        "ts".to_string(),
                        "rs".to_string(),
                    ]),
                    dependency_mode: DependencyMode::Exclude,
                    default_timeout: Duration::from_secs(30),
                    max_concurrent_operations: 4,
                    enable_streaming: true,
                    max_response_size: 50_000,
                },
                tools: ToolsConfig {
                    enabled_categories: vec![
                        ToolCategory::CoreNavigation,
                        ToolCategory::SearchDiscovery,
                    ],
                    disabled_tools: vec!["analyze_transitive_dependencies".to_string()],
                    tool_configs: HashMap::new(),
                    enablement_rules: vec![],
                },
                monitoring: MonitoringConfig {
                    enabled: true,
                    collection_interval: Duration::from_secs(60),
                    monitor_memory: true,
                    monitor_response_times: true,
                    monitor_errors: true,
                    export_metrics: false,
                    metrics_export_path: None,
                    alert_thresholds: AlertThresholds {
                        max_memory_mb: 2048,
                        max_response_time_ms: 10000,
                        max_error_rate: 0.1,
                        min_success_rate: 0.9,
                    },
                },
                security: SecurityConfig {
                    enable_audit_log: false,
                    audit_log_path: None,
                    allowed_paths: vec![],
                    denied_paths: vec![],
                    max_analysis_depth: 100,
                    validate_paths: true,
                    rate_limiting: RateLimitConfig {
                        enabled: false,
                        requests_per_minute: 100,
                        max_concurrent: 10,
                        burst_size: 20,
                    },
                },
                caching: CachingConfig {
                    enabled: true,
                    cache_dir: PathBuf::from("./cache/dev"),
                    max_cache_size_mb: 256,
                    analysis_ttl: Duration::from_secs(3600),
                    content_ttl: Duration::from_secs(1800),
                    enable_compression: false,
                    cleanup_interval: Duration::from_secs(3600),
                },
            },
        );

        // Production profile
        self.profiles.insert(
            "production".to_string(),
            CodePrismProfile {
                name: "production".to_string(),
                description: "Production deployment with high performance and monitoring"
                    .to_string(),
                settings: ServerSettings {
                    name: crate::SERVER_NAME.to_string(),
                    version: crate::VERSION.to_string(),
                    memory_limit_mb: 8192,
                    batch_size: 50,
                    max_file_size_mb: 25,
                    disable_memory_limit: false,
                    exclude_dirs: vec![
                        "node_modules".to_string(),
                        "target".to_string(),
                        ".git".to_string(),
                        "vendor".to_string(),
                        "dist".to_string(),
                        "build".to_string(),
                    ],
                    include_extensions: None, // Include all supported extensions
                    dependency_mode: DependencyMode::Smart,
                    default_timeout: Duration::from_secs(120),
                    max_concurrent_operations: 12,
                    enable_streaming: true,
                    max_response_size: 150_000,
                },
                tools: ToolsConfig {
                    enabled_categories: vec![
                        ToolCategory::CoreNavigation,
                        ToolCategory::SearchDiscovery,
                        ToolCategory::Analysis,
                        ToolCategory::Workflow,
                    ],
                    disabled_tools: vec![],
                    tool_configs: HashMap::new(),
                    enablement_rules: vec![EnablementRule {
                        name: "large_repository".to_string(),
                        condition: EnablementCondition::RepositorySize { max_size_mb: 1000 },
                        actions: vec![EnablementAction::Disable {
                            tools: vec!["find_duplicates".to_string()],
                        }],
                    }],
                },
                monitoring: MonitoringConfig {
                    enabled: true,
                    collection_interval: Duration::from_secs(30),
                    monitor_memory: true,
                    monitor_response_times: true,
                    monitor_errors: true,
                    export_metrics: true,
                    metrics_export_path: Some(PathBuf::from("./metrics")),
                    alert_thresholds: AlertThresholds {
                        max_memory_mb: 10240,
                        max_response_time_ms: 30000,
                        max_error_rate: 0.05,
                        min_success_rate: 0.95,
                    },
                },
                security: SecurityConfig {
                    enable_audit_log: true,
                    audit_log_path: Some(PathBuf::from("./logs/audit.log")),
                    allowed_paths: vec![],
                    denied_paths: vec![
                        PathBuf::from("/etc"),
                        PathBuf::from("/var"),
                        PathBuf::from("/proc"),
                    ],
                    max_analysis_depth: 1000,
                    validate_paths: true,
                    rate_limiting: RateLimitConfig {
                        enabled: true,
                        requests_per_minute: 200,
                        max_concurrent: 15,
                        burst_size: 50,
                    },
                },
                caching: CachingConfig {
                    enabled: true,
                    cache_dir: PathBuf::from("./cache/prod"),
                    max_cache_size_mb: 2048,
                    analysis_ttl: Duration::from_secs(7200),
                    content_ttl: Duration::from_secs(3600),
                    enable_compression: true,
                    cleanup_interval: Duration::from_secs(1800),
                },
            },
        );

        // Enterprise profile
        self.profiles.insert(
            "enterprise".to_string(),
            CodePrismProfile {
                name: "enterprise".to_string(),
                description: "Enterprise deployment with maximum performance and security"
                    .to_string(),
                settings: ServerSettings {
                    name: crate::SERVER_NAME.to_string(),
                    version: crate::VERSION.to_string(),
                    memory_limit_mb: 16384,
                    batch_size: 100,
                    max_file_size_mb: 50,
                    disable_memory_limit: false,
                    exclude_dirs: vec![
                        "node_modules".to_string(),
                        "target".to_string(),
                        ".git".to_string(),
                        "vendor".to_string(),
                        "dist".to_string(),
                        "build".to_string(),
                        "coverage".to_string(),
                    ],
                    include_extensions: None,
                    dependency_mode: DependencyMode::Smart,
                    default_timeout: Duration::from_secs(300),
                    max_concurrent_operations: 24,
                    enable_streaming: true,
                    max_response_size: 500_000,
                },
                tools: ToolsConfig {
                    enabled_categories: vec![
                        ToolCategory::CoreNavigation,
                        ToolCategory::SearchDiscovery,
                        ToolCategory::Analysis,
                        ToolCategory::Workflow,
                    ],
                    disabled_tools: vec![],
                    tool_configs: HashMap::new(),
                    enablement_rules: vec![],
                },
                monitoring: MonitoringConfig {
                    enabled: true,
                    collection_interval: Duration::from_secs(15),
                    monitor_memory: true,
                    monitor_response_times: true,
                    monitor_errors: true,
                    export_metrics: true,
                    metrics_export_path: Some(PathBuf::from("./metrics")),
                    alert_thresholds: AlertThresholds {
                        max_memory_mb: 20480,
                        max_response_time_ms: 60000,
                        max_error_rate: 0.02,
                        min_success_rate: 0.98,
                    },
                },
                security: SecurityConfig {
                    enable_audit_log: true,
                    audit_log_path: Some(PathBuf::from("./logs/audit.log")),
                    allowed_paths: vec![],
                    denied_paths: vec![
                        PathBuf::from("/etc"),
                        PathBuf::from("/var"),
                        PathBuf::from("/proc"),
                        PathBuf::from("/sys"),
                    ],
                    max_analysis_depth: 10000,
                    validate_paths: true,
                    rate_limiting: RateLimitConfig {
                        enabled: true,
                        requests_per_minute: 500,
                        max_concurrent: 30,
                        burst_size: 100,
                    },
                },
                caching: CachingConfig {
                    enabled: true,
                    cache_dir: PathBuf::from("./cache/enterprise"),
                    max_cache_size_mb: 8192,
                    analysis_ttl: Duration::from_secs(14400),
                    content_ttl: Duration::from_secs(7200),
                    enable_compression: true,
                    cleanup_interval: Duration::from_secs(900),
                },
            },
        );

        info!(
            "Registered {} built-in configuration profiles",
            self.profiles.len()
        );
    }

    /// Get available profile names
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&CodePrismProfile> {
        self.profiles.get(name)
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, name: String) -> Result<()> {
        if self.profiles.contains_key(&name) {
            self.active_profile = Some(name.clone());
            info!("Activated configuration profile: {}", name);
            Ok(())
        } else {
            Err(crate::Error::server_init(format!(
                "Profile '{name}' not found"
            )))
        }
    }

    /// Get active profile
    pub fn get_active_profile(&self) -> Option<&CodePrismProfile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Register a custom profile
    pub fn register_profile(&mut self, profile: CodePrismProfile) {
        let name = profile.name.clone();
        self.profiles.insert(name.clone(), profile);
        info!("Registered custom configuration profile: {}", name);
    }

    /// Validate a configuration profile
    pub fn validate_profile(&self, profile: &CodePrismProfile) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Validate memory settings
        if profile.settings.memory_limit_mb < 512 {
            warnings.push("Memory limit is very low, may cause performance issues".to_string());
        }

        if profile.settings.memory_limit_mb > 32768 {
            warnings
                .push("Memory limit is very high, ensure system has sufficient RAM".to_string());
        }

        // Validate batch size
        if profile.settings.batch_size > 200 {
            warnings.push("Batch size is very high, may cause memory pressure".to_string());
        }

        // Validate file size limit
        if profile.settings.max_file_size_mb > 100 {
            warnings
                .push("Max file size is very high, may cause long processing times".to_string());
        }

        // Validate timeout
        if profile.settings.default_timeout.as_secs() > 600 {
            warnings.push("Default timeout is very high, clients may disconnect".to_string());
        }

        // Validate caching
        if profile.caching.enabled && profile.caching.max_cache_size_mb > 10240 {
            warnings.push("Cache size is very large, ensure sufficient disk space".to_string());
        }

        // Validate security settings
        if !profile.security.validate_paths {
            warnings.push("Path validation is disabled, security risk in production".to_string());
        }

        if profile.security.rate_limiting.enabled
            && profile.security.rate_limiting.requests_per_minute > 1000
        {
            warnings.push("Rate limit is very high, may not prevent abuse effectively".to_string());
        }

        Ok(warnings)
    }

    /// Create a profile from environment variables
    pub fn profile_from_env() -> Result<CodePrismProfile> {
        let profile_name =
            std::env::var("CODEPRISM_PROFILE").unwrap_or_else(|_| "development".to_string());

        // Start with base profile if it exists, otherwise use development defaults
        let base_profile = if profile_name == "development"
            || profile_name == "production"
            || profile_name == "enterprise"
        {
            let manager = Self::new();
            manager.get_profile(&profile_name).cloned()
        } else {
            None
        };

        let mut profile = base_profile.unwrap_or_else(|| {
            let manager = Self::new();
            manager.get_profile("development").unwrap().clone()
        });

        // Override with environment variables
        if let Ok(memory_limit) = std::env::var("CODEPRISM_MEMORY_LIMIT_MB") {
            if let Ok(limit) = memory_limit.parse::<usize>() {
                profile.settings.memory_limit_mb = limit;
            }
        }

        if let Ok(batch_size) = std::env::var("CODEPRISM_BATCH_SIZE") {
            if let Ok(size) = batch_size.parse::<usize>() {
                profile.settings.batch_size = size;
            }
        }

        if let Ok(timeout) = std::env::var("CODEPRISM_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                profile.settings.default_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(enable_cache) = std::env::var("CODEPRISM_ENABLE_CACHE") {
            profile.caching.enabled = enable_cache.to_lowercase() == "true";
        }

        if let Ok(cache_dir) = std::env::var("CODEPRISM_CACHE_DIR") {
            profile.caching.cache_dir = PathBuf::from(cache_dir);
        }

        profile.name = format!("{profile_name}_env");
        profile.description = format!("Environment-configured {profile_name} profile");

        Ok(profile)
    }
}

impl Config {
    /// Create configuration from environment variables
    pub async fn from_env() -> Result<Self> {
        let profile = ConfigProfileManager::profile_from_env()?;
        let manager = ConfigProfileManager::new();

        Ok(Self { profile, manager })
    }

    /// Load configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let extension = path_ref.extension().and_then(|s| s.to_str());
        let content = tokio::fs::read_to_string(path_ref).await?;

        let profile: CodePrismProfile = match extension {
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            _ => {
                // Try to detect format by trying each parser
                toml::from_str(&content)
                    .or_else(|_| serde_yaml::from_str(&content))
                    .or_else(|_| serde_json::from_str(&content))?
            }
        };

        let manager = ConfigProfileManager::new();
        Ok(Self { profile, manager })
    }

    /// Save configuration to a file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::to_string_pretty(&self.profile)?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(&self.profile)?,
            Some("json") => serde_json::to_string_pretty(&self.profile)?,
            _ => toml::to_string_pretty(&self.profile)?, // Default to TOML
        };
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        if self.profile.settings.name.is_empty() {
            return Err(crate::Error::server_init("Server name cannot be empty"));
        }

        if self.profile.settings.max_concurrent_operations == 0 {
            return Err(crate::Error::server_init(
                "Max concurrent operations must be greater than 0",
            ));
        }

        if self.profile.settings.max_file_size_mb == 0 {
            return Err(crate::Error::server_init(
                "Max file size must be greater than 0",
            ));
        }

        // Return validation warnings
        self.manager.validate_profile(&self.profile)
    }

    /// Check if a tool is enabled based on configuration
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        // Check if tool is explicitly disabled
        if self
            .profile
            .tools
            .disabled_tools
            .contains(&tool_name.to_string())
        {
            return false;
        }

        // Check if tool category is enabled
        let tool_category = match tool_name {
            "trace_path" | "find_dependencies" | "find_references" | "explain_symbol"
            | "search_symbols" => Some(ToolCategory::CoreNavigation),
            "search_content" | "find_patterns" | "semantic_search" | "search_by_type"
            | "advanced_search" => Some(ToolCategory::SearchDiscovery),
            "analyze_complexity"
            | "analyze_control_flow"
            | "analyze_code_quality"
            | "analyze_performance" => Some(ToolCategory::Analysis),
            "provide_guidance" | "optimize_code" | "batch_process" | "workflow_automation" => {
                Some(ToolCategory::Workflow)
            }
            _ => None,
        };

        if let Some(category) = tool_category {
            self.profile.tools.enabled_categories.contains(&category)
        } else {
            true // Enable unknown tools by default
        }
    }

    /// Get tool-specific configuration
    pub fn get_tool_config(&self, tool_name: &str) -> Option<&ToolConfig> {
        self.profile.tools.tool_configs.get(tool_name)
    }

    /// Get server configuration for backward compatibility
    pub fn server(&self) -> ServerConfig {
        ServerConfig {
            name: self.profile.settings.name.clone(),
            version: self.profile.settings.version.clone(),
            max_concurrent_tools: self.profile.settings.max_concurrent_operations,
            request_timeout_secs: self.profile.settings.default_timeout.as_secs(),
        }
    }

    /// Get tools configuration for backward compatibility
    pub fn tools(&self) -> ToolsConfigCompat {
        ToolsConfigCompat {
            enable_core: self
                .profile
                .tools
                .enabled_categories
                .contains(&ToolCategory::CoreNavigation),
            enable_search: self
                .profile
                .tools
                .enabled_categories
                .contains(&ToolCategory::SearchDiscovery),
            enable_analysis: self
                .profile
                .tools
                .enabled_categories
                .contains(&ToolCategory::Analysis),
            enable_workflow: self
                .profile
                .tools
                .enabled_categories
                .contains(&ToolCategory::Workflow),
        }
    }

    /// Get analysis configuration for backward compatibility
    pub fn analysis(&self) -> AnalysisConfigCompat {
        AnalysisConfigCompat {
            max_file_size_bytes: self.profile.settings.max_file_size_mb * 1024 * 1024,
            max_files_per_request: self.profile.settings.batch_size,
            enable_caching: self.profile.caching.enabled,
            cache_ttl_secs: self.profile.caching.analysis_ttl.as_secs(),
        }
    }
}

/// Backward compatibility structures
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub max_concurrent_tools: usize,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct ToolsConfigCompat {
    pub enable_core: bool,
    pub enable_search: bool,
    pub enable_analysis: bool,
    pub enable_workflow: bool,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfigCompat {
    pub max_file_size_bytes: usize,
    pub max_files_per_request: usize,
    pub enable_caching: bool,
    pub cache_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        let manager = ConfigProfileManager::new();
        let profile = manager.get_profile("development").unwrap().clone();
        Self { profile, manager }
    }
}

impl Default for ConfigProfileManager {
    fn default() -> Self {
        Self::new()
    }
}
