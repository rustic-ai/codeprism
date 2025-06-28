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
    pub context_usage: Vec<ContextInfo>,
    pub state_management: Vec<StateManagementInfo>,
}

/// Component type classification
#[derive(Debug, Clone)]
pub enum ComponentType {
    Functional,
    Class,
    HigherOrderComponent,
    ForwardRef,
    Memo,
    CustomHook,
}

/// React hook information
#[derive(Debug, Clone)]
pub struct HookInfo {
    pub name: String,
    pub hook_type: String,
    pub dependencies: Vec<String>,
    pub custom_hook: bool,
}

/// Context usage information
#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub context_name: String,
    pub usage_type: String, // "provider", "consumer", "useContext"
    pub values_consumed: Vec<String>,
}

/// State management information
#[derive(Debug, Clone)]
pub struct StateManagementInfo {
    pub pattern_type: String, // "useState", "useReducer", "redux", "zustand"
    pub state_variables: Vec<String>,
    pub actions: Vec<String>,
}

/// Props analysis information
#[derive(Debug, Clone)]
pub struct PropsInfo {
    pub prop_names: Vec<String>,
    pub has_prop_types: bool,
    pub has_default_props: bool,
    pub destructured: bool,
    pub typescript_props: bool,
}

/// Node.js pattern information
#[derive(Debug, Clone)]
pub struct NodeJsPatternInfo {
    pub pattern_type: NodePatternType,
    pub framework: String,
    pub route_info: Option<RouteInfo>,
    pub middleware_chain: Vec<String>,
    pub http_methods: Vec<String>,
    pub database_patterns: Vec<DatabasePatternInfo>,
}

/// Database pattern information
#[derive(Debug, Clone)]
pub struct DatabasePatternInfo {
    pub db_type: String,               // "mongodb", "postgresql", "mysql", "redis"
    pub operations: Vec<String>,       // "find", "create", "update", "delete"
    pub orm_framework: Option<String>, // "mongoose", "prisma", "typeorm"
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
    ApiEndpoint,
    WebSocketHandler,
}

/// Route information
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: String,
    pub method: String,
    pub parameters: Vec<String>,
    pub query_params: Vec<String>,
    pub middleware_used: Vec<String>,
}

/// Modern JavaScript feature information
#[derive(Debug, Clone)]
pub struct ModernJsFeatureInfo {
    pub feature_type: ModernFeatureType,
    pub usage_pattern: String,
    pub complexity_score: i32,
    pub best_practices: Vec<String>,
    pub typescript_specific: bool,
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
    TypeScriptInterface,
    TypeScriptEnum,
    TypeScriptGenerics,
}

/// Framework analysis information
#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    pub name: String,
    pub confidence: f32,
    pub version_detected: Option<String>,
    pub features_used: Vec<String>,
    pub best_practices: Vec<String>,
}

/// JavaScript/TypeScript-specific analyzer
pub struct JavaScriptAnalyzer {
    framework_patterns: HashMap<String, Regex>,
    react_patterns: HashMap<String, Regex>,
    nodejs_patterns: HashMap<String, Regex>,
    typescript_patterns: HashMap<String, Regex>,
    #[allow(dead_code)]
    vue_patterns: HashMap<String, Regex>,
    #[allow(dead_code)]
    angular_patterns: HashMap<String, Regex>,
}

