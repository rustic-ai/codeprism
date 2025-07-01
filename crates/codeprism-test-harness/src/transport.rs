//! Transport Layer for MCP Communication
//!
//! This module provides a comprehensive transport abstraction layer that supports
//! multiple communication mechanisms for MCP servers including stdio, HTTP POST,
//! and Server-Sent Events (SSE) with connection management and reliability testing.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::time::{timeout, Instant};

use crate::protocol::jsonrpc::JsonRpcMessage;

/// Transport configuration for different connection types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportConfig {
    /// Standard input/output communication
    Stdio {
        /// Connection timeout in seconds
        timeout: u64,
        /// Buffer size for stdio communication
        buffer_size: usize,
    },
    /// HTTP POST-based communication
    Http {
        /// Base URL for the MCP server
        base_url: String,
        /// Request timeout in seconds
        timeout: u64,
        /// Connection timeout in seconds
        connect_timeout: u64,
        /// Custom headers for requests
        headers: HashMap<String, String>,
    },
    /// Server-Sent Events with bidirectional communication
    Sse {
        /// SSE endpoint URL
        sse_url: String,
        /// HTTP endpoint for sending messages
        send_url: String,
        /// Connection timeout in seconds
        timeout: u64,
        /// Reconnection interval in seconds
        reconnect_interval: u64,
    },
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self::Stdio {
            timeout: 30,
            buffer_size: 8192,
        }
    }
}

/// Transport health status
#[derive(Debug, Clone, PartialEq)]
pub enum TransportHealth {
    /// Transport is healthy and ready
    Healthy,
    /// Transport is connecting or reconnecting
    Connecting,
    /// Transport has encountered recoverable errors
    Degraded,
    /// Transport is disconnected or failed
    Failed,
}

/// Transport-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },
    #[error("Connection timeout after {timeout}s")]
    Timeout { timeout: u64 },
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Transport not supported: {transport_type}")]
    UnsupportedTransport { transport_type: String },
}

/// Generic transport interface for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC message through the transport
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<(), TransportError>;

    /// Receive a JSON-RPC message from the transport
    async fn receive_message(&mut self) -> Result<JsonRpcMessage, TransportError>;

    /// Check the health status of the transport
    async fn health_check(&mut self) -> TransportHealth;

    /// Get transport-specific connection information
    fn connection_info(&self) -> HashMap<String, String>;

    /// Close the transport connection gracefully
    async fn close(&mut self) -> Result<(), TransportError>;

    /// Get transport type identifier
    fn transport_type(&self) -> &'static str;
}

