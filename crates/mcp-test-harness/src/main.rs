//! Generic MCP Test Harness CLI
//!
//! Command-line interface for testing any MCP (Model Context Protocol) server
//! implementation. This tool validates protocol compliance, tests capabilities,
//! and generates comprehensive reports.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mcp_test_harness_lib::{
    init,
    protocol::{ClientInfo, JsonRpcRequest, McpClient},
    version, ReportFormat, SpecLoader, TestHarness, TestReport, TransportType,
};
use serde_json::json;
use std::path::PathBuf;
use tracing::{error, info, warn};

// Use the specific types from schema module
use mcp_test_harness_lib::spec::schema::{
    ExpectedOutput, PromptSpec, ResourceSpec, RetryConfig, ServerCapabilities, ServerConfig,
    ServerSpec, TestCase, TestConfig, ToolSpec,
};

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

    /// Ecosystem integration features
    Ecosystem {
        #[command(subcommand)]
        ecosystem_command: EcosystemCommands,
    },
}

#[derive(Subcommand)]
enum EcosystemCommands {
    /// List popular MCP servers
    ListServers {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Show top N servers by popularity
        #[arg(long, default_value = "10")]
        top: usize,
    },

    /// Generate template for popular server
    Template {
        /// Server name (e.g., codeprism, filesystem, sqlite)
        server: String,
        /// Output file for generated template
        #[arg(short, long, default_value = "server-template.yaml")]
        output: PathBuf,
    },

    /// List available test templates
    Templates,

    /// Generate test configuration from template
    Generate {
        /// Template type (basic, filesystem, database, api, development)
        template: String,
        /// Output file for generated spec
        #[arg(short, long, default_value = "generated-spec.yaml")]
        output: PathBuf,
        /// Template variables in key=value format
        #[arg(long)]
        var: Vec<String>,
    },

