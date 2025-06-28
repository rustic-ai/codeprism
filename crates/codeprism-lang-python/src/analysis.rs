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

/// Python security assessment
#[derive(Debug, Clone)]
pub struct PythonSecurityAssessment {
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

/// Security vulnerability types for Python
#[derive(Debug, Clone)]
pub enum VulnerabilityType {
    SqlInjection,             // Raw SQL queries, unsafe ORM usage
    CommandInjection,         // subprocess, os.system with user input
    DeserializationAttack,    // pickle, yaml.load without safe_load
    PathTraversal,            // os.path.join with user input
    WeakAuthentication,       // Weak password policies, no 2FA
    InsecureDataTransmission, // HTTP instead of HTTPS
    DangerousPickle,          // Unsafe pickle usage
    UnvalidatedInput,         // Missing input validation
    InsecureRandomness,       // Using random instead of secrets
    HardcodedSecrets,         // API keys, passwords in code
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

/// Security feature types for Python
#[derive(Debug, Clone)]
pub enum SecurityFeatureType {
    Authentication,         // JWT, OAuth, session-based
    Authorization,          // RBAC, permissions
    InputValidation,        // Pydantic, Marshmallow, custom validators
    CsrfProtection,         // Django CSRF, Flask-WTF
    DataEncryption,         // cryptography, bcrypt, hashlib
    SecureHeaders,          // Security headers in responses
    RateLimiting,           // Flask-Limiter, slowapi
    SqlInjectionPrevention, // Parameterized queries, ORM usage
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

/// Python performance analysis
#[derive(Debug, Clone)]
pub struct PythonPerformanceAnalysis {
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

/// Performance optimization types for Python
#[derive(Debug, Clone)]
pub enum OptimizationType {
    ListComprehension,       // Using list comprehensions vs loops
    GeneratorUsage,          // Using generators for memory efficiency
    CachingImplementation,   // functools.cache, Redis, memcached
    DatabaseOptimization,    // Query optimization, connection pooling
    AsyncAwaitUsage,         // Proper async/await patterns
    MemoryOptimization,      // __slots__, weak references
    AlgorithmicOptimization, // Using efficient algorithms and data structures
}

/// Performance issue information  
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub recommendation: String,
}

/// Performance issue types for Python
#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    InEfficientLoops,    // Nested loops, unnecessary iterations
    MemoryLeaks,         // Circular references, unclosed resources
    BlockingOperations,  // Sync operations in async context
    InefficientQueries,  // N+1 queries, missing joins
    LargeDataStructures, // Loading large datasets into memory
    UnoptimizedImports,  // Importing heavy modules unnecessarily
    GilContention,       // Threading inefficiencies
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

/// Enhanced Python framework analysis
#[derive(Debug, Clone)]
pub struct PythonFrameworkInfo {
    pub name: String,
    pub confidence: f32,
    pub version_detected: Option<String>,
    pub features_used: Vec<String>,
    pub best_practices: Vec<String>,
    pub framework_specific_analysis: FrameworkSpecificAnalysis,
}

/// Framework-specific analysis
#[derive(Debug, Clone)]
pub enum FrameworkSpecificAnalysis {
    Django(DjangoAnalysis),
    Flask(FlaskAnalysis),
    FastAPI(FastAPIAnalysis),
    Pytest(PytestAnalysis),
    Celery(CeleryAnalysis),
}

/// Django-specific analysis
#[derive(Debug, Clone)]
pub struct DjangoAnalysis {
    pub models_analysis: Vec<DjangoModelInfo>,
    pub views_analysis: Vec<DjangoViewInfo>,
    pub middleware_usage: Vec<String>,
    pub security_middleware: Vec<String>,
    pub signals_usage: Vec<String>,
    pub admin_customization: bool,
}

/// Django model information
#[derive(Debug, Clone)]
pub struct DjangoModelInfo {
    pub name: String,
    pub fields: Vec<DjangoFieldInfo>,
    pub relationships: Vec<String>,
    pub custom_managers: bool,
    pub meta_options: Vec<String>,
}

/// Django field information
#[derive(Debug, Clone)]
pub struct DjangoFieldInfo {
    pub name: String,
    pub field_type: String,
    pub constraints: Vec<String>,
    pub indexes: bool,
}

/// Django view information
#[derive(Debug, Clone)]
pub struct DjangoViewInfo {
    pub name: String,
    pub view_type: DjangoViewType,
    pub permissions: Vec<String>,
    pub mixins: Vec<String>,
}

/// Django view types
#[derive(Debug, Clone)]
pub enum DjangoViewType {
    FunctionBased,
    ClassBased,
    GenericView,
    ViewSet,
}

/// Flask-specific analysis
#[derive(Debug, Clone)]
pub struct FlaskAnalysis {
    pub blueprints: Vec<FlaskBlueprintInfo>,
    pub extensions: Vec<String>,
    pub error_handlers: Vec<String>,
    pub template_usage: bool,
    pub session_management: bool,
}

/// Flask blueprint information
#[derive(Debug, Clone)]
pub struct FlaskBlueprintInfo {
    pub name: String,
    pub url_prefix: Option<String>,
    pub routes: Vec<FlaskRouteInfo>,
}

/// Flask route information
#[derive(Debug, Clone)]
pub struct FlaskRouteInfo {
    pub path: String,
    pub methods: Vec<String>,
    pub endpoint: String,
    pub view_function: String,
}

/// FastAPI-specific analysis
#[derive(Debug, Clone)]
pub struct FastAPIAnalysis {
    pub router_usage: Vec<FastAPIRouterInfo>,
    pub dependency_injection: Vec<String>,
    pub background_tasks: bool,
    pub websocket_endpoints: Vec<String>,
    pub middleware: Vec<String>,
    pub response_models: Vec<String>,
}

/// FastAPI router information
#[derive(Debug, Clone)]
pub struct FastAPIRouterInfo {
    pub prefix: Option<String>,
    pub tags: Vec<String>,
    pub endpoints: Vec<FastAPIEndpointInfo>,
}

/// FastAPI endpoint information
#[derive(Debug, Clone)]
pub struct FastAPIEndpointInfo {
    pub path: String,
    pub method: String,
    pub response_model: Option<String>,
    pub dependencies: Vec<String>,
}

/// Pytest-specific analysis
#[derive(Debug, Clone)]
pub struct PytestAnalysis {
    pub fixtures: Vec<PytestFixtureInfo>,
    pub parametrized_tests: Vec<String>,
    pub markers: Vec<String>,
    pub plugins: Vec<String>,
    pub coverage_setup: bool,
}

/// Pytest fixture information
#[derive(Debug, Clone)]
pub struct PytestFixtureInfo {
    pub name: String,
    pub scope: String,
    pub autouse: bool,
    pub dependencies: Vec<String>,
}

/// Celery-specific analysis
#[derive(Debug, Clone)]
pub struct CeleryAnalysis {
    pub tasks: Vec<CeleryTaskInfo>,
    pub brokers: Vec<String>,
    pub result_backends: Vec<String>,
    pub routing: Vec<String>,
    pub monitoring: bool,
}

/// Celery task information
#[derive(Debug, Clone)]
pub struct CeleryTaskInfo {
    pub name: String,
    pub task_type: CeleryTaskType,
    pub retry_policy: Option<String>,
    pub rate_limit: Option<String>,
}

/// Celery task types
#[derive(Debug, Clone)]
pub enum CeleryTaskType {
    Regular,
    Periodic,
    Chain,
    Group,
    Chord,
}

/// Python type hint analysis result
#[derive(Debug, Clone)]
pub struct PythonTypeHintAnalysis {
    pub overall_coverage: f32,
    pub type_coverage_score: TypeCoverageScore,
    pub type_hints_detected: Vec<TypeHintInfo>,
    pub type_safety_issues: Vec<TypeSafetyIssue>,
    pub modern_type_features: Vec<ModernTypeFeature>,
    pub recommendations: Vec<String>,
}

/// Type coverage scoring
#[derive(Debug, Clone)]
pub enum TypeCoverageScore {
    Excellent, // 90%+ coverage
    Good,      // 70-89% coverage
    Fair,      // 50-69% coverage
    Poor,      // 30-49% coverage
    Minimal,   // <30% coverage
}

/// Type hint information
#[derive(Debug, Clone)]
pub struct TypeHintInfo {
    pub location: String,
    pub hint_type: TypeHintType,
    pub complexity: TypeComplexity,
    pub is_generic: bool,
    pub has_constraints: bool,
    pub python_version_required: String,
}

/// Type hint types
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum TypeHintType {
    SimpleType(String),           // int, str, bool
    UnionType(Vec<String>),       // Union[str, int] or str | int
    GenericType(GenericTypeInfo), // List[T], Dict[K, V]
    ProtocolType(String),         // Protocol for structural typing
    LiteralType(Vec<String>),     // Literal['value1', 'value2']
    CallableType(CallableTypeInfo), // Callable[[int, str], bool]
    TypeVarType(TypeVarInfo),     // TypeVar constraints and bounds
    OptionalType(String),         // Optional[str] or str | None
    FinalType(String),            // Final[int]
    TypedDictType(TypedDictInfo), // TypedDict for structured dicts
}

/// Generic type information
#[derive(Debug, Clone)]
pub struct GenericTypeInfo {
    pub base_type: String,      // List, Dict, Set, etc.
    pub type_parameters: Vec<String>, // [T] or [K, V]
    pub is_covariant: bool,
    pub is_contravariant: bool,
}

/// Callable type information
#[derive(Debug, Clone)]
pub struct CallableTypeInfo {
    pub parameter_types: Vec<String>,
    pub return_type: String,
    pub is_async: bool,
}

/// TypeVar information
#[derive(Debug, Clone)]
pub struct TypeVarInfo {
    pub name: String,
    pub bounds: Vec<String>,
    pub constraints: Vec<String>,
    pub covariant: bool,
    pub contravariant: bool,
}

/// TypedDict information
#[derive(Debug, Clone)]
pub struct TypedDictInfo {
    pub name: String,
    pub fields: Vec<TypedDictField>,
    pub total: bool, // Whether all fields are required
}

/// TypedDict field information
#[derive(Debug, Clone)]
pub struct TypedDictField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

/// Type complexity assessment
#[derive(Debug, Clone)]
pub enum TypeComplexity {
    Simple,    // Basic types like int, str
    Moderate,  // Union types, Optional
    Complex,   // Generic types with multiple parameters
    Advanced,  // Complex nested generics, Protocols
}

/// Type safety issues
#[derive(Debug, Clone)]
pub struct TypeSafetyIssue {
    pub issue_type: TypeSafetyIssueType,
    pub severity: TypeSafetySeverity,
    pub location: String,
    pub description: String,
    pub recommendation: String,
}

/// Type safety issue types
#[derive(Debug, Clone)]
pub enum TypeSafetyIssueType {
    AnyTypeOveruse,           // Too many Any types
    MissingTypeHints,         // Functions without type hints
    InconsistentTypes,        // Type inconsistencies
    TypeIgnoreOveruse,        // Too many # type: ignore comments
    WrongTypeHintSyntax,      // Incorrect type hint syntax
    DeprecatedTypingSyntax,   // Using old typing syntax
    UnreachableCode,          // Dead code due to type narrowing
    TypeVarianceIssue,        // Covariance/contravariance problems
}

/// Type safety severity levels
#[derive(Debug, Clone)]
pub enum TypeSafetySeverity {
    Error,   // Type errors that would cause runtime issues
    Warning, // Type inconsistencies that should be addressed
    Info,    // Suggestions for improvement
}

/// Modern type features (Python 3.8+)
#[derive(Debug, Clone)]
pub struct ModernTypeFeature {
    pub feature_type: ModernTypeFeatureType,
    pub python_version: String,
    pub usage_count: usize,
    pub description: String,
    pub is_best_practice: bool,
}

/// Modern type feature types
#[derive(Debug, Clone)]
pub enum ModernTypeFeatureType {
    PositionalOnlyParams,  // def func(arg, /) -> str:
    UnionSyntaxPy310,      // str | int instead of Union[str, int]
    TypedDict,             // TypedDict for structured dictionaries
    FinalType,             // Final[int] = 42
    LiteralType,           // Literal['red', 'green', 'blue']
    ProtocolType,          // Protocol for structural typing
    TypeGuard,             // TypeGuard for type narrowing
    OverloadDecorator,     // @overload for function overloading
    GenericAlias,          // list[int] instead of List[int] (Python 3.9+)
    ParamSpec,             // ParamSpec for callable signatures (Python 3.10+)
    TypeVarTuple,          // TypeVarTuple for variadic generics (Python 3.11+)
}

/// Pattern for type hint detection
#[derive(Debug, Clone)]
struct TypeHintPattern {
    name: String,
    pattern: Regex,
    hint_type: String,
    complexity: TypeComplexity,
    python_version: String,
}

/// Python async/await pattern analysis result
#[derive(Debug, Clone)]
pub struct PythonAsyncAwaitAnalysis {
    pub overall_async_score: i32,
    pub async_functions_detected: Vec<AsyncFunctionInfo>,
    pub await_usage_patterns: Vec<AwaitUsageInfo>,
    pub concurrency_patterns: Vec<ConcurrencyPatternInfo>,
    pub async_performance_issues: Vec<AsyncPerformanceIssue>,
    pub async_security_issues: Vec<AsyncSecurityIssue>,
    pub modern_async_features: Vec<ModernAsyncFeature>,
    pub recommendations: Vec<String>,
}

/// Async function information
#[derive(Debug, Clone)]
pub struct AsyncFunctionInfo {
    pub name: String,
    pub function_type: AsyncFunctionType,
    pub complexity: AsyncComplexity,
    pub coroutine_type: CoroutineType,
    pub error_handling: AsyncErrorHandling,
    pub has_timeout: bool,
    pub uses_context_manager: bool,
    pub location: String,
}

/// Types of async functions
#[derive(Debug, Clone)]
pub enum AsyncFunctionType {
    RegularAsync,      // async def function()
    AsyncGenerator,    // async def with yield
    AsyncContextManager, // __aenter__, __aexit__
    AsyncIterator,     // __aiter__, __anext__
    AsyncProperty,     // @async_property
    AsyncDecorator,    // Decorates with async functionality
}

/// Async function complexity
#[derive(Debug, Clone)]
pub enum AsyncComplexity {
    Simple,    // Single await or simple operations
    Moderate,  // Multiple awaits, basic control flow
    Complex,   // Nested awaits, exception handling
    Advanced,  // Complex concurrency patterns, resource management
}

/// Coroutine type classification
#[derive(Debug, Clone)]
pub enum CoroutineType {
    Native,         // Native Python coroutines
    Framework(String), // Framework-specific (asyncio, trio, curio)
    Generator,      // Generator-based coroutines (deprecated)
    Hybrid,         // Mixed native and framework
}

/// Async error handling assessment
#[derive(Debug, Clone)]
pub enum AsyncErrorHandling {
    None,       // No error handling
    Basic,      // Simple try/catch
    Timeout,    // Includes timeout handling
    Robust,     // Comprehensive error handling with retries
}

/// Await usage information
#[derive(Debug, Clone)]
pub struct AwaitUsageInfo {
    pub location: String,
    pub context: AwaitContext,
    pub usage_pattern: AwaitUsagePattern,
    pub is_valid: bool,
    pub potential_issues: Vec<AwaitIssue>,
}

/// Context where await is used
#[derive(Debug, Clone)]
pub enum AwaitContext {
    AsyncFunction,     // Inside async def
    AsyncGenerator,    // Inside async generator
    AsyncContextManager, // Inside async context manager
    AsyncIterator,     // Inside async iterator
    SyncContext,       // Invalid: inside sync function
    Comprehension,     // Invalid: inside comprehension
    Lambda,           // Invalid: inside lambda
}

/// Await usage patterns
#[derive(Debug, Clone)]
pub enum AwaitUsagePattern {
    SingleAwait,       // Single await expression
    SequentialAwaits,  // Multiple sequential awaits
    ConditionalAwait,  // Await in conditional
    NestedAwait,       // Await inside await
    GatheredAwait,     // Part of asyncio.gather()
    ConcurrentAwait,   // Concurrent execution pattern
}

/// Await usage issues
#[derive(Debug, Clone)]
pub enum AwaitIssue {
    IllegalContext,    // await in illegal context
    MissingAwait,      // Missing await on coroutine
    BlockingCall,      // Blocking call in async context
    SyncInAsync,       // Sync operation in async function
    ResourceLeak,      // Potential resource leak
    TimeoutMissing,    // Missing timeout handling
}

/// Concurrency pattern information
#[derive(Debug, Clone)]
pub struct ConcurrencyPatternInfo {
    pub pattern_type: ConcurrencyPatternType,
    pub usage_quality: ConcurrencyUsageQuality,
    pub performance_impact: AsyncPerformanceImpact,
    pub location: String,
    pub best_practices_followed: bool,
}

/// Types of concurrency patterns
#[derive(Debug, Clone)]
pub enum ConcurrencyPatternType {
    AsyncioGather,     // asyncio.gather() for concurrent execution
    AsyncioWait,       // asyncio.wait() for coordination
    AsyncioQueue,      // asyncio.Queue for producer-consumer
    AsyncioSemaphore,  // asyncio.Semaphore for rate limiting
    AsyncioLock,       // asyncio.Lock for synchronization
    TaskGroup,         // Python 3.11+ TaskGroup
    ConcurrentFutures, // concurrent.futures integration
    AsyncioTimeout,    // asyncio.timeout() context manager
    AsyncioEvent,      // asyncio.Event for coordination
    AsyncioCondition,  // asyncio.Condition for complex coordination
}

/// Quality of concurrency usage
#[derive(Debug, Clone)]
pub enum ConcurrencyUsageQuality {
    Excellent,  // Optimal usage with best practices
    Good,       // Correct usage with minor optimizations possible
    Adequate,   // Functional but suboptimal
    Poor,       // Problematic usage that should be improved
    Dangerous,  // Usage that can cause deadlocks or race conditions
}

/// Performance impact of async patterns
#[derive(Debug, Clone)]
pub enum AsyncPerformanceImpact {
    Positive,   // Improves performance
    Neutral,    // No significant impact
    Negative,   // Reduces performance
    Critical,   // Severely impacts performance
}

/// Async-specific performance issues
#[derive(Debug, Clone)]
pub struct AsyncPerformanceIssue {
    pub issue_type: AsyncPerformanceIssueType,
    pub severity: AsyncIssueSeverity,
    pub location: String,
    pub description: String,
    pub recommendation: String,
    pub estimated_impact: AsyncPerformanceImpact,
}

/// Types of async performance issues
#[derive(Debug, Clone)]
pub enum AsyncPerformanceIssueType {
    BlockingIOInAsync,    // Sync I/O operations in async functions
    EventLoopBlocking,    // Operations that block the event loop
    GILContentionAsync,   // GIL contention in async code
    AwaitInLoop,          // Inefficient await in loop
    MissingConcurrency,   // Sequential execution where concurrent is possible
    ResourceLeakAsync,    // Async resource leaks
    SyncWrapperOveruse,   // Overuse of sync-to-async wrappers
    AsyncioSubprocessSync, // Sync subprocess calls in async context
    DatabaseBlockingAsync, // Blocking database calls in async functions
    SlowAsyncGenerator,   // Inefficient async generators
}

/// Async issue severity levels
#[derive(Debug, Clone)]
pub enum AsyncIssueSeverity {
    Critical,  // Breaks async functionality
    High,      // Significant performance impact
    Medium,    // Moderate impact on performance
    Low,       // Minor optimization opportunity
    Info,      // Best practice suggestion
}

/// Async-specific security issues
#[derive(Debug, Clone)]
pub struct AsyncSecurityIssue {
    pub issue_type: AsyncSecurityIssueType,
    pub severity: AsyncSecuritySeverity,
    pub location: String,
    pub description: String,
    pub recommendation: String,
}

/// Types of async security issues
#[derive(Debug, Clone)]
pub enum AsyncSecurityIssueType {
    AsyncRaceCondition,   // Race conditions in async code
    SharedStateNoLock,    // Shared mutable state without locking
    AsyncTimeoutVuln,     // Missing timeouts enabling DoS
    TaskCancellationLeak, // Improper task cancellation
    AsyncResourceExposure, // Resource exposure through async operations
    EventLoopPoisoning,   // Event loop manipulation attacks
    AsyncPickleVuln,      // Pickle vulnerabilities in async context
    ConcurrentModification, // Concurrent modification without protection
}

/// Async security severity levels
#[derive(Debug, Clone)]
pub enum AsyncSecuritySeverity {
    Critical,  // Exploitable security vulnerability
    High,      // Significant security risk
    Medium,    // Moderate security concern
    Low,       // Minor security consideration
    Info,      // Security best practice
}

/// Modern async features (Python 3.7+)
#[derive(Debug, Clone)]
pub struct ModernAsyncFeature {
    pub feature_type: ModernAsyncFeatureType,
    pub python_version: String,
    pub usage_count: usize,
    pub description: String,
    pub is_best_practice: bool,
}

/// Types of modern async features
#[derive(Debug, Clone)]
pub enum ModernAsyncFeatureType {
    AsyncContextManager,  // async with statements
    TaskGroups,          // Python 3.11+ TaskGroup
    AsyncioTimeout,      // asyncio.timeout() (Python 3.11+)
    AsyncIterators,      // async for loops
    AsyncGenerators,     // async generators with yield
    AsyncComprehensions, // Async list/dict/set comprehensions
    ContextVars,         // contextvars for async context
    AsyncioRun,          // asyncio.run() for main entry point
    AsyncDecorators,     // Custom async decorators
    StreamAPI,           // asyncio streams for I/O
    SubprocessAsync,     // asyncio subprocess for non-blocking process execution
}

/// Pattern for async detection
#[derive(Debug, Clone)]
struct AsyncPattern {
    name: String,
    pattern: Regex,
    pattern_type: String,
    performance_impact: AsyncPerformanceImpact,
}

/// Python-specific analyzer
pub struct PythonAnalyzer {
    decorator_patterns: HashMap<String, Vec<DecoratorPattern>>,
    metaclass_patterns: HashMap<String, Vec<MetaclassPattern>>,
    security_patterns: HashMap<String, Vec<SecurityPattern>>,
    performance_patterns: HashMap<String, Vec<PerformancePattern>>,
    framework_patterns: HashMap<String, Vec<FrameworkPattern>>,
    type_hint_patterns: HashMap<String, Vec<TypeHintPattern>>,
    async_patterns: HashMap<String, Vec<AsyncPattern>>,
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

#[derive(Debug, Clone)]
struct SecurityPattern {
    #[allow(dead_code)]
    name: String,
    pattern: Regex,
    vulnerability_type: VulnerabilityType,
    severity: VulnerabilitySeverity,
    description: String,
}

#[derive(Debug, Clone)]
struct PerformancePattern {
    #[allow(dead_code)]
    name: String,
    pattern: Regex,
    optimization_type: OptimizationType,
    impact_level: ImpactLevel,
    description: String,
}

#[derive(Debug, Clone)]
struct FrameworkPattern {
    #[allow(dead_code)]
    name: String,
    pattern: Regex,
    framework: String,
    features: Vec<String>,
    confidence: f32,
}

impl PythonAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            decorator_patterns: HashMap::new(),
            metaclass_patterns: HashMap::new(),
            security_patterns: HashMap::new(),
            performance_patterns: HashMap::new(),
            framework_patterns: HashMap::new(),
            type_hint_patterns: HashMap::new(),
            async_patterns: HashMap::new(),
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

