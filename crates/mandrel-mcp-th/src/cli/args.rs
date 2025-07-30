use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mandrel-mcp-th")]
#[command(about = "Mandrel MCP Test Harness - Professional testing and reporting")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Use named configuration profile
    #[arg(long, short = 'p')]
    pub profile: Option<String>,

    /// Auto-detect CI environment and apply optimizations
    #[arg(long)]
    pub detect_ci: bool,

    /// Override environment detection (for testing CI configurations locally)
    #[arg(long)]
    pub simulate_ci: Option<CiSystem>,

    /// Configuration directory for profiles and settings
    #[arg(long, default_value = "~/.config/mandrel-mcp-th")]
    pub config_dir: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate test reports from execution results
    Report(ReportArgs),

    /// Run tests and generate reports
    Run(RunArgs),

    /// Validate configuration files
    Validate(ValidateArgs),

    /// Manage configuration profiles
    Profile(ProfileArgs),

    /// Watch files and auto-generate reports
    Watch(WatchArgs),
}

#[derive(Args, Debug)]
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
    #[arg(long, action = clap::ArgAction::Set, default_value = "true")]
    pub include_performance: bool,

    /// Include validation details in reports
    #[arg(long, action = clap::ArgAction::Set, default_value = "true")]
    pub include_validation: bool,
}

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
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Configuration file to validate
    #[arg()]
    pub config: PathBuf,

    /// Enable strict validation mode (fail on warnings)
    #[arg(long)]
    pub strict: bool,

    /// Output directory for validation reports
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Report formats to generate for validation results
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,

    /// Check JSONPath expressions in test cases
    #[arg(long)]
    pub check_jsonpath: bool,

    /// Validate JSON schema compliance
    #[arg(long)]
    pub check_schema: bool,

    /// Validate MCP protocol compliance
    #[arg(long)]
    pub check_protocol: bool,

    /// Enable all validation checks
    #[arg(long)]
    pub check_all: bool,

    /// Enable detailed validation diagnostics
    #[arg(long)]
    pub detailed: bool,

    /// Only validate, don't suggest fixes
    #[arg(long)]
    pub no_suggestions: bool,
}

#[derive(Args, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommand {
    /// Save current configuration as a named profile
    Save(ProfileSaveArgs),

    /// Load and apply a named profile
    Load(ProfileLoadArgs),

    /// List all available profiles
    List,

    /// Delete a named profile
    Delete(ProfileDeleteArgs),

    /// Export profile to file
    Export(ProfileExportArgs),

    /// Import profile from file
    Import(ProfileImportArgs),

    /// Show profile details
    Show(ProfileShowArgs),
}

#[derive(Args, Debug)]
pub struct ProfileSaveArgs {
    /// Profile name
    pub name: String,

    /// Profile description
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Report formats to include
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,

    /// Template to use
    #[arg(short = 't', long)]
    pub template: Option<TemplateName>,

    /// Organization strategy
    #[arg(long)]
    pub organization: Option<OrganizationStrategy>,

    /// Timestamp format
    #[arg(long)]
    pub timestamp: Option<TimestampFormat>,

    /// Include performance metrics
    #[arg(long)]
    pub include_performance: Option<bool>,

    /// Include validation details
    #[arg(long)]
    pub include_validation: Option<bool>,

    /// Set as default profile
    #[arg(long)]
    pub set_default: bool,
}

#[derive(Args, Debug)]
pub struct ProfileLoadArgs {
    /// Profile name to load
    pub name: String,

    /// Apply profile settings globally
    #[arg(long)]
    pub global: bool,
}

#[derive(Args, Debug)]
pub struct ProfileDeleteArgs {
    /// Profile name to delete
    pub name: String,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ProfileExportArgs {
    /// Profile name to export
    pub name: String,

    /// Output file path
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Export format
    #[arg(long, default_value = "json")]
    pub format: ExportFormat,
}

#[derive(Args, Debug)]
pub struct ProfileImportArgs {
    /// Input file path
    pub input: PathBuf,

    /// Override profile name
    #[arg(long)]
    pub name: Option<String>,

    /// Overwrite existing profile
    #[arg(long)]
    pub overwrite: bool,
}

#[derive(Args, Debug)]
pub struct ProfileShowArgs {
    /// Profile name to show
    pub name: String,

    /// Show in detailed format
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    #[command(subcommand)]
    pub command: WatchCommand,
}

#[derive(Subcommand, Debug)]
pub enum WatchCommand {
    /// Start watching files for changes
    Start(WatchStartArgs),

    /// Stop all watchers
    Stop,

    /// Show watch status
    Status,
}

#[derive(Args, Debug)]
pub struct WatchStartArgs {
    /// Input files or directories to watch
    pub inputs: Vec<PathBuf>,

    /// Output directory for generated reports
    #[arg(short = 'o', long, default_value = "./reports")]
    pub output: PathBuf,

    /// File patterns to watch (glob patterns)
    #[arg(short = 'p', long, value_delimiter = ',')]
    pub patterns: Vec<String>,

    /// Report formats to generate
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub formats: Vec<ReportFormat>,

    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce: u64,

    /// Auto-open reports in browser
    #[arg(long)]
    pub auto_open: bool,

    /// Run in background (daemonize)
    #[arg(long)]
    pub daemon: bool,

    /// Profile to use for report generation
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Junit,
    Html,
    Markdown,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemplateName {
    Professional,
    Executive,
    Technical,
    Minimal,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum OrganizationStrategy {
    #[default]
    Flat, // All files in output directory
    ByDate,     // Organized by date: 2025/01/15/reports
    ByFormat,   // Organized by format: json/, html/, etc.
    ByTemplate, // Organized by template: professional/, technical/
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum TimestampFormat {
    #[default]
    Iso, // 2025-01-15T10-30-45Z
    Unix,  // 1736935845
    Human, // 2025-01-15_10-30-45
    None,  // No timestamp
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CiSystem {
    GitHubActions,
    Jenkins,
    GitLabCi,
    CircleCi,
    Travis,
    Buildkite,
    TeamCity,
    AzureDevOps,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

/// Parse key=value pairs for custom fields
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value format: {s}"));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

impl ReportFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Junit => "xml",
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "md",
        }
    }

    pub fn to_directory_name(&self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Junit => "junit",
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "markdown",
        }
    }
}

impl TemplateName {
    pub fn to_directory_name(&self) -> &'static str {
        match self {
            TemplateName::Professional => "professional",
            TemplateName::Executive => "executive",
            TemplateName::Technical => "technical",
            TemplateName::Minimal => "minimal",
        }
    }
}
