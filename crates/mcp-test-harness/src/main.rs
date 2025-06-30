//! Generic MCP Test Harness CLI
//!
//! Command-line interface for testing any MCP (Model Context Protocol) server
//! implementation. This tool validates protocol compliance, tests capabilities,
//! and generates comprehensive reports.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mcp_test_harness_lib::{
    init, version, ReportFormat, SpecLoader, TestHarness, TestReport, TransportType,
};
use std::path::PathBuf;
use tracing::{error, info, warn};

// Use the specific ServerSpec type from schema module
use mcp_test_harness_lib::spec::schema::ServerSpec;

/// Generic MCP Server Test Harness
#[derive(Parser)]
#[command(
    name = "mcp-test-harness-lib",
    version = version(),
    about = "Test harness for MCP server protocol compliance and functionality",
    long_about = "A comprehensive testing tool for validating MCP (Model Context Protocol) \
                  server implementations. Tests protocol compliance, capabilities, and \
                  generates detailed reports. Designed to work with any MCP server."
)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Configuration file or server spec to use
    #[arg(short, long, value_name = "FILE")]
    spec: Option<PathBuf>,

    /// Command to start the MCP server
    #[arg(long, value_name = "COMMAND")]
    server_cmd: Option<String>,

    /// Arguments to pass to the server command
    #[arg(long, value_name = "ARGS")]
    server_args: Vec<String>,

    /// Transport type for the server connection
    #[arg(long, value_enum, default_value = "stdio")]
    transport: TransportType,

    /// Output directory for test reports
    #[arg(short, long, value_name = "DIR", default_value = "test-reports")]
    output: PathBuf,

    /// Report formats to generate
    #[arg(long, value_enum, default_values = ["html", "json"])]
    format: Vec<ReportFormat>,

    /// Timeout for server startup in seconds
    #[arg(long, default_value = "30")]
    startup_timeout: u64,

    /// Maximum test execution timeout in seconds
    #[arg(long, default_value = "60")]
    test_timeout: u64,

    /// Maximum number of concurrent tests
    #[arg(long, default_value = "4")]
    concurrency: usize,

    /// Stop on first test failure
    #[arg(long)]
    fail_fast: bool,

    /// Show what would be executed without running tests
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a specific MCP server using a specification file
    Test {
        /// Path to the server specification file
        spec_file: PathBuf,
        /// Override server command
        #[arg(long)]
        server_cmd: Option<String>,
        /// Override server arguments
        #[arg(long)]
        server_args: Vec<String>,
    },

    /// Discover server capabilities and generate a spec template
    Discover {
        /// Command to start the MCP server
        server_cmd: String,
        /// Arguments to pass to the server command
        server_args: Vec<String>,
        /// Output file for the generated spec
        #[arg(short, long, default_value = "discovered-server.yaml")]
        output: PathBuf,
    },

    /// Validate a server specification file
    Validate {
        /// Path to the specification file to validate
        spec_file: PathBuf,
    },

    /// Test only MCP protocol compliance (no spec required)
    Protocol {
        /// Command to start the MCP server
        server_cmd: String,
        /// Arguments to pass to the server command
        server_args: Vec<String>,
    },

    /// Generate example server specifications
    Examples {
        /// Output directory for example specs
        #[arg(short, long, default_value = "examples")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;

    // Initialize the test harness library
    init().context("Failed to initialize test harness")?;

    match &cli.command {
        Some(Commands::Test {
            spec_file,
            server_cmd,
            server_args,
        }) => {
            run_spec_test(
                spec_file.clone(),
                server_cmd.clone(),
                server_args.clone(),
                &cli,
            )
            .await
        }
        Some(Commands::Discover {
            server_cmd,
            server_args,
            output,
        }) => discover_server(server_cmd.clone(), server_args.clone()).await,
        Some(Commands::Validate { spec_file }) => validate_spec(spec_file.clone()).await,
        Some(Commands::Protocol {
            server_cmd,
            server_args,
        }) => test_protocol_only(server_cmd.clone(), server_args.clone(), &cli).await,
        Some(Commands::Examples { output }) => generate_examples(output.clone()).await,
        None => {
            // Default behavior - try to find spec or use provided server command
            if let Some(spec_file) = &cli.spec {
                run_spec_test(
                    spec_file.clone(),
                    cli.server_cmd.clone(),
                    cli.server_args.clone(),
                    &cli,
                )
                .await
            } else if let Some(server_cmd) = &cli.server_cmd {
                test_protocol_only(server_cmd.clone(), cli.server_args.clone(), &cli).await
            } else {
                // Look for default spec files
                find_and_run_default_spec(&cli).await
            }
        }
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: u8) -> Result<()> {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    std::env::set_var("RUST_LOG", level);
    Ok(())
}

/// Run tests using a specification file
async fn run_spec_test(
    spec_file: PathBuf,
    server_cmd: Option<String>,
    server_args: Vec<String>,
    cli: &Cli,
) -> Result<()> {
    info!("Loading server specification: {}", spec_file.display());

    // Load server specification
    let loader = SpecLoader::new().context("Failed to create spec loader")?;
    let mut spec = loader
        .load_spec(&spec_file)
        .await
        .with_context(|| format!("Failed to load spec file: {}", spec_file.display()))?;

    // Override server command if provided
    if let Some(cmd) = server_cmd {
        spec.server.command = cmd;
        spec.server.args = server_args;
    }

    if cli.dry_run {
        info!("Dry run mode - would test server: {}", spec.server.command);
        info!("Server args: {:?}", spec.server.args);
        info!("Transport: {:?}", cli.transport);
        return Ok(());
    }

    // Create and run test harness
    let mut harness = TestHarness::new(spec);

    info!("Starting MCP server tests...");
    let results = harness
        .run_all_tests()
        .await
        .context("Failed to execute tests")?;

    // Generate reports
    generate_reports(&results, &cli.output, &cli.format)
        .await
        .context("Failed to generate reports")?;

    // Print summary
    print_test_summary(&results);

    // Exit with appropriate code
    if results.all_tests_passed() {
        info!("✅ All tests passed!");
        Ok(())
    } else {
        error!("❌ Some tests failed!");
        std::process::exit(1);
    }
}

/// Discover capabilities of an MCP server
async fn discover_server(server_cmd: String, args: Vec<String>) -> Result<()> {
    info!("Discovering server capabilities: {} {:?}", server_cmd, args);
    
    // FUTURE: Implement server discovery by connecting to server and querying capabilities
    //         Will include JSON-RPC initialization, capability enumeration, and report generation
    //         Essential for generating initial server specifications automatically
    info!("Discovery functionality will query server capabilities and generate spec template");
    
    Ok(())
}

/// Validate a server specification file
async fn validate_spec(spec_file: PathBuf) -> Result<()> {
    info!("Validating specification: {}", spec_file.display());

    let loader = SpecLoader::new().context("Failed to create spec loader")?;
    match loader.load_spec(&spec_file).await {
        Ok(spec) => {
            info!("✅ Specification is valid");
            info!("Server: {} v{}", spec.name, spec.version);
            info!(
                "Capabilities: tools={}, resources={}, prompts={}",
                spec.capabilities.tools, spec.capabilities.resources, spec.capabilities.prompts
            );
            Ok(())
        }
        Err(e) => {
            error!("❌ Specification validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Test only MCP protocol compliance without server-specific tests
async fn test_protocol_only(server_cmd: String, server_args: Vec<String>, cli: &Cli) -> Result<()> {
    info!("Testing MCP protocol compliance for: {}", server_cmd);
    info!("Server args: {:?}", server_args);

    if cli.dry_run {
        info!("Dry run mode - would test protocol compliance");
        return Ok(());
    }

    // Create minimal spec for protocol testing only
    let spec = ServerSpec::minimal_protocol_spec(server_cmd, server_args);
    let mut harness = TestHarness::new(spec);

    info!("Running protocol compliance tests...");
    let results = harness
        .run_protocol_tests_only()
        .await
        .context("Failed to execute protocol tests")?;

    // Generate reports
    generate_reports(&results, &cli.output, &cli.format)
        .await
        .context("Failed to generate reports")?;

    // Print summary
    print_test_summary(&results);

    if results.all_tests_passed() {
        info!("✅ Protocol compliance tests passed!");
        Ok(())
    } else {
        error!("❌ Protocol compliance tests failed!");
        std::process::exit(1);
    }
}

/// Generate example server specifications
async fn generate_examples(output_dir: PathBuf) -> Result<()> {
    info!("Generating example specifications in: {}", output_dir.display());
    
    // FUTURE: Generate example specs for common server types (CodePrism, custom tools, etc.)
    //         Will include templates for different MCP server architectures
    //         Helps users get started quickly with their own specifications
    info!("Example generation will create template specifications for common use cases");
    
    Ok(())
}

/// Look for and run default specification files
async fn find_and_run_default_spec(cli: &Cli) -> Result<()> {
    let default_specs = [
        "mcp-server.yaml",
        "server-spec.yaml",
        "test-spec.yaml",
        "mcp-test.yaml",
    ];

    for spec_name in &default_specs {
        let spec_path = PathBuf::from(spec_name);
        if spec_path.exists() {
            info!("Found default specification: {}", spec_name);
            return run_spec_test(
                spec_path,
                cli.server_cmd.clone(),
                cli.server_args.clone(),
                cli,
            )
            .await;
        }
    }

    error!("No specification file found and no server command provided.");
    error!("Please provide either:");
    error!("  - A specification file with --spec <file>");
    error!("  - A server command with --server-cmd <command>");
    error!("  - Use 'mcp-test-harness-lib examples' to generate example specs");

    std::process::exit(1);
}

/// Generate test reports in specified formats
async fn generate_reports(
    _results: &TestReport,
    output_dir: &PathBuf,
    formats: &[ReportFormat],
) -> Result<()> {
    std::fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    for format in formats {
        let filename = match format {
            ReportFormat::Html => "report.html",
            ReportFormat::Json => "report.json",
            ReportFormat::Xml => "report.xml",
            ReportFormat::Junit => "junit.xml",
            ReportFormat::Markdown => "report.md",
        };

        let output_path = output_dir.join(filename);
        info!(
            "Generating {} report: {}",
            format.to_string().to_uppercase(),
            output_path.display()
        );

        // TODO: Implement report generation
        // results.generate_report(format, &output_path).await?;
    }

    Ok(())
}

/// Print test execution summary
fn print_test_summary(results: &TestReport) {
    info!("=== Test Execution Summary ===");
    info!("Total Tests: {}", results.stats.total_tests);
    info!("Passed: {}", results.stats.passed_tests);
    info!("Failed: {}", results.stats.failed_tests);
    info!("Skipped: {}", results.stats.skipped_tests);
    info!("Pass Rate: {:.1}%", results.stats.pass_rate());
    info!("Duration: {}ms", results.stats.total_duration_ms);

    if results.stats.failed_tests > 0 {
        warn!("Failed tests:");
        // TODO: List failed test names
    }
}
