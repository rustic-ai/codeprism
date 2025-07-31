//! Python-specific code analysis module

// Temporarily allow clippy warnings for Issue #77 - will be cleaned up in future issues
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::cmp_owned)]

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
pub enum TypeHintType {
    SimpleType(String),             // int, str, bool
    UnionType(Vec<String>),         // Union[str, int] or str | int
    GenericType(GenericTypeInfo),   // List[T], Dict[K, V]
    ProtocolType(String),           // Protocol for structural typing
    LiteralType(Vec<String>),       // Literal['value1', 'value2']
    CallableType(CallableTypeInfo), // Callable[[int, str], bool]
    TypeVarType(TypeVarInfo),       // TypeVar constraints and bounds
    OptionalType(String),           // Optional[str] or str | None
    FinalType(String),              // Final[int]
    TypedDictType(TypedDictInfo),   // TypedDict for structured dicts
}

/// Generic type information
#[derive(Debug, Clone)]
pub struct GenericTypeInfo {
    pub base_type: String,            // List, Dict, Set, etc.
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
    Simple,   // Basic types like int, str
    Moderate, // Union types, Optional
    Complex,  // Generic types with multiple parameters
    Advanced, // Complex nested generics, Protocols
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
    AnyTypeOveruse,         // Too many Any types
    MissingTypeHints,       // Functions without type hints
    InconsistentTypes,      // Type inconsistencies
    TypeIgnoreOveruse,      // Too many # type: ignore comments
    WrongTypeHintSyntax,    // Incorrect type hint syntax
    DeprecatedTypingSyntax, // Using old typing syntax
    UnreachableCode,        // Dead code due to type narrowing
    TypeVarianceIssue,      // Covariance/contravariance problems
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
    PositionalOnlyParams, // def func(arg, /) -> str:
    UnionSyntaxPy310,     // str | int instead of Union[str, int]
    TypedDict,            // TypedDict for structured dictionaries
    FinalType,            // Final[int] = 42
    LiteralType,          // Literal['red', 'green', 'blue']
    ProtocolType,         // Protocol for structural typing
    TypeGuard,            // TypeGuard for type narrowing
    OverloadDecorator,    // @overload for function overloading
    GenericAlias,         // list[int] instead of List[int] (Python 3.9+)
    ParamSpec,            // ParamSpec for callable signatures (Python 3.10+)
    TypeVarTuple,         // TypeVarTuple for variadic generics (Python 3.11+)
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
    RegularAsync,        // async def function()
    AsyncGenerator,      // async def with yield
    AsyncContextManager, // __aenter__, __aexit__
    AsyncIterator,       // __aiter__, __anext__
    AsyncProperty,       // @async_property
    AsyncDecorator,      // Decorates with async functionality
}

/// Async function complexity
#[derive(Debug, Clone)]
pub enum AsyncComplexity {
    Simple,   // Single await or simple operations
    Moderate, // Multiple awaits, basic control flow
    Complex,  // Nested awaits, exception handling
    Advanced, // Complex concurrency patterns, resource management
}

/// Coroutine type classification
#[derive(Debug, Clone)]
pub enum CoroutineType {
    Native,            // Native Python coroutines
    Framework(String), // Framework-specific (asyncio, trio, curio)
    Generator,         // Generator-based coroutines (deprecated)
    Hybrid,            // Mixed native and framework
}

/// Async error handling assessment
#[derive(Debug, Clone)]
pub enum AsyncErrorHandling {
    None,    // No error handling
    Basic,   // Simple try/catch
    Timeout, // Includes timeout handling
    Robust,  // Comprehensive error handling with retries
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
    AsyncFunction,       // Inside async def
    AsyncGenerator,      // Inside async generator
    AsyncContextManager, // Inside async context manager
    AsyncIterator,       // Inside async iterator
    SyncContext,         // Invalid: inside sync function
    Comprehension,       // Invalid: inside comprehension
    Lambda,              // Invalid: inside lambda
}

/// Await usage patterns
#[derive(Debug, Clone)]
pub enum AwaitUsagePattern {
    SingleAwait,      // Single await expression
    SequentialAwaits, // Multiple sequential awaits
    ConditionalAwait, // Await in conditional
    NestedAwait,      // Await inside await
    GatheredAwait,    // Part of asyncio.gather()
    ConcurrentAwait,  // Concurrent execution pattern
}

/// Await usage issues
#[derive(Debug, Clone)]
pub enum AwaitIssue {
    IllegalContext, // await in illegal context
    MissingAwait,   // Missing await on coroutine
    BlockingCall,   // Blocking call in async context
    SyncInAsync,    // Sync operation in async function
    ResourceLeak,   // Potential resource leak
    TimeoutMissing, // Missing timeout handling
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
    Excellent, // Optimal usage with best practices
    Good,      // Correct usage with minor optimizations possible
    Adequate,  // Functional but suboptimal
    Poor,      // Problematic usage that should be improved
    Dangerous, // Usage that can cause deadlocks or race conditions
}

/// Performance impact of async patterns
#[derive(Debug, Clone)]
pub enum AsyncPerformanceImpact {
    Positive, // Improves performance
    Neutral,  // No significant impact
    Negative, // Reduces performance
    Critical, // Severely impacts performance
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
    BlockingIOInAsync,     // Sync I/O operations in async functions
    EventLoopBlocking,     // Operations that block the event loop
    GILContentionAsync,    // GIL contention in async code
    AwaitInLoop,           // Inefficient await in loop
    MissingConcurrency,    // Sequential execution where concurrent is possible
    ResourceLeakAsync,     // Async resource leaks
    SyncWrapperOveruse,    // Overuse of sync-to-async wrappers
    AsyncioSubprocessSync, // Sync subprocess calls in async context
    DatabaseBlockingAsync, // Blocking database calls in async functions
    SlowAsyncGenerator,    // Inefficient async generators
}

/// Async issue severity levels
#[derive(Debug, Clone)]
pub enum AsyncIssueSeverity {
    Critical, // Breaks async functionality
    High,     // Significant performance impact
    Medium,   // Moderate impact on performance
    Low,      // Minor optimization opportunity
    Info,     // Best practice suggestion
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
    AsyncRaceCondition,     // Race conditions in async code
    SharedStateNoLock,      // Shared mutable state without locking
    AsyncTimeoutVuln,       // Missing timeouts enabling DoS
    TaskCancellationLeak,   // Improper task cancellation
    AsyncResourceExposure,  // Resource exposure through async operations
    EventLoopPoisoning,     // Event loop manipulation attacks
    AsyncPickleVuln,        // Pickle vulnerabilities in async context
    ConcurrentModification, // Concurrent modification without protection
}

/// Async security severity levels
#[derive(Debug, Clone)]
pub enum AsyncSecuritySeverity {
    Critical, // Exploitable security vulnerability
    High,     // Significant security risk
    Medium,   // Moderate security concern
    Low,      // Minor security consideration
    Info,     // Security best practice
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
    AsyncContextManager, // async with statements
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

/// Python package dependency analysis result
#[derive(Debug, Clone)]
pub struct PythonPackageDependencyAnalysis {
    pub overall_health_score: i32,
    pub requirements_files: Vec<RequirementsFileInfo>,
    pub dependencies: Vec<RequirementInfo>,
    pub dependency_issues: Vec<DependencyIssue>,
    pub virtual_environments: Vec<VirtualEnvironmentInfo>,
    pub import_analysis: Vec<ImportAnalysisInfo>,
    pub security_vulnerabilities: Vec<SecurityVulnerabilityInfo>,
    pub license_analysis: Vec<LicenseInfo>,
    pub recommendations: Vec<String>,
}

/// Requirements file information
#[derive(Debug, Clone)]
pub struct RequirementsFileInfo {
    pub file_path: String,
    pub file_type: RequirementsFileType,
    pub dependencies_count: usize,
    pub has_version_pins: bool,
    pub has_hashes: bool,
    pub uses_constraints: bool,
    pub quality_score: DependencyQualityScore,
}

/// Requirements file types
#[derive(Debug, Clone)]
pub enum RequirementsFileType {
    RequirementsTxt, // requirements.txt
    PyprojectToml,   // pyproject.toml with dependencies
    SetupPy,         // setup.py with install_requires
    Pipfile,         // Pipfile for pipenv
    PipfileLock,     // Pipfile.lock with locked versions
    PoetryLock,      // poetry.lock for poetry
    CondaYml,        // environment.yml for conda
    SetupCfg,        // setup.cfg with install_requires
}

/// Individual requirement information
#[derive(Debug, Clone)]
pub struct RequirementInfo {
    pub name: String,
    pub version_spec: String,
    pub source: RequirementSource,
    pub is_dev_dependency: bool,
    pub is_optional: bool,
    pub extras: Vec<String>,
    pub markers: Vec<String>,
    pub metadata: PackageMetadata,
}

/// Requirement source types
#[derive(Debug, Clone)]
pub enum RequirementSource {
    PyPI,                // Standard PyPI package
    Git(String),         // Git repository URL
    Local(String),       // Local file path
    URL(String),         // Direct URL
    VCS(String, String), // Version control (type, url)
}

/// Package metadata information
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub last_updated: Option<String>,
    pub download_count: Option<u64>,
    pub maintenance_status: MaintenanceStatus,
}

/// Package maintenance status
#[derive(Debug, Clone)]
pub enum MaintenanceStatus {
    Active,     // Recently updated, active development
    Maintained, // Occasional updates, bug fixes
    Stable,     // Mature, infrequent updates
    Deprecated, // No longer maintained
    Abandoned,  // No updates for extended period
    Unknown,    // Unable to determine status
}

/// Dependency quality scoring
#[derive(Debug, Clone)]
pub enum DependencyQualityScore {
    Excellent, // 90-100: Well-managed, secure, up-to-date
    Good,      // 70-89: Good practices, minor issues
    Fair,      // 50-69: Adequate, some improvements needed
    Poor,      // 30-49: Multiple issues, needs attention
    Critical,  // 0-29: Major problems, immediate action required
}

/// Dependency issues
#[derive(Debug, Clone)]
pub struct DependencyIssue {
    pub issue_type: DependencyIssueType,
    pub severity: DependencyIssueSeverity,
    pub affected_packages: Vec<String>,
    pub description: String,
    pub recommendation: String,
    pub auto_fixable: bool,
}

/// Dependency issue types
#[derive(Debug, Clone)]
pub enum DependencyIssueType {
    VersionConflict,        // Conflicting version requirements
    CircularDependency,     // Circular dependency detected
    UnusedDependency,       // Declared but not imported
    MissingDependency,      // Imported but not declared
    SecurityVulnerability,  // Known security issue
    LicenseIncompatibility, // Incompatible licenses
    DeprecatedPackage,      // Package is deprecated
    UnpinnedVersion,        // Version not pinned for production
    OutdatedVersion,        // Newer version available
    DevDependencyInProd,    // Dev dependency in production requirements
    DuplicateDependency,    // Same package in multiple files
}

/// Dependency issue severity
#[derive(Debug, Clone)]
pub enum DependencyIssueSeverity {
    Critical, // Security vulnerabilities, license violations
    High,     // Version conflicts, missing dependencies
    Medium,   // Deprecated packages, outdated versions
    Low,      // Unused dependencies, minor optimizations
    Info,     // Best practice suggestions
}

/// Virtual environment information
#[derive(Debug, Clone)]
pub struct VirtualEnvironmentInfo {
    pub env_type: VirtualEnvironmentType,
    pub location: String,
    pub python_version: String,
    pub is_active: bool,
    pub packages_count: usize,
    pub env_variables: Vec<EnvironmentVariable>,
    pub configuration: VirtualEnvironmentConfig,
}

/// Virtual environment types
#[derive(Debug, Clone)]
pub enum VirtualEnvironmentType {
    Venv,           // Python venv
    Virtualenv,     // virtualenv package
    Conda,          // Anaconda/Miniconda
    Pipenv,         // Pipenv virtual environment
    Poetry,         // Poetry virtual environment
    Docker,         // Docker container environment
    Pyenv,          // pyenv version management
    Custom(String), // Custom environment type
}

/// Environment variable information
#[derive(Debug, Clone)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub is_sensitive: bool,
    pub purpose: EnvironmentVariablePurpose,
}

/// Environment variable purposes
#[derive(Debug, Clone)]
pub enum EnvironmentVariablePurpose {
    Configuration, // Application configuration
    Secret,        // API keys, passwords
    Path,          // PYTHONPATH, PATH modifications
    Development,   // Development-specific settings
    Production,    // Production environment settings
    Testing,       // Test configuration
    Unknown,       // Unable to categorize
}

/// Virtual environment configuration
#[derive(Debug, Clone)]
pub struct VirtualEnvironmentConfig {
    pub isolated: bool,
    pub system_site_packages: bool,
    pub pip_version: Option<String>,
    pub setuptools_version: Option<String>,
    pub custom_configurations: Vec<String>,
}

/// Import analysis information
#[derive(Debug, Clone)]
pub struct ImportAnalysisInfo {
    pub import_statement: String,
    pub import_type: ImportType,
    pub module_category: ModuleCategory,
    pub usage_count: usize,
    pub is_unused: bool,
    pub import_issues: Vec<ImportIssue>,
    pub optimization_suggestions: Vec<String>,
}

/// Import types
#[derive(Debug, Clone)]
pub enum ImportType {
    StandardImport,    // import module
    FromImport,        // from module import item
    StarImport,        // from module import *
    AliasImport,       // import module as alias
    RelativeImport,    // from .module import item
    ConditionalImport, // Import inside if/try block
    DynamicImport,     // importlib.import_module()
}

/// Module categories
#[derive(Debug, Clone)]
pub enum ModuleCategory {
    StandardLibrary, // Built-in Python modules
    ThirdParty,      // External packages from PyPI
    Local,           // Local project modules
    BuiltIn,         // Built-in functions and types
    Unknown,         // Unable to categorize
}

/// Import issues
#[derive(Debug, Clone, PartialEq)]
pub enum ImportIssue {
    CircularImport,      // Circular import detected
    StarImportDangerous, // from module import * is problematic
    UnusedImport,        // Import not used in code
    RedundantImport,     // Import duplicated
    WrongImportOrder,    // PEP 8 import order violation
    MissingImport,       // Required import not found
    SlowImport,          // Import is known to be slow
    DeprecatedImport,    // Importing deprecated module
}

/// Security vulnerability information
#[derive(Debug, Clone)]
pub struct SecurityVulnerabilityInfo {
    pub cve_id: Option<String>,
    pub advisory_id: Option<String>,
    pub package_name: String,
    pub affected_versions: Vec<String>,
    pub fixed_version: Option<String>,
    pub severity: SecurityVulnerabilitySeverity,
    pub vulnerability_type: VulnerabilityCategory,
    pub description: String,
    pub references: Vec<String>,
    pub published_date: Option<String>,
    pub last_modified: Option<String>,
}

/// Security vulnerability severity (CVSS-based)
#[derive(Debug, Clone)]
pub enum SecurityVulnerabilitySeverity {
    Critical, // CVSS 9.0-10.0
    High,     // CVSS 7.0-8.9
    Medium,   // CVSS 4.0-6.9
    Low,      // CVSS 0.1-3.9
    None,     // CVSS 0.0
    Unknown,  // Severity not available
}

/// Vulnerability categories
#[derive(Debug, Clone)]
pub enum VulnerabilityCategory {
    CodeExecution,         // Remote or arbitrary code execution
    SqlInjection,          // SQL injection vulnerabilities
    XSS,                   // Cross-site scripting
    CSRF,                  // Cross-site request forgery
    PathTraversal,         // Directory traversal attacks
    Deserialization,       // Unsafe deserialization
    Cryptographic,         // Cryptographic weaknesses
    DoS,                   // Denial of service
    PrivilegeEscalation,   // Privilege escalation
    InformationDisclosure, // Information disclosure
    InputValidation,       // Input validation issues
    AuthenticationBypass,  // Authentication bypass
    Other(String),         // Other vulnerability types
}

