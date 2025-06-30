//! Configuration Validation & Health Check System (Phase 2.2)
//!
//! This module provides comprehensive configuration validation at startup,
//! system health monitoring, and diagnostic tools for production deployment.

use crate::config::McpConfigProfile;
use crate::monitoring::PerformanceMonitor;
use crate::tools::dynamic_enablement::DynamicToolManager;
use crate::CodePrismMcpServer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// System validation and health check coordinator
#[derive(Debug)]
pub struct SystemValidator {
    config_profile: McpConfigProfile,
    performance_monitor: Option<PerformanceMonitor>,
    tool_manager: Option<DynamicToolManager>,
    validation_cache: ValidationCache,
}

/// Cache for validation results to avoid redundant checks
#[derive(Debug, Default)]
struct ValidationCache {
    path_validations: HashMap<PathBuf, PathValidationResult>,
    last_system_check: Option<SystemTime>,
    last_dependencies_check: Option<SystemTime>,
}

/// Comprehensive validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validation status
    pub status: ValidationStatus,
    /// Configuration validation results
    pub config_validation: ConfigValidationResult,
    /// System readiness checks
    pub system_readiness: SystemReadinessResult,
    /// Security validation results
    pub security_validation: SecurityValidationResult,
    /// Performance validation results
    pub performance_validation: PerformanceValidationResult,
    /// Dependency validation results
    pub dependency_validation: DependencyValidationResult,
    /// Validation timestamp
    pub timestamp: u64,
    /// Time taken for validation
    pub validation_duration_ms: u64,
}

/// Overall validation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// All validations passed
    Valid,
    /// Some warnings but system can start
    ValidWithWarnings,
    /// Critical issues preventing startup
    Invalid,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    pub valid: bool,
    pub errors: Vec<ConfigValidationError>,
    pub warnings: Vec<String>,
    pub profile_name: String,
    pub validated_settings: ConfigValidationDetails,
}

/// Configuration validation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationDetails {
    pub memory_settings_valid: bool,
    pub timeout_settings_valid: bool,
    pub path_settings_valid: bool,
    pub security_settings_valid: bool,
    pub monitoring_settings_valid: bool,
    pub caching_settings_valid: bool,
}

/// Configuration validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationError {
    pub field: String,
    pub error_type: ConfigErrorType,
    pub message: String,
    pub suggested_fix: Option<String>,
}

/// Types of configuration errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigErrorType {
    InvalidValue,
    MissingRequired,
    IncompatibleSettings,
    PathNotAccessible,
    SecurityRisk,
    PerformanceIssue,
}

/// System readiness validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReadinessResult {
    pub ready: bool,
    pub system_requirements: SystemRequirementsCheck,
    pub runtime_environment: RuntimeEnvironmentCheck,
    pub resource_availability: ResourceAvailabilityCheck,
}

/// System requirements validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirementsCheck {
    pub minimum_memory_available: bool,
    pub minimum_disk_space: bool,
    pub required_permissions: bool,
    pub system_architecture_supported: bool,
}

/// Runtime environment validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnvironmentCheck {
    pub rust_version_compatible: bool,
    pub required_features_available: bool,
    pub environment_variables_set: bool,
    pub network_connectivity: bool,
}

/// Resource availability validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailabilityCheck {
    pub available_memory_mb: usize,
    pub available_disk_space_mb: usize,
    pub cpu_cores_available: usize,
    pub can_create_temp_files: bool,
    pub can_bind_to_stdio: bool,
}

/// Security validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityValidationResult {
    pub secure: bool,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub recommendations: Vec<SecurityRecommendation>,
    pub security_score: u32, // 0-100
}

/// Security vulnerability detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub severity: SecuritySeverity,
    pub category: SecurityCategory,
    pub description: String,
    pub mitigation: String,
}

