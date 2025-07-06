# CLI Integration System Design Document

## Problem Statement

The Mandrel MCP Test Harness has a comprehensive reporting system that generates JSON, JUnit XML, HTML, and Markdown reports with enterprise-grade features. However, users currently cannot access these reporting capabilities through a convenient command-line interface, limiting adoption and CI/CD integration.

**Key Issues:**
- No CLI access to reporting functionality  
- Manual report generation requires Rust code integration
- Difficult CI/CD pipeline integration
- No file output management or organization
- Limited user configurability for templates and branding

## Proposed Solution

Create a comprehensive CLI interface that exposes all reporting functionality through intuitive commands, with file management, template selection, and CI/CD integration capabilities.

### High-Level Architecture

```rust
// CLI Command Structure
mandrel-mcp-th report [OPTIONS]

// Core CLI Components
pub struct ReportCli {
    config: CliConfig,
    generator: ReportGenerator,
    file_manager: FileManager,
}

// Configuration Management
pub struct CliConfig {
    pub output_directory: PathBuf,
    pub formats: Vec<OutputFormat>,
    pub template: Option<BuiltInTemplate>,
    pub branding_config: Option<PathBuf>,
    pub custom_fields: HashMap<String, String>,
}

// File Output Management
pub struct FileManager {
    pub base_directory: PathBuf,
    pub timestamp_format: TimestampFormat,
    pub organization_strategy: OrganizationStrategy,
}
```

### Command Design

**Basic Report Generation:**
```bash
# Generate JSON report
mandrel-mcp-th report --format json --output ./reports/

# Generate multiple formats
mandrel-mcp-th report --formats json,junit,html,markdown --output ./reports/

# Use specific template
mandrel-mcp-th report --format html --template professional --output ./reports/
```

**Advanced Configuration:**
```bash
# Custom branding
mandrel-mcp-th report --format html --branding-config ./branding.json --output ./reports/

# Custom fields for metadata
mandrel-mcp-th report --format markdown --custom-field team="QA Team" --custom-field build="v1.2.3"

# Directory organization
mandrel-mcp-th report --formats json,html --output ./reports/ --organize-by date --timestamp iso
```

**CI/CD Integration:**
```bash
# Exit codes for CI/CD
mandrel-mcp-th report --format junit --output ./test-results/ --fail-on-errors

# Watch mode for continuous integration
mandrel-mcp-th report --watch --format html --output ./live-reports/ --auto-refresh
```

## API Design

### Core CLI Interface

```rust
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "mandrel-mcp-th")]
#[command(about = "Mandrel MCP Test Harness - Professional testing and reporting")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate test reports from execution results
    Report(ReportArgs),
    
    /// Run tests and generate reports
    Run(RunArgs),
    
    /// Validate configuration files
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct ReportArgs {
    /// Input file containing test results (JSON format)
    #[arg(short = 'i', long)]
    pub input: PathBuf,
    
    /// Output directory for generated reports
    #[arg(short = 'o', long, default_value = "./reports")]
    pub output: PathBuf,
    
    /// Report formats to generate
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,
    
    /// Built-in template to use
    #[arg(short = 't', long)]
    pub template: Option<TemplateName>,
    
    /// Custom branding configuration file
    #[arg(long)]
    pub branding_config: Option<PathBuf>,
    
    /// Custom fields for metadata (key=value format)
    #[arg(long = "custom-field", value_parser = parse_key_val)]
    pub custom_fields: Vec<(String, String)>,
    
    /// Organization strategy for output files
    #[arg(long, default_value = "flat")]
    pub organize_by: OrganizationStrategy,
    
    /// Timestamp format for file naming
    #[arg(long, default_value = "iso")]
    pub timestamp: TimestampFormat,
    
    /// Fail with non-zero exit code if tests failed
    #[arg(long)]
    pub fail_on_errors: bool,
    
    /// Include performance metrics in reports
    #[arg(long, default_value = "true")]
    pub include_performance: bool,
    
    /// Include validation details in reports
    #[arg(long, default_value = "true")]
    pub include_validation: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ReportFormat {
    Json,
    Junit,
    Html,
    Markdown,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TemplateName {
    Professional,
    Executive,
    Technical,
    Minimal,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OrganizationStrategy {
    Flat,        // All files in output directory
    ByDate,      // Organized by date: 2025/01/15/reports
    ByFormat,    // Organized by format: json/, html/, etc.
    ByTemplate,  // Organized by template: professional/, technical/
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TimestampFormat {
    Iso,         // 2025-01-15T10-30-45Z
    Unix,        // 1736935845
    Human,       // 2025-01-15_10-30-45
    None,        // No timestamp
}
```

