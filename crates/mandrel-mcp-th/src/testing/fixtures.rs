//! Test Fixtures
//!
//! Provides test data and fixtures for integration testing.

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Test data management
pub struct TestFixtures {
    pub spec_files: HashMap<String, PathBuf>,
    pub expected_outputs: HashMap<String, Value>,
    pub error_scenarios: Vec<ErrorScenario>,
}

impl TestFixtures {
    pub fn load_all() -> Result<Self, TestFixturesError> {
        Ok(Self {
            spec_files: HashMap::new(),
            expected_outputs: HashMap::new(),
            error_scenarios: Vec::new(),
        })
    }

    pub fn get_spec(&self, name: &str) -> Option<&PathBuf> {
        self.spec_files.get(name)
    }

    pub fn get_expected_output(&self, spec_name: &str) -> Option<&Value> {
        self.expected_outputs.get(spec_name)
    }

    pub fn create_temp_spec(&self, _content: &str) -> Result<PathBuf, TestFixturesError> {
        Ok(PathBuf::from("temp_spec.yaml"))
    }
}

/// Error scenario testing
pub struct ErrorScenario {
    pub name: String,
    pub spec_content: String,
    pub expected_error: ExpectedError,
}

pub enum ExpectedError {
    InvalidYaml,
    ServerConnectionFailure,
    ToolExecutionTimeout,
    MalformedResponse,
    ValidationFailure,
}

/// Test fixtures errors
#[derive(Error, Debug)]
pub enum TestFixturesError {
    #[error("Load error: {0}")]
    Load(String),

    #[error("File error: {0}")]
    File(String),

    #[error("Parse error: {0}")]
    Parse(String),
}
