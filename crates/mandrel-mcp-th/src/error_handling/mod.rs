//! Comprehensive Error Handling and Logging System
//!
//! This module implements a robust error handling framework with structured logging
//! that provides clear diagnostics and debugging capabilities throughout the entire
//! test execution pipeline.

pub mod errors;
pub mod logging;
pub mod metrics;
pub mod recovery;

pub use errors::*;
pub use logging::*;
pub use metrics::*;
pub use recovery::*;
