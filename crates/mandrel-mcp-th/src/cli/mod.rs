//! Command-line interface for Mandrel MCP Test Harness

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser, Debug)]
#[command(name = "moth")]
#[command(about = "Mandrel MCP Test Harness - moth binary for command-line testing")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(long, global = true, default_value = "json")]
    pub output: OutputFormat,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run test specifications
    Test {
        /// Path to test specification file or directory
        spec: PathBuf,

        /// Output file for test results
        #[arg(short, long)]
        output_file: Option<PathBuf>,

        /// Stop execution on first failure
        #[arg(short, long)]
        fail_fast: bool,

        /// Test filter pattern
        #[arg(short = 'F', long)]
        filter: Option<String>,

        /// Maximum number of concurrent tests
        #[arg(short, long, default_value = "4")]
        concurrency: usize,
    },

    /// Validate test specifications
    Validate {
        /// Path to test specification file or directory
        spec: PathBuf,
    },

    /// List available tests
    List {
        /// Path to test specification file or directory
        spec: PathBuf,

        /// Show detailed test information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show version information
    Version,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Html,
    Junit,
    Text,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Junit => write!(f, "junit"),
            OutputFormat::Text => write!(f, "text"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_test_command() {
        let args = vec![
            "moth",
            "test",
            "spec.yaml",
            "--fail-fast",
            "--output-file",
            "results.json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(!cli.verbose);
        assert!(matches!(cli.output, OutputFormat::Json));

        if let Commands::Test {
            spec,
            output_file,
            fail_fast,
            filter,
            concurrency,
        } = cli.command
        {
            assert_eq!(spec, PathBuf::from("spec.yaml"));
            assert_eq!(output_file, Some(PathBuf::from("results.json")));
            assert!(fail_fast);
            assert_eq!(filter, None);
            assert_eq!(concurrency, 4);
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_cli_parsing_validate_command() {
        let args = vec!["moth", "validate", "spec.yaml"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Validate { spec } = cli.command {
            assert_eq!(spec, PathBuf::from("spec.yaml"));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn test_cli_parsing_list_command() {
        let args = vec!["moth", "list", "spec.yaml", "--detailed"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::List { spec, detailed } = cli.command {
            assert_eq!(spec, PathBuf::from("spec.yaml"));
            assert!(detailed);
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_cli_parsing_version_command() {
        let args = vec!["moth", "version"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(matches!(cli.command, Commands::Version));
    }

    #[test]
    fn test_cli_global_flags() {
        let args = vec!["moth", "--verbose", "--output", "html", "test", "spec.yaml"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert!(matches!(cli.output, OutputFormat::Html));
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Junit.to_string(), "junit");
        assert_eq!(OutputFormat::Text.to_string(), "text");
    }

    #[test]
    fn test_cli_with_all_test_options() {
        let args = vec![
            "moth",
            "test",
            "spec.yaml",
            "--fail-fast",
            "--output-file",
            "results.json",
            "-F",
            "filesystem",
            "--concurrency",
            "8",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Test {
            spec,
            output_file,
            fail_fast,
            filter,
            concurrency,
        } = cli.command
        {
            assert_eq!(spec, PathBuf::from("spec.yaml"));
            assert_eq!(output_file, Some(PathBuf::from("results.json")));
            assert!(fail_fast);
            assert_eq!(filter, Some("filesystem".to_string()));
            assert_eq!(concurrency, 8);
        } else {
            panic!("Expected Test command");
        }
    }
}
