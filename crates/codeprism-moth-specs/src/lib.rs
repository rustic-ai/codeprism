//! CodePrism Moth Test Specifications
//!
//! This crate contains comprehensive test specifications for the CodePrism MCP Server
//! using the Mandrel (Moth) test harness format.
//!
//! ## Test Categories
//!
//! - **Comprehensive**: Language-specific comprehensive test suites
//! - **Tools**: Tool category-focused test suites  
//! - **Workflows**: End-to-end workflow testing
//!
//! ## Usage
//!
//! These specifications are designed to be used with the Mandrel MCP test harness:
//!
//! ```bash
//! moth test crates/codeprism-moth-specs/codeprism/comprehensive/codeprism-python-comprehensive.yaml
//! ```

#[cfg(feature = "validation")]
pub mod validation {
    use std::path::Path;

    /// Validate that a YAML specification file is syntactically correct
    pub fn validate_specification(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let _parsed: serde_yaml::Value = serde_yaml::from_str(&content)?;
        Ok(())
    }
}

/// Metadata about the available test specifications
pub mod specs {
    /// Available comprehensive language test suites
    pub const COMPREHENSIVE_SPECS: &[&str] = &[
        "codeprism/comprehensive/codeprism-java-comprehensive.yaml",
        "codeprism/comprehensive/codeprism-javascript-comprehensive.yaml",
        "codeprism/comprehensive/codeprism-python-comprehensive.yaml",
        "codeprism/comprehensive/codeprism-rust-comprehensive.yaml",
    ];

    /// Available tool category test suites
    pub const TOOL_SPECS: &[&str] = &[
        "codeprism/tools/codeprism-complexity-analysis.yaml",
        "codeprism/tools/codeprism-core-navigation.yaml",
        "codeprism/tools/codeprism-flow-analysis.yaml",
        "codeprism/tools/codeprism-javascript-analysis.yaml",
        "codeprism/tools/codeprism-search-discovery.yaml",
        "codeprism/tools/codeprism-specialized-analysis.yaml",
    ];

    /// Available workflow test suites
    pub const WORKFLOW_SPECS: &[&str] =
        &["codeprism/workflows/codeprism-workflow-orchestration.yaml"];

    /// All available specifications
    pub fn all_specs() -> Vec<&'static str> {
        let mut specs = Vec::new();
        specs.extend_from_slice(COMPREHENSIVE_SPECS);
        specs.extend_from_slice(TOOL_SPECS);
        specs.extend_from_slice(WORKFLOW_SPECS);
        specs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_spec_path(spec: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(spec);
        path
    }

    #[test]
    fn test_all_comprehensive_specs_exist() {
        for spec in specs::COMPREHENSIVE_SPECS {
            let path = get_spec_path(spec);
            assert!(
                path.exists(),
                "Specification file not found: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_all_tool_specs_exist() {
        for spec in specs::TOOL_SPECS {
            let path = get_spec_path(spec);
            assert!(
                path.exists(),
                "Specification file not found: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_all_workflow_specs_exist() {
        for spec in specs::WORKFLOW_SPECS {
            let path = get_spec_path(spec);
            assert!(
                path.exists(),
                "Specification file not found: {}",
                path.display()
            );
        }
    }

    #[cfg(feature = "validation")]
    #[test]
    fn test_all_specs_are_valid_yaml() {
        for spec in specs::all_specs() {
            let path = get_spec_path(spec);
            if path.exists() {
                validation::validate_specification(&path)
                    .unwrap_or_else(|e| panic!("Invalid YAML in {}: {}", path.display(), e));
            }
        }
    }
}
