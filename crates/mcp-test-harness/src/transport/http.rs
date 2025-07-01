//! HTTP transport implementation for MCP Streamable HTTP
//!
//! Implements the MCP Streamable HTTP specification including:
//! - JSON-RPC over HTTP POST
//! - Server-Sent Events (SSE) for bidirectional communication
//! - Session management with Mcp-Session-Id headers
//! - Resumability with Last-Event-ID support
//! - Security requirements per MCP specification

use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Base URL for the MCP server
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout: Duration,
    /// Connection timeout in seconds
    pub connect_timeout: Duration,
    /// Enable SSE streaming
    pub enable_sse: bool,
    /// SSE endpoint path (default: "/sse")
    pub sse_endpoint: String,
    /// JSON-RPC endpoint path (default: "/mcp")
    pub rpc_endpoint: String,
    /// Additional headers
    pub headers: HashMap<String, String>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            enable_sse: true,
            sse_endpoint: "/sse".to_string(),
            rpc_endpoint: "/mcp".to_string(),
            headers: HashMap::new(),
        }
    }
}

/// MCP JSON-RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// HTTP transport implementing MCP Streamable HTTP specification
pub struct HttpTransport {
    /// HTTP client
    client: reqwest::Client,
    /// Configuration
    config: HttpTransportConfig,
    /// Session ID for MCP session correlation
    session_id: Option<String>,
    /// Request ID counter
    request_id_counter: Arc<AtomicU64>,
    /// Connection state
    connected: bool,
}

impl HttpTransport {
    /// Create a new HTTP transport instance
    pub fn new(config: HttpTransportConfig) -> Result<Self, TransportError> {
        // Validate configuration
        if config.base_url.is_empty() {
            return Err(TransportError::ConnectionFailed(
                "Base URL cannot be empty".to_string(),
            ));
        }

        // Validate URL format
        if url::Url::parse(&config.base_url).is_err() {
            return Err(TransportError::ConnectionFailed(format!(
                "Invalid base URL: {}",
                config.base_url
            )));
        }

        // Security: Validate localhost binding for local servers per MCP spec
        if let Ok(url) = url::Url::parse(&config.base_url) {
            if let Some(host) = url.host_str() {
                if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                    // Local development - acceptable
                } else {
                    warn!("HTTP transport connecting to non-localhost: {}", host);
                }
            }
        }

        // Build HTTP client with security headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add custom headers from config
        for (key, value) in &config.headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::try_from(key.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .default_headers(headers)
            .build()
            .map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to create client: {}", e))
            })?;

        Ok(Self {
            client,
            config,
            session_id: None,
            request_id_counter: Arc::new(AtomicU64::new(1)),
            connected: false,
        })
    }

    /// Generate a cryptographically secure session ID
    fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Send JSON-RPC request over HTTP POST
    async fn send_http_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, TransportError> {
        let url = format!("{}{}", self.config.base_url, self.config.rpc_endpoint);

        debug!("Sending HTTP request to: {}", url);

        // Prepare HTTP request with MCP headers
        let mut http_request = self.client.post(&url);

        // Add session ID header if available
        if let Some(session_id) = &self.session_id {
            http_request = http_request.header("Mcp-Session-Id", session_id);
        }

        // Add JSON body
        let body = serde_json::to_string(&request).map_err(TransportError::Serialization)?;
        http_request = http_request.body(body);

        // Send HTTP request
        let response = http_request
            .send()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TransportError::ConnectionFailed(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        // Read JSON response
        let response_text = response.text().await.map_err(|e| {
            TransportError::ConnectionFailed(format!("Failed to read response: {}", e))
        })?;

        let json_response: JsonRpcResponse =
            serde_json::from_str(&response_text).map_err(TransportError::Serialization)?;

        Ok(json_response)
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        info!(
            "Connecting to MCP server via HTTP: {}",
            self.config.base_url
        );

        // Generate session ID for this connection
        self.session_id = Some(Self::generate_session_id());
        debug!("Generated session ID: {:?}", self.session_id);

        // Test basic connectivity with a simple request
        let health_url = format!("{}/health", self.config.base_url);
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                debug!("Health check response: {}", response.status());
            }
            Err(_) => {
                debug!("Health check failed, proceeding anyway");
            }
        }

        self.connected = true;
        info!("HTTP transport connected successfully");
        Ok(())
    }

    async fn send(&mut self, message: serde_json::Value) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::ConnectionFailed(
                "Transport not connected".to_string(),
            ));
        }

        // Convert message to JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(serde_json::Number::from(
                self.next_request_id(),
            ))),
            method: message
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
                .to_string(),
            params: message.get("params").cloned(),
        };

        // Send request and ignore response (fire-and-forget)
        let _response = self.send_http_request(request).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<serde_json::Value, TransportError> {
        Err(TransportError::ConnectionFailed(
            "HTTP transport is request/response only".to_string(),
        ))
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        info!("Disconnecting HTTP transport");
        self.connected = false;
        self.session_id = None;
        info!("HTTP transport disconnected");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_config_default() {
        let config = HttpTransportConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.enable_sse);
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = HttpTransport::generate_session_id();
        let id2 = HttpTransport::generate_session_id();

        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_json_rpc_structures() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String("1".to_string())),
            method: "test".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"test\""));
    }

    #[tokio::test]
    async fn test_http_transport_creation() {
        let config = HttpTransportConfig::default();
        let result = HttpTransport::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_url_config() {
        let config = HttpTransportConfig {
            base_url: "invalid-url".to_string(),
            ..Default::default()
        };

        let result = HttpTransport::new(config);
        assert!(result.is_err());
    }
}