impl JavaScriptAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            framework_patterns: HashMap::new(),
            react_patterns: HashMap::new(),
            nodejs_patterns: HashMap::new(),
            typescript_patterns: HashMap::new(),
            vue_patterns: HashMap::new(),
            angular_patterns: HashMap::new(),
        };
        analyzer.initialize_patterns();
        analyzer
    }

    fn initialize_patterns(&mut self) {
        // Framework detection patterns
        self.framework_patterns.insert(
            "React".to_string(),
            Regex::new(r"import React|from.*react|jsx|\.jsx").unwrap(),
        );
        self.framework_patterns.insert(
            "Express".to_string(),
            Regex::new(r"express\(\)|app\.\w+\(").unwrap(),
        );
        self.framework_patterns.insert(
            "Vue".to_string(),
            Regex::new(r"Vue\.|new Vue|@Component|\.vue|v-if|v-for").unwrap(),
        );
        self.framework_patterns.insert(
            "Angular".to_string(),
            Regex::new(r"@Component|@Injectable|@NgModule|angular|\.component\.ts").unwrap(),
        );
        self.framework_patterns.insert(
            "Next.js".to_string(),
            Regex::new(r"next/|getStaticProps|getServerSideProps|pages/").unwrap(),
        );

        // React patterns (expanded)
        self.react_patterns.insert(
            "useState".to_string(),
            Regex::new(r"useState\s*\(").unwrap(),
        );
        self.react_patterns.insert(
            "useEffect".to_string(),
            Regex::new(r"useEffect\s*\(").unwrap(),
        );
        self.react_patterns.insert(
            "useContext".to_string(),
            Regex::new(r"useContext\s*\(").unwrap(),
        );
        self.react_patterns.insert(
            "useReducer".to_string(),
            Regex::new(r"useReducer\s*\(").unwrap(),
        );
        self.react_patterns
            .insert("useMemo".to_string(), Regex::new(r"useMemo\s*\(").unwrap());
        self.react_patterns.insert(
            "useCallback".to_string(),
            Regex::new(r"useCallback\s*\(").unwrap(),
        );
        self.react_patterns
            .insert("useRef".to_string(), Regex::new(r"useRef\s*\(").unwrap());
        self.react_patterns.insert(
            "custom_hook".to_string(),
            Regex::new(r"function\s+use[A-Z]\w*|const\s+use[A-Z]\w*\s*=").unwrap(),
        );
        self.react_patterns.insert(
            "context_provider".to_string(),
            Regex::new(r"\.Provider|createContext|React\.createContext").unwrap(),
        );
        self.react_patterns.insert(
            "hoc".to_string(),
            Regex::new(r"function\s+with[A-Z]\w*|const\s+with[A-Z]\w*").unwrap(),
        );

        // Node.js patterns (expanded)
        self.nodejs_patterns.insert(
            "route".to_string(),
            Regex::new(r"app\.(get|post|put|delete|patch)\s*\(").unwrap(),
        );
        self.nodejs_patterns.insert(
            "middleware".to_string(),
            Regex::new(r"app\.use\s*\(|function\s*\(\s*req\s*,\s*res\s*,\s*next\s*\)").unwrap(),
        );
        self.nodejs_patterns.insert(
            "mongodb".to_string(),
            Regex::new(r"mongoose\.|\.find\(|\.findOne\(|\.save\(|\.updateOne\(").unwrap(),
        );
        self.nodejs_patterns.insert(
            "postgresql".to_string(),
            Regex::new(r"pg\.|pool\.query|client\.query|SELECT\s+\*\s+FROM").unwrap(),
        );
        self.nodejs_patterns.insert(
            "redis".to_string(),
            Regex::new(r"redis\.|\.get\(|\.set\(|\.hget\(|\.hset\(").unwrap(),
        );

        // TypeScript patterns
        self.typescript_patterns.insert(
            "interface".to_string(),
            Regex::new(r"interface\s+[A-Z]\w*").unwrap(),
        );
        self.typescript_patterns.insert(
            "type_alias".to_string(),
            Regex::new(r"type\s+[A-Z]\w*\s*=").unwrap(),
        );
        self.typescript_patterns
            .insert("enum".to_string(), Regex::new(r"enum\s+[A-Z]\w*").unwrap());
        self.typescript_patterns.insert(
            "generics".to_string(),
            Regex::new(r"<[A-Z]\w*(?:\s*,\s*[A-Z]\w*)*>").unwrap(),
        );
        self.typescript_patterns
            .insert("decorators".to_string(), Regex::new(r"@\w+\s*\(").unwrap());
    }

    /// Detect framework usage with enhanced analysis
    pub fn detect_frameworks(&self, content: &str) -> Result<Vec<FrameworkInfo>> {
        let mut frameworks = Vec::new();

        for (framework_name, pattern) in &self.framework_patterns {
            if pattern.is_match(content) {
                let (confidence, features) = self.analyze_framework_usage(framework_name, content);

                frameworks.push(FrameworkInfo {
                    name: framework_name.clone(),
                    confidence,
                    version_detected: self.detect_framework_version(framework_name, content),
                    features_used: features,
                    best_practices: self.get_framework_recommendations(framework_name),
                });
            }
        }

        Ok(frameworks)
    }

    fn analyze_framework_usage(&self, framework: &str, content: &str) -> (f32, Vec<String>) {
        let mut confidence: f32 = match framework {
            "React" => 0.7,
            "Express" => 0.7,
            "Vue" => 0.7,
            "Angular" => 0.7,
            "Next.js" => 0.7,
            _ => 0.5,
        };

        let mut features = Vec::new();

        match framework {
            "React" => {
                if content.contains("useState") {
                    confidence += 0.1;
                    features.push("Hooks".to_string());
                }
                if content.contains("JSX") || content.contains("<") {
                    confidence += 0.1;
                    features.push("JSX".to_string());
                }
                if content.contains("Component") {
                    confidence += 0.05;
                    features.push("Components".to_string());
                }
            }
            "Express" => {
                if content.contains("app.get") || content.contains("app.post") {
                    confidence += 0.1;
                    features.push("Routing".to_string());
                }
                if content.contains("middleware") {
                    confidence += 0.05;
                    features.push("Middleware".to_string());
                }
            }
            "Vue" => {
                if content.contains("v-if") || content.contains("v-for") {
                    confidence += 0.1;
                    features.push("Directives".to_string());
                }
                if content.contains("@Component") {
                    confidence += 0.1;
                    features.push("Components".to_string());
                }
            }
            "Angular" => {
                if content.contains("@Component") {
                    confidence += 0.15;
                    features.push("Components".to_string());
                }
                if content.contains("@Injectable") {
                    confidence += 0.1;
                    features.push("Services".to_string());
                }
            }
            _ => {}
        }

        (confidence.min(1.0), features)
    }

    fn detect_framework_version(&self, _framework: &str, _content: &str) -> Option<String> {
        // Simplified version detection - in practice would parse package.json or imports
        None
    }

    fn get_framework_recommendations(&self, framework: &str) -> Vec<String> {
        match framework {
            "React" => vec![
                "Use functional components with hooks".to_string(),
                "Implement proper error boundaries".to_string(),
                "Use React.memo for performance optimization".to_string(),
            ],
            "Express" => vec![
                "Add proper error handling middleware".to_string(),
                "Implement request validation".to_string(),
                "Use helmet for security headers".to_string(),
            ],
            "Vue" => vec![
                "Use composition API for better type safety".to_string(),
                "Implement proper component naming conventions".to_string(),
            ],
            "Angular" => vec![
                "Use OnPush change detection strategy".to_string(),
                "Implement proper dependency injection".to_string(),
                "Use reactive forms for complex forms".to_string(),
            ],
            _ => vec!["Follow framework best practices".to_string()],
        }
    }

    /// Analyze React components and patterns with enhanced capabilities
    pub fn analyze_react_patterns(&self, content: &str) -> Result<Vec<ReactComponentInfo>> {
        let mut components = Vec::new();
        let functional_component_pattern =
            Regex::new(r"function\s+([A-Z]\w*)\s*\(|const\s+([A-Z]\w*)\s*=\s*\([^)]*\)\s*=>")?;
        let class_component_pattern =
            Regex::new(r"class\s+([A-Z]\w*)\s+extends\s+(?:React\.)?Component")?;

        // Analyze functional components
        for captures in functional_component_pattern.captures_iter(content) {
            let component_name = captures
                .get(1)
                .or_else(|| captures.get(2))
                .unwrap()
                .as_str()
                .to_string();

            let hooks_used = self.analyze_hooks_usage(content)?;
            let context_usage = self.analyze_context_usage(content)?;
            let state_management = self.analyze_state_management(content)?;
            let jsx_elements = self.extract_jsx_elements(content)?;

            components.push(ReactComponentInfo {
                name: component_name,
                component_type: if content.contains(&format!("use{}", "")) {
                    ComponentType::CustomHook
                } else {
                    ComponentType::Functional
                },
                hooks_used,
                props_analysis: self.analyze_props(content)?,
                lifecycle_methods: Vec::new(), // Functional components don't have lifecycle methods
                jsx_elements,
                context_usage,
                state_management,
            });
        }

        // Analyze class components
        for captures in class_component_pattern.captures_iter(content) {
            let component_name = captures.get(1).unwrap().as_str().to_string();
            let lifecycle_methods = self.detect_lifecycle_methods(content)?;
            let jsx_elements = self.extract_jsx_elements(content)?;

            components.push(ReactComponentInfo {
                name: component_name,
                component_type: ComponentType::Class,
                hooks_used: Vec::new(), // Class components don't use hooks
                props_analysis: self.analyze_props(content)?,
                lifecycle_methods,
                jsx_elements,
                context_usage: Vec::new(), // Simplified for class components
                state_management: Vec::new(), // Simplified for class components
            });
        }

        Ok(components)
    }

    fn analyze_hooks_usage(&self, content: &str) -> Result<Vec<HookInfo>> {
        let mut hooks = Vec::new();

        let hook_types = vec![
            ("useState", "state"),
            ("useEffect", "effect"),
            ("useContext", "context"),
            ("useReducer", "reducer"),
            ("useMemo", "memoization"),
            ("useCallback", "callback"),
            ("useRef", "reference"),
        ];

        for (hook_name, hook_type) in hook_types {
            if let Some(pattern) = self.react_patterns.get(hook_name) {
                if pattern.is_match(content) {
                    let dependencies = self.extract_hook_dependencies(content, hook_name);
                    hooks.push(HookInfo {
                        name: hook_name.to_string(),
                        hook_type: hook_type.to_string(),
                        dependencies,
                        custom_hook: false,
                    });
                }
            }
        }

        // Detect custom hooks
        if let Some(custom_hook_pattern) = self.react_patterns.get("custom_hook") {
            for _captures in custom_hook_pattern.captures_iter(content) {
                hooks.push(HookInfo {
                    name: "custom_hook".to_string(),
                    hook_type: "custom".to_string(),
                    dependencies: Vec::new(),
                    custom_hook: true,
                });
            }
        }

        Ok(hooks)
    }

    fn analyze_context_usage(&self, content: &str) -> Result<Vec<ContextInfo>> {
        let mut context_usage = Vec::new();

        let context_provider_pattern = Regex::new(r"(\w+)\.Provider")?;
        let use_context_pattern = Regex::new(r"useContext\s*\(\s*(\w+)\s*\)")?;
        let _create_context_pattern = Regex::new(r"createContext\s*\(\s*([^)]*)\s*\)")?;

        // Detect context providers
        for captures in context_provider_pattern.captures_iter(content) {
            let context_name = captures.get(1).unwrap().as_str().to_string();
            context_usage.push(ContextInfo {
                context_name,
                usage_type: "provider".to_string(),
                values_consumed: Vec::new(),
            });
        }

        // Detect useContext usage
        for captures in use_context_pattern.captures_iter(content) {
            let context_name = captures.get(1).unwrap().as_str().to_string();
            context_usage.push(ContextInfo {
                context_name,
                usage_type: "useContext".to_string(),
                values_consumed: Vec::new(),
            });
        }

        Ok(context_usage)
    }

    fn analyze_state_management(&self, content: &str) -> Result<Vec<StateManagementInfo>> {
        let mut state_management = Vec::new();

        // Redux patterns
        if content.contains("useSelector")
            || content.contains("useDispatch")
            || content.contains("connect")
        {
            state_management.push(StateManagementInfo {
                pattern_type: "redux".to_string(),
                state_variables: self.extract_redux_state_variables(content),
                actions: self.extract_redux_actions(content),
            });
        }

        // Zustand patterns
        if content.contains("create(") && content.contains("useStore") {
            state_management.push(StateManagementInfo {
                pattern_type: "zustand".to_string(),
                state_variables: Vec::new(),
                actions: Vec::new(),
            });
        }

        Ok(state_management)
    }

    fn extract_hook_dependencies(&self, content: &str, hook_name: &str) -> Vec<String> {
        let pattern = format!(r"{}[^;]*?\[([^\]]*)\]", hook_name);
        if let Ok(regex) = Regex::new(&pattern) {
            if let Some(captures) = regex.captures(content) {
                let deps = captures.get(1).unwrap().as_str();
                return deps
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    fn extract_redux_state_variables(&self, _content: &str) -> Vec<String> {
        // Simplified implementation
        Vec::new()
    }

    fn extract_redux_actions(&self, _content: &str) -> Vec<String> {
        // Simplified implementation
        Vec::new()
    }

    fn detect_lifecycle_methods(&self, content: &str) -> Result<Vec<String>> {
        let lifecycle_methods = vec![
            "componentDidMount",
            "componentDidUpdate",
            "componentWillUnmount",
            "componentDidCatch",
            "getSnapshotBeforeUpdate",
            "getDerivedStateFromProps",
            "shouldComponentUpdate",
            "render",
            "constructor",
        ];

        let mut found_methods = Vec::new();
        for method in lifecycle_methods {
            let pattern = format!(r"{}s*\(", method);
            if Regex::new(&pattern)?.is_match(content) {
                found_methods.push(method.to_string());
            }
        }

        Ok(found_methods)
    }

    fn analyze_props(&self, content: &str) -> Result<PropsInfo> {
        let destructuring_pattern = Regex::new(r"\{\s*([^}]+)\s*\}\s*=\s*props")?;
        let prop_types_pattern = Regex::new(r"\.propTypes\s*=")?;
        let typescript_props_pattern = Regex::new(r":\s*\w+Props")?;

        let mut prop_names = Vec::new();
        let destructured = destructuring_pattern.is_match(content);

        if let Some(captures) = destructuring_pattern.captures(content) {
            let props_str = captures.get(1).unwrap().as_str();
            prop_names = props_str
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
        }

        Ok(PropsInfo {
            prop_names,
            has_prop_types: prop_types_pattern.is_match(content),
            has_default_props: false,
            destructured,
            typescript_props: typescript_props_pattern.is_match(content),
        })
    }

    fn extract_jsx_elements(&self, content: &str) -> Result<Vec<String>> {
        let jsx_pattern = Regex::new(r"<([A-Z]\w*)")?;
        let mut elements = Vec::new();

        for captures in jsx_pattern.captures_iter(content) {
            let element = captures.get(1).unwrap().as_str().to_string();
            if !elements.contains(&element) {
                elements.push(element);
            }
        }

        Ok(elements)
    }

    /// Analyze Node.js patterns with enhanced database detection
    pub fn analyze_nodejs_patterns(&self, content: &str) -> Result<Vec<NodeJsPatternInfo>> {
        let mut patterns = Vec::new();

        if self.nodejs_patterns.get("route").unwrap().is_match(content) {
            let database_patterns = self.analyze_database_patterns(content)?;

            patterns.push(NodeJsPatternInfo {
                pattern_type: NodePatternType::ExpressRoute,
                framework: "Express".to_string(),
                route_info: self.extract_route_info(content),
                middleware_chain: self.extract_middleware_chain(content),
                http_methods: vec!["GET".to_string(), "POST".to_string()],
                database_patterns,
            });
        }

        if self
            .nodejs_patterns
            .get("middleware")
            .unwrap()
            .is_match(content)
        {
            patterns.push(NodeJsPatternInfo {
                pattern_type: NodePatternType::ExpressMiddleware,
                framework: "Express".to_string(),
                route_info: None,
                middleware_chain: Vec::new(),
                http_methods: Vec::new(),
                database_patterns: Vec::new(),
            });
        }

        Ok(patterns)
    }

    fn analyze_database_patterns(&self, content: &str) -> Result<Vec<DatabasePatternInfo>> {
        let mut db_patterns = Vec::new();

        // MongoDB/Mongoose patterns
        if self
            .nodejs_patterns
            .get("mongodb")
            .unwrap()
            .is_match(content)
        {
            db_patterns.push(DatabasePatternInfo {
                db_type: "mongodb".to_string(),
                operations: self.extract_db_operations(content, "mongodb"),
                orm_framework: if content.contains("mongoose") {
                    Some("mongoose".to_string())
                } else {
                    None
                },
            });
        }

        // PostgreSQL patterns
        if self
            .nodejs_patterns
            .get("postgresql")
            .unwrap()
            .is_match(content)
        {
            db_patterns.push(DatabasePatternInfo {
                db_type: "postgresql".to_string(),
                operations: self.extract_db_operations(content, "postgresql"),
                orm_framework: self.detect_orm_framework(content),
            });
        }

        // Redis patterns
        if self.nodejs_patterns.get("redis").unwrap().is_match(content) {
            db_patterns.push(DatabasePatternInfo {
                db_type: "redis".to_string(),
                operations: self.extract_db_operations(content, "redis"),
                orm_framework: None,
            });
        }

        Ok(db_patterns)
    }

    fn extract_db_operations(&self, content: &str, db_type: &str) -> Vec<String> {
        let mut operations = Vec::new();

        let patterns = match db_type {
            "mongodb" => vec![
                "find",
                "findOne",
                "save",
                "updateOne",
                "deleteOne",
                "insertOne",
            ],
            "postgresql" => vec!["SELECT", "INSERT", "UPDATE", "DELETE"],
            "redis" => vec!["get", "set", "hget", "hset", "lpush", "rpop"],
            _ => vec![],
        };

        for operation in patterns {
            if content.contains(operation) {
                operations.push(operation.to_string());
            }
        }

        operations
    }

    fn detect_orm_framework(&self, content: &str) -> Option<String> {
        if content.contains("prisma") {
            Some("prisma".to_string())
        } else if content.contains("typeorm") {
            Some("typeorm".to_string())
        } else if content.contains("sequelize") {
            Some("sequelize".to_string())
        } else {
            None
        }
    }

    fn extract_route_info(&self, content: &str) -> Option<RouteInfo> {
        // Simplified approach - just extract method from app.method() pattern
        let route_pattern = Regex::new(r"app\.(\w+)\s*\(").unwrap();

        if let Some(captures) = route_pattern.captures(content) {
            let method = captures.get(1).unwrap().as_str().to_uppercase();

            Some(RouteInfo {
                path: "/unknown".to_string(), // Simplified - would need more complex parsing for actual path
                method,
                parameters: Vec::new(),
                query_params: Vec::new(),
                middleware_used: Vec::new(),
            })
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn extract_route_parameters(&self, path: &str) -> Vec<String> {
        let param_pattern = Regex::new(r":(\w+)").unwrap();
        param_pattern
            .captures_iter(path)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect()
    }

    fn extract_middleware_chain(&self, _content: &str) -> Vec<String> {
        // Simplified implementation - would analyze middleware usage in real implementation
        Vec::new()
    }

    /// Analyze modern JavaScript features with TypeScript support
    pub fn analyze_modern_js_features(&self, content: &str) -> Result<Vec<ModernJsFeatureInfo>> {
        let mut features = Vec::new();

        // Async/await
        let async_pattern = Regex::new(r"async\s+function|await\s+")?;
        if async_pattern.is_match(content) {
            features.push(ModernJsFeatureInfo {
                feature_type: ModernFeatureType::AsyncAwait,
                usage_pattern: "async/await".to_string(),
                complexity_score: 3,
                best_practices: vec!["Handle errors with try/catch".to_string()],
                typescript_specific: false,
            });
        }

        // Destructuring
        let destructuring_pattern = Regex::new(r"(?:const|let|var)\s*\{[^}]+\}\s*=")?;
        if destructuring_pattern.is_match(content) {
            features.push(ModernJsFeatureInfo {
                feature_type: ModernFeatureType::Destructuring,
                usage_pattern: "object/array destructuring".to_string(),
                complexity_score: 2,
                best_practices: vec!["Use default values for optional properties".to_string()],
                typescript_specific: false,
            });
        }

        // Optional chaining
        let optional_chaining_pattern = Regex::new(r"\?\.")?;
        if optional_chaining_pattern.is_match(content) {
            features.push(ModernJsFeatureInfo {
                feature_type: ModernFeatureType::OptionalChaining,
                usage_pattern: "optional chaining operator".to_string(),
                complexity_score: 2,
                best_practices: vec!["Prefer over manual null checks".to_string()],
                typescript_specific: false,
            });
        }

        // TypeScript-specific features
        if let Some(interface_pattern) = self.typescript_patterns.get("interface") {
            if interface_pattern.is_match(content) {
                features.push(ModernJsFeatureInfo {
                    feature_type: ModernFeatureType::TypeScriptInterface,
                    usage_pattern: "TypeScript interfaces".to_string(),
                    complexity_score: 3,
                    best_practices: vec!["Use interfaces for object shape definitions".to_string()],
                    typescript_specific: true,
                });
            }
        }

        if let Some(enum_pattern) = self.typescript_patterns.get("enum") {
            if enum_pattern.is_match(content) {
                features.push(ModernJsFeatureInfo {
                    feature_type: ModernFeatureType::TypeScriptEnum,
                    usage_pattern: "TypeScript enums".to_string(),
                    complexity_score: 2,
                    best_practices: vec![
                        "Consider const assertions for better performance".to_string()
                    ],
                    typescript_specific: true,
                });
            }
        }

        Ok(features)
    }

    /// Get comprehensive recommendations based on analysis
    pub fn get_comprehensive_recommendations(
        &self,
        frameworks: &[FrameworkInfo],
        components: &[ReactComponentInfo],
        nodejs_patterns: &[NodeJsPatternInfo],
        modern_features: &[ModernJsFeatureInfo],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Framework-specific recommendations
        for framework in frameworks {
            recommendations.extend(framework.best_practices.clone());
        }

        // React-specific recommendations
        if !components.is_empty() {
            let functional_count = components
                .iter()
                .filter(|c| matches!(c.component_type, ComponentType::Functional))
                .count();
            let class_count = components
                .iter()
                .filter(|c| matches!(c.component_type, ComponentType::Class))
                .count();

            if class_count > functional_count {
                recommendations.push("Consider migrating class components to functional components with hooks for better performance and maintainability".to_string());
            }

            let components_without_props = components
                .iter()
                .filter(|c| !c.props_analysis.typescript_props && !c.props_analysis.has_prop_types)
                .count();
            if components_without_props > 0 {
                recommendations.push("Add TypeScript interfaces or PropTypes for better type safety and development experience".to_string());
            }

            // Hook-specific recommendations
            for component in components {
                if component
                    .hooks_used
                    .iter()
                    .any(|h| h.name == "useEffect" && h.dependencies.is_empty())
                {
                    recommendations.push(
                        "Specify dependencies array for useEffect to prevent infinite re-renders"
                            .to_string(),
                    );
                    break;
                }
            }
        }

        // Node.js recommendations
        for pattern in nodejs_patterns {
            if pattern.database_patterns.is_empty()
                && matches!(pattern.pattern_type, NodePatternType::ExpressRoute)
            {
                recommendations.push(
                    "Consider adding database integration for persistent data storage".to_string(),
                );
            }

            if pattern.middleware_chain.is_empty() {
                recommendations.push(
                    "Add authentication and validation middleware to secure API endpoints"
                        .to_string(),
                );
            }
        }

        // Modern JavaScript recommendations
        let async_usage = modern_features
            .iter()
            .any(|f| matches!(f.feature_type, ModernFeatureType::AsyncAwait));
        if async_usage {
            recommendations.push(
                "Ensure proper error handling with try/catch blocks for async operations"
                    .to_string(),
            );
        }

        let typescript_usage = modern_features.iter().any(|f| f.typescript_specific);
        if !typescript_usage {
            recommendations.push(
                "Consider adopting TypeScript for better type safety and developer experience"
                    .to_string(),
            );
        }

        // General recommendations
        recommendations
            .push("Use ESLint and Prettier for consistent code formatting and quality".to_string());
        recommendations
            .push("Implement comprehensive testing with unit and integration tests".to_string());
        recommendations.push(
            "Consider using a state management solution for complex applications".to_string(),
        );

        recommendations
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

        let react_code = "import React from 'react'; function App() { return <div>Hello</div>; }";
        let frameworks = analyzer.detect_frameworks(react_code).unwrap();
        assert!(frameworks.iter().any(|f| f.name == "React"));
        assert!(frameworks[0].confidence > 0.7);

        let express_code =
            "const express = require('express'); app.get('/api/users', (req, res) => {});";
        let frameworks = analyzer.detect_frameworks(express_code).unwrap();
        assert!(frameworks.iter().any(|f| f.name == "Express"));
    }

    #[test]
    fn test_react_component_detection() {
        let analyzer = JavaScriptAnalyzer::new();

        let code = "function MyComponent(props) { const [state, setState] = useState(0); const data = useContext(MyContext); return <div>Hello</div>; }";
        let components = analyzer.analyze_react_patterns(code).unwrap();

        assert!(!components.is_empty());
        assert_eq!(components[0].name, "MyComponent");
        assert!(matches!(
            components[0].component_type,
            ComponentType::Functional | ComponentType::CustomHook
        ));
        assert!(!components[0].hooks_used.is_empty());
        assert!(!components[0].context_usage.is_empty());
    }

    #[test]
    fn test_advanced_hooks_detection() {
        let analyzer = JavaScriptAnalyzer::new();

        let code = r#"
            function MyComponent() {
                const [count, setCount] = useState(0);
                const memoizedValue = useMemo(() => expensiveCalculation(count), [count]);
                const callback = useCallback(() => doSomething(count), [count]);
                const ref = useRef(null);
                
                useEffect(() => {
                    console.log('Effect runs');
                }, [count]);
                
                return <div ref={ref}>{memoizedValue}</div>;
            }
        "#;

        let components = analyzer.analyze_react_patterns(code).unwrap();
        assert!(!components.is_empty());

        let hooks = &components[0].hooks_used;
        assert!(hooks.iter().any(|h| h.name == "useState"));
        assert!(hooks.iter().any(|h| h.name == "useMemo"));
        assert!(hooks.iter().any(|h| h.name == "useCallback"));
        assert!(hooks.iter().any(|h| h.name == "useEffect"));
        assert!(hooks.iter().any(|h| h.name == "useRef"));
    }

    #[test]
    fn test_database_pattern_detection() {
        let analyzer = JavaScriptAnalyzer::new();

        let mongodb_code = r#"
            app.get('/users', async (req, res) => {
                const users = await User.find({});
                res.json(users);
            });
        "#;

        let patterns = analyzer.analyze_nodejs_patterns(mongodb_code).unwrap();
        assert!(!patterns.is_empty());
        assert!(!patterns[0].database_patterns.is_empty());
        assert_eq!(patterns[0].database_patterns[0].db_type, "mongodb");
    }

    #[test]
    fn test_typescript_feature_detection() {
        let analyzer = JavaScriptAnalyzer::new();

        let typescript_code = r#"
            interface User {
                id: number;
                name: string;
            }
            
            enum Status {
                Active,
                Inactive
            }
            
            function getUser<T>(id: T): Promise<User> {
                return fetch(`/api/users/${id}`).then(res => res.json());
            }
        "#;

        let features = analyzer
            .analyze_modern_js_features(typescript_code)
            .unwrap();
        assert!(features
            .iter()
            .any(|f| matches!(f.feature_type, ModernFeatureType::TypeScriptInterface)));
        assert!(features
            .iter()
            .any(|f| matches!(f.feature_type, ModernFeatureType::TypeScriptEnum)));
        assert!(features.iter().any(|f| f.typescript_specific));
    }

    #[test]
    fn test_comprehensive_recommendations() {
        let analyzer = JavaScriptAnalyzer::new();

        let frameworks = vec![FrameworkInfo {
            name: "React".to_string(),
            confidence: 0.9,
            version_detected: None,
            features_used: vec!["Hooks".to_string()],
            best_practices: vec!["Use functional components".to_string()],
        }];

        let components = vec![ReactComponentInfo {
            name: "TestComponent".to_string(),
            component_type: ComponentType::Class,
            hooks_used: Vec::new(),
            props_analysis: PropsInfo {
                prop_names: Vec::new(),
                has_prop_types: false,
                has_default_props: false,
                destructured: false,
                typescript_props: false,
            },
            lifecycle_methods: vec!["componentDidMount".to_string()],
            jsx_elements: Vec::new(),
            context_usage: Vec::new(),
            state_management: Vec::new(),
        }];

        let recommendations =
            analyzer.get_comprehensive_recommendations(&frameworks, &components, &[], &[]);

        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.contains("functional components")));
        assert!(recommendations.iter().any(|r| r.contains("TypeScript")));
    }
}
