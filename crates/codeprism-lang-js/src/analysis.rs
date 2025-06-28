//! JavaScript/TypeScript-specific code analysis module

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// React component information
#[derive(Debug, Clone)]
pub struct ReactComponentInfo {
    pub name: String,
    pub component_type: ComponentType,
    pub hooks_used: Vec<HookInfo>,
    pub props_analysis: PropsInfo,
    pub lifecycle_methods: Vec<String>,
    pub jsx_elements: Vec<String>,
}

/// Component type classification
#[derive(Debug, Clone)]
pub enum ComponentType {
    Functional,
    Class,
    HigherOrderComponent,
    ForwardRef,
    Memo,
}

/// React hook information
#[derive(Debug, Clone)]
pub struct HookInfo {
    pub name: String,
    pub hook_type: String,
    pub dependencies: Vec<String>,
    pub custom_hook: bool,
}

/// Props analysis information
#[derive(Debug, Clone)]
pub struct PropsInfo {
    pub prop_names: Vec<String>,
    pub has_prop_types: bool,
    pub has_default_props: bool,
    pub destructured: bool,
}

/// Node.js pattern information
#[derive(Debug, Clone)]
pub struct NodeJsPatternInfo {
    pub pattern_type: NodePatternType,
    pub framework: String,
    pub route_info: Option<RouteInfo>,
    pub middleware_chain: Vec<String>,
    pub http_methods: Vec<String>,
}

/// Node.js pattern types
#[derive(Debug, Clone)]
pub enum NodePatternType {
    ExpressRoute,
    ExpressMiddleware,
    FastifyRoute,
    KoaRoute,
    DatabaseQuery,
    ErrorHandler,
    AuthMiddleware,
}

/// Route information
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: String,
    pub method: String,
    pub parameters: Vec<String>,
    pub query_params: Vec<String>,
}

/// Modern JavaScript feature information
#[derive(Debug, Clone)]
pub struct ModernJsFeatureInfo {
    pub feature_type: ModernFeatureType,
    pub usage_pattern: String,
    pub complexity_score: i32,
    pub best_practices: Vec<String>,
}

/// Modern JavaScript feature types
#[derive(Debug, Clone)]
pub enum ModernFeatureType {
    AsyncAwait,
    Destructuring,
    SpreadOperator,
    ArrowFunction,
    TemplateString,
    DynamicImport,
    OptionalChaining,
    NullishCoalescing,
    ClassFields,
    Decorator,
}

/// JavaScript/TypeScript-specific analyzer
pub struct JavaScriptAnalyzer {
    framework_patterns: HashMap<String, Regex>,
    react_patterns: HashMap<String, Regex>,
    nodejs_patterns: HashMap<String, Regex>,
}

impl JavaScriptAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            framework_patterns: HashMap::new(),
            react_patterns: HashMap::new(),
            nodejs_patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Framework detection patterns
        self.framework_patterns.insert(
            "React".to_string(),
            Regex::new(r"import React|from.*react").unwrap(),
        );
        self.framework_patterns.insert(
            "Express".to_string(),
            Regex::new(r"express\(\)|app\.\w+\(").unwrap(),
        );

        // React patterns
        self.react_patterns.insert(
            "useState".to_string(),
            Regex::new(r"useState\s*\(").unwrap(),
        );
        self.react_patterns.insert(
            "useEffect".to_string(),
            Regex::new(r"useEffect\s*\(").unwrap(),
        );

        // Node.js patterns
        self.nodejs_patterns.insert(
            "route".to_string(),
            Regex::new(r"app\.(get|post|put|delete)\s*\(").unwrap(),
        );
    }

    /// Detect framework usage
    pub fn detect_frameworks(&self, content: &str) -> Result<Vec<(String, f32)>> {
        let mut frameworks = Vec::new();

        for (framework_name, pattern) in &self.framework_patterns {
            if pattern.is_match(content) {
                let confidence = match framework_name.as_str() {
                    "React" => 0.9,
                    "Express" => 0.85,
                    _ => 0.5,
                };
                frameworks.push((framework_name.clone(), confidence));
            }
        }

        Ok(frameworks)
    }

    /// Analyze React components and patterns
    pub fn analyze_react_patterns(&self, content: &str) -> Result<Vec<ReactComponentInfo>> {
        let mut components = Vec::new();
        let functional_component_pattern = Regex::new(r"function\s+([A-Z]\w*)\s*\(")?;

        for captures in functional_component_pattern.captures_iter(content) {
            let component_name = captures.get(1).unwrap().as_str().to_string();
            let mut hooks_used = Vec::new();

            if self
                .react_patterns
                .get("useState")
                .unwrap()
                .is_match(content)
            {
                hooks_used.push(HookInfo {
                    name: "useState".to_string(),
                    hook_type: "state".to_string(),
                    dependencies: Vec::new(),
                    custom_hook: false,
                });
            }

            components.push(ReactComponentInfo {
                name: component_name,
                component_type: ComponentType::Functional,
                hooks_used,
                props_analysis: PropsInfo {
                    prop_names: Vec::new(),
                    has_prop_types: false,
                    has_default_props: false,
                    destructured: false,
                },
                lifecycle_methods: Vec::new(),
                jsx_elements: Vec::new(),
            });
        }

        Ok(components)
    }

    /// Analyze Node.js patterns
    pub fn analyze_nodejs_patterns(&self, content: &str) -> Result<Vec<NodeJsPatternInfo>> {
        let mut patterns = Vec::new();

        if self.nodejs_patterns.get("route").unwrap().is_match(content) {
            patterns.push(NodeJsPatternInfo {
                pattern_type: NodePatternType::ExpressRoute,
                framework: "Express".to_string(),
                route_info: None,
                middleware_chain: Vec::new(),
                http_methods: vec!["GET".to_string(), "POST".to_string()],
            });
        }

        Ok(patterns)
    }

    /// Analyze modern JavaScript features
    pub fn analyze_modern_js_features(&self, content: &str) -> Result<Vec<ModernJsFeatureInfo>> {
        let mut features = Vec::new();
        let async_pattern = Regex::new(r"async\s+function|await\s+")?;

        if async_pattern.is_match(content) {
            features.push(ModernJsFeatureInfo {
                feature_type: ModernFeatureType::AsyncAwait,
                usage_pattern: "async/await".to_string(),
                complexity_score: 3,
                best_practices: vec!["Handle errors with try/catch".to_string()],
            });
        }

        Ok(features)
    }
}

impl Default for JavaScriptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_detection() {
        let analyzer = JavaScriptAnalyzer::new();
        let react_code = "import React from 'react';";
        let frameworks = analyzer.detect_frameworks(react_code).unwrap();
        assert!(frameworks.iter().any(|(name, _)| name == "React"));
    }

    #[test]
    fn test_react_component_detection() {
        let analyzer = JavaScriptAnalyzer::new();
        let code = "function MyComponent(props) { const [state, setState] = useState(0); return <div>Hello</div>; }";
        let components = analyzer.analyze_react_patterns(code).unwrap();
        assert!(!components.is_empty());
        assert_eq!(components[0].name, "MyComponent");
        assert!(matches!(
            components[0].component_type,
            ComponentType::Functional
        ));
    }
}
