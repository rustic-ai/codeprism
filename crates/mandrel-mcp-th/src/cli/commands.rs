//! Command implementations for MOTH CLI

use crate::error::{Error, Result};
use std::path::PathBuf;
use tracing::{info, warn};

/// Handle the test command
pub async fn handle_test(
    spec: PathBuf,
    output_file: Option<PathBuf>,
    fail_fast: bool,
    filter: Option<String>,
    concurrency: usize,
) -> Result<()> {
    info!("Running tests from specification: {}", spec.display());
    info!("Concurrency: {}, Fail fast: {}", concurrency, fail_fast);

    if let Some(filter) = &filter {
        info!("Test filter: {}", filter);
    }

    if let Some(output_file) = &output_file {
        info!("Output file: {}", output_file.display());
    }

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // PLANNED(#189): Implement actual test execution with MCP client
    warn!("Test execution not yet implemented");

    Ok(())
}

/// Handle the validate command
pub async fn handle_validate(spec: PathBuf) -> Result<()> {
    info!("Validating specification: {}", spec.display());

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // PLANNED(#192): Implement actual specification validation with YAML parser
    warn!("Specification validation not yet implemented");

    Ok(())
}

/// Handle the list command
pub async fn handle_list(spec: PathBuf, detailed: bool) -> Result<()> {
    info!("Listing tests from specification: {}", spec.display());
    info!("Detailed output: {}", detailed);

    // Validate specification file exists
    if !spec.exists() {
        return Err(Error::spec(format!(
            "Specification file not found: {}",
            spec.display()
        )));
    }

    // PLANNED(#192): Implement actual test listing from YAML specifications
    warn!("Test listing not yet implemented");

    Ok(())
}

/// Handle the version command
pub fn handle_version() -> Result<()> {
    println!(
        "moth {} - Mandrel MCP Test Harness",
        env!("CARGO_PKG_VERSION")
    );
    println!("MOdel context protocol Test Harness binary");
    println!("Built with official rmcp SDK");
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_handle_test_missing_spec() {
        let result = handle_test(PathBuf::from("nonexistent.yaml"), None, false, None, 4).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_test_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, "# test spec").unwrap();

        let result = handle_test(spec_file, None, false, None, 4).await;

        // Should succeed but warn about not implemented
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_test_with_all_options() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        let output_file = temp_dir.path().join("output.json");
        fs::write(&spec_file, "# test spec").unwrap();

        let result = handle_test(
            spec_file,
            Some(output_file),
            true,
            Some("filesystem".to_string()),
            8,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_validate_missing_spec() {
        let result = handle_validate(PathBuf::from("nonexistent.yaml")).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_validate_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, "# test spec").unwrap();

        let result = handle_validate(spec_file).await;

        // Should succeed but warn about not implemented
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_list_missing_spec() {
        let result = handle_list(PathBuf::from("nonexistent.yaml"), false).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Spec(_)));
        assert!(err.to_string().contains("Specification file not found"));
    }

    #[tokio::test]
    async fn test_handle_list_existing_spec() {
        let temp_dir = tempdir().unwrap();
        let spec_file = temp_dir.path().join("test.yaml");
        fs::write(&spec_file, "# test spec").unwrap();

        let result = handle_list(spec_file, true).await;

        // Should succeed but warn about not implemented
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_version() {
        let result = handle_version();
        assert!(result.is_ok());
    }
}