/// License information
#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub package_name: String,
    pub license_type: LicenseType,
    pub license_text: Option<String>,
    pub compatibility: LicenseCompatibility,
    pub commercial_use_allowed: bool,
    pub distribution_allowed: bool,
    pub modification_allowed: bool,
    pub patent_grant: bool,
    pub copyleft: bool,
}

/// License types
#[derive(Debug, Clone)]
pub enum LicenseType {
    MIT,
    Apache2,
    GPL2,
    GPL3,
    BSD2Clause,
    BSD3Clause,
    LGPL,
    Mozilla,
    Unlicense,
    Proprietary,
    Custom(String),
    Unknown,
}

/// License compatibility assessment
#[derive(Debug, Clone)]
pub enum LicenseCompatibility {
    Compatible,              // Fully compatible with project license
    ConditionallyCompatible, // Compatible under certain conditions
    Incompatible,            // License conflict detected
    RequiresReview,          // Manual review required
    Unknown,                 // Unable to determine compatibility
}

/// Pattern for dependency detection
#[derive(Debug, Clone)]
struct DependencyPattern {
    name: String,
    pattern: Regex,
    #[allow(dead_code)] // Will be used for dependency file type analysis
    file_type: RequirementsFileType,
    extraction_method: String,
}

/// Modern Python features analysis result
#[derive(Debug, Clone)]
pub struct ModernPythonFeatureAnalysis {
    pub overall_modernity_score: i32,
    pub python_version_detected: PythonVersionDetected,
    pub dataclass_features: Vec<DataclassInfo>,
    pub context_manager_features: Vec<ContextManagerInfo>,
    pub fstring_features: Vec<FStringInfo>,
    pub pattern_matching_features: Vec<PatternMatchingInfo>,
    pub generator_features: Vec<GeneratorInfo>,
    pub decorator_features: Vec<ModernDecoratorInfo>,
    pub modern_syntax_features: Vec<ModernSyntaxInfo>,
    pub recommendations: Vec<String>,
}

/// Python version detection
#[derive(Debug, Clone)]
pub struct PythonVersionDetected {
    pub minimum_version: String,
    pub features_by_version: Vec<VersionFeature>,
    pub compatibility_issues: Vec<CompatibilityIssue>,
}

/// Version-specific feature
#[derive(Debug, Clone)]
pub struct VersionFeature {
    pub feature_name: String,
    pub python_version: String,
    pub usage_count: usize,
    pub is_best_practice: bool,
}

/// Compatibility issue
#[derive(Debug, Clone)]
pub struct CompatibilityIssue {
    pub issue_type: CompatibilityIssueType,
    pub severity: CompatibilitySeverity,
    pub feature_name: String,
    pub required_version: String,
    pub description: String,
    pub recommendation: String,
}

/// Compatibility issue types
#[derive(Debug, Clone)]
pub enum CompatibilityIssueType {
    VersionMismatch,   // Feature requires newer Python version
    DeprecatedFeature, // Feature is deprecated
    SyntaxError,       // Syntax not supported in target version
    ImportError,       // Module not available in target version
    BehaviorChange,    // Feature behavior changed between versions
}

/// Compatibility severity
#[derive(Debug, Clone)]
pub enum CompatibilitySeverity {
    Critical, // Code will not run
    High,     // Major functionality affected
    Medium,   // Minor issues or warnings
    Low,      // Style or performance recommendations
    Info,     // Informational only
}

/// Dataclass analysis information
#[derive(Debug, Clone)]
pub struct DataclassInfo {
    pub class_name: String,
    pub dataclass_type: DataclassType,
    pub fields: Vec<DataclassField>,
    pub features_used: Vec<DataclassFeature>,
    pub complexity: FeatureComplexity,
    pub best_practices_score: i32,
    pub recommendations: Vec<String>,
}

/// Dataclass types
#[derive(Debug, Clone, PartialEq)]
pub enum DataclassType {
    StandardDataclass, // @dataclass
    PydanticModel,     // Pydantic BaseModel
    NamedTuple,        // typing.NamedTuple
    AttrsClass,        // attrs @attr.s
    SimpleNamespace,   // types.SimpleNamespace
}

/// Dataclass field information
#[derive(Debug, Clone)]
pub struct DataclassField {
    pub name: String,
    pub field_type: String,
    pub has_default: bool,
    pub is_optional: bool,
    pub validation_rules: Vec<String>,
    pub metadata: Vec<String>,
}

/// Dataclass features
#[derive(Debug, Clone, PartialEq)]
pub enum DataclassFeature {
    FrozenClass,        // frozen=True
    InitGeneration,     // init=True/False
    ReprGeneration,     // repr=True/False
    EqGeneration,       // eq=True/False
    OrderGeneration,    // order=True
    HashGeneration,     // unsafe_hash=True
    SlotsUsage,         // __slots__
    PostInitProcessing, // __post_init__
    FieldFactories,     // default_factory
    KwOnlyFields,       // kw_only fields
}

/// Context manager analysis information
#[derive(Debug, Clone)]
pub struct ContextManagerInfo {
    pub context_type: ContextManagerType,
    pub usage_pattern: ContextUsagePattern,
    pub resource_management: ResourceManagementQuality,
    pub error_handling: ContextErrorHandling,
    pub is_async: bool,
    pub nested_level: usize,
    pub best_practices_followed: bool,
}

/// Context manager types
#[derive(Debug, Clone)]
pub enum ContextManagerType {
    BuiltInFileManager,   // open() with statement
    CustomContextManager, // Custom __enter__/__exit__
    ContextlibManager,    // @contextmanager decorator
    AsyncContextManager,  // async __aenter__/__aexit__
    DatabaseConnection,   // DB connection managers
    LockManager,          // Threading/asyncio locks
    TemporaryResource,    // tempfile, temporary dirs
    ExceptionSuppression, // contextlib.suppress
}

/// Context usage patterns
#[derive(Debug, Clone)]
pub enum ContextUsagePattern {
    SingleContext,      // Single with statement
    MultipleContexts,   // Multiple with variables
    NestedContexts,     // Nested with statements
    AsyncContext,       // async with
    ConditionalContext, // with in conditional blocks
    LoopContext,        // with in loops
}

/// Resource management quality
#[derive(Debug, Clone)]
pub enum ResourceManagementQuality {
    Excellent, // Proper resource cleanup, error handling
    Good,      // Good resource management with minor issues
    Adequate,  // Basic resource management
    Poor,      // Resource leaks possible
    Dangerous, // Major resource management issues
}

/// Context error handling
#[derive(Debug, Clone)]
pub enum ContextErrorHandling {
    Comprehensive, // Full exception handling
    Basic,         // Basic error handling
    Minimal,       // Minimal error handling
    None,          // No error handling
}

/// F-string analysis information
#[derive(Debug, Clone)]
pub struct FStringInfo {
    pub expression: String,
    pub complexity: FStringComplexity,
    pub features_used: Vec<FStringFeature>,
    pub performance_impact: PerformanceImpact,
    pub formatting_quality: FormattingQuality,
    pub readability_score: i32,
}

/// F-string complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum FStringComplexity {
    Simple,   // Basic variable interpolation
    Moderate, // Simple expressions and formatting
    Complex,  // Complex expressions, nested calls
    Advanced, // Very complex expressions, performance concerns
}

/// F-string features
#[derive(Debug, Clone, PartialEq)]
pub enum FStringFeature {
    BasicInterpolation,   // f"{variable}"
    ExpressionEvaluation, // f"{expression()}"
    FormatSpecifiers,     // f"{value:.2f}"
    ConversionFlags,      // f"{value!r}"
    NestedFormatting,     // f"{f'{inner}'}"
    MultilineString,      // Multiline f-strings
    RawFString,           // rf"string"
    ComplexExpressions,   // f"{complex.expression}"
}

/// Performance impact assessment
#[derive(Debug, Clone)]
pub enum PerformanceImpact {
    Positive, // Better performance than alternatives
    Neutral,  // Similar performance
    Negative, // Worse performance
    Critical, // Significantly worse performance
}

/// Formatting quality assessment
#[derive(Debug, Clone)]
pub enum FormattingQuality {
    Excellent,  // Clear, readable, appropriate formatting
    Good,       // Good formatting with minor issues
    Adequate,   // Basic formatting, functional
    Poor,       // Poor formatting choices
    Unreadable, // Very poor formatting, hard to read
}

/// Pattern matching analysis (Python 3.10+)
#[derive(Debug, Clone)]
pub struct PatternMatchingInfo {
    pub match_statement: String,
    pub pattern_types: Vec<PatternType>,
    pub complexity: PatternComplexity,
    pub has_guards: bool,
    pub is_exhaustive: bool,
    pub performance_characteristics: MatchPerformance,
    pub best_practices_score: i32,
}

/// Pattern types in match statements
#[derive(Debug, Clone)]
pub enum PatternType {
    LiteralPattern,  // case 42:
    VariablePattern, // case x:
    WildcardPattern, // case _:
    ValuePattern,    // case Color.RED:
    GroupPattern,    // case (x, y):
    SequencePattern, // case [x, *rest]:
    MappingPattern,  // case {"key": value}:
    ClassPattern,    // case Point(x, y):
    OrPattern,       // case x | y:
    AsPattern,       // case x as y:
    GuardedPattern,  // case x if condition:
}

/// Pattern matching complexity
#[derive(Debug, Clone)]
pub enum PatternComplexity {
    Simple,   // Basic literal/variable patterns
    Moderate, // Some structured patterns
    Complex,  // Complex nested patterns
    Advanced, // Very complex patterns with guards
}

/// Match statement performance characteristics
#[derive(Debug, Clone)]
pub enum MatchPerformance {
    Optimal,  // Efficient pattern matching
    Good,     // Good performance
    Fair,     // Adequate performance
    Poor,     // Inefficient patterns
    Critical, // Very poor performance
}

/// Generator analysis information
#[derive(Debug, Clone)]
pub struct GeneratorInfo {
    pub generator_type: GeneratorType,
    pub usage_pattern: GeneratorUsagePattern,
    pub memory_efficiency: MemoryEfficiency,
    pub complexity: GeneratorComplexity,
    pub is_async: bool,
    pub yield_analysis: YieldAnalysis,
    pub optimization_opportunities: Vec<String>,
}

/// Generator types
#[derive(Debug, Clone)]
pub enum GeneratorType {
    GeneratorFunction,   // def func(): yield
    GeneratorExpression, // (x for x in iterable)
    AsyncGenerator,      // async def func(): yield
    Comprehension,       // List/dict/set comprehensions
    IteratorProtocol,    // __iter__, __next__
}

/// Generator usage patterns
#[derive(Debug, Clone)]
pub enum GeneratorUsagePattern {
    SimpleIteration,    // Basic iteration
    DataTransformation, // Data processing pipeline
    LazyEvaluation,     // Lazy computation
    InfiniteSequence,   // Infinite generators
    Coroutine,          // Coroutine patterns
    Pipeline,           // Chained generators
}

/// Memory efficiency assessment
#[derive(Debug, Clone)]
pub enum MemoryEfficiency {
    Excellent, // Very memory efficient
    Good,      // Good memory usage
    Adequate,  // Acceptable memory usage
    Poor,      // Inefficient memory usage
    Critical,  // Very poor memory usage
}

/// Generator complexity
#[derive(Debug, Clone)]
pub enum GeneratorComplexity {
    Simple,   // Basic yield statements
    Moderate, // Some control flow
    Complex,  // Complex logic, multiple yields
    Advanced, // Very complex generators
}

/// Yield analysis
#[derive(Debug, Clone)]
pub struct YieldAnalysis {
    pub yield_count: usize,
    pub has_yield_from: bool,
    pub has_send_values: bool,
    pub has_throw_values: bool,
    pub has_close_handling: bool,
}

/// Modern decorator analysis
#[derive(Debug, Clone)]
pub struct ModernDecoratorInfo {
    pub decorator_name: String,
    pub decorator_category: DecoratorCategory,
    pub usage_pattern: DecoratorUsagePattern,
    pub complexity: DecoratorComplexity,
    pub is_factory: bool,
    pub is_async: bool,
    pub parameters: Vec<String>,
    pub best_practices_score: i32,
}

/// Decorator categories
#[derive(Debug, Clone)]
pub enum DecoratorCategory {
    BuiltIn,            // @property, @staticmethod, @classmethod
    FunctoolsDecorator, // @wraps, @singledispatch, @cache
    AsyncDecorator,     // Async-related decorators
    DataValidation,     // Pydantic, dataclass validators
    WebFramework,       // Flask, FastAPI decorators
    Testing,            // pytest, unittest decorators
    Performance,        // @lru_cache, timing decorators
    Custom,             // Custom user decorators
}

/// Decorator usage patterns
#[derive(Debug, Clone)]
pub enum DecoratorUsagePattern {
    SingleDecorator,        // Single decorator
    StackedDecorators,      // Multiple decorators
    ParameterizedDecorator, // Decorator with parameters
    ConditionalDecorator,   // Conditional application
    DynamicDecorator,       // Runtime decoration
}

/// Decorator complexity
#[derive(Debug, Clone)]
pub enum DecoratorComplexity {
    Simple,   // Basic decorators
    Moderate, // Parameterized decorators
    Complex,  // Complex decorator logic
    Advanced, // Very complex decorator patterns
}

/// Modern syntax features
#[derive(Debug, Clone)]
pub struct ModernSyntaxInfo {
    pub feature_type: ModernSyntaxType,
    pub python_version: String,
    pub usage_count: usize,
    pub complexity: SyntaxComplexity,
    pub best_practices_followed: bool,
    pub migration_suggestions: Vec<String>,
}

/// Modern syntax types
#[derive(Debug, Clone, PartialEq)]
pub enum ModernSyntaxType {
    WalrusOperator,       // := (Python 3.8+)
    PositionalOnlyParams, // def func(a, /, b) (Python 3.8+)
    TypeUnionOperator,    // int | str (Python 3.10+)
    ExceptionGroups,      // except* (Python 3.11+)
    GenericTypeHints,     // list[int] (Python 3.9+)
    StringPrefixChaining, // rf"string" (Python 3.6+)
    DictUnionOperator,    // dict1 | dict2 (Python 3.9+)
    RemovePrefix,         // str.removeprefix() (Python 3.9+)
    ContextVars,          // contextvars (Python 3.7+)
}

/// Syntax complexity
#[derive(Debug, Clone)]
pub enum SyntaxComplexity {
    Simple,   // Basic usage
    Moderate, // Standard usage
    Complex,  // Advanced usage
    Expert,   // Expert-level usage
}

/// Feature complexity assessment
#[derive(Debug, Clone)]
pub enum FeatureComplexity {
    Simple,   // Basic feature usage
    Moderate, // Standard feature usage
    Complex,  // Advanced feature usage
    Expert,   // Expert-level feature usage
}

/// Pattern for modern feature detection
#[derive(Debug, Clone)]
struct ModernFeaturePattern {
    name: String,
    pattern: Regex,
    #[allow(dead_code)] // Will be used for feature categorization
    feature_type: String,
    python_version: String,
    complexity: FeatureComplexity,
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
    dependency_patterns: HashMap<String, Vec<DependencyPattern>>,
    modern_feature_patterns: HashMap<String, Vec<ModernFeaturePattern>>,
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
    #[allow(dead_code)] // Will be used for pattern identification
    name: String,
    pattern: Regex,
    impact: String,
}

#[derive(Debug, Clone)]
struct SecurityPattern {
    #[allow(dead_code)] // Will be used for pattern identification
    name: String,
    pattern: Regex,
    vulnerability_type: VulnerabilityType,
    severity: VulnerabilitySeverity,
    description: String,
}

#[derive(Debug, Clone)]
struct PerformancePattern {
    #[allow(dead_code)] // Will be used for pattern identification
    name: String,
    pattern: Regex,
    optimization_type: OptimizationType,
    impact_level: ImpactLevel,
    description: String,
}

