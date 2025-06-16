//! GCore daemon service

use clap::Parser;

#[derive(Parser)]
#[command(name = "gcore-daemon")]
#[command(about = "GCore background daemon", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "gcore.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting GCore daemon with config: {}", args.config);

    // TODO: Implement daemon logic
    // - Load configuration
    // - Start file watcher
    // - Connect to Neo4j
    // - Start processing loop

    // Keep running
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down GCore daemon");

    Ok(())
}
