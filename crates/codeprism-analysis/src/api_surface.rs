//! API surface analysis module

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// API element information
#[derive(Debug, Clone)]
pub struct ApiElement {
    pub element_type: String,
    pub name: String,
    pub visibility: String,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub deprecated: bool,
    pub breaking_change_risk: String,
}

/// API surface analyzer
pub struct ApiSurfaceAnalyzer {
    patterns: HashMap<String, Vec<ApiPattern>>,
}

#[derive(Debug, Clone)]
struct ApiPattern {
    _name: String,
    pattern: Regex,
    element_type: String,
    visibility_pattern: Option<Regex>,
}

impl ApiSurfaceAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Public API patterns
        let public_api_patterns = vec![
            ApiPattern {
                _name: "Public Function".to_string(),
                pattern: Regex::new(r"(?m)^(pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)").unwrap(),
                element_type: "function".to_string(),
                visibility_pattern: Some(Regex::new(r"^pub\s+").unwrap()),
            },
            ApiPattern {
                _name: "Public Class".to_string(),
                pattern: Regex::new(r"(?m)^(pub\s+)?(?:class|struct)\s+(\w+)").unwrap(),
                element_type: "class".to_string(),
                visibility_pattern: Some(Regex::new(r"^pub\s+").unwrap()),
            },
            ApiPattern {
                _name: "Public Method".to_string(),
                pattern: Regex::new(
                    r"(?m)^\s*(pub\s+)?(?:async\s+)?(?:def|fn)\s+(\w+)\s*\([^)]*\)",
                )
                .unwrap(),
                element_type: "method".to_string(),
                visibility_pattern: Some(Regex::new(r"^\s*pub\s+").unwrap()),
            },
            ApiPattern {
                _name: "Public Constant".to_string(),
                pattern: Regex::new(r"(?m)^(pub\s+)?const\s+(\w+)").unwrap(),
                element_type: "constant".to_string(),
                visibility_pattern: Some(Regex::new(r"^pub\s+").unwrap()),
            },
        ];
        self.patterns
            .insert("public_api".to_string(), public_api_patterns);

        // Versioning patterns
        let versioning_patterns = vec![
            ApiPattern {
                _name: "Version Annotation".to_string(),
                pattern: Regex::new(r#"@version\s*\(\s*["']([\d.]+)["']\s*\)"#).unwrap(),
                element_type: "version".to_string(),
                visibility_pattern: None,
            },
            ApiPattern {
                _name: "Since Annotation".to_string(),
                pattern: Regex::new(r#"@since\s*\(\s*["']([\d.]+)["']\s*\)"#).unwrap(),
                element_type: "version".to_string(),
                visibility_pattern: None,
            },
            ApiPattern {
                _name: "Deprecated Annotation".to_string(),
                pattern: Regex::new(r"@deprecated|#\[deprecated\]|@Deprecated").unwrap(),
                element_type: "deprecated".to_string(),
                visibility_pattern: None,
            },
        ];
        self.patterns
            .insert("versioning".to_string(), versioning_patterns);

        // Breaking change patterns
        let breaking_change_patterns = vec![
            ApiPattern {
                _name: "Parameter Change".to_string(),
                pattern: Regex::new(r"(?m)fn\s+\w+\s*\([^)]*\w+\s*:\s*\w+[^)]*\)").unwrap(),
                element_type: "breaking_change".to_string(),
                visibility_pattern: None,
            },
            ApiPattern {
                _name: "Return Type Change".to_string(),
                pattern: Regex::new(r"(?m)fn\s+\w+\s*\([^)]*\)\s*->\s*\w+").unwrap(),
                element_type: "breaking_change".to_string(),
                visibility_pattern: None,
            },
        ];
        self.patterns
            .insert("breaking_changes".to_string(), breaking_change_patterns);

        // Documentation patterns
        let documentation_patterns = vec![
            ApiPattern {
                _name: "Doc Comment".to_string(),
                pattern: Regex::new(r#"(?m)^\s*///.*$|^\s*#.*$|^\s*""".*?""""#).unwrap(),
                element_type: "documentation".to_string(),
                visibility_pattern: None,
            },
            ApiPattern {
                _name: "Missing Documentation".to_string(),
                pattern: Regex::new(r#"(?m)^(pub\s+)?(?:fn|class|struct)\s+\w+"#).unwrap(),
                element_type: "missing_docs".to_string(),
                visibility_pattern: Some(Regex::new(r"^pub\s+").unwrap()),
            },
        ];
        self.patterns
            .insert("documentation".to_string(), documentation_patterns);

        // Compatibility patterns
        let compatibility_patterns = vec![
            ApiPattern {
                _name: "Generic Type".to_string(),
                pattern: Regex::new(
                    r"<[A-Z]\w*(?:\s*:\s*\w+)?(?:\s*,\s*[A-Z]\w*(?:\s*:\s*\w+)?)*>",
                )
                .unwrap(),
                element_type: "generic".to_string(),
                visibility_pattern: None,
            },
            ApiPattern {
                _name: "Optional Parameter".to_string(),
                pattern: Regex::new(r"\w+\s*:\s*Option<\w+>|\w+\s*=\s*\w+").unwrap(),
                element_type: "optional".to_string(),
                visibility_pattern: None,
            },
        ];
        self.patterns
            .insert("compatibility".to_string(), compatibility_patterns);
    }

    /// Analyze API surface
    pub fn analyze_api_surface(
        &self,
        content: &str,
        analysis_types: &[String],
        include_private_apis: bool,
    ) -> Result<Vec<ApiElement>> {
        let mut elements = Vec::new();

        let target_types = if analysis_types.contains(&"all".to_string()) {
            self.patterns.keys().cloned().collect::<Vec<_>>()
        } else {
            analysis_types.to_vec()
        };

        for analysis_type in target_types {
            if let Some(patterns) = self.patterns.get(&analysis_type) {
                for pattern in patterns {
                    for captures in pattern.pattern.captures_iter(content) {
                        let full_match = captures.get(0).unwrap().as_str();
                        let name = captures
                            .get(2)
                            .or_else(|| captures.get(1))
                            .map(|m| m.as_str())
                            .unwrap_or("unknown");

                        let is_public =
                            if let Some(visibility_pattern) = &pattern.visibility_pattern {
                                visibility_pattern.is_match(full_match)
                            } else {
                                true // Assume public if no visibility pattern
                            };

                        if is_public || include_private_apis {
                            elements.push(ApiElement {
                                element_type: pattern.element_type.clone(),
                                name: name.to_string(),
                                visibility: if is_public {
                                    "public".to_string()
                                } else {
                                    "private".to_string()
                                },
                                signature: Some(full_match.to_string()),
                                documentation: self.extract_documentation(content, full_match),
                                deprecated: self.is_deprecated(content, full_match),
                                breaking_change_risk: self
                                    .assess_breaking_change_risk(&pattern.element_type),
                            });
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    /// Extract documentation for an API element
    fn extract_documentation(&self, content: &str, element: &str) -> Option<String> {
        // Look for documentation comments above the element
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(element) {
                // Look backwards for documentation
                let mut docs = Vec::new();
                for j in (0..i).rev() {
                    let prev_line = lines[j].trim();
                    if prev_line.starts_with("///")
                        || prev_line.starts_with("#")
                        || prev_line.starts_with("\"\"\"")
                    {
                        docs.insert(0, prev_line);
                    } else if !prev_line.is_empty() {
                        break;
                    }
                }
                if !docs.is_empty() {
                    return Some(docs.join("\n"));
                }
            }
        }
        None
    }

    /// Check if an element is deprecated
    fn is_deprecated(&self, content: &str, element: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(element) {
                // Look backwards for deprecation annotations
                for j in (0..i).rev() {
                    let prev_line = lines[j].trim();
                    if prev_line.contains("@deprecated")
                        || prev_line.contains("#[deprecated]")
                        || prev_line.contains("@Deprecated")
                    {
                        return true;
                    } else if !prev_line.is_empty()
                        && !prev_line.starts_with("///")
                        && !prev_line.starts_with("#")
                    {
                        break;
                    }
                }
            }
        }
        false
    }

    /// Assess breaking change risk
    fn assess_breaking_change_risk(&self, element_type: &str) -> String {
        match element_type {
            "function" | "method" => "medium".to_string(),
            "class" | "struct" => "high".to_string(),
            "constant" => "low".to_string(),
            "generic" => "high".to_string(),
            _ => "low".to_string(),
        }
    }

    /// Check if element is considered public API
    pub fn is_public_api_element(&self, name: &str) -> bool {
        // Simple heuristic - in practice, this would be more sophisticated
        !name.starts_with('_') && !name.starts_with("internal") && !name.starts_with("private")
    }

    /// Get API recommendations
    pub fn get_api_recommendations(&self, elements: &[ApiElement]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if elements.is_empty() {
            recommendations.push(
                "No API elements detected. Consider defining clear public interfaces.".to_string(),
            );
            return recommendations;
        }

        // Count different types of issues
        let public_elements = elements.iter().filter(|e| e.visibility == "public").count();
        let documented_elements = elements
            .iter()
            .filter(|e| e.documentation.is_some())
            .count();
        let deprecated_elements = elements.iter().filter(|e| e.deprecated).count();

        if public_elements > 0 {
            let documentation_coverage =
                (documented_elements as f64 / public_elements as f64) * 100.0;

            if documentation_coverage < 80.0 {
                recommendations.push(format!(
                    "API documentation coverage is {documentation_coverage:.1}%. Consider documenting more public APIs."
                ));
            }
        }

        if deprecated_elements > 0 {
            recommendations.push(format!(
                "{deprecated_elements} deprecated API elements found. Plan migration strategy for users."
            ));
        }

        // Check for high-risk breaking changes
        let high_risk_elements = elements
            .iter()
            .filter(|e| e.breaking_change_risk == "high")
            .count();
        if high_risk_elements > 0 {
            recommendations.push(format!(
                "{high_risk_elements} high-risk API elements detected. Changes to these may break compatibility."
            ));
        }

        recommendations.push("Use semantic versioning for API changes.".to_string());
        recommendations.push("Consider API versioning strategy for major changes.".to_string());
        recommendations.push("Implement API compatibility testing.".to_string());
        recommendations.push("Document API lifecycle and deprecation policies.".to_string());

        recommendations
    }

    /// Analyze public API elements
    pub fn analyze_public_api(&self, content: &str) -> Result<Vec<Value>> {
        let elements = self.analyze_api_surface(content, &["public_api".to_string()], false)?;

        Ok(elements
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "type": e.element_type,
                    "name": e.name,
                    "visibility": e.visibility,
                    "signature": e.signature,
                    "documented": e.documentation.is_some(),
                    "deprecated": e.deprecated,
                    "breaking_change_risk": e.breaking_change_risk
                })
            })
            .collect())
    }

    /// Analyze API versioning
    pub fn analyze_api_versioning(&self, content: &str) -> Result<Vec<Value>> {
        let elements = self.analyze_api_surface(content, &["versioning".to_string()], true)?;

        Ok(elements
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "type": e.element_type,
                    "name": e.name,
                    "signature": e.signature,
                    "deprecated": e.deprecated
                })
            })
            .collect())
    }

    /// Detect potential breaking changes
    pub fn detect_api_breaking_changes(&self, content: &str) -> Result<Vec<Value>> {
        let elements =
            self.analyze_api_surface(content, &["breaking_changes".to_string()], false)?;

        Ok(elements
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "type": e.element_type,
                    "name": e.name,
                    "signature": e.signature,
                    "risk_level": e.breaking_change_risk,
                    "recommendation": match e.breaking_change_risk.as_str() {
                        "high" => "Major version bump recommended",
                        "medium" => "Minor version bump may be needed",
                        _ => "Patch version acceptable"
                    }
                })
            })
            .collect())
    }

    /// Analyze documentation coverage
    pub fn analyze_api_documentation_coverage(&self, content: &str) -> Result<Vec<Value>> {
        let elements = self.analyze_api_surface(content, &["public_api".to_string()], false)?;

        let total_public = elements.len();
        let documented = elements
            .iter()
            .filter(|e| e.documentation.is_some())
            .count();
        let coverage = if total_public > 0 {
            (documented as f64 / total_public as f64) * 100.0
        } else {
            100.0
        };

        Ok(vec![serde_json::json!({
            "total_public_apis": total_public,
            "documented_apis": documented,
            "coverage_percentage": coverage,
            "undocumented_apis": elements.into_iter()
                .filter(|e| e.documentation.is_none())
                .map(|e| serde_json::json!({
                    "name": e.name,
                    "type": e.element_type,
                    "signature": e.signature
                }))
                .collect::<Vec<_>>()
        })])
    }
}