        // Security patterns
        let security_patterns = vec![
            SecurityPattern {
                name: "SQL Injection Risk".to_string(),
                pattern: Regex::new(r#"(?:execute|raw)\s*\(\s*[f"'].*%.*[f"']"#).unwrap(),
                vulnerability_type: VulnerabilityType::SqlInjection,
                severity: VulnerabilitySeverity::High,
                description: "Potential SQL injection via string formatting".to_string(),
            },
            SecurityPattern {
                name: "Command Injection Risk".to_string(),
                pattern: Regex::new(r"(?:subprocess|os\.system)\s*\(.*(?:input|request)").unwrap(),
                vulnerability_type: VulnerabilityType::CommandInjection,
                severity: VulnerabilitySeverity::Critical,
                description: "Command injection via user input".to_string(),
            },
            SecurityPattern {
                name: "Unsafe Pickle Usage".to_string(),
                pattern: Regex::new(r"pickle\.loads?\s*\(").unwrap(),
                vulnerability_type: VulnerabilityType::DeserializationAttack,
                severity: VulnerabilitySeverity::High,
                description: "Unsafe pickle deserialization".to_string(),
            },
            SecurityPattern {
                name: "Hardcoded Secrets".to_string(),
                pattern: Regex::new(r#"(?:password|secret|key|token)\s*=\s*[f"'][^"']*[f"']"#)
                    .unwrap(),
                vulnerability_type: VulnerabilityType::HardcodedSecrets,
                severity: VulnerabilitySeverity::Medium,
                description: "Hardcoded credentials in source code".to_string(),
            },
        ];
        self.security_patterns
            .insert("vulnerabilities".to_string(), security_patterns);

        // Performance patterns
        let performance_patterns = vec![
            PerformancePattern {
                name: "List Comprehension Optimization".to_string(),
                pattern: Regex::new(r"\[.*for.*in.*\]").unwrap(),
                optimization_type: OptimizationType::ListComprehension,
                impact_level: ImpactLevel::Medium,
                description: "Efficient list comprehension usage".to_string(),
            },
            PerformancePattern {
                name: "Generator Usage".to_string(),
                pattern: Regex::new(r"\(.*for.*in.*\)").unwrap(),
                optimization_type: OptimizationType::GeneratorUsage,
                impact_level: ImpactLevel::High,
                description: "Memory-efficient generator expression".to_string(),
            },
            PerformancePattern {
                name: "Caching Implementation".to_string(),
                pattern: Regex::new(r"@(?:lru_cache|cache)").unwrap(),
                optimization_type: OptimizationType::CachingImplementation,
                impact_level: ImpactLevel::High,
                description: "Function result caching".to_string(),
            },
            PerformancePattern {
                name: "Async/Await Usage".to_string(),
                pattern: Regex::new(r"async\s+def|await\s+").unwrap(),
                optimization_type: OptimizationType::AsyncAwaitUsage,
                impact_level: ImpactLevel::High,
                description: "Asynchronous programming patterns".to_string(),
            },
        ];
        self.performance_patterns
            .insert("optimizations".to_string(), performance_patterns);

        // Framework patterns
        let framework_patterns = vec![
            FrameworkPattern {
                name: "Django Framework".to_string(),
                pattern: Regex::new(r"from\s+django|import\s+django").unwrap(),
                framework: "Django".to_string(),
                features: vec![
                    "Models".to_string(),
                    "Views".to_string(),
                    "Templates".to_string(),
                ],
                confidence: 0.9,
            },
            FrameworkPattern {
                name: "Flask Framework".to_string(),
                pattern: Regex::new(r"from\s+flask|import\s+flask").unwrap(),
                framework: "Flask".to_string(),
                features: vec![
                    "Routes".to_string(),
                    "Blueprints".to_string(),
                    "Templates".to_string(),
                ],
                confidence: 0.9,
            },
            FrameworkPattern {
                name: "FastAPI Framework".to_string(),
                pattern: Regex::new(r"from\s+fastapi|import\s+fastapi").unwrap(),
                framework: "FastAPI".to_string(),
                features: vec![
                    "API Routes".to_string(),
                    "Dependency Injection".to_string(),
                    "Validation".to_string(),
                ],
                confidence: 0.9,
            },
            FrameworkPattern {
                name: "Pytest Framework".to_string(),
                pattern: Regex::new(r"import\s+pytest|@pytest").unwrap(),
                framework: "Pytest".to_string(),
                features: vec![
                    "Fixtures".to_string(),
                    "Parametrization".to_string(),
                    "Markers".to_string(),
                ],
                confidence: 0.8,
            },
        ];
        self.framework_patterns
            .insert("web_frameworks".to_string(), framework_patterns);

        // Type hint patterns
        let type_hint_patterns = vec![
            TypeHintPattern {
                name: "Union Type".to_string(),
                pattern: Regex::new(r"Union\[([^]]+)\]").unwrap(),
                hint_type: "union".to_string(),
                complexity: TypeComplexity::Moderate,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "Union Type (PEP 604)".to_string(),
                pattern: Regex::new(r"(\w+)\s*\|\s*(\w+)").unwrap(),
                hint_type: "union_new".to_string(),
                complexity: TypeComplexity::Moderate,
                python_version: "3.10+".to_string(),
            },
            TypeHintPattern {
                name: "Optional Type".to_string(),
                pattern: Regex::new(r"Optional\[([^]]+)\]").unwrap(),
                hint_type: "optional".to_string(),
                complexity: TypeComplexity::Moderate,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "Generic List".to_string(),
                pattern: Regex::new(r"List\[([^]]+)\]").unwrap(),
                hint_type: "generic_list".to_string(),
                complexity: TypeComplexity::Complex,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "Generic Dict".to_string(),
                pattern: Regex::new(r"Dict\[([^]]+),\s*([^]]+)\]").unwrap(),
                hint_type: "generic_dict".to_string(),
                complexity: TypeComplexity::Complex,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "Callable Type".to_string(),
                pattern: Regex::new(r"Callable\[\[([^]]*)\],\s*([^]]+)\]").unwrap(),
                hint_type: "callable".to_string(),
                complexity: TypeComplexity::Advanced,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "TypeVar".to_string(),
                pattern: Regex::new(r#"TypeVar\s*\(\s*["'](\w+)["']"#).unwrap(),
                hint_type: "typevar".to_string(),
                complexity: TypeComplexity::Advanced,
                python_version: "3.5+".to_string(),
            },
            TypeHintPattern {
                name: "Protocol".to_string(),
                pattern: Regex::new(r"class\s+\w+\s*\([^)]*Protocol[^)]*\)").unwrap(),
                hint_type: "protocol".to_string(),
                complexity: TypeComplexity::Advanced,
                python_version: "3.8+".to_string(),
            },
            TypeHintPattern {
                name: "Literal Type".to_string(),
                pattern: Regex::new(r"Literal\[([^]]+)\]").unwrap(),
                hint_type: "literal".to_string(),
                complexity: TypeComplexity::Moderate,
                python_version: "3.8+".to_string(),
            },
            TypeHintPattern {
                name: "Final Type".to_string(),
                pattern: Regex::new(r"Final\[([^]]+)\]").unwrap(),
                hint_type: "final".to_string(),
                complexity: TypeComplexity::Moderate,
                python_version: "3.8+".to_string(),
            },
            TypeHintPattern {
                name: "TypedDict".to_string(),
                pattern: Regex::new(r"class\s+(\w+)\s*\(\s*TypedDict\s*\)").unwrap(),
                hint_type: "typeddict".to_string(),
                complexity: TypeComplexity::Complex,
                python_version: "3.8+".to_string(),
            },
            TypeHintPattern {
                name: "Generic Alias (Python 3.9+)".to_string(),
                pattern: Regex::new(r"\b(list|dict|set|tuple)\s*\[([^]]+)\]").unwrap(),
                hint_type: "generic_alias".to_string(),
                complexity: TypeComplexity::Simple,
                python_version: "3.9+".to_string(),
            },
        ];
        self.type_hint_patterns
            .insert("type_hints".to_string(), type_hint_patterns);

        // Async/await patterns
        let async_patterns = vec![
            AsyncPattern {
                name: "Async Function".to_string(),
                pattern: Regex::new(r"async\s+def\s+(\w+)").unwrap(),
                pattern_type: "function".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async Generator".to_string(),
                pattern: Regex::new(r"async\s+def\s+\w+.*yield").unwrap(),
                pattern_type: "generator".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async Context Manager".to_string(),
                pattern: Regex::new(r"async\s+def\s+__aenter__|async\s+def\s+__aexit__").unwrap(),
                pattern_type: "context_manager".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async Iterator".to_string(),
                pattern: Regex::new(r"async\s+def\s+__aiter__|async\s+def\s+__anext__").unwrap(),
                pattern_type: "iterator".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Await Expression".to_string(),
                pattern: Regex::new(r"await\s+(\w+[\w\.\(\)]*)+").unwrap(),
                pattern_type: "await".to_string(),
                performance_impact: AsyncPerformanceImpact::Neutral,
            },
            AsyncPattern {
                name: "Asyncio Gather".to_string(),
                pattern: Regex::new(r"asyncio\.gather\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Wait".to_string(),
                pattern: Regex::new(r"asyncio\.wait\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Queue".to_string(),
                pattern: Regex::new(r"asyncio\.Queue\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Semaphore".to_string(),
                pattern: Regex::new(r"asyncio\.Semaphore\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Lock".to_string(),
                pattern: Regex::new(r"asyncio\.Lock\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "TaskGroup".to_string(),
                pattern: Regex::new(r"asyncio\.TaskGroup\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Timeout".to_string(),
                pattern: Regex::new(r"asyncio\.timeout\s*\(").unwrap(),
                pattern_type: "timeout".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async With Statement".to_string(),
                pattern: Regex::new(r"async\s+with\s+").unwrap(),
                pattern_type: "context_manager".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async For Loop".to_string(),
                pattern: Regex::new(r"async\s+for\s+").unwrap(),
                pattern_type: "iterator".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Blocking IO in Async".to_string(),
                pattern: Regex::new(r"(?:open|input|print)\s*\(.*\).*await|await.*(?:open|input|print)\s*\(").unwrap(),
                pattern_type: "performance_issue".to_string(),
                performance_impact: AsyncPerformanceImpact::Critical,
            },
        ];
        self.async_patterns.insert("functions".to_string(), async_patterns);

        let concurrency_patterns = vec![
            AsyncPattern {
                name: "Concurrent Futures".to_string(),
                pattern: Regex::new(r"concurrent\.futures").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Event".to_string(),
                pattern: Regex::new(r"asyncio\.Event\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Asyncio Condition".to_string(),
                pattern: Regex::new(r"asyncio\.Condition\s*\(").unwrap(),
                pattern_type: "concurrency".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
        ];
        self.async_patterns.insert("concurrency".to_string(), concurrency_patterns);

        let modern_async_patterns = vec![
            AsyncPattern {
                name: "Asyncio Run".to_string(),
                pattern: Regex::new(r"asyncio\.run\s*\(").unwrap(),
                pattern_type: "modern".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Context Variables".to_string(),
                pattern: Regex::new(r"contextvars\.ContextVar").unwrap(),
                pattern_type: "modern".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
            AsyncPattern {
                name: "Async Comprehension".to_string(),
                pattern: Regex::new(r"\[.*async\s+for.*\]|\{.*async\s+for.*\}").unwrap(),
                pattern_type: "modern".to_string(),
                performance_impact: AsyncPerformanceImpact::Positive,
            },
        ];
        self.async_patterns.insert("modern".to_string(), modern_async_patterns);
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

        metaclass_pattern
            .captures(bases_str)
            .map(|captures| captures.get(1).unwrap().as_str().to_string())
    }

    /// Analyze Python security vulnerabilities and features
    pub fn analyze_security(&self, content: &str) -> Result<PythonSecurityAssessment> {
        let mut vulnerabilities = Vec::new();
        let mut security_features = Vec::new();

        // Detect vulnerabilities
        for patterns in self.security_patterns.values() {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();

                    vulnerabilities.push(SecurityVulnerability {
                        vulnerability_type: pattern.vulnerability_type.clone(),
                        severity: pattern.severity.clone(),
                        description: pattern.description.clone(),
                        location: full_match.to_string(),
                        recommendation: self
                            .get_security_recommendation(&pattern.vulnerability_type),
                    });
                }
            }
        }

        // Detect security features
        if content.contains("bcrypt") || content.contains("hashlib") {
            security_features.push(SecurityFeature {
                feature_type: SecurityFeatureType::DataEncryption,
                implementation_quality: ImplementationQuality::Good,
                description: "Password hashing implementation detected".to_string(),
            });
        }

        if content.contains("@csrf_exempt") || content.contains("CsrfViewMiddleware") {
            security_features.push(SecurityFeature {
                feature_type: SecurityFeatureType::CsrfProtection,
                implementation_quality: ImplementationQuality::Good,
                description: "CSRF protection implementation detected".to_string(),
            });
        }

        if content.contains("pydantic") || content.contains("marshmallow") {
            security_features.push(SecurityFeature {
                feature_type: SecurityFeatureType::InputValidation,
                implementation_quality: ImplementationQuality::Good,
                description: "Input validation framework detected".to_string(),
            });
        }

        // Determine overall security level
        let level = self.determine_security_level(&vulnerabilities, &security_features);
        let recommendations =
            self.get_security_recommendations(&vulnerabilities, &security_features);

        Ok(PythonSecurityAssessment {
            level,
            vulnerabilities_detected: vulnerabilities,
            security_features,
            recommendations,
        })
    }

    /// Analyze Python performance patterns and issues
    pub fn analyze_performance(&self, content: &str) -> Result<PythonPerformanceAnalysis> {
        let mut optimizations = Vec::new();
        let mut issues = Vec::new();

        // Detect performance optimizations
        for patterns in self.performance_patterns.values() {
            for pattern in patterns {
                for _captures in pattern.pattern.captures_iter(content) {
                    optimizations.push(PerformanceOptimization {
                        optimization_type: pattern.optimization_type.clone(),
                        impact_level: pattern.impact_level.clone(),
                        description: pattern.description.clone(),
                        best_practices_followed: true,
                    });
                }
            }
        }

        // Detect performance issues
        if content.contains("for") && content.contains("for") && content.matches("for").count() > 1
        {
            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::InEfficientLoops,
                severity: IssueSeverity::Medium,
                description: "Nested loops detected - consider optimization".to_string(),
                recommendation: "Use list comprehensions or optimize algorithm".to_string(),
            });
        }

        if content.contains("def __del__") {
            issues.push(PerformanceIssue {
                issue_type: PerformanceIssueType::MemoryLeaks,
                severity: IssueSeverity::High,
                description: "Manual destructor detected - potential memory management issue"
                    .to_string(),
                recommendation: "Use context managers or weak references".to_string(),
            });
        }

        // Calculate overall score
        let overall_score = self.calculate_performance_score(&optimizations, &issues);
        let recommendations = self.get_performance_recommendations(&optimizations, &issues);

        Ok(PythonPerformanceAnalysis {
            overall_score,
            optimizations_detected: optimizations,
            performance_issues: issues,
            recommendations,
        })
    }

    /// Analyze Python frameworks with enhanced details
    pub fn analyze_frameworks(&self, content: &str) -> Result<Vec<PythonFrameworkInfo>> {
        let mut frameworks = Vec::new();

        for patterns in self.framework_patterns.values() {
            for pattern in patterns {
                if pattern.pattern.is_match(content) {
                    let framework_specific_analysis = match pattern.framework.as_str() {
                        "Django" => FrameworkSpecificAnalysis::Django(
                            self.analyze_django_specifics(content),
                        ),
                        "Flask" => {
                            FrameworkSpecificAnalysis::Flask(self.analyze_flask_specifics(content))
                        }
                        "FastAPI" => FrameworkSpecificAnalysis::FastAPI(
                            self.analyze_fastapi_specifics(content),
                        ),
                        "Pytest" => FrameworkSpecificAnalysis::Pytest(
                            self.analyze_pytest_specifics(content),
                        ),
                        _ => continue,
                    };

                    frameworks.push(PythonFrameworkInfo {
                        name: pattern.framework.clone(),
                        confidence: pattern.confidence,
                        version_detected: None, // Could be enhanced to detect versions
                        features_used: pattern.features.clone(),
                        best_practices: self.get_framework_best_practices(&pattern.framework),
                        framework_specific_analysis,
                    });
                }
            }
        }

        Ok(frameworks)
    }

    /// Analyze Python type hints comprehensively
    pub fn analyze_type_hints(&self, content: &str) -> Result<PythonTypeHintAnalysis> {
        let mut type_hints_detected = Vec::new();
        let mut type_safety_issues = Vec::new();
        let mut modern_type_features = Vec::new();

        // Detect type hints using patterns
        for patterns in self.type_hint_patterns.values() {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();
                    
                    let hint_type = self.parse_type_hint_type(&pattern.hint_type, &captures);
                    let is_generic = self.is_generic_type(&pattern.hint_type);
                    let has_constraints = self.has_type_constraints(&pattern.hint_type);

                    type_hints_detected.push(TypeHintInfo {
                        location: full_match.to_string(),
                        hint_type,
                        complexity: pattern.complexity.clone(),
                        is_generic,
                        has_constraints,
                        python_version_required: pattern.python_version.clone(),
                    });

                    // Check for modern type features
                    if pattern.python_version.starts_with("3.8")
                        || pattern.python_version.starts_with("3.9")
                        || pattern.python_version.starts_with("3.10")
                    {
                        let feature_type = self.get_modern_feature_type(&pattern.hint_type);
                        if let Some(feature_type) = feature_type {
                            modern_type_features.push(ModernTypeFeature {
                                feature_type,
                                python_version: pattern.python_version.clone(),
                                usage_count: 1,
                                description: format!("Modern type feature: {}", pattern.name),
                                is_best_practice: true,
                            });
                        }
                    }
                }
            }
        }

        // Detect type safety issues
        self.detect_type_safety_issues(content, &mut type_safety_issues);

        // Calculate type coverage
        let overall_coverage = self.calculate_type_coverage(content, &type_hints_detected);
        let type_coverage_score = self.get_coverage_score(overall_coverage);

        // Generate recommendations
        let recommendations = self.get_type_hint_recommendations(
            &type_hints_detected,
            &type_safety_issues,
            overall_coverage,
        );

        Ok(PythonTypeHintAnalysis {
            overall_coverage,
            type_coverage_score,
            type_hints_detected,
            type_safety_issues,
            modern_type_features,
            recommendations,
        })
    }

    /// Analyze Python async/await patterns comprehensively
    pub fn analyze_async_await(&self, content: &str) -> Result<PythonAsyncAwaitAnalysis> {
        let mut async_functions_detected = Vec::new();
        let mut await_usage_patterns = Vec::new();
        let mut concurrency_patterns = Vec::new();
        let mut async_performance_issues = Vec::new();
        let mut async_security_issues = Vec::new();
        let mut modern_async_features = Vec::new();

        // Analyze async functions
        self.analyze_async_functions(content, &mut async_functions_detected);

        // Analyze await usage patterns
        self.analyze_await_usage(content, &mut await_usage_patterns);

        // Analyze concurrency patterns
        self.analyze_concurrency_patterns(content, &mut concurrency_patterns);

        // Detect async performance issues
        self.detect_async_performance_issues(content, &mut async_performance_issues);

        // Detect async security issues
        self.detect_async_security_issues(content, &mut async_security_issues);

        // Detect modern async features
        self.detect_modern_async_features(content, &mut modern_async_features);

        // Calculate overall async score
        let overall_async_score = self.calculate_async_score(
            &async_functions_detected,
            &concurrency_patterns,
            &async_performance_issues,
            &async_security_issues,
        );

        // Generate recommendations
        let recommendations = self.get_async_recommendations(
            &async_functions_detected,
            &await_usage_patterns,
            &concurrency_patterns,
            &async_performance_issues,
            &async_security_issues,
        );

        Ok(PythonAsyncAwaitAnalysis {
            overall_async_score,
            async_functions_detected,
            await_usage_patterns,
            concurrency_patterns,
            async_performance_issues,
            async_security_issues,
            modern_async_features,
            recommendations,
        })
    }

    /// Analyze async functions in detail
    fn analyze_async_functions(&self, content: &str, async_functions: &mut Vec<AsyncFunctionInfo>) {
        for patterns in self.async_patterns.values() {
            for pattern in patterns {
                if pattern.pattern_type == "function" || pattern.pattern_type == "generator" 
                    || pattern.pattern_type == "context_manager" || pattern.pattern_type == "iterator" {
                    
                    for captures in pattern.pattern.captures_iter(content) {
                        let full_match = captures.get(0).unwrap().as_str();
                        let function_name = captures.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "anonymous".to_string());

                        let function_type = self.determine_async_function_type(&pattern.name, full_match);
                        let complexity = self.assess_async_complexity(content, full_match);
                        let coroutine_type = self.classify_coroutine_type(content);
                        let error_handling = self.assess_error_handling(content, full_match);
                        let has_timeout = self.has_timeout_handling(content, full_match);
                        let uses_context_manager = self.uses_async_context_manager(content, full_match);

                        async_functions.push(AsyncFunctionInfo {
                            name: function_name,
                            function_type,
                            complexity,
                            coroutine_type,
                            error_handling,
                            has_timeout,
                            uses_context_manager,
                            location: full_match.to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Analyze await usage patterns
    fn analyze_await_usage(&self, content: &str, await_patterns: &mut Vec<AwaitUsageInfo>) {
        let await_pattern = Regex::new(r"await\s+([^;\n]+)").unwrap();
        
        for captures in await_pattern.captures_iter(content) {
            let full_match = captures.get(0).unwrap().as_str();
            let await_expr = captures.get(1).unwrap().as_str();

            let context = self.determine_await_context(content, full_match);
            let usage_pattern = self.classify_await_usage_pattern(content, await_expr);
            let is_valid = self.validate_await_usage(&context);
            let potential_issues = self.detect_await_issues(content, await_expr, &context);

            await_patterns.push(AwaitUsageInfo {
                location: full_match.to_string(),
                context,
                usage_pattern,
                is_valid,
                potential_issues,
            });
        }
    }

    /// Analyze concurrency patterns
    fn analyze_concurrency_patterns(&self, content: &str, concurrency_patterns: &mut Vec<ConcurrencyPatternInfo>) {
        for patterns in self.async_patterns.values() {
            for pattern in patterns {
                if pattern.pattern_type == "concurrency" {
                    for captures in pattern.pattern.captures_iter(content) {
                        let full_match = captures.get(0).unwrap().as_str();

                        let pattern_type = self.map_to_concurrency_pattern_type(&pattern.name);
                        let usage_quality = self.assess_concurrency_usage_quality(content, full_match);
                        let best_practices_followed = self.check_concurrency_best_practices(content, full_match);

                        concurrency_patterns.push(ConcurrencyPatternInfo {
                            pattern_type,
                            usage_quality,
                            performance_impact: pattern.performance_impact.clone(),
                            location: full_match.to_string(),
                            best_practices_followed,
                        });
                    }
                }
            }
        }
    }

    /// Detect async performance issues
    fn detect_async_performance_issues(&self, content: &str, issues: &mut Vec<AsyncPerformanceIssue>) {
        // Detect blocking IO in async functions - look for blocking calls within async function bodies
        if content.contains("async def") && (content.contains("time.sleep") || content.contains("open(") || content.contains("input(")) {
            issues.push(AsyncPerformanceIssue {
                issue_type: AsyncPerformanceIssueType::BlockingIOInAsync,
                severity: AsyncIssueSeverity::High,
                location: "Async function with blocking operations".to_string(),
                description: "Blocking I/O operation in async function".to_string(),
                recommendation: "Use async I/O operations or run_in_executor".to_string(),
                estimated_impact: AsyncPerformanceImpact::Critical,
            });
        }

        // Detect await in loops
        let await_loop_pattern = Regex::new(r"for\s+.*?:\s*.*?await\s+").unwrap();
        for captures in await_loop_pattern.captures_iter(content) {
            let full_match = captures.get(0).unwrap().as_str();
            issues.push(AsyncPerformanceIssue {
                issue_type: AsyncPerformanceIssueType::AwaitInLoop,
                severity: AsyncIssueSeverity::Medium,
                location: full_match.to_string(),
                description: "Sequential await in loop - consider asyncio.gather()".to_string(),
                recommendation: "Use asyncio.gather() or asyncio.as_completed() for concurrent execution".to_string(),
                estimated_impact: AsyncPerformanceImpact::Negative,
            });
        }

        // Detect missing concurrency opportunities
        let sequential_await_pattern = Regex::new(r"await\s+\w+.*\n.*await\s+\w+").unwrap();
        for captures in sequential_await_pattern.captures_iter(content) {
            let full_match = captures.get(0).unwrap().as_str();
            issues.push(AsyncPerformanceIssue {
                issue_type: AsyncPerformanceIssueType::MissingConcurrency,
                severity: AsyncIssueSeverity::Medium,
                location: full_match.to_string(),
                description: "Sequential await calls could be concurrent".to_string(),
                recommendation: "Consider using asyncio.gather() for independent operations".to_string(),
                estimated_impact: AsyncPerformanceImpact::Negative,
            });
        }
    }

    /// Detect async security issues
    fn detect_async_security_issues(&self, content: &str, issues: &mut Vec<AsyncSecurityIssue>) {
        // Detect missing timeouts
        let await_pattern = Regex::new(r"await\s+").unwrap();
        let timeout_count = content.matches("asyncio.wait_for").count() + content.matches("asyncio.timeout").count();
        let await_count = await_pattern.find_iter(content).count();

        if await_count > timeout_count + 2 {
            issues.push(AsyncSecurityIssue {
                issue_type: AsyncSecurityIssueType::AsyncTimeoutVuln,
                severity: AsyncSecuritySeverity::Medium,
                location: "Multiple async operations".to_string(),
                description: "Missing timeout handling in async operations".to_string(),
                recommendation: "Add timeouts to prevent DoS attacks".to_string(),
            });
        }

        // Detect shared state without locking
        let shared_state_pattern = Regex::new(r"(?:global|class)\s+\w+.*=.*\n.*async\s+def.*\w+.*=").unwrap();
        for captures in shared_state_pattern.captures_iter(content) {
            let full_match = captures.get(0).unwrap().as_str();
            if !content.contains("asyncio.Lock") && !content.contains("asyncio.Semaphore") {
                issues.push(AsyncSecurityIssue {
                    issue_type: AsyncSecurityIssueType::SharedStateNoLock,
                    severity: AsyncSecuritySeverity::High,
                    location: full_match.to_string(),
                    description: "Shared mutable state without proper locking".to_string(),
                    recommendation: "Use asyncio.Lock or asyncio.Semaphore for thread safety".to_string(),
                });
            }
        }

        // Detect race condition patterns
        if content.contains("asyncio.gather") && !content.contains("asyncio.Lock") && content.matches("=").count() > 3 {
            issues.push(AsyncSecurityIssue {
                issue_type: AsyncSecurityIssueType::AsyncRaceCondition,
                severity: AsyncSecuritySeverity::Medium,
                location: "Concurrent operations".to_string(),
                description: "Potential race condition in concurrent operations".to_string(),
                recommendation: "Review shared resource access and add synchronization".to_string(),
            });
        }
    }

    /// Detect modern async features
    fn detect_modern_async_features(&self, content: &str, features: &mut Vec<ModernAsyncFeature>) {
        let modern_patterns = &[
            ("async with", ModernAsyncFeatureType::AsyncContextManager, "3.7+"),
            ("asyncio.TaskGroup", ModernAsyncFeatureType::TaskGroups, "3.11+"),
            ("asyncio.timeout", ModernAsyncFeatureType::AsyncioTimeout, "3.11+"),
            ("async for", ModernAsyncFeatureType::AsyncIterators, "3.7+"),
            ("contextvars", ModernAsyncFeatureType::ContextVars, "3.7+"),
            ("asyncio.run", ModernAsyncFeatureType::AsyncioRun, "3.7+"),
        ];

        for (pattern_str, feature_type, version) in modern_patterns {
            let count = content.matches(pattern_str).count();
            if count > 0 {
                features.push(ModernAsyncFeature {
                    feature_type: feature_type.clone(),
                    python_version: version.to_string(),
                    usage_count: count,
                    description: format!("Modern async feature: {}", pattern_str),
                    is_best_practice: true,
                });
            }
        }

        // Detect async comprehensions
        let async_comp_pattern = Regex::new(r"\[.*async\s+for.*\]|\{.*async\s+for.*\}").unwrap();
        let comp_count = async_comp_pattern.find_iter(content).count();
        if comp_count > 0 {
            features.push(ModernAsyncFeature {
                feature_type: ModernAsyncFeatureType::AsyncComprehensions,
                python_version: "3.6+".to_string(),
                usage_count: comp_count,
                description: "Async comprehensions for concurrent iteration".to_string(),
                is_best_practice: true,
            });
        }
    }

    /// Helper methods for async analysis
    fn determine_async_function_type(&self, pattern_name: &str, _full_match: &str) -> AsyncFunctionType {
        match pattern_name {
            "Async Function" => AsyncFunctionType::RegularAsync,
            "Async Generator" => AsyncFunctionType::AsyncGenerator,
            "Async Context Manager" => AsyncFunctionType::AsyncContextManager,
            "Async Iterator" => AsyncFunctionType::AsyncIterator,
            _ => AsyncFunctionType::RegularAsync,
        }
    }

    fn assess_async_complexity(&self, _content: &str, function_match: &str) -> AsyncComplexity {
        let await_count = function_match.matches("await").count();
        let try_count = function_match.matches("try").count();
        let gather_count = function_match.matches("gather").count();

        match (await_count, try_count, gather_count) {
            (0..=1, 0, 0) => AsyncComplexity::Simple,
            (2..=3, 0..=1, 0..=1) => AsyncComplexity::Moderate,
            (4..=6, 1..=2, 0..=2) => AsyncComplexity::Complex,
            _ => AsyncComplexity::Advanced,
        }
    }

    fn classify_coroutine_type(&self, content: &str) -> CoroutineType {
        if content.contains("asyncio") {
            CoroutineType::Framework("asyncio".to_string())
        } else if content.contains("trio") {
            CoroutineType::Framework("trio".to_string())
        } else if content.contains("curio") {
            CoroutineType::Framework("curio".to_string())
        } else if content.contains("yield from") {
            CoroutineType::Generator
        } else {
            CoroutineType::Native
        }
    }

    fn assess_error_handling(&self, _content: &str, function_match: &str) -> AsyncErrorHandling {
        if function_match.contains("asyncio.timeout") || function_match.contains("asyncio.wait_for") {
            AsyncErrorHandling::Robust
        } else if function_match.contains("timeout") {
            AsyncErrorHandling::Timeout
        } else if function_match.contains("try") && function_match.contains("except") {
            AsyncErrorHandling::Basic
        } else {
            AsyncErrorHandling::None
        }
    }

    fn has_timeout_handling(&self, _content: &str, function_match: &str) -> bool {
        function_match.contains("timeout") || function_match.contains("asyncio.wait_for")
    }

    fn uses_async_context_manager(&self, _content: &str, function_match: &str) -> bool {
        function_match.contains("async with")
    }

    fn determine_await_context(&self, content: &str, await_match: &str) -> AwaitContext {
        // Simplified context detection - in practice, would need more sophisticated parsing
        if content.contains("async def") {
            if await_match.contains("__aenter__") || await_match.contains("__aexit__") {
                AwaitContext::AsyncContextManager
            } else if await_match.contains("__aiter__") || await_match.contains("__anext__") {
                AwaitContext::AsyncIterator
            } else if await_match.contains("yield") {
                AwaitContext::AsyncGenerator
            } else {
                AwaitContext::AsyncFunction
            }
        } else if await_match.contains("[") && await_match.contains("for") {
            AwaitContext::Comprehension
        } else if await_match.contains("lambda") {
            AwaitContext::Lambda
        } else {
            AwaitContext::SyncContext
        }
    }

    fn classify_await_usage_pattern(&self, content: &str, await_expr: &str) -> AwaitUsagePattern {
        if content.contains("asyncio.gather") {
            AwaitUsagePattern::GatheredAwait
        } else if await_expr.contains("await") {
            AwaitUsagePattern::NestedAwait
        } else if content.contains("if") && await_expr.contains("await") {
            AwaitUsagePattern::ConditionalAwait
        } else if content.matches("await").count() > 1 {
            AwaitUsagePattern::SequentialAwaits
        } else {
            AwaitUsagePattern::SingleAwait
        }
    }

    fn validate_await_usage(&self, context: &AwaitContext) -> bool {
        !matches!(context, AwaitContext::SyncContext | AwaitContext::Comprehension | AwaitContext::Lambda)
    }

    fn detect_await_issues(&self, content: &str, await_expr: &str, context: &AwaitContext) -> Vec<AwaitIssue> {
        let mut issues = Vec::new();

        if !self.validate_await_usage(context) {
            issues.push(AwaitIssue::IllegalContext);
        }

        if await_expr.contains("open(") || await_expr.contains("input(") {
            issues.push(AwaitIssue::BlockingCall);
        }

        if !content.contains("timeout") && !content.contains("asyncio.wait_for") {
            issues.push(AwaitIssue::TimeoutMissing);
        }

        issues
    }

    fn map_to_concurrency_pattern_type(&self, pattern_name: &str) -> ConcurrencyPatternType {
        match pattern_name {
            "Asyncio Gather" => ConcurrencyPatternType::AsyncioGather,
            "Asyncio Wait" => ConcurrencyPatternType::AsyncioWait,
            "Asyncio Queue" => ConcurrencyPatternType::AsyncioQueue,
            "Asyncio Semaphore" => ConcurrencyPatternType::AsyncioSemaphore,
            "Asyncio Lock" => ConcurrencyPatternType::AsyncioLock,
            "TaskGroup" => ConcurrencyPatternType::TaskGroup,
            "Concurrent Futures" => ConcurrencyPatternType::ConcurrentFutures,
            "Asyncio Timeout" => ConcurrencyPatternType::AsyncioTimeout,
            "Asyncio Event" => ConcurrencyPatternType::AsyncioEvent,
            "Asyncio Condition" => ConcurrencyPatternType::AsyncioCondition,
            _ => ConcurrencyPatternType::AsyncioGather,
        }
    }

    fn assess_concurrency_usage_quality(&self, content: &str, pattern_match: &str) -> ConcurrencyUsageQuality {
        let has_error_handling = pattern_match.contains("try") || pattern_match.contains("except");
        let has_timeout = pattern_match.contains("timeout") || content.contains("asyncio.wait_for");
        let has_proper_cleanup = pattern_match.contains("finally") || pattern_match.contains("async with");

        match (has_error_handling, has_timeout, has_proper_cleanup) {
            (true, true, true) => ConcurrencyUsageQuality::Excellent,
            (true, true, false) | (true, false, true) => ConcurrencyUsageQuality::Good,
            (true, false, false) | (false, true, false) | (false, true, true) => ConcurrencyUsageQuality::Adequate,
            (false, false, true) => ConcurrencyUsageQuality::Poor,
            (false, false, false) => ConcurrencyUsageQuality::Dangerous,
        }
    }

    fn check_concurrency_best_practices(&self, content: &str, _pattern_match: &str) -> bool {
        content.contains("async with") && 
        (content.contains("timeout") || content.contains("asyncio.wait_for")) &&
        content.contains("try")
    }

    fn calculate_async_score(
        &self,
        async_functions: &[AsyncFunctionInfo],
        concurrency_patterns: &[ConcurrencyPatternInfo],
        performance_issues: &[AsyncPerformanceIssue],
        security_issues: &[AsyncSecurityIssue],
    ) -> i32 {
        let base_score = 50;
        
        // Bonus for async functions
        let async_bonus = async_functions.len() as i32 * 5;
        
        // Bonus for good concurrency patterns
        let concurrency_bonus = concurrency_patterns.iter()
            .map(|p| match p.usage_quality {
                ConcurrencyUsageQuality::Excellent => 10,
                ConcurrencyUsageQuality::Good => 7,
                ConcurrencyUsageQuality::Adequate => 4,
                ConcurrencyUsageQuality::Poor => 1,
                ConcurrencyUsageQuality::Dangerous => -5,
            })
            .sum::<i32>();

        // Penalty for issues
        let performance_penalty = performance_issues.iter()
            .map(|i| match i.severity {
                AsyncIssueSeverity::Critical => 20,
                AsyncIssueSeverity::High => 15,
                AsyncIssueSeverity::Medium => 10,
                AsyncIssueSeverity::Low => 5,
                AsyncIssueSeverity::Info => 1,
            })
            .sum::<i32>();

        let security_penalty = security_issues.iter()
            .map(|i| match i.severity {
                AsyncSecuritySeverity::Critical => 25,
                AsyncSecuritySeverity::High => 20,
                AsyncSecuritySeverity::Medium => 15,
                AsyncSecuritySeverity::Low => 10,
                AsyncSecuritySeverity::Info => 5,
            })
            .sum::<i32>();

        (base_score + async_bonus + concurrency_bonus - performance_penalty - security_penalty).clamp(0, 100)
    }

    fn get_async_recommendations(
        &self,
        async_functions: &[AsyncFunctionInfo],
        await_patterns: &[AwaitUsageInfo],
        concurrency_patterns: &[ConcurrencyPatternInfo],
        performance_issues: &[AsyncPerformanceIssue],
        security_issues: &[AsyncSecurityIssue],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if async_functions.is_empty() {
            recommendations.push("Consider using async/await for I/O bound operations".to_string());
        }

        if !performance_issues.is_empty() {
            recommendations.push("Address async performance issues for better efficiency".to_string());
        }

        if !security_issues.is_empty() {
            recommendations.push("Fix async security vulnerabilities".to_string());
        }

        let has_poor_concurrency = concurrency_patterns.iter()
            .any(|p| matches!(p.usage_quality, ConcurrencyUsageQuality::Poor | ConcurrencyUsageQuality::Dangerous));

        if has_poor_concurrency {
            recommendations.push("Improve concurrency pattern usage with proper error handling".to_string());
        }

        let has_invalid_await = await_patterns.iter().any(|p| !p.is_valid);
        if has_invalid_await {
            recommendations.push("Fix invalid await usage in sync contexts".to_string());
        }

        let missing_timeouts = async_functions.iter().any(|f| !f.has_timeout);
        if missing_timeouts {
            recommendations.push("Add timeout handling to prevent hanging operations".to_string());
        }

        recommendations.push("Use asyncio.gather() for concurrent independent operations".to_string());
        recommendations.push("Implement proper async context managers for resource cleanup".to_string());
        recommendations.push("Consider using Python 3.11+ TaskGroup for structured concurrency".to_string());

        recommendations
    }

    /// Helper methods for type hint analysis
    fn parse_type_hint_type(&self, hint_type: &str, captures: &regex::Captures) -> TypeHintType {
        match hint_type {
            "union" => {
                let types_str = captures.get(1).unwrap().as_str();
                let union_types = types_str.split(',').map(|s| s.trim().to_string()).collect();
                TypeHintType::UnionType(union_types)
            }
            "union_new" => {
                let type1 = captures.get(1).unwrap().as_str().to_string();
                let type2 = captures.get(2).unwrap().as_str().to_string();
                TypeHintType::UnionType(vec![type1, type2])
            }
            "optional" => {
                let inner_type = captures.get(1).unwrap().as_str().to_string();
                TypeHintType::OptionalType(inner_type)
            }
            "generic_list" => {
                let element_type = captures.get(1).unwrap().as_str().to_string();
                TypeHintType::GenericType(GenericTypeInfo {
                    base_type: "List".to_string(),
                    type_parameters: vec![element_type],
                    is_covariant: true,
                    is_contravariant: false,
                })
            }
            "generic_dict" => {
                let key_type = captures.get(1).unwrap().as_str().to_string();
                let value_type = captures.get(2).unwrap().as_str().to_string();
                TypeHintType::GenericType(GenericTypeInfo {
                    base_type: "Dict".to_string(),
                    type_parameters: vec![key_type, value_type],
                    is_covariant: false,
                    is_contravariant: false,
                })
            }
            "callable" => {
                let params_str = captures.get(1).unwrap().as_str();
                let return_type = captures.get(2).unwrap().as_str().to_string();
                let parameter_types = if params_str.is_empty() {
                    Vec::new()
                } else {
                    params_str.split(',').map(|s| s.trim().to_string()).collect()
                };
                TypeHintType::CallableType(CallableTypeInfo {
                    parameter_types,
                    return_type,
                    is_async: false,
                })
            }
            "typevar" => {
                let var_name = captures.get(1).unwrap().as_str().to_string();
                TypeHintType::TypeVarType(TypeVarInfo {
                    name: var_name,
                    bounds: Vec::new(),
                    constraints: Vec::new(),
                    covariant: false,
                    contravariant: false,
                })
            }
            "protocol" => {
                TypeHintType::ProtocolType("Protocol".to_string())
            }
            "literal" => {
                let values_str = captures.get(1).unwrap().as_str();
                let literal_values = values_str.split(',').map(|s| s.trim().to_string()).collect();
                TypeHintType::LiteralType(literal_values)
            }
            "final" => {
                let final_type = captures.get(1).unwrap().as_str().to_string();
                TypeHintType::FinalType(final_type)
            }
            "typeddict" => {
                let class_name = captures.get(1).unwrap().as_str().to_string();
                TypeHintType::TypedDictType(TypedDictInfo {
                    name: class_name,
                    fields: Vec::new(), // Would need more parsing for actual fields
                    total: true,
                })
            }
            "generic_alias" => {
                let base_type = captures.get(1).unwrap().as_str().to_string();
                let element_type = captures.get(2).unwrap().as_str().to_string();
                TypeHintType::GenericType(GenericTypeInfo {
                    base_type,
                    type_parameters: vec![element_type],
                    is_covariant: true,
                    is_contravariant: false,
                })
            }
            _ => TypeHintType::SimpleType("Unknown".to_string()),
        }
    }

    fn is_generic_type(&self, hint_type: &str) -> bool {
        matches!(hint_type, "generic_list" | "generic_dict" | "generic_alias")
    }

    fn has_type_constraints(&self, hint_type: &str) -> bool {
        matches!(hint_type, "typevar" | "protocol" | "literal")
    }

    fn get_modern_feature_type(&self, hint_type: &str) -> Option<ModernTypeFeatureType> {
        match hint_type {
            "union_new" => Some(ModernTypeFeatureType::UnionSyntaxPy310),
            "typeddict" => Some(ModernTypeFeatureType::TypedDict),
            "final" => Some(ModernTypeFeatureType::FinalType),
            "literal" => Some(ModernTypeFeatureType::LiteralType),
            "protocol" => Some(ModernTypeFeatureType::ProtocolType),
            "generic_alias" => Some(ModernTypeFeatureType::GenericAlias),
            _ => None,
        }
    }

    fn detect_type_safety_issues(&self, content: &str, issues: &mut Vec<TypeSafetyIssue>) {
        // Detect Any type overuse
        let any_count = content.matches("Any").count();
        if any_count > 5 {
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::AnyTypeOveruse,
                severity: TypeSafetySeverity::Warning,
                location: "Multiple locations".to_string(),
                description: format!("Found {} uses of Any type", any_count),
                recommendation: "Consider using more specific type hints".to_string(),
            });
        }

        // Detect missing type hints
        let func_pattern = Regex::new(r"def\s+\w+\s*\([^)]*\)\s*:").unwrap();
        let typed_func_pattern = Regex::new(r"def\s+\w+\s*\([^)]*\)\s*->\s*\w+:").unwrap();
        
        let total_functions = func_pattern.find_iter(content).count();
        let typed_functions = typed_func_pattern.find_iter(content).count();
        
        if total_functions > typed_functions && total_functions > 0 {
            let missing_hints = total_functions - typed_functions;
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::MissingTypeHints,
                severity: TypeSafetySeverity::Warning,
                location: "Function definitions".to_string(),
                description: format!("{} functions missing return type hints", missing_hints),
                recommendation: "Add return type annotations to functions".to_string(),
            });
        }

        // Detect type: ignore overuse
        let ignore_count = content.matches("# type: ignore").count();
        if ignore_count > 3 {
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::TypeIgnoreOveruse,
                severity: TypeSafetySeverity::Info,
                location: "Multiple locations".to_string(),
                description: format!("Found {} type: ignore comments", ignore_count),
                recommendation: "Review and fix type issues instead of ignoring them".to_string(),
            });
        }

        // Detect deprecated typing syntax
        if content.contains("typing.List") || content.contains("typing.Dict") {
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::DeprecatedTypingSyntax,
                severity: TypeSafetySeverity::Info,
                location: "Import statements".to_string(),
                description: "Using deprecated typing imports".to_string(),
                recommendation: "Use built-in generics (list, dict) for Python 3.9+".to_string(),
            });
        }
    }

    fn calculate_type_coverage(&self, content: &str, _type_hints: &[TypeHintInfo]) -> f32 {
        let func_pattern = Regex::new(r"def\s+\w+").unwrap();
        let total_functions = func_pattern.find_iter(content).count();
        
        if total_functions == 0 {
            return 0.0;
        }

        // Count functions with type annotations (parameter or return type hints)
        let typed_func_pattern = Regex::new(r"def\s+\w+\s*\([^)]*:\s*\w+|def\s+\w+\s*\([^)]*\)\s*->\s*\w+").unwrap();
        let typed_functions = typed_func_pattern.find_iter(content).count();
        
        (typed_functions as f32 / total_functions as f32) * 100.0
    }

    fn get_coverage_score(&self, coverage: f32) -> TypeCoverageScore {
        match coverage {
            score if score >= 90.0 => TypeCoverageScore::Excellent,
            score if score >= 70.0 => TypeCoverageScore::Good,
            score if score >= 50.0 => TypeCoverageScore::Fair,
            score if score >= 30.0 => TypeCoverageScore::Poor,
            _ => TypeCoverageScore::Minimal,
        }
    }

    fn get_type_hint_recommendations(
        &self,
        type_hints: &[TypeHintInfo],
        issues: &[TypeSafetyIssue],
        coverage: f32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if coverage < 70.0 {
            recommendations.push("Increase type hint coverage for better type safety".to_string());
        }

        if !issues.is_empty() {
            recommendations.push("Address type safety issues identified in the code".to_string());
        }

        let has_modern_features = type_hints.iter().any(|h| 
            h.python_version_required.starts_with("3.8") || 
            h.python_version_required.starts_with("3.9") ||
            h.python_version_required.starts_with("3.10")
        );

        if !has_modern_features {
            recommendations.push("Consider using modern Python type features (3.8+)".to_string());
        }

        let has_complex_types = type_hints.iter().any(|h| 
            matches!(h.complexity, TypeComplexity::Complex | TypeComplexity::Advanced)
        );

        if has_complex_types {
            recommendations.push("Document complex type relationships for maintainability".to_string());
        }

        if type_hints.iter().any(|h| h.is_generic) {
            recommendations.push("Ensure generic type constraints are properly defined".to_string());
        }

        recommendations.push("Use type checkers like mypy for static type validation".to_string());
        recommendations.push("Consider Protocol types for structural typing".to_string());

        recommendations
    }

    /// Helper methods for security analysis
    fn get_security_recommendation(&self, vulnerability_type: &VulnerabilityType) -> String {
        match vulnerability_type {
            VulnerabilityType::SqlInjection => {
                "Use parameterized queries or ORM methods".to_string()
            }
            VulnerabilityType::CommandInjection => {
                "Sanitize user input and avoid shell execution".to_string()
            }
            VulnerabilityType::DeserializationAttack => {
                "Use safe deserialization methods like json.loads".to_string()
            }
            VulnerabilityType::HardcodedSecrets => {
                "Use environment variables or secret management".to_string()
            }
            _ => "Review security implementation".to_string(),
        }
    }

    fn determine_security_level(
        &self,
        vulnerabilities: &[SecurityVulnerability],
        security_features: &[SecurityFeature],
    ) -> SecurityLevel {
        let critical_vulns = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, VulnerabilitySeverity::Critical))
            .count();
        let high_vulns = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, VulnerabilitySeverity::High))
            .count();

