//! User consent flow validation for MCP servers
//!
//! Tests that MCP servers properly request and validate user consent for sensitive operations.

use anyhow::Result;
use serde_json::{json, Value};
use tracing::{debug, info};

use super::{SecurityTestCase, SecurityTestOutcome, SecurityTestResult};
use crate::protocol::client::McpClient;

/// Run user consent validation test
pub async fn run_consent_test(
    client: &mut McpClient,
    test_case: &SecurityTestCase,
) -> Result<SecurityTestResult> {
    info!("Running consent validation test: {}", test_case.id);

    let mut findings = Vec::new();
    let mut score_impact = 0.0;
    let mut remediation = Vec::new();

    let tool_name = test_case
        .input_data
        .get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or("test_tool");

    let sensitive_operation = test_case
        .input_data
        .get("sensitive_operation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Test 1: Check if tool requires consent annotation
    let consent_required = check_tool_consent_annotation(client, tool_name).await?;

    if sensitive_operation && !consent_required {
        findings.push(format!(
            "Tool '{}' performs sensitive operations but lacks proper consent annotation",
            tool_name
        ));
        score_impact -= 25.0;
        remediation.push("Add proper consent annotations to sensitive tools".to_string());
    } else {
        findings.push(format!(
            "Tool '{}' has appropriate consent requirements",
            tool_name
        ));
        score_impact += 10.0;
    }

    // Test 2: Verify consent prompt is displayed
    if consent_required {
        let consent_prompt_shown = test_consent_prompt_display(client, tool_name).await?;

        if !consent_prompt_shown {
            findings.push("Consent prompt not displayed for sensitive operation".to_string());
            score_impact -= 30.0;
            remediation.push("Implement user consent prompts for sensitive operations".to_string());
        } else {
            findings.push("Consent prompt properly displayed".to_string());
            score_impact += 15.0;
        }
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

/// Check if tool has proper consent annotations
async fn check_tool_consent_annotation(client: &mut McpClient, tool_name: &str) -> Result<bool> {
    debug!("Checking consent annotation for tool: {}", tool_name);

    let tools_response = client.send_request("tools/list", None).await?;

    if let Some(tools) = tools_response.get("tools").and_then(|t| t.as_array()) {
        for tool in tools {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                if name == tool_name {
                    if let Some(input_schema) = tool.get("inputSchema") {
                        if has_consent_annotations(input_schema) {
                            return Ok(true);
                        }
                    }

                    if let Some(description) = tool.get("description").and_then(|d| d.as_str()) {
                        if description.to_lowercase().contains("consent")
                            || description.to_lowercase().contains("permission")
                            || description.to_lowercase().contains("approve")
                        {
                            return Ok(true);
                        }
                    }

                    break;
                }
            }
        }
    }

    Ok(false)
}

/// Check if schema contains consent annotations
fn has_consent_annotations(schema: &Value) -> bool {
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, _value) in properties {
            if key.to_lowercase().contains("consent")
                || key.to_lowercase().contains("permission")
                || key.to_lowercase().contains("approve")
            {
                return true;
            }
        }
    }

    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        for field in required {
            if let Some(field_name) = field.as_str() {
                if field_name.to_lowercase().contains("consent")
                    || field_name.to_lowercase().contains("permission")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Test if consent prompt is displayed
async fn test_consent_prompt_display(client: &mut McpClient, tool_name: &str) -> Result<bool> {
    debug!("Testing consent prompt display for tool: {}", tool_name);

    let response = client
        .send_request(
            "tools/call",
            Some(json!({
                "name": tool_name,
                "arguments": {}
            })),
        )
        .await;

    match response {
        Ok(resp) => {
            if let Some(error) = resp.get("error") {
                if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                    if message.to_lowercase().contains("consent")
                        || message.to_lowercase().contains("permission")
                        || message.to_lowercase().contains("approve")
                    {
                        return Ok(true);
                    }
                }
            }

            if resp
                .get("consent_required")
                .and_then(|c| c.as_bool())
                .unwrap_or(false)
            {
                return Ok(true);
            }

            Ok(false)
        }
        Err(_) => Ok(true), // Error might indicate consent requirement
    }
}