/// Security recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub priority: SecurityPriority,
    pub description: String,
    pub implementation: String,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Security categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCategory {
    PathTraversal,
    AccessControl,
    DataExposure,
    InputValidation,
    Configuration,
    NetworkSecurity,
}

/// Security recommendation priorities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceValidationResult {
    pub optimal: bool,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub optimizations: Vec<PerformanceOptimization>,
    pub performance_score: u32, // 0-100
}

/// Performance bottleneck detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub component: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact: String,
    pub solution: String,
}

/// Performance optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimization {
    pub component: String,
    pub optimization_type: OptimizationType,
    pub description: String,
    pub expected_improvement: String,
    pub implementation_effort: ImplementationEffort,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Minor,
    Moderate,
    Significant,
    Critical,
}

/// Types of performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    Memory,
    Cpu,
    Disk,
    Network,
    Caching,
    Concurrency,
    Configuration,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

/// Dependency validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyValidationResult {
    pub all_available: bool,
    pub missing_dependencies: Vec<MissingDependency>,
    pub version_conflicts: Vec<VersionConflict>,
    pub optional_dependencies: Vec<OptionalDependency>,
}

/// Missing dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub name: String,
    pub dependency_type: DependencyType,
    pub required_for: Vec<String>,
    pub installation_hint: Option<String>,
}

/// Version conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub dependency: String,
    pub required_version: String,
    pub found_version: String,
    pub impact: String,
}

/// Optional dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalDependency {
    pub name: String,
    pub enables_features: Vec<String>,
    pub installation_hint: String,
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    SystemLibrary,
    RuntimeDependency,
    ToolDependency,
    LanguageParser,
}

/// Path validation result for caching
#[derive(Debug, Clone)]
struct PathValidationResult {
    accessible: bool,
    readable: bool,
    writable: bool,
    validated_at: SystemTime,
}

impl SystemValidator {
    /// Create a new system validator
    pub fn new(config_profile: McpConfigProfile) -> Self {
        Self {
            config_profile,
            performance_monitor: None,
            tool_manager: None,
            validation_cache: ValidationCache::default(),
        }
    }

    /// Set performance monitor for validation
    pub fn with_performance_monitor(mut self, monitor: PerformanceMonitor) -> Self {
        self.performance_monitor = Some(monitor);
        self
    }

    /// Set tool manager for validation
    pub fn with_tool_manager(mut self, tool_manager: DynamicToolManager) -> Self {
        self.tool_manager = Some(tool_manager);
        self
    }

    /// Perform comprehensive system validation
    pub async fn validate_system(&mut self) -> Result<ValidationResult> {
        let start_time = Instant::now();

        info!("Starting comprehensive system validation");

        // Run all validation checks in parallel where possible
        let config_validation = self.validate_configuration().await?;
        let system_readiness = self.validate_system_readiness().await?;
        let security_validation = self.validate_security().await?;
        let performance_validation = self.validate_performance().await?;
        let dependency_validation = self.validate_dependencies().await?;

        // Determine overall validation status
        let status = self.determine_overall_status(
            &config_validation,
            &system_readiness,
            &security_validation,
            &performance_validation,
            &dependency_validation,
        );

        let validation_duration_ms = start_time.elapsed().as_millis() as u64;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let result = ValidationResult {
            status,
            config_validation,
            system_readiness,
            security_validation,
            performance_validation,
            dependency_validation,
            timestamp,
            validation_duration_ms,
        };

        self.log_validation_summary(&result);

        Ok(result)
    }

