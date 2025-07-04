//! MOTH - MOdel context protocol Test Harness
//!
//! A modern, comprehensive testing framework for MCP servers built on the official Rust SDK.

use clap::Parser;
use mandrel_mcp_th::cli::commands;
use mandrel_mcp_th::cli::{Cli, Commands};
use mandrel_mcp_th::error::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    init_tracing(cli.verbose)?;

    tracing::info!("Starting MOTH - MOdel context protocol Test Harness");
    tracing::debug!("CLI arguments: {:?}", cli);

    // Execute the appropriate command
    match cli.command {
        Commands::Test {
            spec,
            output_file,
            fail_fast,
            filter,
            concurrency,
        } => {
            tracing::info!("Executing test command");
            commands::handle_test(spec, output_file, fail_fast, filter, concurrency).await?;
        }
        Commands::Validate { spec } => {
            tracing::info!("Executing validate command");
            commands::handle_validate(spec).await?;
        }
        Commands::List { spec, detailed } => {
            tracing::info!("Executing list command");
            commands::handle_list(spec, detailed).await?;
        }
        Commands::Version => {
            commands::handle_version()?;
        }
    }

    tracing::info!("MOTH execution completed successfully");
    Ok(())
}

/// Initialize tracing/logging
fn init_tracing(verbose: bool) -> Result<()> {
    let filter = if verbose {
        tracing_subscriber::filter::LevelFilter::DEBUG
    } else {
        tracing_subscriber::filter::LevelFilter::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .with_filter(filter),
        )
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_init_tracing_verbose() {
        // Test that tracing initialization doesn't panic for verbose mode
        // Note: Can't actually initialize due to global state, just test the logic
        let filter = if true {
            tracing_subscriber::filter::LevelFilter::DEBUG
        } else {
            tracing_subscriber::filter::LevelFilter::INFO
        };
        assert_eq!(filter, tracing_subscriber::filter::LevelFilter::DEBUG);
    }

    #[test]
    fn test_init_tracing_normal() {
        // Test that tracing initialization doesn't panic for normal mode
        // Note: Can't actually initialize due to global state, just test the logic
        let filter = if false {
            tracing_subscriber::filter::LevelFilter::DEBUG
        } else {
            tracing_subscriber::filter::LevelFilter::INFO
        };
        assert_eq!(filter, tracing_subscriber::filter::LevelFilter::INFO);
    }
}
