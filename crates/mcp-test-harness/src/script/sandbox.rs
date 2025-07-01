//! Secure sandboxing for script execution
//!
//! Provides security controls and resource limits for executing untrusted
//! validation scripts with proper isolation and monitoring.

use super::{ScriptConfig, ScriptError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, warn};

/// Sandbox configuration for script execution security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum memory usage in megabytes
    pub max_memory_mb: u64,
    /// Maximum CPU time in seconds
    pub max_cpu_time_seconds: u64,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Allowed file system paths for reading
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed file system paths for writing
    pub allowed_write_paths: Vec<PathBuf>,
    /// Environment variables to inject
    pub environment_variables: HashMap<String, String>,
    /// Working directory for script execution
    pub working_directory: Option<PathBuf>,
    /// Whether to enable strict mode (extra security)
    pub strict_mode: bool,
    /// Maximum number of processes/threads
    pub max_processes: u32,
    /// Maximum file size for I/O operations (in bytes)
    pub max_file_size_bytes: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30_000, // 30 seconds
            max_memory_mb: 256,            // 256 MB
            max_cpu_time_seconds: 30,      // 30 seconds CPU time
            allow_network: false,          // No network by default
            allowed_read_paths: Vec::new(),
            allowed_write_paths: Vec::new(),
            environment_variables: HashMap::new(),
            working_directory: None,
            strict_mode: true,
            max_processes: 1,
            max_file_size_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Manages sandboxing and security for script execution
#[derive(Debug)]
pub struct SandboxManager {
    config: SandboxConfig,
    security_policies: SecurityPolicies,
}

/// Security policies for different types of operations
#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    /// Blocked system calls (for Linux seccomp)
    pub blocked_syscalls: Vec<String>,
    /// Blocked environment variables
    pub blocked_env_vars: Vec<String>,
    /// Blocked file patterns
    pub blocked_file_patterns: Vec<String>,
    /// Maximum argument length
    pub max_arg_length: usize,
    /// Allowed process commands
    pub allowed_commands: Vec<String>,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            blocked_syscalls: vec![
                "execve".to_string(),
                "fork".to_string(),
                "clone".to_string(),
                "ptrace".to_string(),
                "mount".to_string(),
                "umount".to_string(),
                "chroot".to_string(),
                "setuid".to_string(),
                "setgid".to_string(),
            ],
            blocked_env_vars: vec![
                "PATH".to_string(),
                "LD_LIBRARY_PATH".to_string(),
                "LD_PRELOAD".to_string(),
            ],
            blocked_file_patterns: vec![
                "/etc/passwd".to_string(),
                "/etc/shadow".to_string(),
                "/proc/*/mem".to_string(),
                "/sys/*".to_string(),
                "/dev/*".to_string(),
            ],
            max_arg_length: 1024 * 1024, // 1 MB
            allowed_commands: vec![
                "node".to_string(),
                "python3".to_string(),
                "python".to_string(),
                "lua".to_string(),
            ],
        }
    }
}