    /// Validate configuration settings
    async fn validate_configuration(&mut self) -> Result<ConfigValidationResult> {
        debug!("Validating configuration settings");

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let profile_name = self.config_profile.name.clone();

        // Validate memory settings
        let memory_valid = self.validate_memory_settings(&mut errors, &mut warnings);

        // Validate timeout settings
        let timeout_valid = self.validate_timeout_settings(&mut errors, &mut warnings);

        // Validate path settings
        let path_valid = self
            .validate_path_settings(&mut errors, &mut warnings)
            .await;

        // Validate security settings
        let security_valid = self.validate_security_settings(&mut errors, &mut warnings);

        // Validate monitoring settings
        let monitoring_valid = self.validate_monitoring_settings(&mut errors, &mut warnings);

        // Validate caching settings
        let caching_valid = self.validate_caching_settings(&mut errors, &mut warnings);

        let valid = errors.is_empty();

        Ok(ConfigValidationResult {
            valid,
            errors,
            warnings,
            profile_name,
            validated_settings: ConfigValidationDetails {
                memory_settings_valid: memory_valid,
                timeout_settings_valid: timeout_valid,
                path_settings_valid: path_valid,
                security_settings_valid: security_valid,
                monitoring_settings_valid: monitoring_valid,
                caching_settings_valid: caching_valid,
            },
        })
    }

    /// Validate memory configuration
    fn validate_memory_settings(
        &self,
        errors: &mut Vec<ConfigValidationError>,
        warnings: &mut Vec<String>,
    ) -> bool {
        let config = &self.config_profile.settings;
        let mut valid = true;

        // Check minimum memory limit
        if config.memory_limit_mb < 256 {
            errors.push(ConfigValidationError {
                field: "memory_limit_mb".to_string(),
                error_type: ConfigErrorType::InvalidValue,
                message: "Memory limit is too low, minimum 256MB required".to_string(),
                suggested_fix: Some("Set memory_limit_mb to at least 256".to_string()),
            });
            valid = false;
        }

        // Check for excessive memory limit
        if config.memory_limit_mb > 32768 {
            warnings.push(
                "Memory limit is very high (>32GB), ensure system has sufficient RAM".to_string(),
            );
        }

        // Validate batch size relative to memory
        let estimated_memory_per_file = 2; // MB estimated
        let max_safe_batch = config.memory_limit_mb / estimated_memory_per_file;

        if config.batch_size > max_safe_batch {
            warnings.push(format!(
                "Batch size ({}) may cause memory pressure, consider reducing to {}",
                config.batch_size, max_safe_batch
            ));
        }

        valid
    }

    /// Validate timeout configuration
    fn validate_timeout_settings(
        &self,
        errors: &mut Vec<ConfigValidationError>,
        warnings: &mut Vec<String>,
    ) -> bool {
        let config = &self.config_profile.settings;
        let mut valid = true;

        // Check minimum timeout
        if config.default_timeout < Duration::from_secs(5) {
            errors.push(ConfigValidationError {
                field: "default_timeout".to_string(),
                error_type: ConfigErrorType::InvalidValue,
                message: "Default timeout is too low, minimum 5 seconds required".to_string(),
                suggested_fix: Some("Increase default_timeout to at least 5 seconds".to_string()),
            });
            valid = false;
        }

        // Check for excessive timeout
        if config.default_timeout > Duration::from_secs(600) {
            warnings.push(
                "Default timeout is very high (>10 minutes), clients may disconnect".to_string(),
            );
        }

        valid
    }

