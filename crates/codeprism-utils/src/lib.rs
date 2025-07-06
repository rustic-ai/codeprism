//! CodePrism Utilities - Lightweight utility functions for the CodePrism ecosystem
//!
//! This crate provides essential utilities that can be shared across CodePrism components
//! without pulling in heavy dependencies. Currently includes:
//!
//! - **File Watching**: Real-time file system monitoring with debouncing
//! - **Error Handling**: Lightweight error types for utility operations
//!
//! ## Features
//!
//! - `file-watcher` (default): File system monitoring capabilities

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;

#[cfg(feature = "file-watcher")]
pub mod watcher;

// Re-export commonly used types
pub use error::{Error, Result};

#[cfg(feature = "file-watcher")]
pub use watcher::{ChangeEvent, ChangeKind, FileWatcher};