        if critical_vulns > 0 {
            SecurityLevel::Vulnerable
        } else if high_vulns > 2 {
            SecurityLevel::Low
        } else if security_features.len() > 2 {
            SecurityLevel::High
        } else {
            SecurityLevel::Medium
        }
    }

    fn get_security_recommendations(
        &self,
        vulnerabilities: &[SecurityVulnerability],
        _security_features: &[SecurityFeature],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !vulnerabilities.is_empty() {
            recommendations.push("Address security vulnerabilities identified in code".to_string());
        }

        recommendations.push("Implement comprehensive input validation".to_string());
        recommendations.push("Use secure authentication and authorization".to_string());
        recommendations.push("Enable security headers and CSRF protection".to_string());

        recommendations
    }

    /// Helper methods for performance analysis
    fn calculate_performance_score(
        &self,
        optimizations: &[PerformanceOptimization],
        issues: &[PerformanceIssue],
    ) -> i32 {
        let base_score = 50;
        let optimization_bonus = optimizations.len() as i32 * 10;
        let issue_penalty = issues
            .iter()
            .map(|i| match i.severity {
                IssueSeverity::Critical => 20,
                IssueSeverity::High => 15,
                IssueSeverity::Medium => 10,
                IssueSeverity::Low => 5,
            })
            .sum::<i32>();

        (base_score + optimization_bonus - issue_penalty).clamp(0, 100)
    }

