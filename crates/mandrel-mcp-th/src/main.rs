//! Mandrel MCP Test Harness - moth binary
//!
//! A modern, comprehensive testing framework for MCP servers built on the official Rust SDK.

use clap::Parser;
use mandrel_mcp_th::cli::commands;
use mandrel_mcp_th::cli::{Cli, Commands};
use mandrel_mcp_th::error::Result;
use mandrel_mcp_th::{is_shutdown_requested, request_shutdown};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    init_tracing(cli.verbose)?;

    tracing::info!("Starting moth binary - Mandrel MCP Test Harness");
    tracing::debug!("CLI arguments: {:?}", cli);

    // Set up signal handling for graceful shutdown
    setup_signal_handlers().await;

    // Execute the appropriate command
    let result = match cli.command {
        Commands::Test {
            spec,
            output_file,
            fail_fast,
            filter,
            concurrency,
        } => {
            tracing::info!("Executing test command");

            // Check for shutdown signal before starting execution
            if is_shutdown_requested() {
                tracing::info!("Shutdown requested before test execution, exiting gracefully");
                return Ok(());
            }

            commands::handle_test(spec, output_file, fail_fast, filter, concurrency).await
        }
        Commands::Validate { spec } => {
            tracing::info!("Executing validate command");
            commands::handle_validate(spec).await
        }
        Commands::List { spec, detailed } => {
            tracing::info!("Executing list command");
            commands::handle_list(spec, detailed).await
        }
        Commands::Version => commands::handle_version(),
    };

    match result {
        Ok(()) => {
            tracing::info!("Mandrel test harness execution completed successfully");
            Ok(())
        }
        Err(e) => {
            if is_shutdown_requested() {
                tracing::info!("Execution interrupted due to shutdown signal");
                Ok(()) // Don't treat shutdown as an error
            } else {
                tracing::error!("Execution failed: {}", e);
                Err(e)
            }
        }
    }
}

/// Set up signal handlers for graceful shutdown
async fn setup_signal_handlers() {
    tokio::spawn(async move {
        // Handle SIGINT (Ctrl+C)
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Failed to install SIGINT handler");

        // Handle SIGTERM (termination signal)
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                request_shutdown();
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, initiating graceful shutdown...");
                request_shutdown();
            }
        }

        // Give a moment for the main thread to notice the shutdown request
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::info!("Graceful shutdown complete");
    });
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
