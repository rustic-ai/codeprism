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
    // Phase 1.3 additions
    UnionTypes,
    IntersectionTypes,
    ConditionalTypes,
    MappedTypes,
    TemplateLiteralTypes,
    TupleTypes,
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

/// Security assessment for Node.js applications (Phase 1.3)
#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub level: SecurityLevel,
    pub vulnerabilities_detected: Vec<SecurityVulnerability>,
    pub security_features: Vec<SecurityFeature>,
    pub recommendations: Vec<String>,
}

/// Security levels
#[derive(Debug, Clone)]
pub enum SecurityLevel {
    High,       // Well-secured with multiple layers
    Medium,     // Basic security measures present
    Low,        // Minimal security implementation
    Vulnerable, // Security issues detected
}

/// Security vulnerability information
#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub vulnerability_type: VulnerabilityType,
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub location: String,
    pub recommendation: String,
}

/// Security vulnerability types
#[derive(Debug, Clone)]
pub enum VulnerabilityType {
    SqlInjection,
    XssRisk,
    CsrfMissing,
    WeakAuthentication,
    InsecureDataTransmission,
    DangerousEval,
    UnvalidatedInput,
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

/// Security feature information
#[derive(Debug, Clone)]
pub struct SecurityFeature {
    pub feature_type: SecurityFeatureType,
    pub implementation_quality: ImplementationQuality,
    pub description: String,
}

/// Security feature types
#[derive(Debug, Clone)]
pub enum SecurityFeatureType {
    Authentication,
    Authorization,
    InputValidation,
    CsrfProtection,
    DataEncryption,
    SecureHeaders,
    RateLimiting,
}

/// Implementation quality assessment
#[derive(Debug, Clone)]
pub enum ImplementationQuality {
    Excellent,
    Good,
    Adequate,
    Poor,
    Missing,
}

/// Performance analysis for modern JavaScript applications (Phase 1.3)
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub overall_score: i32,
    pub optimizations_detected: Vec<PerformanceOptimization>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub recommendations: Vec<String>,
}

/// Performance optimization information
#[derive(Debug, Clone)]
pub struct PerformanceOptimization {
    pub optimization_type: OptimizationType,
    pub impact_level: ImpactLevel,
    pub description: String,
    pub best_practices_followed: bool,
}

/// Performance optimization types
#[derive(Debug, Clone)]
pub enum OptimizationType {
    LazyLoading,
    CodeSplitting,
    Memoization,
    Caching,
    DatabaseOptimization,
    BundleOptimization,
    ImageOptimization,
    AssetMinification,
}

/// Performance issue information
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub recommendation: String,
}

/// Performance issue types
#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    LargeBundle,
    UnoptimizedImages,
    MissingCaching,
    InefficientQueries,
    MemoryLeaks,
    BlockingOperations,
    ExcessiveRerendering,
}

/// Issue severity levels
#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Impact levels for performance metrics
#[derive(Debug, Clone)]
pub enum ImpactLevel {
    High,     // Significant performance impact
    Medium,   // Moderate performance impact
    Low,      // Minor performance impact
    Positive, // Performance optimization
}

/// Vue.js component analysis (Phase 1.3)
#[derive(Debug, Clone)]
pub struct VueComponentInfo {
    pub name: String,
    pub component_type: VueComponentType,
    pub composition_api: bool,
    pub props: Vec<VuePropsInfo>,
    pub emits: Vec<String>,
    pub directives: Vec<VueDirective>,
    pub lifecycle_hooks: Vec<String>,
    pub composables: Vec<String>,
}

/// Vue component types
#[derive(Debug, Clone)]
pub enum VueComponentType {
    SingleFileComponent,
    OptionsAPI,
    CompositionAPI,
    FunctionalComponent,
    AsyncComponent,
}

/// Vue props information
#[derive(Debug, Clone)]
pub struct VuePropsInfo {
    pub name: String,
    pub prop_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub validator: bool,
}

/// Vue directive information
#[derive(Debug, Clone)]
pub struct VueDirective {
    pub name: String,
    pub directive_type: VueDirectiveType,
    pub has_modifiers: bool,
    pub dynamic_argument: bool,
}

/// Vue directive types
#[derive(Debug, Clone)]
pub enum VueDirectiveType {
    BuiltIn,   // v-if, v-for, v-model, etc.
    Custom,    // User-defined directives
    Component, // Component-specific directives
}

/// Angular component analysis (Phase 1.3)
#[derive(Debug, Clone)]
pub struct AngularComponentInfo {
    pub name: String,
    pub component_type: AngularComponentType,
    pub selector: String,
    pub inputs: Vec<AngularInputInfo>,
    pub outputs: Vec<AngularOutputInfo>,
    pub lifecycle_hooks: Vec<String>,
    pub services: Vec<String>,
    pub change_detection: ChangeDetectionStrategy,
}

/// Angular component types
#[derive(Debug, Clone)]
pub enum AngularComponentType {
    Component,
    Directive,
    Pipe,
    Service,
    Guard,
    Resolver,
}

/// Angular input information
#[derive(Debug, Clone)]
pub struct AngularInputInfo {
    pub name: String,
    pub alias: Option<String>,
    pub input_type: String,
    pub required: bool,
}

/// Angular output information
#[derive(Debug, Clone)]
pub struct AngularOutputInfo {
    pub name: String,
    pub alias: Option<String>,
    pub event_type: String,
}

