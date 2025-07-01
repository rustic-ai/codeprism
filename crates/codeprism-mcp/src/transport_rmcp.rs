//! MCP Transport layer implementation using RMCP SDK
//!
//! This module provides the same API as our original transport.rs but uses
//! the official RMCP SDK transport layer underneath. This allows for
//! incremental migration while maintaining compatibility.

use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use tracing::{debug, info, warn};

use crate::protocol_rmcp::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Transport trait for MCP communication (compatibility layer)
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

/// Transport message types (compatibility layer)
#[derive(Debug, Clone)]
pub enum TransportMessage {
    /// JSON-RPC request
    Request(JsonRpcRequest),
    /// JSON-RPC response
    Response(JsonRpcResponse),
    /// JSON-RPC notification
    Notification(JsonRpcNotification),
}

/// RMCP-based stdio transport implementation
///
/// This transport uses the RMCP SDK's AsyncRwTransport with stdin/stdout
/// for newline-delimited JSON-RPC 2.0 communication as specified by MCP.
pub struct StdioTransport {
    /// The underlying RMCP transport
    rmcp_transport: Option<
        rmcp::transport::async_rw::AsyncRwTransport<
            rmcp::RoleServer,
            tokio::io::Stdin,
            tokio::io::Stdout,
        >,
    >,
    /// Whether the transport is started
    started: bool,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            rmcp_transport: None,
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

        info!("Starting RMCP-based stdio transport for MCP communication");

        // Create RMCP stdio transport using the new server method
        let (stdin, stdout) = rmcp::transport::io::stdio();
        self.rmcp_transport = Some(rmcp::transport::async_rw::AsyncRwTransport::new_server(
            stdin, stdout,
        ));

        self.started = true;
        debug!("RMCP stdio transport started successfully");

        Ok(())
    }

    async fn send(&mut self, message: TransportMessage) -> Result<()> {
        if !self.started {
            return Err(anyhow::anyhow!("Transport not started"));
        }

        let rmcp_transport = self
            .rmcp_transport
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("RMCP transport not initialized"))?;

        // Convert our TransportMessage to RMCP message
        let rmcp_message = convert_to_rmcp_message(message)?;

        // Send via RMCP transport
        rmcp::transport::Transport::send(rmcp_transport, rmcp_message)
            .await
            .map_err(|e| anyhow::anyhow!("RMCP transport send failed: {}", e))?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<TransportMessage>> {
        if !self.started {
            return Err(anyhow::anyhow!("Transport not started"));
        }

        let rmcp_transport = self
            .rmcp_transport
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("RMCP transport not initialized"))?;

        // Receive via RMCP transport
        match rmcp::transport::Transport::receive(rmcp_transport).await {
            Some(rmcp_message) => {
                debug!("Received RMCP message");
                // Convert RMCP message to our TransportMessage
                let transport_message = convert_from_rmcp_message(rmcp_message)?;
                Ok(Some(transport_message))
            }
            None => {
                debug!("RMCP transport closed");
                Ok(None)
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        if !self.started {
            return Ok(());
        }

        info!("Closing RMCP stdio transport");

        if let Some(rmcp_transport) = self.rmcp_transport.as_mut() {
            if let Err(e) = rmcp::transport::Transport::close(rmcp_transport).await {
                warn!("Error closing RMCP transport: {}", e);
            }
        }

        self.rmcp_transport = None;
        self.started = false;
        debug!("RMCP stdio transport closed");

        Ok(())
    }
}

