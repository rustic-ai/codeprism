//! CodePrism MCP Server Binary
//!
//! This binary provides the main entry point for running the CodePrism MCP server.
//! It handles command-line arguments, configuration loading, and server startup.

use anyhow::Result;
use clap::Parser;
use codeprism_mcp_server::{CodePrismMcpServer, Config};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// CodePrism - Code analysis and insights tool
#[derive(Parser, Debug)]
#[command(
    name = "codeprism",
    version = codeprism_mcp_server::VERSION,
    about = "CodePrism - Advanced code analysis and insights"
)]
struct Cli {
    /// Run as MCP (Model Context Protocol) server
    #[arg(long)]
    mcp: bool,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Configuration profile (development, production, enterprise)
    #[arg(short, long, value_name = "PROFILE")]
    profile: Option<String>,

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
    let config = load_config(cli.config.as_deref(), cli.profile.as_deref()).await?;

    // Validate config and exit if requested
    if cli.validate_config {
        info!("Configuration is valid");
        return Ok(());
    }

    // Check what mode to run in
    if cli.mcp {
        // Run as MCP server
        info!("Starting MCP server mode");
        let server = CodePrismMcpServer::new(config).await?;
        server.run().await?;
    } else {
        // Show usage information when no mode is specified
        println!("CodePrism v{}", codeprism_mcp_server::VERSION);
        println!("Advanced code analysis and insights");
        println!();
        println!("For Claude Desktop integration, use:");
        println!("  codeprism --mcp");
        println!();
        println!("For help:");
        println!("  codeprism --help");
    }

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

/// Load configuration from file, environment, or use defaults
async fn load_config(config_path: Option<&str>, profile: Option<&str>) -> Result<Config> {
    match config_path {
        Some(path) => {
            info!("Loading configuration from file: {}", path);
            Config::from_file(path).await.map_err(Into::into)
        }
        None => {
            // Set profile from CLI if provided
            if let Some(prof) = profile {
                std::env::set_var("CODEPRISM_PROFILE", prof);
                info!("Using configuration profile: {}", prof);
            }

            // Try loading from environment variables first
            if std::env::var("CODEPRISM_PROFILE").is_ok()
                || std::env::var("CODEPRISM_MEMORY_LIMIT_MB").is_ok()
                || std::env::var("CODEPRISM_BATCH_SIZE").is_ok()
            {
                info!("Loading configuration from environment variables");
                Config::from_env().await.map_err(Into::into)
            } else {
                info!("Using default configuration (development profile)");
                Ok(Config::default())
            }
        }
    }
}
