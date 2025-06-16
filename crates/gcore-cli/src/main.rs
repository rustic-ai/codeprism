//! GCore CLI tool

use clap::Parser;

#[derive(Parser)]
#[command(name = "gcore-cli")]
#[command(about = "GCore command-line interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Parse a repository
    Parse {
        /// Path to the repository
        path: String,
    },
    /// Trace code paths
    Trace {
        /// Starting point
        from: String,
        /// Target to trace to
        #[arg(long)]
        to: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { path } => {
            println!("Parsing repository at: {}", path);
            // TODO: Implement parsing
        }
        Commands::Trace { from, to } => {
            println!("Tracing from {} to {}", from, to);
            // TODO: Implement tracing
        }
    }
}
