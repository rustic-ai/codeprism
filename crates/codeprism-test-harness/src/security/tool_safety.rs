//! Tool safety validation for MCP servers
//!
//! Tests that MCP tools have proper safety annotations and confirmation
//! requirements for dangerous operations.

use anyhow::Result;
use serde_json::{json, Value};
use tracing::{debug, info};

use super::{SecurityTestCase, SecurityTestOutcome, SecurityTestResult};
use crate::protocol::client::McpClient;

/// Run tool safety test
pub async fn run_safety_test(
    client: &mut McpClient,
    test_case: &SecurityTestCase,
) -> Result<SecurityTestResult> {
    info!("Running tool safety test: {}", test_case.id);

    let mut findings = Vec::new();
    let mut score_impact = 0.0;
    let mut remediation = Vec::new();

    // Get available tools
    let tools = get_available_tools(client).await?;

    // Test tool safety annotations
    let annotation_result = test_safety_annotations(&tools).await?;
    if annotation_result.properly_annotated {
        findings.push("Tools have proper safety annotations".to_string());
        score_impact += 20.0;
    } else {
        findings.push("Tools missing safety annotations".to_string());
        score_impact -= 25.0;
        remediation.push("Add safety annotations to dangerous tools".to_string());
    }

    // Test dangerous operation detection
    let detection_result = test_dangerous_operation_detection(client).await?;
    if detection_result.properly_detected {
        findings.push("Dangerous operations properly detected".to_string());
        score_impact += 25.0;
    } else {
        findings.push("Dangerous operations not properly detected".to_string());
        score_impact -= 30.0;
        remediation.push("Implement dangerous operation detection".to_string());
    }

    let outcome = if score_impact >= 0.0 {
        SecurityTestOutcome::Passed
    } else {
        SecurityTestOutcome::Failed
    };

    Ok(SecurityTestResult {
        test_id: test_case.id.clone(),
        outcome,
        findings,
        score_impact,
        remediation,
    })
}

/// Get available tools from server
async fn get_available_tools(client: &mut McpClient) -> Result<Vec<ToolInfo>> {
    debug!("Getting available tools");

    let response = client.send_request("tools/list", None).await?;

    let mut tools = Vec::new();
    if let Some(tools_array) = response.get("tools").and_then(|t| t.as_array()) {
        for tool in tools_array {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                tools.push(ToolInfo {
                    name: name.to_string(),
                    description: tool
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    input_schema: tool.get("inputSchema").cloned(),
                    metadata: tool.clone(),
                });
            }
        }
    }

    Ok(tools)
}

/// Test safety annotations
async fn test_safety_annotations(tools: &[ToolInfo]) -> Result<SafetyTestResult> {
    debug!("Testing safety annotations");

    let mut properly_annotated = true;

    for tool in tools {
        if is_potentially_dangerous_tool(tool) && !has_safety_annotations(tool) {
            properly_annotated = false;
            break;
        }
    }

    Ok(SafetyTestResult {
        properly_annotated,
        properly_detected: false,
        details: format!("Annotations checked for {} tools", tools.len()),
    })
}

/// Test dangerous operation detection
async fn test_dangerous_operation_detection(client: &mut McpClient) -> Result<SafetyTestResult> {
    debug!("Testing dangerous operation detection");

    let dangerous_operations = vec![
        json!({"action": "delete_all", "target": "*"}),
        json!({"command": "rm -rf /"}),
        json!({"operation": "format_disk"}),
    ];

    let mut properly_detected = true;
    for operation in dangerous_operations {
        let response = client
            .send_request(
                "tools/call",
                Some(json!({
                    "name": "dangerous_test",
                    "arguments": operation
                })),
            )
            .await;

        if response.is_ok() {
            properly_detected = false;
            break;
        }
    }

    Ok(SafetyTestResult {
        properly_annotated: false,
        properly_detected,
        details: "Dangerous operation detection tested".to_string(),
    })
}

/// Check if tool is potentially dangerous
fn is_potentially_dangerous_tool(tool: &ToolInfo) -> bool {
    let dangerous_keywords = [
        "delete", "remove", "destroy", "format", "execute", "command",
    ];
    let combined_text = format!(
        "{} {}",
        tool.name.to_lowercase(),
        tool.description.to_lowercase()
    );

    dangerous_keywords
        .iter()
        .any(|keyword| combined_text.contains(keyword))
}

/// Check if tool has safety annotations
fn has_safety_annotations(tool: &ToolInfo) -> bool {
    if let Some(metadata) = tool.metadata.as_object() {
        return metadata.contains_key("dangerous")
            || metadata.contains_key("confirmation_required")
            || metadata.contains_key("safety_level");
    }
    false
}

/// Tool information
#[allow(dead_code)]
#[derive(Debug)]
struct ToolInfo {
    name: String,
    description: String,
    input_schema: Option<Value>,
    metadata: Value,
}

/// Safety test result
#[allow(dead_code)]
#[derive(Debug)]
struct SafetyTestResult {
    properly_annotated: bool,
    properly_detected: bool,
    details: String,
}
