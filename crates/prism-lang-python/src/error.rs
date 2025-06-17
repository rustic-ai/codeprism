//! Error types for Python parser

use std::path::PathBuf;
use thiserror::Error;

/// Error type for Python parser
#[derive(Error, Debug)]
pub enum Error {
    /// Parse error
    #[error("Failed to parse {file}: {message}")]
    ParseError { file: PathBuf, message: String },
    
    /// Tree-sitter error
    #[error("Tree-sitter error in {file}: {message}")]
    TreeSitterError { file: PathBuf, message: String },
    
    /// AST mapping error
    #[error("AST mapping error in {file}: {message}")]
    AstMappingError { file: PathBuf, message: String },
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Generic error
    #[error("Python parser error: {0}")]
    Generic(String),
}

impl Error {
    /// Create a parse error
    pub fn parse(file: &std::path::Path, message: &str) -> Self {
        Self::ParseError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }
    
    /// Create a tree-sitter error
    pub fn tree_sitter(file: &std::path::Path, message: &str) -> Self {
        Self::TreeSitterError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }
    
    /// Create an AST mapping error
    pub fn ast_mapping(file: &std::path::Path, message: &str) -> Self {
        Self::AstMappingError {
            file: file.to_path_buf(),
            message: message.to_string(),
        }
    }
    
    /// Create a generic error
    pub fn generic(message: &str) -> Self {
        Self::Generic(message.to_string())
    }
}

/// Result type for Python parser
pub type Result<T> = std::result::Result<T, Error>; 