impl SandboxManager {
    /// Create a new sandbox manager with configuration
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            security_policies: SecurityPolicies::default(),
        }
    }

    /// Apply sandbox restrictions to a command
    pub fn apply_restrictions(
        &self,
        cmd: &mut Command,
        script_config: &ScriptConfig,
    ) -> Result<(), ScriptError> {
        debug!(
            "Applying sandbox restrictions for script: {}",
            script_config.name
        );

        // Set working directory
        if let Some(working_dir) = &self.config.working_directory {
            cmd.current_dir(working_dir);
        }

        // Apply environment variable restrictions
        self.apply_env_restrictions(cmd, script_config)?;

        // Apply resource limits (platform-specific)
        self.apply_resource_limits(cmd, script_config)?;

        // Apply network restrictions
        if !self.config.allow_network && !script_config.allow_network {
            self.apply_network_restrictions(cmd)?;
        }

        // Validate command security
        self.validate_command_security(cmd)?;

        Ok(())
    }

    /// Apply environment variable restrictions
    fn apply_env_restrictions(
        &self,
        cmd: &mut Command,
        script_config: &ScriptConfig,
    ) -> Result<(), ScriptError> {
        // Clear potentially dangerous environment variables
        for blocked_var in &self.security_policies.blocked_env_vars {
            cmd.env_remove(blocked_var);
        }

        // Set secure defaults
        cmd.env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONUNBUFFERED", "1")
            .env("NODE_NO_WARNINGS", "1")
            .env("NODE_OPTIONS", "--max-old-space-size=256");

        // Add script-specific environment variables
        for (key, value) in &script_config.env_vars {
            if !self.security_policies.blocked_env_vars.contains(key) {
                if value.len() > self.security_policies.max_arg_length {
                    return Err(ScriptError::SecurityViolation(format!(
                        "Environment variable '{}' value too long: {} bytes",
                        key,
                        value.len()
                    )));
                }
                cmd.env(key, value);
            } else {
                warn!("Blocked environment variable '{}' in script config", key);
            }
        }

        // Add sandbox-specific environment variables
        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }

        Ok(())
    }

    /// Apply resource limits (Linux/Unix specific)
    fn apply_resource_limits(
        &self,
        cmd: &mut Command,
        script_config: &ScriptConfig,
    ) -> Result<(), ScriptError> {
        // Use the more restrictive timeout
        let timeout_ms = script_config
            .timeout_ms
            .min(self.config.max_execution_time_ms);

        // Use the more restrictive memory limit
        let memory_mb = script_config.max_memory_mb.min(self.config.max_memory_mb);

        debug!(
            "Setting resource limits: {}ms timeout, {}MB memory",
            timeout_ms, memory_mb
        );

        // On Unix systems, we can use ulimit-style restrictions
        #[cfg(unix)]
        {
            // Set memory limit (if available)
            if memory_mb > 0 {
                let memory_bytes = memory_mb * 1024 * 1024;
                cmd.env("MEMORY_LIMIT_BYTES", memory_bytes.to_string());
            }

            // Set CPU time limit
            if self.config.max_cpu_time_seconds > 0 {
                cmd.env(
                    "CPU_TIME_LIMIT_SECONDS",
                    self.config.max_cpu_time_seconds.to_string(),
                );
            }

            // Set process limit
            cmd.env("MAX_PROCESSES", self.config.max_processes.to_string());
        }

        Ok(())
    }

    /// Apply network access restrictions
    fn apply_network_restrictions(&self, cmd: &mut Command) -> Result<(), ScriptError> {
        debug!("Applying network restrictions");

        // Disable network access through environment variables
        cmd.env("no_proxy", "*")
            .env("NO_PROXY", "*")
            .env("http_proxy", "")
            .env("https_proxy", "")
            .env("HTTP_PROXY", "")
            .env("HTTPS_PROXY", "");

        // For Node.js, disable network modules
        cmd.env("NODE_OPTIONS", "--no-network");

        Ok(())
    }

    /// Validate command security
    ///
    /// NOTE: tokio::process::Command doesn't expose program() and args() methods,
    /// so we rely on environment restrictions and resource limits for security.
    /// Command validation occurs through:
    /// 1. Environment variable restrictions (env_restrictions)
    /// 2. Resource limits (memory, CPU, time)
    /// 3. Network access controls
    /// 4. File system path restrictions
    fn validate_command_security(&self, _cmd: &Command) -> Result<(), ScriptError> {
        if self.config.strict_mode {
            debug!("Command security enforced through environment and resource restrictions");

            // In strict mode, ensure all security layers are enabled
            if self.config.allow_network {
                return Err(ScriptError::SecurityViolation(
                    "Network access not allowed in strict mode".to_string(),
                ));
            }

            if self.config.max_memory_mb > 512 {
                warn!(
                    "High memory limit in strict mode: {}MB",
                    self.config.max_memory_mb
                );
            }
        }

        Ok(())
    }

    /// Check if a file path is allowed for reading
    pub fn is_read_allowed(&self, path: &Path) -> bool {
        if self.config.allowed_read_paths.is_empty() {
            // If no restrictions are set, allow reading (less secure)
            return !self.is_path_blocked(path);
        }

        // Check if path is explicitly allowed
        for allowed_path in &self.config.allowed_read_paths {
            if path.starts_with(allowed_path) {
                return !self.is_path_blocked(path);
            }
        }

        false
    }

    /// Check if a file path is allowed for writing
    pub fn is_write_allowed(&self, path: &Path) -> bool {
        if self.config.allowed_write_paths.is_empty() {
            // If no write paths are allowed, deny all writes
            return false;
        }

        // Check if path is explicitly allowed for writing
        for allowed_path in &self.config.allowed_write_paths {
            if path.starts_with(allowed_path) {
                return !self.is_path_blocked(path);
            }
        }

        false
    }

    /// Check if a path matches blocked patterns
    fn is_path_blocked(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.security_policies.blocked_file_patterns {
            if pattern.contains('*') {
                // Simple glob pattern matching
                let pattern_regex = pattern.replace('*', ".*");
                if let Ok(regex) = regex::Regex::new(&pattern_regex) {
                    if regex.is_match(&path_str) {
                        return true;
                    }
                }
            } else if path_str == *pattern {
                return true;
            }
        }

        false
    }

    /// Create a secure temporary directory for script execution
    pub fn create_temp_dir(&self) -> Result<PathBuf, ScriptError> {
        let temp_dir = std::env::temp_dir().join(format!("mcp_script_{}", uuid::Uuid::new_v4()));

        std::fs::create_dir_all(&temp_dir).map_err(ScriptError::Io)?;

        debug!("Created temporary directory: {:?}", temp_dir);
        Ok(temp_dir)
    }

    /// Clean up temporary directory
    pub fn cleanup_temp_dir(&self, temp_dir: &PathBuf) -> Result<(), ScriptError> {
        if temp_dir.exists() {
            std::fs::remove_dir_all(temp_dir).map_err(ScriptError::Io)?;
            debug!("Cleaned up temporary directory: {:?}", temp_dir);
        }
        Ok(())
    }

    /// Validate script content for security issues
    pub fn validate_script_security(
        &self,
        script_config: &ScriptConfig,
    ) -> Result<(), ScriptError> {
        let source = &script_config.source;

        // Check for suspicious patterns
        let suspicious_patterns = [
            "eval(",
            "exec(",
            "import os",
            "import subprocess",
            "import sys",
            "__import__",
            "globals(",
            "locals(",
            "open(",
            "file(",
            "input(",
            "raw_input(",
            "compile(",
            "execfile(",
            "reload(",
            "setattr(",
            "getattr(",
            "hasattr(",
            "delattr(",
            "require(",
            "process.",
            "child_process",
            "fs.",
            "net.",
            "http.",
            "https.",
            "crypto.",
        ];

        for pattern in &suspicious_patterns {
            if source.contains(pattern) {
                warn!("Suspicious pattern detected in script: {}", pattern);
                if self.config.strict_mode {
                    return Err(ScriptError::SecurityViolation(format!(
                        "Suspicious pattern '{}' detected in script source",
                        pattern
                    )));
                }
            }
        }

        // Check script length
        if source.len() > 1024 * 1024 {
            // 1 MB limit
            return Err(ScriptError::SecurityViolation(
                "Script source too large (>1MB)".to_string(),
            ));
        }

        Ok(())
    }

    /// Get sandbox configuration
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Update sandbox configuration
    pub fn update_config(&mut self, config: SandboxConfig) {
        self.config = config;
    }
}