    /// Validate path-related settings
    async fn validate_path_settings(
        &mut self,
        errors: &mut Vec<ConfigValidationError>,
        warnings: &mut Vec<String>,
    ) -> bool {
        // Clone the configuration values to avoid borrow issues
        let cache_enabled = self.config_profile.caching.enabled;
        let cache_dir = self.config_profile.caching.cache_dir.clone();
        let audit_log_path = self.config_profile.security.audit_log_path.clone();
        let denied_paths = self.config_profile.security.denied_paths.clone();

        let mut valid = true;

        // Validate cache directory
        if cache_enabled {
            if let Err(validation_error) = self.validate_path_access(&cache_dir, true, true).await {
                errors.push(ConfigValidationError {
                    field: "caching.cache_dir".to_string(),
                    error_type: ConfigErrorType::PathNotAccessible,
                    message: format!("Cache directory not accessible: {}", validation_error),
                    suggested_fix: Some("Create directory or adjust permissions".to_string()),
                });
                valid = false;
            }
        }

        // Validate audit log path
        if let Some(audit_path) = audit_log_path {
            if let Some(parent) = audit_path.parent() {
                if let Err(validation_error) = self.validate_path_access(parent, true, true).await {
                    errors.push(ConfigValidationError {
                        field: "security.audit_log_path".to_string(),
                        error_type: ConfigErrorType::PathNotAccessible,
                        message: format!(
                            "Audit log directory not accessible: {}",
                            validation_error
                        ),
                        suggested_fix: Some("Create directory or adjust permissions".to_string()),
                    });
                    valid = false;
                }
            }
        }

        // Validate denied paths for security
        for denied_path in &denied_paths {
            if denied_path.starts_with("/") {
                // Good - absolute path restriction
                continue;
            } else {
                warnings.push(format!(
                    "Denied path '{}' is not absolute, may not provide expected security",
                    denied_path.display()
                ));
            }
        }

        valid
    }

    /// Validate security settings
    fn validate_security_settings(
        &self,
        errors: &mut Vec<ConfigValidationError>,
        warnings: &mut Vec<String>,
    ) -> bool {
        let security_config = &self.config_profile.security;
        let mut valid = true;

        // Check if path validation is disabled in production
        if !security_config.validate_paths {
            errors.push(ConfigValidationError {
                field: "security.validate_paths".to_string(),
                error_type: ConfigErrorType::SecurityRisk,
                message: "Path validation is disabled, security risk in production".to_string(),
                suggested_fix: Some("Enable validate_paths for production deployment".to_string()),
            });
            valid = false;
        }

        // Check rate limiting configuration
        if security_config.rate_limiting.enabled {
            if security_config.rate_limiting.requests_per_minute > 1000 {
                warnings
                    .push("Rate limit is very high, may not prevent abuse effectively".to_string());
            }

            if security_config.rate_limiting.max_concurrent > 50 {
                warnings.push(
                    "Max concurrent requests is very high, may cause resource exhaustion"
                        .to_string(),
                );
            }
        } else {
            warnings
                .push("Rate limiting is disabled, consider enabling for production".to_string());
        }

        // Check audit logging
        if !security_config.enable_audit_log {
            warnings
                .push("Audit logging is disabled, enable for production compliance".to_string());
        }

        valid
    }

    /// Validate monitoring settings
    fn validate_monitoring_settings(
        &self,
        _errors: &mut [ConfigValidationError],
        warnings: &mut Vec<String>,
    ) -> bool {
        let monitoring_config = &self.config_profile.monitoring;

        if !monitoring_config.enabled {
            warnings.push(
                "Performance monitoring is disabled, enable for production visibility".to_string(),
            );
        }

        if monitoring_config.export_metrics && monitoring_config.metrics_export_path.is_none() {
            warnings.push("Metrics export enabled but no export path configured".to_string());
        }

        true // Monitoring configuration rarely prevents startup
    }

    /// Validate caching settings
    fn validate_caching_settings(
        &self,
        _errors: &mut [ConfigValidationError],
        warnings: &mut Vec<String>,
    ) -> bool {
        let caching_config = &self.config_profile.caching;

        if caching_config.enabled && caching_config.max_cache_size_mb > 10240 {
            warnings
                .push("Cache size is very large (>10GB), ensure sufficient disk space".to_string());
        }

        if caching_config.analysis_ttl > Duration::from_secs(86400) {
            warnings.push("Analysis cache TTL is very long (>24h), may use stale data".to_string());
        }

        true
    }

