# Issue #243: CLI Advanced Configuration Features Integration

## Problem Statement

The CLI is **missing critical advanced configuration features** despite having comprehensive enterprise-grade implementations in the backend. This represents the **largest CLI integration gap** with 1000+ lines of enterprise features completely inaccessible to users.

## Proposed Solution

Wire up existing `ProfileManager`, `WatchManager`, `EnvironmentDetector`, and related components to provide enterprise-grade configuration management, file watching, and CI/CD integration through the CLI.

## Current State Analysis

### ✅ **What's Already Implemented:**

#### **1. Profile Management System (500+ lines)**
- `ProfileManager` - Complete profile save/load/delete/export/import
- `ConfigProfile` - Comprehensive configuration profiles with environment vars
- YAML-based profile storage with team sharing capabilities
- Profile export/import for enterprise workflows

#### **2. Watch Mode System (200+ lines)**
- `WatchManager` - Complete file watching with debouncing
- `WatchConfig` - Auto-regeneration on file changes
- Multiple format generation, auto-open reports
- Configurable debouncing and pattern matching

#### **3. CI/CD Integration System (100+ lines)**
- `EnvironmentDetector` - Auto-detects GitHub Actions, Jenkins, GitLab CI, etc.
- `CiConfig` - CI-optimized configurations
- Automatic CI-specific paths, formats, and environment metadata

#### **4. Advanced Configuration Infrastructure**
- `ValidationEngine` - Configuration validation
- `FileManagerConfig` - File organization strategies
- `BrandingConfig` - Custom branding management
- Environment variable integration

### ❌ **What's Missing:**

#### **1. Profile Management Commands (ZERO CLI Exposure)**
- No `moth profile` commands
- Cannot save/load/share configurations
- No team collaboration features

#### **2. Watch Mode Command (ZERO CLI Exposure)**
- No `moth watch` command
- No auto-regeneration capabilities
- No modern development workflow support

#### **3. CI/CD Integration (ZERO CLI Exposure)**
- No CI mode detection
- No environment-optimized configurations
- No CI-specific output formats

#### **4. Limited RunArgs & ValidateArgs**
- Missing 90% of available configuration options
- No profile usage
- No environment integration
- No timeout/retry controls

#### **5. Global Configuration Management**
- No `moth config` commands
- No global settings management
- No project initialization

## API Design

### **New CLI Commands Structure**
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate test reports from execution results
    Report(ReportArgs),
    /// Run tests and generate reports
    Run(RunArgs),
    /// Validate configuration files
    Validate(ValidateArgs),
    
    // NEW: Profile management commands
    /// Manage configuration profiles
    Profile(ProfileCommand),
    /// Watch files and auto-regenerate reports
    Watch(WatchArgs),
    /// Global configuration management
    Config(ConfigCommand),
    /// Initialize project with moth configuration
    Init(InitArgs),
}
```

### **Profile Management Commands**
```rust
#[derive(Subcommand, Debug)]
pub enum ProfileCommand {
    /// List available profiles
    List,
    /// Save current configuration as a profile
    Save(ProfileSaveArgs),
    /// Load a configuration profile
    Load(ProfileLoadArgs),
    /// Export a profile to file
    Export(ProfileExportArgs),
    /// Import a profile from file
    Import(ProfileImportArgs),
    /// Delete a profile
    Delete(ProfileDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ProfileSaveArgs {
    /// Profile name
    #[arg()]
    pub name: String,
    /// Profile description
    #[arg(short = 'd', long)]
    pub description: Option<String>,
    /// Base configuration from current run/report args
    #[arg(long)]
    pub from_current: bool,
}

#[derive(Args, Debug)]
pub struct ProfileLoadArgs {
    /// Profile name to load
    #[arg()]
    pub name: String,
}
```

### **Watch Mode Command**
```rust
#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Input patterns to watch (files or globs)
    #[arg()]
    pub patterns: Vec<String>,
    
    /// Output directory for generated reports
    #[arg(short = 'o', long, default_value = "./reports")]
    pub output: PathBuf,
    
    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce: u64,
    
    /// Report formats to generate on change
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,
    
    /// Auto-open generated reports
    #[arg(long)]
    pub auto_open: bool,
    
    /// Configuration profile to use
    #[arg(short = 'p', long)]
    pub profile: Option<String>,
}
```

### **Enhanced RunArgs with Advanced Options**
```rust
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Test configuration file
    #[arg()]
    pub config: PathBuf,