    /// Benchmark performance against baseline
    Benchmark {
        /// Server specification file
        spec_file: PathBuf,
        /// Baseline file for comparison
        #[arg(long)]
        baseline: Option<PathBuf>,
        /// Save results as new baseline
        #[arg(long)]
        save_baseline: bool,
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
            output: _,
        }) => discover_server(server_cmd.clone(), server_args.clone()).await,
        Some(Commands::Validate { spec_file }) => validate_spec(spec_file.clone()).await,
        Some(Commands::Protocol {
            server_cmd,
            server_args,
        }) => test_protocol_only(server_cmd.clone(), server_args.clone(), &cli).await,
        Some(Commands::Examples { output }) => generate_examples(output.clone()).await,
        Some(Commands::Ecosystem { ecosystem_command }) => {
            handle_ecosystem_command(ecosystem_command).await
        }
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

    // Create a minimal spec for discovery
    let _discovery_spec = ServerSpec::minimal_protocol_spec(server_cmd.clone(), args.clone());

    // Create MCP client for discovery
    let mut client = McpClient::new();

    // Connect to the server
    info!("Connecting to server for discovery...");
    client
        .connect_stdio(server_cmd.clone(), args.clone(), None)
        .await
        .context("Failed to connect to server for discovery")?;

    // Initialize MCP session to get server capabilities
    info!("Initializing MCP session...");
    let client_info = ClientInfo {
        name: "MCP Test Harness Discovery".to_string(),
        version: "1.0.0".to_string(),
    };

    let init_result = client
        .initialize(client_info)
        .await
        .context("Failed to initialize MCP session")?;

    info!("Server initialized successfully!");
    info!("Protocol version: {}", init_result.protocol_version);
    info!(
        "Server info: {} v{}",
        init_result.server_info.name, init_result.server_info.version
    );

    // Discover tools if supported
    let mut discovered_tools = Vec::new();
    if init_result.capabilities.tools.is_some() {
        info!("Discovering available tools...");
        match client
            .send_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(1)),
                method: "tools/list".to_string(),
                params: Some(json!({})),
            })
            .await
        {
            Ok(response) => {
                if let Some(result) = response.result {
                    if let Some(tools) = result.get("tools") {
                        if let Some(tools_array) = tools.as_array() {
                            for tool in tools_array {
                                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                                    let description = tool
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("No description");

                                    discovered_tools.push(ToolSpec {
                                        name: name.to_string(),
                                        description: Some(description.to_string()),
                                        input_schema: None,
                                        output_schema: None,
                                        tests: vec![TestCase {
                                            name: format!("{}_basic_test", name),
                                            description: Some(format!(
                                                "Basic test for {} tool",
                                                name
                                            )),
                                            input: json!({}),
                                            expected: ExpectedOutput {
                                                error: false,
                                                error_code: None,
                                                error_message_contains: None,
                                                schema_file: None,
                                                schema: None,
                                                fields: vec![],
                                                allow_extra_fields: true,
                                            },
                                            performance: None,
                                            skip: false,
                                            tags: vec![
                                                "basic".to_string(),
                                                "auto-generated".to_string(),
                                            ],
                                        }],
                                    });
                                    info!("  - Tool: {} ({})", name, description);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Failed to list tools: {}", e),
        }
    }

    // Discover resources if supported
    let mut discovered_resources = Vec::new();
    if init_result.capabilities.resources.is_some() {
        info!("Discovering available resources...");
        match client
            .send_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(2)),
                method: "resources/list".to_string(),
                params: Some(json!({})),
            })
            .await
        {
            Ok(response) => {
                if let Some(result) = response.result {
                    if let Some(resources) = result.get("resources") {
                        if let Some(resources_array) = resources.as_array() {
                            for resource in resources_array {
                                if let Some(uri) = resource.get("uri").and_then(|u| u.as_str()) {
                                    let name = resource
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("Unknown Resource");
                                    let mime_type =
                                        resource.get("mimeType").and_then(|m| m.as_str());

                                    discovered_resources.push(ResourceSpec {
                                        uri_template: uri.to_string(),
                                        name: name.to_string(),
                                        mime_type: mime_type.map(|s| s.to_string()),
                                        tests: vec![TestCase {
                                            name: format!(
                                                "{}_basic_test",
                                                name.replace(' ', "_").to_lowercase()
                                            ),
                                            description: Some(format!(
                                                "Basic test for {} resource",
                                                name
                                            )),
                                            input: json!({"uri": uri}),
                                            expected: ExpectedOutput {
                                                error: false,
                                                error_code: None,
                                                error_message_contains: None,
                                                schema_file: None,
                                                schema: None,
                                                fields: vec![],
                                                allow_extra_fields: true,
                                            },
                                            performance: None,
                                            skip: false,
                                            tags: vec![
                                                "basic".to_string(),
                                                "auto-generated".to_string(),
                                            ],
                                        }],
                                    });
                                    info!("  - Resource: {} ({})", name, uri);
                                    if let Some(mime) = mime_type {
                                        info!("    MIME type: {}", mime);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Failed to list resources: {}", e),
        }
    }

    // Discover prompts if supported
    let mut discovered_prompts = Vec::new();
    if init_result.capabilities.prompts.is_some() {
        info!("Discovering available prompts...");
        match client
            .send_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(3)),
                method: "prompts/list".to_string(),
                params: Some(json!({})),
            })
            .await
        {
            Ok(response) => {
                if let Some(result) = response.result {
                    if let Some(prompts) = result.get("prompts") {
                        if let Some(prompts_array) = prompts.as_array() {
                            for prompt in prompts_array {
                                if let Some(name) = prompt.get("name").and_then(|n| n.as_str()) {
                                    let description = prompt
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("No description");

                                    discovered_prompts.push(PromptSpec {
                                        name: name.to_string(),
                                        description: Some(description.to_string()),
                                        arguments: vec![], // FUTURE: Parse arguments from prompt definition (tracked in #124)
                                        tests: vec![TestCase {
                                            name: format!("{}_basic_test", name),
                                            description: Some(format!(
                                                "Basic test for {} prompt",
                                                name
                                            )),
                                            input: json!({"name": name}),
                                            expected: ExpectedOutput {
                                                error: false,
                                                error_code: None,
                                                error_message_contains: None,
                                                schema_file: None,
                                                schema: None,
                                                fields: vec![],
                                                allow_extra_fields: true,
                                            },
                                            performance: None,
                                            skip: false,
                                            tags: vec![
                                                "basic".to_string(),
                                                "auto-generated".to_string(),
                                            ],
                                        }],
                                    });
                                    info!("  - Prompt: {} ({})", name, description);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Failed to list prompts: {}", e),
        }
    }

    // Disconnect from server
    client
        .disconnect()
        .await
        .context("Failed to disconnect from server")?;

    // Generate complete server specification
    let discovered_spec = ServerSpec {
        name: init_result.server_info.name,
        version: init_result.server_info.version,
        description: Some("Auto-discovered server specification".to_string()),
        capabilities: ServerCapabilities {
            tools: !discovered_tools.is_empty(),
            resources: !discovered_resources.is_empty(),
            prompts: !discovered_prompts.is_empty(),
            sampling: init_result.capabilities.resources.is_some(), // Basic detection
            logging: false, // Cannot auto-detect this easily
            experimental: None,
        },
        server: ServerConfig {
            command: server_cmd,
            args,
            env: std::collections::HashMap::new(),
            working_dir: None,
            transport: "stdio".to_string(),
            startup_timeout_seconds: 30,
            shutdown_timeout_seconds: 10,
        },
        tools: if discovered_tools.is_empty() {
            None
        } else {
            Some(discovered_tools)
        },
        resources: if discovered_resources.is_empty() {
            None
        } else {
            Some(discovered_resources)
        },
        prompts: if discovered_prompts.is_empty() {
            None
        } else {
            Some(discovered_prompts)
        },
        test_config: Some(TestConfig {
            timeout_seconds: 30,
            max_concurrency: 4,
            fail_fast: false,
            retry: Some(RetryConfig {
                max_retries: 2,
                retry_delay_ms: 1000,
                exponential_backoff: true,
            }),
        }),
        metadata: Some({
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "discovered_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            );
            metadata.insert(
                "protocol_version".to_string(),
                json!(init_result.protocol_version),
            );
            metadata.insert("discovery_tool".to_string(), json!("mcp-test-harness"));
            metadata
        }),
    };

    // Generate YAML output
    let output_file = "discovered-server.yaml";
    let yaml_content = serde_yaml::to_string(&discovered_spec)
        .context("Failed to serialize discovered spec to YAML")?;

    tokio::fs::write(output_file, yaml_content)
        .await
        .context("Failed to write discovered spec to file")?;

    info!("✅ Server discovery completed!");
    info!("Generated specification: {}", output_file);
    info!("Summary:");
    info!(
        "  - Tools: {}",
        discovered_spec.tools.as_ref().map(|t| t.len()).unwrap_or(0)
    );
    info!(
        "  - Resources: {}",
        discovered_spec
            .resources
            .as_ref()
            .map(|r| r.len())
            .unwrap_or(0)
    );
    info!(
        "  - Prompts: {}",
        discovered_spec
            .prompts
            .as_ref()
            .map(|p| p.len())
            .unwrap_or(0)
    );

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
    info!(
        "Generating example specifications in: {}",
        output_dir.display()
    );

    // Create output directory
    tokio::fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create output directory")?;

    // Generate different types of example specifications

    // 1. Simple Tool Server Example
    let simple_tool_spec = ServerSpec {
        name: "Simple Tool Server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Example server with basic tools".to_string()),
        capabilities: ServerCapabilities {
            tools: true,
            resources: false,
            prompts: false,
            sampling: false,
            logging: false,
            experimental: None,
        },
        server: ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: std::collections::HashMap::new(),
            working_dir: None,
            transport: "stdio".to_string(),
            startup_timeout_seconds: 30,
            shutdown_timeout_seconds: 10,
        },
        tools: Some(vec![ToolSpec {
            name: "echo".to_string(),
            description: Some("Echo the input message back".to_string()),
            input_schema: None,
            output_schema: None,
            tests: vec![TestCase {
                name: "echo_basic_test".to_string(),
                description: Some("Test basic echo functionality".to_string()),
                input: json!({"message": "Hello, World!"}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["basic".to_string(), "example".to_string()],
            }],
        }]),
        resources: None,
        prompts: None,
        test_config: Some(TestConfig {
            timeout_seconds: 30,
            max_concurrency: 4,
            fail_fast: false,
            retry: Some(RetryConfig {
                max_retries: 2,
                retry_delay_ms: 1000,
                exponential_backoff: true,
            }),
        }),
        metadata: None,
    };

    // 2. Database Server Example
    let database_spec = ServerSpec {
        name: "Database MCP Server".to_string(),
        version: "1.2.0".to_string(),
        description: Some("MCP server for database operations".to_string()),
        capabilities: ServerCapabilities {
            tools: true,
            resources: true,
            prompts: false,
            sampling: false,
            logging: true,
            experimental: None,
        },
        server: ServerConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "database_mcp_server".to_string()],
            env: {
                let mut env = std::collections::HashMap::new();
                env.insert("DATABASE_URL".to_string(), "sqlite:///app.db".to_string());
                env
            },
            working_dir: Some("./database-server".to_string()),
            transport: "stdio".to_string(),
            startup_timeout_seconds: 45,
            shutdown_timeout_seconds: 15,
        },
        tools: Some(vec![
            ToolSpec {
                name: "execute_query".to_string(),
                description: Some("Execute SQL query on the database".to_string()),
                input_schema: None,
                output_schema: None,
                tests: vec![TestCase {
                    name: "select_query_test".to_string(),
                    description: Some("Test SELECT query execution".to_string()),
                    input: json!({"query": "SELECT COUNT(*) FROM users"}),
                    expected: ExpectedOutput {
                        error: false,
                        error_code: None,
                        error_message_contains: None,
                        schema_file: None,
                        schema: None,
                        fields: vec![],
                        allow_extra_fields: true,
                    },
                    performance: None,
                    skip: false,
                    tags: vec!["database".to_string(), "sql".to_string()],
                }],
            },
            ToolSpec {
                name: "get_schema".to_string(),
                description: Some("Get database schema information".to_string()),
                input_schema: None,
                output_schema: None,
                tests: vec![TestCase {
                    name: "schema_info_test".to_string(),
                    description: Some("Test schema information retrieval".to_string()),
                    input: json!({}),
                    expected: ExpectedOutput {
                        error: false,
                        error_code: None,
                        error_message_contains: None,
                        schema_file: None,
                        schema: None,
                        fields: vec![],
                        allow_extra_fields: true,
                    },
                    performance: None,
                    skip: false,
                    tags: vec!["database".to_string(), "schema".to_string()],
                }],
            },
        ]),
        resources: Some(vec![ResourceSpec {
            uri_template: "database://tables/{table_name}".to_string(),
            name: "Database Table".to_string(),
            mime_type: Some("application/json".to_string()),
            tests: vec![TestCase {
                name: "table_resource_test".to_string(),
                description: Some("Test table resource access".to_string()),
                input: json!({"uri": "database://tables/users"}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["database".to_string(), "resource".to_string()],
            }],
        }]),
        prompts: None,
        test_config: Some(TestConfig {
            timeout_seconds: 60,
            max_concurrency: 2,
            fail_fast: false,
            retry: Some(RetryConfig {
                max_retries: 3,
                retry_delay_ms: 2000,
                exponential_backoff: true,
            }),
        }),
        metadata: Some({
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "security_features".to_string(),
                json!(["parameterized_queries", "connection_pooling"]),
            );
            metadata.insert(
                "supported_databases".to_string(),
                json!(["sqlite", "postgresql", "mysql"]),
            );
            metadata
        }),
    };

    // 3. File System Server Example
    let filesystem_spec = ServerSpec {
        name: "File System MCP Server".to_string(),
        version: "2.0.0".to_string(),
        description: Some("MCP server for file system operations".to_string()),
        capabilities: ServerCapabilities {
            tools: true,
            resources: true,
            prompts: false,
            sampling: false,
            logging: true,
            experimental: None,
        },
        server: ServerConfig {
            command: "npx".to_string(),
            args: vec![
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/allowed/path".to_string(),
            ],
            env: std::collections::HashMap::new(),
            working_dir: None,
            transport: "stdio".to_string(),
            startup_timeout_seconds: 30,
            shutdown_timeout_seconds: 10,
        },
        tools: Some(vec![ToolSpec {
            name: "read_file".to_string(),
            description: Some("Read contents of a file".to_string()),
            input_schema: None,
            output_schema: None,
            tests: vec![TestCase {
                name: "read_text_file".to_string(),
                description: Some("Test reading a text file".to_string()),
                input: json!({"path": "/allowed/path/test.txt"}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["filesystem".to_string(), "read".to_string()],
            }],
        }]),
        resources: Some(vec![ResourceSpec {
            uri_template: "file://{path}".to_string(),
            name: "File Resource".to_string(),
            mime_type: Some("text/plain".to_string()),
            tests: vec![TestCase {
                name: "file_resource_access".to_string(),
                description: Some("Test file resource access".to_string()),
                input: json!({"uri": "file:///allowed/path/example.txt"}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["filesystem".to_string(), "resource".to_string()],
            }],
        }]),
        prompts: None,
        test_config: Some(TestConfig {
            timeout_seconds: 30,
            max_concurrency: 4,
            fail_fast: false,
            retry: Some(RetryConfig {
                max_retries: 2,
                retry_delay_ms: 1000,
                exponential_backoff: true,
            }),
        }),
        metadata: Some({
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "security_features".to_string(),
                json!(["path_restriction", "file_type_validation"]),
            );
            metadata.insert(
                "supported_operations".to_string(),
                json!(["read", "write", "list", "search"]),
            );
            metadata
        }),
    };

    // 4. API Wrapper Server Example
    let api_wrapper_spec = ServerSpec {
        name: "API Wrapper Server".to_string(),
        version: "1.5.0".to_string(),
        description: Some("MCP server that wraps external APIs".to_string()),
        capabilities: ServerCapabilities {
            tools: true,
            resources: false,
            prompts: true,
            sampling: false,
            logging: true,
            experimental: None,
        },
        server: ServerConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "api_wrapper_server".to_string()],
            env: {
                let mut env = std::collections::HashMap::new();
                env.insert("API_KEY".to_string(), "${API_KEY}".to_string());
                env.insert(
                    "API_BASE_URL".to_string(),
                    "https://api.example.com".to_string(),
                );
                env
            },
            working_dir: None,
            transport: "stdio".to_string(),
            startup_timeout_seconds: 30,
            shutdown_timeout_seconds: 10,
        },
        tools: Some(vec![ToolSpec {
            name: "get_weather".to_string(),
            description: Some("Get weather information for a location".to_string()),
            input_schema: None,
            output_schema: None,
            tests: vec![TestCase {
                name: "weather_query_test".to_string(),
                description: Some("Test weather information retrieval".to_string()),
                input: json!({"location": "San Francisco, CA"}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["api".to_string(), "weather".to_string()],
            }],
        }]),
        resources: None,
        prompts: Some(vec![PromptSpec {
            name: "weather_report".to_string(),
            description: Some("Generate a weather report for a location".to_string()),
            arguments: vec![],
            tests: vec![TestCase {
                name: "weather_report_test".to_string(),
                description: Some("Test weather report generation".to_string()),
                input: json!({"name": "weather_report", "arguments": {"location": "New York"}}),
                expected: ExpectedOutput {
                    error: false,
                    error_code: None,
                    error_message_contains: None,
                    schema_file: None,
                    schema: None,
                    fields: vec![],
                    allow_extra_fields: true,
                },
                performance: None,
                skip: false,
                tags: vec!["api".to_string(), "prompt".to_string()],
            }],
        }]),
        test_config: Some(TestConfig {
            timeout_seconds: 45,
            max_concurrency: 3,
            fail_fast: false,
            retry: Some(RetryConfig {
                max_retries: 3,
                retry_delay_ms: 1500,
                exponential_backoff: true,
            }),
        }),
        metadata: Some({
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("api_version".to_string(), json!("v2"));
            metadata.insert(
                "rate_limits".to_string(),
                json!({"requests_per_minute": 100}),
            );
            metadata
        }),
    };

    // Write example specifications to files
    let examples = vec![
        ("simple-server.yaml", &simple_tool_spec),
        ("database-server.yaml", &database_spec),
        ("filesystem-server.yaml", &filesystem_spec),
        ("api-wrapper-server.yaml", &api_wrapper_spec),
    ];

    let examples_count = examples.len();
    for (filename, spec) in examples {
        let file_path = output_dir.join(filename);
        let yaml_content = serde_yaml::to_string(spec)
            .with_context(|| format!("Failed to serialize {} to YAML", filename))?;

        tokio::fs::write(&file_path, yaml_content)
            .await
            .with_context(|| format!("Failed to write {}", file_path.display()))?;

        info!("✅ Generated: {}", file_path.display());
    }

    // Generate README file
    let readme_content = r#"# MCP Server Test Specifications

This directory contains example MCP server test specifications that demonstrate
different types of MCP server implementations and testing approaches.

## Example Files

### simple-server.yaml
Basic example showing a simple MCP server with tool capabilities.
- **Use case**: Getting started with MCP testing
- **Features**: Basic tool testing, simple configuration

### database-server.yaml  
Example for database-backed MCP servers.
- **Use case**: Testing database integration servers
- **Features**: SQL tools, database resources, environment configuration

### filesystem-server.yaml
Example for file system operation servers.
- **Use case**: Testing file access and manipulation servers
- **Features**: File tools, file resources, security constraints

### api-wrapper-server.yaml
Example for servers that wrap external APIs.
- **Use case**: Testing API integration servers
- **Features**: API tools, prompts, environment variables

## Usage

1. Copy an example that matches your server type
2. Modify the `server` section with your actual command and arguments
3. Update tool names and test cases to match your server's capabilities
4. Run tests with: `mcp-test-harness test <spec-file>`

## Customization

- **Tools**: Add/modify tools in the `tools` section
- **Resources**: Define resources your server provides
- **Prompts**: Add prompt templates if supported
- **Test Cases**: Create comprehensive test scenarios
- **Performance**: Set performance requirements and thresholds

For more information, see the MCP Test Harness documentation.
"#;

    let readme_path = output_dir.join("README.md");
    tokio::fs::write(&readme_path, readme_content)
        .await
        .with_context(|| format!("Failed to write {}", readme_path.display()))?;

    info!("✅ Generated: {}", readme_path.display());

    info!("🎉 Example generation completed!");
    info!(
        "Generated {} example specifications in {}",
        examples_count,
        output_dir.display()
    );
    info!("Use these examples as starting points for your own MCP server tests.");

    Ok(())
}

/// Handle ecosystem integration commands
async fn handle_ecosystem_command(command: &EcosystemCommands) -> Result<()> {
    use mcp_test_harness_lib::ecosystem::{PopularServers, TemplateManager};

    match command {
        EcosystemCommands::ListServers { tag, top } => {
            let servers = PopularServers::new();

            let server_list = if let Some(tag_filter) = tag {
                servers.search_by_tag(tag_filter)
            } else {
                servers.top_servers(*top)
            };

            info!("🌟 Popular MCP Servers:");
            for (i, server) in server_list.iter().enumerate() {
                info!("{}. {} v{}", i + 1, server.name, server.version);
                info!("   {}", server.description);
                info!("   Author: {}", server.author);
                info!("   Tags: {}", server.tags.join(", "));
                info!("   Popularity: {:.1}/10", server.popularity_score);
                if let Some(repo) = &server.repository_url {
                    info!("   Repository: {}", repo);
                }
                info!("");
            }

            info!("Use 'mcp-test-harness ecosystem template <server>' to generate a template.");
        }

        EcosystemCommands::Template { server, output } => {
            let servers = PopularServers::new();

            match servers.generate_template(server) {
                Ok(template) => {
                    tokio::fs::write(output, &template.template_content)
                        .await
                        .context("Failed to write template file")?;

                    info!("✅ Generated template for {} server", server);
                    info!("Template saved to: {}", output.display());
                    info!("Required variables: {:?}", template.required_variables);
                }
                Err(e) => {
                    error!("❌ Failed to generate template for '{}': {}", server, e);
                    std::process::exit(1);
                }
            }
        }

        EcosystemCommands::Templates => {
            let manager = TemplateManager::new();
            let templates = manager.available_templates();

            info!("📋 Available Test Templates:");
            for template_type in templates {
                if let Some(template) = manager.get_template(template_type) {
                    info!("• {}: {}", template.name, template.description);
                }
            }

            info!("");
            info!("Use 'mcp-test-harness ecosystem generate <template>' to create a spec from template.");
        }

        EcosystemCommands::Generate {
            template,
            output,
            var,
        } => {
            let manager = TemplateManager::new();

            // Parse variables
            let mut variables = std::collections::HashMap::new();
            for var_pair in var {
                let parts: Vec<&str> = var_pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    variables.insert(parts[0].to_string(), parts[1].to_string());
                } else {
                    warn!("Invalid variable format: {}. Use key=value", var_pair);
                }
            }

            // Determine template type
            let template_type = match template.as_str() {
                "basic" => mcp_test_harness_lib::ecosystem::TemplateType::BasicCompliance,
                "filesystem" => mcp_test_harness_lib::ecosystem::TemplateType::FileSystem,
                "database" => mcp_test_harness_lib::ecosystem::TemplateType::Database,
                "api" => mcp_test_harness_lib::ecosystem::TemplateType::ApiWrapper,
                "development" => mcp_test_harness_lib::ecosystem::TemplateType::DevelopmentTool,
                _ => {
                    error!("❌ Unknown template type: {}", template);
                    error!("Available types: basic, filesystem, database, api, development");
                    std::process::exit(1);
                }
            };

            match manager.generate_spec(&template_type, &variables) {
                Ok(spec) => {
                    let yaml_content =
                        serde_yaml::to_string(&spec).context("Failed to serialize spec to YAML")?;

                    tokio::fs::write(output, yaml_content)
                        .await
                        .context("Failed to write generated spec")?;

                    info!("✅ Generated {} template specification", template);
                    info!("Specification saved to: {}", output.display());
                }
                Err(e) => {
                    error!("❌ Failed to generate specification: {}", e);
                    std::process::exit(1);
                }
            }
        }

        EcosystemCommands::Benchmark {
            spec_file,
            baseline,
            save_baseline,
        } => {
            info!("🚀 Running performance benchmarks...");

            // Load specification
            let loader = SpecLoader::new().context("Failed to create spec loader")?;
            let spec = loader
                .load_spec(spec_file)
                .await
                .with_context(|| format!("Failed to load spec file: {}", spec_file.display()))?;

            // Create and run test harness with performance focus
            let mut harness = TestHarness::new(spec);
            let results = harness
                .run_all_tests()
                .await
                .context("Failed to execute benchmark tests")?;

            // FUTURE: Implement detailed benchmark analysis and comparison
            info!("📊 Benchmark Results:");
            info!("Total tests: {}", results.stats.total_tests);
            info!("Passed: {}", results.stats.passed_tests);
            info!("Failed: {}", results.stats.failed_tests);

            if *save_baseline {
                // FUTURE: Save results as baseline
                info!("💾 Saving baseline for future comparisons...");
                info!("Baseline functionality will be implemented in a future update.");
            }

            if let Some(_baseline_file) = baseline {
                // FUTURE: Compare against baseline
                info!("📈 Comparing against baseline...");
                info!("Baseline comparison functionality will be implemented in a future update.");
            }
        }
    }

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

        // FUTURE: Implement report generation (tracked in #124)
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
        // FUTURE: List failed test names (tracked in #124)
    }
}