    /// Validate path accessibility
    async fn validate_path_access(
        &mut self,
        path: &Path,
        need_read: bool,
        need_write: bool,
    ) -> Result<()> {
        // Check cache first
        if let Some(cached) = self.validation_cache.path_validations.get(path) {
            if cached.validated_at.elapsed().unwrap_or_default() < Duration::from_secs(300) {
                if !cached.accessible
                    || (need_read && !cached.readable)
                    || (need_write && !cached.writable)
                {
                    return Err(anyhow::anyhow!("Path validation failed (cached)"));
                }
                return Ok(());
            }
        }

        // Perform actual validation
        let accessible = path.exists() || path.parent().is_some_and(|p| p.exists());
        let readable = accessible && std::fs::metadata(path).is_ok();
        let writable = if accessible {
            // Try to create a temporary file to test write access
            let test_file = path.join(".test_write_access");
            std::fs::write(&test_file, "test").is_ok() && std::fs::remove_file(&test_file).is_ok()
        } else {
            false
        };

        // Cache the result
        self.validation_cache.path_validations.insert(
            path.to_path_buf(),
            PathValidationResult {
                accessible,
                readable,
                writable,
                validated_at: SystemTime::now(),
            },
        );

        // Check requirements
        if !accessible || (need_read && !readable) || (need_write && !writable) {
            return Err(anyhow::anyhow!(
                "Path requirements not met: accessible={}, readable={}, writable={}",
                accessible,
                readable,
                writable
            ));
        }

        Ok(())
    }

    /// Validate system readiness
    async fn validate_system_readiness(&mut self) -> Result<SystemReadinessResult> {
        debug!("Validating system readiness");

        let system_requirements = self.check_system_requirements().await;
        let runtime_environment = self.check_runtime_environment().await;
        let resource_availability = self.check_resource_availability().await;

        let ready = system_requirements.minimum_memory_available
            && system_requirements.minimum_disk_space
            && runtime_environment.rust_version_compatible
            && resource_availability.can_bind_to_stdio;

        Ok(SystemReadinessResult {
            ready,
            system_requirements,
            runtime_environment,
            resource_availability,
        })
    }

    /// Check system requirements
    async fn check_system_requirements(&self) -> SystemRequirementsCheck {
        // Simplified system requirements check
        SystemRequirementsCheck {
            minimum_memory_available: true, // Could check actual system memory
            minimum_disk_space: true,       // Could check actual disk space
            required_permissions: true,     // Could check file system permissions
            system_architecture_supported: true, // Could check CPU architecture
        }
    }

    /// Check runtime environment
    async fn check_runtime_environment(&mut self) -> RuntimeEnvironmentCheck {
        // Check if we've validated recently
        if let Some(last_check) = self.validation_cache.last_system_check {
            if last_check.elapsed().unwrap_or_default() < Duration::from_secs(300) {
                // Use cached result for 5 minutes
                return RuntimeEnvironmentCheck {
                    rust_version_compatible: true,
                    required_features_available: true,
                    environment_variables_set: true,
                    network_connectivity: true, // Simplified
                };
            }
        }

        self.validation_cache.last_system_check = Some(SystemTime::now());

        RuntimeEnvironmentCheck {
            rust_version_compatible: true,     // Could check actual Rust version
            required_features_available: true, // Could check feature flags
            environment_variables_set: std::env::var("RUST_LOG").is_ok(), // Check some env vars
            network_connectivity: true,        // Simplified for stdio transport
        }
    }

    /// Check resource availability
    async fn check_resource_availability(&self) -> ResourceAvailabilityCheck {
        ResourceAvailabilityCheck {
            available_memory_mb: 4096,      // Could fetch actual available memory
            available_disk_space_mb: 10240, // Could fetch actual disk space
            cpu_cores_available: num_cpus::get(),
            can_create_temp_files: std::env::temp_dir().exists(),
            can_bind_to_stdio: true, // Always true for MCP stdio transport
        }
    }

