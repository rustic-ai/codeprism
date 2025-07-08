//! Testing infrastructure for MOTH
//!
//! This module provides testing utilities for real MCP server integration testing,
//! test fixtures, and end-to-end testing framework.

pub mod fixtures;
pub mod integration_framework;

// Re-export commonly used types
pub use fixtures::{ErrorScenario, TestFixtures, TestFixturesError};
pub use integration_framework::{IntegrationTestError, IntegrationTestFramework, TestResult};