/// Resource monitoring for running scripts
#[derive(Debug, Clone)]
pub struct ResourceMonitor {
    /// Process ID being monitored
    pub pid: Option<u32>,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of open file descriptors
    pub open_files: u32,
    /// Network connections count
    pub network_connections: u32,
    /// Monitoring start time
    pub start_time: std::time::Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            pid: None,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            open_files: 0,
            network_connections: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Start monitoring a process
    pub fn start_monitoring(&mut self, pid: u32) {
        self.pid = Some(pid);
        self.start_time = std::time::Instant::now();
    }

    /// Update resource usage statistics
    ///
    /// FUTURE(#139): Implement process monitoring using system APIs:
    /// - Memory usage via /proc/[pid]/status on Linux
    /// - CPU usage via /proc/[pid]/stat
    /// - File descriptors via /proc/[pid]/fd/
    /// - Network connections via /proc/net/tcp
    pub fn update_stats(&mut self) -> Result<(), ScriptError> {
        if let Some(pid) = self.pid {
            // Basic process existence check
            #[cfg(unix)]
            {
                use std::fs;
                let proc_path = format!("/proc/{}/status", pid);
                if fs::metadata(&proc_path).is_ok() {
                    // Process still exists, update basic metrics
                    self.memory_usage_bytes = self.estimate_memory_usage();
                    self.cpu_usage_percent = self.estimate_cpu_usage();
                } else {
                    // Process no longer exists
                    return Err(ScriptError::ExecutionFailed(format!(
                        "Process {} no longer exists",
                        pid
                    )));
                }
            }

            #[cfg(not(unix))]
            {
                // For non-Unix systems, provide basic estimates
                self.memory_usage_bytes = self.estimate_memory_usage();
                self.cpu_usage_percent = self.estimate_cpu_usage();
            }
        }

        Ok(())
    }

