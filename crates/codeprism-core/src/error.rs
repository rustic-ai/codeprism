//! Error types for codeprism

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for codeprism operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for codeprism
#[derive(Debug, Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parser error
    #[error("Parse error in {file}: {message}")]
    Parse {
        /// File that failed to parse
        file: PathBuf,
        /// Error message
        message: String,
    },

    /// Language not supported
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    /// Tree-sitter error
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// Graph storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// File watcher error
    #[error("File watcher error: {0}")]
    Watcher(String),

    /// Indexing error
    #[error("Indexing error: {0}")]
    Indexing(String),

    /// Invalid node ID
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),

    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Edge not found
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

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

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage(message.into())
    }

    /// Create a tree-sitter error
    pub fn tree_sitter(message: impl Into<String>) -> Self {
        Self::TreeSitter(message.into())
    }

    /// Create a watcher error
    pub fn watcher(message: impl Into<String>) -> Self {
        Self::Watcher(message.into())
    }

    /// Create an indexing error
    pub fn indexing(message: impl Into<String>) -> Self {
        Self::Indexing(message.into())
    }

    /// Create an IO error from a string message
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            message.into(),
        ))
    }

    /// Create an other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let err = Error::parse("test.js", "syntax error");
        match err {
            Error::Parse { file, message } => {
                assert_eq!(file, PathBuf::from("test.js"));
                assert_eq!(message, "syntax error");
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_parse_error_display() {
        let err = Error::parse("src/main.rs", "unexpected token");
        let display = format!("{}", err);
        assert!(display.contains("Parse error in src/main.rs"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_storage_error() {
        let err = Error::storage("connection failed");
        assert!(matches!(err, Error::Storage(_)));
        assert_eq!(format!("{}", err), "Storage error: connection failed");
    }

    #[test]
    fn test_tree_sitter_error() {
        let err = Error::tree_sitter("grammar not found");
        assert!(matches!(err, Error::TreeSitter(_)));
        assert_eq!(format!("{}", err), "Tree-sitter error: grammar not found");
    }

    #[test]
    fn test_watcher_error() {
        let err = Error::watcher("failed to watch directory");
        assert!(matches!(err, Error::Watcher(_)));
        assert_eq!(
            format!("{}", err),
            "File watcher error: failed to watch directory"
        );
    }

    #[test]
    fn test_indexing_error() {
        let err = Error::indexing("failed to index");
        assert!(matches!(err, Error::Indexing(_)));
        assert_eq!(format!("{}", err), "Indexing error: failed to index");
    }

    #[test]
    fn test_other_error() {
        let err = Error::other("generic error");
        assert!(matches!(err, Error::Other(_)));
        assert_eq!(format!("{}", err), "generic error");
    }

    #[test]
    fn test_unsupported_language_error() {
        let err = Error::UnsupportedLanguage("brainfuck".to_string());
        assert_eq!(format!("{}", err), "Language not supported: brainfuck");
    }

    #[test]
    fn test_config_error() {
        let err = Error::Config("invalid TOML".to_string());
        assert_eq!(format!("{}", err), "Configuration error: invalid TOML");
    }

    #[test]
    fn test_node_errors() {
        let node_err = Error::InvalidNodeId("malformed-id".to_string());
        assert_eq!(format!("{}", node_err), "Invalid node ID: malformed-id");

        let not_found_err = Error::NodeNotFound("node123".to_string());
        assert_eq!(format!("{}", not_found_err), "Node not found: node123");

        let edge_err = Error::EdgeNotFound("edge456".to_string());
        assert_eq!(format!("{}", edge_err), "Edge not found: edge456");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());
        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::Serialization(_)));
    }
}