/// Angular change detection strategies
#[derive(Debug, Clone)]
pub enum ChangeDetectionStrategy {
    Default,
    OnPush,
    Detached,
}

/// Enhanced TypeScript analysis (Phase 1.3)
#[derive(Debug, Clone)]
pub struct TypeScriptAnalysisInfo {
    pub generics_usage: Vec<GenericInfo>,
    pub type_constraints: Vec<TypeConstraint>,
    pub utility_types: Vec<UtilityTypeUsage>,
    pub type_guards: Vec<TypeGuard>,
    pub conditional_types: Vec<ConditionalType>,
    pub mapped_types: Vec<MappedType>,
    pub complexity_score: i32,
}

/// Generic type information
#[derive(Debug, Clone)]
pub struct GenericInfo {
    pub name: String,
    pub constraints: Vec<String>,
    pub default_type: Option<String>,
    pub variance: TypeVariance,
    pub usage_context: GenericContext,
}

/// Type variance
#[derive(Debug, Clone)]
pub enum TypeVariance {
    Covariant,
    Contravariant,
    Invariant,
    Bivariant,
}

/// Generic usage context
#[derive(Debug, Clone)]
pub enum GenericContext {
    Function,
    Interface,
    Class,
    TypeAlias,
    Utility,
}

/// Type constraint information
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub constraint_type: ConstraintType,
    pub target_type: String,
    pub constraint_expression: String,
    pub complexity: i32,
}

/// Type constraint types
#[derive(Debug, Clone)]
pub enum ConstraintType {
    Extends,
    Keyof,
    Typeof,
    Conditional,
    Mapped,
    Template,
}

/// Utility type usage
#[derive(Debug, Clone)]
pub struct UtilityTypeUsage {
    pub utility_name: String,
    pub usage_pattern: String,
    pub complexity_impact: i32,
    pub best_practice_score: f32,
}

/// Type guard information
#[derive(Debug, Clone)]
pub struct TypeGuard {
    pub guard_type: TypeGuardType,
    pub target_types: Vec<String>,
    pub predicate_function: String,
    pub runtime_safety: bool,
}

/// Type guard types
#[derive(Debug, Clone)]
pub enum TypeGuardType {
    UserDefined,
    BuiltIn,
    AssertionFunction,
    DiscriminatedUnion,
}

/// Conditional type information
#[derive(Debug, Clone)]
pub struct ConditionalType {
    pub condition: String,
    pub true_type: String,
    pub false_type: String,
    pub complexity_score: i32,
}

/// Mapped type information
#[derive(Debug, Clone)]
pub struct MappedType {
    pub source_type: String,
    pub transformation: String,
    pub modifiers: Vec<TypeModifier>,
    pub complexity_score: i32,
}

/// Type modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum TypeModifier {
    Optional,
    Required,
    Readonly,
    Mutable,
}

/// WebSocket pattern analysis (Phase 1.3)
#[derive(Debug, Clone)]
pub struct WebSocketAnalysis {
    pub implementation_type: WebSocketImplementationType,
    pub patterns: Vec<WebSocketPattern>,
    pub real_time_features: Vec<RealTimeFeature>,
    pub security_assessment: WebSocketSecurityAssessment,
    pub performance_metrics: WebSocketPerformanceMetrics,
}

/// WebSocket implementation types
#[derive(Debug, Clone)]
pub enum WebSocketImplementationType {
    SocketIO,
    NativeWebSocket,
    SignalR,
    SockJS,
    Custom,
}

/// WebSocket pattern information
#[derive(Debug, Clone)]
pub struct WebSocketPattern {
    pub pattern_type: WebSocketPatternType,
    pub event_handlers: Vec<String>,
    pub room_management: bool,
    pub authentication: bool,
    pub error_handling: bool,
    pub reconnection_logic: bool,
}

/// WebSocket pattern types
#[derive(Debug, Clone)]
pub enum WebSocketPatternType {
    RealTimeChat,
    LiveUpdates,
    GameMultiplayer,
    CollaborativeEditing,
    NotificationSystem,
    StreamingData,
    FileTransfer,
}

/// Real-time feature information
#[derive(Debug, Clone)]
pub struct RealTimeFeature {
    pub feature_name: String,
    pub implementation_quality: ImplementationQuality,
    pub scalability_considerations: Vec<String>,
    pub latency_optimization: bool,
}

/// WebSocket security assessment
#[derive(Debug, Clone)]
pub struct WebSocketSecurityAssessment {
    pub authentication_method: Option<WebSocketAuthMethod>,
    pub authorization_checks: bool,
    pub rate_limiting: bool,
    pub input_validation: bool,
    pub origin_checks: bool,
    pub ssl_tls_enforced: bool,
}

/// WebSocket authentication methods
#[derive(Debug, Clone)]
pub enum WebSocketAuthMethod {
    Jwt,
    SessionBased,
    ApiKey,
    OAuth,
    Custom,
    None,
}

/// WebSocket performance metrics
#[derive(Debug, Clone)]
pub struct WebSocketPerformanceMetrics {
    pub connection_pooling: bool,
    pub message_batching: bool,
    pub compression_enabled: bool,
    pub heartbeat_implementation: bool,
    pub scaling_strategy: ScalingStrategy,
}

/// Scaling strategies for WebSocket
#[derive(Debug, Clone)]
pub enum ScalingStrategy {
    SingleInstance,
    LoadBalanced,
    Clustered,
    Microservices,
    Redis,
    RabbitMQ,
}