### File Management System

```rust
pub struct FileManager {
    base_directory: PathBuf,
    organization: OrganizationStrategy,
    timestamp: TimestampFormat,
}

impl FileManager {
    pub fn new(
        base_directory: PathBuf,
        organization: OrganizationStrategy,
        timestamp: TimestampFormat,
    ) -> Result<Self> {
        // Ensure base directory exists
        std::fs::create_dir_all(&base_directory)?;
        
        Ok(FileManager {
            base_directory,
            organization,
            timestamp,
        })
    }
    
    pub fn generate_output_path(
        &self,
        format: &ReportFormat,
        template: Option<&TemplateName>,
        suite_name: &str,
    ) -> Result<PathBuf> {
        let mut path = self.base_directory.clone();
        
        // Apply organization strategy
        match self.organization {
            OrganizationStrategy::Flat => {},
            OrganizationStrategy::ByDate => {
                let now = chrono::Utc::now();
                path.push(format!("{}", now.format("%Y")));
                path.push(format!("{}", now.format("%m")));
                path.push(format!("{}", now.format("%d")));
            },
            OrganizationStrategy::ByFormat => {
                path.push(format.to_directory_name());
            },
            OrganizationStrategy::ByTemplate => {
                if let Some(template) = template {
                    path.push(template.to_directory_name());
                }
            },
        }
        
        // Ensure directory exists
        std::fs::create_dir_all(&path)?;
        
        // Generate filename
        let timestamp = self.generate_timestamp();
        let extension = format.file_extension();
        let filename = match timestamp {
            Some(ts) => format!("{}_{}.{}", suite_name, ts, extension),
            None => format!("{}.{}", suite_name, extension),
        };
        
        path.push(filename);
        Ok(path)
    }
    
    pub fn write_report(&self, path: &PathBuf, content: &str) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }
    
    fn generate_timestamp(&self) -> Option<String> {
        let now = chrono::Utc::now();
        match self.timestamp {
            TimestampFormat::Iso => Some(now.format("%Y-%m-%dT%H-%M-%SZ").to_string()),
            TimestampFormat::Unix => Some(now.timestamp().to_string()),
            TimestampFormat::Human => Some(now.format("%Y-%m-%d_%H-%M-%S").to_string()),
            TimestampFormat::None => None,
        }
    }
}
```

### Branding Configuration System

```rust
#[derive(Deserialize, Debug)]
pub struct BrandingConfig {
    pub company_name: Option<String>,
    pub logo_path: Option<PathBuf>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub css_overrides: Option<String>,
    pub custom_css_file: Option<PathBuf>,
}

impl BrandingConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: BrandingConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn to_branding_info(&self) -> BrandingInfo {
        BrandingInfo {
            company_name: self.company_name.clone(),
            logo_path: self.logo_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            primary_color: self.primary_color.clone(),
            secondary_color: self.secondary_color.clone(),
            css_overrides: self.load_css_overrides(),
        }
    }
    
    fn load_css_overrides(&self) -> Option<String> {
        // Priority: direct css_overrides, then css_file
        if let Some(css) = &self.css_overrides {
            return Some(css.clone());
        }
        
        if let Some(css_file) = &self.custom_css_file {
            if let Ok(content) = std::fs::read_to_string(css_file) {
                return Some(content);
            }
        }
        
        None
    }
}
```

## Implementation Plan

### Phase 5A: CLI Foundation (RED → GREEN → REFACTOR)

