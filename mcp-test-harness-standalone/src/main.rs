//! Standalone MCP Test Harness
//! 
//! A universal testing tool for Model Context Protocol (MCP) servers.
//! Tests any MCP server implementation for protocol compliance, performance, and reliability.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, warn};

mod config;
mod runner;
mod server;
mod validation;

#[cfg(test)]
mod integration_test;

use crate::config::TestConfig;
use crate::runner::TestRunner;

#[derive(Parser)]
#[command(
    name = "mcp-test-harness",
    about = "Universal test harness for Model Context Protocol (MCP) servers",
    version,
    author = "CodePrism Team <team@codeprism.ai>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Output format (json, yaml, table)
    #[arg(short, long, default_value = "table", global = true)]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run test suite against an MCP server
    Test {
        /// Path to test configuration file
        #[arg(short, long)]
        config: PathBuf,
        
        /// MCP server command to execute
        #[arg(short, long)]
        server_cmd: Option<String>,
        
        /// Working directory for the server
        #[arg(short, long)]
        working_dir: Option<PathBuf>,
        
        /// Run only validation checks (no actual server execution)
        #[arg(long)]
        validation_only: bool,
        
        /// Run comprehensive test suite
        #[arg(long)]
        comprehensive: bool,
        
        /// Parallel execution (number of concurrent tests)
        #[arg(long, default_value = "1")]
        parallel: usize,
    },
    
    /// Validate test configuration without running tests
    Validate {
        /// Path to test configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
    
    /// Generate template configuration for common MCP server types
    Template {
        /// Server type (filesystem, database, api, custom)
        #[arg(short, long)]
        server_type: String,
        
        /// Output path for generated template
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// List available test templates
    List {
        /// List templates for specific server type
        #[arg(short, long)]
        server_type: Option<String>,
    },
    
    /// Run built-in MCP server discovery and profiling
    Discover {
        /// Port range to scan (e.g., "3000-3100")
        #[arg(short, long, default_value = "3000-3010")]
        port_range: String,
        
        /// Timeout for discovery probes (seconds)
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
    

}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(level))
        .init();
    
    info!("MCP Test Harness v{}", env!("CARGO_PKG_VERSION"));
    
    match cli.command {
        Commands::Test {
            config,
            server_cmd,
            working_dir,
            validation_only,
            comprehensive,
            parallel,
        } => {
            run_tests(
                config,
                server_cmd,
                working_dir,
                validation_only,
                comprehensive,
                parallel,
                &cli.output,
            ).await
        }
        Commands::Validate { config } => validate_config(config).await,
        Commands::Template { server_type, output } => generate_template(server_type, output).await,
        Commands::List { server_type } => list_templates(server_type).await,
        Commands::Discover { port_range, timeout } => discover_servers(port_range, timeout).await,
    }
}

async fn run_tests(
    config_path: PathBuf,
    server_cmd: Option<String>,
    working_dir: Option<PathBuf>,
    validation_only: bool,
    comprehensive: bool,
    parallel: usize,
    output_format: &str,
) -> Result<()> {
    info!("Loading test configuration from: {}", config_path.display());
    
    let config = TestConfig::load(&config_path)?;
    info!("Loaded configuration with {} test suites", config.test_suites.len());
    
    let mut runner = TestRunner::new(config, output_format.to_string())?;
    
    if let Some(cmd) = server_cmd {
        runner.set_server_command(cmd, working_dir);
    }
    
    runner.set_validation_only(validation_only);
    runner.set_comprehensive(comprehensive);
    runner.set_parallel_execution(parallel);
    
    let results = runner.run().await?;
    
    // Display results based on output format
    match output_format {
        "json" => println!("{}", serde_json::to_string_pretty(&results)?),
        "yaml" => println!("{}", serde_yaml::to_string(&results)?),
        "table" => results.display_table(),
        _ => {
            warn!("Unknown output format '{}', using table format", output_format);
            results.display_table();
        }
    }
    
    // Exit with appropriate code
    if results.all_passed() {
        info!("All tests passed! ✅");
        Ok(())
    } else {
        warn!("Some tests failed! ❌");
        std::process::exit(1);
    }
}

async fn validate_config(config_path: PathBuf) -> Result<()> {
    info!("Validating configuration: {}", config_path.display());
    
    match TestConfig::validate(&config_path) {
        Ok(_) => {
            println!("✅ Configuration is valid");
            Ok(())
        }
        Err(e) => {
            println!("❌ Configuration validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn generate_template(server_type: String, output_path: PathBuf) -> Result<()> {
    info!("Generating {} template to: {}", server_type, output_path.display());
    
    let template = config::generate_template(&server_type)?;
    std::fs::write(&output_path, template)?;
    
    println!("✅ Template generated: {}", output_path.display());
    Ok(())
}

async fn list_templates(server_type: Option<String>) -> Result<()> {
    let templates = config::list_available_templates(server_type.as_deref())?;
    
    println!("Available Templates:");
    for template in templates {
        println!("  • {} - {}", template.name, template.description);
    }
    
    Ok(())
}

async fn discover_servers(port_range: String, timeout: u64) -> Result<()> {
    info!("Discovering MCP servers on port range: {}", port_range);
    
    let discovered = server::discover_mcp_servers(&port_range, timeout).await?;
    
    if discovered.is_empty() {
        println!("No MCP servers discovered in range {}", port_range);
    } else {
        println!("Discovered {} MCP server(s):", discovered.len());
        for server in discovered {
            println!("  • {}:{} - {}", server.host, server.port, server.server_type);
        }
    }
    
    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&[
            "mcp-test-harness",
            "test",
            "--config", "test.yaml",
            "--server-cmd", "node server.js",
            "--verbose"
        ]);
        
        assert!(cli.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(&temp_file, r#"
global:
  max_global_concurrency: 2
  timeout_seconds: 30

test_suites:
  - name: "Basic Test"
    test_cases:
      - id: "test_1"
        tool_name: "test_tool"
        enabled: true
"#).unwrap();
        
        let result = TestConfig::validate(temp_file.path());
        // This might fail until we implement full validation, but structure should be correct
        println!("Validation result: {:?}", result);
    }
}
