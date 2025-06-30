//! Data privacy compliance testing for MCP servers
//!
//! Tests compliance with GDPR, CCPA, and other data privacy regulations.

use anyhow::Result;
use serde_json::json;
use tracing::{debug, info};

use super::{SecurityTestCase, SecurityTestOutcome, SecurityTestResult};
use crate::protocol::client::McpClient;

/// Run data privacy compliance test
pub async fn run_privacy_test(
    client: &mut McpClient,
    test_case: &SecurityTestCase,
) -> Result<SecurityTestResult> {
    info!("Running privacy compliance test: {}", test_case.id);

    let mut findings = Vec::new();
    let mut score_impact = 0.0;
    let mut remediation = Vec::new();

    let regulation = test_case
        .input_data
        .get("regulation")
        .and_then(|v| v.as_str())
        .unwrap_or("GDPR");

    // Test data classification
    let classification_result = test_data_classification(client).await?;
    if classification_result {
        findings.push("Personal data properly identified and classified".to_string());
        score_impact += 15.0;
    } else {
        findings.push("Personal data not properly classified".to_string());
        score_impact -= 20.0;
        remediation.push("Implement proper data classification for personal data".to_string());
    }

    // Test data subject rights
    let rights_result = test_data_subject_rights(client, regulation).await?;
    if rights_result {
        findings.push("Data subject rights properly implemented".to_string());
        score_impact += 20.0;
    } else {
        findings.push("Data subject rights not properly implemented".to_string());
        score_impact -= 25.0;
        remediation.push("Implement required data subject rights".to_string());
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

/// Test data classification capabilities
async fn test_data_classification(client: &mut McpClient) -> Result<bool> {
    debug!("Testing data classification");

    let response = client
        .send_request(
            "data/classify",
            Some(json!({
                "data_type": "personal_data"
            })),
        )
        .await;

    Ok(response.is_ok())
}

/// Test data subject rights implementation
async fn test_data_subject_rights(client: &mut McpClient, regulation: &str) -> Result<bool> {
    debug!("Testing data subject rights for regulation: {}", regulation);

    let rights = match regulation {
        "GDPR" => vec!["access", "rectification", "erasure", "portability"],
        "CCPA" => vec!["know", "delete", "opt_out"],
        _ => vec!["access", "delete"],
    };

    let mut all_implemented = true;
    for right in rights {
        let endpoint = format!("rights/{}", right);
        let response = client.send_request(&endpoint, None).await;
        if response.is_err() {
            all_implemented = false;
            break;
        }
    }

    Ok(all_implemented)
}
