//! MCP Ecosystem Integration Framework
//!
//! This module provides tools for integrating with the broader MCP ecosystem,
//! including pre-built configurations for popular servers, template systems,
//! performance benchmarking, and community features.

pub mod benchmarks;
pub mod community;
pub mod servers;
pub mod templates;

// Re-export main types for public API
pub use benchmarks::{BenchmarkResult, BenchmarkSuite, PerformanceBaseline};
pub use community::{CommunityConfig, ResultComparison, TestShare};
pub use servers::{PopularServers, ServerProfile, ServerTemplate};
pub use templates::{TemplateManager, TemplateType, TestTemplate};