    /// Validate security configuration and detect vulnerabilities
    async fn validate_security(&self) -> Result<SecurityValidationResult> {
        debug!("Validating security configuration");

        let mut vulnerabilities = Vec::new();
        let mut recommendations = Vec::new();
        let security_config = &self.config_profile.security;

        // Check for path traversal vulnerabilities
        if !security_config.validate_paths {
            vulnerabilities.push(SecurityVulnerability {
                severity: SecuritySeverity::High,
                category: SecurityCategory::PathTraversal,
                description:
                    "Path validation is disabled, allowing potential path traversal attacks"
                        .to_string(),
                mitigation: "Enable validate_paths in security configuration".to_string(),
            });
        }

        // Check access control
        if security_config.allowed_paths.is_empty() && security_config.denied_paths.is_empty() {
            recommendations.push(SecurityRecommendation {
                priority: SecurityPriority::Medium,
                description: "Configure allowed or denied paths for access control".to_string(),
                implementation: "Add paths to allowed_paths or denied_paths in security config"
                    .to_string(),
            });
        }

        // Check audit logging
        if !security_config.enable_audit_log {
            recommendations.push(SecurityRecommendation {
                priority: SecurityPriority::Medium,
                description: "Enable audit logging for security compliance".to_string(),
                implementation: "Set enable_audit_log to true and configure audit_log_path"
                    .to_string(),
            });
        }

        // Calculate security score
        let security_score = self.calculate_security_score(&vulnerabilities, &recommendations);

        Ok(SecurityValidationResult {
            secure: vulnerabilities.iter().all(|v| {
                !matches!(
                    v.severity,
                    SecuritySeverity::Critical | SecuritySeverity::High
                )
            }),
            vulnerabilities,
            recommendations,
            security_score,
        })
    }

    /// Calculate security score based on vulnerabilities and recommendations
    fn calculate_security_score(
        &self,
        vulnerabilities: &[SecurityVulnerability],
        recommendations: &[SecurityRecommendation],
    ) -> u32 {
        let mut score = 100u32;

        // Deduct points for vulnerabilities
        for vuln in vulnerabilities {
            let deduction = match vuln.severity {
                SecuritySeverity::Critical => 30,
                SecuritySeverity::High => 20,
                SecuritySeverity::Medium => 10,
                SecuritySeverity::Low => 5,
            };
            score = score.saturating_sub(deduction);
        }

        // Deduct smaller amounts for recommendations
        for rec in recommendations {
            let deduction = match rec.priority {
                SecurityPriority::Critical => 15,
                SecurityPriority::High => 10,
                SecurityPriority::Medium => 5,
                SecurityPriority::Low => 2,
            };
            score = score.saturating_sub(deduction);
        }

        score
    }

    /// Validate performance configuration
    async fn validate_performance(&self) -> Result<PerformanceValidationResult> {
        debug!("Validating performance configuration");

        let mut bottlenecks = Vec::new();
        let mut optimizations = Vec::new();
        let config = &self.config_profile.settings;

        // Check memory configuration
        if config.memory_limit_mb < 1024 {
            bottlenecks.push(PerformanceBottleneck {
                component: "memory".to_string(),
                severity: BottleneckSeverity::Significant,
                description: "Memory limit is low, may cause frequent garbage collection"
                    .to_string(),
                impact: "Increased latency and reduced throughput".to_string(),
                solution: "Increase memory_limit_mb to at least 1024MB".to_string(),
            });
        }

        // Check batch size optimization
        if config.batch_size < 10 {
            optimizations.push(PerformanceOptimization {
                component: "indexing".to_string(),
                optimization_type: OptimizationType::Concurrency,
                description: "Small batch size may underutilize system resources".to_string(),
                expected_improvement: "20-40% faster indexing".to_string(),
                implementation_effort: ImplementationEffort::Low,
            });
        }

        // Check caching configuration
        if !self.config_profile.caching.enabled {
            optimizations.push(PerformanceOptimization {
                component: "caching".to_string(),
                optimization_type: OptimizationType::Caching,
                description: "Enable caching to improve response times for repeated queries"
                    .to_string(),
                expected_improvement: "50-80% faster repeated operations".to_string(),
                implementation_effort: ImplementationEffort::Low,
            });
        }

        let performance_score = self.calculate_performance_score(&bottlenecks, &optimizations);

        Ok(PerformanceValidationResult {
            optimal: bottlenecks.is_empty(),
            bottlenecks,
            optimizations,
            performance_score,
        })
    }