**RED Phase:**
1. Write failing tests for CLI argument parsing
2. Write failing tests for file output management
3. Write failing tests for basic report generation
4. Write failing tests for error handling and exit codes

**GREEN Phase:**
1. Implement clap-based argument parsing
2. Implement FileManager with path generation
3. Implement basic report generation integration
4. Implement error handling and proper exit codes

**REFACTOR Phase:**
1. Optimize CLI performance and memory usage
2. Improve error messages and user experience
3. Add comprehensive logging and debug output
4. Optimize file I/O operations

### Phase 5B: Advanced Configuration (RED → GREEN → REFACTOR)

**RED Phase:**
1. Write failing tests for branding configuration loading
2. Write failing tests for custom fields parsing
3. Write failing tests for multiple format generation
4. Write failing tests for organization strategies

**GREEN Phase:**
1. Implement branding configuration system
2. Implement custom fields parsing and validation
3. Implement multiple format generation
4. Implement all organization strategies

**REFACTOR Phase:**
1. Optimize configuration validation
2. Improve configuration error messages
3. Add configuration file examples and templates
4. Optimize multi-format generation performance

### Phase 5B: Advanced Configuration - Enhanced Design

#### Watch Mode System
```rust
pub struct WatchManager {
    watcher: RecommendedWatcher,
    config: WatchConfig,
    report_generator: Arc<ReportGenerator>,
}

#[derive(Clone, Debug)]
pub struct WatchConfig {
    pub input_patterns: Vec<String>,
    pub output_directory: PathBuf,
    pub debounce_ms: u64,
    pub formats: Vec<ReportFormat>,
    pub auto_open: bool,
}

impl WatchManager {
    pub async fn start_watching(&mut self) -> Result<()> {
        // Monitor file system events
        // Debounce rapid changes
        // Auto-regenerate reports
        // Optional: Auto-open in browser
    }
}
```

**Command Usage:**
```bash
# Watch for changes and auto-regenerate
mandrel-mcp-th report --watch --input-pattern "test-results/*.json" --output ./live-reports/

# Watch with debouncing
mandrel-mcp-th report --watch --debounce 2000 --auto-open --format html
```

#### Configuration Profile System
```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigProfile {
    pub name: String,
    pub description: Option<String>,
    pub report_config: ReportConfig,
    pub file_management: FileManagerConfig,
    pub branding: Option<BrandingConfig>,
    pub environment_vars: HashMap<String, String>,
}

pub struct ProfileManager {
    profiles_directory: PathBuf,
    active_profile: Option<String>,
}

impl ProfileManager {
    pub fn save_profile(&self, profile: &ConfigProfile) -> Result<()> {
        // Save profile to ~/.config/mandrel-mcp-th/profiles/
    }
    
    pub fn load_profile(&self, name: &str) -> Result<ConfigProfile> {
        // Load profile from disk
    }
    
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        // List available profiles
    }
}
```

**Command Usage:**
```bash
# Save current configuration as profile
mandrel-mcp-th profile save --name "ci-reports" --description "Standard CI/CD reporting"

# Use saved profile
mandrel-mcp-th report --profile "ci-reports" --input test-results.json

# List available profiles
mandrel-mcp-th profile list

# Export/import profiles for team sharing
mandrel-mcp-th profile export --name "ci-reports" --output ./team-profiles/
mandrel-mcp-th profile import --file ./team-profiles/ci-reports.json
```

#### Enhanced Validation System
```rust
pub struct ValidationEngine {
    schema_validator: JsonSchemaValidator,
    template_validator: TemplateValidator,
    config_validator: ConfigValidator,
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

impl ValidationEngine {
    pub fn validate_input_file(&self, path: &PathBuf) -> Result<ValidationResult> {
        // Validate JSON schema
        // Check for required fields
        // Validate data types and ranges
        // Provide specific error locations
    }
    
    pub fn validate_template(&self, template: &TemplateSource) -> Result<ValidationResult> {
        // Validate template syntax
        // Check for required variables
        // Validate CSS and HTML structure
        // Check for security issues
    }
    
    pub fn validate_configuration(&self, config: &ReportConfig) -> Result<ValidationResult> {
        // Validate all configuration fields
        // Check file paths exist
        // Validate color formats
        // Check for conflicts
    }
}
```

