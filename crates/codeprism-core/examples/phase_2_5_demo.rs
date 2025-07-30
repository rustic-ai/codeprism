//! Phase 2.5 Implementation Demo
//!
//! This example demonstrates the newly implemented Phase 2.5 components:
//! - Repository Scanner: Discovers and filters source files
//! - Bulk Indexer: Processes files in parallel and builds graph patches
//! - Repository Manager: Orchestrates scanning and indexing operations
//! - File Monitoring Pipeline: Real-time incremental updates
//!
//! Usage: cargo run --example phase_2_5_demo

use codeprism_core::prelude::*;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 CodePrism Phase 2.5 Implementation Demo");
    println!("========================================\n");

    // Create a test repository with some source files
    let repo_dir = create_test_repository().await?;
    println!(
        "✅ Created test repository at: {}",
        repo_dir.path().display()
    );

    // Step 1: Demonstrate Repository Scanner
    println!("\n📂 Step 1: Repository Scanning");
    let scanner = RepositoryScanner::new();
    let progress_reporter = Arc::new(NoOpProgressReporter);

    let scan_result = scanner
        .scan_repository(repo_dir.path(), progress_reporter)
        .await?;

    println!(
        "   Discovered {} files in {}ms",
        scan_result.total_files, scan_result.duration_ms
    );

    for (language, files) in &scan_result.files_by_language {
        println!("   - {}: {} files", language, files.len());
        for file in files.iter().take(3) {
            println!(
                "     • {}",
                file.path.file_name().unwrap().to_string_lossy()
            );
        }
    }

    // Step 2: Demonstrate Bulk Indexer
    println!("\n⚡ Step 2: Bulk Indexing");
    let language_registry = Arc::new(LanguageRegistry::new());

    // For demo purposes, we'll work without language parsers registered
    // In real usage, you'd register parsers for JavaScript, Python, etc.
    let parser_engine = Arc::new(ParserEngine::new(language_registry.clone()));

    let indexing_config =
        IndexingConfig::new("demo_repo".to_string(), "demo_commit_123".to_string());

    let indexer = BulkIndexer::new(indexing_config, parser_engine.clone());
    let indexing_progress = Arc::new(IndexingProgressReporter::new(false));

    // Note: This will generate errors since we don't have language parsers registered,
    // but demonstrates the indexing pipeline working
    let indexing_result = indexer
        .index_scan_result(&scan_result, indexing_progress)
        .await?;

    println!(
        "   Processed {} files",
        indexing_result.stats.files_processed
    );
    println!("   Generated {} patches", indexing_result.patches.len());
    println!(
        "   Processing time: {}ms",
        indexing_result.stats.duration_ms
    );
    if indexing_result.stats.error_count > 0 {
        println!(
            "   ⚠️  {} errors (expected - no parsers registered)",
            indexing_result.stats.error_count
        );
    }

    // Step 3: Demonstrate Repository Manager
    println!("\n🏛️  Step 3: Repository Management");
    let mut repo_manager = RepositoryManager::new(language_registry);

    let repo_config = RepositoryConfig::new("demo_repo".to_string(), repo_dir.path())
        .with_name("Demo Repository".to_string())
        .with_description("A demonstration repository for Phase 2.5".to_string());

    repo_manager.register_repository(repo_config)?;
    println!("   ✅ Registered repository");

    let repos = repo_manager.list_repositories();
    println!("   📊 Repository count: {}", repos.len());

    for repo in repos {
        println!(
            "   - {}: {} ({:?})",
            repo.config.name, repo.config.repo_id, repo.health
        );
    }

    // Demonstrate health check
    let health = repo_manager.health_check("demo_repo").await?;
    println!("   🩺 Health status: {health:?}");

    // Step 4: Demonstrate File Monitoring Pipeline (setup only)
    println!("\n👁️  Step 4: File Monitoring Pipeline");
    let pipeline_config =
        PipelineConfig::new("demo_repo".to_string(), "monitoring_commit".to_string());

    let event_handler = Arc::new(LoggingEventHandler::new(false));

    let pipeline = MonitoringPipeline::new(
        pipeline_config,
        parser_engine,
        event_handler as Arc<dyn PipelineEventHandler>,
    )?;

    println!("   ✅ Created monitoring pipeline");
    println!(
        "   📈 Pipeline stats: {} events processed",
        pipeline.get_stats().events_processed
    );

    // Note: We don't actually start monitoring to avoid indefinite running
    println!("   💡 Pipeline ready for real-time file monitoring");

    // Step 5: Show overall statistics
    println!("\n📊 Step 5: Overall Statistics");
    let total_stats = repo_manager.get_total_stats();
    for (key, value) in &total_stats {
        println!("   - {key}: {value}");
    }

    println!("\n🎉 Phase 2.5 Demo Complete!");
    println!("\nImplemented Components:");
    println!("  ✅ Repository Scanner - File discovery and filtering");
    println!("  ✅ Bulk Indexer - Parallel processing and graph building");
    println!("  ✅ Repository Manager - High-level repository operations");
    println!("  ✅ File Monitoring Pipeline - Real-time incremental updates");
    println!("\nNext Phase: MCP Protocol Compliance and CLI Integration");

    Ok(())
}