/// Advanced Node.js pattern analysis (Phase 1.3)
#[derive(Debug, Clone)]
pub struct AdvancedNodePatternInfo {
    pub pattern_type: AdvancedNodePatternType,
    pub middleware_chain: Vec<MiddlewareInfo>,
    pub error_handling: Vec<ErrorHandlingPattern>,
    pub performance_indicators: Vec<PerformanceIndicator>,
    pub microservice_patterns: Vec<MicroservicePattern>,
    pub database_patterns: Vec<DatabasePattern>,
}

/// Advanced Node.js pattern types
#[derive(Debug, Clone)]
pub enum AdvancedNodePatternType {
    SecurityMiddleware,
    PerformanceMiddleware,
    WebSocketHandler,
    MicroserviceGateway,
    EventDrivenArchitecture,
    StreamProcessing,
    BackgroundJobProcessor,
}

/// Middleware information
#[derive(Debug, Clone)]
pub struct MiddlewareInfo {
    pub name: String,
    pub middleware_type: MiddlewareType,
    pub order: i32,
    pub dependencies: Vec<String>,
    pub security_impact: Option<String>,
    pub performance_impact: Option<String>,
}

/// Middleware types
#[derive(Debug, Clone)]
pub enum MiddlewareType {
    Authentication,
    Authorization,
    RateLimiting,
    Cors,
    Logging,
    Validation,
    Caching,
    Compression,
    ErrorHandling,
    Custom,
}

/// Error handling pattern information
#[derive(Debug, Clone)]
pub struct ErrorHandlingPattern {
    pub pattern_type: ErrorHandlingType,
    pub implementation_quality: ImplementationQuality,
    pub error_classification: Vec<ErrorClassification>,
    pub recovery_strategies: Vec<String>,
    pub monitoring_integration: bool,
}

/// Error handling types
#[derive(Debug, Clone)]
pub enum ErrorHandlingType {
    TryCatch,
    PromiseChain,
    AsyncAwait,
    EventEmitter,
    Circuit,
    Retry,
    Fallback,
}

/// Error classification
#[derive(Debug, Clone)]
pub struct ErrorClassification {
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub handling_strategy: String,
    pub user_impact: UserImpact,
}

/// Error types
#[derive(Debug, Clone)]
pub enum ErrorType {
    ValidationError,
    AuthenticationError,
    AuthorizationError,
    DatabaseError,
    NetworkError,
    BusinessLogicError,
    SystemError,
    UnknownError,
}

/// Error severity levels
#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// User impact levels
#[derive(Debug, Clone)]
pub enum UserImpact {
    ServiceUnavailable,
    FeatureImpaired,
    PerformanceDegraded,
    NoImpact,
}

/// Performance indicator for advanced patterns
#[derive(Debug, Clone)]
pub struct PerformanceIndicator {
    pub indicator_type: PerformanceType,
    pub impact_level: ImpactLevel,
    pub description: String,
    pub recommendation: String,
    pub metrics: Option<PerformanceMetrics>,
}

/// Performance types
#[derive(Debug, Clone)]
pub enum PerformanceType {
    Memory,
    Cpu,
    Network,
    Disk,
    Database,
    Caching,
    Bundling,
    Rendering,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub response_time: Option<f64>,
    pub throughput: Option<f64>,
    pub memory_usage: Option<f64>,
    pub cpu_usage: Option<f64>,
    pub error_rate: Option<f64>,
}

/// Microservice pattern information
#[derive(Debug, Clone)]
pub struct MicroservicePattern {
    pub pattern_name: String,
    pub service_communication: ServiceCommunication,
    pub data_consistency: DataConsistency,
    pub fault_tolerance: FaultTolerance,
    pub monitoring_observability: MonitoringLevel,
}

/// Service communication patterns
#[derive(Debug, Clone)]
pub enum ServiceCommunication {
    Http,
    GraphQL,
    GRpc,
    MessageQueue,
    EventStream,
    WebSocket,
}

/// Data consistency patterns
#[derive(Debug, Clone)]
pub enum DataConsistency {
    Strong,
    Eventual,
    Weak,
    Session,
    Causal,
}

/// Fault tolerance patterns
#[derive(Debug, Clone)]
pub enum FaultTolerance {
    CircuitBreaker,
    Retry,
    Timeout,
    Bulkhead,
    Fallback,
    None,
}

/// Monitoring levels
#[derive(Debug, Clone)]
pub enum MonitoringLevel {
    Comprehensive,
    Basic,
    Minimal,
    None,
}

/// Database pattern information
#[derive(Debug, Clone)]
pub struct DatabasePattern {
    pub database_type: DatabaseType,
    pub access_pattern: DatabaseAccessPattern,
    pub optimization_level: OptimizationLevel,
    pub connection_management: ConnectionManagement,
    pub transaction_handling: TransactionHandling,
}

/// Database types
#[derive(Debug, Clone)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MongoDB,
    Redis,
    Elasticsearch,
    SQLite,
    Cassandra,
    DynamoDB,
}

/// Database access patterns
#[derive(Debug, Clone)]
pub enum DatabaseAccessPattern {
    DirectAccess,
    Orm,
    QueryBuilder,
    Repository,
    ActiveRecord,
    DataMapper,
}

/// Optimization levels
#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    High,
    Medium,
    Low,
    None,
}

