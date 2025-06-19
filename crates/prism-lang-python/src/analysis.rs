//! Python-specific code analysis module

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Python metaclass information
#[derive(Debug, Clone)]
pub struct MetaclassInfo {
    pub name: String,
    pub metaclass_type: String,
    pub impact: String,
    pub attributes_modified: Vec<String>,
    pub methods_modified: Vec<String>,
}

/// Python decorator information
#[derive(Debug, Clone)]
pub struct DecoratorInfo {
    pub name: String,
    pub decorator_type: String,
    pub framework: Option<String>,
    pub effects: Vec<String>,
    pub is_factory: bool,
    pub parameters: Vec<String>,
}

/// Python inheritance information
#[derive(Debug, Clone)]
pub struct InheritanceInfo {
    pub class_name: String,
    pub base_classes: Vec<String>,
    pub mro: Vec<String>,
    pub has_diamond_inheritance: bool,
    pub mixins: Vec<String>,
    pub metaclass: Option<String>,
}

/// Python-specific analyzer
pub struct PythonAnalyzer {
    decorator_patterns: HashMap<String, Vec<DecoratorPattern>>,
    metaclass_patterns: HashMap<String, Vec<MetaclassPattern>>,
}

#[derive(Debug, Clone)]
struct DecoratorPattern {
    name: String,
    pattern: Regex,
    framework: Option<String>,
    effects: Vec<String>,
    is_factory: bool,
}

#[derive(Debug, Clone)]
struct MetaclassPattern {
    #[allow(dead_code)]
    name: String,
    pattern: Regex,
    impact: String,
}

