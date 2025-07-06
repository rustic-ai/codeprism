//! Mandrel MCP Test Harness - moth binary
//!
//! A modern, comprehensive testing framework for MCP servers built on the official Rust SDK.

use mandrel_mcp_th::cli::CliApp;
use mandrel_mcp_th::error::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments, but handle the case where tests might invoke with just --quiet
    let args: Vec<String> = std::env::args().collect();

    // If invoked with just --quiet (from test harness), exit gracefully
    if args.len() == 2 && args[1] == "--quiet" {
        std::process::exit(0);
    }

    // Initialize CLI application
    let app = CliApp::new()?;

    // Set up signal handling for graceful shutdown
    let ctrl_c = signal::ctrl_c();

    // Run CLI application
    tokio::select! {
        result = app.run() => {
            let exit_code = result?;
            std::process::exit(exit_code);
        }
        _ = ctrl_c => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
            std::process::exit(130); // Standard exit code for SIGINT
        }
    }
}