**Command Usage:**
```bash
# Comprehensive validation
mandrel-mcp-th validate --input test-results.json --config ./config.json --template custom.html

# Schema validation only
mandrel-mcp-th validate --schema-only --input test-results.json

# Template validation
mandrel-mcp-th validate --template-only --template ./templates/custom.html
```

#### Template Management System
```rust
pub struct TemplateManager {
    templates_directory: PathBuf,
    cache: Arc<RwLock<HashMap<String, Template>>>,
}

pub struct Template {
    pub name: String,
    pub description: String,
    pub version: String,
    pub content: String,
    pub metadata: TemplateMetadata,
}

impl TemplateManager {
    pub fn discover_templates(&self) -> Result<Vec<Template>> {
        // Scan for custom templates
        // Load built-in templates
        // Validate all templates
    }
    
    pub fn install_template(&self, source: &str) -> Result<()> {
        // Install from URL, file, or registry
    }
    
    pub fn preview_template(&self, name: &str, sample_data: &TestReport) -> Result<String> {
        // Generate preview with sample data
    }
}
```

**Command Usage:**
```bash
# List available templates
mandrel-mcp-th template list

# Install template from URL
mandrel-mcp-th template install --url https://example.com/template.html --name "custom-dark"

# Preview template with sample data
mandrel-mcp-th template preview --name "professional" --sample-data ./test-results.json

# Validate template
mandrel-mcp-th template validate --template ./custom-template.html
```

#### Environment Integration
```rust
pub struct EnvironmentDetector {
    pub ci_system: Option<CiSystem>,
    pub environment_type: EnvironmentType,
    pub available_vars: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum CiSystem {
    GitHubActions,
    Jenkins,
    GitLabCI,
    CircleCI,
    Travis,
    BuildKite,
    TeamCity,
}

impl EnvironmentDetector {
    pub fn detect_ci_system() -> Option<CiSystem> {
        // Detect CI system from environment variables
    }
    
    pub fn get_ci_specific_config(&self) -> Result<CiConfig> {
        // Return CI-specific optimizations
    }
    
    pub fn setup_output_paths(&self) -> Result<PathBuf> {
        // Use CI-standard output directories
    }
}
```

**Automatic CI Detection:**
```bash
# Automatically optimizes for detected CI system
mandrel-mcp-th report --auto-ci --input test-results.json

# Override CI detection
mandrel-mcp-th report --ci-system github-actions --input test-results.json
```

#### Performance Optimization
```rust
pub struct PerformanceOptimizer {
    pub parallel_generation: bool,
    pub memory_limit: Option<usize>,
    pub streaming_output: bool,
}

impl PerformanceOptimizer {
    pub async fn generate_reports_parallel(&self, configs: Vec<ReportConfig>) -> Result<Vec<PathBuf>> {
        // Generate multiple formats in parallel
        // Stream large outputs to disk
        // Monitor memory usage
    }
    
    pub fn optimize_for_size(&self, suite_size: usize) -> PerformanceConfig {
        // Adjust settings based on test suite size
    }
}
```

**Performance Commands:**
```bash
# Parallel generation
mandrel-mcp-th report --parallel --formats json,html,markdown --input large-results.json

# Memory-conscious mode  
mandrel-mcp-th report --memory-limit 256MB --streaming --input huge-results.json

# Performance analysis
mandrel-mcp-th report --profile-performance --input test-results.json
```

### Enhanced CLI Arguments for Phase 5B

