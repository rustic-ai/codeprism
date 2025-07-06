//! Error types for CodePrism utilities

use thiserror::Error;

/// Result type alias for utility operations
pub type Result<T> = std::result::Result<T, Error>;

/// Lightweight error types for utility operations
#[derive(Error, Debug)]
pub enum Error {
    /// File watcher errors
    #[cfg(feature = "file-watcher")]
    #[error("File watcher error: {0}")]
    Watcher(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// General utility errors
    #[error("Utility error: {0}")]
    Utility(String),
}

impl Error {
    /// Create a file watcher error
    #[cfg(feature = "file-watcher")]
    pub fn watcher<S: Into<String>>(msg: S) -> Self {
        Self::Watcher(msg.into())
    }

    /// Create a general utility error
    pub fn utility<S: Into<String>>(msg: S) -> Self {
        Self::Utility(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::utility("test error");
        assert_eq!(err.to_string(), "Utility error: test error");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
    }

    #[cfg(feature = "file-watcher")]
    #[test]
    fn test_watcher_error() {
        let err = Error::watcher("test watcher error");
        assert!(matches!(err, Error::Watcher(_)));
        assert_eq!(err.to_string(), "File watcher error: test watcher error");
    }
}