/// Standard I/O transport implementation
pub struct StdioTransport {
    /// Writer for sending messages
    writer: BufWriter<tokio::process::ChildStdin>,
    /// Reader for receiving messages
    reader: BufReader<tokio::process::ChildStdout>,
    /// Configuration
    config: TransportConfig,
    /// Last activity timestamp
    last_activity: Instant,
    /// Connection status
    health: TransportHealth,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new(
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        config: TransportConfig,
    ) -> Self {
        Self {
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            config,
            last_activity: Instant::now(),
            health: TransportHealth::Healthy,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<(), TransportError> {
        let timeout_duration = match &self.config {
            TransportConfig::Stdio { timeout, .. } => Duration::from_secs(*timeout),
            _ => Duration::from_secs(30),
        };

        let send_result = timeout(timeout_duration, async {
            let json_str = serde_json::to_string(&message)?;
            self.writer.write_all(json_str.as_bytes()).await?;
            self.writer.write_all(b"\n").await?;
            self.writer.flush().await?;
            Ok::<(), TransportError>(())
        })
        .await;

        match send_result {
            Ok(Ok(())) => {
                self.last_activity = Instant::now();
                self.health = TransportHealth::Healthy;
                Ok(())
            }
            Ok(Err(e)) => {
                self.health = TransportHealth::Failed;
                Err(e)
            }
            Err(_) => {
                self.health = TransportHealth::Failed;
                Err(TransportError::Timeout {
                    timeout: timeout_duration.as_secs(),
                })
            }
        }
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage, TransportError> {
        let timeout_duration = match &self.config {
            TransportConfig::Stdio { timeout, .. } => Duration::from_secs(*timeout),
            _ => Duration::from_secs(30),
        };

        let receive_result = timeout(timeout_duration, async {
            let mut line = String::new();
            self.reader.read_line(&mut line).await?;
            let message: JsonRpcMessage = serde_json::from_str(&line)?;
            Ok::<JsonRpcMessage, TransportError>(message)
        })
        .await;

        match receive_result {
            Ok(Ok(message)) => {
                self.last_activity = Instant::now();
                self.health = TransportHealth::Healthy;
                Ok(message)
            }
            Ok(Err(e)) => {
                self.health = TransportHealth::Failed;
                Err(e)
            }
            Err(_) => {
                self.health = TransportHealth::Failed;
                Err(TransportError::Timeout {
                    timeout: timeout_duration.as_secs(),
                })
            }
        }
    }

    async fn health_check(&mut self) -> TransportHealth {
        // Check if we've been idle too long
        if self.last_activity.elapsed() > Duration::from_secs(300) {
            self.health = TransportHealth::Degraded;
        }
        self.health.clone()
    }

    fn connection_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("type".to_string(), "stdio".to_string());
        info.insert(
            "last_activity".to_string(),
            format!("{:.1}s ago", self.last_activity.elapsed().as_secs_f64()),
        );
        if let TransportConfig::Stdio {
            timeout,
            buffer_size,
        } = &self.config
        {
            info.insert("timeout".to_string(), format!("{}s", timeout));
            info.insert("buffer_size".to_string(), buffer_size.to_string());
        }
        info
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.writer.flush().await?;
        self.health = TransportHealth::Failed;
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}

/// HTTP POST-based transport implementation
pub struct HttpTransport {
    /// HTTP client
    client: reqwest::Client,
    /// Base URL for the MCP server
    base_url: String,
    /// Configuration
    config: TransportConfig,
    /// Last activity timestamp
    last_activity: Instant,
    /// Connection status
    health: TransportHealth,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
    /// Pending response for request/response pattern
    pending_response: Option<JsonRpcMessage>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(config: TransportConfig) -> Result<Self, TransportError> {
        let (base_url, timeout, connect_timeout, headers) = match &config {
            TransportConfig::Http {
                base_url,
                timeout,
                connect_timeout,
                headers,
            } => (
                base_url.clone(),
                *timeout,
                *connect_timeout,
                headers.clone(),
            ),
            _ => {
                return Err(TransportError::UnsupportedTransport {
                    transport_type: "Expected HTTP config".to_string(),
                });
            }
        };

        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .connect_timeout(Duration::from_secs(connect_timeout));

        // Add custom headers
        let mut default_headers = reqwest::header::HeaderMap::new();
        for (key, value) in headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                default_headers.insert(header_name, header_value);
            }
        }
        default_headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        client_builder = client_builder.default_headers(default_headers);

        let client = client_builder.build()?;

        Ok(Self {
            client,
            base_url,
            config,
            last_activity: Instant::now(),
            health: TransportHealth::Connecting,
            request_id: std::sync::atomic::AtomicU64::new(1),
            pending_response: None,
        })
    }

    /// Generate next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Send a request and receive response in one operation (HTTP pattern)
    pub async fn request_response(
        &mut self,
        message: JsonRpcMessage,
    ) -> Result<JsonRpcMessage, TransportError> {
        // Send the message
        self.send_message(message).await?;

        // Get the response
        if let Some(response) = self.pending_response.take() {
            Ok(response)
        } else {
            Err(TransportError::ProtocolError(
                "No response received".to_string(),
            ))
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_message(&mut self, mut message: JsonRpcMessage) -> Result<(), TransportError> {
        // For HTTP, we need to ensure requests have IDs
        if message.is_request() && message.id.is_none() {
            message.id = Some(serde_json::Value::Number(serde_json::Number::from(
                self.next_request_id(),
            )));
        }

        let json_body = serde_json::to_string(&message)?;

        let response = self
            .client
            .post(&self.base_url)
            .body(json_body)
            .send()
            .await?;

        if response.status().is_success() {
            // Get the response body and store it for receive_message()
            let response_text = response.text().await?;
            if !response_text.is_empty() {
                let response_message: JsonRpcMessage = serde_json::from_str(&response_text)?;
                self.pending_response = Some(response_message);
            }

            self.last_activity = Instant::now();
            self.health = TransportHealth::Healthy;
            Ok(())
        } else {
            self.health = TransportHealth::Degraded;
            Err(TransportError::ProtocolError(format!(
                "HTTP error: {} {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage, TransportError> {
        // For HTTP request/response pattern, return the pending response
        if let Some(response) = self.pending_response.take() {
            self.last_activity = Instant::now();
            Ok(response)
        } else {
            Err(TransportError::ProtocolError(
                "No pending response - use send_message first".to_string(),
            ))
        }
    }

    async fn health_check(&mut self) -> TransportHealth {
        // Perform a simple health check by sending a HEAD request
        match self.client.head(&self.base_url).send().await {
            Ok(response) if response.status().is_success() => {
                self.health = TransportHealth::Healthy;
            }
            Ok(_) => {
                self.health = TransportHealth::Degraded;
            }
            Err(_) => {
                self.health = TransportHealth::Failed;
            }
        }

        self.health.clone()
    }

    fn connection_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("type".to_string(), "http".to_string());
        info.insert("base_url".to_string(), self.base_url.clone());
        info.insert(
            "last_activity".to_string(),
            format!("{:.1}s ago", self.last_activity.elapsed().as_secs_f64()),
        );
        if let TransportConfig::Http {
            timeout,
            connect_timeout,
            ..
        } = &self.config
        {
            info.insert("timeout".to_string(), format!("{}s", timeout));
            info.insert(
                "connect_timeout".to_string(),
                format!("{}s", connect_timeout),
            );
        }
        info
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.health = TransportHealth::Failed;
        self.pending_response = None;
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

/// Server-Sent Events transport implementation
pub struct SseTransport {
    /// HTTP client for sending messages
    client: reqwest::Client,
    /// SSE endpoint URL
    sse_url: String,
    /// HTTP endpoint for sending messages
    send_url: String,
    /// Configuration
    config: TransportConfig,
    /// Last activity timestamp
    last_activity: Instant,
    /// Connection status
    health: TransportHealth,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl SseTransport {
    /// Create a new SSE transport
    pub fn new(config: TransportConfig) -> Result<Self, TransportError> {
        let (sse_url, send_url, timeout) = match &config {
            TransportConfig::Sse {
                sse_url,
                send_url,
                timeout,
                ..
            } => (sse_url.clone(), send_url.clone(), *timeout),
            _ => {
                return Err(TransportError::UnsupportedTransport {
                    transport_type: "Expected SSE config".to_string(),
                });
            }
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()?;

        Ok(Self {
            client,
            sse_url,
            send_url,
            config,
            last_activity: Instant::now(),
            health: TransportHealth::Connecting,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Generate next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl Transport for SseTransport {
    async fn send_message(&mut self, mut message: JsonRpcMessage) -> Result<(), TransportError> {
        // Ensure requests have IDs
        if message.is_request() && message.id.is_none() {
            message.id = Some(serde_json::Value::Number(serde_json::Number::from(
                self.next_request_id(),
            )));
        }

        let json_body = serde_json::to_string(&message)?;

        let response = self
            .client
            .post(&self.send_url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        if response.status().is_success() {
            self.last_activity = Instant::now();
            self.health = TransportHealth::Healthy;
            Ok(())
        } else {
            self.health = TransportHealth::Degraded;
            Err(TransportError::ProtocolError(format!(
                "SSE send error: {} {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage, TransportError> {
        // SSE stream reading functionality for future transport types
        // This would involve connecting to the SSE endpoint and parsing SSE events
        // Return error for unimplemented SSE transport
        Err(TransportError::UnsupportedTransport {
            transport_type: "SSE receive_message not yet implemented".to_string(),
        })
    }

    async fn health_check(&mut self) -> TransportHealth {
        // Check if both endpoints are reachable
        let send_check = self.client.head(&self.send_url).send().await;
        let sse_check = self.client.head(&self.sse_url).send().await;

        match (send_check, sse_check) {
            (Ok(_), Ok(_)) => {
                self.health = TransportHealth::Healthy;
            }
            (Ok(_), Err(_)) | (Err(_), Ok(_)) => {
                self.health = TransportHealth::Degraded;
            }
            (Err(_), Err(_)) => {
                self.health = TransportHealth::Failed;
            }
        }

        self.health.clone()
    }

    fn connection_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("type".to_string(), "sse".to_string());
        info.insert("sse_url".to_string(), self.sse_url.clone());
        info.insert("send_url".to_string(), self.send_url.clone());
        info.insert(
            "last_activity".to_string(),
            format!("{:.1}s ago", self.last_activity.elapsed().as_secs_f64()),
        );
        if let TransportConfig::Sse {
            timeout,
            reconnect_interval,
            ..
        } = &self.config
        {
            info.insert("timeout".to_string(), format!("{}s", timeout));
            info.insert(
                "reconnect_interval".to_string(),
                format!("{}s", reconnect_interval),
            );
        }
        info
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.health = TransportHealth::Failed;
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "sse"
    }
}

/// Transport factory for creating different transport types
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport based on configuration
    pub fn create_transport(config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        match config {
            TransportConfig::Http { .. } => {
                let transport = HttpTransport::new(config)?;
                Ok(Box::new(transport))
            }
            TransportConfig::Sse { .. } => {
                let transport = SseTransport::new(config)?;
                Ok(Box::new(transport))
            }
            TransportConfig::Stdio { .. } => Err(TransportError::UnsupportedTransport {
                transport_type:
                    "Stdio transport requires process handles - use ServerProcess::spawn"
                        .to_string(),
            }),
        }
    }

    /// Get supported transport types
    pub fn supported_transports() -> Vec<&'static str> {
        vec!["stdio", "http", "sse"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert!(matches!(config, TransportConfig::Stdio { .. }));
    }

    #[test]
    fn test_transport_health_enum() {
        assert_eq!(TransportHealth::Healthy, TransportHealth::Healthy);
        assert_ne!(TransportHealth::Healthy, TransportHealth::Failed);
    }

    #[test]
    fn test_http_transport_creation() {
        let config = TransportConfig::Http {
            base_url: "http://localhost:8080/mcp".to_string(),
            timeout: 30,
            connect_timeout: 10,
            headers: HashMap::new(),
        };

        let result = HttpTransport::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sse_transport_creation() {
        let config = TransportConfig::Sse {
            sse_url: "http://localhost:8080/sse".to_string(),
            send_url: "http://localhost:8080/send".to_string(),
            timeout: 30,
            reconnect_interval: 5,
        };

        let result = SseTransport::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_factory() {
        let transports = TransportFactory::supported_transports();
        assert!(transports.contains(&"http"));
        assert!(transports.contains(&"sse"));
        assert!(transports.contains(&"stdio"));
    }

    #[test]
    fn test_transport_error_types() {
        let error = TransportError::Timeout { timeout: 30 };
        assert!(error.to_string().contains("30"));

        let error = TransportError::UnsupportedTransport {
            transport_type: "invalid".to_string(),
        };
        assert!(error.to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn test_http_transport_connection_info() {
        let config = TransportConfig::Http {
            base_url: "http://localhost:8080/mcp".to_string(),
            timeout: 30,
            connect_timeout: 10,
            headers: HashMap::new(),
        };

        let transport = HttpTransport::new(config).unwrap();
        let info = transport.connection_info();

        assert_eq!(info.get("type"), Some(&"http".to_string()));
        assert_eq!(
            info.get("base_url"),
            Some(&"http://localhost:8080/mcp".to_string())
        );
        assert!(info.contains_key("timeout"));
    }

    #[tokio::test]
    async fn test_sse_transport_connection_info() {
        let config = TransportConfig::Sse {
            sse_url: "http://localhost:8080/sse".to_string(),
            send_url: "http://localhost:8080/send".to_string(),
            timeout: 30,
            reconnect_interval: 5,
        };

        let transport = SseTransport::new(config).unwrap();
        let info = transport.connection_info();

        assert_eq!(info.get("type"), Some(&"sse".to_string()));
        assert_eq!(
            info.get("sse_url"),
            Some(&"http://localhost:8080/sse".to_string())
        );
        assert_eq!(
            info.get("send_url"),
            Some(&"http://localhost:8080/send".to_string())
        );
    }
}