/// Connection management patterns
#[derive(Debug, Clone)]
pub enum ConnectionManagement {
    Pool,
    SingleConnection,
    PerRequest,
    Lazy,
    Cached,
}

/// Transaction handling patterns
#[derive(Debug, Clone)]
pub enum TransactionHandling {
    Acid,
    Eventually,
    TwoPhase,
    Saga,
    Compensating,
    None,
}

/// JavaScript/TypeScript-specific analyzer
pub struct JavaScriptAnalyzer {
    framework_patterns: HashMap<String, Regex>,
    react_patterns: HashMap<String, Regex>,
    nodejs_patterns: HashMap<String, Regex>,
    typescript_patterns: HashMap<String, Regex>,
    vue_patterns: HashMap<String, Regex>,
    angular_patterns: HashMap<String, Regex>,
    // Phase 1.3 additions
    security_patterns: HashMap<String, Regex>,
    performance_patterns: HashMap<String, Regex>,
    websocket_patterns: HashMap<String, Regex>,
    advanced_typescript_patterns: HashMap<String, Regex>,
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
            security_patterns: HashMap::new(),
            performance_patterns: HashMap::new(),
            websocket_patterns: HashMap::new(),
            advanced_typescript_patterns: HashMap::new(),
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

        // Phase 1.3: Enhanced Vue.js patterns
        self.vue_patterns.insert(
            "single_file_component".to_string(),
            Regex::new(r"<template>|<script>|<style>").unwrap(),
        );
        self.vue_patterns.insert(
            "composition_api".to_string(),
            Regex::new(r"setup\s*\(|ref\s*\(|reactive\s*\(|computed\s*\(").unwrap(),
        );
        self.vue_patterns.insert(
            "options_api".to_string(),
            Regex::new(r"data\s*\(\s*\)|methods\s*:|computed\s*:|watch\s*:").unwrap(),
        );
        self.vue_patterns.insert(
            "directives".to_string(),
            Regex::new(r"v-if|v-for|v-model|v-show|v-bind|v-on").unwrap(),
        );
        self.vue_patterns.insert(
            "composables".to_string(),
            Regex::new(r"use[A-Z]\w*\s*\(").unwrap(),
        );

        // Phase 1.3: Enhanced Angular patterns
        self.angular_patterns.insert(
            "component_decorator".to_string(),
            Regex::new(r"@Component\s*\(\s*\{").unwrap(),
        );
        self.angular_patterns.insert(
            "service_decorator".to_string(),
            Regex::new(r"@Injectable\s*\(\s*\{").unwrap(),
        );
        self.angular_patterns.insert(
            "input_output".to_string(),
            Regex::new(r"@Input\s*\(\s*\)|@Output\s*\(\s*\)").unwrap(),
        );
        self.angular_patterns.insert(
            "lifecycle_hooks".to_string(),
            Regex::new(r"ngOnInit|ngOnDestroy|ngOnChanges|ngAfterViewInit").unwrap(),
        );
        self.angular_patterns.insert(
            "dependency_injection".to_string(),
            Regex::new(r"constructor\s*\(\s*private|inject\s*\(").unwrap(),
        );

        // Phase 1.3: Security patterns
        self.security_patterns.insert(
            "authentication_middleware".to_string(),
            Regex::new(r"passport\.|jwt\.|authenticate\s*\(|verify\s*\(").unwrap(),
        );
        self.security_patterns.insert(
            "input_validation".to_string(),
            Regex::new(r"validate\s*\(|sanitize\s*\(|escape\s*\(|joi\.|yup\.").unwrap(),
        );
        self.security_patterns.insert(
            "csrf_protection".to_string(),
            Regex::new(r"csrf\.|csrfToken|__RequestVerificationToken").unwrap(),
        );
        self.security_patterns.insert(
            "sql_injection_risk".to_string(),
            Regex::new(r"query\s*\(\s*.*\$\{|execute\s*\(\s*.*\+").unwrap(),
        );
        self.security_patterns.insert(
            "xss_vulnerability".to_string(),
            Regex::new(r"innerHTML\s*=|dangerouslySetInnerHTML|eval\s*\(").unwrap(),
        );

        // Phase 1.3: Performance patterns
        self.performance_patterns.insert(
            "lazy_loading".to_string(),
            Regex::new(r"lazy\s*\(|import\s*\(|React\.lazy|defineAsyncComponent").unwrap(),
        );
        self.performance_patterns.insert(
            "memoization".to_string(),
            Regex::new(r"useMemo\s*\(|useCallback\s*\(|memo\s*\(|React\.memo").unwrap(),
        );
        self.performance_patterns.insert(
            "caching".to_string(),
            Regex::new(r"cache\.|redis\.|localStorage|sessionStorage").unwrap(),
        );
        self.performance_patterns.insert(
            "database_optimization".to_string(),
            Regex::new(r"\.populate\s*\(|\.select\s*\(|\.limit\s*\(|\.sort\s*\(").unwrap(),
        );

        // Phase 1.3: WebSocket patterns
        self.websocket_patterns.insert(
            "socket_io".to_string(),
            Regex::new(r"socket\.io|io\s*\(|socket\.emit|socket\.on").unwrap(),
        );
        self.websocket_patterns.insert(
            "native_websocket".to_string(),
            Regex::new(r"new WebSocket\s*\(|ws\.|WebSocketServer").unwrap(),
        );
        self.websocket_patterns.insert(
            "room_management".to_string(),
            Regex::new(r"\.join\s*\(|\.leave\s*\(|\.to\s*\(|\.in\s*\(").unwrap(),
        );

