//! Security analysis module

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

/// Security vulnerability information
#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub vulnerability_type: String,
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
}

/// Security analyzer for code analysis
pub struct SecurityAnalyzer {
    patterns: HashMap<String, Vec<VulnerabilityPattern>>,
}

#[derive(Debug, Clone)]
struct VulnerabilityPattern {
    name: String,
    pattern: Regex,
    severity: String,
    description: String,
    recommendation: String,
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
            },
            VulnerabilityPattern {
                name: "SQL Injection Format".to_string(),
                pattern: Regex::new(r#"(?i)(query|execute|exec)\s*\(\s*['""][^'"]*%[sd][^'"]*['""]"#).unwrap(),
                severity: "high".to_string(),
                description: "SQL query using string formatting detected".to_string(),
                recommendation: "Use parameterized queries instead of string formatting".to_string(),
            },
        ];
        self.patterns.insert("injection".to_string(), sql_patterns);

        // Authentication patterns
        let auth_patterns = vec![
            VulnerabilityPattern {
                name: "Hardcoded Password".to_string(),
                pattern: Regex::new(r#"(?i)(password|pwd|passwd)\s*=\s*['""][^'""]{3,}['""]"#).unwrap(),
                severity: "high".to_string(),
                description: "Hardcoded password detected".to_string(),
                recommendation: "Store passwords securely using environment variables or secure vaults".to_string(),
            },
            VulnerabilityPattern {
                name: "Weak Password Check".to_string(),
                pattern: Regex::new(r#"(?i)len\s*\(\s*password\s*\)\s*[<>=]\s*[1-5]"#).unwrap(),
                severity: "medium".to_string(),
                description: "Weak password length requirement detected".to_string(),
                recommendation: "Enforce stronger password requirements (minimum 8 characters)".to_string(),
            },
        ];
        self.patterns.insert("authentication".to_string(), auth_patterns);

        // Crypto patterns
        let crypto_patterns = vec![
            VulnerabilityPattern {
                name: "Weak Crypto Algorithm".to_string(),
                pattern: Regex::new(r#"(?i)(md5|sha1|des|rc4)\s*\("#).unwrap(),
                severity: "high".to_string(),
                description: "Weak cryptographic algorithm detected".to_string(),
                recommendation: "Use stronger algorithms like SHA-256, AES, or bcrypt".to_string(),
            },
            VulnerabilityPattern {
                name: "Hardcoded Crypto Key".to_string(),
                pattern: Regex::new(r#"(?i)(key|secret|token)\s*=\s*['""][a-fA-F0-9]{16,}['""]"#).unwrap(),
                severity: "critical".to_string(),
                description: "Hardcoded cryptographic key detected".to_string(),
                recommendation: "Store keys securely using key management systems".to_string(),
            },
        ];
        self.patterns.insert("crypto".to_string(), crypto_patterns);

        // Data exposure patterns
        let exposure_patterns = vec![
            VulnerabilityPattern {
                name: "Debug Information Exposure".to_string(),
                pattern: Regex::new(r#"(?i)(print|console\.log|debug|trace)\s*\([^)]*(?:password|token|key|secret)"#).unwrap(),
                severity: "medium".to_string(),
                description: "Sensitive information in debug output detected".to_string(),
                recommendation: "Remove debug statements containing sensitive data".to_string(),
            },
            VulnerabilityPattern {
                name: "Error Information Disclosure".to_string(),
                pattern: Regex::new(r#"(?i)except\s+\w+\s+as\s+\w+:\s*print\s*\(\s*\w+"#).unwrap(),
                severity: "low".to_string(),
                description: "Exception details exposed to user".to_string(),
                recommendation: "Log errors securely without exposing internal details".to_string(),
            },
        ];
        self.patterns.insert("data_exposure".to_string(), exposure_patterns);

        // Unsafe patterns
        let unsafe_patterns = vec![
            VulnerabilityPattern {
                name: "Command Injection".to_string(),
                pattern: Regex::new(r#"(?i)(system|exec|popen|subprocess)\s*\([^)]*\+[^)]*\)"#).unwrap(),
                severity: "critical".to_string(),
                description: "Potential command injection vulnerability detected".to_string(),
                recommendation: "Validate and sanitize input, use safe alternatives".to_string(),
            },
            VulnerabilityPattern {
                name: "Path Traversal".to_string(),
                pattern: Regex::new(r#"(?i)(open|file|read)\s*\([^)]*\.\./[^)]*\)"#).unwrap(),
                severity: "high".to_string(),
                description: "Potential path traversal vulnerability detected".to_string(),
                recommendation: "Validate file paths and use safe path operations".to_string(),
            },
        ];
        self.patterns.insert("unsafe_patterns".to_string(), unsafe_patterns);
    }

    /// Analyze content for security vulnerabilities
    pub fn analyze_content(
        &self,
        content: &str,
        vulnerability_types: &[String],
        severity_threshold: &str,
    ) -> Result<Vec<SecurityVulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        let target_types = if vulnerability_types.contains(&"all".to_string()) {
            self.patterns.keys().cloned().collect::<Vec<_>>()
        } else {
            vulnerability_types.to_vec()
        };

        for vuln_type in target_types {
            if let Some(patterns) = self.patterns.get(&vuln_type) {
                for pattern in patterns {
                    if self.meets_severity_threshold(&pattern.severity, severity_threshold) {
                        if let Some(captures) = pattern.pattern.find(content) {
                            vulnerabilities.push(SecurityVulnerability {
                                vulnerability_type: pattern.name.clone(),
                                severity: pattern.severity.clone(),
                                description: pattern.description.clone(),
                                location: Some(format!("Position: {}", captures.start())),
                                recommendation: pattern.recommendation.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    /// Check if severity meets threshold
    fn meets_severity_threshold(&self, severity: &str, threshold: &str) -> bool {
        let severity_levels = ["low", "medium", "high", "critical"];
        let severity_idx = severity_levels.iter().position(|&s| s == severity).unwrap_or(0);
        let threshold_idx = severity_levels.iter().position(|&s| s == threshold).unwrap_or(0);
        
        severity_idx >= threshold_idx
    }

    /// Get security recommendations based on vulnerabilities
    pub fn get_security_recommendations(&self, vulnerabilities: &[SecurityVulnerability]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if vulnerabilities.is_empty() {
            recommendations.push("No security vulnerabilities detected. Continue following security best practices.".to_string());
            return recommendations;
        }

        // Group by vulnerability type
        let mut vuln_counts = HashMap::new();
        for vuln in vulnerabilities {
            *vuln_counts.entry(vuln.vulnerability_type.clone()).or_insert(0) += 1;
        }

        // General recommendations based on found vulnerabilities
        if vuln_counts.contains_key("SQL Injection") {
            recommendations.push("Implement input validation and use parameterized queries for all database operations.".to_string());
        }

        if vuln_counts.contains_key("Hardcoded Password") || vuln_counts.contains_key("Hardcoded Crypto Key") {
            recommendations.push("Use environment variables or secure key management systems for sensitive data.".to_string());
        }

        if vuln_counts.contains_key("Command Injection") {
            recommendations.push("Validate all user input and use safe alternatives to system commands.".to_string());
        }

        if vuln_counts.contains_key("Weak Crypto Algorithm") {
            recommendations.push("Upgrade to modern, secure cryptographic algorithms (SHA-256, AES-256, etc.).".to_string());
        }

        recommendations.push("Conduct regular security audits and penetration testing.".to_string());
        recommendations.push("Implement proper error handling that doesn't expose sensitive information.".to_string());
        
        recommendations
    }

    /// Analyze for specific vulnerability patterns
    pub fn detect_injection_vulnerabilities(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities = self.analyze_content(content, &["injection".to_string()], "low")?;
        
        Ok(vulnerabilities.into_iter().map(|v| serde_json::json!({
            "type": v.vulnerability_type,
            "severity": v.severity,
            "description": v.description,
            "location": v.location,
            "recommendation": v.recommendation
        })).collect())
    }

    /// Analyze for authentication issues
    pub fn detect_authentication_issues(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities = self.analyze_content(content, &["authentication".to_string()], "low")?;
        
        Ok(vulnerabilities.into_iter().map(|v| serde_json::json!({
            "type": v.vulnerability_type,
            "severity": v.severity,
            "description": v.description,
            "location": v.location,
            "recommendation": v.recommendation
        })).collect())
    }

    /// Analyze for data exposure issues
    pub fn detect_data_exposure_issues(&self, content: &str) -> Result<Vec<Value>> {
        let vulnerabilities = self.analyze_content(content, &["data_exposure".to_string()], "low")?;
        
        Ok(vulnerabilities.into_iter().map(|v| serde_json::json!({
            "type": v.vulnerability_type,
            "severity": v.severity,
            "description": v.description,
            "location": v.location,
            "recommendation": v.recommendation
        })).collect())
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
        let vulnerabilities = analyzer.analyze_content(vulnerable_code, &["injection".to_string()], "low").unwrap();
        
        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].vulnerability_type, "SQL Injection");
    }

    #[test]
    fn test_hardcoded_password_detection() {
        let analyzer = SecurityAnalyzer::new();
        
        let vulnerable_code = r#"password = "admin123""#;
        let vulnerabilities = analyzer.analyze_content(vulnerable_code, &["authentication".to_string()], "low").unwrap();
        
        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].vulnerability_type, "Hardcoded Password");
    }

    #[test]
    fn test_weak_crypto_detection() {
        let analyzer = SecurityAnalyzer::new();
        
        let vulnerable_code = r#"hash = md5(password)"#;
        let vulnerabilities = analyzer.analyze_content(vulnerable_code, &["crypto".to_string()], "low").unwrap();
        
        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].vulnerability_type, "Weak Crypto Algorithm");
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
        
        let vulnerabilities = vec![
            SecurityVulnerability {
                vulnerability_type: "SQL Injection".to_string(),
                severity: "high".to_string(),
                description: "Test".to_string(),
                location: None,
                recommendation: "Test".to_string(),
            }
        ];
        
        let recommendations = analyzer.get_security_recommendations(&vulnerabilities);
        assert!(!recommendations.is_empty());
    }
} 