//! MCP Transport layer implementation
//! 
//! This module implements the MCP transport layer using stdio as the primary transport.
//! Messages are JSON-RPC 2.0 format, delimited by newlines, as specified by MCP.

use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tracing::{debug, error, info, warn};

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport {
    /// Start the transport
    async fn start(&mut self) -> Result<()>;
    
    /// Send a JSON-RPC message
    async fn send(&mut self, message: TransportMessage) -> Result<()>;
    
    /// Receive a JSON-RPC message
    async fn receive(&mut self) -> Result<Option<TransportMessage>>;
    
    /// Close the transport
    async fn close(&mut self) -> Result<()>;
}

/// Transport message types
#[derive(Debug, Clone)]
pub enum TransportMessage {
    /// JSON-RPC request
    Request(JsonRpcRequest),
    /// JSON-RPC response
    Response(JsonRpcResponse),
    /// JSON-RPC notification
    Notification(JsonRpcNotification),
}

/// Stdio transport implementation for MCP
/// 
/// This transport uses stdin/stdout for communication with newline-delimited
/// JSON-RPC 2.0 messages, as specified by the MCP standard.
pub struct StdioTransport {
    /// Input reader for receiving messages
    input: Option<tokio::io::Lines<BufReader<tokio::io::Stdin>>>,
    /// Output writer for sending messages  
    output: Option<BufWriter<tokio::io::Stdout>>,
    /// Whether the transport is started
    started: bool,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            input: None,
            output: None,
            started: false,
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }

        info!("Starting stdio transport for MCP communication");

        // Set up stdin reader
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        self.input = Some(reader.lines());

        // Set up stdout writer
        let stdout = tokio::io::stdout();
        self.output = Some(BufWriter::new(stdout));

        self.started = true;
        debug!("Stdio transport started successfully");
        
        Ok(())
    }

    async fn send(&mut self, message: TransportMessage) -> Result<()> {
        if !self.started {
            return Err(anyhow::anyhow!("Transport not started"));
        }

        let output = self.output.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Output not initialized"))?;

        let json_str = match message {
            TransportMessage::Request(req) => {
                debug!("Sending JSON-RPC request: method={}, id={:?}", req.method, req.id);
                serde_json::to_string(&req)?
            }
            TransportMessage::Response(resp) => {
                debug!("Sending JSON-RPC response: id={:?}", resp.id);
                serde_json::to_string(&resp)?
            }
            TransportMessage::Notification(notif) => {
                debug!("Sending JSON-RPC notification: method={}", notif.method);
                serde_json::to_string(&notif)?
            }
        };

        // Send the message as a line (MCP spec requires newline delimiting)
        output.write_all(json_str.as_bytes()).await
            .map_err(|e| anyhow::anyhow!("Failed to write message: {}", e))?;
        output.write_all(b"\n").await
            .map_err(|e| anyhow::anyhow!("Failed to write newline: {}", e))?;
        output.flush().await
            .map_err(|e| anyhow::anyhow!("Failed to flush output: {}", e))?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<TransportMessage>> {
        if !self.started {
            return Err(anyhow::anyhow!("Transport not started"));
        }

        let input = self.input.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Input not initialized"))?;

        // Read the next line
        match input.next_line().await {
            Ok(Some(line)) => {
                if line.trim().is_empty() {
                    // Skip empty lines
                    return Ok(None);
                }

                debug!("Received message: {}", line);

                // Try to parse as different JSON-RPC message types
                // First, try to determine the message type by checking for fields
                let json_value: serde_json::Value = serde_json::from_str(&line)
                    .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

                if json_value.get("id").is_some() {
                    if json_value.get("method").is_some() {
                        // Has both id and method -> Request
                        let request: JsonRpcRequest = serde_json::from_str(&line)
                            .map_err(|e| anyhow::anyhow!("Failed to parse request: {}", e))?;
                        
                        debug!("Parsed JSON-RPC request: method={}, id={:?}", request.method, request.id);
                        Ok(Some(TransportMessage::Request(request)))
                    } else {
                        // Has id but no method -> Response
                        let response: JsonRpcResponse = serde_json::from_str(&line)
                            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;
                        
                        debug!("Parsed JSON-RPC response: id={:?}", response.id);
                        Ok(Some(TransportMessage::Response(response)))
                    }
                } else if json_value.get("method").is_some() {
                    // Has method but no id -> Notification
                    let notification: JsonRpcNotification = serde_json::from_str(&line)
                        .map_err(|e| anyhow::anyhow!("Failed to parse notification: {}", e))?;
                    
                    debug!("Parsed JSON-RPC notification: method={}", notification.method);
                    Ok(Some(TransportMessage::Notification(notification)))
                } else {
                    error!("Invalid JSON-RPC message format: {}", line);
                    Err(anyhow::anyhow!("Invalid JSON-RPC message format"))
                }
            }
            Ok(None) => {
                // End of input stream
                debug!("End of input stream");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to read line: {}", e);
                Err(anyhow::anyhow!("Failed to read line: {}", e))
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        if !self.started {
            return Ok(());
        }

        info!("Closing stdio transport");

        // Close output stream
        if let Some(mut output) = self.output.take() {
            if let Err(e) = output.flush().await {
                warn!("Error flushing output stream: {}", e);
            }
        }

        // Input stream will be closed automatically when dropped
        self.input.take();

        self.started = false;
        debug!("Stdio transport closed");
        
        Ok(())
    }
}

impl TransportMessage {
    /// Get the JSON-RPC version of the message
    pub fn jsonrpc_version(&self) -> &str {
        match self {
            TransportMessage::Request(req) => &req.jsonrpc,
            TransportMessage::Response(resp) => &resp.jsonrpc,
            TransportMessage::Notification(notif) => &notif.jsonrpc,
        }
    }

    /// Get the method name if this is a request or notification
    pub fn method(&self) -> Option<&str> {
        match self {
            TransportMessage::Request(req) => Some(&req.method),
            TransportMessage::Response(_) => None,
            TransportMessage::Notification(notif) => Some(&notif.method),
        }
    }

    /// Get the ID if this is a request or response
    pub fn id(&self) -> Option<&serde_json::Value> {
        match self {
            TransportMessage::Request(req) => Some(&req.id),
            TransportMessage::Response(resp) => Some(&resp.id),
            TransportMessage::Notification(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};

    #[test]
    fn test_transport_message_methods() {
        let request = JsonRpcRequest::new(
            serde_json::Value::Number(1.into()),
            "test_method".to_string(),
            None,
        );
        let msg = TransportMessage::Request(request);

        assert_eq!(msg.jsonrpc_version(), "2.0");
        assert_eq!(msg.method(), Some("test_method"));
        assert!(msg.id().is_some());
    }

    #[test]
    fn test_transport_message_response() {
        let response = JsonRpcResponse::success(
            serde_json::Value::Number(1.into()),
            serde_json::json!({"result": "success"}),
        );
        let msg = TransportMessage::Response(response);

        assert_eq!(msg.jsonrpc_version(), "2.0");
        assert_eq!(msg.method(), None);
        assert!(msg.id().is_some());
    }

    #[test]
    fn test_transport_message_notification() {
        let notification = JsonRpcNotification::new(
            "test_notification".to_string(),
            None,
        );
        let msg = TransportMessage::Notification(notification);

        assert_eq!(msg.jsonrpc_version(), "2.0");
        assert_eq!(msg.method(), Some("test_notification"));
        assert_eq!(msg.id(), None);
    }

    #[test]
    fn test_stdio_transport_creation() {
        let transport = StdioTransport::new();
        assert!(!transport.started);
    }

    #[test]
    fn test_stdio_transport_default() {
        let transport = StdioTransport::default();
        assert!(!transport.started);
    }
} 