        // Phase 1.3: Advanced TypeScript patterns
        self.advanced_typescript_patterns.insert(
            "union_intersection".to_string(),
            Regex::new(r"\w+\s*\|\s*\w+|\w+\s*&\s*\w+").unwrap(),
        );
        self.advanced_typescript_patterns.insert(
            "conditional_types".to_string(),
            Regex::new(r"extends\s+[^?]+\?\s*[^:]+\s*:\s*\w+").unwrap(),
        );
        self.advanced_typescript_patterns.insert(
            "mapped_types".to_string(),
            Regex::new(r"Partial<|Required<|Readonly<|Pick<|Omit<|\[\s*\w+\s+in\s+keyof").unwrap(),
        );
        self.advanced_typescript_patterns.insert(
            "utility_types".to_string(),
            Regex::new(r"Record<|Exclude<|Extract<|ReturnType<|Parameters<|keyof\s+\w+").unwrap(),
        );
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

    // Phase 1.3: Advanced Analysis Methods

    /// Analyze Vue.js components and patterns (Phase 1.3)
    pub fn analyze_vue_patterns(&self, content: &str) -> Result<Vec<VueComponentInfo>> {
        let mut components = Vec::new();

        // Single File Component detection
        if self
            .vue_patterns
            .get("single_file_component")
            .unwrap()
            .is_match(content)
        {
            let component_name = "VueComponent".to_string(); // Simplified extraction

            let composition_api = self
                .vue_patterns
                .get("composition_api")
                .unwrap()
                .is_match(content);
            let options_api = self
                .vue_patterns
                .get("options_api")
                .unwrap()
                .is_match(content);

            let component_type = if composition_api {
                VueComponentType::CompositionAPI
            } else if options_api {
                VueComponentType::OptionsAPI
            } else {
                VueComponentType::SingleFileComponent
            };

            components.push(VueComponentInfo {
                name: component_name,
                component_type,
                composition_api,
                props: Vec::new(), // Simplified for Phase 1.3
                emits: Vec::new(),
                directives: if self
                    .vue_patterns
                    .get("directives")
                    .unwrap()
                    .is_match(content)
                {
                    vec![VueDirective {
                        name: "v-if".to_string(),
                        directive_type: VueDirectiveType::BuiltIn,
                        has_modifiers: false,
                        dynamic_argument: false,
                    }]
                } else {
                    Vec::new()
                },
                lifecycle_hooks: Vec::new(),
                composables: if self
                    .vue_patterns
                    .get("composables")
                    .unwrap()
                    .is_match(content)
                {
                    vec!["useComposable".to_string()]
                } else {
                    Vec::new()
                },
            });
        }

        Ok(components)
    }

    /// Analyze Angular components and patterns (Phase 1.3)
    pub fn analyze_angular_patterns(&self, content: &str) -> Result<Vec<AngularComponentInfo>> {
        let mut components = Vec::new();

        // Component decorator detection
        if self
            .angular_patterns
            .get("component_decorator")
            .unwrap()
            .is_match(content)
        {
            let selector = "app-component".to_string(); // Simplified extraction
            let component_name = "AngularComponent".to_string(); // Simplified extraction

            let component_type = if self
                .angular_patterns
                .get("service_decorator")
                .unwrap()
                .is_match(content)
            {
                AngularComponentType::Service
            } else {
                AngularComponentType::Component
            };

            components.push(AngularComponentInfo {
                name: component_name,
                component_type,
                selector,
                inputs: Vec::new(), // Simplified for Phase 1.3
                outputs: Vec::new(),
                lifecycle_hooks: if self
                    .angular_patterns
                    .get("lifecycle_hooks")
                    .unwrap()
                    .is_match(content)
                {
                    vec!["ngOnInit".to_string()]
                } else {
                    Vec::new()
                },
                services: Vec::new(),
                change_detection: ChangeDetectionStrategy::Default,
            });
        }

        Ok(components)
    }