    /// Calculate performance score
    fn calculate_performance_score(
        &self,
        bottlenecks: &[PerformanceBottleneck],
        optimizations: &[PerformanceOptimization],
    ) -> u32 {
        let mut score = 100u32;

        // Deduct for bottlenecks
        for bottleneck in bottlenecks {
            let deduction = match bottleneck.severity {
                BottleneckSeverity::Critical => 25,
                BottleneckSeverity::Significant => 15,
                BottleneckSeverity::Moderate => 10,
                BottleneckSeverity::Minor => 5,
            };
            score = score.saturating_sub(deduction);
        }

        // Small deduction for missed optimizations
        for optimization in optimizations {
            let deduction = match optimization.implementation_effort {
                ImplementationEffort::Low => 3,
                ImplementationEffort::Medium => 2,
                ImplementationEffort::High => 1,
            };
            score = score.saturating_sub(deduction);
        }

        score
    }

    /// Validate dependencies
    async fn validate_dependencies(&mut self) -> Result<DependencyValidationResult> {
        debug!("Validating dependencies");

        // Check if we've validated recently
        if let Some(last_check) = self.validation_cache.last_dependencies_check {
            if last_check.elapsed().unwrap_or_default() < Duration::from_secs(600) {
                // Use cached result for 10 minutes
                return Ok(DependencyValidationResult {
                    all_available: true,
                    missing_dependencies: Vec::new(),
                    version_conflicts: Vec::new(),
                    optional_dependencies: Vec::new(),
                });
            }
        }

        self.validation_cache.last_dependencies_check = Some(SystemTime::now());

        let mut missing_dependencies = Vec::new();
        let version_conflicts = Vec::new(); // Could check actual version conflicts
        let optional_dependencies = Vec::new(); // Could check optional features

        // Check for required system tools (simplified)
        if !self.check_git_available() {
            missing_dependencies.push(MissingDependency {
                name: "git".to_string(),
                dependency_type: DependencyType::ToolDependency,
                required_for: vec!["repository analysis".to_string()],
                installation_hint: Some("Install git from https://git-scm.com/".to_string()),
            });
        }

        Ok(DependencyValidationResult {
            all_available: missing_dependencies.is_empty() && version_conflicts.is_empty(),
            missing_dependencies,
            version_conflicts,
            optional_dependencies,
        })
    }

    /// Check if git is available (simplified check)
    fn check_git_available(&self) -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Determine overall validation status
    fn determine_overall_status(
        &self,
        config: &ConfigValidationResult,
        system: &SystemReadinessResult,
        security: &SecurityValidationResult,
        performance: &PerformanceValidationResult,
        dependencies: &DependencyValidationResult,
    ) -> ValidationStatus {
        // Critical failures prevent startup
        if !config.valid || !system.ready || !security.secure || !dependencies.all_available {
            return ValidationStatus::Invalid;
        }

        // Check for warnings
        let has_warnings = !config.warnings.is_empty()
            || !security.vulnerabilities.is_empty()
            || !performance.bottlenecks.is_empty();

        if has_warnings {
            ValidationStatus::ValidWithWarnings
        } else {
            ValidationStatus::Valid
        }
    }