```rust
#[derive(Args)]
pub struct ReportArgs {
    // ... existing fields from Phase 5A ...
    
    // Watch Mode
    #[arg(long)]
    pub watch: bool,
    
    #[arg(long, default_value = "*.json")]
    pub input_pattern: String,
    
    #[arg(long, default_value = "500")]
    pub debounce: u64,
    
    #[arg(long)]
    pub auto_open: bool,
    
    // Configuration Profiles
    #[arg(long)]
    pub profile: Option<String>,
    
    #[arg(long)]
    pub save_profile: Option<String>,
    
    // Enhanced Validation
    #[arg(long)]
    pub validate_only: bool,
    
    #[arg(long)]
    pub strict_validation: bool,
    
    // CI/CD Integration
    #[arg(long)]
    pub auto_ci: bool,
    
    #[arg(long)]
    pub ci_system: Option<CiSystem>,
    
    // Performance
    #[arg(long)]
    pub parallel: bool,
    
    #[arg(long)]
    pub memory_limit: Option<String>,
    
    #[arg(long)]
    pub streaming: bool,
    
    #[arg(long)]
    pub profile_performance: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Report(ReportArgs),
    Run(RunArgs),
    Validate(ValidateArgs),
    
    // New commands for Phase 5B
    Profile(ProfileArgs),
    Template(TemplateArgs),
    Watch(WatchArgs),
}

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub action: ProfileAction,
}

#[derive(Subcommand)]
pub enum ProfileAction {
    List,
    Save { name: String, description: Option<String> },
    Load { name: String },
    Delete { name: String },
    Export { name: String, output: PathBuf },
    Import { file: PathBuf },
}

#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub action: TemplateAction,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    List,
    Install { url: String, name: String },
    Preview { name: String, sample_data: PathBuf },
    Validate { template: PathBuf },
    Remove { name: String },
}
```

### Implementation Testing Strategy

#### Unit Tests for Phase 5B
```rust
#[cfg(test)]
mod phase5b_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_watch_mode_file_detection() {
        // Test file system monitoring
        // Test debouncing logic
        // Test auto-regeneration
    }
    
    #[test]
    fn test_configuration_profile_management() {
        // Test profile saving/loading
        // Test profile validation
        // Test profile sharing
    }
    
    #[test]
    fn test_enhanced_validation_engine() {
        // Test schema validation
        // Test template validation  
        // Test configuration validation
    }
    
    #[test]
    fn test_ci_system_detection() {
        // Test GitHub Actions detection
        // Test Jenkins detection
        // Test environment variable mapping
    }
    
    #[test]
    fn test_performance_optimization() {
        // Test parallel generation
        // Test memory limits
        // Test streaming output
    }
}
```

### Success Criteria for Phase 5B

1. **Watch Mode**: Automatically regenerate reports when input files change
2. **Configuration Profiles**: Save, load, and share complex configurations
3. **Enhanced Validation**: Comprehensive validation with actionable error messages
4. **Template Management**: Install, preview, and manage custom templates
5. **CI/CD Integration**: Auto-detect CI systems and optimize output
6. **Performance**: Handle large test suites (1000+ tests) efficiently

This advanced configuration system will transform our CLI from a basic report generator into a comprehensive, enterprise-ready testing and reporting platform that integrates seamlessly with any development workflow.

### Phase 5C: CI/CD Integration (RED → GREEN → REFACTOR)

**RED Phase:**
1. Write failing tests for exit code behavior
2. Write failing tests for watch mode functionality
3. Write failing tests for CI/CD output formatting
4. Write failing tests for environment variable support

**GREEN Phase:**
1. Implement proper exit codes for CI/CD
2. Implement watch mode with file monitoring
3. Implement CI/CD-friendly output formatting
4. Implement environment variable configuration

**REFACTOR Phase:**
1. Optimize watch mode performance
2. Improve CI/CD integration documentation
3. Add comprehensive examples for popular CI systems
4. Optimize memory usage for long-running processes

## Testing Strategy

### Unit Tests
- CLI argument parsing with valid/invalid inputs
- File path generation for all organization strategies
- Branding configuration loading and validation
- Custom fields parsing and error handling
- Multi-format report generation
- Exit code behavior under different conditions

### Integration Tests
- End-to-end CLI execution with real test data
- File system integration with various output formats
- Configuration file loading and validation
- Multi-format output verification
- Watch mode functionality with file system events

### Performance Tests
- Large report generation performance
- Memory usage during multi-format generation
- File I/O performance with various organization strategies
- Watch mode resource utilization

