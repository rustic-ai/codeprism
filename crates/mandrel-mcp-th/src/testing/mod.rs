//! Testing infrastructure for MOTH
//!
//! This module provides testing utilities for real MCP server integration testing,
//! test fixtures, and end-to-end testing framework.

pub mod fixtures;
pub mod integration_framework;

// Re-export main components
pub use fixtures::{TestFixtures, TestFixturesError};
pub use integration_framework::IntegrationTestFramework;