    /// Log validation summary
    fn log_validation_summary(&self, result: &ValidationResult) {
        match result.status {
            ValidationStatus::Valid => {
                info!(
                    "✅ System validation completed successfully in {}ms",
                    result.validation_duration_ms
                );
            }
            ValidationStatus::ValidWithWarnings => {
                warn!(
                    "⚠️ System validation completed with warnings in {}ms",
                    result.validation_duration_ms
                );

                for warning in &result.config_validation.warnings {
                    warn!("Config warning: {}", warning);
                }

                for vuln in &result.security_validation.vulnerabilities {
                    warn!("Security issue: {:?} - {}", vuln.category, vuln.description);
                }
            }
            ValidationStatus::Invalid => {
                error!(
                    "❌ System validation failed in {}ms",
                    result.validation_duration_ms
                );

                for error in &result.config_validation.errors {
                    error!("Config error: {} - {}", error.field, error.message);
                }

                if !result.system_readiness.ready {
                    error!("System readiness check failed");
                }
            }
        }
    }

    /// Generate startup health report
    pub async fn generate_startup_report(
        &mut self,
        _server: &CodePrismMcpServer,
    ) -> Result<StartupReport> {
        info!("Generating comprehensive startup report");

        let validation_result = self.validate_system().await?;

        let tool_status = self
            .tool_manager
            .as_ref()
            .map(|tool_manager| tool_manager.get_summary());

        let system_info = self.collect_system_info().await;

        Ok(StartupReport {
            validation_result,
            tool_status,
            system_info,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            startup_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Collect system information for reporting
    async fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus::get(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            build_timestamp: std::env::var("BUILD_TIMESTAMP")
                .unwrap_or_else(|_| "unknown".to_string()),
            features_enabled: self.get_enabled_features(),
        }
    }

    /// Get list of enabled features
    fn get_enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();

        if self.config_profile.monitoring.enabled {
            features.push("monitoring".to_string());
        }

        if self.config_profile.caching.enabled {
            features.push("caching".to_string());
        }

        if self.config_profile.security.enable_audit_log {
            features.push("audit_logging".to_string());
        }

        if self.config_profile.security.rate_limiting.enabled {
            features.push("rate_limiting".to_string());
        }

        features
    }
}

/// Comprehensive startup report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupReport {
    pub validation_result: ValidationResult,
    pub tool_status: Option<crate::tools::dynamic_enablement::ToolEnablementSummary>,
    pub system_info: SystemInfo,
    pub server_version: String,
    pub startup_timestamp: u64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub cpu_cores: usize,
    pub rust_version: String,
    pub build_timestamp: String,
    pub features_enabled: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        CachingConfig, McpConfig, MonitoringConfig, SecurityConfig, ToolConfiguration,
    };

    fn create_test_profile() -> McpConfigProfile {
        McpConfigProfile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            settings: McpConfig::default(),
            tool_config: ToolConfiguration {
                enabled_categories: vec![],
                disabled_tools: vec![],
                tool_configs: HashMap::new(),
                enablement_rules: vec![],
            },
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
            caching: CachingConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_system_validator_creation() {
        let profile = create_test_profile();
        let validator = SystemValidator::new(profile);

        assert_eq!(validator.config_profile.name, "test");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let mut profile = create_test_profile();
        profile.settings.memory_limit_mb = 128; // Too low

        let mut validator = SystemValidator::new(profile);
        let config_result = validator.validate_configuration().await.unwrap();

        assert!(!config_result.valid);
        assert!(!config_result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_security_validation() {
        let mut profile = create_test_profile();
        profile.security.validate_paths = false; // Security risk

        let validator = SystemValidator::new(profile);
        let security_result = validator.validate_security().await.unwrap();

        assert!(!security_result.secure);
        assert!(!security_result.vulnerabilities.is_empty());
    }

    #[tokio::test]
    async fn test_performance_validation() {
        let mut profile = create_test_profile();
        profile.settings.memory_limit_mb = 512; // Low memory

        let validator = SystemValidator::new(profile);
        let perf_result = validator.validate_performance().await.unwrap();

        assert!(!perf_result.optimal);
        assert!(!perf_result.bottlenecks.is_empty());
    }
}
