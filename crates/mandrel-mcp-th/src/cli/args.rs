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
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate test reports from execution results
    Report(ReportArgs),

    /// Run tests and generate reports
    Run(RunArgs),

    /// Validate configuration files
    Validate(ValidateArgs),
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
    #[arg(long, default_value = "true")]
    pub include_performance: bool,

    /// Include validation details in reports
    #[arg(long, default_value = "true")]
    pub include_validation: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Test configuration file
    #[arg(short = 'c', long)]
    pub config: PathBuf,

    /// Output directory for generated reports
    #[arg(short = 'o', long, default_value = "./reports")]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Configuration file to validate
    #[arg()]
    pub config: PathBuf,
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

/// Parse key=value pairs for custom fields
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value format: {}", s));
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
