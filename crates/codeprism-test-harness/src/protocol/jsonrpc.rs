//! JSON-RPC 2.0 Implementation for MCP Protocol
//!
//! This module provides a complete implementation of JSON-RPC 2.0 specification
//! as required by the Model Context Protocol, including message validation,
//! serialization, and error handling.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// JSON-RPC 2.0 version string
pub const JSONRPC_VERSION: &str = "2.0";

/// A complete JSON-RPC 2.0 message that can be a request, response, or notification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcMessage {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request/response ID (not present for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Method name (present for requests and notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Parameters (present for requests and notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Result (present for successful responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (present for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: Value,
    /// Method name
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response (success)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: Value,
    /// Result data
    pub result: Value,
}

/// JSON-RPC 2.0 Error Response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: Value,
    /// Error information
    pub error: JsonRpcError,
}

/// JSON-RPC 2.0 Notification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC 2.0 error codes
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum JsonRpcErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    // Server error range: -32000 to -32099
}

/// Errors that can occur during JSON-RPC operations
#[derive(Debug, thiserror::Error)]
pub enum JsonRpcProcessingError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl JsonRpcMessage {
    /// Create a new request message
    pub fn request(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(Value::String(Uuid::new_v4().to_string())),
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a new request message with specific ID
    pub fn request_with_id(id: Value, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a new successful response message
    pub fn response(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Create a new error response message
    pub fn error_response(id: Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Create a new notification message
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        self.method.is_some() && self.id.is_some()
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        self.id.is_some()
            && self.method.is_none()
            && (self.result.is_some() || self.error.is_some())
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Validate JSON-RPC 2.0 compliance
    pub fn validate(&self) -> Result<(), JsonRpcProcessingError> {
        // Check JSON-RPC version
        if self.jsonrpc != JSONRPC_VERSION {
            return Err(JsonRpcProcessingError::InvalidRequest(format!(
                "Invalid jsonrpc version: {}",
                self.jsonrpc
            )));
        }

        // Validate based on message type
        if self.is_request() {
            self.validate_request()
        } else if self.is_response() {
            self.validate_response()
        } else if self.is_notification() {
            self.validate_notification()
        } else {
            Err(JsonRpcProcessingError::InvalidRequest(
                "Message must be a request, response, or notification".to_string(),
            ))
        }
    }

    /// Validate request message
    fn validate_request(&self) -> Result<(), JsonRpcProcessingError> {
        if self.method.is_none() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Request must have a method".to_string(),
            ));
        }

        if self.id.is_none() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Request must have an id".to_string(),
            ));
        }

        if self.result.is_some() || self.error.is_some() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Request cannot have result or error fields".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate response message
    fn validate_response(&self) -> Result<(), JsonRpcProcessingError> {
        if self.id.is_none() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Response must have an id".to_string(),
            ));
        }

        if self.method.is_some() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Response cannot have a method field".to_string(),
            ));
        }

        // Must have either result or error, but not both
        match (self.result.is_some(), self.error.is_some()) {
            (true, false) | (false, true) => Ok(()),
            (true, true) => Err(JsonRpcProcessingError::InvalidRequest(
                "Response cannot have both result and error".to_string(),
            )),
            (false, false) => Err(JsonRpcProcessingError::InvalidRequest(
                "Response must have either result or error".to_string(),
            )),
        }
    }

    /// Validate notification message
    fn validate_notification(&self) -> Result<(), JsonRpcProcessingError> {
        if self.method.is_none() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Notification must have a method".to_string(),
            ));
        }

        if self.id.is_some() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Notification cannot have an id".to_string(),
            ));
        }

        if self.result.is_some() || self.error.is_some() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Notification cannot have result or error fields".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert to typed request
    pub fn into_request(self) -> Result<JsonRpcRequest, JsonRpcProcessingError> {
        if !self.is_request() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Message is not a request".to_string(),
            ));
        }

        Ok(JsonRpcRequest {
            jsonrpc: self.jsonrpc,
            id: self.id.unwrap(),
            method: self.method.unwrap(),
            params: self.params,
        })
    }

    /// Convert to typed response
    pub fn into_response(self) -> Result<JsonRpcResponse, JsonRpcProcessingError> {
        if !self.is_response() || self.is_error() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Message is not a successful response".to_string(),
            ));
        }

        Ok(JsonRpcResponse {
            jsonrpc: self.jsonrpc,
            id: self.id.unwrap(),
            result: self.result.unwrap(),
        })
    }

    /// Convert to typed notification
    pub fn into_notification(self) -> Result<JsonRpcNotification, JsonRpcProcessingError> {
        if !self.is_notification() {
            return Err(JsonRpcProcessingError::InvalidRequest(
                "Message is not a notification".to_string(),
            ));
        }

        Ok(JsonRpcNotification {
            jsonrpc: self.jsonrpc,
            method: self.method.unwrap(),
            params: self.params,
        })
    }
}

