//! Error types for JavaScript/TypeScript parser

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for JavaScript parser operations
pub type Result<T> = std::result::Result<T, Error>;

/// JavaScript/TypeScript parser error type
#[derive(Debug, Error)]
pub enum Error {
    /// Tree-sitter parsing error
    #[error("Parse error in {file}: {message}")]
    Parse {
        /// File that failed to parse
        file: PathBuf,
        /// Error message
        message: String,
    },

    /// Tree-sitter language error
    #[error("Failed to set language: {0}")]
    Language(String),

    /// Node extraction error
    #[error("Failed to extract node at {file}:{line}:{column}: {message}")]
    NodeExtraction {
        /// File path
        file: PathBuf,
        /// Line number
        line: usize,
        /// Column number
        column: usize,
        /// Error message
        message: String,
    },

    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a parse error
    pub fn parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse {
            file: file.into(),
            message: message.into(),
        }
    }

    /// Create a node extraction error
    pub fn node_extraction(
        file: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::NodeExtraction {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a language error
    pub fn language(message: impl Into<String>) -> Self {
        Self::Language(message.into())
    }

    /// Create an other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}
