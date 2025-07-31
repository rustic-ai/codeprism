//! Security analysis module

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Security vulnerability information
#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub vulnerability_type: String,
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
    pub cvss_score: Option<f32>,
    pub owasp_category: Option<String>,
    pub confidence: f32,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
}

/// CVSS Score components for vulnerability assessment
#[derive(Debug, Clone)]
pub struct CvssScore {
    pub base_score: f32,
    pub impact_score: f32,
    pub exploitability_score: f32,
    pub severity_level: String,
}

/// Security analyzer for code analysis
pub struct SecurityAnalyzer {
    patterns: HashMap<String, Vec<VulnerabilityPattern>>,
}

#[derive(Debug, Clone)]
pub struct VulnerabilityPattern {
    name: String,
    pattern: Regex,
    severity: String,
    description: String,
    recommendation: String,
    cvss_base_score: f32,
    owasp_category: Option<String>,
    confidence: f32,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // SQL Injection patterns
        let sql_patterns = vec![
            VulnerabilityPattern {
                name: "SQL Injection".to_string(),
                pattern: Regex::new(r#"(?i)(query|execute|exec)\s*\([^)]*\+[^)]*\)"#).unwrap(),
                severity: "high".to_string(),
                description: "Potential SQL injection vulnerability detected".to_string(),
                recommendation: "Use parameterized queries or prepared statements".to_string(),
                cvss_base_score: 8.1,
                owasp_category: Some("A03:2021 – Injection".to_string()),
                confidence: 0.8,
            },
            VulnerabilityPattern {
                name: "SQL Injection Format".to_string(),
                pattern: Regex::new(
                    r#"(?i)(query|execute|exec)\s*\(\s*['""][^'"]*%[sd][^'"]*['""]"#,
                )
                .unwrap(),
                severity: "high".to_string(),
                description: "SQL query using string formatting detected".to_string(),
                recommendation: "Use parameterized queries instead of string formatting"
                    .to_string(),
                cvss_base_score: 8.1,
                owasp_category: Some("A03:2021 – Injection".to_string()),
                confidence: 0.9,
            },
        ];
        self.patterns.insert("injection".to_string(), sql_patterns);

        // XSS vulnerability patterns
        let xss_patterns = vec![
            VulnerabilityPattern {
                name: "XSS via innerHTML".to_string(),
                pattern: Regex::new(r#"(?i)\.innerHTML\s*=\s*[^;]*\+[^;]*"#).unwrap(),
                severity: "high".to_string(),
                description: "Potential XSS vulnerability through innerHTML assignment".to_string(),
                recommendation: "Use textContent or properly sanitize HTML content".to_string(),
                cvss_base_score: 7.5,
                owasp_category: Some("A07:2021 – Cross-Site Scripting (XSS)".to_string()),
                confidence: 0.8,
            },
            VulnerabilityPattern {
                name: "XSS via document.write".to_string(),
                pattern: Regex::new(r#"(?i)document\.write\s*\([^)]*\+[^)]*\)"#).unwrap(),
                severity: "high".to_string(),
                description: "Potential XSS vulnerability through document.write".to_string(),
                recommendation: "Avoid document.write, use DOM manipulation methods".to_string(),
                cvss_base_score: 7.5,
                owasp_category: Some("A07:2021 – Cross-Site Scripting (XSS)".to_string()),
                confidence: 0.9,
            },
            VulnerabilityPattern {
                name: "XSS via eval".to_string(),
                pattern: Regex::new(r#"(?i)eval\s*\([^)]*\+[^)]*\)"#).unwrap(),
                severity: "critical".to_string(),
                description: "Critical XSS vulnerability through eval function".to_string(),
                recommendation: "Never use eval with user input, use JSON.parse for data"
                    .to_string(),
                cvss_base_score: 9.3,
                owasp_category: Some("A07:2021 – Cross-Site Scripting (XSS)".to_string()),
                confidence: 0.95,
            },
        ];
        self.patterns.insert("xss".to_string(), xss_patterns);

        // CSRF vulnerability patterns
        let csrf_patterns = vec![
            VulnerabilityPattern {
                name: "Missing CSRF Token".to_string(),
                pattern: Regex::new(r#"(?i)<form[^>]*method\s*=\s*['""]post['""][^>]*>"#).unwrap(),
                severity: "medium".to_string(),
                description: "POST form without visible CSRF protection".to_string(),
                recommendation: "Implement CSRF tokens for all state-changing operations"
                    .to_string(),
                cvss_base_score: 6.5,
                owasp_category: Some("A01:2021 – Broken Access Control".to_string()),
                confidence: 0.6,
            },
            VulnerabilityPattern {
                name: "AJAX without CSRF".to_string(),
                pattern: Regex::new(
                    r#"(?i)\$\.post\s*\([^)]*\)|fetch\s*\([^)]*method:\s*['""]POST['""]"#,
                )
                .unwrap(),
                severity: "medium".to_string(),
                description: "AJAX POST request without visible CSRF protection".to_string(),
                recommendation: "Include CSRF tokens in AJAX requests headers".to_string(),
                cvss_base_score: 6.5,
                owasp_category: Some("A01:2021 – Broken Access Control".to_string()),
                confidence: 0.5,
            },
        ];
        self.patterns.insert("csrf".to_string(), csrf_patterns);

        // Authentication patterns (enhanced)
        let auth_patterns = vec![
            VulnerabilityPattern {
                name: "Hardcoded Password".to_string(),
                pattern: Regex::new(r#"(?i)(password|pwd|passwd)\s*=\s*['""][^'""]{3,}['""]"#)
                    .unwrap(),
                severity: "critical".to_string(),
                description: "Hardcoded password detected".to_string(),
                recommendation:
                    "Store passwords securely using environment variables or secure vaults"
                        .to_string(),
                cvss_base_score: 9.1,
                owasp_category: Some(
                    "A07:2021 – Identification and Authentication Failures".to_string(),
                ),
                confidence: 0.9,
            },
            VulnerabilityPattern {
                name: "Weak Password Check".to_string(),
                pattern: Regex::new(r#"(?i)len\s*\(\s*password\s*\)\s*[<>=]\s*[1-5]"#).unwrap(),
                severity: "medium".to_string(),
                description: "Weak password length requirement detected".to_string(),
                recommendation: "Enforce stronger password requirements (minimum 8 characters)"
                    .to_string(),
                cvss_base_score: 5.3,
                owasp_category: Some(
                    "A07:2021 – Identification and Authentication Failures".to_string(),
                ),
                confidence: 0.8,
            },
            VulnerabilityPattern {
                name: "Hardcoded API Key".to_string(),
                pattern: Regex::new(
                    r#"(?i)(api_key|apikey|access_key)\s*=\s*['""][a-zA-Z0-9]{16,}['""]"#,
                )
                .unwrap(),
                severity: "critical".to_string(),
                description: "Hardcoded API key detected".to_string(),
                recommendation: "Store API keys in environment variables or secure configuration"
                    .to_string(),
                cvss_base_score: 9.1,
                owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
                confidence: 0.95,
            },
        ];
        self.patterns
            .insert("authentication".to_string(), auth_patterns);

        // Crypto patterns (enhanced)
        let crypto_patterns = vec![
            VulnerabilityPattern {
                name: "Weak Crypto Algorithm".to_string(),
                pattern: Regex::new(r#"(?i)(md5|sha1|des|rc4)\s*\("#).unwrap(),
                severity: "high".to_string(),
                description: "Weak cryptographic algorithm detected".to_string(),
                recommendation: "Use stronger algorithms like SHA-256, AES, or bcrypt".to_string(),
                cvss_base_score: 7.4,
                owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
                confidence: 0.9,
            },
            VulnerabilityPattern {
                name: "Hardcoded Crypto Key".to_string(),
                pattern: Regex::new(r#"(?i)(key|secret|token)\s*=\s*['""][a-fA-F0-9]{16,}['""]"#)
                    .unwrap(),
                severity: "critical".to_string(),
                description: "Hardcoded cryptographic key detected".to_string(),
                recommendation: "Store keys securely using key management systems".to_string(),
                cvss_base_score: 9.8,
                owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
                confidence: 0.9,
            },
            VulnerabilityPattern {
                name: "Weak Random Number Generation".to_string(),
                pattern: Regex::new(r#"(?i)(Math\.random|random\.randint)\s*\("#).unwrap(),
                severity: "medium".to_string(),
                description: "Weak random number generation for security purposes".to_string(),
                recommendation: "Use cryptographically secure random number generators".to_string(),
                cvss_base_score: 5.9,
                owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
                confidence: 0.7,
            },
        ];
        self.patterns.insert("crypto".to_string(), crypto_patterns);

        // Data exposure patterns (enhanced)
        let exposure_patterns = vec![
            VulnerabilityPattern {
                name: "Debug Information Exposure".to_string(),
                pattern: Regex::new(r#"(?i)(print|console\.log|debug|trace)\s*\([^)]*(?:password|token|key|secret)"#).unwrap(),
                severity: "medium".to_string(),
                description: "Sensitive information in debug output detected".to_string(),
                recommendation: "Remove debug statements containing sensitive data".to_string(),
                cvss_base_score: 5.3,
                owasp_category: Some("A09:2021 – Security Logging and Monitoring Failures".to_string()),
                confidence: 0.8,
            },
            VulnerabilityPattern {
                name: "Error Information Disclosure".to_string(),
                pattern: Regex::new(r#"(?i)except\s+\w+\s+as\s+\w+:\s*print\s*\(\s*\w+"#).unwrap(),
                severity: "low".to_string(),
                description: "Exception details exposed to user".to_string(),
                recommendation: "Log errors securely without exposing internal details".to_string(),
                cvss_base_score: 3.7,
                owasp_category: Some("A09:2021 – Security Logging and Monitoring Failures".to_string()),
                confidence: 0.6,
            },
            VulnerabilityPattern {
                name: "Sensitive Data in URL".to_string(),
                pattern: Regex::new(r#"(?i)(password|token|key|secret)=[^&\s]+"#).unwrap(),
                severity: "high".to_string(),
                description: "Sensitive information exposed in URL parameters".to_string(),
                recommendation: "Use POST requests or secure headers for sensitive data".to_string(),
                cvss_base_score: 7.5,
                owasp_category: Some("A02:2021 – Cryptographic Failures".to_string()),
                confidence: 0.9,
            },
        ];
        self.patterns
            .insert("data_exposure".to_string(), exposure_patterns);

        // Unsafe patterns (enhanced)
        let unsafe_patterns = vec![
            VulnerabilityPattern {
                name: "Command Injection".to_string(),
                pattern: Regex::new(r#"(?i)(system|exec|popen|subprocess)\s*\([^)]*\+[^)]*\)"#)
                    .unwrap(),
                severity: "critical".to_string(),
                description: "Potential command injection vulnerability detected".to_string(),
                recommendation: "Validate and sanitize input, use safe alternatives".to_string(),
                cvss_base_score: 9.8,
                owasp_category: Some("A03:2021 – Injection".to_string()),
                confidence: 0.9,
            },
            VulnerabilityPattern {
                name: "Path Traversal".to_string(),
                pattern: Regex::new(r#"(?i)(open|file|read)\s*\([^)]*\.\./[^)]*\)"#).unwrap(),
                severity: "high".to_string(),
                description: "Potential path traversal vulnerability detected".to_string(),
                recommendation: "Validate file paths and use safe path operations".to_string(),
                cvss_base_score: 7.5,
                owasp_category: Some("A01:2021 – Broken Access Control".to_string()),
                confidence: 0.8,
            },
            VulnerabilityPattern {
                name: "Deserialization of Untrusted Data".to_string(),
                pattern: Regex::new(
                    r#"(?i)(pickle\.loads|yaml\.load|json\.loads)\s*\([^)]*input[^)]*\)"#,
                )
                .unwrap(),
                severity: "critical".to_string(),
                description: "Unsafe deserialization of user input".to_string(),
                recommendation: "Validate and sanitize data before deserialization".to_string(),
                cvss_base_score: 9.8,
                owasp_category: Some("A08:2021 – Software and Data Integrity Failures".to_string()),
                confidence: 0.85,
            },
        ];
        self.patterns
            .insert("unsafe_patterns".to_string(), unsafe_patterns);
    }

    /// Calculate CVSS score for a vulnerability
    pub fn calculate_cvss_score(
        &self,
        pattern: &VulnerabilityPattern,
        context: Option<&str>,
    ) -> CvssScore {
        let mut base_score = pattern.cvss_base_score;

        // Adjust score based on context
        if let Some(ctx) = context {
            // Lower score for test files
            if ctx.contains("test") || ctx.contains("spec") {
                base_score *= 0.7;
            }
            // Higher score for production/main code
            if ctx.contains("main") || ctx.contains("prod") {
                base_score *= 1.1;
            }
        }

        // Ensure score stays within CVSS range
        base_score = base_score.clamp(0.0, 10.0);

        let severity_level = match base_score {
            0.0..=3.9 => "Low",
            4.0..=6.9 => "Medium",
            7.0..=8.9 => "High",
            9.0..=10.0 => "Critical",
            _ => "Unknown",
        }
        .to_string();

        CvssScore {
            base_score,
            impact_score: base_score * 0.6, // Simplified calculation
            exploitability_score: base_score * 0.4, // Simplified calculation
            severity_level,
        }
    }

    /// Analyze content for security vulnerabilities with enhanced reporting
    pub fn analyze_content_with_location(
        &self,
        content: &str,
        file_path: Option<&str>,
        vulnerability_types: &[String],
        severity_threshold: &str,
    ) -> Result<Vec<SecurityVulnerability>> {
        let mut vulnerabilities = Vec::new();

        let target_types = if vulnerability_types.contains(&"all".to_string()) {
            self.patterns.keys().cloned().collect::<Vec<_>>()
        } else {
            vulnerability_types.to_vec()
        };

        let lines: Vec<&str> = content.lines().collect();

        for vuln_type in target_types {
            if let Some(patterns) = self.patterns.get(&vuln_type) {
                for pattern in patterns {
                    if self.meets_severity_threshold(&pattern.severity, severity_threshold) {
                        for (line_idx, line) in lines.iter().enumerate() {
                            if let Some(capture) = pattern.pattern.find(line) {
                                let cvss_score = self.calculate_cvss_score(pattern, file_path);

                                vulnerabilities.push(SecurityVulnerability {
                                    vulnerability_type: pattern.name.clone(),
                                    severity: pattern.severity.clone(),
                                    description: pattern.description.clone(),
                                    location: Some(format!(
                                        "Line {}: Position {}",
                                        line_idx + 1,
                                        capture.start()
                                    )),
                                    recommendation: pattern.recommendation.clone(),
                                    cvss_score: Some(cvss_score.base_score),
                                    owasp_category: pattern.owasp_category.clone(),
                                    confidence: pattern.confidence,
                                    file_path: file_path.map(|s| s.to_string()),
                                    line_number: Some(line_idx + 1),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    /// Analyze content for security vulnerabilities (legacy method for compatibility)
    pub fn analyze_content(
        &self,
        content: &str,
        vulnerability_types: &[String],
        severity_threshold: &str,
    ) -> Result<Vec<SecurityVulnerability>> {
        self.analyze_content_with_location(content, None, vulnerability_types, severity_threshold)
    }

    /// Check if severity meets threshold
    fn meets_severity_threshold(&self, severity: &str, threshold: &str) -> bool {
        let severity_levels = ["low", "medium", "high", "critical"];
        let severity_idx = severity_levels
            .iter()
            .position(|&s| s == severity)
            .unwrap_or(0);
        let threshold_idx = severity_levels
            .iter()
            .position(|&s| s == threshold)
            .unwrap_or(0);

        severity_idx >= threshold_idx
    }

    /// Get security recommendations based on vulnerabilities with OWASP mapping
    pub fn get_security_recommendations(
        &self,
        vulnerabilities: &[SecurityVulnerability],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if vulnerabilities.is_empty() {
            recommendations.push(
                "No security vulnerabilities detected. Continue following security best practices."
                    .to_string(),
            );
            return recommendations;
        }

        // Group by vulnerability type and severity
        let mut vuln_counts = HashMap::new();
        let mut severity_counts = HashMap::new();
        let mut owasp_categories = HashSet::new();

        for vuln in vulnerabilities {
            *vuln_counts
                .entry(vuln.vulnerability_type.clone())
                .or_insert(0) += 1;
            *severity_counts.entry(vuln.severity.clone()).or_insert(0) += 1;
            if let Some(ref category) = vuln.owasp_category {
                owasp_categories.insert(category.clone());
            }
        }

        // Priority recommendations based on severity
        if severity_counts.get("critical").unwrap_or(&0) > &0 {
            recommendations.push("🚨 CRITICAL: Address critical vulnerabilities immediately - these pose severe security risks.".to_string());
        }
        if severity_counts.get("high").unwrap_or(&0) > &0 {
            recommendations.push(
                "⚠️  HIGH PRIORITY: High-severity vulnerabilities require urgent attention."
                    .to_string(),
            );
        }

        // Specific recommendations based on vulnerability types
        if vuln_counts.contains_key("SQL Injection")
            || vuln_counts.contains_key("SQL Injection Format")
        {
            recommendations.push("🛡️  Implement input validation and use parameterized queries for all database operations.".to_string());
        }

        if vuln_counts.contains_key("XSS via innerHTML")
            || vuln_counts.contains_key("XSS via document.write")
            || vuln_counts.contains_key("XSS via eval")
        {
            recommendations.push("🔒 Sanitize all user input and use safe DOM manipulation methods. Implement Content Security Policy (CSP).".to_string());
        }

        if vuln_counts.contains_key("Missing CSRF Token")
            || vuln_counts.contains_key("AJAX without CSRF")
        {
            recommendations.push("🔐 Implement anti-CSRF tokens for all state-changing operations and AJAX requests.".to_string());
        }

        if vuln_counts.contains_key("Hardcoded Password")
            || vuln_counts.contains_key("Hardcoded API Key")
            || vuln_counts.contains_key("Hardcoded Crypto Key")
        {
            recommendations.push("🗝️  Use environment variables or secure key management systems for sensitive data.".to_string());
        }

        if vuln_counts.contains_key("Command Injection") {
            recommendations.push(
                "⚡ Validate all user input and use safe alternatives to system commands."
                    .to_string(),
            );
        }

        if vuln_counts.contains_key("Weak Crypto Algorithm")
            || vuln_counts.contains_key("Weak Random Number Generation")
        {
            recommendations.push(
                "🔐 Upgrade to modern, secure cryptographic algorithms (SHA-256, AES-256, etc.)."
                    .to_string(),
            );
        }

        if vuln_counts.contains_key("Path Traversal") {
            recommendations.push(
                "📁 Implement proper path validation and use safe file operations.".to_string(),
            );
        }

        if vuln_counts.contains_key("Deserialization of Untrusted Data") {
            recommendations.push(
                "⚠️  Never deserialize untrusted data. Implement strict input validation."
                    .to_string(),
            );
        }

        // OWASP-based recommendations
        if !owasp_categories.is_empty() {
            recommendations.push(format!(
                "📋 OWASP Top 10 categories affected: {}",
                owasp_categories
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // General security recommendations
        recommendations
            .push("🔍 Conduct regular security audits and penetration testing.".to_string());
        recommendations.push(
            "📚 Follow OWASP security guidelines and implement security training for developers."
                .to_string(),
        );
        recommendations.push(
            "🛡️  Implement proper error handling that doesn't expose sensitive information."
                .to_string(),
        );
        recommendations
            .push("📊 Set up security monitoring and logging for threat detection.".to_string());

        recommendations
    }

    /// Generate comprehensive security report
    pub fn generate_security_report(
        &self,
        vulnerabilities: &[SecurityVulnerability],
    ) -> serde_json::Value {
        let mut severity_counts = HashMap::new();
        let mut owasp_counts = HashMap::new();
        let mut total_cvss_score = 0.0;
        let mut high_confidence_vulns = 0;

        for vuln in vulnerabilities {
            *severity_counts.entry(vuln.severity.clone()).or_insert(0) += 1;
            if let Some(ref owasp) = vuln.owasp_category {
                *owasp_counts.entry(owasp.clone()).or_insert(0) += 1;
            }
            if let Some(score) = vuln.cvss_score {
                total_cvss_score += score;
            }
            if vuln.confidence >= 0.8 {
                high_confidence_vulns += 1;
            }
        }

        let avg_cvss_score = if !vulnerabilities.is_empty() {
            total_cvss_score / vulnerabilities.len() as f32
        } else {
            0.0
        };

        let security_score = match avg_cvss_score {
            0.0..=3.9 => 85 + (15.0 * (4.0 - avg_cvss_score) / 4.0) as i32,
            4.0..=6.9 => 60 + (25.0 * (7.0 - avg_cvss_score) / 3.0) as i32,
            7.0..=8.9 => 30 + (30.0 * (9.0 - avg_cvss_score) / 2.0) as i32,
            9.0..=10.0 => (30.0 * (10.0 - avg_cvss_score)) as i32,
            _ => 0,
        };

        serde_json::json!({
            "summary": {
                "total_vulnerabilities": vulnerabilities.len(),
                "high_confidence_findings": high_confidence_vulns,
                "average_cvss_score": avg_cvss_score,
                "security_score": security_score.max(0)
            },
            "severity_breakdown": severity_counts,
            "owasp_categories": owasp_counts,
            "recommendations": self.get_security_recommendations(vulnerabilities)
        })
    }

    /// Analyze for specific vulnerability patterns
    pub fn detect_injection_vulnerabilities(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities = self.analyze_content(content, &["injection".to_string()], "low")?;

        Ok(vulnerabilities
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "type": v.vulnerability_type,
                    "severity": v.severity,
                    "description": v.description,
                    "location": v.location,
                    "recommendation": v.recommendation
                })
            })
            .collect())
    }

    /// Analyze for authentication issues
    pub fn detect_authentication_issues(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities =
            self.analyze_content(content, &["authentication".to_string()], "low")?;

        Ok(vulnerabilities
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "type": v.vulnerability_type,
                    "severity": v.severity,
                    "description": v.description,
                    "location": v.location,
                    "recommendation": v.recommendation
                })
            })
            .collect())
    }

    /// Analyze for data exposure issues
    pub fn detect_data_exposure_issues(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities =
            self.analyze_content(content, &["data_exposure".to_string()], "low")?;

        Ok(vulnerabilities
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "type": v.vulnerability_type,
                    "severity": v.severity,
                    "description": v.description,
                    "location": v.location,
                    "recommendation": v.recommendation
                })
            })
            .collect())
    }
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let analyzer = SecurityAnalyzer::new();

        let vulnerable_code = r#"query("SELECT * FROM users WHERE id = " + user_id)"#;
        let vulnerabilities = analyzer
            .analyze_content(vulnerable_code, &["injection".to_string()], "low")
            .unwrap();

        assert!(
            !vulnerabilities.is_empty(),
            "Should find security vulnerabilities"
        );
        assert_eq!(vulnerabilities[0].vulnerability_type, "SQL Injection");
    }

    #[test]
    fn test_hardcoded_password_detection() {
        let analyzer = SecurityAnalyzer::new();

        let vulnerable_code = r#"password = "admin123""#;
        let vulnerabilities = analyzer
            .analyze_content(vulnerable_code, &["authentication".to_string()], "low")
            .unwrap();

        assert!(
            !vulnerabilities.is_empty(),
            "Should find security vulnerabilities"
        );
        assert_eq!(vulnerabilities[0].vulnerability_type, "Hardcoded Password");
    }

    #[test]
    fn test_weak_crypto_detection() {
        let analyzer = SecurityAnalyzer::new();

        let vulnerable_code = r#"hash = md5(password)"#;
        let vulnerabilities = analyzer
            .analyze_content(vulnerable_code, &["crypto".to_string()], "low")
            .unwrap();

        assert!(
            !vulnerabilities.is_empty(),
            "Should find security vulnerabilities"
        );
        assert_eq!(
            vulnerabilities[0].vulnerability_type,
            "Weak Crypto Algorithm"
        );
    }

    #[test]
    fn test_severity_threshold() {
        let analyzer = SecurityAnalyzer::new();

        assert!(analyzer.meets_severity_threshold("high", "medium"));
        assert!(!analyzer.meets_severity_threshold("low", "high"));
        assert!(analyzer.meets_severity_threshold("critical", "high"));
    }

    #[test]
    fn test_security_recommendations() {
        let analyzer = SecurityAnalyzer::new();

        let vulnerabilities = vec![SecurityVulnerability {
            vulnerability_type: "SQL Injection".to_string(),
            severity: "high".to_string(),
            description: "Test".to_string(),
            location: None,
            recommendation: "Test".to_string(),
            cvss_score: None,
            owasp_category: None,
            confidence: 0.0,
            file_path: None,
            line_number: None,
        }];

        let recommendations = analyzer.get_security_recommendations(&vulnerabilities);
        assert!(!recommendations.is_empty(), "Should not be empty");
    }
}