    fn get_performance_recommendations(
        &self,
        _optimizations: &[PerformanceOptimization],
        issues: &[PerformanceIssue],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !issues.is_empty() {
            recommendations.push("Address performance issues identified in code".to_string());
        }

        recommendations.push("Use list comprehensions and generator expressions".to_string());
        recommendations.push("Implement caching for expensive operations".to_string());
        recommendations.push("Consider async/await for I/O operations".to_string());

        recommendations
    }

    /// Framework-specific analysis methods
    fn analyze_django_specifics(&self, content: &str) -> DjangoAnalysis {
        DjangoAnalysis {
            models_analysis: self.extract_django_models(content),
            views_analysis: self.extract_django_views(content),
            middleware_usage: self.extract_django_middleware(content),
            security_middleware: self.extract_django_security_middleware(content),
            signals_usage: self.extract_django_signals(content),
            admin_customization: content.contains("admin.site.register")
                || content.contains("ModelAdmin"),
        }
    }

    fn analyze_flask_specifics(&self, content: &str) -> FlaskAnalysis {
        FlaskAnalysis {
            blueprints: self.extract_flask_blueprints(content),
            extensions: self.extract_flask_extensions(content),
            error_handlers: self.extract_flask_error_handlers(content),
            template_usage: content.contains("render_template"),
            session_management: content.contains("session["),
        }
    }

