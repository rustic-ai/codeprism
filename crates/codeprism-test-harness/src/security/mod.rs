//! Security testing framework for MCP servers
//!
//! Provides comprehensive security testing including OWASP compliance,
//! GDPR/CCPA privacy regulation testing, and vulnerability assessment.

use anyhow::Result;
use serde_json::Value;

use tracing::{debug, info};

use crate::protocol::client::McpClient;

pub mod authorization;
pub mod consent_validation;
pub mod input_security;
pub mod privacy_compliance;
pub mod protocol_security;
pub mod tool_safety;

/// Security test case definition
#[derive(Debug, Clone)]
pub struct SecurityTestCase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub test_type: SecurityTestType,
    pub input_data: Value,
    pub expected_behavior: SecurityExpectedBehavior,
}

/// Types of security tests
#[derive(Debug, Clone)]
pub enum SecurityTestType {
    Authorization,
    InputValidation,
    DataPrivacy,
    ProtocolSecurity,
    ToolSafety,
    ConsentValidation,
}

/// Expected security behavior
#[derive(Debug, Clone)]
pub struct SecurityExpectedBehavior {
    pub should_block: bool,
    pub required_warnings: Vec<String>,
    pub compliance_standards: Vec<String>,
}

/// Security test result
#[derive(Debug, Clone)]
pub struct SecurityTestResult {
    pub test_id: String,
    pub outcome: SecurityTestOutcome,
    pub findings: Vec<String>,
    pub score_impact: f64,
    pub remediation: Vec<String>,
}

/// Security test outcome
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityTestOutcome {
    Passed,
    Failed,
    Warning,
    Skipped,
}

/// Vulnerability severity levels
#[derive(Debug, Clone)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Main security test framework
pub struct SecurityTestFramework {
    config: SecurityConfig,
}

/// Security framework configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub owasp_enabled: bool,
    pub gdpr_enabled: bool,
    pub ccpa_enabled: bool,
    pub custom_tests: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            owasp_enabled: true,
            gdpr_enabled: true,
            ccpa_enabled: true,
            custom_tests: Vec::new(),
        }
    }
}

impl SecurityTestFramework {
    /// Create new security test framework
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Run comprehensive security tests
    pub async fn run_security_tests(&self, client: &mut McpClient) -> Result<SecurityTestResults> {
        info!("Starting comprehensive security testing");

        let mut results = SecurityTestResults {
            total_tests: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            findings: Vec::new(),
            overall_score: 0.0,
        };

        // Run different test categories
        if self.config.owasp_enabled {
            let owasp_results = self.run_owasp_tests(client).await?;
            results.merge(owasp_results);
        }

        if self.config.gdpr_enabled {
            let gdpr_results = self.run_gdpr_tests(client).await?;
            results.merge(gdpr_results);
        }

        results.calculate_overall_score();
        Ok(results)
    }

    /// Run OWASP Top 10 tests
    async fn run_owasp_tests(&self, client: &mut McpClient) -> Result<SecurityTestResults> {
        debug!("Running OWASP Top 10 2021 tests");

        let mut results = SecurityTestResults::new();

        // A01: Broken Access Control
        let auth_test = SecurityTestCase {
            id: "owasp_a01".to_string(),
            name: "Broken Access Control".to_string(),
            description: "Test for access control vulnerabilities".to_string(),
            test_type: SecurityTestType::Authorization,
            input_data: serde_json::json!({"test_unauthorized_access": true}),
            expected_behavior: SecurityExpectedBehavior {
                should_block: true,
                required_warnings: vec!["access_denied".to_string()],
                compliance_standards: vec!["OWASP_A01".to_string()],
            },
        };

        let auth_result = authorization::run_authorization_test(client, &auth_test).await?;
        results.add_result(auth_result);

        // A03: Injection
        let injection_test = SecurityTestCase {
            id: "owasp_a03".to_string(),
            name: "Injection".to_string(),
            description: "Test for injection vulnerabilities".to_string(),
            test_type: SecurityTestType::InputValidation,
            input_data: serde_json::json!({"test_sql_injection": true}),
            expected_behavior: SecurityExpectedBehavior {
                should_block: true,
                required_warnings: vec!["injection_detected".to_string()],
                compliance_standards: vec!["OWASP_A03".to_string()],
            },
        };

        let injection_result =
            input_security::run_input_security_test(client, &injection_test).await?;
        results.add_result(injection_result);

        Ok(results)
    }

    /// Run GDPR compliance tests
    async fn run_gdpr_tests(&self, client: &mut McpClient) -> Result<SecurityTestResults> {
        debug!("Running GDPR compliance tests");

        let mut results = SecurityTestResults::new();

        let gdpr_test = SecurityTestCase {
            id: "gdpr_consent".to_string(),
            name: "GDPR Consent Validation".to_string(),
            description: "Test GDPR Article 7 consent requirements".to_string(),
            test_type: SecurityTestType::ConsentValidation,
            input_data: serde_json::json!({"regulation": "GDPR"}),
            expected_behavior: SecurityExpectedBehavior {
                should_block: false,
                required_warnings: Vec::new(),
                compliance_standards: vec!["GDPR_Article_7".to_string()],
            },
        };

        let gdpr_result = consent_validation::run_consent_test(client, &gdpr_test).await?;
        results.add_result(gdpr_result);

        Ok(results)
    }
}

/// Security test results collection
#[derive(Debug)]
pub struct SecurityTestResults {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub warnings: u32,
    pub findings: Vec<SecurityTestResult>,
    pub overall_score: f64,
}

impl SecurityTestResults {
    fn new() -> Self {
        Self {
            total_tests: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            findings: Vec::new(),
            overall_score: 0.0,
        }
    }

    fn add_result(&mut self, result: SecurityTestResult) {
        self.total_tests += 1;
        match result.outcome {
            SecurityTestOutcome::Passed => self.passed += 1,
            SecurityTestOutcome::Failed => self.failed += 1,
            SecurityTestOutcome::Warning => self.warnings += 1,
            SecurityTestOutcome::Skipped => {}
        }
        self.findings.push(result);
    }

    fn merge(&mut self, other: SecurityTestResults) {
        self.total_tests += other.total_tests;
        self.passed += other.passed;
        self.failed += other.failed;
        self.warnings += other.warnings;
        self.findings.extend(other.findings);
    }

    fn calculate_overall_score(&mut self) {
        if self.total_tests == 0 {
            self.overall_score = 0.0;
            return;
        }

        let passed_weight = 1.0;
        let warning_weight = 0.5;
        let failed_weight = 0.0;

        let weighted_score = (self.passed as f64 * passed_weight)
            + (self.warnings as f64 * warning_weight)
            + (self.failed as f64 * failed_weight);

        self.overall_score = (weighted_score / self.total_tests as f64) * 100.0;
    }
}
