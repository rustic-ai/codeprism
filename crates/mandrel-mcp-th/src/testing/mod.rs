//! Testing infrastructure for MOTH
//!
//! This module provides testing utilities for real MCP server integration testing, 
//! test fixtures, and end-to-end testing framework.

pub mod integration_framework;
pub mod fixtures;

// Re-export commonly used types
pub use integration_framework::{IntegrationTestFramework, IntegrationTestError, TestResult};
pub use fixtures::{TestFixtures, TestFixturesError, ErrorScenario}; 