    fn analyze_fastapi_specifics(&self, content: &str) -> FastAPIAnalysis {
        FastAPIAnalysis {
            router_usage: self.extract_fastapi_routers(content),
            dependency_injection: self.extract_fastapi_dependencies(content),
            background_tasks: content.contains("BackgroundTasks"),
            websocket_endpoints: self.extract_fastapi_websockets(content),
            middleware: self.extract_fastapi_middleware(content),
            response_models: self.extract_fastapi_response_models(content),
        }
    }

    fn analyze_pytest_specifics(&self, content: &str) -> PytestAnalysis {
        PytestAnalysis {
            fixtures: self.extract_pytest_fixtures(content),
            parametrized_tests: self.extract_pytest_parametrized(content),
            markers: self.extract_pytest_markers(content),
            plugins: self.extract_pytest_plugins(content),
            coverage_setup: content.contains("pytest-cov") || content.contains("coverage"),
        }
    }

    // Simplified extraction methods (can be enhanced with more complex parsing)
    fn extract_django_models(&self, content: &str) -> Vec<DjangoModelInfo> {
        let model_pattern = Regex::new(r"class\s+(\w+)\s*\([^)]*Model[^)]*\)").unwrap();
        model_pattern
            .captures_iter(content)
            .map(|captures| {
                let model_name = captures.get(1).unwrap().as_str().to_string();
                DjangoModelInfo {
                    name: model_name,
                    fields: Vec::new(), // Simplified
                    relationships: Vec::new(),
                    custom_managers: false,
                    meta_options: Vec::new(),
                }
            })
            .collect()
    }