    /// Analyze security patterns and vulnerabilities (Phase 1.3)
    pub fn analyze_security_assessment(&self, content: &str) -> Result<SecurityAssessment> {
        let mut vulnerabilities = Vec::new();
        let mut security_features = Vec::new();
        let mut security_score = 0;

        // Check for authentication middleware
        if self
            .security_patterns
            .get("authentication_middleware")
            .unwrap()
            .is_match(content)
        {
            security_score += 2;
            security_features.push(SecurityFeature {
                feature_type: SecurityFeatureType::Authentication,
                implementation_quality: ImplementationQuality::Good,
                description: "Authentication middleware detected".to_string(),
            });
        }

        // Check for input validation
        if self
            .security_patterns
            .get("input_validation")
            .unwrap()
            .is_match(content)
        {
            security_score += 2;
            security_features.push(SecurityFeature {
                feature_type: SecurityFeatureType::InputValidation,
                implementation_quality: ImplementationQuality::Good,
                description: "Input validation patterns detected".to_string(),
            });
        }

        // Check for vulnerabilities
        if self
            .security_patterns
            .get("sql_injection_risk")
            .unwrap()
            .is_match(content)
        {
            security_score -= 3;
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: VulnerabilityType::SqlInjection,
                severity: VulnerabilitySeverity::High,
                description: "Potential SQL injection vulnerability".to_string(),
                location: "Query construction".to_string(),
                recommendation: "Use parameterized queries".to_string(),
            });
        }

        if self
            .security_patterns
            .get("xss_vulnerability")
            .unwrap()
            .is_match(content)
        {
            security_score -= 3;
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: VulnerabilityType::XssRisk,
                severity: VulnerabilitySeverity::High,
                description: "Potential XSS vulnerability".to_string(),
                location: "DOM manipulation".to_string(),
                recommendation: "Sanitize user input".to_string(),
            });
        }

        let level = match security_score {
            score if score >= 3 => SecurityLevel::High,
            score if score >= 1 => SecurityLevel::Medium,
            score if score >= 0 => SecurityLevel::Low,
            _ => SecurityLevel::Vulnerable,
        };

        Ok(SecurityAssessment {
            level,
            vulnerabilities_detected: vulnerabilities,
            security_features,
            recommendations: vec![
                "Implement comprehensive input validation".to_string(),
                "Add authentication and authorization middleware".to_string(),
                "Use HTTPS for all communications".to_string(),
            ],
        })
    }

    /// Analyze performance patterns and optimizations (Phase 1.3)
    pub fn analyze_performance_patterns(&self, content: &str) -> Result<PerformanceAnalysis> {
        let mut optimizations = Vec::new();
        let issues = Vec::new();
        let mut score = 50; // Base score

        // Check for lazy loading
        if self
            .performance_patterns
            .get("lazy_loading")
            .unwrap()
            .is_match(content)
        {
            score += 15;
            optimizations.push(PerformanceOptimization {
                optimization_type: OptimizationType::LazyLoading,
                impact_level: ImpactLevel::Positive,
                description: "Lazy loading implementation detected".to_string(),
                best_practices_followed: true,
            });
        }

        // Check for memoization
        if self
            .performance_patterns
            .get("memoization")
            .unwrap()
            .is_match(content)
        {
            score += 10;
            optimizations.push(PerformanceOptimization {
                optimization_type: OptimizationType::Memoization,
                impact_level: ImpactLevel::Positive,
                description: "Memoization patterns detected".to_string(),
                best_practices_followed: true,
            });
        }

        // Check for caching
        if self
            .performance_patterns
            .get("caching")
            .unwrap()
            .is_match(content)
        {
            score += 10;
            optimizations.push(PerformanceOptimization {
                optimization_type: OptimizationType::Caching,
                impact_level: ImpactLevel::Positive,
                description: "Caching mechanisms detected".to_string(),
                best_practices_followed: true,
            });
        }

        // Check for database optimization
        if self
            .performance_patterns
            .get("database_optimization")
            .unwrap()
            .is_match(content)
        {
            score += 5;
            optimizations.push(PerformanceOptimization {
                optimization_type: OptimizationType::DatabaseOptimization,
                impact_level: ImpactLevel::Medium,
                description: "Database optimization patterns detected".to_string(),
                best_practices_followed: true,
            });
        }

        Ok(PerformanceAnalysis {
            overall_score: score.min(100),
            optimizations_detected: optimizations,
            performance_issues: issues,
            recommendations: vec![
                "Implement lazy loading for non-critical resources".to_string(),
                "Use memoization for expensive calculations".to_string(),
                "Add appropriate caching strategies".to_string(),
                "Optimize database queries with indexing".to_string(),
            ],
        })
    }

    /// Analyze WebSocket patterns and real-time features (Phase 1.3)
    pub fn analyze_websocket_patterns(&self, content: &str) -> Result<Option<WebSocketAnalysis>> {
        let socket_io_detected = self
            .websocket_patterns
            .get("socket_io")
            .unwrap()
            .is_match(content);
        let native_ws_detected = self
            .websocket_patterns
            .get("native_websocket")
            .unwrap()
            .is_match(content);

        if !socket_io_detected && !native_ws_detected {
            return Ok(None);
        }

        let implementation_type = if socket_io_detected {
            WebSocketImplementationType::SocketIO
        } else {
            WebSocketImplementationType::NativeWebSocket
        };

        let mut patterns = Vec::new();
        let room_management = self
            .websocket_patterns
            .get("room_management")
            .unwrap()
            .is_match(content);

        patterns.push(WebSocketPattern {
            pattern_type: WebSocketPatternType::RealTimeChat,
            event_handlers: vec!["message".to_string(), "connect".to_string()],
            room_management,
            authentication: false, // Simplified detection
            error_handling: false,
            reconnection_logic: false,
        });

        Ok(Some(WebSocketAnalysis {
            implementation_type,
            patterns,
            real_time_features: vec![RealTimeFeature {
                feature_name: "Real-time messaging".to_string(),
                implementation_quality: ImplementationQuality::Good,
                scalability_considerations: vec!["Consider using Redis for scaling".to_string()],
                latency_optimization: false,
            }],
            security_assessment: WebSocketSecurityAssessment {
                authentication_method: None,
                authorization_checks: false,
                rate_limiting: false,
                input_validation: false,
                origin_checks: false,
                ssl_tls_enforced: false,
            },
            performance_metrics: WebSocketPerformanceMetrics {
                connection_pooling: false,
                message_batching: false,
                compression_enabled: false,
                heartbeat_implementation: false,
                scaling_strategy: ScalingStrategy::SingleInstance,
            },
        }))
    }

    /// Analyze enhanced TypeScript features (Phase 1.3)
    pub fn analyze_enhanced_typescript(&self, content: &str) -> Result<TypeScriptAnalysisInfo> {
        let mut analysis = TypeScriptAnalysisInfo {
            generics_usage: Vec::new(),
            type_constraints: Vec::new(),
            utility_types: Vec::new(),
            type_guards: Vec::new(),
            conditional_types: Vec::new(),
            mapped_types: Vec::new(),
            complexity_score: 0,
        };

        let mut complexity = 0;

        // Check for union/intersection types
        if self
            .advanced_typescript_patterns
            .get("union_intersection")
            .unwrap()
            .is_match(content)
        {
            complexity += 2;
        }

        // Check for conditional types
        if self
            .advanced_typescript_patterns
            .get("conditional_types")
            .unwrap()
            .is_match(content)
        {
            complexity += 4;
            analysis.conditional_types.push(ConditionalType {
                condition: "T extends U".to_string(),
                true_type: "T".to_string(),
                false_type: "never".to_string(),
                complexity_score: 4,
            });
        }

        // Check for mapped types
        if self
            .advanced_typescript_patterns
            .get("mapped_types")
            .unwrap()
            .is_match(content)
        {
            complexity += 3;
            analysis.mapped_types.push(MappedType {
                source_type: "T".to_string(),
                transformation: "Partial<T>".to_string(),
                modifiers: vec![TypeModifier::Optional],
                complexity_score: 3,
            });
        }

        // Check for utility types
        if self
            .advanced_typescript_patterns
            .get("utility_types")
            .unwrap()
            .is_match(content)
        {
            complexity += 2;
            analysis.utility_types.push(UtilityTypeUsage {
                utility_name: "Partial".to_string(),
                usage_pattern: "Partial<T>".to_string(),
                complexity_impact: 2,
                best_practice_score: 0.8,
            });
        }

        analysis.complexity_score = complexity;
        Ok(analysis)
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

    // Phase 1.3 Tests

    #[test]
    fn test_vue_component_analysis() {
        let analyzer = JavaScriptAnalyzer::new();

        let vue_code = r#"
            <template>
                <div v-if="show">
                    <p v-for="item in items" :key="item.id">{{ item.name }}</p>
                </div>
            </template>
            
            <script>
            import { ref, computed, onMounted } from 'vue'
            
            export default {
                setup() {
                    const count = ref(0)
                    const doubled = computed(() => count.value * 2)
                    
                    onMounted(() => {
                        console.log('Component mounted')
                    })
                    
                    return { count, doubled }
                }
            }
            </script>
        "#;

        let components = analyzer.analyze_vue_patterns(vue_code).unwrap();
        assert!(!components.is_empty());
        assert_eq!(components[0].name, "VueComponent");
        assert!(matches!(
            components[0].component_type,
            VueComponentType::CompositionAPI
        ));
        assert!(components[0].composition_api);
        assert!(!components[0].directives.is_empty());
        assert_eq!(components[0].directives[0].name, "v-if");
    }

    #[test]
    fn test_angular_component_analysis() {
        let analyzer = JavaScriptAnalyzer::new();

        let angular_code = r#"
            import { Component, OnInit, Input, Output, EventEmitter } from '@angular/core';
            
            @Component({
                selector: 'app-user',
                templateUrl: './user.component.html'
            })
            export class UserComponent implements OnInit {
                @Input() user: User;
                @Output() userSelected = new EventEmitter<User>();
                
                constructor(private userService: UserService) {}
                
                ngOnInit() {
                    this.loadUser();
                }
                
                ngOnDestroy() {
                    // cleanup
                }
            }
        "#;

        let components = analyzer.analyze_angular_patterns(angular_code).unwrap();
        assert!(!components.is_empty());
        assert_eq!(components[0].name, "AngularComponent");
        assert!(matches!(
            components[0].component_type,
            AngularComponentType::Component
        ));
        assert_eq!(components[0].selector, "app-component");
        assert!(!components[0].lifecycle_hooks.is_empty());
        assert_eq!(components[0].lifecycle_hooks[0], "ngOnInit");
    }

    #[test]
    fn test_security_assessment() {
        let analyzer = JavaScriptAnalyzer::new();

        let secure_code = r#"
            const jwt = require('jsonwebtoken');
            const joi = require('joi');
            
            app.use(passport.authenticate('jwt', { session: false }));
            
            const schema = joi.object({
                email: joi.string().email().required(),
                password: joi.string().min(6).required()
            });
            
            app.post('/login', async (req, res) => {
                const { error } = schema.validate(req.body);
                if (error) return res.status(400).send(error.details);
                
                // Safe query using parameterized query
                const user = await User.findOne({ email: req.body.email });
                res.json({ token: jwt.sign({ id: user.id }, process.env.JWT_SECRET) });
            });
        "#;

        let assessment = analyzer.analyze_security_assessment(secure_code).unwrap();
        assert!(matches!(
            assessment.level,
            SecurityLevel::High | SecurityLevel::Medium
        ));
        assert!(!assessment.security_features.is_empty());

        let vulnerable_code = r#"
            app.get('/user/:id', (req, res) => {
                // SQL injection vulnerability
                const query = `SELECT * FROM users WHERE id = ${req.params.id}`;
                db.query(query, (err, results) => {
                    res.innerHTML = results[0].bio; // XSS vulnerability
                });
            });
        "#;

        let vulnerable_assessment = analyzer
            .analyze_security_assessment(vulnerable_code)
            .unwrap();
        assert!(matches!(
            vulnerable_assessment.level,
            SecurityLevel::Vulnerable
        ));
        assert!(!vulnerable_assessment.vulnerabilities_detected.is_empty());
    }

    #[test]
    fn test_performance_analysis() {
        let analyzer = JavaScriptAnalyzer::new();

        let optimized_code = r#"
            import { lazy, memo, useMemo, useCallback } from 'react';
            
            const LazyComponent = lazy(() => import('./HeavyComponent'));
            
            const OptimizedComponent = memo(({ data, onUpdate }) => {
                const expensiveValue = useMemo(() => {
                    return data.reduce((acc, item) => acc + item.value, 0);
                }, [data]);
                
                const handleClick = useCallback(() => {
                    onUpdate(expensiveValue);
                }, [expensiveValue, onUpdate]);
                
                return <LazyComponent value={expensiveValue} onClick={handleClick} />;
            });
            
            // Database optimization
            const users = await User.find({})
                .select('name email')
                .limit(10)
                .sort({ createdAt: -1 });
                
            // Caching
            const cachedData = cache.get('user-data') || await fetchUserData();
        "#;

        let analysis = analyzer
            .analyze_performance_patterns(optimized_code)
            .unwrap();
        assert!(analysis.overall_score > 50);
        assert!(!analysis.optimizations_detected.is_empty());

        let optimizations = &analysis.optimizations_detected;
        assert!(optimizations
            .iter()
            .any(|o| matches!(o.optimization_type, OptimizationType::LazyLoading)));
        assert!(optimizations
            .iter()
            .any(|o| matches!(o.optimization_type, OptimizationType::Memoization)));
        assert!(optimizations
            .iter()
            .any(|o| matches!(o.optimization_type, OptimizationType::Caching)));
        assert!(optimizations
            .iter()
            .any(|o| matches!(o.optimization_type, OptimizationType::DatabaseOptimization)));
    }

    #[test]
    fn test_websocket_analysis() {
        let analyzer = JavaScriptAnalyzer::new();

        let socket_io_code = r#"
            const io = require('socket.io')(server);
            
            io.on('connection', (socket) => {
                console.log('User connected');
                
                socket.join('room1');
                socket.to('room1').emit('user-joined', { id: socket.id });
                
                socket.on('message', (data) => {
                    io.to('room1').emit('message', data);
                });
                
                socket.on('disconnect', () => {
                    console.log('User disconnected');
                });
            });
        "#;

        let analysis = analyzer.analyze_websocket_patterns(socket_io_code).unwrap();
        assert!(analysis.is_some());

        let ws_analysis = analysis.unwrap();
        assert!(matches!(
            ws_analysis.implementation_type,
            WebSocketImplementationType::SocketIO
        ));
        assert!(!ws_analysis.patterns.is_empty());
        assert!(ws_analysis.patterns[0].room_management);
        assert!(!ws_analysis.patterns[0].event_handlers.is_empty());

        let native_ws_code = r#"
            const WebSocket = require('ws');
            const wss = new WebSocketServer({ port: 8080 });
            
            wss.on('connection', (ws) => {
                ws.on('message', (data) => {
                    ws.send('Echo: ' + data);
                });
            });
        "#;

        let native_analysis = analyzer.analyze_websocket_patterns(native_ws_code).unwrap();
        assert!(native_analysis.is_some());
        assert!(matches!(
            native_analysis.unwrap().implementation_type,
            WebSocketImplementationType::NativeWebSocket
        ));
    }

    #[test]
    fn test_enhanced_typescript_analysis() {
        let analyzer = JavaScriptAnalyzer::new();

        let complex_ts_code = r#"
            type User = {
                id: number;
                name: string;
            };
            
            type Admin = {
                permissions: string[];
            };
            
            // Union and intersection types
            type UserOrAdmin = User | Admin;
            type SuperUser = User & Admin;
            
            // Conditional types
            type NonNullable<T> = T extends null | undefined ? never : T;
            
            // Mapped types
            type Partial<T> = {
                [P in keyof T]?: T[P];
            };
            
            // Utility types
            type UserKeys = keyof User;
            type UserName = Pick<User, 'name'>;
            type PartialUser = Partial<User>;
            type UserRecord = Record<string, User>;
        "#;

        let analysis = analyzer
            .analyze_enhanced_typescript(complex_ts_code)
            .unwrap();
        assert!(analysis.complexity_score > 0);
        assert!(!analysis.conditional_types.is_empty());
        assert!(!analysis.mapped_types.is_empty());
        assert!(!analysis.utility_types.is_empty());

        let conditional_type = &analysis.conditional_types[0];
        assert_eq!(conditional_type.complexity_score, 4);

        let mapped_type = &analysis.mapped_types[0];
        assert!(mapped_type.modifiers.contains(&TypeModifier::Optional));

        let utility_type = &analysis.utility_types[0];
        assert_eq!(utility_type.utility_name, "Partial");
        assert!(utility_type.best_practice_score > 0.0);
    }

    #[test]
    fn test_no_websocket_patterns() {
        let analyzer = JavaScriptAnalyzer::new();

        let regular_code = r#"
            function regularFunction() {
                console.log('No WebSocket here');
                return fetch('/api/data').then(res => res.json());
            }
        "#;

        let analysis = analyzer.analyze_websocket_patterns(regular_code).unwrap();
        assert!(analysis.is_none());
    }
}