impl PythonAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            decorator_patterns: HashMap::new(),
            metaclass_patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Framework decorators
        let framework_decorators = vec![
            DecoratorPattern {
                name: "Flask Route".to_string(),
                pattern: Regex::new(r"@app\.route\s*\([^)]*\)").unwrap(),
                framework: Some("Flask".to_string()),
                effects: vec!["URL routing".to_string(), "HTTP method binding".to_string()],
                is_factory: true,
            },
            DecoratorPattern {
                name: "Django View".to_string(),
                pattern: Regex::new(r"@(?:login_required|permission_required|csrf_exempt)")
                    .unwrap(),
                framework: Some("Django".to_string()),
                effects: vec!["Authentication".to_string(), "Authorization".to_string()],
                is_factory: false,
            },
            DecoratorPattern {
                name: "FastAPI Route".to_string(),
                pattern: Regex::new(r"@app\.(?:get|post|put|delete|patch)\s*\([^)]*\)").unwrap(),
                framework: Some("FastAPI".to_string()),
                effects: vec!["API endpoint".to_string(), "Request validation".to_string()],
                is_factory: true,
            },
            DecoratorPattern {
                name: "Pytest Fixture".to_string(),
                pattern: Regex::new(r"@pytest\.fixture\s*(?:\([^)]*\))?").unwrap(),
                framework: Some("pytest".to_string()),
                effects: vec!["Test setup".to_string(), "Dependency injection".to_string()],
                is_factory: true,
            },
            DecoratorPattern {
                name: "SQLAlchemy Event".to_string(),
                pattern: Regex::new(r"@event\.listens_for\s*\([^)]*\)").unwrap(),
                framework: Some("SQLAlchemy".to_string()),
                effects: vec!["Database event handling".to_string()],
                is_factory: true,
            },
            DecoratorPattern {
                name: "Celery Task".to_string(),
                pattern: Regex::new(r"@(?:celery\.)?task\s*(?:\([^)]*\))?").unwrap(),
                framework: Some("Celery".to_string()),
                effects: vec![
                    "Async task execution".to_string(),
                    "Queue processing".to_string(),
                ],
                is_factory: true,
            },
        ];
        self.decorator_patterns
            .insert("framework".to_string(), framework_decorators);

        // Pattern decorators
        let pattern_decorators = vec![
            DecoratorPattern {
                name: "Caching Decorator".to_string(),
                pattern: Regex::new(r"@(?:cache|lru_cache|memoize)").unwrap(),
                framework: None,
                effects: vec![
                    "Result caching".to_string(),
                    "Performance optimization".to_string(),
                ],
                is_factory: false,
            },
            DecoratorPattern {
                name: "Validation Decorator".to_string(),
                pattern: Regex::new(r"@(?:validate|validator|check)").unwrap(),
                framework: None,
                effects: vec!["Input validation".to_string(), "Type checking".to_string()],
                is_factory: false,
            },
            DecoratorPattern {
                name: "Authorization Decorator".to_string(),
                pattern: Regex::new(r"@(?:requires_auth|authorized|permission)").unwrap(),
                framework: None,
                effects: vec![
                    "Access control".to_string(),
                    "Security enforcement".to_string(),
                ],
                is_factory: false,
            },
            DecoratorPattern {
                name: "Logging Decorator".to_string(),
                pattern: Regex::new(r"@(?:log|trace|audit)").unwrap(),
                framework: None,
                effects: vec![
                    "Function logging".to_string(),
                    "Execution tracing".to_string(),
                ],
                is_factory: false,
            },
        ];
        self.decorator_patterns
            .insert("patterns".to_string(), pattern_decorators);

        // Metaclass patterns
        let metaclass_patterns = vec![
            MetaclassPattern {
                name: "Registry Metaclass".to_string(),
                pattern: Regex::new(r"class\s+\w+\s*\([^)]*metaclass\s*=\s*\w*Registry\w*")
                    .unwrap(),
                impact: "Automatic class registration".to_string(),
            },
            MetaclassPattern {
                name: "Singleton Metaclass".to_string(),
                pattern: Regex::new(r"class\s+\w+\s*\([^)]*metaclass\s*=\s*\w*Singleton\w*")
                    .unwrap(),
                impact: "Single instance enforcement".to_string(),
            },
            MetaclassPattern {
                name: "Attribute Injection Metaclass".to_string(),
                pattern: Regex::new(r"class\s+\w+\s*\([^)]*metaclass\s*=\s*\w*Inject\w*").unwrap(),
                impact: "Dynamic attribute injection".to_string(),
            },
            MetaclassPattern {
                name: "ORM Metaclass".to_string(),
                pattern: Regex::new(r"class\s+\w+\s*\([^)]*metaclass\s*=\s*\w*Model\w*").unwrap(),
                impact: "Database mapping and validation".to_string(),
            },
        ];
        self.metaclass_patterns
            .insert("common".to_string(), metaclass_patterns);
    }

    /// Analyze Python decorators
    pub fn analyze_decorators(&self, content: &str) -> Result<Vec<DecoratorInfo>> {
        let mut decorators = Vec::new();

        for (category, patterns) in &self.decorator_patterns {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();

                    decorators.push(DecoratorInfo {
                        name: pattern.name.clone(),
                        decorator_type: category.clone(),
                        framework: pattern.framework.clone(),
                        effects: pattern.effects.clone(),
                        is_factory: pattern.is_factory,
                        parameters: self.extract_decorator_parameters(full_match),
                    });
                }
            }
        }

        Ok(decorators)
    }

    /// Analyze Python metaclasses
    pub fn analyze_metaclasses(&self, content: &str) -> Result<Vec<MetaclassInfo>> {
        let mut metaclasses = Vec::new();

        for (category, patterns) in &self.metaclass_patterns {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();
                    let class_name = self.extract_class_name(full_match);

                    metaclasses.push(MetaclassInfo {
                        name: class_name,
                        metaclass_type: category.clone(),
                        impact: pattern.impact.clone(),
                        attributes_modified: self.find_modified_attributes(content, full_match),
                        methods_modified: self.find_modified_methods(content, full_match),
                    });
                }
            }
        }

        Ok(metaclasses)
    }

    /// Analyze Python inheritance
    pub fn analyze_inheritance(&self, content: &str) -> Result<Vec<InheritanceInfo>> {
        let mut inheritance_info = Vec::new();

        let class_pattern = Regex::new(r"class\s+(\w+)\s*\(([^)]*)\)").unwrap();

        for captures in class_pattern.captures_iter(content) {
            let class_name = captures.get(1).unwrap().as_str().to_string();
            let bases_str = captures.get(2).unwrap().as_str();

            let base_classes = self.parse_base_classes(bases_str);
            let mro = self.calculate_mro(&class_name, &base_classes);
            let has_diamond = self.detect_diamond_inheritance(&base_classes);
            let mixins = self.identify_mixins(&base_classes);
            let metaclass = self.extract_metaclass(bases_str);

            inheritance_info.push(InheritanceInfo {
                class_name,
                base_classes,
                mro,
                has_diamond_inheritance: has_diamond,
                mixins,
                metaclass,
            });
        }

        Ok(inheritance_info)
    }

    /// Extract decorator parameters
    fn extract_decorator_parameters(&self, decorator: &str) -> Vec<String> {
        let param_pattern = Regex::new(r"\(([^)]*)\)").unwrap();

        if let Some(captures) = param_pattern.captures(decorator) {
            let params_str = captures.get(1).unwrap().as_str();
            params_str
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract class name from class definition
    fn extract_class_name(&self, class_def: &str) -> String {
        let name_pattern = Regex::new(r"class\s+(\w+)").unwrap();

        if let Some(captures) = name_pattern.captures(class_def) {
            captures.get(1).unwrap().as_str().to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Find attributes modified by metaclass
    fn find_modified_attributes(&self, _content: &str, _class_def: &str) -> Vec<String> {
        // Simplified implementation - in practice, this would analyze the metaclass code
        vec!["__new__".to_string(), "__init__".to_string()]
    }

    /// Find methods modified by metaclass
    fn find_modified_methods(&self, _content: &str, _class_def: &str) -> Vec<String> {
        // Simplified implementation - in practice, this would analyze the metaclass code
        vec!["__call__".to_string()]
    }

    /// Parse base classes from inheritance declaration
    fn parse_base_classes(&self, bases_str: &str) -> Vec<String> {
        bases_str
            .split(',')
            .map(|base| {
                // Remove metaclass and other keyword arguments
                let clean_base = base.split('=').next().unwrap_or(base).trim();
                clean_base.to_string()
            })
            .filter(|base| !base.is_empty() && !base.contains("metaclass"))
            .collect()
    }

    /// Calculate Method Resolution Order (simplified)
    fn calculate_mro(&self, class_name: &str, base_classes: &[String]) -> Vec<String> {
        let mut mro = vec![class_name.to_string()];
        mro.extend(base_classes.iter().cloned());
        mro.push("object".to_string());
        mro
    }

    /// Detect diamond inheritance pattern
    fn detect_diamond_inheritance(&self, base_classes: &[String]) -> bool {
        // Simplified check - in practice, this would build the full inheritance graph
        base_classes.len() > 1
    }

    /// Identify mixin classes
    fn identify_mixins(&self, base_classes: &[String]) -> Vec<String> {
        base_classes
            .iter()
            .filter(|base| base.ends_with("Mixin") || base.ends_with("Mix"))
            .cloned()
            .collect()
    }

    /// Extract metaclass from base classes
    fn extract_metaclass(&self, bases_str: &str) -> Option<String> {
        let metaclass_pattern = Regex::new(r"metaclass\s*=\s*(\w+)").unwrap();

        metaclass_pattern.captures(bases_str).map(|captures| captures.get(1).unwrap().as_str().to_string())
    }

    /// Get Python-specific recommendations
    pub fn get_python_recommendations(
        &self,
        decorators: &[DecoratorInfo],
        metaclasses: &[MetaclassInfo],
        inheritance: &[InheritanceInfo],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Decorator recommendations
        let framework_decorators = decorators.iter().filter(|d| d.framework.is_some()).count();
        if framework_decorators > 0 {
            recommendations
                .push("Consider documenting framework-specific decorator behavior.".to_string());
        }

        let factory_decorators = decorators.iter().filter(|d| d.is_factory).count();
        if factory_decorators > 0 {
            recommendations.push(
                "Factory decorators detected - ensure proper parameter validation.".to_string(),
            );
        }

        // Metaclass recommendations
        if !metaclasses.is_empty() {
            recommendations.push(
                "Metaclasses detected - document their behavior and impact on subclasses."
                    .to_string(),
            );
            recommendations.push(
                "Consider if metaclass functionality could be achieved with simpler patterns."
                    .to_string(),
            );
        }

        // Inheritance recommendations
        let diamond_inheritance = inheritance
            .iter()
            .filter(|i| i.has_diamond_inheritance)
            .count();
        if diamond_inheritance > 0 {
            recommendations.push(
                "Diamond inheritance detected - verify MRO behavior is as expected.".to_string(),
            );
        }

        let complex_inheritance = inheritance
            .iter()
            .filter(|i| i.base_classes.len() > 2)
            .count();
        if complex_inheritance > 0 {
            recommendations.push(
                "Complex inheritance hierarchies detected - consider composition over inheritance."
                    .to_string(),
            );
        }

        recommendations
            .push("Use type hints for better code documentation and IDE support.".to_string());
        recommendations.push("Consider using dataclasses for simple data containers.".to_string());

        recommendations
    }
}

impl Default for PythonAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decorator_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = "@app.route('/test')\ndef test_view():\n    pass";
        let decorators = analyzer.analyze_decorators(code).unwrap();

        assert!(!decorators.is_empty());
        assert!(decorators.iter().any(|d| d.name == "Flask Route"));
    }

    #[test]
    fn test_metaclass_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = "class TestClass(BaseClass, metaclass=RegistryMeta):\n    pass";
        let metaclasses = analyzer.analyze_metaclasses(code).unwrap();

        assert!(!metaclasses.is_empty());
        assert!(metaclasses.iter().any(|m| m.name == "TestClass"));
    }

    #[test]
    fn test_inheritance_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = "class Child(Parent1, Parent2):\n    pass";
        let inheritance = analyzer.analyze_inheritance(code).unwrap();

        assert!(!inheritance.is_empty());
        assert_eq!(inheritance[0].class_name, "Child");
        assert_eq!(inheritance[0].base_classes.len(), 2);
    }

    #[test]
    fn test_decorator_parameter_extraction() {
        let analyzer = PythonAnalyzer::new();

        let decorator = "@app.route('/test', methods=['GET', 'POST'])";
        let params = analyzer.extract_decorator_parameters(decorator);

        assert!(!params.is_empty());
    }

    #[test]
    fn test_diamond_inheritance_detection() {
        let analyzer = PythonAnalyzer::new();

        let base_classes = vec!["Parent1".to_string(), "Parent2".to_string()];
        assert!(analyzer.detect_diamond_inheritance(&base_classes));

        let single_base = vec!["Parent".to_string()];
        assert!(!analyzer.detect_diamond_inheritance(&single_base));
    }

    #[test]
    fn test_mixin_identification() {
        let analyzer = PythonAnalyzer::new();

        let base_classes = vec![
            "BaseMixin".to_string(),
            "RegularClass".to_string(),
            "UtilMix".to_string(),
        ];
        let mixins = analyzer.identify_mixins(&base_classes);

        assert_eq!(mixins.len(), 2);
        assert!(mixins.contains(&"BaseMixin".to_string()));
        assert!(mixins.contains(&"UtilMix".to_string()));
    }
}