    fn extract_django_views(&self, _content: &str) -> Vec<DjangoViewInfo> {
        Vec::new() // Simplified implementation
    }

    fn extract_django_middleware(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_django_security_middleware(&self, content: &str) -> Vec<String> {
        let mut middleware = Vec::new();
        if content.contains("SecurityMiddleware") {
            middleware.push("SecurityMiddleware".to_string());
        }
        if content.contains("CsrfViewMiddleware") {
            middleware.push("CsrfViewMiddleware".to_string());
        }
        middleware
    }

    fn extract_django_signals(&self, content: &str) -> Vec<String> {
        let mut signals = Vec::new();
        if content.contains("post_save") {
            signals.push("post_save".to_string());
        }
        if content.contains("pre_save") {
            signals.push("pre_save".to_string());
        }
        signals
    }

    fn extract_flask_blueprints(&self, _content: &str) -> Vec<FlaskBlueprintInfo> {
        Vec::new() // Simplified implementation
    }

    fn extract_flask_extensions(&self, content: &str) -> Vec<String> {
        let mut extensions = Vec::new();
        if content.contains("Flask-Login") {
            extensions.push("Flask-Login".to_string());
        }
        if content.contains("Flask-SQLAlchemy") {
            extensions.push("Flask-SQLAlchemy".to_string());
        }
        extensions
    }

    fn extract_flask_error_handlers(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_fastapi_routers(&self, _content: &str) -> Vec<FastAPIRouterInfo> {
        Vec::new() // Simplified implementation
    }

    fn extract_fastapi_dependencies(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_fastapi_websockets(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_fastapi_middleware(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_fastapi_response_models(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_pytest_fixtures(&self, _content: &str) -> Vec<PytestFixtureInfo> {
        Vec::new() // Simplified implementation
    }

    fn extract_pytest_parametrized(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_pytest_markers(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn extract_pytest_plugins(&self, _content: &str) -> Vec<String> {
        Vec::new() // Simplified implementation
    }

    fn get_framework_best_practices(&self, framework: &str) -> Vec<String> {
        match framework {
            "Django" => vec![
                "Use Django ORM instead of raw SQL".to_string(),
                "Implement proper authentication and authorization".to_string(),
                "Use Django forms for input validation".to_string(),
            ],
            "Flask" => vec![
                "Use blueprints for application modularity".to_string(),
                "Implement proper error handling".to_string(),
                "Use Flask-WTF for form handling".to_string(),
            ],
            "FastAPI" => vec![
                "Use Pydantic models for request/response validation".to_string(),
                "Implement proper dependency injection".to_string(),
                "Use async/await for I/O operations".to_string(),
            ],
            _ => Vec::new(),
        }
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

    #[test]
    fn test_type_hint_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = r#"
from typing import List, Dict, Union, Optional, Literal, Final
from typing_extensions import Protocol

def process_data(items: List[str], mapping: Dict[str, int]) -> Optional[str]:
    return None

def handle_union(value: Union[str, int]) -> str:
    return str(value)

class MyProtocol(Protocol):
    def method(self) -> None: ...

CONSTANT: Final[str] = "value"
MODE: Literal["read", "write"] = "read"
        "#;

        let result = analyzer.analyze_type_hints(code).unwrap();

        assert!(!result.type_hints_detected.is_empty());
        assert!(result.type_hints_detected.iter().any(|h| 
            matches!(h.hint_type, TypeHintType::GenericType(_))
        ));
        assert!(result.type_hints_detected.iter().any(|h| 
            matches!(h.hint_type, TypeHintType::UnionType(_))
        ));
        assert!(result.type_hints_detected.iter().any(|h| 
            matches!(h.hint_type, TypeHintType::OptionalType(_))
        ));
        assert!(result.overall_coverage > 0.0);
    }

    #[test]
    fn test_modern_type_features() {
        let analyzer = PythonAnalyzer::new();

        let code = r#"
from typing import Final, Literal
from typing_extensions import TypedDict

class UserDict(TypedDict):
    name: str
    age: int

CONSTANT: Final[int] = 42
STATUS: Literal["active", "inactive"] = "active"

# Python 3.10+ union syntax
def process(value: str | int) -> str | None:
    return None
        "#;

        let result = analyzer.analyze_type_hints(code).unwrap();

        assert!(!result.modern_type_features.is_empty());
        assert!(result.modern_type_features.iter().any(|f| 
            matches!(f.feature_type, ModernTypeFeatureType::TypedDict)
        ));
        assert!(result.modern_type_features.iter().any(|f| 
            matches!(f.feature_type, ModernTypeFeatureType::FinalType)
        ));
        assert!(result.modern_type_features.iter().any(|f| 
            matches!(f.feature_type, ModernTypeFeatureType::LiteralType)
        ));
    }

    #[test]
    fn test_type_safety_issues() {
        let analyzer = PythonAnalyzer::new();

        let code = r#"
from typing import Any

def untyped_function():
    return "hello"

def another_untyped():
    pass

def bad_any_usage(x: Any, y: Any, z: Any, a: Any, b: Any, c: Any) -> Any:
    return x

# type: ignore
# type: ignore  
# type: ignore
# type: ignore
        "#;

        let result = analyzer.analyze_type_hints(code).unwrap();

        assert!(!result.type_safety_issues.is_empty());
        assert!(result.type_safety_issues.iter().any(|issue| 
            matches!(issue.issue_type, TypeSafetyIssueType::AnyTypeOveruse)
        ));
        assert!(result.type_safety_issues.iter().any(|issue| 
            matches!(issue.issue_type, TypeSafetyIssueType::MissingTypeHints)
        ));
        assert!(result.type_safety_issues.iter().any(|issue| 
            matches!(issue.issue_type, TypeSafetyIssueType::TypeIgnoreOveruse)
        ));
    }

    #[test] 
    fn test_type_coverage_calculation() {
        let analyzer = PythonAnalyzer::new();

        // High coverage code
        let high_coverage_code = r#"
def typed_func1(x: int) -> str:
    return str(x)

def typed_func2(y: str) -> int:
    return len(y)
        "#;

        let result = analyzer.analyze_type_hints(high_coverage_code).unwrap();
        assert!(result.overall_coverage > 50.0);
        assert!(matches!(result.type_coverage_score, 
            TypeCoverageScore::Good | TypeCoverageScore::Excellent | TypeCoverageScore::Fair
        ));

        // Low coverage code
        let low_coverage_code = r#"
def untyped_func1():
    return "hello"

def untyped_func2():
    return 42

def typed_func(x: int) -> str:
    return str(x)
        "#;

        let result = analyzer.analyze_type_hints(low_coverage_code).unwrap();
        assert!(result.overall_coverage < 100.0);
    }

    #[test]
    fn test_async_await_analysis() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import asyncio

async def fetch_data():
    await asyncio.sleep(1)
    return "data"

async def process_items():
    results = await asyncio.gather(
        fetch_data(),
        fetch_data(),
        fetch_data()
    )
    return results

async def with_context():
    async with asyncio.timeout(5):
        return await fetch_data()
"#;

        let result = analyzer.analyze_async_await(content).unwrap();
        
        // Should detect async functions
        assert!(!result.async_functions_detected.is_empty());
        assert!(result.async_functions_detected.len() >= 3);
        
        // Should detect concurrency patterns
        assert!(!result.concurrency_patterns.is_empty());
        
        // Should detect modern async features
        assert!(!result.modern_async_features.is_empty());
        
        // Should have a reasonable async score
        assert!(result.overall_async_score > 50);
    }

    #[test]
    fn test_async_performance_issues() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import asyncio
import time

async def bad_function():
    # Blocking operations in async function
    time.sleep(1)
    with open("file.txt") as f:
        data = f.read()
    
    # Sequential awaits that could be concurrent
    result1 = await fetch_data()
    result2 = await fetch_data()
    
    return result1 + result2

async def fetch_data():
    await asyncio.sleep(0.1)
    return "data"
"#;

        let result = analyzer.analyze_async_await(content).unwrap();
        
        // Should detect performance issues
        assert!(!result.async_performance_issues.is_empty());
        
        // Should detect blocking operations
        let blocking_issues: Vec<_> = result.async_performance_issues.iter()
            .filter(|issue| matches!(issue.issue_type, AsyncPerformanceIssueType::BlockingIOInAsync))
            .collect();
        assert!(!blocking_issues.is_empty());
        
        // Should detect missing concurrency
        let concurrency_issues: Vec<_> = result.async_performance_issues.iter()
            .filter(|issue| matches!(issue.issue_type, AsyncPerformanceIssueType::MissingConcurrency))
            .collect();
        assert!(!concurrency_issues.is_empty());
    }

    #[test]
    fn test_async_security_issues() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import asyncio

shared_data = {}

async def unsafe_function():
    # No timeout handling
    await some_external_service()
    
    # Shared state modification without locking
    shared_data["key"] = "value"
    
    # Multiple concurrent operations without proper synchronization
    await asyncio.gather(
        modify_shared_data(),
        modify_shared_data(),
        modify_shared_data()
    )

async def some_external_service():
    await asyncio.sleep(1)

async def modify_shared_data():
    shared_data["counter"] = shared_data.get("counter", 0) + 1
"#;

        let result = analyzer.analyze_async_await(content).unwrap();
        
        // Should detect security issues
        assert!(!result.async_security_issues.is_empty());
        
        // Should detect timeout vulnerability
        let timeout_issues: Vec<_> = result.async_security_issues.iter()
            .filter(|issue| matches!(issue.issue_type, AsyncSecurityIssueType::AsyncTimeoutVuln))
            .collect();
        assert!(!timeout_issues.is_empty());
    }

    #[test]
    fn test_modern_async_features() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import asyncio
import contextvars

async def modern_async():
    # Modern async features
    async with asyncio.timeout(5):
        data = await fetch_data()
    
    # Async comprehension
    results = [await process(item) async for item in async_generator()]
    
    # Context variables
    context_var = contextvars.ContextVar('user_id')
    
    # TaskGroup (Python 3.11+)
    async with asyncio.TaskGroup() as tg:
        task1 = tg.create_task(fetch_data())
        task2 = tg.create_task(fetch_data())
    
    return results

async def async_generator():
    for i in range(3):
        yield f"item_{i}"

async def fetch_data():
    return "data"

async def process(item):
    return f"processed_{item}"

if __name__ == "__main__":
    asyncio.run(modern_async())
"#;

        let result = analyzer.analyze_async_await(content).unwrap();
        
        // Should detect modern async features
        assert!(!result.modern_async_features.is_empty());
        
        // Should detect async context managers
        let context_manager_features: Vec<_> = result.modern_async_features.iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::AsyncContextManager))
            .collect();
        assert!(!context_manager_features.is_empty());
        
        // Should detect TaskGroups
        let task_group_features: Vec<_> = result.modern_async_features.iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::TaskGroups))
            .collect();
        assert!(!task_group_features.is_empty());
        
        // Should detect asyncio.run
        let asyncio_run_features: Vec<_> = result.modern_async_features.iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::AsyncioRun))
            .collect();
        assert!(!asyncio_run_features.is_empty());
        
        // Should have recommendations
        assert!(!result.recommendations.is_empty());
    }
}