impl JsonRpcError {
    /// Create a standard JSON-RPC error
    pub fn standard(code: JsonRpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            data: None,
        }
    }

    /// Create a standard JSON-RPC error with additional data
    pub fn standard_with_data(
        code: JsonRpcErrorCode,
        message: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::standard(JsonRpcErrorCode::ParseError, message)
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::standard(JsonRpcErrorCode::InvalidRequest, message)
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::standard(
            JsonRpcErrorCode::MethodNotFound,
            format!("Method not found: {}", method.into()),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::standard(JsonRpcErrorCode::InvalidParams, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::standard(JsonRpcErrorCode::InternalError, message)
    }
}

/// Parse a JSON-RPC message from a string
pub fn parse_message(json: &str) -> Result<JsonRpcMessage, JsonRpcProcessingError> {
    let message: JsonRpcMessage = serde_json::from_str(json)
        .map_err(|e| JsonRpcProcessingError::ParseError(e.to_string()))?;

    message.validate()?;
    Ok(message)
}

/// Serialize a JSON-RPC message to a string
pub fn serialize_message(message: &JsonRpcMessage) -> Result<String, JsonRpcProcessingError> {
    message.validate()?;
    serde_json::to_string(message).map_err(JsonRpcProcessingError::Serialization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_creation() {
        let request = JsonRpcMessage::request("test_method", Some(json!({"param": "value"})));

        assert_eq!(request.jsonrpc, JSONRPC_VERSION);
        assert!(request.id.is_some());
        assert_eq!(request.method, Some("test_method".to_string()));
        assert!(request.is_request());
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_response_creation() {
        let id = json!("test-id");
        let response = JsonRpcMessage::response(id.clone(), json!({"status": "success"}));

        assert_eq!(response.jsonrpc, JSONRPC_VERSION);
        assert_eq!(response.id, Some(id));
        assert!(response.result.is_some());
        assert!(response.is_response());
        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_notification_creation() {
        let notification =
            JsonRpcMessage::notification("notify", Some(json!({"message": "hello"})));

        assert_eq!(notification.jsonrpc, JSONRPC_VERSION);
        assert!(notification.id.is_none());
        assert_eq!(notification.method, Some("notify".to_string()));
        assert!(notification.is_notification());
        assert!(notification.validate().is_ok());
    }

    #[test]
    fn test_error_response() {
        let id = json!("error-id");
        let error = JsonRpcError::invalid_request("Bad request");
        let response = JsonRpcMessage::error_response(id.clone(), error);

        assert_eq!(response.jsonrpc, JSONRPC_VERSION);
        assert_eq!(response.id, Some(id));
        assert!(response.error.is_some());
        assert!(response.is_error());
        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_validation_failures() {
        // Invalid version
        let mut msg = JsonRpcMessage::request("test", None);
        msg.jsonrpc = "1.0".to_string();
        assert!(msg.validate().is_err());

        // Request without method
        let mut msg = JsonRpcMessage::request("test", None);
        msg.method = None;
        assert!(msg.validate().is_err());

        // Response with both result and error
        let mut msg = JsonRpcMessage::response(json!("id"), json!("result"));
        msg.error = Some(JsonRpcError::internal_error("test"));
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let request = JsonRpcMessage::request("test", Some(json!({"param": 42})));
        let json_str = serialize_message(&request).unwrap();
        let parsed = parse_message(&json_str).unwrap();

        assert_eq!(request.jsonrpc, parsed.jsonrpc);
        assert_eq!(request.method, parsed.method);
        assert_eq!(request.params, parsed.params);
    }
}
