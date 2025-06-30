//! Specification handling for MCP test harness
//!
//! This module provides functionality for loading, validating, and working with
//! MCP server specifications. Specifications define the server capabilities,
//! test cases, and expected behaviors in a data-driven format.

use anyhow::Result;
use std::path::Path;

pub mod loader;
pub mod schema;
pub mod validator;

// Re-export the main types from schema module
pub use loader::SpecLoader;
pub use schema::{ServerCapabilities, ServerConfig, ServerSpec, TestCase, ValidationError};
pub use validator::ResponseValidator;

/// Load a specification from a file
///
/// This is a convenience function that creates a new `SpecLoader` and loads
/// the specification from the given path.
///
/// # Arguments
/// * `path` - Path to the specification file
///
/// # Returns
/// A validated `ServerSpec` instance ready for use.
///
/// # Errors
/// Returns `ValidationError` if the file cannot be loaded or validated.
///
/// # Examples
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use mcp_test_harness_lib::spec::load_spec;
///
/// let spec = load_spec("examples/simple-server.yaml").await?;
/// println!("Loaded server: {} v{}", spec.name, spec.version);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub async fn load_spec<P: AsRef<Path>>(path: P) -> Result<ServerSpec, ValidationError> {
    let loader = SpecLoader::new()?;
    loader.load_spec(path).await
}