impl Default for ApiSurfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_function_detection() {
        let analyzer = ApiSurfaceAnalyzer::new();

        let code = "pub fn test_function(x: i32) -> i32 { x + 1 }";
        let elements = analyzer
            .analyze_api_surface(code, &["public_api".to_string()], false)
            .unwrap();

        assert!(!elements.is_empty());
        assert!(elements
            .iter()
            .any(|e| e.name == "test_function" && e.visibility == "public"));
    }

    #[test]
    fn test_deprecated_detection() {
        let analyzer = ApiSurfaceAnalyzer::new();

        let code = "#[deprecated]\npub fn old_function() {}";
        let elements = analyzer
            .analyze_api_surface(code, &["public_api".to_string()], false)
            .unwrap();

        assert!(!elements.is_empty());
        assert!(elements.iter().any(|e| e.deprecated));
    }

    #[test]
    fn test_documentation_extraction() {
        let analyzer = ApiSurfaceAnalyzer::new();

        let code = "/// This is a test function\npub fn documented_function() {}";
        let elements = analyzer
            .analyze_api_surface(code, &["public_api".to_string()], false)
            .unwrap();

        assert!(!elements.is_empty());
        assert!(elements.iter().any(|e| e.documentation.is_some()));
    }

    #[test]
    fn test_breaking_change_risk_assessment() {
        let analyzer = ApiSurfaceAnalyzer::new();

        assert_eq!(analyzer.assess_breaking_change_risk("class"), "high");
        assert_eq!(analyzer.assess_breaking_change_risk("function"), "medium");
        assert_eq!(analyzer.assess_breaking_change_risk("constant"), "low");
    }

    #[test]
    fn test_public_api_element_check() {
        let analyzer = ApiSurfaceAnalyzer::new();

        assert!(analyzer.is_public_api_element("public_function"));
        assert!(!analyzer.is_public_api_element("_private_function"));
        assert!(!analyzer.is_public_api_element("internal_function"));
    }

    #[test]
    fn test_api_recommendations() {
        let analyzer = ApiSurfaceAnalyzer::new();

        let elements = vec![ApiElement {
            element_type: "function".to_string(),
            name: "test".to_string(),
            visibility: "public".to_string(),
            signature: None,
            documentation: None,
            deprecated: false,
            breaking_change_risk: "medium".to_string(),
        }];

        let recommendations = analyzer.get_api_recommendations(&elements);
        assert!(!recommendations.is_empty());
    }
}
