//! Prism MCP Server Binary
//! 
//! This binary provides a command-line interface for running the Prism MCP server.
//! It supports stdio transport for integration with MCP clients like Claude Desktop and Cursor.

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber;

use prism_mcp::server::McpServer;

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

    let matches = Command::new("prism-mcp")
        .version("0.1.0")
        .author("DragonScale Team")
        .about("Prism Model Context Protocol Server")
        .long_about(
            "A Model Context Protocol (MCP) compliant server that provides access to code repositories \
             through standardized Resources, Tools, and Prompts. Integrates with MCP clients like \
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
            Arg::new("memory-limit")
                .long("memory-limit")
                .help("Memory limit for indexing in MB (default: 4096)")
                .value_name("MB")
        )
        .arg(
            Arg::new("batch-size")
                .long("batch-size")
                .help("Batch size for parallel processing (default: 30)")
                .value_name("SIZE")
        )
        .arg(
            Arg::new("max-file-size")
                .long("max-file-size")
                .help("Maximum file size to process in MB (default: 10)")
                .value_name("MB")
        )
        .arg(
            Arg::new("exclude-dirs")
                .long("exclude-dirs")
                .help("Comma-separated list of directories to exclude (default: comprehensive list including .tox, venv, node_modules, etc.)")
                .value_name("DIRS")
        )
        .arg(
            Arg::new("include-extensions")
                .long("include-extensions")
                .help("Comma-separated list of file extensions to include (default: common programming languages)")
                .value_name("EXTS")
        )
        .arg(
            Arg::new("disable-memory-limit")
                .long("disable-memory-limit")
                .help("Disable memory limit checking (use with caution)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include-deps")
                .long("include-deps")
                .help("Include dependency directories (.tox, venv, node_modules) for complete code analysis")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("smart-deps")
                .long("smart-deps")
                .help("Smart dependency scanning - include only public APIs and commonly referenced files")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Set log level based on verbose flag
    if matches.get_flag("verbose") {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    info!("Starting Prism MCP Server");

    // Parse configuration options
    let memory_limit_mb = matches.get_one::<String>("memory-limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4096);
    
    let batch_size = matches.get_one::<String>("batch-size")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30);
    
    let max_file_size_mb = matches.get_one::<String>("max-file-size")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    let disable_memory_limit = matches.get_flag("disable-memory-limit");
    let include_deps = matches.get_flag("include-deps");
    let smart_deps = matches.get_flag("smart-deps");
    
    let exclude_dirs = matches.get_one::<String>("exclude-dirs")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
        .unwrap_or_else(|| {
            let mut dirs = vec![
                ".git".to_string(),
                "build".to_string(),
                "dist".to_string(),
                // IDE and editor directories
                ".vscode".to_string(),
                ".idea".to_string(),
                // OS files  
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ];

            // Add dependency directories unless specifically requested to include them
            if !include_deps && !smart_deps {
                dirs.extend(vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".venv".to_string(),
                    "__pycache__".to_string(),
                    "vendor".to_string(),
                    // Python virtual environments and package caches
                    ".tox".to_string(),
                    "venv".to_string(),
                    ".env".to_string(),
                    "env".to_string(),
                    ".pytest_cache".to_string(),
                    ".mypy_cache".to_string(),
                    ".ruff_cache".to_string(),
                    // Web build artifacts
                    ".next".to_string(),
                    ".nuxt".to_string(),
                    "coverage".to_string(),
                    ".coverage".to_string(),
                ]);
            } else if smart_deps {
                // In smart mode, exclude some dependency subdirectories but keep main ones
                dirs.extend(vec![
                    // Keep main dependency dirs but exclude their caches/builds
                    "__pycache__".to_string(),
                    ".pytest_cache".to_string(),
                    ".mypy_cache".to_string(),
                    ".ruff_cache".to_string(),
                    ".coverage".to_string(),
                ]);
            }

            dirs
        });
    
    let include_extensions = matches.get_one::<String>("include-extensions")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
        .or_else(|| Some(vec![
            // Default to common programming language extensions
            "py".to_string(),
            "js".to_string(), 
            "ts".to_string(),
            "jsx".to_string(),
            "tsx".to_string(),
            "rs".to_string(),
            "java".to_string(),
            "cpp".to_string(),
            "c".to_string(),
            "h".to_string(),
            "hpp".to_string(),
            "go".to_string(),
            "php".to_string(),
            "rb".to_string(),
            "kt".to_string(),
            "swift".to_string(),
        ]));

    // Log configuration
    info!("Configuration:");
    info!("  Memory limit: {}MB{}", memory_limit_mb, if disable_memory_limit { " (disabled)" } else { "" });
    info!("  Batch size: {}", batch_size);
    info!("  Max file size: {}MB", max_file_size_mb);
    info!("  Dependency scanning: {}", 
        if include_deps { "Full (includes all dependencies)" }
        else if smart_deps { "Smart (includes dependency APIs only)" }
        else { "Minimal (excludes dependencies)" });
    info!("  Excluded directories: {:?}", exclude_dirs);
    if let Some(ref exts) = include_extensions {
        info!("  Included extensions: {:?}", exts);
    }

    // Create MCP server with custom configuration
    let server = McpServer::new_with_config(
        memory_limit_mb,
        batch_size,
        max_file_size_mb,
        disable_memory_limit,
        exclude_dirs,
        include_extensions,
        if include_deps { Some("include_all".to_string()) }
        else if smart_deps { Some("smart".to_string()) }
        else { Some("exclude".to_string()) },
    )
        .map_err(|e| {
            error!("Failed to create MCP server: {}", e);
            e
        })?;

    // Initialize with repository if provided
    if let Some(repo_path) = matches.get_one::<String>("repository") {
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
                .args(&["-sh", path.to_str().unwrap()])
                .output()
            {
                if let Ok(size_str) = String::from_utf8(output.stdout) {
                    let size = size_str.split_whitespace().next().unwrap_or("unknown");
                    info!("Repository size: {}", size);
                    
                    // Parse size and warn if large
                    if size.contains('G') && size.chars().next().unwrap_or('0').to_digit(10).unwrap_or(0) > 1 {
                        warn!("Large repository detected ({}). Consider using filtering options or increasing memory limit.", size);
                        warn!("Use --exclude-dirs to exclude directories like node_modules, target, .git");
                        warn!("Use --include-extensions to limit file types, e.g., --include-extensions py,js,ts");
                        warn!("Use --memory-limit to increase memory limit, e.g., --memory-limit 4096");
                    }
                }
            }
        }
        
        server.initialize_with_repository(&path).await
            .map_err(|e| {
                error!("Failed to initialize repository: {}", e);
                e
            })?;
    } else {
        info!("No repository specified - server will start without repository context");
        info!("Repository can be specified as: prism-mcp <path>");
    }

    // Run the server with stdio transport
    info!("Starting MCP server with stdio transport");
    server.run_stdio().await
        .map_err(|e| {
            error!("MCP server error: {}", e);
            e
        })?;

    info!("Prism MCP Server stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Integration tests for the binary would typically use external test harnesses
    // since testing stdio interaction requires careful setup of stdin/stdout mocking
    
    #[test]
    fn test_binary_compilation() {
        // This test just ensures the binary compiles successfully
        assert!(true);
    }
} 