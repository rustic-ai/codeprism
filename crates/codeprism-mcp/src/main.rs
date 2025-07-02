//! CodePrism MCP Server Binary
//!
//! Native RMCP MCP Server using the official Rust SDK with stdio and SSE transport support.
//! Provides all CodePrism code analysis tools as native MCP tools with comprehensive schema documentation.

use anyhow::Result;
use clap::{Arg, Command};
use rmcp::transport::sse_server::SseServer;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing::{error, info, warn};

use codeprism_mcp::server::CodePrismRmcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logging to stderr so it doesn't interfere with MCP stdio)
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stderr)
        .init();

    let matches = Command::new("codeprism-mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .author("DragonScale Team")
        .about("CodePrism Model Context Protocol Server (Native RMCP)")
        .long_about(
            "A native RMCP (Rust Model Context Protocol) server that provides advanced code analysis \
             and repository exploration tools. Uses the official MCP Rust SDK with comprehensive \
             tool support including complexity analysis, dependency tracing, security analysis, \
             performance profiling, and more. Integrates seamlessly with MCP clients like \
             Claude Desktop, Cursor, and other AI applications."
        )
        .arg(
            Arg::new("repository")
                .help("Path to the repository to analyze")
                .value_name("PATH")
                .index(1)
                .required(false)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("transport")
                .long("transport")
                .help("Transport protocol to use: 'stdio' or 'sse'")
                .value_name("TRANSPORT")
                .default_value("stdio")
        )
        .arg(
            Arg::new("host")
                .long("host")
                .help("Host to bind for SSE transport (default: 127.0.0.1)")
                .value_name("HOST")
                .default_value("127.0.0.1")
        )
        .arg(
            Arg::new("port")
                .long("port")
                .help("Port to bind for SSE transport (default: 3000)")
                .value_name("PORT")
                .default_value("3000")
        )
        .get_matches();

    // Set log level based on verbose flag
    if matches.get_flag("verbose") {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    info!(
        "Starting CodePrism Native RMCP MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create the native RMCP server
    let mut server = CodePrismRmcpServer::new().map_err(|e| {
        error!("Failed to create RMCP server: {}", e);
        e
    })?;

    // Initialize with repository if provided (check CLI args first, then environment variable)
    let repo_path_str = matches
        .get_one::<String>("repository")
        .cloned()
        .or_else(|| std::env::var("REPOSITORY_PATH").ok());

    if let Some(repo_path) = repo_path_str {
        let path = PathBuf::from(repo_path);

        if !path.exists() {
            error!("Repository path does not exist: {}", path.display());
            std::process::exit(1);
        }

        if !path.is_dir() {
            error!("Repository path is not a directory: {}", path.display());
            std::process::exit(1);
        }

        info!("Initializing with repository: {}", path.display());

        // Warn about large repositories
        if let Ok(_metadata) = std::fs::metadata(&path) {
            if let Ok(output) = std::process::Command::new("du")
                .args(["-sh", path.to_str().unwrap()])
                .output()
            {
                if let Ok(size_str) = String::from_utf8(output.stdout) {
                    let size = size_str.split_whitespace().next().unwrap_or("unknown");
                    info!("Repository size: {}", size);

                    // Parse size and warn if large
                    if size.contains('G')
                        && size.chars().next().unwrap_or('0').to_digit(10).unwrap_or(0) > 1
                    {
                        warn!("Large repository detected ({}). Consider using filtering in your MCP client configuration.", size);
                        warn!("Use content filtering tools like search_content with file_pattern to limit scope.");
                        warn!("Use find_files with exclude_patterns to filter out unwanted directories.");
                    }
                }
            }
        }

        server
            .initialize_with_repository(&path)
            .await
            .map_err(|e| {
                error!("Failed to initialize repository: {}", e);
                e
            })?;
    } else {
        info!("No repository specified - server will start without repository context");
        info!("Repository can be specified as: codeprism-mcp <path> or via REPOSITORY_PATH environment variable");
        info!("Repository will be loaded on first tool use if tools can infer the path.");
    }

    // Select transport based on configuration
    let transport = matches.get_one::<String>("transport").unwrap();

    match transport.as_str() {
        "stdio" => {
            info!("Starting native RMCP server with stdio transport");
            info!("Available tools: repository_stats, content_stats, analyze_complexity, search_symbols,");
            info!(
                "                 search_content, find_files, find_references, find_dependencies,"
            );
            info!(
                "                 trace_path, explain_symbol, trace_data_flow, trace_inheritance,"
            );
            info!("                 detect_patterns, analyze_decorators, find_duplicates, find_unused_code,");
            info!("                 analyze_transitive_dependencies, analyze_security, analyze_performance, analyze_api_surface");

            // Use stdio transport following mcp-containerd pattern
            let service = server
                .serve((tokio::io::stdin(), tokio::io::stdout()))
                .await
                .map_err(|e| {
                    error!("RMCP server error: {}", e);
                    anyhow::anyhow!("Server failed: {}", e)
                })?;

            service.waiting().await?;
        }
        "sse" => {
            let host = matches.get_one::<String>("host").unwrap();
            let port = matches
                .get_one::<String>("port")
                .unwrap()
                .parse::<u16>()
                .map_err(|e| {
                    error!("Invalid port number: {}", e);
                    e
                })?;

            info!(
                "Starting native RMCP server with SSE transport on {}:{}",
                host, port
            );
            info!("Server will be available at: http://{}:{}/sse", host, port);
            info!("Available tools: repository_stats, content_stats, analyze_complexity, search_symbols,");
            info!(
                "                 search_content, find_files, find_references, find_dependencies,"
            );
            info!(
                "                 trace_path, explain_symbol, trace_data_flow, trace_inheritance,"
            );
            info!("                 detect_patterns, analyze_decorators, find_duplicates, find_unused_code,");
            info!("                 analyze_transitive_dependencies, analyze_security, analyze_performance, analyze_api_surface");

            // Use SSE server following mcp-containerd pattern
            let ct = SseServer::serve(format!("{}:{}", host, port).parse()?)
                .await?
                .with_service(move || server.clone());

            tokio::signal::ctrl_c().await?;
            ct.cancel();
        }
        _ => {
            error!("Unsupported transport: {}. Use 'stdio' or 'sse'", transport);
            std::process::exit(1);
        }
    }

    info!("CodePrism Native RMCP MCP Server stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        // Test that the server can be created successfully
        let result = CodePrismRmcpServer::new();
        assert!(result.is_ok(), "Server creation should succeed");
    }

    #[test]
    fn test_cli_argument_parsing() {
        // Test basic argument parsing
        let app = Command::new("test")
            .arg(Arg::new("repository").index(1))
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("transport")
                    .long("transport")
                    .default_value("stdio"),
            );

        // Test default values
        let matches = app.clone().try_get_matches_from(vec!["test"]).unwrap();
        assert_eq!(
            matches.get_one::<String>("transport"),
            Some(&"stdio".to_string())
        );

        // Test with repository path
        let matches = app
            .clone()
            .try_get_matches_from(vec!["test", "/path/to/repo"])
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("repository"),
            Some(&"/path/to/repo".to_string())
        );

        // Test with transport option
        let matches = app
            .try_get_matches_from(vec!["test", "--transport", "sse"])
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("transport"),
            Some(&"sse".to_string())
        );
    }
}