#[derive(Debug, Clone)]
struct FrameworkPattern {
    #[allow(dead_code)] // Will be used for pattern identification
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
            dependency_patterns: HashMap::new(),
            modern_feature_patterns: HashMap::new(),
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
                pattern: Regex::new(
                    r"(?:open|input|print)\s*\(.*\).*await|await.*(?:open|input|print)\s*\(",
                )
                .unwrap(),
                pattern_type: "performance_issue".to_string(),
                performance_impact: AsyncPerformanceImpact::Critical,
            },
        ];
        self.async_patterns
            .insert("functions".to_string(), async_patterns);

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
        self.async_patterns
            .insert("concurrency".to_string(), concurrency_patterns);

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
        self.async_patterns
            .insert("modern".to_string(), modern_async_patterns);

        // Dependency analysis patterns
        let requirements_patterns = vec![
            DependencyPattern {
                name: "Requirements.txt".to_string(),
                pattern: Regex::new(r"(?m)^([a-zA-Z0-9_-]+)([><=!~]+)?([0-9.]+)?").unwrap(),
                file_type: RequirementsFileType::RequirementsTxt,
                extraction_method: "line_by_line".to_string(),
            },
            DependencyPattern {
                name: "Pyproject.toml Dependencies".to_string(),
                pattern: Regex::new(r#"dependencies\s*=\s*\[(.*?)\]"#).unwrap(),
                file_type: RequirementsFileType::PyprojectToml,
                extraction_method: "toml_array".to_string(),
            },
            DependencyPattern {
                name: "Setup.py Install Requires".to_string(),
                pattern: Regex::new(r"install_requires\s*=\s*\[(.*?)\]").unwrap(),
                file_type: RequirementsFileType::SetupPy,
                extraction_method: "python_list".to_string(),
            },
            DependencyPattern {
                name: "Pipfile Dependencies".to_string(),
                pattern: Regex::new(r"\[packages\](.*?)\[").unwrap(),
                file_type: RequirementsFileType::Pipfile,
                extraction_method: "toml_section".to_string(),
            },
            DependencyPattern {
                name: "Poetry Lock".to_string(),
                pattern: Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap(),
                file_type: RequirementsFileType::PoetryLock,
                extraction_method: "toml_blocks".to_string(),
            },
            DependencyPattern {
                name: "Conda Environment".to_string(),
                pattern: Regex::new(r"dependencies:\s*\n(.*?)(?:\n\w|$)").unwrap(),
                file_type: RequirementsFileType::CondaYml,
                extraction_method: "yaml_list".to_string(),
            },
        ];
        self.dependency_patterns
            .insert("requirements".to_string(), requirements_patterns);

        let import_patterns = vec![
            DependencyPattern {
                name: "Standard Import".to_string(),
                pattern: Regex::new(r"(?m)^import\s+([a-zA-Z_][a-zA-Z0-9_.]*)").unwrap(),
                file_type: RequirementsFileType::SetupPy, // Python source files
                extraction_method: "import_statement".to_string(),
            },
            DependencyPattern {
                name: "From Import".to_string(),
                pattern: Regex::new(r"(?m)^from\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+import").unwrap(),
                file_type: RequirementsFileType::SetupPy, // Python source files
                extraction_method: "import_statement".to_string(),
            },
            DependencyPattern {
                name: "Star Import".to_string(),
                pattern: Regex::new(r"(?m)^from\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+import\s+\*")
                    .unwrap(),
                file_type: RequirementsFileType::SetupPy, // Python source files
                extraction_method: "import_statement".to_string(),
            },
            DependencyPattern {
                name: "Alias Import".to_string(),
                pattern: Regex::new(r"(?m)^import\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+as\s+").unwrap(),
                file_type: RequirementsFileType::SetupPy, // Python source files
                extraction_method: "import_statement".to_string(),
            },
            DependencyPattern {
                name: "Relative Import".to_string(),
                pattern: Regex::new(r"(?m)^from\s+(\.+)([a-zA-Z_][a-zA-Z0-9_.]*)\s+import")
                    .unwrap(),
                file_type: RequirementsFileType::SetupPy, // Python source files
                extraction_method: "import_statement".to_string(),
            },
        ];
        self.dependency_patterns
            .insert("imports".to_string(), import_patterns);

        // Modern Python feature patterns
        let dataclass_patterns = vec![
            ModernFeaturePattern {
                name: "Dataclass Decorator".to_string(),
                pattern: Regex::new(r"@dataclass").unwrap(),
                feature_type: "dataclass".to_string(),
                python_version: "3.7+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
            ModernFeaturePattern {
                name: "Pydantic Model".to_string(),
                pattern: Regex::new(r"class\s+\w+\(BaseModel\)").unwrap(),
                feature_type: "dataclass".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Named Tuple".to_string(),
                pattern: Regex::new(r"NamedTuple|namedtuple").unwrap(),
                feature_type: "dataclass".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Simple,
            },
            ModernFeaturePattern {
                name: "Slots Usage".to_string(),
                pattern: Regex::new(r"__slots__\s*=").unwrap(),
                feature_type: "dataclass".to_string(),
                python_version: "3.0+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
        ];
        self.modern_feature_patterns
            .insert("dataclass".to_string(), dataclass_patterns);

        let context_manager_patterns = vec![
            ModernFeaturePattern {
                name: "With Statement".to_string(),
                pattern: Regex::new(r"(?m)^(\s*)with\s+").unwrap(),
                feature_type: "context_manager".to_string(),
                python_version: "2.5+".to_string(),
                complexity: FeatureComplexity::Simple,
            },
            ModernFeaturePattern {
                name: "Async With".to_string(),
                pattern: Regex::new(r"(?m)^(\s*)async\s+with\s+").unwrap(),
                feature_type: "context_manager".to_string(),
                python_version: "3.5+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Context Manager Protocol".to_string(),
                pattern: Regex::new(r"__enter__|__exit__").unwrap(),
                feature_type: "context_manager".to_string(),
                python_version: "2.5+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Contextlib Manager".to_string(),
                pattern: Regex::new(r"@contextmanager").unwrap(),
                feature_type: "context_manager".to_string(),
                python_version: "2.5+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
        ];
        self.modern_feature_patterns
            .insert("context_manager".to_string(), context_manager_patterns);

        let fstring_patterns = vec![
            ModernFeaturePattern {
                name: "F-String Basic".to_string(),
                pattern: Regex::new(r#"f["'][^"']*\{[^}]+\}[^"']*["']"#).unwrap(),
                feature_type: "fstring".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Simple,
            },
            ModernFeaturePattern {
                name: "F-String Complex".to_string(),
                pattern: Regex::new(r#"f["'][^"']*\{[^}]*\([^)]*\)[^}]*\}[^"']*["']"#).unwrap(),
                feature_type: "fstring".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Raw F-String".to_string(),
                pattern: Regex::new(r#"rf["']|fr["']"#).unwrap(),
                feature_type: "fstring".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
        ];
        self.modern_feature_patterns
            .insert("fstring".to_string(), fstring_patterns);

        let pattern_matching_patterns = vec![
            ModernFeaturePattern {
                name: "Match Statement".to_string(),
                pattern: Regex::new(r"(?m)^(\s*)match\s+").unwrap(),
                feature_type: "pattern_matching".to_string(),
                python_version: "3.10+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Case Pattern".to_string(),
                pattern: Regex::new(r"(?m)^(\s*)case\s+").unwrap(),
                feature_type: "pattern_matching".to_string(),
                python_version: "3.10+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Guard Pattern".to_string(),
                pattern: Regex::new(r"case\s+.*\s+if\s+").unwrap(),
                feature_type: "pattern_matching".to_string(),
                python_version: "3.10+".to_string(),
                complexity: FeatureComplexity::Expert,
            },
        ];
        self.modern_feature_patterns
            .insert("pattern_matching".to_string(), pattern_matching_patterns);

        let generator_patterns = vec![
            ModernFeaturePattern {
                name: "Generator Function".to_string(),
                pattern: Regex::new(r"def\s+\w+.*?yield").unwrap(),
                feature_type: "generator".to_string(),
                python_version: "2.2+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
            ModernFeaturePattern {
                name: "Generator Expression".to_string(),
                pattern: Regex::new(r"\([^)]*for\s+\w+\s+in\s+[^)]*\)").unwrap(),
                feature_type: "generator".to_string(),
                python_version: "2.4+".to_string(),
                complexity: FeatureComplexity::Simple,
            },
            ModernFeaturePattern {
                name: "Async Generator".to_string(),
                pattern: Regex::new(r"async\s+def\s+\w+.*?yield").unwrap(),
                feature_type: "generator".to_string(),
                python_version: "3.6+".to_string(),
                complexity: FeatureComplexity::Expert,
            },
            ModernFeaturePattern {
                name: "Yield From".to_string(),
                pattern: Regex::new(r"yield\s+from\s+").unwrap(),
                feature_type: "generator".to_string(),
                python_version: "3.3+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
        ];
        self.modern_feature_patterns
            .insert("generator".to_string(), generator_patterns);

        let modern_syntax_patterns = vec![
            ModernFeaturePattern {
                name: "Walrus Operator".to_string(),
                pattern: Regex::new(r":=").unwrap(),
                feature_type: "modern_syntax".to_string(),
                python_version: "3.8+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
            ModernFeaturePattern {
                name: "Positional Only Parameters".to_string(),
                pattern: Regex::new(r"def\s+\w+\([^)]*,\s*/\s*[,)]").unwrap(),
                feature_type: "modern_syntax".to_string(),
                python_version: "3.8+".to_string(),
                complexity: FeatureComplexity::Complex,
            },
            ModernFeaturePattern {
                name: "Union Type Operator".to_string(),
                pattern: Regex::new(r"\w+\s*\|\s*\w+").unwrap(),
                feature_type: "modern_syntax".to_string(),
                python_version: "3.10+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
            ModernFeaturePattern {
                name: "Dictionary Union".to_string(),
                pattern: Regex::new(r"\w+\s*\|\s*\{").unwrap(),
                feature_type: "modern_syntax".to_string(),
                python_version: "3.9+".to_string(),
                complexity: FeatureComplexity::Simple,
            },
            ModernFeaturePattern {
                name: "Generic Type Hints".to_string(),
                pattern: Regex::new(r"list\[|dict\[|set\[|tuple\[").unwrap(),
                feature_type: "modern_syntax".to_string(),
                python_version: "3.9+".to_string(),
                complexity: FeatureComplexity::Moderate,
            },
        ];
        self.modern_feature_patterns
            .insert("modern_syntax".to_string(), modern_syntax_patterns);
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
                                description: format!(
                                    "Modern type feature: {name}",
                                    name = pattern.name
                                ),
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

    /// Analyze Python package dependencies comprehensively
    pub fn analyze_package_dependencies(
        &self,
        content: &str,
    ) -> Result<PythonPackageDependencyAnalysis> {
        let mut requirements_files = Vec::new();
        let mut dependencies = Vec::new();
        let mut dependency_issues = Vec::new();
        let mut virtual_environments = Vec::new();
        let mut import_analysis = Vec::new();
        let mut security_vulnerabilities = Vec::new();
        let mut license_analysis = Vec::new();

        // Analyze requirements files
        self.analyze_requirements_files(content, &mut requirements_files, &mut dependencies);

        // Analyze imports
        self.analyze_imports(content, &mut import_analysis);

        // Detect dependency issues
        self.detect_dependency_issues(
            content,
            &dependencies,
            &import_analysis,
            &mut dependency_issues,
        );

        // Detect virtual environments
        self.detect_virtual_environments(content, &mut virtual_environments);

        // Perform security vulnerability scanning
        self.scan_security_vulnerabilities(&dependencies, &mut security_vulnerabilities);

        // Analyze licenses
        self.analyze_licenses(&dependencies, &mut license_analysis);

        // Calculate overall health score
        let overall_health_score = self.calculate_dependency_health_score(
            &requirements_files,
            &dependencies,
            &dependency_issues,
            &security_vulnerabilities,
        );

        // Generate recommendations
        let recommendations = self.get_dependency_recommendations(
            &requirements_files,
            &dependencies,
            &dependency_issues,
            &import_analysis,
            &security_vulnerabilities,
        );

        Ok(PythonPackageDependencyAnalysis {
            overall_health_score,
            requirements_files,
            dependencies,
            dependency_issues,
            virtual_environments,
            import_analysis,
            security_vulnerabilities,
            license_analysis,
            recommendations,
        })
    }

    /// Analyze requirements files
    fn analyze_requirements_files(
        &self,
        content: &str,
        requirements_files: &mut Vec<RequirementsFileInfo>,
        dependencies: &mut Vec<RequirementInfo>,
    ) {
        for patterns in self.dependency_patterns.values() {
            for pattern in patterns {
                if pattern.extraction_method == "line_by_line" {
                    // Handle requirements.txt format
                    for line in content.lines() {
                        if let Some(captures) = pattern.pattern.captures(line) {
                            let package_name = captures.get(1).unwrap().as_str();
                            let version_spec = captures
                                .get(2)
                                .and_then(|m| {
                                    captures.get(3).map(|v| {
                                        let m_str = m.as_str();
                                        let v_str = v.as_str();
                                        format!("{m_str}{v_str}")
                                    })
                                })
                                .unwrap_or_else(|| "*".to_string());

                            dependencies.push(RequirementInfo {
                                name: package_name.to_string(),
                                version_spec,
                                source: RequirementSource::PyPI,
                                is_dev_dependency: false,
                                is_optional: false,
                                extras: Vec::new(),
                                markers: Vec::new(),
                                metadata: self.get_package_metadata(package_name),
                            });
                        }
                    }

                    if !dependencies.is_empty() {
                        requirements_files.push(RequirementsFileInfo {
                            file_path: "requirements.txt".to_string(),
                            file_type: RequirementsFileType::RequirementsTxt,
                            dependencies_count: dependencies.len(),
                            has_version_pins: dependencies.iter().any(|d| d.version_spec != "*"),
                            has_hashes: false,
                            uses_constraints: false,
                            quality_score: self.assess_requirements_quality(dependencies),
                        });
                    }
                } else if pattern.extraction_method == "toml_array"
                    && content.contains("pyproject.toml")
                {
                    // Handle pyproject.toml format
                    if let Some(captures) = pattern.pattern.captures(content) {
                        let deps_str = captures.get(1).unwrap().as_str();
                        for dep in deps_str.split(',') {
                            let clean_dep = dep.trim().trim_matches('"').trim_matches('\'');
                            if !clean_dep.is_empty() {
                                let parts: Vec<&str> =
                                    clean_dep.split(['>', '<', '=', '!', '~']).collect();
                                let package_name = parts[0].trim();
                                let version_spec = if parts.len() > 1 {
                                    clean_dep[package_name.len()..].to_string()
                                } else {
                                    "*".to_string()
                                };

                                dependencies.push(RequirementInfo {
                                    name: package_name.to_string(),
                                    version_spec,
                                    source: RequirementSource::PyPI,
                                    is_dev_dependency: false,
                                    is_optional: false,
                                    extras: Vec::new(),
                                    markers: Vec::new(),
                                    metadata: self.get_package_metadata(package_name),
                                });
                            }
                        }

                        requirements_files.push(RequirementsFileInfo {
                            file_path: "pyproject.toml".to_string(),
                            file_type: RequirementsFileType::PyprojectToml,
                            dependencies_count: dependencies.len(),
                            has_version_pins: dependencies.iter().any(|d| d.version_spec != "*"),
                            has_hashes: false,
                            uses_constraints: false,
                            quality_score: self.assess_requirements_quality(dependencies),
                        });
                    }
                }
            }
        }
    }

    /// Analyze import statements
    fn analyze_imports(&self, content: &str, import_analysis: &mut Vec<ImportAnalysisInfo>) {
        if let Some(import_patterns) = self.dependency_patterns.get("imports") {
            for pattern in import_patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let module_name = if pattern.name == "Relative Import" && captures.len() > 2 {
                        format!(
                            "{}{}",
                            captures.get(1).unwrap().as_str(),
                            captures.get(2).unwrap().as_str()
                        )
                    } else {
                        captures.get(1).unwrap().as_str().to_string()
                    };

                    let import_type = self.classify_import_type(&pattern.name);
                    let module_category = self.categorize_module(&module_name);
                    let usage_count = content.matches(&module_name).count();
                    let is_unused = usage_count <= 1; // Only the import itself
                    let import_issues =
                        self.detect_import_issues(&pattern.name, &module_name, content);

                    import_analysis.push(ImportAnalysisInfo {
                        import_statement: captures.get(0).unwrap().as_str().to_string(),
                        import_type,
                        module_category,
                        usage_count,
                        is_unused,
                        import_issues,
                        optimization_suggestions: self
                            .get_import_optimization_suggestions(&pattern.name, &module_name),
                    });
                }
            }
        }
    }

    /// Detect dependency issues
    fn detect_dependency_issues(
        &self,
        _content: &str,
        dependencies: &[RequirementInfo],
        import_analysis: &[ImportAnalysisInfo],
        issues: &mut Vec<DependencyIssue>,
    ) {
        // Detect unused dependencies
        for dep in dependencies {
            let is_imported = import_analysis.iter().any(|imp| {
                imp.import_statement.contains(&dep.name)
                    || imp.import_statement.contains(&dep.name.replace("-", "_"))
            });

            if !is_imported {
                issues.push(DependencyIssue {
                    issue_type: DependencyIssueType::UnusedDependency,
                    severity: DependencyIssueSeverity::Low,
                    affected_packages: vec![dep.name.clone()],
                    description: format!(
                        "Dependency '{name}' declared but not imported",
                        name = dep.name
                    ),
                    recommendation: "Remove unused dependency or add import statement".to_string(),
                    auto_fixable: false,
                });
            }
        }

        // Detect missing dependencies
        for import in import_analysis {
            if matches!(import.module_category, ModuleCategory::ThirdParty) {
                let module_root = import
                    .import_statement
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .split('.')
                    .next()
                    .unwrap_or("");

                let is_declared = dependencies.iter().any(|dep| {
                    dep.name == module_root || dep.name.replace("-", "_") == module_root
                });

                if !is_declared && !module_root.is_empty() {
                    issues.push(DependencyIssue {
                        issue_type: DependencyIssueType::MissingDependency,
                        severity: DependencyIssueSeverity::High,
                        affected_packages: vec![module_root.to_string()],
                        description: format!(
                            "Module '{module_root}' imported but not declared as dependency"
                        ),
                        recommendation: "Add missing dependency to requirements".to_string(),
                        auto_fixable: true,
                    });
                }
            }
        }

        // Detect unpinned versions
        for dep in dependencies {
            if dep.version_spec == "*" || dep.version_spec.is_empty() {
                issues.push(DependencyIssue {
                    issue_type: DependencyIssueType::UnpinnedVersion,
                    severity: DependencyIssueSeverity::Medium,
                    affected_packages: vec![dep.name.clone()],
                    description: format!(
                        "Dependency '{name}' has no version constraint",
                        name = dep.name
                    ),
                    recommendation: "Pin dependency versions for reproducible builds".to_string(),
                    auto_fixable: false,
                });
            }
        }

        // Detect deprecated packages
        let deprecated_packages = ["imp", "optparse", "platform", "distutils"];
        for dep in dependencies {
            if deprecated_packages.contains(&dep.name.as_str()) {
                issues.push(DependencyIssue {
                    issue_type: DependencyIssueType::DeprecatedPackage,
                    severity: DependencyIssueSeverity::Medium,
                    affected_packages: vec![dep.name.clone()],
                    description: format!("Package '{name}' is deprecated", name = dep.name),
                    recommendation: "Consider migrating to modern alternatives".to_string(),
                    auto_fixable: false,
                });
            }
        }
    }

    /// Detect virtual environments
    fn detect_virtual_environments(
        &self,
        content: &str,
        virtual_environments: &mut Vec<VirtualEnvironmentInfo>,
    ) {
        // Check for virtual environment indicators
        if content.contains("venv") || content.contains("virtualenv") {
            virtual_environments.push(VirtualEnvironmentInfo {
                env_type: VirtualEnvironmentType::Venv,
                location: "./venv".to_string(),
                python_version: "3.x".to_string(),
                is_active: true,
                packages_count: 0,
                env_variables: Vec::new(),
                configuration: VirtualEnvironmentConfig {
                    isolated: true,
                    system_site_packages: false,
                    pip_version: None,
                    setuptools_version: None,
                    custom_configurations: Vec::new(),
                },
            });
        }

        if content.contains("conda") || content.contains("environment.yml") {
            virtual_environments.push(VirtualEnvironmentInfo {
                env_type: VirtualEnvironmentType::Conda,
                location: "conda environment".to_string(),
                python_version: "3.x".to_string(),
                is_active: true,
                packages_count: 0,
                env_variables: Vec::new(),
                configuration: VirtualEnvironmentConfig {
                    isolated: true,
                    system_site_packages: false,
                    pip_version: None,
                    setuptools_version: None,
                    custom_configurations: Vec::new(),
                },
            });
        }

        if content.contains("pipenv") || content.contains("Pipfile") {
            virtual_environments.push(VirtualEnvironmentInfo {
                env_type: VirtualEnvironmentType::Pipenv,
                location: "pipenv environment".to_string(),
                python_version: "3.x".to_string(),
                is_active: true,
                packages_count: 0,
                env_variables: Vec::new(),
                configuration: VirtualEnvironmentConfig {
                    isolated: true,
                    system_site_packages: false,
                    pip_version: None,
                    setuptools_version: None,
                    custom_configurations: Vec::new(),
                },
            });
        }
    }

    /// Scan for security vulnerabilities
    fn scan_security_vulnerabilities(
        &self,
        dependencies: &[RequirementInfo],
        vulnerabilities: &mut Vec<SecurityVulnerabilityInfo>,
    ) {
        // Simplified vulnerability database - in production, this would query real databases
        let known_vulnerabilities = vec![
            ("urllib3", "1.25.8", "CVE-2020-26137", "Critical"),
            ("requests", "2.19.1", "CVE-2018-18074", "High"),
            ("pyyaml", "5.3.1", "CVE-2020-14343", "High"),
            ("django", "2.2.12", "CVE-2020-13254", "Medium"),
            ("flask", "1.1.1", "CVE-2019-1010083", "Medium"),
        ];

        for dep in dependencies {
            for (vuln_package, vuln_version, cve_id, severity) in &known_vulnerabilities {
                if dep.name == *vuln_package {
                    let severity_enum = match *severity {
                        "Critical" => SecurityVulnerabilitySeverity::Critical,
                        "High" => SecurityVulnerabilitySeverity::High,
                        "Medium" => SecurityVulnerabilitySeverity::Medium,
                        _ => SecurityVulnerabilitySeverity::Low,
                    };

                    vulnerabilities.push(SecurityVulnerabilityInfo {
                        cve_id: Some(cve_id.to_string()),
                        advisory_id: None,
                        package_name: dep.name.clone(),
                        affected_versions: vec![vuln_version.to_string()],
                        fixed_version: Some("Latest".to_string()),
                        severity: severity_enum,
                        vulnerability_type: VulnerabilityCategory::CodeExecution,
                        description: format!(
                            "Security vulnerability in {vuln_package} {vuln_version}"
                        ),
                        references: vec![format!(
                            "https://cve.mitre.org/cgi-bin/cvename.cgi?name={cve_id}"
                        )],
                        published_date: Some("2020-01-01".to_string()),
                        last_modified: Some("2020-01-01".to_string()),
                    });
                }
            }
        }
    }

    /// Analyze package licenses
    fn analyze_licenses(
        &self,
        dependencies: &[RequirementInfo],
        license_analysis: &mut Vec<LicenseInfo>,
    ) {
        for dep in dependencies {
            // Use metadata from package
            let license_type = self.parse_license_type(&dep.metadata.license);
            let compatibility = self.assess_license_compatibility(&license_type);

            license_analysis.push(LicenseInfo {
                package_name: dep.name.clone(),
                license_type: license_type.clone(),
                license_text: None,
                compatibility,
                commercial_use_allowed: self.is_commercial_use_allowed(&license_type),
                distribution_allowed: self.is_distribution_allowed(&license_type),
                modification_allowed: self.is_modification_allowed(&license_type),
                patent_grant: self.has_patent_grant(&license_type),
                copyleft: self.is_copyleft(&license_type),
            });
        }
    }

    /// Helper methods for dependency analysis
    fn get_package_metadata(&self, package_name: &str) -> PackageMetadata {
        // Simplified metadata - in production, this would query PyPI API
        PackageMetadata {
            description: format!("Package: {package_name}"),
            author: "Unknown".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            documentation: None,
            last_updated: None,
            download_count: None,
            maintenance_status: MaintenanceStatus::Unknown,
        }
    }

    fn assess_requirements_quality(
        &self,
        dependencies: &[RequirementInfo],
    ) -> DependencyQualityScore {
        let pinned_count = dependencies
            .iter()
            .filter(|d| d.version_spec != "*")
            .count();
        let pinned_ratio = if dependencies.is_empty() {
            0.0
        } else {
            pinned_count as f32 / dependencies.len() as f32
        };

        match pinned_ratio {
            r if r >= 0.9 => DependencyQualityScore::Excellent,
            r if r >= 0.7 => DependencyQualityScore::Good,
            r if r >= 0.5 => DependencyQualityScore::Fair,
            r if r >= 0.3 => DependencyQualityScore::Poor,
            _ => DependencyQualityScore::Critical,
        }
    }

    fn classify_import_type(&self, pattern_name: &str) -> ImportType {
        match pattern_name {
            "Standard Import" => ImportType::StandardImport,
            "From Import" => ImportType::FromImport,
            "Star Import" => ImportType::StarImport,
            "Alias Import" => ImportType::AliasImport,
            "Relative Import" => ImportType::RelativeImport,
            _ => ImportType::StandardImport,
        }
    }

    fn categorize_module(&self, module_name: &str) -> ModuleCategory {
        // Standard library modules
        let stdlib_modules = vec![
            "os",
            "sys",
            "re",
            "json",
            "urllib",
            "http",
            "datetime",
            "collections",
            "itertools",
            "functools",
            "pathlib",
            "typing",
            "asyncio",
            "threading",
            "multiprocessing",
            "subprocess",
            "logging",
            "unittest",
            "sqlite3",
        ];

        let root_module = module_name.split('.').next().unwrap_or(module_name);

        if stdlib_modules.contains(&root_module) {
            ModuleCategory::StandardLibrary
        } else if root_module.starts_with('.') {
            ModuleCategory::Local
        } else {
            ModuleCategory::ThirdParty
        }
    }

    fn detect_import_issues(
        &self,
        pattern_name: &str,
        module_name: &str,
        content: &str,
    ) -> Vec<ImportIssue> {
        let mut issues = Vec::new();

        if pattern_name == "Star Import" {
            issues.push(ImportIssue::StarImportDangerous);
        }

        // Check for circular imports (simplified)
        if content.contains(&format!("from {module_name} import"))
            && content.contains(&format!("import {module_name}"))
        {
            issues.push(ImportIssue::CircularImport);
        }

        // Check for deprecated modules
        let deprecated_modules = ["imp", "optparse", "platform.dist"];
        if deprecated_modules
            .iter()
            .any(|&dep| module_name.contains(dep))
        {
            issues.push(ImportIssue::DeprecatedImport);
        }

        issues
    }

    fn get_import_optimization_suggestions(
        &self,
        pattern_name: &str,
        module_name: &str,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if pattern_name == "Star Import" {
            suggestions.push("Replace star import with specific imports".to_string());
        }

        if module_name == "pandas" || module_name == "numpy" {
            suggestions.push("Consider lazy loading for large libraries".to_string());
        }

        suggestions
    }

    fn parse_license_type(&self, license_str: &str) -> LicenseType {
        match license_str.to_lowercase().as_str() {
            "mit" => LicenseType::MIT,
            "apache-2.0" | "apache 2.0" => LicenseType::Apache2,
            "gpl-2.0" | "gpl v2" => LicenseType::GPL2,
            "gpl-3.0" | "gpl v3" => LicenseType::GPL3,
            "bsd-2-clause" => LicenseType::BSD2Clause,
            "bsd-3-clause" => LicenseType::BSD3Clause,
            "lgpl" => LicenseType::LGPL,
            "mozilla" | "mpl-2.0" => LicenseType::Mozilla,
            "unlicense" => LicenseType::Unlicense,
            _ => LicenseType::Unknown,
        }
    }

    fn assess_license_compatibility(&self, license_type: &LicenseType) -> LicenseCompatibility {
        match license_type {
            LicenseType::MIT
            | LicenseType::Apache2
            | LicenseType::BSD2Clause
            | LicenseType::BSD3Clause => LicenseCompatibility::Compatible,
            LicenseType::GPL2 | LicenseType::GPL3 => LicenseCompatibility::RequiresReview,
            LicenseType::LGPL => LicenseCompatibility::ConditionallyCompatible,
            _ => LicenseCompatibility::Unknown,
        }
    }

    fn is_commercial_use_allowed(&self, license_type: &LicenseType) -> bool {
        !matches!(license_type, LicenseType::GPL2 | LicenseType::GPL3)
    }

    fn is_distribution_allowed(&self, license_type: &LicenseType) -> bool {
        !matches!(license_type, LicenseType::Proprietary)
    }

    fn is_modification_allowed(&self, license_type: &LicenseType) -> bool {
        !matches!(license_type, LicenseType::Proprietary)
    }

    fn has_patent_grant(&self, license_type: &LicenseType) -> bool {
        matches!(license_type, LicenseType::Apache2 | LicenseType::Mozilla)
    }

    fn is_copyleft(&self, license_type: &LicenseType) -> bool {
        matches!(
            license_type,
            LicenseType::GPL2 | LicenseType::GPL3 | LicenseType::LGPL
        )
    }

    fn calculate_dependency_health_score(
        &self,
        requirements_files: &[RequirementsFileInfo],
        dependencies: &[RequirementInfo],
        issues: &[DependencyIssue],
        vulnerabilities: &[SecurityVulnerabilityInfo],
    ) -> i32 {
        let mut score = 100;

        // Deduct points for issues
        for issue in issues {
            let deduction = match issue.severity {
                DependencyIssueSeverity::Critical => 20,
                DependencyIssueSeverity::High => 15,
                DependencyIssueSeverity::Medium => 10,
                DependencyIssueSeverity::Low => 5,
                DependencyIssueSeverity::Info => 2,
            };
            score -= deduction;
        }

        // Deduct points for vulnerabilities
        for vuln in vulnerabilities {
            let deduction = match vuln.severity {
                SecurityVulnerabilitySeverity::Critical => 25,
                SecurityVulnerabilitySeverity::High => 20,
                SecurityVulnerabilitySeverity::Medium => 15,
                SecurityVulnerabilitySeverity::Low => 10,
                _ => 5,
            };
            score -= deduction;
        }

        // Add points for good practices
        if !requirements_files.is_empty() {
            score += 10;
        }

        let pinned_deps = dependencies
            .iter()
            .filter(|d| d.version_spec != "*")
            .count();
        let pinned_ratio = if dependencies.is_empty() {
            0.0
        } else {
            pinned_deps as f32 / dependencies.len() as f32
        };

        score += (pinned_ratio * 20.0) as i32;

        score.clamp(0, 100)
    }

    fn get_dependency_recommendations(
        &self,
        requirements_files: &[RequirementsFileInfo],
        dependencies: &[RequirementInfo],
        issues: &[DependencyIssue],
        import_analysis: &[ImportAnalysisInfo],
        vulnerabilities: &[SecurityVulnerabilityInfo],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if requirements_files.is_empty() {
            recommendations
                .push("Create a requirements.txt file to track dependencies".to_string());
        }

        let unpinned_count = dependencies
            .iter()
            .filter(|d| d.version_spec == "*")
            .count();
        if unpinned_count > 0 {
            recommendations.push(format!(
                "Pin {unpinned_count} unpinned dependencies for reproducible builds"
            ));
        }

        if !vulnerabilities.is_empty() {
            recommendations.push(format!(
                "Update {} packages with known security vulnerabilities",
                vulnerabilities.len()
            ));
        }

        let unused_imports = import_analysis.iter().filter(|i| i.is_unused).count();
        if unused_imports > 0 {
            recommendations.push(format!("Remove {unused_imports} unused import statements"));
        }

        let star_imports = import_analysis
            .iter()
            .filter(|i| matches!(i.import_type, ImportType::StarImport))
            .count();
        if star_imports > 0 {
            recommendations.push(format!(
                "Replace {star_imports} star imports with specific imports"
            ));
        }

        let critical_issues = issues
            .iter()
            .filter(|i| matches!(i.severity, DependencyIssueSeverity::Critical))
            .count();
        if critical_issues > 0 {
            recommendations.push("Address critical dependency issues immediately".to_string());
        }

        recommendations
            .push("Consider using dependency scanning tools like Safety or Bandit".to_string());
        recommendations.push("Set up automated dependency updates with Dependabot".to_string());

        recommendations
    }

    /// Analyze modern Python features comprehensively
    pub fn analyze_modern_features(&self, content: &str) -> Result<ModernPythonFeatureAnalysis> {
        let mut dataclass_features = Vec::new();
        let mut context_manager_features = Vec::new();
        let mut fstring_features = Vec::new();
        let mut pattern_matching_features = Vec::new();
        let mut generator_features = Vec::new();
        let mut decorator_features = Vec::new();
        let mut modern_syntax_features = Vec::new();

        // Analyze dataclasses
        self.analyze_dataclasses(content, &mut dataclass_features);

        // Analyze context managers
        self.analyze_context_managers(content, &mut context_manager_features);

        // Analyze f-strings
        self.analyze_fstrings(content, &mut fstring_features);

        // Analyze pattern matching
        self.analyze_pattern_matching(content, &mut pattern_matching_features);

        // Analyze generators
        self.analyze_generators(content, &mut generator_features);

        // Analyze modern decorators
        self.analyze_modern_decorators(content, &mut decorator_features);

        // Analyze modern syntax
        self.analyze_modern_syntax(content, &mut modern_syntax_features);

        // Detect Python version and compatibility
        let python_version_detected = self.detect_python_version(content);

        // Calculate overall modernity score
        let overall_modernity_score = self.calculate_modernity_score(
            &dataclass_features,
            &context_manager_features,
            &fstring_features,
            &pattern_matching_features,
            &generator_features,
            &decorator_features,
            &modern_syntax_features,
        );

        // Generate recommendations
        let recommendations = self.get_modern_feature_recommendations(
            &dataclass_features,
            &context_manager_features,
            &fstring_features,
            &pattern_matching_features,
            &generator_features,
            &decorator_features,
            &modern_syntax_features,
            &python_version_detected,
        );

        Ok(ModernPythonFeatureAnalysis {
            overall_modernity_score,
            python_version_detected,
            dataclass_features,
            context_manager_features,
            fstring_features,
            pattern_matching_features,
            generator_features,
            decorator_features,
            modern_syntax_features,
            recommendations,
        })
    }

    /// Analyze dataclass usage patterns
    fn analyze_dataclasses(&self, content: &str, dataclass_features: &mut Vec<DataclassInfo>) {
        if let Some(patterns) = self.modern_feature_patterns.get("dataclass") {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();

                    let dataclass_type = match pattern.name.as_str() {
                        "Dataclass Decorator" => DataclassType::StandardDataclass,
                        "Pydantic Model" => DataclassType::PydanticModel,
                        "Named Tuple" => DataclassType::NamedTuple,
                        _ => DataclassType::StandardDataclass,
                    };

                    let class_name = self.extract_dataclass_name(full_match, content);
                    let fields = self.analyze_dataclass_fields(content, &class_name);
                    let features_used = self.detect_dataclass_features(content, &class_name);
                    let complexity = self.assess_dataclass_complexity(&fields, &features_used);
                    let best_practices_score =
                        self.score_dataclass_best_practices(&fields, &features_used);
                    let recommendations =
                        self.get_dataclass_recommendations(&features_used, best_practices_score);

                    dataclass_features.push(DataclassInfo {
                        class_name,
                        dataclass_type,
                        fields,
                        features_used,
                        complexity,
                        best_practices_score,
                        recommendations,
                    });
                }
            }
        }
    }

    /// Analyze context manager usage
    fn analyze_context_managers(
        &self,
        content: &str,
        context_features: &mut Vec<ContextManagerInfo>,
    ) {
        if let Some(patterns) = self.modern_feature_patterns.get("context_manager") {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();

                    let context_type = match pattern.name.as_str() {
                        "With Statement" => ContextManagerType::BuiltInFileManager,
                        "Async With" => ContextManagerType::AsyncContextManager,
                        "Context Manager Protocol" => ContextManagerType::CustomContextManager,
                        "Contextlib Manager" => ContextManagerType::ContextlibManager,
                        _ => ContextManagerType::BuiltInFileManager,
                    };

                    let usage_pattern = self.analyze_context_usage_pattern(content, full_match);
                    let resource_management =
                        self.assess_resource_management_quality(content, full_match);
                    let error_handling = self.assess_context_error_handling(content, full_match);
                    let is_async = pattern.name.contains("Async");
                    let nested_level = self.calculate_context_nesting_level(content, full_match);
                    let best_practices_followed =
                        self.check_context_best_practices(content, full_match);

                    context_features.push(ContextManagerInfo {
                        context_type,
                        usage_pattern,
                        resource_management,
                        error_handling,
                        is_async,
                        nested_level,
                        best_practices_followed,
                    });
                }
            }
        }
    }

    /// Analyze f-string usage
    fn analyze_fstrings(&self, content: &str, fstring_features: &mut Vec<FStringInfo>) {
        if let Some(patterns) = self.modern_feature_patterns.get("fstring") {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let expression = captures.get(0).unwrap().as_str().to_string();

                    let complexity = self.assess_fstring_complexity(&expression);
                    let features_used = self.detect_fstring_features(&expression);
                    let performance_impact =
                        self.assess_fstring_performance(&expression, &features_used);
                    let formatting_quality = self.assess_formatting_quality(&expression);
                    let readability_score = self.calculate_fstring_readability(&expression);

                    fstring_features.push(FStringInfo {
                        expression,
                        complexity,
                        features_used,
                        performance_impact,
                        formatting_quality,
                        readability_score,
                    });
                }
            }
        }
    }

    /// Analyze pattern matching (Python 3.10+)
    fn analyze_pattern_matching(
        &self,
        content: &str,
        pattern_features: &mut Vec<PatternMatchingInfo>,
    ) {
        if let Some(_patterns) = self.modern_feature_patterns.get("pattern_matching") {
            // Look for complete match statements
            let match_regex = Regex::new(r"(?m)^(\s*)match\s+[^:]+:(.*?)$").unwrap();
            for captures in match_regex.captures_iter(content) {
                let match_statement = captures.get(0).unwrap().as_str().to_string();

                let pattern_types = self.analyze_match_patterns(&match_statement);
                let complexity = self.assess_pattern_complexity(&pattern_types);
                let has_guards = match_statement.contains("if ");
                let is_exhaustive = self.check_pattern_exhaustiveness(&match_statement);
                let performance_characteristics =
                    self.assess_match_performance(&pattern_types, &match_statement);
                let best_practices_score =
                    self.score_pattern_best_practices(&pattern_types, has_guards, is_exhaustive);

                pattern_features.push(PatternMatchingInfo {
                    match_statement,
                    pattern_types,
                    complexity,
                    has_guards,
                    is_exhaustive,
                    performance_characteristics,
                    best_practices_score,
                });
            }
        }
    }

    /// Analyze generator patterns
    fn analyze_generators(&self, content: &str, generator_features: &mut Vec<GeneratorInfo>) {
        if let Some(patterns) = self.modern_feature_patterns.get("generator") {
            for pattern in patterns {
                for captures in pattern.pattern.captures_iter(content) {
                    let full_match = captures.get(0).unwrap().as_str();

                    let generator_type = match pattern.name.as_str() {
                        "Generator Function" => GeneratorType::GeneratorFunction,
                        "Generator Expression" => GeneratorType::GeneratorExpression,
                        "Async Generator" => GeneratorType::AsyncGenerator,
                        _ => GeneratorType::GeneratorFunction,
                    };

                    let usage_pattern = self.classify_generator_usage_pattern(content, full_match);
                    let memory_efficiency =
                        self.assess_generator_memory_efficiency(content, full_match);
                    let complexity = self.assess_generator_complexity(content, full_match);
                    let is_async = pattern.name.contains("Async");
                    let yield_analysis = self.analyze_yield_usage(content, full_match);
                    let optimization_opportunities =
                        self.identify_generator_optimizations(content, full_match);

                    generator_features.push(GeneratorInfo {
                        generator_type,
                        usage_pattern,
                        memory_efficiency,
                        complexity,
                        is_async,
                        yield_analysis,
                        optimization_opportunities,
                    });
                }
            }
        }
    }

    /// Analyze modern decorator usage
    fn analyze_modern_decorators(
        &self,
        content: &str,
        decorator_features: &mut Vec<ModernDecoratorInfo>,
    ) {
        let decorator_regex = Regex::new(r"@(\w+)(?:\([^)]*\))?").unwrap();
        for captures in decorator_regex.captures_iter(content) {
            let decorator_name = captures.get(1).unwrap().as_str().to_string();
            let full_decorator = captures.get(0).unwrap().as_str();

            let decorator_category = self.classify_decorator_category(&decorator_name);
            let usage_pattern = self.analyze_decorator_usage_pattern(content, full_decorator);
            let complexity = self.assess_decorator_complexity(full_decorator);
            let is_factory = full_decorator.contains('(');
            let is_async = content.contains("async def") && content.contains(&decorator_name);
            let parameters = self.extract_decorator_parameters(full_decorator);
            let best_practices_score =
                self.score_decorator_best_practices(&decorator_category, &usage_pattern);

            decorator_features.push(ModernDecoratorInfo {
                decorator_name,
                decorator_category,
                usage_pattern,
                complexity,
                is_factory,
                is_async,
                parameters,
                best_practices_score,
            });
        }
    }

    /// Analyze modern syntax features
    fn analyze_modern_syntax(&self, content: &str, syntax_features: &mut Vec<ModernSyntaxInfo>) {
        if let Some(patterns) = self.modern_feature_patterns.get("modern_syntax") {
            for pattern in patterns {
                let usage_count = pattern.pattern.find_iter(content).count();
                if usage_count > 0 {
                    let feature_type = match pattern.name.as_str() {
                        "Walrus Operator" => ModernSyntaxType::WalrusOperator,
                        "Positional Only Parameters" => ModernSyntaxType::PositionalOnlyParams,
                        "Union Type Operator" => ModernSyntaxType::TypeUnionOperator,
                        "Dictionary Union" => ModernSyntaxType::DictUnionOperator,
                        "Generic Type Hints" => ModernSyntaxType::GenericTypeHints,
                        _ => ModernSyntaxType::WalrusOperator,
                    };

                    let complexity = self.assess_syntax_complexity(content, &pattern.pattern);
                    let best_practices_followed =
                        self.check_syntax_best_practices(content, &feature_type);
                    let migration_suggestions =
                        self.get_syntax_migration_suggestions(&feature_type, usage_count);

                    syntax_features.push(ModernSyntaxInfo {
                        feature_type,
                        python_version: pattern.python_version.clone(),
                        usage_count,
                        complexity,
                        best_practices_followed,
                        migration_suggestions,
                    });
                }
            }
        }
    }

    /// Detect minimum Python version required
    fn detect_python_version(&self, content: &str) -> PythonVersionDetected {
        let mut features_by_version = Vec::new();
        let mut compatibility_issues = Vec::new();
        let mut minimum_version = "3.6".to_string(); // Default modern minimum

        // Check for version-specific features
        for patterns in self.modern_feature_patterns.values() {
            for pattern in patterns {
                if pattern.pattern.is_match(content) {
                    let version_required = pattern.python_version.clone();

                    // Update minimum version if this feature requires newer version
                    if self.is_newer_version(&version_required, &minimum_version) {
                        minimum_version = version_required.clone();
                    }

                    features_by_version.push(VersionFeature {
                        feature_name: pattern.name.clone(),
                        python_version: version_required,
                        usage_count: pattern.pattern.find_iter(content).count(),
                        is_best_practice: matches!(
                            pattern.complexity,
                            FeatureComplexity::Simple | FeatureComplexity::Moderate
                        ),
                    });
                }
            }
        }

        // Check for compatibility issues
        if content.contains("print ") && !content.contains("print(") {
            compatibility_issues.push(CompatibilityIssue {
                issue_type: CompatibilityIssueType::SyntaxError,
                severity: CompatibilitySeverity::Critical,
                feature_name: "Print Statement".to_string(),
                required_version: "2.x".to_string(),
                description: "Print statement syntax not supported in Python 3".to_string(),
                recommendation: "Use print() function instead".to_string(),
            });
        }

        PythonVersionDetected {
            minimum_version,
            features_by_version,
            compatibility_issues,
        }
    }

    /// Helper methods for modern feature analysis
    fn extract_dataclass_name(&self, _match: &str, content: &str) -> String {
        // Look for class definition near the dataclass decorator
        if let Some(class_match) = Regex::new(r"class\s+(\w+)").unwrap().find(content) {
            if let Some(captures) = Regex::new(r"class\s+(\w+)")
                .unwrap()
                .captures(class_match.as_str())
            {
                return captures.get(1).unwrap().as_str().to_string();
            }
        }
        "UnknownClass".to_string()
    }

    fn analyze_dataclass_fields(&self, content: &str, class_name: &str) -> Vec<DataclassField> {
        // Simplified field analysis - in production this would parse the class body
        let mut fields = Vec::new();

        // Look for field annotations in the class
        let field_regex = Regex::new(&format!(r"class\s+{class_name}.*?(\w+):\s*(\w+)")).unwrap();
        for captures in field_regex.captures_iter(content) {
            if captures.len() >= 3 {
                let field_name = captures.get(1).unwrap().as_str().to_string();
                let field_type = captures.get(2).unwrap().as_str().to_string();

                fields.push(DataclassField {
                    name: field_name,
                    field_type,
                    has_default: false,
                    is_optional: false,
                    validation_rules: Vec::new(),
                    metadata: Vec::new(),
                });
            }
        }

        fields
    }

    fn detect_dataclass_features(&self, content: &str, _class_name: &str) -> Vec<DataclassFeature> {
        let mut features = Vec::new();

        if content.contains("frozen=True") {
            features.push(DataclassFeature::FrozenClass);
        }
        if content.contains("__slots__") {
            features.push(DataclassFeature::SlotsUsage);
        }
        if content.contains("__post_init__") {
            features.push(DataclassFeature::PostInitProcessing);
        }
        if content.contains("default_factory") {
            features.push(DataclassFeature::FieldFactories);
        }

        features
    }

    fn assess_dataclass_complexity(
        &self,
        fields: &[DataclassField],
        features: &[DataclassFeature],
    ) -> FeatureComplexity {
        let complexity_score = fields.len() + features.len() * 2;

        match complexity_score {
            0..=3 => FeatureComplexity::Simple,
            4..=8 => FeatureComplexity::Moderate,
            9..=15 => FeatureComplexity::Complex,
            _ => FeatureComplexity::Expert,
        }
    }

    fn score_dataclass_best_practices(
        &self,
        _fields: &[DataclassField],
        features: &[DataclassFeature],
    ) -> i32 {
        let mut score = 60; // Base score

        // Add points for good practices
        if features.contains(&DataclassFeature::FrozenClass) {
            score += 15; // Immutability is good
        }
        if features.contains(&DataclassFeature::SlotsUsage) {
            score += 10; // Memory optimization
        }
        if features.contains(&DataclassFeature::PostInitProcessing) {
            score += 10; // Proper initialization
        }

        score.min(100)
    }

    fn get_dataclass_recommendations(
        &self,
        features: &[DataclassFeature],
        score: i32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score < 70 {
            recommendations.push("Consider using dataclass best practices".to_string());
        }

        if !features.contains(&DataclassFeature::SlotsUsage) {
            recommendations.push("Consider using __slots__ for memory optimization".to_string());
        }

        if !features.contains(&DataclassFeature::FrozenClass) {
            recommendations.push("Consider making dataclass frozen for immutability".to_string());
        }

        recommendations
    }

    /// Calculate overall modernity score
    #[allow(clippy::too_many_arguments)] // Complex analysis requires multiple feature sets
    fn calculate_modernity_score(
        &self,
        dataclass_features: &[DataclassInfo],
        context_manager_features: &[ContextManagerInfo],
        fstring_features: &[FStringInfo],
        pattern_matching_features: &[PatternMatchingInfo],
        generator_features: &[GeneratorInfo],
        decorator_features: &[ModernDecoratorInfo],
        modern_syntax_features: &[ModernSyntaxInfo],
    ) -> i32 {
        let mut score = 50i32; // Base score

        // Add points for modern features
        score += (dataclass_features.len() * 8).min(20) as i32;
        score += (context_manager_features.len() * 5).min(15) as i32;
        score += (fstring_features.len() * 3).min(10) as i32;
        score += (pattern_matching_features.len() * 10).min(20) as i32;
        score += (generator_features.len() * 4).min(12) as i32;
        score += (decorator_features.len() * 2).min(10) as i32;
        score += (modern_syntax_features.len() * 3).min(13) as i32;

        score.min(100)
    }

    /// Generate modern feature recommendations
    #[allow(clippy::too_many_arguments)] // Complex analysis requires multiple feature sets
    fn get_modern_feature_recommendations(
        &self,
        dataclass_features: &[DataclassInfo],
        context_manager_features: &[ContextManagerInfo],
        fstring_features: &[FStringInfo],
        _pattern_matching_features: &[PatternMatchingInfo],
        generator_features: &[GeneratorInfo],
        _decorator_features: &[ModernDecoratorInfo],
        modern_syntax_features: &[ModernSyntaxInfo],
        python_version: &PythonVersionDetected,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if dataclass_features.is_empty() {
            recommendations.push(
                "Consider using @dataclass for data classes instead of manual __init__".to_string(),
            );
        }

        if context_manager_features.is_empty() {
            recommendations
                .push("Use context managers (with statements) for resource management".to_string());
        }

        if fstring_features.is_empty() {
            recommendations.push(
                "Consider using f-strings for string formatting instead of .format()".to_string(),
            );
        }

        if generator_features.is_empty() {
            recommendations
                .push("Consider using generators for memory-efficient iteration".to_string());
        }

        // Version-specific recommendations
        if self.is_version_supported("3.10", &python_version.minimum_version) {
            recommendations.push(
                "Consider using pattern matching (match/case) for complex conditionals".to_string(),
            );
        }

        if self.is_version_supported("3.8", &python_version.minimum_version) {
            let has_walrus = modern_syntax_features
                .iter()
                .any(|f| matches!(f.feature_type, ModernSyntaxType::WalrusOperator));
            if !has_walrus {
                recommendations.push(
                    "Consider using walrus operator (:=) for assignment expressions".to_string(),
                );
            }
        }

        recommendations
    }

    /// Analyze context usage pattern
    fn analyze_context_usage_pattern(
        &self,
        content: &str,
        _context_match: &str,
    ) -> ContextUsagePattern {
        if content.contains("async with") {
            ContextUsagePattern::AsyncContext
        } else if content.matches("with").count() > 1 {
            ContextUsagePattern::MultipleContexts
        } else {
            ContextUsagePattern::SingleContext
        }
    }

    /// Assess resource management quality
    fn assess_resource_management_quality(
        &self,
        content: &str,
        _context_match: &str,
    ) -> ResourceManagementQuality {
        let has_error_handling = content.contains("try") || content.contains("except");
        let has_finally = content.contains("finally");
        let has_proper_cleanup = content.contains("close()") || content.contains("__exit__");

        match (has_error_handling, has_finally, has_proper_cleanup) {
            (true, true, true) => ResourceManagementQuality::Excellent,
            (true, _, true) => ResourceManagementQuality::Good,
            (true, _, _) => ResourceManagementQuality::Adequate,
            (false, _, true) => ResourceManagementQuality::Poor,
            _ => ResourceManagementQuality::Dangerous,
        }
    }

    /// Assess context error handling
    fn assess_context_error_handling(
        &self,
        content: &str,
        _context_match: &str,
    ) -> ContextErrorHandling {
        if content.contains("except") && content.contains("finally") {
            ContextErrorHandling::Comprehensive
        } else if content.contains("except") {
            ContextErrorHandling::Basic
        } else if content.contains("try") {
            ContextErrorHandling::Minimal
        } else {
            ContextErrorHandling::None
        }
    }

    /// Calculate context nesting level
    fn calculate_context_nesting_level(&self, content: &str, _context_match: &str) -> usize {
        // Count actual with statement patterns
        let with_pattern = Regex::new(r"\bwith\s+[^:]+:").unwrap();
        let with_count = with_pattern.find_iter(content).count();
        if with_count > 3 {
            3 // Cap at 3 for simplicity
        } else {
            with_count
        }
    }

    /// Check context best practices
    fn check_context_best_practices(&self, content: &str, _context_match: &str) -> bool {
        // Check for good practices
        let has_appropriate_error_handling = content.contains("except");
        let not_overly_nested = self.calculate_context_nesting_level(content, _context_match) <= 2;
        let has_resource_cleanup = content.contains("close") || content.contains("__exit__");

        has_appropriate_error_handling && not_overly_nested && has_resource_cleanup
    }

    /// Helper methods for f-string analysis
    fn assess_fstring_complexity(&self, expression: &str) -> FStringComplexity {
        // Count actual f-string expression braces
        let brace_pattern = Regex::new(r"\{[^}]+\}").unwrap();
        let brace_count = brace_pattern.find_iter(expression).count();
        let has_function_calls = expression.contains('(');
        let has_format_spec = expression.contains(':');

        match (brace_count, has_function_calls, has_format_spec) {
            (1, false, false) => FStringComplexity::Simple,
            (1..=2, _, true) | (1..=2, true, _) => FStringComplexity::Moderate,
            (3..=5, _, _) => FStringComplexity::Complex,
            _ => FStringComplexity::Advanced,
        }
    }

    fn detect_fstring_features(&self, expression: &str) -> Vec<FStringFeature> {
        let mut features = Vec::new();

        if expression.contains('{') && expression.contains('}') {
            features.push(FStringFeature::BasicInterpolation);
        }
        if expression.contains('(') {
            features.push(FStringFeature::ExpressionEvaluation);
        }
        if expression.contains(':') {
            features.push(FStringFeature::FormatSpecifiers);
        }
        if expression.contains('!') {
            features.push(FStringFeature::ConversionFlags);
        }
        if expression.starts_with("rf") || expression.starts_with("fr") {
            features.push(FStringFeature::RawFString);
        }

        features
    }

    fn assess_fstring_performance(
        &self,
        _expression: &str,
        features: &[FStringFeature],
    ) -> PerformanceImpact {
        if features.contains(&FStringFeature::ComplexExpressions) {
            PerformanceImpact::Negative
        } else if features.contains(&FStringFeature::ExpressionEvaluation) {
            PerformanceImpact::Neutral
        } else {
            PerformanceImpact::Positive
        }
    }

    fn assess_formatting_quality(&self, expression: &str) -> FormattingQuality {
        let length = expression.len();
        let complexity = self.assess_fstring_complexity(expression);

        match (length, complexity) {
            (0..=50, FStringComplexity::Simple) => FormattingQuality::Excellent,
            (0..=100, FStringComplexity::Moderate) => FormattingQuality::Good,
            (0..=150, FStringComplexity::Complex) => FormattingQuality::Adequate,
            (0..=200, _) => FormattingQuality::Poor,
            _ => FormattingQuality::Unreadable,
        }
    }

    fn calculate_fstring_readability(&self, expression: &str) -> i32 {
        let mut score = 100;

        if expression.len() > 100 {
            score -= 20;
        }
        if expression.matches('{').count() > 3 {
            score -= 15;
        }
        if expression.contains("()") {
            score -= 10;
        }

        score.max(0)
    }

    /// Version comparison helper
    fn is_newer_version(&self, version1: &str, version2: &str) -> bool {
        // Simple version comparison (would be more sophisticated in production)
        let v1 = version1.trim_end_matches('+');
        let v2 = version2.trim_end_matches('+');
        v1 > v2
    }

    fn is_version_supported(&self, required: &str, current: &str) -> bool {
        // Check if current version supports the required version
        self.is_newer_version(current, required) || current == required
    }

    /// Pattern matching helper methods
    fn analyze_match_patterns(&self, match_statement: &str) -> Vec<PatternType> {
        let mut patterns = Vec::new();

        if match_statement.contains("case _:") {
            patterns.push(PatternType::WildcardPattern);
        }
        if match_statement.contains("case ") && match_statement.contains("if ") {
            patterns.push(PatternType::GuardedPattern);
        }
        if match_statement.contains("case [") {
            patterns.push(PatternType::SequencePattern);
        }
        if match_statement.contains("case {") {
            patterns.push(PatternType::MappingPattern);
        }
        if match_statement.contains(" | ") {
            patterns.push(PatternType::OrPattern);
        }

        if patterns.is_empty() {
            patterns.push(PatternType::LiteralPattern);
        }

        patterns
    }

    fn assess_pattern_complexity(&self, patterns: &[PatternType]) -> PatternComplexity {
        let complexity_score = patterns.len()
            + patterns
                .iter()
                .map(|p| match p {
                    PatternType::GuardedPattern => 3,
                    PatternType::SequencePattern | PatternType::MappingPattern => 2,
                    PatternType::OrPattern => 2,
                    _ => 1,
                })
                .sum::<usize>();

        match complexity_score {
            0..=3 => PatternComplexity::Simple,
            4..=8 => PatternComplexity::Moderate,
            9..=15 => PatternComplexity::Complex,
            _ => PatternComplexity::Advanced,
        }
    }

    fn check_pattern_exhaustiveness(&self, match_statement: &str) -> bool {
        match_statement.contains("case _:") || match_statement.contains("case default:")
    }

    fn assess_match_performance(
        &self,
        patterns: &[PatternType],
        _match_statement: &str,
    ) -> MatchPerformance {
        let has_complex_patterns = patterns.iter().any(|p| {
            matches!(
                p,
                PatternType::GuardedPattern
                    | PatternType::SequencePattern
                    | PatternType::MappingPattern
            )
        });

        if has_complex_patterns && patterns.len() > 10 {
            MatchPerformance::Poor
        } else if has_complex_patterns {
            MatchPerformance::Fair
        } else if patterns.len() <= 5 {
            MatchPerformance::Optimal
        } else {
            MatchPerformance::Good
        }
    }

    fn score_pattern_best_practices(
        &self,
        patterns: &[PatternType],
        has_guards: bool,
        is_exhaustive: bool,
    ) -> i32 {
        let mut score = 70;

        if is_exhaustive {
            score += 20;
        }
        if has_guards {
            score += 10; // Guards can be good for complex logic
        }
        if patterns.len() <= 5 {
            score += 10; // Simple patterns are better
        }

        score.min(100)
    }

    /// Generator helper methods
    fn classify_generator_usage_pattern(
        &self,
        content: &str,
        _generator_match: &str,
    ) -> GeneratorUsagePattern {
        if content.contains("yield from") {
            GeneratorUsagePattern::Pipeline
        } else if content.contains("while True") {
            GeneratorUsagePattern::InfiniteSequence
        } else if content.contains("map") || content.contains("filter") {
            GeneratorUsagePattern::DataTransformation
        } else {
            GeneratorUsagePattern::SimpleIteration
        }
    }

    fn assess_generator_memory_efficiency(
        &self,
        content: &str,
        _generator_match: &str,
    ) -> MemoryEfficiency {
        let has_large_data_structures =
            content.contains("list(") || content.contains("[") && content.len() > 100;
        let uses_yield_properly = content.contains("yield") && !content.contains("return [");

        match (uses_yield_properly, has_large_data_structures) {
            (true, false) => MemoryEfficiency::Excellent,
            (true, true) => MemoryEfficiency::Good,
            (false, false) => MemoryEfficiency::Adequate,
            (false, true) => MemoryEfficiency::Poor,
        }
    }

    fn assess_generator_complexity(
        &self,
        content: &str,
        _generator_match: &str,
    ) -> GeneratorComplexity {
        // Count actual yield statements with proper pattern matching
        let yield_pattern = Regex::new(r"\byield\s+").unwrap();
        let yield_count = yield_pattern.find_iter(content).count();
        let has_complex_logic = content.contains("if") && content.contains("for");
        // Count actual for loops
        let for_pattern = Regex::new(r"\bfor\s+\w+\s+in\s+").unwrap();
        let has_nested_loops = for_pattern.find_iter(content).count() > 1;

        match (yield_count, has_complex_logic, has_nested_loops) {
            (1, false, false) => GeneratorComplexity::Simple,
            (1..=3, _, false) => GeneratorComplexity::Moderate,
            (_, true, true) => GeneratorComplexity::Advanced,
            _ => GeneratorComplexity::Complex,
        }
    }

    fn analyze_yield_usage(&self, content: &str, _generator_match: &str) -> YieldAnalysis {
        // Count actual yield statements
        let yield_pattern = Regex::new(r"\byield\s+").unwrap();
        YieldAnalysis {
            yield_count: yield_pattern.find_iter(content).count(),
            has_yield_from: content.contains("yield from"),
            has_send_values: content.contains(".send("),
            has_throw_values: content.contains(".throw("),
            has_close_handling: content.contains(".close("),
        }
    }

    fn identify_generator_optimizations(
        &self,
        content: &str,
        _generator_match: &str,
    ) -> Vec<String> {
        let mut optimizations = Vec::new();

        if content.contains("list(") && content.contains("yield") {
            optimizations.push("Avoid converting generator to list unless necessary".to_string());
        }

        if content.matches("for").count() > 2 {
            optimizations
                .push("Consider breaking complex generator into smaller functions".to_string());
        }

        if !content.contains("yield from") && content.contains("for") && content.contains("yield") {
            optimizations
                .push("Consider using 'yield from' for delegating to sub-generators".to_string());
        }

        optimizations
    }

    /// Decorator helper methods
    fn classify_decorator_category(&self, decorator_name: &str) -> DecoratorCategory {
        match decorator_name {
            "property" | "staticmethod" | "classmethod" => DecoratorCategory::BuiltIn,
            "wraps" | "singledispatch" | "cache" | "lru_cache" => {
                DecoratorCategory::FunctoolsDecorator
            }
            "app.route" | "api.route" | "route" => DecoratorCategory::WebFramework,
            "pytest.fixture" | "pytest.mark" | "unittest.mock" => DecoratorCategory::Testing,
            "dataclass" | "validator" => DecoratorCategory::DataValidation,
            _ => DecoratorCategory::Custom,
        }
    }

    fn analyze_decorator_usage_pattern(
        &self,
        content: &str,
        decorator: &str,
    ) -> DecoratorUsagePattern {
        let decorator_context = self.get_decorator_context(content, decorator);

        if decorator_context.contains('@') && decorator_context.matches('@').count() > 1 {
            DecoratorUsagePattern::StackedDecorators
        } else if decorator.contains('(') {
            DecoratorUsagePattern::ParameterizedDecorator
        } else {
            DecoratorUsagePattern::SingleDecorator
        }
    }

    fn get_decorator_context(&self, content: &str, decorator: &str) -> String {
        // Get surrounding context for the decorator
        if let Some(pos) = content.find(decorator) {
            let start = pos.saturating_sub(100);
            let end = (pos + decorator.len() + 100).min(content.len());
            content[start..end].to_string()
        } else {
            decorator.to_string()
        }
    }

    fn assess_decorator_complexity(&self, decorator: &str) -> DecoratorComplexity {
        if decorator.contains('(') && decorator.contains(',') {
            DecoratorComplexity::Complex
        } else if decorator.contains('(') {
            DecoratorComplexity::Moderate
        } else {
            DecoratorComplexity::Simple
        }
    }

    fn score_decorator_best_practices(
        &self,
        category: &DecoratorCategory,
        usage_pattern: &DecoratorUsagePattern,
    ) -> i32 {
        let mut score = 80;

        match category {
            DecoratorCategory::BuiltIn => score += 10,
            DecoratorCategory::FunctoolsDecorator => score += 15,
            DecoratorCategory::Performance => score += 10,
            _ => {}
        }

        match usage_pattern {
            DecoratorUsagePattern::SingleDecorator => score += 5,
            DecoratorUsagePattern::StackedDecorators => score -= 5, // Can be complex
            _ => {}
        }

        score.min(100)
    }

    /// Modern syntax helper methods
    fn assess_syntax_complexity(&self, content: &str, pattern: &Regex) -> SyntaxComplexity {
        let usage_count = pattern.find_iter(content).count();
        let total_lines = content.lines().count();
        let usage_density = if total_lines > 0 {
            usage_count as f32 / total_lines as f32
        } else {
            0.0
        };

        match usage_density {
            d if d > 0.1 => SyntaxComplexity::Expert,
            d if d > 0.05 => SyntaxComplexity::Complex,
            d if d > 0.01 => SyntaxComplexity::Moderate,
            _ => SyntaxComplexity::Simple,
        }
    }

    fn check_syntax_best_practices(&self, content: &str, feature_type: &ModernSyntaxType) -> bool {
        match feature_type {
            ModernSyntaxType::WalrusOperator => {
                // Check if walrus operator is used appropriately (not overused)
                content.matches(":=").count() <= content.lines().count() / 10
            }
            ModernSyntaxType::PositionalOnlyParams => {
                // Check if positional-only parameters are used with good reason
                content.contains("def ") && content.contains("/")
            }
            ModernSyntaxType::TypeUnionOperator => {
                // Check if union operator is used instead of typing.Union
                !content.contains("typing.Union") || content.contains(" | ")
            }
            _ => true, // Default to good practices for other features
        }
    }

    fn get_syntax_migration_suggestions(
        &self,
        feature_type: &ModernSyntaxType,
        usage_count: usize,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match feature_type {
            ModernSyntaxType::WalrusOperator => {
                if usage_count > 10 {
                    suggestions.push(
                        "Consider if all walrus operator usages improve readability".to_string(),
                    );
                }
                suggestions.push(
                    "Use walrus operator to reduce code duplication in conditions".to_string(),
                );
            }
            ModernSyntaxType::GenericTypeHints => {
                suggestions.push(
                    "Migrate from typing.List/Dict to built-in list/dict for type hints"
                        .to_string(),
                );
            }
            ModernSyntaxType::TypeUnionOperator => {
                suggestions.push(
                    "Replace typing.Union with | operator for cleaner type hints".to_string(),
                );
            }
            _ => {}
        }

        suggestions
    }

    /// Analyze async functions in detail
    fn analyze_async_functions(&self, content: &str, async_functions: &mut Vec<AsyncFunctionInfo>) {
        for patterns in self.async_patterns.values() {
            for pattern in patterns {
                if pattern.pattern_type == "function"
                    || pattern.pattern_type == "generator"
                    || pattern.pattern_type == "context_manager"
                    || pattern.pattern_type == "iterator"
                {
                    for captures in pattern.pattern.captures_iter(content) {
                        let full_match = captures.get(0).unwrap().as_str();
                        let function_name = captures
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "anonymous".to_string());

                        let function_type =
                            self.determine_async_function_type(&pattern.name, full_match);
                        let complexity = self.assess_async_complexity(content, full_match);
                        let coroutine_type = self.classify_coroutine_type(content);
                        let error_handling = self.assess_error_handling(content, full_match);
                        let has_timeout = self.has_timeout_handling(content, full_match);
                        let uses_context_manager =
                            self.uses_async_context_manager(content, full_match);

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
    fn analyze_concurrency_patterns(
        &self,
        content: &str,
        concurrency_patterns: &mut Vec<ConcurrencyPatternInfo>,
    ) {
        for patterns in self.async_patterns.values() {
            for pattern in patterns {
                if pattern.pattern_type == "concurrency" {
                    for captures in pattern.pattern.captures_iter(content) {
                        let full_match = captures.get(0).unwrap().as_str();

                        let pattern_type = self.map_to_concurrency_pattern_type(&pattern.name);
                        let usage_quality =
                            self.assess_concurrency_usage_quality(content, full_match);
                        let best_practices_followed =
                            self.check_concurrency_best_practices(content, full_match);

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
    fn detect_async_performance_issues(
        &self,
        content: &str,
        issues: &mut Vec<AsyncPerformanceIssue>,
    ) {
        // Detect blocking IO in async functions - look for blocking calls within async function bodies
        if content.contains("async def")
            && (content.contains("time.sleep")
                || content.contains("open(")
                || content.contains("input("))
        {
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
                recommendation:
                    "Use asyncio.gather() or asyncio.as_completed() for concurrent execution"
                        .to_string(),
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
                recommendation: "Consider using asyncio.gather() for independent operations"
                    .to_string(),
                estimated_impact: AsyncPerformanceImpact::Negative,
            });
        }
    }

    /// Detect async security issues
    fn detect_async_security_issues(&self, content: &str, issues: &mut Vec<AsyncSecurityIssue>) {
        // Detect missing timeouts
        let await_pattern = Regex::new(r"await\s+").unwrap();
        // Count actual timeout function calls with proper pattern matching
        let wait_for_pattern = Regex::new(r"\basyncio\.wait_for\s*\(").unwrap();
        let timeout_pattern = Regex::new(r"\basyncio\.timeout\s*\(").unwrap();
        let timeout_count = wait_for_pattern.find_iter(content).count()
            + timeout_pattern.find_iter(content).count();
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
        let shared_state_pattern =
            Regex::new(r"(?:global|class)\s+\w+.*=.*\n.*async\s+def.*\w+.*=").unwrap();
        for captures in shared_state_pattern.captures_iter(content) {
            let full_match = captures.get(0).unwrap().as_str();
            if !content.contains("asyncio.Lock") && !content.contains("asyncio.Semaphore") {
                issues.push(AsyncSecurityIssue {
                    issue_type: AsyncSecurityIssueType::SharedStateNoLock,
                    severity: AsyncSecuritySeverity::High,
                    location: full_match.to_string(),
                    description: "Shared mutable state without proper locking".to_string(),
                    recommendation: "Use asyncio.Lock or asyncio.Semaphore for thread safety"
                        .to_string(),
                });
            }
        }

        // Detect race condition patterns
        if content.contains("asyncio.gather")
            && !content.contains("asyncio.Lock")
            && content.matches("=").count() > 3
        {
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
            (
                "async with",
                ModernAsyncFeatureType::AsyncContextManager,
                "3.7+",
            ),
            (
                "asyncio.TaskGroup",
                ModernAsyncFeatureType::TaskGroups,
                "3.11+",
            ),
            (
                "asyncio.timeout",
                ModernAsyncFeatureType::AsyncioTimeout,
                "3.11+",
            ),
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
                    description: format!("Modern async feature: {pattern_str}"),
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
    fn determine_async_function_type(
        &self,
        pattern_name: &str,
        _full_match: &str,
    ) -> AsyncFunctionType {
        match pattern_name {
            "Async Function" => AsyncFunctionType::RegularAsync,
            "Async Generator" => AsyncFunctionType::AsyncGenerator,
            "Async Context Manager" => AsyncFunctionType::AsyncContextManager,
            "Async Iterator" => AsyncFunctionType::AsyncIterator,
            _ => AsyncFunctionType::RegularAsync,
        }
    }

    fn assess_async_complexity(&self, _content: &str, function_match: &str) -> AsyncComplexity {
        // Count actual async patterns with proper parsing
        let await_pattern = Regex::new(r"\bawait\s+").unwrap();
        let try_pattern = Regex::new(r"\btry\s*:").unwrap();
        let gather_pattern = Regex::new(r"\basyncio\.gather\s*\(").unwrap();

        let await_count = await_pattern.find_iter(function_match).count();
        let try_count = try_pattern.find_iter(function_match).count();
        let gather_count = gather_pattern.find_iter(function_match).count();

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
        if function_match.contains("asyncio.timeout") || function_match.contains("asyncio.wait_for")
        {
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
        !matches!(
            context,
            AwaitContext::SyncContext | AwaitContext::Comprehension | AwaitContext::Lambda
        )
    }

    fn detect_await_issues(
        &self,
        content: &str,
        await_expr: &str,
        context: &AwaitContext,
    ) -> Vec<AwaitIssue> {
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

    fn assess_concurrency_usage_quality(
        &self,
        content: &str,
        pattern_match: &str,
    ) -> ConcurrencyUsageQuality {
        let has_error_handling = pattern_match.contains("try") || pattern_match.contains("except");
        let has_timeout = pattern_match.contains("timeout") || content.contains("asyncio.wait_for");
        let has_proper_cleanup =
            pattern_match.contains("finally") || pattern_match.contains("async with");

        match (has_error_handling, has_timeout, has_proper_cleanup) {
            (true, true, true) => ConcurrencyUsageQuality::Excellent,
            (true, true, false) | (true, false, true) => ConcurrencyUsageQuality::Good,
            (true, false, false) | (false, true, false) | (false, true, true) => {
                ConcurrencyUsageQuality::Adequate
            }
            (false, false, true) => ConcurrencyUsageQuality::Poor,
            (false, false, false) => ConcurrencyUsageQuality::Dangerous,
        }
    }

    fn check_concurrency_best_practices(&self, content: &str, _pattern_match: &str) -> bool {
        content.contains("async with")
            && (content.contains("timeout") || content.contains("asyncio.wait_for"))
            && content.contains("try")
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
        let concurrency_bonus = concurrency_patterns
            .iter()
            .map(|p| match p.usage_quality {
                ConcurrencyUsageQuality::Excellent => 10,
                ConcurrencyUsageQuality::Good => 7,
                ConcurrencyUsageQuality::Adequate => 4,
                ConcurrencyUsageQuality::Poor => 1,
                ConcurrencyUsageQuality::Dangerous => -5,
            })
            .sum::<i32>();

        // Penalty for issues
        let performance_penalty = performance_issues
            .iter()
            .map(|i| match i.severity {
                AsyncIssueSeverity::Critical => 20,
                AsyncIssueSeverity::High => 15,
                AsyncIssueSeverity::Medium => 10,
                AsyncIssueSeverity::Low => 5,
                AsyncIssueSeverity::Info => 1,
            })
            .sum::<i32>();

        let security_penalty = security_issues
            .iter()
            .map(|i| match i.severity {
                AsyncSecuritySeverity::Critical => 25,
                AsyncSecuritySeverity::High => 20,
                AsyncSecuritySeverity::Medium => 15,
                AsyncSecuritySeverity::Low => 10,
                AsyncSecuritySeverity::Info => 5,
            })
            .sum::<i32>();

        (base_score + async_bonus + concurrency_bonus - performance_penalty - security_penalty)
            .clamp(0, 100)
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
            recommendations
                .push("Address async performance issues for better efficiency".to_string());
        }

        if !security_issues.is_empty() {
            recommendations.push("Fix async security vulnerabilities".to_string());
        }

        let has_poor_concurrency = concurrency_patterns.iter().any(|p| {
            matches!(
                p.usage_quality,
                ConcurrencyUsageQuality::Poor | ConcurrencyUsageQuality::Dangerous
            )
        });

        if has_poor_concurrency {
            recommendations
                .push("Improve concurrency pattern usage with proper error handling".to_string());
        }

        let has_invalid_await = await_patterns.iter().any(|p| !p.is_valid);
        if has_invalid_await {
            recommendations.push("Fix invalid await usage in sync contexts".to_string());
        }

        let missing_timeouts = async_functions.iter().any(|f| !f.has_timeout);
        if missing_timeouts {
            recommendations.push("Add timeout handling to prevent hanging operations".to_string());
        }

        recommendations
            .push("Use asyncio.gather() for concurrent independent operations".to_string());
        recommendations
            .push("Implement proper async context managers for resource cleanup".to_string());
        recommendations
            .push("Consider using Python 3.11+ TaskGroup for structured concurrency".to_string());

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
                    params_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
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
            "protocol" => TypeHintType::ProtocolType("Protocol".to_string()),
            "literal" => {
                let values_str = captures.get(1).unwrap().as_str();
                let literal_values = values_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
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
                    fields: Vec::new(), // Could use more parsing for fields
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
        // Count actual Any type usage with proper pattern
        let any_pattern = Regex::new(r"\bAny\b").unwrap();
        let any_count = any_pattern.find_iter(content).count();
        if any_count > 5 {
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::AnyTypeOveruse,
                severity: TypeSafetySeverity::Warning,
                location: "Multiple locations".to_string(),
                description: format!("Found {any_count} uses of Any type"),
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
                description: format!("{missing_hints} functions missing return type hints"),
                recommendation: "Add return type annotations to functions".to_string(),
            });
        }

        // Detect type: ignore overuse
        // Count actual type ignore comments with proper pattern
        let ignore_pattern = Regex::new(r"#\s*type:\s*ignore").unwrap();
        let ignore_count = ignore_pattern.find_iter(content).count();
        if ignore_count > 3 {
            issues.push(TypeSafetyIssue {
                issue_type: TypeSafetyIssueType::TypeIgnoreOveruse,
                severity: TypeSafetySeverity::Info,
                location: "Multiple locations".to_string(),
                description: format!("Found {ignore_count} type: ignore comments"),
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
        let typed_func_pattern =
            Regex::new(r"def\s+\w+\s*\([^)]*:\s*\w+|def\s+\w+\s*\([^)]*\)\s*->\s*\w+").unwrap();
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

        let has_modern_features = type_hints.iter().any(|h| {
            h.python_version_required.starts_with("3.8")
                || h.python_version_required.starts_with("3.9")
                || h.python_version_required.starts_with("3.10")
        });

        if !has_modern_features {
            recommendations.push("Consider using modern Python type features (3.8+)".to_string());
        }

        let has_complex_types = type_hints.iter().any(|h| {
            matches!(
                h.complexity,
                TypeComplexity::Complex | TypeComplexity::Advanced
            )
        });

        if has_complex_types {
            recommendations
                .push("Document complex type relationships for maintainability".to_string());
        }

        if type_hints.iter().any(|h| h.is_generic) {
            recommendations
                .push("Ensure generic type constraints are properly defined".to_string());
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

        assert!(!decorators.is_empty(), "Should detect decorator in code");

        // Validate specific decorator detection
        let flask_decorator = decorators
            .iter()
            .find(|d| d.name == "Flask Route")
            .expect("Should detect Flask route decorator");

        assert!(
            !flask_decorator.parameters.is_empty(),
            "Flask route should have path parameter"
        );
        assert!(
            flask_decorator.parameters.contains(&"'/test'".to_string()),
            "Should capture the route path"
        );
    }

    #[test]
    fn test_metaclass_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = "class TestClass(BaseClass, metaclass=RegistryMeta):\n    pass";
        let metaclasses = analyzer.analyze_metaclasses(code).unwrap();

        assert!(!metaclasses.is_empty(), "Should detect metaclass usage");

        // Validate specific metaclass detection
        let test_metaclass = metaclasses
            .iter()
            .find(|m| m.name == "TestClass")
            .expect("Should detect TestClass with metaclass");

        // The metaclass analysis categorizes this as "common" pattern, not the literal type name
        assert_eq!(
            test_metaclass.metaclass_type, "common",
            "Should categorize metaclass pattern correctly"
        );
        assert!(
            !test_metaclass.impact.is_empty(),
            "Should analyze metaclass impact"
        );
    }

    #[test]
    fn test_inheritance_analysis() {
        let analyzer = PythonAnalyzer::new();

        let code = "class Child(Parent1, Parent2):\n    pass";
        let inheritance = analyzer.analyze_inheritance(code).unwrap();

        assert!(!inheritance.is_empty(), "Should not be empty");
        assert_eq!(inheritance[0].class_name, "Child");
        assert_eq!(inheritance[0].base_classes.len(), 2, "Should have 2 items");
    }

    #[test]
    fn test_decorator_parameter_extraction() {
        let analyzer = PythonAnalyzer::new();

        let decorator = "@app.route('/test', methods=['GET', 'POST'])";
        let params = analyzer.extract_decorator_parameters(decorator);

        assert!(!params.is_empty(), "Should not be empty");
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

        assert_eq!(mixins.len(), 2, "Should have 2 items");
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

        assert!(
            !result.type_hints_detected.is_empty(),
            "Should not be empty"
        );
        assert!(result
            .type_hints_detected
            .iter()
            .any(|h| matches!(h.hint_type, TypeHintType::GenericType(_))));
        assert!(result
            .type_hints_detected
            .iter()
            .any(|h| matches!(h.hint_type, TypeHintType::UnionType(_))));
        assert!(result
            .type_hints_detected
            .iter()
            .any(|h| matches!(h.hint_type, TypeHintType::OptionalType(_))));
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

        assert!(
            !result.modern_type_features.is_empty(),
            "Should not be empty"
        );
        assert!(result
            .modern_type_features
            .iter()
            .any(|f| matches!(f.feature_type, ModernTypeFeatureType::TypedDict)));
        assert!(result
            .modern_type_features
            .iter()
            .any(|f| matches!(f.feature_type, ModernTypeFeatureType::FinalType)));
        assert!(result
            .modern_type_features
            .iter()
            .any(|f| matches!(f.feature_type, ModernTypeFeatureType::LiteralType)));
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

        assert!(!result.type_safety_issues.is_empty(), "Should not be empty");
        assert!(result
            .type_safety_issues
            .iter()
            .any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::AnyTypeOveruse)));
        assert!(result
            .type_safety_issues
            .iter()
            .any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::MissingTypeHints)));
        assert!(result
            .type_safety_issues
            .iter()
            .any(|issue| matches!(issue.issue_type, TypeSafetyIssueType::TypeIgnoreOveruse)));
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
        assert!(matches!(
            result.type_coverage_score,
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
        assert!(
            !result.async_functions_detected.is_empty(),
            "Should not be empty"
        );
        assert!(result.async_functions_detected.len() >= 3);

        // Should detect concurrency patterns
        assert!(
            !result.concurrency_patterns.is_empty(),
            "Should not be empty"
        );

        // Should detect modern async features
        assert!(
            !result.modern_async_features.is_empty(),
            "Should not be empty"
        );

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
        assert!(
            !result.async_performance_issues.is_empty(),
            "Should not be empty"
        );

        // Should detect blocking operations
        let blocking_issues: Vec<_> = result
            .async_performance_issues
            .iter()
            .filter(|issue| {
                matches!(
                    issue.issue_type,
                    AsyncPerformanceIssueType::BlockingIOInAsync
                )
            })
            .collect();
        assert!(!blocking_issues.is_empty(), "Should not be empty");

        // Should detect missing concurrency
        let concurrency_issues: Vec<_> = result
            .async_performance_issues
            .iter()
            .filter(|issue| {
                matches!(
                    issue.issue_type,
                    AsyncPerformanceIssueType::MissingConcurrency
                )
            })
            .collect();
        assert!(!concurrency_issues.is_empty(), "Should not be empty");
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
        assert!(
            !result.async_security_issues.is_empty(),
            "Should not be empty"
        );

        // Should detect timeout vulnerability
        let timeout_issues: Vec<_> = result
            .async_security_issues
            .iter()
            .filter(|issue| matches!(issue.issue_type, AsyncSecurityIssueType::AsyncTimeoutVuln))
            .collect();
        assert!(!timeout_issues.is_empty(), "Should not be empty");
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
        assert!(
            !result.modern_async_features.is_empty(),
            "Should not be empty"
        );

        // Should detect async context managers
        let context_manager_features: Vec<_> = result
            .modern_async_features
            .iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::AsyncContextManager))
            .collect();
        assert!(!context_manager_features.is_empty(), "Should not be empty");

        // Should detect TaskGroups
        let task_group_features: Vec<_> = result
            .modern_async_features
            .iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::TaskGroups))
            .collect();
        assert!(!task_group_features.is_empty(), "Should not be empty");

        // Should detect asyncio.run
        let asyncio_run_features: Vec<_> = result
            .modern_async_features
            .iter()
            .filter(|f| matches!(f.feature_type, ModernAsyncFeatureType::AsyncioRun))
            .collect();
        assert!(!asyncio_run_features.is_empty(), "Should not be empty");

        // Should have recommendations
        assert!(!result.recommendations.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_package_dependency_analysis() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import requests
import pandas as pd
from flask import Flask
import numpy
from django.db import models

django>=3.2.0
requests==2.28.1
pandas>=1.5.0
numpy
flask==2.0.1
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should detect dependencies
        assert!(!result.dependencies.is_empty(), "Should not be empty");

        // Should detect imports
        assert!(!result.import_analysis.is_empty(), "Should not be empty");
        assert!(result.import_analysis.len() >= 5);

        // Should have a reasonable health score
        assert!(result.overall_health_score >= 0);
        assert!(result.overall_health_score <= 100);

        // Should detect issues
        assert!(!result.dependency_issues.is_empty(), "Should not be empty");

        // Should provide recommendations
        assert!(!result.recommendations.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_dependency_issue_detection() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import requests
import missing_package
from some_package import *

# Requirements:
requests==2.28.1
unused_package==1.0.0
imp==1.0.0
django
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should detect unused dependency
        let unused_issues: Vec<_> = result
            .dependency_issues
            .iter()
            .filter(|issue| matches!(issue.issue_type, DependencyIssueType::UnusedDependency))
            .collect();
        assert!(!unused_issues.is_empty(), "Should not be empty");

        // Should detect missing dependency
        let missing_issues: Vec<_> = result
            .dependency_issues
            .iter()
            .filter(|issue| matches!(issue.issue_type, DependencyIssueType::MissingDependency))
            .collect();
        assert!(!missing_issues.is_empty(), "Should not be empty");

        // Should detect unpinned version
        let unpinned_issues: Vec<_> = result
            .dependency_issues
            .iter()
            .filter(|issue| matches!(issue.issue_type, DependencyIssueType::UnpinnedVersion))
            .collect();
        assert!(!unpinned_issues.is_empty(), "Should not be empty");

        // Should detect deprecated package
        let deprecated_issues: Vec<_> = result
            .dependency_issues
            .iter()
            .filter(|issue| matches!(issue.issue_type, DependencyIssueType::DeprecatedPackage))
            .collect();
        assert!(!deprecated_issues.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_import_analysis() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import os
import sys
import requests
import pandas as pd
from flask import Flask, render_template
from mymodule import function
from .relative import local_function
from package import *
import numpy as np
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should categorize imports correctly
        let stdlib_imports: Vec<_> = result
            .import_analysis
            .iter()
            .filter(|imp| matches!(imp.module_category, ModuleCategory::StandardLibrary))
            .collect();
        assert!(stdlib_imports.len() >= 2); // os, sys

        let third_party_imports: Vec<_> = result
            .import_analysis
            .iter()
            .filter(|imp| matches!(imp.module_category, ModuleCategory::ThirdParty))
            .collect();
        assert!(third_party_imports.len() >= 3); // requests, pandas, flask, numpy

        // Should detect star import issues
        let star_import_issues: Vec<_> = result
            .import_analysis
            .iter()
            .filter(|imp| {
                imp.import_issues
                    .contains(&ImportIssue::StarImportDangerous)
            })
            .collect();
        assert!(!star_import_issues.is_empty(), "Should not be empty");

        // Should detect different import types
        let from_imports: Vec<_> = result
            .import_analysis
            .iter()
            .filter(|imp| matches!(imp.import_type, ImportType::FromImport))
            .collect();
        assert!(!from_imports.is_empty(), "Should not be empty");

        let alias_imports: Vec<_> = result
            .import_analysis
            .iter()
            .filter(|imp| matches!(imp.import_type, ImportType::AliasImport))
            .collect();
        assert!(!alias_imports.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_security_vulnerability_scanning() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import urllib3
import requests
import pyyaml

urllib3==1.25.8
requests==2.19.1
pyyaml==5.3.1
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should detect security vulnerabilities
        assert!(
            !result.security_vulnerabilities.is_empty(),
            "Should not be empty"
        );

        // Should have vulnerabilities for known packages
        let urllib3_vulns: Vec<_> = result
            .security_vulnerabilities
            .iter()
            .filter(|vuln| vuln.package_name == "urllib3")
            .collect();
        assert!(!urllib3_vulns.is_empty(), "Should not be empty");

        // Should categorize severity correctly
        let critical_vulns: Vec<_> = result
            .security_vulnerabilities
            .iter()
            .filter(|vuln| matches!(vuln.severity, SecurityVulnerabilitySeverity::Critical))
            .collect();
        assert!(!critical_vulns.is_empty(), "Should not be empty");

        // Should have CVE information
        let vulns_with_cve: Vec<_> = result
            .security_vulnerabilities
            .iter()
            .filter(|vuln| vuln.cve_id.is_some())
            .collect();
        assert!(!vulns_with_cve.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_virtual_environment_detection() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
# Virtual environment indicators
python -m venv myenv
source myenv/bin/activate
pip install -r requirements.txt

# Conda environment
conda create -n myproject python=3.9
conda activate myproject

# Pipenv
pipenv install requests
pipenv shell
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should detect virtual environments
        assert!(
            !result.virtual_environments.is_empty(),
            "Should not be empty"
        );

        // Should detect different environment types
        let venv_envs: Vec<_> = result
            .virtual_environments
            .iter()
            .filter(|env| matches!(env.env_type, VirtualEnvironmentType::Venv))
            .collect();
        assert!(!venv_envs.is_empty(), "Should not be empty");

        let conda_envs: Vec<_> = result
            .virtual_environments
            .iter()
            .filter(|env| matches!(env.env_type, VirtualEnvironmentType::Conda))
            .collect();
        assert!(!conda_envs.is_empty(), "Should not be empty");

        let pipenv_envs: Vec<_> = result
            .virtual_environments
            .iter()
            .filter(|env| matches!(env.env_type, VirtualEnvironmentType::Pipenv))
            .collect();
        assert!(!pipenv_envs.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_license_analysis() {
        let analyzer = PythonAnalyzer::new();
        let content = r#"
import requests
import django
import flask

requests==2.28.1
django==4.1.0
flask==2.0.1
"#;

        let result = analyzer.analyze_package_dependencies(content).unwrap();

        // Should analyze licenses
        assert!(!result.license_analysis.is_empty(), "Should not be empty");

        // Should have license information for dependencies
        assert_eq!(result.license_analysis.len(), result.dependencies.len());

        // Should assess compatibility
        let compatible_licenses: Vec<_> = result
            .license_analysis
            .iter()
            .filter(|license| matches!(license.compatibility, LicenseCompatibility::Compatible))
            .collect();
        assert!(!compatible_licenses.is_empty(), "Should not be empty");

        // Should have license metadata
        for license in &result.license_analysis {
            assert!(!license.package_name.is_empty(), "Should not be empty");
            assert!(matches!(
                license.license_type,
                LicenseType::MIT
                    | LicenseType::Apache2
                    | LicenseType::BSD2Clause
                    | LicenseType::BSD3Clause
                    | LicenseType::Unknown
            ));
        }
    }

    #[test]
    fn test_modern_features_analysis() {
        let analyzer = PythonAnalyzer::new();
        let code = r#"
            from dataclasses import dataclass
            from typing import Optional
            import asyncio

            @dataclass
            class User:
                name: str
                age: int = 0

            async def process_data():
                async with asyncio.timeout(10):
                    data = [x for x in range(100)]
                    processed = (item * 2 for item in data)
                    return f"Processed {len(data)} items"

            def modern_syntax(value: str | int) -> str:
                if (result := calculate(value)) > 10:
                    return f"Result: {result:.2f}"
                return "Too small"

            match value:
                case int() if value > 100:
                    print("Large number")
                case str():
                    print("String value")
                case _:
                    print("Other")
        "#;

        let analysis = analyzer.analyze_modern_features(code).unwrap();

        // Should detect dataclass usage
        assert!(
            !analysis.dataclass_features.is_empty(),
            "Should not be empty"
        );

        // Should detect f-string usage
        assert!(!analysis.fstring_features.is_empty(), "Should not be empty");

        // Should detect modern syntax (walrus operator, union types)
        assert!(
            !analysis.modern_syntax_features.is_empty(),
            "Should not be empty"
        );

        // Should have a reasonable modernity score
        assert!(analysis.overall_modernity_score > 50);
        assert!(analysis.overall_modernity_score <= 100);

        // Should detect appropriate Python version
        assert!(analysis
            .python_version_detected
            .minimum_version
            .starts_with("3."));

        // Should provide recommendations
        assert!(!analysis.recommendations.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_dataclass_feature_detection() {
        let analyzer = PythonAnalyzer::new();
        let code = r#"
            from dataclasses import dataclass, field
            from typing import List

            @dataclass(frozen=True)
            class ImmutableUser:
                name: str
                tags: List[str] = field(default_factory=list)
                
                def __post_init__(self):
                    object.__setattr__(self, 'processed', True)

            class SlottedClass:
                __slots__ = ['x', 'y']
                
                def __init__(self, x, y):
                    self.x = x
                    self.y = y
        "#;

        let analysis = analyzer.analyze_modern_features(code).unwrap();

        assert!(
            !analysis.dataclass_features.is_empty(),
            "Should not be empty"
        );
        let dataclass_info = &analysis.dataclass_features[0];

        assert_eq!(
            dataclass_info.dataclass_type,
            DataclassType::StandardDataclass
        );
        assert!(dataclass_info
            .features_used
            .contains(&DataclassFeature::FrozenClass));
        assert!(dataclass_info
            .features_used
            .contains(&DataclassFeature::PostInitProcessing));
        assert!(dataclass_info
            .features_used
            .contains(&DataclassFeature::FieldFactories));
        assert!(dataclass_info.best_practices_score > 70);
    }

    #[test]
    fn test_fstring_complexity_analysis() {
        let analyzer = PythonAnalyzer::new();
        let code = r#"
            name = "Alice"
            value = 42.5
            
            # Simple f-string
            simple = f"Hello {name}"
            
            # Complex f-string with formatting
            complex = f"Value: {value:.2f}, Length: {len(name)}"
            
            # Very complex f-string
            advanced = f"Result: {calculate_result(value) if value > 0 else 'N/A':.3f}"
        "#;

        let analysis = analyzer.analyze_modern_features(code).unwrap();

        assert!(!analysis.fstring_features.is_empty(), "Should not be empty");
        assert!(analysis.fstring_features.len() >= 3);

        // Should detect different complexity levels
        let complexities: Vec<_> = analysis
            .fstring_features
            .iter()
            .map(|f| &f.complexity)
            .collect();

        assert!(complexities.contains(&&FStringComplexity::Simple));
        assert!(
            complexities.contains(&&FStringComplexity::Moderate)
                || complexities.contains(&&FStringComplexity::Complex)
        );
    }

    #[test]
    fn test_modern_syntax_features() {
        let analyzer = PythonAnalyzer::new();
        let code = r#"
            # Walrus operator (Python 3.8+)
            if (length := len(data)) > 10:
                print(f"Long data: {length}")

            # Union type syntax (Python 3.10+)
            def process(value: str | int | None) -> str | None:
                return str(value) if value is not None else None

            # Positional-only parameters (Python 3.8+)
            def func(a, b, /, c, d):
                return a + b + c + d

            # Modern type hints (Python 3.9+)
            data: list[dict[str, int]] = []
            mapping: dict[str, set[int]] = {}
        "#;

        let analysis = analyzer.analyze_modern_features(code).unwrap();

        assert!(
            !analysis.modern_syntax_features.is_empty(),
            "Should not be empty"
        );

        let syntax_types: Vec<_> = analysis
            .modern_syntax_features
            .iter()
            .map(|s| &s.feature_type)
            .collect();

        assert!(syntax_types.contains(&&ModernSyntaxType::WalrusOperator));
        assert!(syntax_types.contains(&&ModernSyntaxType::TypeUnionOperator));
        assert!(syntax_types.contains(&&ModernSyntaxType::PositionalOnlyParams));
        assert!(syntax_types.contains(&&ModernSyntaxType::GenericTypeHints));

        // Should detect Python 3.10+ as minimum version due to union operator
        assert!(analysis.python_version_detected.minimum_version >= "3.10".to_string());
    }
}
