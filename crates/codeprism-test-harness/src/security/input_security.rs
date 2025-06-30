//! Input security testing for MCP servers
//!
//! Tests for injection vulnerabilities, input validation, and
//! sanitization effectiveness.

use anyhow::Result;
use serde_json::json;
use tracing::{debug, info};

use super::{SecurityTestCase, SecurityTestOutcome, SecurityTestResult};
use crate::protocol::client::McpClient;

/// Run input security test
pub async fn run_input_security_test(
    client: &mut McpClient,
    test_case: &SecurityTestCase,
) -> Result<SecurityTestResult> {
    info!("Running input security test: {}", test_case.id);

    let mut findings = Vec::new();
    let mut score_impact = 0.0;
    let mut remediation = Vec::new();

    // Test SQL injection
    let sql_result = test_sql_injection(client).await?;
    if sql_result.injection_blocked {
        findings.push("SQL injection properly blocked".to_string());
        score_impact += 25.0;
    } else {
        findings.push("SQL injection vulnerability detected".to_string());
        score_impact -= 40.0;
        remediation.push("Implement SQL injection prevention".to_string());
    }

    // Test XSS prevention
    let xss_result = test_xss_prevention(client).await?;
    if xss_result.xss_blocked {
        findings.push("XSS attacks properly blocked".to_string());
        score_impact += 20.0;
    } else {
        findings.push("XSS vulnerability detected".to_string());
        score_impact -= 30.0;
        remediation.push("Implement XSS prevention and output sanitization".to_string());
    }

    // Test command injection
    let cmd_result = test_command_injection(client).await?;
    if cmd_result.command_blocked {
        findings.push("Command injection properly blocked".to_string());
        score_impact += 30.0;
    } else {
        findings.push("Command injection vulnerability detected".to_string());
        score_impact -= 50.0;
        remediation.push("Implement command injection prevention".to_string());
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

/// Test SQL injection vulnerability
async fn test_sql_injection(client: &mut McpClient) -> Result<InputSecurityResult> {
    debug!("Testing SQL injection");

    let sql_payloads = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM sensitive_data --",
        "'; INSERT INTO users (admin) VALUES (1); --",
    ];

    let mut injection_blocked = true;
    for payload in sql_payloads {
        let response = client
            .send_request(
                "tools/call",
                Some(json!({
                    "name": "database_query",
                    "arguments": {
                        "query": payload
                    }
                })),
            )
            .await;

        if let Ok(resp) = response {
            if resp.get("error").is_none() {
                injection_blocked = false;
                break;
            }
        }
    }

    Ok(InputSecurityResult {
        injection_blocked,
        xss_blocked: false,
        command_blocked: false,
        details: format!("SQL injection blocked: {}", injection_blocked),
    })
}

/// Test XSS prevention
async fn test_xss_prevention(client: &mut McpClient) -> Result<InputSecurityResult> {
    debug!("Testing XSS prevention");

    let xss_payloads = vec![
        "<script>alert('xss')</script>",
        "javascript:alert('xss')",
        "<img src=x onerror=alert('xss')>",
        "';alert('xss');//",
    ];

    let mut xss_blocked = true;
    for payload in xss_payloads {
        let response = client
            .send_request(
                "tools/call",
                Some(json!({
                    "name": "content_display",
                    "arguments": {
                        "content": payload
                    }
                })),
            )
            .await;

        if let Ok(resp) = response {
            if let Some(content) = resp.get("result") {
                if content.to_string().contains("script")
                    || content.to_string().contains("javascript")
                {
                    xss_blocked = false;
                    break;
                }
            }
        }
    }

    Ok(InputSecurityResult {
        injection_blocked: false,
        xss_blocked,
        command_blocked: false,
        details: format!("XSS blocked: {}", xss_blocked),
    })
}

/// Test command injection
async fn test_command_injection(client: &mut McpClient) -> Result<InputSecurityResult> {
    debug!("Testing command injection");

    let cmd_payloads = vec!["; ls -la", "&& cat /etc/passwd", "| whoami", "`id`"];

    let mut command_blocked = true;
    for payload in cmd_payloads {
        let response = client
            .send_request(
                "tools/call",
                Some(json!({
                    "name": "system_command",
                    "arguments": {
                        "command": format!("echo test{}", payload)
                    }
                })),
            )
            .await;

        if response.is_ok() {
            command_blocked = false;
            break;
        }
    }

    Ok(InputSecurityResult {
        injection_blocked: false,
        xss_blocked: false,
        command_blocked,
        details: format!("Command injection blocked: {}", command_blocked),
    })
}

/// Input security test result
#[allow(dead_code)]
#[derive(Debug, Default)]
struct InputSecurityResult {
    injection_blocked: bool,
    xss_blocked: bool,
    command_blocked: bool,
    details: String,
}
