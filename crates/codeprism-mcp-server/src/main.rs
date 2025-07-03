//! CodePrism MCP Server Binary
//!
//! This binary provides the main entry point for running the CodePrism MCP server.
//! It handles command-line arguments, configuration loading, and server startup.

use anyhow::Result;
use clap::Parser;
use codeprism_mcp_server::{CodePrismMcpServer, Config};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// CodePrism MCP Server - Expose code analysis capabilities via MCP protocol
#[derive(Parser, Debug)]
#[command(
    name = "codeprism-mcp-server",
    version = codeprism_mcp_server::VERSION,
    about = "CodePrism MCP Server - Code analysis via Model Context Protocol"
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Validate configuration and exit
    #[arg(long)]
    validate_config: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level)?;

    info!(
        "Starting {} v{}",
        codeprism_mcp_server::SERVER_NAME,
        codeprism_mcp_server::VERSION
    );

    // Load configuration
    let config = load_config(cli.config.as_deref()).await?;

    // Validate config and exit if requested
    if cli.validate_config {
        info!("Configuration is valid");
        return Ok(());
    }

    // Create and run the MCP server
    let server = CodePrismMcpServer::new(config).await?;
    server.run().await?;

    Ok(())
}

/// Initialize logging with the specified level
fn init_logging(log_level: &str) -> Result<()> {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .init();

    Ok(())
}

/// Load configuration from file or use defaults
async fn load_config(config_path: Option<&str>) -> Result<Config> {
    match config_path {
        Some(path) => {
            info!("Loading configuration from: {}", path);
            Config::from_file(path).await.map_err(Into::into)
        }
        None => {
            info!("Using default configuration");
            Ok(Config::default())
        }
    }
}