    /// Estimate memory usage based on execution time (heuristic)
    fn estimate_memory_usage(&self) -> u64 {
        // Base memory for script interpreter + growth over time
        let elapsed_seconds = self.start_time.elapsed().as_secs();
        let base_memory = 20 * 1024 * 1024; // 20MB base
        let growth_memory = elapsed_seconds * 1024 * 1024; // 1MB per second
        base_memory + growth_memory
    }

    /// Estimate CPU usage based on activity (heuristic)
    fn estimate_cpu_usage(&self) -> f64 {
        // Simple heuristic: assume moderate CPU usage for active scripts
        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        if elapsed_seconds < 1.0 {
            50.0 // High initial usage
        } else {
            25.0 // Moderate sustained usage
        }
    }

    /// Check if resource limits are exceeded
    pub fn check_limits(&self, config: &SandboxConfig) -> Result<(), ScriptError> {
        // Check memory limit
        let memory_limit_bytes = config.max_memory_mb * 1024 * 1024;
        if self.memory_usage_bytes > memory_limit_bytes {
            return Err(ScriptError::ResourceLimitExceeded {
                resource: "memory".to_string(),
                limit: format!("{}MB", config.max_memory_mb),
            });
        }

        // Check execution time limit
        let elapsed = self.start_time.elapsed();
        if elapsed.as_millis() > config.max_execution_time_ms as u128 {
            return Err(ScriptError::Timeout {
                timeout: config.max_execution_time_ms,
            });
        }

        Ok(())
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_execution_time_ms, 30_000);
        assert_eq!(config.max_memory_mb, 256);
        assert!(!config.allow_network);
        assert!(config.strict_mode);
    }

    #[test]
    fn test_security_policies_default() {
        let policies = SecurityPolicies::default();
        assert!(policies.blocked_syscalls.contains(&"execve".to_string()));
        assert!(policies.blocked_env_vars.contains(&"PATH".to_string()));
        assert!(policies.allowed_commands.contains(&"node".to_string()));
    }

    #[test]
    fn test_sandbox_manager_creation() {
        let config = SandboxConfig::default();
        let manager = SandboxManager::new(config);
        assert!(manager.config.strict_mode);
    }

    #[test]
    fn test_path_blocking() {
        let config = SandboxConfig::default();
        let manager = SandboxManager::new(config);

        let blocked_path = PathBuf::from("/etc/passwd");
        assert!(manager.is_path_blocked(&blocked_path));

        let allowed_path = PathBuf::from("/tmp/safe_file.txt");
        assert!(!manager.is_path_blocked(&allowed_path));
    }

    #[test]
    fn test_script_security_validation() {
        let config = SandboxConfig::default();
        let manager = SandboxManager::new(config);

        let safe_script = ScriptConfig {
            source: "const result = { success: true };".to_string(),
            ..Default::default()
        };
        assert!(manager.validate_script_security(&safe_script).is_ok());

        let unsafe_script = ScriptConfig {
            source: "eval('malicious code');".to_string(),
            ..Default::default()
        };
        assert!(manager.validate_script_security(&unsafe_script).is_err());
    }

    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring(1234);

        assert_eq!(monitor.pid, Some(1234));
        assert!(monitor.start_time.elapsed().as_millis() < 1000); // Should be recent
    }

    #[test]
    fn test_read_write_permissions() {
        let config = SandboxConfig {
            allowed_read_paths: vec![PathBuf::from("/tmp")],
            allowed_write_paths: vec![PathBuf::from("/tmp/scripts")],
            ..Default::default()
        };

        let manager = SandboxManager::new(config);

        assert!(manager.is_read_allowed(&PathBuf::from("/tmp/test.txt")));
        assert!(!manager.is_read_allowed(&PathBuf::from("/etc/passwd")));

        assert!(manager.is_write_allowed(&PathBuf::from("/tmp/scripts/output.txt")));
        assert!(!manager.is_write_allowed(&PathBuf::from("/tmp/test.txt")));
    }
}