    /// Output directory for generated reports
    #[arg(short = 'o', long, default_value = "./reports")]
    pub output: Option<PathBuf>,

    /// Run tests in parallel
    #[arg(long)]
    pub parallel: bool,

    /// Stop execution on first test failure
    #[arg(long)]
    pub fail_fast: bool,

    // NEW: Advanced configuration options
    /// Configuration profile to use
    #[arg(short = 'p', long)]
    pub profile: Option<String>,
    
    /// Enable CI mode with auto-detection
    #[arg(long)]
    pub ci_mode: bool,
    
    /// Force specific CI system detection
    #[arg(long)]
    pub ci_system: Option<CiSystem>,
    
    /// Watch mode - regenerate on file changes
    #[arg(long)]
    pub watch: bool,
    
    /// Timeout for test execution
    #[arg(long)]
    pub timeout: Option<String>,
    
    /// Number of retry attempts for failed tests
    #[arg(long, default_value = "3")]
    pub retries: u32,
    
    /// Environment variables (key=value format)
    #[arg(long = "env", value_parser = parse_key_val)]
    pub environment: Vec<(String, String)>,

    // Existing advanced reporting options (from Issue #241)
    /// Report formats to generate
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,

    /// Built-in template to use for HTML reports
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
}
```

### **Global Configuration Commands**
```rust
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Initialize global moth configuration
    Init,
    /// Get configuration value
    Get(ConfigGetArgs),
    /// Set configuration value
    Set(ConfigSetArgs),
    /// List all configuration values
    List,
    /// Reset configuration to defaults
    Reset,
    /// Detect CI environment
    DetectCi,
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// Configuration key
    #[arg()]
    pub key: String,
    /// Configuration value
    #[arg()]
    pub value: String,
}
```

## Implementation Plan

### **Phase 1: Core CLI Integration (Days 1-2)**

#### **1.1: Profile Management Commands**
```rust
impl CliApp {
    async fn handle_profile_command(&self, cmd: &ProfileCommand) -> Result<i32> {
        let profile_manager = ProfileManager::new(get_profiles_directory()?)?;
        
        match cmd {
            ProfileCommand::List => {
                let profiles = profile_manager.list_profiles()?;
                for profile in profiles {
                    println!("{}", profile);
                }
                Ok(0)
            },
            ProfileCommand::Save(args) => {
                let profile = self.create_profile_from_current_config(args)?;
                profile_manager.save_profile(&profile)?;
                println!("Profile '{}' saved successfully", args.name);
                Ok(0)
            },
            // ... other profile commands
        }
    }
}
```

#### **1.2: Watch Mode Command**
```rust
impl CliApp {
    async fn handle_watch_command(&self, args: &WatchArgs) -> Result<i32> {
        let watch_config = WatchConfig {
            input_patterns: args.patterns.clone(),
            output_directory: args.output.clone(),
            debounce_ms: args.debounce,
            formats: args.formats.clone(),
            auto_open: args.auto_open,
        };
        
        let mut watch_manager = WatchManager::new(watch_config)?;
        
        println!("Starting watch mode for patterns: {:?}", args.patterns);
        println!("Press Ctrl+C to stop...");
        
        watch_manager.start_watching().await?;
        Ok(0)
    }
}
```

#### **1.3: Enhanced RunArgs Processing**
```rust
impl CliApp {
    async fn handle_run_command(&self, args: &RunArgs) -> Result<i32> {
        // 1. Load profile if specified
        let mut effective_config = if let Some(profile_name) = &args.profile {
            self.load_profile_config(profile_name)?
        } else {
            Default::default()
        };
        
        // 2. Apply CLI argument overrides
        self.apply_cli_overrides(&mut effective_config, args)?;
        
        // 3. Apply CI mode if enabled
        if args.ci_mode {
            self.apply_ci_optimizations(&mut effective_config, args.ci_system.as_ref())?;
        }
        
        // 4. Set up environment variables
        for (key, value) in &args.environment {
            std::env::set_var(key, value);
        }
        
        // 5. Execute tests with enhanced configuration
        let test_results = if args.watch {
            self.execute_tests_with_watch(&args.config, &effective_config).await?
        } else {
            self.execute_tests_with_config(&args.config, &effective_config).await?
        };
        
        // 6. Generate reports using enhanced configuration
        self.generate_reports_with_config(&test_results, &effective_config).await?;
        
        Ok(if test_results.failed > 0 { 1 } else { 0 })
    }
}
```

### **Phase 2: Enhanced Arguments (Days 2-3)**

#### **2.1: CI Integration**
```rust
fn apply_ci_optimizations(
    &self, 
    config: &mut EffectiveConfig, 
    ci_system: Option<&CiSystem>
) -> Result<()> {
    let detector = EnvironmentDetector::new();
    
    let ci_config = if let Some(system) = ci_system {
        detector.get_ci_specific_config_for_system(system)?
    } else {
        detector.get_ci_specific_config()?
    };
    
    // Apply CI-optimized settings
    config.output_directory = ci_config.output_directory;
    config.formats = ci_config.formats;
    config.fail_on_errors = ci_config.fail_on_errors;
    config.environment_metadata.extend(ci_config.environment_metadata);
    
    Ok(())
}
```

#### **2.2: Profile Configuration Loading**
```rust
fn load_profile_config(&self, profile_name: &str) -> Result<EffectiveConfig> {
    let profile_manager = ProfileManager::new(get_profiles_directory()?)?;
    let profile = profile_manager.load_profile(profile_name)?;
    
    Ok(EffectiveConfig {
        report_config: profile.report_config,
        file_management: profile.file_management,
        branding: profile.branding,
        environment_vars: profile.environment_vars,
        ..Default::default()
    })
}
```

### **Phase 3: Configuration Management (Days 3-4)**

#### **3.1: Global Configuration**
```rust
impl CliApp {
    async fn handle_config_command(&self, cmd: &ConfigCommand) -> Result<i32> {
        let config_manager = GlobalConfigManager::new()?;
        
        match cmd {
            ConfigCommand::Set(args) => {
                config_manager.set(&args.key, &args.value)?;
                println!("Configuration set: {} = {}", args.key, args.value);
                Ok(0)
            },
            ConfigCommand::Get(args) => {
                if let Some(value) = config_manager.get(&args.key)? {
                    println!("{}", value);
                } else {
                    println!("Configuration key '{}' not found", args.key);
                }
                Ok(0)
            },
            ConfigCommand::DetectCi => {
                let detector = EnvironmentDetector::new();
                if let Some(ci_system) = detector.detect_ci_system() {
                    println!("Detected CI system: {:?}", ci_system);
                    let ci_config = detector.get_ci_specific_config()?;
                    println!("Recommended settings:");
                    println!("  Output directory: {}", ci_config.output_directory.display());
                    println!("  Formats: {:?}", ci_config.formats);
                    println!("  Fail on errors: {}", ci_config.fail_on_errors);
                } else {
                    println!("No CI system detected");
                }
                Ok(0)
            },
            // ... other config commands
        }
    }
}
```

#### **3.2: Project Initialization**
```rust
impl CliApp {
    async fn handle_init_command(&self, args: &InitArgs) -> Result<i32> {
        let project_dir = std::env::current_dir()?;
        let config_file = project_dir.join(".moth.yaml");
        
        if config_file.exists() && !args.force {
            return Err("Configuration file already exists. Use --force to overwrite.".into());
        }
        
        // Create default project configuration
        let default_config = ProjectConfig::default();
        let yaml_content = serde_yml::to_string(&default_config)?;
        std::fs::write(&config_file, yaml_content)?;
        
        // Create reports directory
        std::fs::create_dir_all(project_dir.join("reports"))?;
        
        println!("Initialized moth project in {}", project_dir.display());
        println!("Configuration file: {}", config_file.display());
        
        Ok(0)
    }
}
```

## Success Criteria

### **Functional Requirements**

#### **Profile Management**
- `moth profile save "ci-profile" --description "CI/CD optimized"` creates profile
- `moth profile list` shows all available profiles
- `moth profile load "ci-profile"` applies profile settings
- `moth profile export "ci-profile" --output team-config.yaml` exports for sharing
- `moth profile import team-config.yaml` imports shared configuration

#### **Watch Mode**
- `moth watch config.yaml --formats html,junit --auto-open` starts file watching
- `moth watch *.json --output ./reports --debounce 1000` watches multiple patterns
- File changes trigger automatic report regeneration
- Debouncing prevents excessive regeneration

#### **CI Integration**
- `moth run config.yaml --ci-mode` automatically optimizes for detected CI
- `moth run config.yaml --ci-system github-actions` forces specific CI optimizations
- `moth config detect-ci` shows detected CI environment and recommendations

#### **Enhanced Run Command**
- `moth run config.yaml --profile "ci-profile" --watch` combines features
- `moth run config.yaml --env BUILD_ID=123 --timeout 5m` adds environment and controls
- `moth run config.yaml --formats html,junit --template professional` uses advanced reporting

#### **Global Configuration**
- `moth config set default-template professional` sets global defaults
- `moth config get --all` shows all configuration
- `moth init` creates project-level configuration

### **Quality Requirements**
- Profile operations complete in <1 second
- Watch mode detects changes within debounce period
- CI detection works for all major CI systems
- Memory usage <200MB during watch mode
- Configuration files use standard YAML format

### **Integration Requirements**
- Compatible with all existing functionality
- Backward compatible CLI interface
- Works with existing test specifications
- Integrates with existing reporting system

## Breaking Changes

**None** - All new functionality is additive:
- All existing commands continue to work unchanged
- New commands and options require explicit usage
- Default behavior remains the same

## Alternative Approaches Considered

### **Option A: Configuration Files Only**
- Use only configuration files, no CLI options
- **Rejected**: CLI options provide better discoverability and CI/CD compatibility

### **Option B: Separate Binaries**
- Create `moth-profile`, `moth-watch`, `moth-config` binaries
- **Rejected**: Users expect integrated CLI experience

### **Option C: Plugin Architecture**
- Make advanced features optional plugins
- **Rejected**: Increases complexity, features are core functionality

## Testing Strategy

### **Unit Tests**
- Test all new CLI argument parsing
- Test profile management operations
- Test watch mode configuration and file detection
- Test CI detection and configuration generation
- Test global configuration management

### **Integration Tests**
- Test complete profile workflow (save → load → use)
- Test watch mode with real file changes
- Test CI mode with different CI environments
- Test combination of features (profile + watch + CI)

### **End-to-End Tests**
- Test enterprise workflow: init → profile → watch → CI
- Test team collaboration: export → import → use
- Test CI/CD pipeline integration
- Test performance with large configurations

## Rollout Plan

### **Phase 1: Core Commands (Week 1)**
- Implement profile management commands
- Implement watch mode command
- Basic CI integration

### **Phase 2: Enhanced Arguments (Week 2)**
- Extend RunArgs and ValidateArgs with advanced options
- Add profile loading and CI optimization
- Add environment variable integration

### **Phase 3: Configuration Management (Week 3)**
- Implement global configuration commands
- Add project initialization
- Add comprehensive CI detection

### **Phase 4: Polish and Documentation (Week 4)**
- Comprehensive testing and bug fixes
- Documentation and examples
- Performance optimization

This implementation will unlock the full enterprise configuration capabilities, making moth a truly enterprise-grade tool suitable for team collaboration and CI/CD integration. 