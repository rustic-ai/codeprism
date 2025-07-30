//! Custom error types demonstrating Rust error handling patterns.

use thiserror::Error;

/// Main application errors
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Authorization failed: insufficient permissions")]
    AuthorizationFailed,
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Resource conflict: {resource} already exists")]
    Conflict { resource: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Service unavailable")]
    ServiceUnavailable,
    
    #[error("Unknown error occurred")]
    Unknown,
}

/// Database-specific errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Deadlock detected")]
    Deadlock,
    
    #[error("Timeout occurred")]
    Timeout,
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field '{field}' is required")]
    Required { field: String },
    
    #[error("Field '{field}' has invalid format")]
    InvalidFormat { field: String },
    
    #[error("Field '{field}' is too short (minimum: {min})")]
    TooShort { field: String, min: usize },
    
    #[error("Field '{field}' is too long (maximum: {max})")]
    TooLong { field: String, max: usize },
    
    #[error("Field '{field}' is out of range ({min}-{max})")]
    OutOfRange { field: String, min: i64, max: i64 },
    
    #[error("User name cannot be empty")]
    EmptyName,
    
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    
    #[error("Invalid age: {0}")]
    InvalidAge(u32),
    
    #[error("Password too weak")]
    WeakPassword,
    
    #[error("Duplicate value for field '{field}'")]
    Duplicate { field: String },
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Request timeout")]
    Timeout,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("DNS resolution failed")]
    DnsResolution,
    
    #[error("SSL/TLS error: {0}")]
    Tls(String),
    
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, ApplicationError>;

/// Error conversion utilities
impl From<reqwest::Error> for NetworkError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            NetworkError::Timeout
        } else if err.is_connect() {
            NetworkError::ConnectionRefused
        } else {
            NetworkError::Http {
                status: err.status().map(|s| s.as_u16()).unwrap_or(0),
                message: err.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for NetworkError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_syntax() || err.is_data() {
            NetworkError::Deserialization(err.to_string())
        } else {
            NetworkError::Serialization(err.to_string())
        }
    }
}

/// Error context utilities
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> AppResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<ApplicationError>,
{
    fn with_context(self, context: &str) -> AppResult<T> {
        self.map_err(|e| {
            let base_error = e.into();
            ApplicationError::Configuration(format!("{}: {}", context, base_error))
        })
    }
}

/// Error reporting utilities
pub fn report_error(error: &ApplicationError) {
    match error {
        ApplicationError::Database(db_err) => {
            tracing::error!("Database error occurred: {}", db_err);
        }
        ApplicationError::Network(net_err) => {
            tracing::warn!("Network error occurred: {}", net_err);
        }
        ApplicationError::Validation(val_err) => {
            tracing::info!("Validation error: {}", val_err);
        }
        ApplicationError::AuthenticationFailed => {
            tracing::warn!("Authentication attempt failed");
        }
        ApplicationError::AuthorizationFailed => {
            tracing::warn!("Authorization failed - insufficient permissions");
        }
        ApplicationError::NotFound { resource } => {
            tracing::info!("Resource not found: {}", resource);
        }
        ApplicationError::Conflict { resource } => {
            tracing::warn!("Resource conflict: {}", resource);
        }
        ApplicationError::RateLimitExceeded => {
            tracing::warn!("Rate limit exceeded");
        }
        ApplicationError::ServiceUnavailable => {
            tracing::error!("Service unavailable");
        }
        _ => {
            tracing::error!("Application error: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let validation_error = ValidationError::EmptyName;
        let app_error = ApplicationError::Validation(validation_error);
        
        assert!(matches!(app_error, ApplicationError::Validation(_)));
    }

    #[test]
    fn test_error_chaining() {
        let db_error = DatabaseError::ConnectionFailed("Connection timeout".to_string());
        let app_error = ApplicationError::Database(db_error);
        
        let error_string = app_error.to_string();
        assert!(error_string.contains("Database error"));
        assert!(error_string.contains("Connection timeout"));
    }

    #[test]
    fn test_validation_errors() {
        let errors = vec![
            ValidationError::Required { field: "name".to_string() },
            ValidationError::InvalidFormat { field: "email".to_string() },
            ValidationError::TooShort { field: "password".to_string(), min: 8 },
            ValidationError::OutOfRange { field: "age".to_string(), min: 0, max: 120 },
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty(), "Error string should not be empty");
            
            // Verify error strings contain meaningful information
            match error {
                ValidationError::Required { field } => {
                    assert!(error_string.contains(&field), "Error should contain field name: {}", field);
                    assert!(error_string.contains("required"), "Error should indicate requirement");
                },
                ValidationError::InvalidFormat { field } => {
                    assert!(error_string.contains(&field), "Error should contain field name: {}", field);
                    assert!(error_string.contains("format"), "Error should mention format issue");
                },
                ValidationError::TooShort { field, min } => {
                    assert!(error_string.contains(&field), "Error should contain field name: {}", field);
                    assert!(error_string.contains(&min.to_string()), "Error should contain minimum length: {}", min);
                },
                ValidationError::OutOfRange { field, min, max } => {
                    assert!(error_string.contains(&field), "Error should contain field name: {}", field);
                    assert!(error_string.contains(&min.to_string()), "Error should contain min value: {}", min);
                    assert!(error_string.contains(&max.to_string()), "Error should contain max value: {}", max);
                },
                _ => {}
            }
        }
    }
} 