### CI/CD Tests
- Exit code verification in CI environments
- JUnit XML output validation for CI systems
- Watch mode stability under continuous load
- Environment variable configuration in CI

## Success Criteria

### Functional Requirements
1. **CLI generates all report formats** (JSON, JUnit XML, HTML, Markdown)
2. **File organization works** for all strategies (flat, by-date, by-format, by-template)  
3. **Custom branding loads correctly** from JSON configuration files
4. **Multi-format generation** produces consistent, valid output
5. **Exit codes work properly** for CI/CD integration (0 for success, 1 for test failures)

### Performance Requirements
1. **Report generation < 5 seconds** for typical test suites (100 tests)
2. **Memory usage < 100MB** for large reports (1000+ tests)
3. **File I/O < 1 second** for multi-format output
4. **Watch mode < 50MB** memory footprint during monitoring

### Usability Requirements
1. **Intuitive command structure** following Unix conventions
2. **Clear error messages** with actionable suggestions
3. **Comprehensive help documentation** with examples
4. **Configuration validation** with specific error locations

### Integration Requirements
1. **Works with GitHub Actions, Jenkins, GitLab CI** (JUnit XML validation)
2. **Compatible with Docker environments** (file permissions, paths)
3. **Environment variable support** for secure CI/CD configuration
4. **Cross-platform compatibility** (Linux, macOS, Windows)

## Alternatives Considered

### Alternative 1: Library-Only Approach
**Description:** Keep reporting as library-only, require users to write Rust code
**Pros:** Simpler implementation, more flexible for advanced users
**Cons:** Higher barrier to entry, difficult CI/CD integration
**Decision:** Rejected - CLI is essential for adoption

### Alternative 2: Configuration File-Only Interface
**Description:** Use TOML/YAML configuration files instead of CLI arguments
**Pros:** Complex configurations, version controllable
**Cons:** Less convenient for simple operations, learning curve
**Decision:** Hybrid approach - CLI for common operations, config files for complex scenarios

### Alternative 3: Web Interface
**Description:** Provide web-based interface for report generation
**Pros:** User-friendly, visual configuration
**Cons:** More complex, requires server infrastructure, not CI/CD friendly
**Decision:** Future consideration - CLI is higher priority

### Alternative 4: Plugin Architecture
**Description:** Make CLI extensible with plugins for custom formats
**Pros:** Highly extensible, community contributions
**Cons:** Increased complexity, security concerns
**Decision:** Future consideration - start with built-in formats

## Dependencies

### New Dependencies
```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "env"] }
notify = "6.1"  # For watch mode file monitoring
dirs = "5.0"    # For standard directory detection
```

### Development Dependencies
```toml
[dev-dependencies]
tempfile = "3.8"     # For temporary directory testing
assert_cmd = "2.0"   # For CLI testing
predicates = "3.0"   # For assertion predicates
```

## File Structure

```
crates/mandrel-mcp-th/
├── src/
│   ├── cli/
│   │   ├── mod.rs              # CLI module exports
│   │   ├── args.rs             # Argument parsing
│   │   ├── commands.rs         # Command implementations  
│   │   ├── file_manager.rs     # File output management
│   │   ├── branding.rs         # Branding configuration
│   │   └── watch.rs            # Watch mode implementation
│   ├── main.rs                 # CLI entry point
│   └── lib.rs                  # Library exports
├── examples/
│   ├── branding.json           # Example branding config
│   ├── ci-integration.sh       # CI/CD integration examples
│   └── watch-mode.sh           # Watch mode examples
└── tests/
    ├── cli/
    │   ├── integration_tests.rs # End-to-end CLI tests
    │   ├── file_management.rs   # File output tests
    │   └── configuration.rs     # Configuration tests
    └── fixtures/
        ├── test-results.json    # Sample test data
        └── branding-configs/    # Sample configurations
```

This comprehensive CLI integration will make the Mandrel MCP Test Harness accessible to all users and enable seamless CI/CD integration while maintaining the enterprise-grade quality of our reporting system. 