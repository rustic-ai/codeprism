//! Error types for Java parser

use std::path::Path;
use thiserror::Error;

/// Error type for Java parser
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to parse the file
    #[error("Parse error in {file}: {message}")]
    Parse { file: String, message: String },

    /// Tree-sitter error
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// Invalid Java syntax
    #[error("Invalid Java syntax in {file} at line {line}: {message}")]
    InvalidSyntax {
        file: String,
        line: usize,
        message: String,
    },

    /// Unsupported Java language feature
    #[error("Unsupported Java feature in {file}: {feature}")]
    UnsupportedFeature { file: String, feature: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// UTF-8 encoding error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl Error {
    /// Create a parse error
    pub fn parse(file: &Path, message: &str) -> Self {
        Self::Parse {
            file: file.display().to_string(),
            message: message.to_string(),
        }
    }

    /// Create an invalid syntax error
    pub fn invalid_syntax(file: &Path, line: usize, message: &str) -> Self {
        Self::InvalidSyntax {
            file: file.display().to_string(),
            line,
            message: message.to_string(),
        }
    }

    /// Create an unsupported feature error
    pub fn unsupported_feature(file: &Path, feature: &str) -> Self {
        Self::UnsupportedFeature {
            file: file.display().to_string(),
            feature: feature.to_string(),
        }
    }
}

/// Result type for Java parser
pub type Result<T> = std::result::Result<T, Error>;