/// Create a test repository with sample source files
async fn create_test_repository() -> Result<TempDir> {
    let temp_dir =
        TempDir::new().map_err(|e| Error::io(format!("Failed to create temp directory: {e}")))?;

    let repo_path = temp_dir.path();

    // Create various source files
    fs::write(
        repo_path.join("main.js"),
        r#"
function main() {
    console.log('Hello from JavaScript!');
    calculateSum(10, 20);
}

function calculateSum(a, b) {
    return a + b;
}

module.exports = { main, calculateSum };
"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    fs::write(
        repo_path.join("utils.py"),
        r#"
def helper_function():
    """A helpful utility function."""
    return "This is helpful!"

class DataProcessor:
    def __init__(self, data):
        self.data = data
    
    def process(self):
        return [x * 2 for x in self.data]

if __name__ == "__main__":
    processor = DataProcessor([1, 2, 3, 4, 5])
    print(processor.process())
"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    fs::write(
        repo_path.join("config.ts"),
        r#"
interface AppConfig {
    name: string;
    version: string;
    features: string[];
}

export const config: AppConfig = {
    name: "Prism Demo",
    version: "1.0.0",
    features: ["scanning", "indexing", "monitoring"]
};

export function getFeatureCount(): number {
    return config.features.length;
}
"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    // Create subdirectory with more files
    fs::create_dir(repo_path.join("src"))
        .await
        .map_err(|e| Error::io(format!("Failed to create directory: {e}")))?;

    fs::write(
        repo_path.join("src/helper.js"),
        r#"
const { calculateSum } = require('../main');

function advancedCalculation(numbers) {
    return numbers.reduce((sum, num) => calculateSum(sum, num), 0);
}

module.exports = { advancedCalculation };
"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    fs::write(
        repo_path.join("src/data.py"),
        r#"
from utils import DataProcessor

def load_data():
    return [10, 20, 30, 40, 50]

def main():
    data = load_data()
    processor = DataProcessor(data)
    result = processor.process()
    print(f"Processed data: {result}")

if __name__ == "__main__":
    main()
"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    // Add some files that should be ignored
    fs::write(
        repo_path.join("README.md"),
        "# Demo Repository\nThis is a demonstration repository for CodePrism Phase 2.5.",
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    fs::write(
        repo_path.join("package.json"),
        r#"{"name": "demo", "version": "1.0.0"}"#,
    )
    .await
    .map_err(|e| Error::io(format!("Failed to write file: {e}")))?;

    Ok(temp_dir)
}