/// Convert our TransportMessage to RMCP JsonRpcMessage
fn convert_to_rmcp_message(
    message: TransportMessage,
) -> Result<rmcp::service::TxJsonRpcMessage<rmcp::RoleServer>> {
    match message {
        TransportMessage::Request(req) => {
            // For server role, we typically don't send requests, but if we do,
            // we need to use the raw JSON message format
            let _json_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "id": req.id,
                "method": req.method,
                "params": req.params
            });

            // NOTE: Servers typically don't send requests in MCP protocol
            Err(anyhow::anyhow!("Server role should not send requests"))
        }
        TransportMessage::Response(resp) => {
            // Convert to RMCP response using ServerResult
            if let Some(error) = resp.error {
                // Error response
                let error_data = rmcp::model::ErrorData::new(
                    rmcp::model::ErrorCode(error.code),
                    error.message,
                    error.data,
                );
                let rmcp_error = rmcp::model::JsonRpcError {
                    jsonrpc: rmcp::model::JsonRpcVersion2_0,
                    id: json_value_to_request_id(resp.id),
                    error: error_data,
                };
                Ok(rmcp::model::JsonRpcMessage::Error(rmcp_error))
            } else {
                // Success response - convert JSON value to ServerResult
                let server_result: rmcp::model::ServerResult =
                    serde_json::from_value(resp.result.unwrap_or(serde_json::Value::Null))?;
                let rmcp_resp = rmcp::model::JsonRpcResponse {
                    jsonrpc: rmcp::model::JsonRpcVersion2_0,
                    id: json_value_to_request_id(resp.id),
                    result: server_result,
                };
                Ok(rmcp::model::JsonRpcMessage::Response(rmcp_resp))
            }
        }
        TransportMessage::Notification(notif) => {
            // Convert to RMCP notification using ServerNotification
            let server_notification: rmcp::model::ServerNotification =
                serde_json::from_value(serde_json::json!({
                    "method": notif.method,
                    "params": notif.params
                }))?;

            let rmcp_notif = rmcp::model::JsonRpcNotification {
                jsonrpc: rmcp::model::JsonRpcVersion2_0,
                notification: server_notification,
            };
            Ok(rmcp::model::JsonRpcMessage::Notification(rmcp_notif))
        }
    }
}

/// Convert RMCP JsonRpcMessage to our TransportMessage
fn convert_from_rmcp_message(
    rmcp_message: rmcp::service::RxJsonRpcMessage<rmcp::RoleServer>,
) -> Result<TransportMessage> {
    match rmcp_message {
        rmcp::model::JsonRpcMessage::Request(rmcp_req) => {
            // For server role, we receive ClientRequest messages
            // Convert the ClientRequest to generic JSON to extract method/params
            let client_req_json = serde_json::to_value(&rmcp_req.request)?;
            let method = client_req_json
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
                .to_string();
            let params = client_req_json.get("params").cloned();

            let req = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: request_id_to_json_value(rmcp_req.id),
                method,
                params,
            };
            Ok(TransportMessage::Request(req))
        }
        rmcp::model::JsonRpcMessage::Response(rmcp_resp) => {
            // Convert ClientResult to JSON value
            let result_json = serde_json::to_value(rmcp_resp.result)?;
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id_to_json_value(rmcp_resp.id),
                result: Some(result_json),
                error: None,
            };
            Ok(TransportMessage::Response(resp))
        }
        rmcp::model::JsonRpcMessage::Error(rmcp_error) => {
            let error = crate::protocol_rmcp::JsonRpcError {
                code: rmcp_error.error.code.0,
                message: rmcp_error.error.message.to_string(),
                data: rmcp_error.error.data,
            };
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id_to_json_value(rmcp_error.id),
                result: None,
                error: Some(error),
            };
            Ok(TransportMessage::Response(resp))
        }
        rmcp::model::JsonRpcMessage::Notification(rmcp_notif) => {
            // Convert ClientNotification to JSON to extract method/params
            let client_notif_json = serde_json::to_value(&rmcp_notif.notification)?;
            let method = client_notif_json
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
                .to_string();
            let params = client_notif_json.get("params").cloned();

            let notif = JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method,
                params,
            };
            Ok(TransportMessage::Notification(notif))
        }
        _ => Err(anyhow::anyhow!("Unsupported RMCP message type")),
    }
}

// Helper functions for RequestId conversion
fn json_value_to_request_id(value: serde_json::Value) -> rmcp::model::RequestId {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                rmcp::model::RequestId::Number(i as u32)
            } else {
                rmcp::model::RequestId::String(n.to_string().into())
            }
        }
        serde_json::Value::String(s) => rmcp::model::RequestId::String(s.into()),
        _ => rmcp::model::RequestId::String(value.to_string().into()),
    }
}

fn request_id_to_json_value(id: rmcp::model::RequestId) -> serde_json::Value {
    match id {
        rmcp::model::RequestId::Number(n) => serde_json::Value::Number(n.into()),
        rmcp::model::RequestId::String(s) => serde_json::Value::String(s.to_string()),
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
        let notification = JsonRpcNotification::new("test_notification".to_string(), None);
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

    #[test]
    fn test_request_id_conversion() {
        // Test number conversion
        let json_num = serde_json::Value::Number(42.into());
        let request_id = json_value_to_request_id(json_num.clone());
        let back_to_json = request_id_to_json_value(request_id);
        assert_eq!(json_num, back_to_json);

        // Test string conversion
        let json_str = serde_json::Value::String("test-id".to_string());
        let request_id = json_value_to_request_id(json_str.clone());
        let back_to_json = request_id_to_json_value(request_id);
        assert_eq!(json_str, back_to_json);
    }
}
