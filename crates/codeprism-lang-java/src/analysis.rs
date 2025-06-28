//! Java-specific code analysis module

/// Java-specific analyzer
pub struct JavaAnalyzer {}

impl JavaAnalyzer {
    /// Create a new Java analyzer
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze Java code for common patterns and issues
    pub fn analyze_code(&self, content: &str) -> JavaAnalysisResult {
        let mut patterns = Vec::new();
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Basic pattern detection
        if content.contains("public static final") {
            patterns.push("Constants pattern detected".to_string());
        }
        if content.contains("private static") && content.contains("getInstance") {
            patterns.push("Singleton pattern detected".to_string());
        }
        if content.contains("@Override") {
            patterns.push("Method overriding detected".to_string());
        }
        if content.contains("implements") {
            patterns.push("Interface implementation detected".to_string());
        }

        // Basic issue detection
        if content.contains("System.out.print") {
            issues.push("Direct console output detected - consider using logging framework".to_string());
        }
        if content.contains("throws Exception") {
            issues.push("Generic Exception throwing detected - use specific exceptions".to_string());
        }
        if content.contains("String +") {
            issues.push("String concatenation in loop detected - consider StringBuilder".to_string());
        }

        // Basic recommendations
        if content.contains("public class") {
            recommendations.push("Consider adding JavaDoc documentation for public classes".to_string());
        }
        if content.contains("public") && !content.contains("@") {
            recommendations.push("Consider adding appropriate annotations for public APIs".to_string());
        }

        JavaAnalysisResult {
            patterns_detected: patterns,
            issues_found: issues,
            recommendations,
        }
    }
}

impl Default for JavaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of Java code analysis
#[derive(Debug)]
pub struct JavaAnalysisResult {
    /// Design patterns detected
    pub patterns_detected: Vec<String>,
    /// Issues found in the code
    pub issues_found: Vec<String>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = JavaAnalyzer::new();
        let result = analyzer.analyze_code("public class Test {}");

        // Basic test to ensure the analyzer works
        assert!(!result.recommendations.is_empty()); // Should have recommendations for public class
        assert!(result.patterns_detected.is_empty()); // Simple class shouldn't detect complex patterns
        assert!(result.issues_found.is_empty()); // Simple class shouldn't have issues
    }

    #[test]
    fn test_pattern_detection() {
        let analyzer = JavaAnalyzer::new();
        
        // Test singleton pattern detection
        let singleton_code = r#"
        public class Singleton {
            private static Singleton instance;
            private static Singleton getInstance() {
                return instance;
            }
        }
        "#;
        let result = analyzer.analyze_code(singleton_code);
        assert!(result.patterns_detected.iter().any(|p| p.contains("Singleton")));

        // Test override pattern detection
        let override_code = r#"
        public class Child extends Parent {
            @Override
            public void method() {}
        }
        "#;
        let result = analyzer.analyze_code(override_code);
        assert!(result.patterns_detected.iter().any(|p| p.contains("overriding")));
    }

    #[test]
    fn test_issue_detection() {
        let analyzer = JavaAnalyzer::new();
        
        // Test console output detection
        let console_code = "System.out.println(\"Hello\");";
        let result = analyzer.analyze_code(console_code);
        assert!(result.issues_found.iter().any(|i| i.contains("console output")));

        // Test generic exception detection
        let exception_code = "public void method() throws Exception {}";
        let result = analyzer.analyze_code(exception_code);
        assert!(result.issues_found.iter().any(|i| i.contains("Generic Exception")));
    }
}
