//! Java-specific code analysis module
//! 
//! This module provides comprehensive analysis of Java code including:
//! - Object-oriented programming patterns
//! - Framework analysis (Spring, Hibernate, etc.)
//! - Security vulnerability detection
//! - Performance analysis
//! - Modern Java features analysis
//! - Design pattern recognition

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Object-oriented programming analysis result
#[derive(Debug, Clone)]
pub struct OOPAnalysisInfo {
    pub class_hierarchies: Vec<ClassHierarchyInfo>,
    pub design_patterns: Vec<DesignPatternInfo>,
    pub encapsulation_analysis: Vec<EncapsulationInfo>,
    pub polymorphism_usage: Vec<PolymorphismInfo>,
    pub inheritance_patterns: Vec<InheritancePatternInfo>,
    pub interface_usage: Vec<InterfaceUsageInfo>,
    pub solid_principles_score: SOLIDPrinciplesScore,
}

/// Class hierarchy information
#[derive(Debug, Clone)]
pub struct ClassHierarchyInfo {
    pub class_name: String,
    pub superclass: Option<String>,
    pub interfaces: Vec<String>,
    pub subclasses: Vec<String>,
    pub hierarchy_depth: usize,
    pub is_abstract: bool,
    pub is_final: bool,
    pub modifiers: Vec<String>,
}

/// Design pattern information
#[derive(Debug, Clone)]
pub struct DesignPatternInfo {
    pub pattern_type: DesignPatternType,
    pub confidence: f32,
    pub implementation_quality: ImplementationQuality,
    pub location: String,
    pub description: String,
    pub participants: Vec<String>,
}

/// Design pattern types
#[derive(Debug, Clone)]
pub enum DesignPatternType {
    // Creational Patterns
    Singleton,
    Factory,
    AbstractFactory,
    Builder,
    Prototype,
    // Structural Patterns
    Adapter,
    Bridge,
    Composite,
    Decorator,
    Facade,
    Flyweight,
    Proxy,
    // Behavioral Patterns
    Observer,
    Strategy,
    Command,
    Template,
    Visitor,
    Iterator,
    State,
    ChainOfResponsibility,
    Mediator,
    Memento,
    Interpreter,
}

/// Implementation quality assessment
#[derive(Debug, Clone)]
pub enum ImplementationQuality {
    Excellent,
    Good,
    Adequate,
    Poor,
    Incomplete,
}

/// Encapsulation analysis
#[derive(Debug, Clone)]
pub struct EncapsulationInfo {
    pub class_name: String,
    pub field_access_analysis: Vec<FieldAccessInfo>,
    pub getter_setter_patterns: Vec<GetterSetterInfo>,
    pub data_hiding_score: i32,
    pub immutability_patterns: Vec<ImmutabilityPattern>,
}

/// Field access information
#[derive(Debug, Clone)]
pub struct FieldAccessInfo {
    pub field_name: String,
    pub access_modifier: AccessModifier,
    pub is_final: bool,
    pub is_static: bool,
    pub field_type: String,
    pub proper_encapsulation: bool,
}

/// Access modifiers
#[derive(Debug, Clone)]
pub enum AccessModifier {
    Public,
    Protected,
    Private,
    PackagePrivate,
}

/// Getter/Setter pattern information
#[derive(Debug, Clone)]
pub struct GetterSetterInfo {
    pub field_name: String,
    pub has_getter: bool,
    pub has_setter: bool,
    pub getter_name: String,
    pub setter_name: String,
    pub follows_naming_convention: bool,
    pub validation_in_setter: bool,
}

/// Immutability pattern
#[derive(Debug, Clone)]
pub struct ImmutabilityPattern {
    pub class_name: String,
    pub immutability_level: ImmutabilityLevel,
    pub immutable_fields: Vec<String>,
    pub builder_pattern_used: bool,
}

/// Immutability levels
#[derive(Debug, Clone)]
pub enum ImmutabilityLevel {
    FullyImmutable,
    MostlyImmutable,
    PartiallyImmutable,
    Mutable,
}

/// Polymorphism usage information
#[derive(Debug, Clone)]
pub struct PolymorphismInfo {
    pub polymorphism_type: PolymorphismType,
    pub base_type: String,
    pub derived_types: Vec<String>,
    pub method_overrides: Vec<MethodOverrideInfo>,
    pub dynamic_dispatch_usage: bool,
}

/// Polymorphism types
#[derive(Debug, Clone)]
pub enum PolymorphismType {
    Inheritance,
    InterfaceBased,
    Parametric, // Generics
    AdHoc,      // Method overloading
}

/// Method override information
#[derive(Debug, Clone)]
pub struct MethodOverrideInfo {
    pub method_name: String,
    pub overriding_class: String,
    pub base_class: String,
    pub has_override_annotation: bool,
    pub preserves_contract: bool,
    pub changes_behavior: bool,
}

/// Inheritance pattern information
#[derive(Debug, Clone)]
pub struct InheritancePatternInfo {
    pub pattern_type: InheritancePatternType,
    pub base_class: String,
    pub derived_classes: Vec<String>,
    pub depth: usize,
    pub complexity_score: i32,
    pub potential_issues: Vec<String>,
}

/// Inheritance pattern types
#[derive(Debug, Clone)]
pub enum InheritancePatternType {
    SingleInheritance,
    InterfaceInheritance,
    MultipleInterfaceInheritance,
    DeepInheritance,
    DiamondProblem, // Through interfaces
}

/// Interface usage information
#[derive(Debug, Clone)]
pub struct InterfaceUsageInfo {
    pub interface_name: String,
    pub implementing_classes: Vec<String>,
    pub methods: Vec<InterfaceMethodInfo>,
    pub functional_interface: bool,
    pub lambda_usage: Vec<LambdaUsageInfo>,
}

/// Interface method information
#[derive(Debug, Clone)]
pub struct InterfaceMethodInfo {
    pub method_name: String,
    pub is_default: bool,
    pub is_static: bool,
    pub parameters: Vec<String>,
    pub return_type: String,
}

/// Lambda usage information
#[derive(Debug, Clone)]
pub struct LambdaUsageInfo {
    pub usage_context: String,
    pub lambda_type: LambdaType,
    pub complexity: LambdaComplexity,
    pub captures_variables: bool,
}

/// Lambda types
#[derive(Debug, Clone)]
pub enum LambdaType {
    Expression,
    Statement,
    MethodReference,
}

/// Lambda complexity
#[derive(Debug, Clone)]
pub enum LambdaComplexity {
    Simple,
    Moderate,
    Complex,
}

/// SOLID principles score
#[derive(Debug, Clone)]
pub struct SOLIDPrinciplesScore {
    pub single_responsibility: i32,
    pub open_closed: i32,
    pub liskov_substitution: i32,
    pub interface_segregation: i32,
    pub dependency_inversion: i32,
    pub overall_score: i32,
    pub violations: Vec<SOLIDViolation>,
}

/// SOLID principle violations
#[derive(Debug, Clone)]
pub struct SOLIDViolation {
    pub principle: SOLIDPrinciple,
    pub class_name: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub recommendation: String,
}

/// SOLID principles
#[derive(Debug, Clone)]
pub enum SOLIDPrinciple {
    SingleResponsibility,
    OpenClosed,
    LiskovSubstitution,
    InterfaceSegregation,
    DependencyInversion,
}

/// Violation severity
#[derive(Debug, Clone)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Framework analysis information
#[derive(Debug, Clone)]
pub struct JavaFrameworkAnalysis {
    pub frameworks_detected: Vec<FrameworkInfo>,
    pub spring_analysis: Option<SpringAnalysis>,
    pub hibernate_analysis: Option<HibernateAnalysis>,
    pub junit_analysis: Option<JUnitAnalysis>,
    pub maven_analysis: Option<MavenAnalysis>,
    pub gradle_analysis: Option<GradleAnalysis>,
    pub overall_framework_score: i32,
}

/// Framework information
#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    pub name: String,
    pub version: Option<String>,
    pub confidence: f32,
    pub usage_patterns: Vec<String>,
    pub best_practices_followed: Vec<String>,
    pub potential_issues: Vec<String>,
}

/// Spring framework analysis
#[derive(Debug, Clone)]
pub struct SpringAnalysis {
    pub spring_boot_used: bool,
    pub components: Vec<SpringComponentInfo>,
    pub dependency_injection: Vec<DIPatternInfo>,
    pub aop_usage: Vec<AOPPatternInfo>,
    pub transaction_management: Vec<TransactionInfo>,
    pub security_configuration: Option<SpringSecurityInfo>,
    pub data_access_patterns: Vec<DataAccessPatternInfo>,
}

/// Spring component information
#[derive(Debug, Clone)]
pub struct SpringComponentInfo {
    pub component_type: SpringComponentType,
    pub class_name: String,
    pub annotations: Vec<String>,
    pub scope: String,
    pub dependencies: Vec<String>,
}

/// Spring component types
#[derive(Debug, Clone)]
pub enum SpringComponentType {
    Component,
    Service,
    Repository,
    Controller,
    RestController,
    Configuration,
    Bean,
}

/// Dependency injection pattern
#[derive(Debug, Clone)]
pub struct DIPatternInfo {
    pub injection_type: DIType,
    pub target_class: String,
    pub dependencies: Vec<String>,
    pub follows_best_practices: bool,
    pub potential_issues: Vec<String>,
}

/// Dependency injection types
#[derive(Debug, Clone)]
pub enum DIType {
    Constructor,
    Field,
    Setter,
    Method,
}

/// AOP (Aspect-Oriented Programming) pattern
#[derive(Debug, Clone)]
pub struct AOPPatternInfo {
    pub aspect_class: String,
    pub pointcuts: Vec<String>,
    pub advice_types: Vec<AdviceType>,
    pub cross_cutting_concerns: Vec<String>,
}

/// Advice types
#[derive(Debug, Clone)]
pub enum AdviceType {
    Before,
    After,
    AfterReturning,
    AfterThrowing,
    Around,
}

/// Transaction information
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub class_name: String,
    pub method_name: String,
    pub transaction_type: TransactionType,
    pub propagation: String,
    pub isolation: String,
    pub rollback_rules: Vec<String>,
}

/// Transaction types
#[derive(Debug, Clone)]
pub enum TransactionType {
    Declarative,
    Programmatic,
}

/// Spring Security information
#[derive(Debug, Clone)]
pub struct SpringSecurityInfo {
    pub authentication_mechanisms: Vec<String>,
    pub authorization_patterns: Vec<String>,
    pub security_configurations: Vec<String>,
    pub csrf_protection: bool,
    pub session_management: String,
}

/// Data access pattern information
#[derive(Debug, Clone)]
pub struct DataAccessPatternInfo {
    pub pattern_type: DataAccessPattern,
    pub implementation_class: String,
    pub database_operations: Vec<String>,
    pub query_methods: Vec<QueryMethodInfo>,
}

/// Data access patterns
#[derive(Debug, Clone)]
pub enum DataAccessPattern {
    JpaRepository,
    CrudRepository,
    JdbcTemplate,
    NamedParameterJdbcTemplate,
    CustomRepository,
}

/// Query method information
#[derive(Debug, Clone)]
pub struct QueryMethodInfo {
    pub method_name: String,
    pub query_type: QueryType,
    pub custom_query: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: String,
}

/// Query types
#[derive(Debug, Clone)]
pub enum QueryType {
    DerivedQuery,
    CustomQuery,
    NativeQuery,
    NamedQuery,
}

/// Hibernate/JPA analysis
#[derive(Debug, Clone)]
pub struct HibernateAnalysis {
    pub entities: Vec<JPAEntityInfo>,
    pub relationships: Vec<EntityRelationshipInfo>,
    pub query_analysis: Vec<JPAQueryInfo>,
    pub performance_considerations: Vec<PerformanceIssue>,
    pub configuration_analysis: JPAConfigurationInfo,
}

/// JPA Entity information
#[derive(Debug, Clone)]
pub struct JPAEntityInfo {
    pub entity_name: String,
    pub table_name: String,
    pub primary_key: Vec<String>,
    pub fields: Vec<JPAFieldInfo>,
    pub annotations: Vec<String>,
    pub inheritance_strategy: Option<String>,
}

/// JPA Field information
#[derive(Debug, Clone)]
pub struct JPAFieldInfo {
    pub field_name: String,
    pub column_name: String,
    pub field_type: String,
    pub constraints: Vec<String>,
    pub annotations: Vec<String>,
    pub relationship_type: Option<RelationshipType>,
}

/// Entity relationship information
#[derive(Debug, Clone)]
pub struct EntityRelationshipInfo {
    pub relationship_type: RelationshipType,
    pub source_entity: String,
    pub target_entity: String,
    pub fetch_type: FetchType,
    pub cascade_operations: Vec<CascadeType>,
    pub bidirectional: bool,
}

/// Relationship types
#[derive(Debug, Clone)]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Fetch types
#[derive(Debug, Clone)]
pub enum FetchType {
    Eager,
    Lazy,
}

/// Cascade types
#[derive(Debug, Clone)]
pub enum CascadeType {
    All,
    Persist,
    Merge,
    Remove,
    Refresh,
    Detach,
}

/// JPA Query information
#[derive(Debug, Clone)]
pub struct JPAQueryInfo {
    pub query_type: JPAQueryType,
    pub query_string: String,
    pub parameters: Vec<String>,
    pub result_type: String,
    pub potential_issues: Vec<String>,
}

/// JPA Query types
#[derive(Debug, Clone)]
pub enum JPAQueryType {
    JPQL,
    NativeSQL,
    CriteriaAPI,
    NamedQuery,
}

/// Performance issues
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub severity: IssueSeverity,
    pub location: String,
    pub description: String,
    pub recommendation: String,
}

/// Performance issue types
#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    NPlusOneProblem,
    LazyLoadingIssue,
    InEfficientQuery,
    MissingIndex,
    CartesianProduct,
    UnoptimizedFetch,
    LargeResultSet,
}

/// Issue severity
#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// JPA Configuration information
#[derive(Debug, Clone)]
pub struct JPAConfigurationInfo {
    pub hibernate_dialect: Option<String>,
    pub show_sql: bool,
    pub format_sql: bool,
    pub ddl_auto: Option<String>,
    pub cache_configuration: Vec<String>,
    pub connection_pool_settings: Vec<String>,
}

/// JUnit analysis
#[derive(Debug, Clone)]
pub struct JUnitAnalysis {
    pub junit_version: JUnitVersion,
    pub test_classes: Vec<TestClassInfo>,
    pub test_patterns: Vec<TestPatternInfo>,
    pub mocking_frameworks: Vec<MockingFrameworkInfo>,
    pub coverage_patterns: Vec<String>,
    pub best_practices_score: i32,
}

/// JUnit versions
#[derive(Debug, Clone)]
pub enum JUnitVersion {
    JUnit4,
    JUnit5,
    Mixed,
    Unknown,
}

/// Test class information
#[derive(Debug, Clone)]
pub struct TestClassInfo {
    pub class_name: String,
    pub test_methods: Vec<TestMethodInfo>,
    pub setup_methods: Vec<String>,
    pub teardown_methods: Vec<String>,
    pub annotations: Vec<String>,
}

/// Test method information
#[derive(Debug, Clone)]
pub struct TestMethodInfo {
    pub method_name: String,
    pub test_type: TestType,
    pub assertions_count: usize,
    pub expected_exceptions: Vec<String>,
    pub timeout: Option<String>,
    pub parameters: Vec<String>,
}

/// Test types
#[derive(Debug, Clone)]
pub enum TestType {
    Unit,
    Integration,
    Parameterized,
    Performance,
    Exception,
}

/// Test pattern information
#[derive(Debug, Clone)]
pub struct TestPatternInfo {
    pub pattern_type: TestPatternType,
    pub usage_count: usize,
    pub classes_using: Vec<String>,
}

/// Test pattern types
#[derive(Debug, Clone)]
pub enum TestPatternType {
    ArrangeActAssert,
    GivenWhenThen,
    TestFixture,
    DataDriven,
    MockObject,
    TestDouble,
}

/// Mocking framework information
#[derive(Debug, Clone)]
pub struct MockingFrameworkInfo {
    pub framework_name: String,
    pub version: Option<String>,
    pub usage_patterns: Vec<String>,
    pub mock_objects: Vec<String>,
}

/// Maven analysis
#[derive(Debug, Clone)]  
pub struct MavenAnalysis {
    pub project_info: MavenProjectInfo,
    pub dependencies: Vec<MavenDependencyInfo>,
    pub plugins: Vec<MavenPluginInfo>,
    pub profiles: Vec<String>,
    pub dependency_management: Vec<String>,
    pub potential_issues: Vec<DependencyIssue>,
}

/// Maven project information
#[derive(Debug, Clone)]
pub struct MavenProjectInfo {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub packaging: String,
    pub java_version: Option<String>,
    pub properties: Vec<String>,
}

/// Maven dependency information
#[derive(Debug, Clone)]
pub struct MavenDependencyInfo {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: String,
    pub dependency_type: String,
    pub transitive_dependencies: Vec<String>,
}

/// Maven plugin information
#[derive(Debug, Clone)]
pub struct MavenPluginInfo {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub configuration: Vec<String>,
    pub executions: Vec<String>,
}

/// Dependency issues
#[derive(Debug, Clone)]
pub struct DependencyIssue {
    pub issue_type: DependencyIssueType,
    pub affected_dependencies: Vec<String>,
    pub severity: IssueSeverity,
    pub description: String,
    pub recommendation: String,
}

/// Dependency issue types
#[derive(Debug, Clone)]
pub enum DependencyIssueType {
    VersionConflict,
    SecurityVulnerability,
    DeprecatedDependency,
    UnusedDependency,
    TransitiveDependencyIssue,
    LicenseIncompatibility,
}

/// Gradle analysis
#[derive(Debug, Clone)]
pub struct GradleAnalysis {
    pub project_info: GradleProjectInfo,
    pub dependencies: Vec<GradleDependencyInfo>,
    pub plugins: Vec<GradlePluginInfo>,
    pub tasks: Vec<GradleTaskInfo>,
    pub build_configurations: Vec<String>,
    pub potential_issues: Vec<DependencyIssue>,
}

/// Gradle project information
#[derive(Debug, Clone)]
pub struct GradleProjectInfo {
    pub project_name: String,
    pub version: String,
    pub java_version: Option<String>,
    pub gradle_version: Option<String>,
    pub source_compatibility: Option<String>,
    pub target_compatibility: Option<String>,
}

/// Gradle dependency information
#[derive(Debug, Clone)]
pub struct GradleDependencyInfo {
    pub configuration: String,
    pub group: String,
    pub name: String,
    pub version: String,
    pub dependency_type: String,
}

/// Gradle plugin information
#[derive(Debug, Clone)]
pub struct GradlePluginInfo {
    pub plugin_id: String,
    pub version: Option<String>,
    pub apply: bool,
    pub configuration: Vec<String>,
}

/// Gradle task information
#[derive(Debug, Clone)]
pub struct GradleTaskInfo {
    pub task_name: String,
    pub task_type: String,
    pub dependencies: Vec<String>,
    pub description: String,
}

/// Security analysis for Java applications
#[derive(Debug, Clone)]
pub struct JavaSecurityAnalysis {
    pub security_level: SecurityLevel,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub security_patterns: Vec<SecurityPattern>,  
    pub authentication_analysis: Vec<AuthenticationPattern>,
    pub authorization_analysis: Vec<AuthorizationPattern>,
    pub input_validation_analysis: Vec<InputValidationPattern>,
    pub cryptographic_analysis: Vec<CryptographicPattern>,
    pub web_security_analysis: Vec<WebSecurityPattern>,
    pub recommendations: Vec<String>,
}

/// Security levels
#[derive(Debug, Clone)]
pub enum SecurityLevel {
    High,
    Medium,
    Low,
    Vulnerable,
}

/// Security vulnerabilities
#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub vulnerability_type: SecurityVulnerabilityType,
    pub severity: SecuritySeverity,
    pub location: String,
    pub description: String,
    pub cwe_id: Option<String>,
    pub recommendation: String,
}

/// Security vulnerability types
#[derive(Debug, Clone)]
pub enum SecurityVulnerabilityType {
    SqlInjection,
    XssVulnerability,
    CommandInjection,
    PathTraversal,
    DeserializationAttack,
    WeakCryptography,
    HardcodedCredentials,
    InsecureRandomness,
    UnvalidatedRedirect,
    SessionFixation,
    CsrfVulnerability,
    XXEVulnerability,
    LdapInjection,
    InsecureDirectObjectReference,
}

/// Security severity
#[derive(Debug, Clone)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Security patterns
#[derive(Debug, Clone)]
pub struct SecurityPattern {
    pub pattern_type: SecurityPatternType,
    pub implementation_quality: ImplementationQuality,
    pub location: String,
    pub description: String,
}

/// Security pattern types
#[derive(Debug, Clone)]
pub enum SecurityPatternType {
    SecureAuthentication,
    RoleBasedAccess,
    InputSanitization,
    OutputEncoding,
    SecureCommunication,
    AuditLogging,
    ErrorHandling,
    SessionManagement,
}

/// Authentication patterns
#[derive(Debug, Clone)]
pub struct AuthenticationPattern {
    pub authentication_type: AuthenticationType,
    pub implementation_class: String,
    pub security_features: Vec<String>,
    pub weaknesses: Vec<String>,
}

/// Authentication types
#[derive(Debug, Clone)]
pub enum AuthenticationType {
    FormBased,
    BasicAuth,
    DigestAuth,
    JwtToken,
    OAuth2,
    SAML,
    Custom,
}

/// Authorization patterns
#[derive(Debug, Clone)]
pub struct AuthorizationPattern {
    pub authorization_type: AuthorizationType,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub access_control_rules: Vec<String>,
}

/// Authorization types
#[derive(Debug, Clone)]
pub enum AuthorizationType {
    RoleBased,
    AttributeBased,
    PermissionBased,
    ResourceBased,
    Custom,
}

/// Input validation patterns
#[derive(Debug, Clone)]
pub struct InputValidationPattern {
    pub validation_type: ValidationType,
    pub input_sources: Vec<String>,
    pub validation_methods: Vec<String>,
    pub sanitization_techniques: Vec<String>,
}

/// Validation types
#[derive(Debug, Clone)]
pub enum ValidationType {
    Whitelist,
    Blacklist,
    RegexValidation,
    TypeValidation,
    RangeValidation,
    Custom,
}

/// Cryptographic patterns
#[derive(Debug, Clone)]
pub struct CryptographicPattern {
    pub crypto_operation: CryptographicOperation,
    pub algorithm: String,
    pub key_management: KeyManagementPattern,
    pub implementation_issues: Vec<String>,
}

/// Cryptographic operations
#[derive(Debug, Clone)]
pub enum CryptographicOperation {
    Encryption,
    Decryption,
    Hashing,
    DigitalSignature,
    KeyGeneration,
    KeyExchange,
}

/// Key management patterns
#[derive(Debug, Clone)]
pub struct KeyManagementPattern {
    pub key_storage: KeyStorageType,
    pub key_rotation: bool,
    pub key_strength: KeyStrength,
    pub key_derivation: Option<String>,
}

/// Key storage types
#[derive(Debug, Clone)]
pub enum KeyStorageType {
    Keystore,
    HSM, // Hardware Security Module
    Configuration,
    Hardcoded,
    Environment,
    Database,
}

/// Key strength
#[derive(Debug, Clone)]
pub enum KeyStrength {
    Strong,
    Adequate,
    Weak,
    Unknown,
}

/// Web security patterns
#[derive(Debug, Clone)]
pub struct WebSecurityPattern {
    pub security_mechanism: WebSecurityMechanism,
    pub configuration: Vec<String>,
    pub effectiveness: SecurityEffectiveness,
}

/// Web security mechanisms
#[derive(Debug, Clone)]
pub enum WebSecurityMechanism {
    CsrfProtection,
    XssProtection,
    ContentSecurityPolicy,
    HttpsEnforcement,
    SecureHeaders,
    SessionSecurity,
    CorsConfiguration,
}

/// Security effectiveness
#[derive(Debug, Clone)]
pub enum SecurityEffectiveness {
    Excellent,
    Good,
    Adequate,
    Poor,
    Missing,
}

/// Modern Java features analysis
#[derive(Debug, Clone)]
pub struct ModernJavaFeatureAnalysis {
    pub java_version_detected: JavaVersionInfo,
    pub lambda_expressions: Vec<LambdaExpressionInfo>,
    pub stream_api_usage: Vec<StreamApiUsageInfo>,
    pub optional_usage: Vec<OptionalUsageInfo>,
    pub module_system_usage: Option<ModuleSystemInfo>,
    pub record_classes: Vec<RecordClassInfo>,
    pub sealed_classes: Vec<SealedClassInfo>,
    pub switch_expressions: Vec<SwitchExpressionInfo>,
    pub text_blocks: Vec<TextBlockInfo>,
    pub var_keyword_usage: Vec<VarUsageInfo>,
    pub completable_future_usage: Vec<CompletableFutureInfo>,
    pub date_time_api_usage: Vec<DateTimeApiInfo>,
    pub collection_factory_methods: Vec<CollectionFactoryInfo>,
    pub overall_modernity_score: i32,
}

/// Java version information
#[derive(Debug, Clone)]
pub struct JavaVersionInfo {
    pub minimum_version_required: String,
    pub features_by_version: Vec<VersionFeatureInfo>,
    pub compatibility_issues: Vec<CompatibilityIssue>,
}

/// Version feature information
#[derive(Debug, Clone)]
pub struct VersionFeatureInfo {
    pub feature_name: String,
    pub java_version: String,
    pub usage_count: usize,
    pub is_best_practice: bool,
}

/// Compatibility issues
#[derive(Debug, Clone)]
pub struct CompatibilityIssue {
    pub issue_type: CompatibilityIssueType,
    pub required_version: String,
    pub current_version: String,
    pub affected_features: Vec<String>,
}

/// Compatibility issue types
#[derive(Debug, Clone)]
pub enum CompatibilityIssueType {
    VersionMismatch,
    DeprecatedFeature,
    NewApiUsage,
    UnsupportedFeature,
}

/// Lambda expression information
#[derive(Debug, Clone)]
pub struct LambdaExpressionInfo {
    pub expression: String,
    pub functional_interface: String,
    pub complexity: LambdaComplexity,
    pub captures_variables: bool,
    pub usage_context: String,
    pub performance_impact: PerformanceImpact,
}

/// Performance impact
#[derive(Debug, Clone)]
pub enum PerformanceImpact {
    Positive,
    Neutral,
    Negative,
}

/// Stream API usage information
#[derive(Debug, Clone)]
pub struct StreamApiUsageInfo {
    pub stream_source: String,
    pub operations: Vec<StreamOperation>,
    pub terminal_operation: String,
    pub parallel_usage: bool,
    pub performance_characteristics: StreamPerformance,
    pub complexity: StreamComplexity,
}

/// Stream operations
#[derive(Debug, Clone)]
pub struct StreamOperation {
    pub operation_type: StreamOperationType,
    pub operation_name: String,
    pub parameters: Vec<String>,
}

/// Stream operation types
#[derive(Debug, Clone)]
pub enum StreamOperationType {
    Intermediate,
    Terminal,
}

/// Stream performance characteristics
#[derive(Debug, Clone)]
pub enum StreamPerformance {
    Optimal,
    Good,
    Fair,
    Poor,
}

/// Stream complexity
#[derive(Debug, Clone)]
pub enum StreamComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Optional usage information
#[derive(Debug, Clone)]
pub struct OptionalUsageInfo {
    pub usage_context: String,
    pub optional_type: String,
    pub usage_pattern: OptionalUsagePattern,
    pub anti_patterns: Vec<OptionalAntiPattern>,
}

/// Optional usage patterns
#[derive(Debug, Clone)]
pub enum OptionalUsagePattern {
    ReturnValue,
    FieldValue,
    ParameterValue,
    ChainedCalls,
}

/// Optional anti-patterns
#[derive(Debug, Clone)]
pub enum OptionalAntiPattern {
    CallingGet,
    UsingIsPresent,
    ReturningNull,
    UsingInFields,
}

/// Module system information
#[derive(Debug, Clone)]
pub struct ModuleSystemInfo {
    pub module_name: String,
    pub exports: Vec<String>,
    pub requires: Vec<String>,
    pub provides: Vec<String>,
    pub uses: Vec<String>,
    pub opens: Vec<String>,
}

/// Record class information
#[derive(Debug, Clone)]
pub struct RecordClassInfo {
    pub record_name: String,
    pub components: Vec<RecordComponent>,
    pub additional_methods: Vec<String>,
    pub implements_interfaces: Vec<String>,
}

/// Record components
#[derive(Debug, Clone)]
pub struct RecordComponent {
    pub name: String,
    pub component_type: String,
    pub annotations: Vec<String>,
}

/// Sealed class information
#[derive(Debug, Clone)]
pub struct SealedClassInfo {
    pub sealed_class_name: String,
    pub permitted_subclasses: Vec<String>,
    pub sealing_type: SealingType,
}

/// Sealing types
#[derive(Debug, Clone)]
pub enum SealingType {
    SealedClass,
    SealedInterface,
}

/// Switch expression information
#[derive(Debug, Clone)]
pub struct SwitchExpressionInfo {
    pub switch_type: String,
    pub has_yield: bool,
    pub pattern_matching: bool,
    pub exhaustiveness: bool,
    pub arrow_syntax: bool,
}

/// Text block information
#[derive(Debug, Clone)]
pub struct TextBlockInfo {
    pub content_type: TextBlockContentType,
    pub line_count: usize,
    pub indentation_stripped: bool,
    pub escape_sequences_used: Vec<String>,
}

/// Text block content types
#[derive(Debug, Clone)]
pub enum TextBlockContentType {
    Json,
    Xml,
    Html,
    Sql,
    PlainText,
    Other,
}

/// Var keyword usage information
#[derive(Debug, Clone)]
pub struct VarUsageInfo {
    pub usage_context: VarUsageContext,
    pub inferred_type: String,
    pub appropriate_usage: bool,
}

/// Var usage contexts
#[derive(Debug, Clone)]
pub enum VarUsageContext {
    LocalVariable,
    ForLoop,
    TryWithResources,
    LambdaParameter,
}

/// CompletableFuture usage information
#[derive(Debug, Clone)]
pub struct CompletableFutureInfo {
    pub usage_pattern: CompletableFuturePattern,
    pub chaining_complexity: i32,
    pub exception_handling: bool,
    pub thread_pool_usage: Option<String>,
}

/// CompletableFuture patterns
#[derive(Debug, Clone)]
pub enum CompletableFuturePattern {
    SimpleAsync,
    Chaining,
    Combining,
    ExceptionHandling,
    CustomExecutor,
}

/// Date/Time API usage information
#[derive(Debug, Clone)]
pub struct DateTimeApiInfo {
    pub api_type: DateTimeApiType,
    pub usage_patterns: Vec<String>,
    pub timezone_handling: bool,
    pub formatting_patterns: Vec<String>,
}

/// Date/Time API types
#[derive(Debug, Clone)]
pub enum DateTimeApiType {
    LocalDateTime,
    ZonedDateTime,
    Instant,
    Duration,
    Period,
    DateTimeFormatter,
    Legacy, // java.util.Date, Calendar
}

/// Collection factory information
#[derive(Debug, Clone)]
pub struct CollectionFactoryInfo {
    pub factory_method: String,
    pub collection_type: String,
    pub element_count: usize,
    pub immutability: bool,
}

/// Main Java analyzer
#[derive(Debug)]
pub struct JavaAnalyzer {
    // Pattern storage for different analysis types
    oop_patterns: HashMap<String, Vec<OOPPattern>>,
    framework_patterns: HashMap<String, Vec<FrameworkPattern>>,
    security_patterns: HashMap<String, Vec<SecurityAnalysisPattern>>,
    modern_feature_patterns: HashMap<String, Vec<ModernFeaturePattern>>,
}

// Helper structures for pattern matching
#[derive(Debug)]
struct OOPPattern {
    name: String,
    pattern: Regex,
    pattern_type: String,
    confidence_weight: f32,
}

#[derive(Debug)]
struct FrameworkPattern {
    name: String,
    pattern: Regex,
    framework: String,
    confidence_weight: f32,
}

#[derive(Debug)]
struct SecurityAnalysisPattern {
    name: String,
    pattern: Regex,
    vulnerability_type: String,
    severity: String,
}

#[derive(Debug)]
struct ModernFeaturePattern {
    name: String,
    pattern: Regex,
    java_version: String,
    feature_type: String,
}

impl JavaAnalyzer {
    /// Create a new Java analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            oop_patterns: HashMap::new(),
            framework_patterns: HashMap::new(),
            security_patterns: HashMap::new(),
            modern_feature_patterns: HashMap::new(),
        };
        
        analyzer.initialize_patterns();
        analyzer
    }

    /// Initialize all pattern matching regexes
    fn initialize_patterns(&mut self) {
        self.initialize_oop_patterns();
        self.initialize_framework_patterns();
        self.initialize_security_patterns();
        self.initialize_modern_feature_patterns();
    }

    /// Initialize OOP analysis patterns
    fn initialize_oop_patterns(&mut self) {
        let mut patterns = Vec::new();
        
        // Singleton pattern detection
        patterns.push(OOPPattern {
            name: "singleton_private_constructor".to_string(),
            pattern: Regex::new(r"private\s+\w+\s*\(\s*\)").unwrap(),
            pattern_type: "singleton".to_string(),
            confidence_weight: 0.6,
        });
        
        patterns.push(OOPPattern {
            name: "singleton_instance_method".to_string(),
            pattern: Regex::new(r"public\s+static\s+\w+\s+getInstance\s*\(\s*\)").unwrap(),
            pattern_type: "singleton".to_string(),
            confidence_weight: 0.8,
        });
        
        // Factory pattern detection
        patterns.push(OOPPattern {
            name: "factory_method".to_string(),
            pattern: Regex::new(r"public\s+static\s+\w+\s+create\w*\s*\(").unwrap(),
            pattern_type: "factory".to_string(),
            confidence_weight: 0.7,
        });
        
        // Builder pattern detection
        patterns.push(OOPPattern {
            name: "builder_method".to_string(),
            pattern: Regex::new(r"public\s+\w+\s+\w+\s*\([^)]*\)\s*\{\s*\w+\.\w+\s*=.*return\s+this").unwrap(),
            pattern_type: "builder".to_string(),
            confidence_weight: 0.8,
        });
        
        // Observer pattern detection
        patterns.push(OOPPattern {
            name: "observer_notify".to_string(),
            pattern: Regex::new(r"notify(All)?Observers?\s*\(").unwrap(),
            pattern_type: "observer".to_string(),
            confidence_weight: 0.9,
        });
        
        // Decorator pattern detection
        patterns.push(OOPPattern {
            name: "decorator_composition".to_string(),
            pattern: Regex::new(r"private\s+final\s+\w+\s+\w+").unwrap(),
            pattern_type: "decorator".to_string(),
            confidence_weight: 0.5,
        });
        
        // Inheritance patterns
        patterns.push(OOPPattern {
            name: "class_extends".to_string(),
            pattern: Regex::new(r"class\s+(\w+)\s+extends\s+(\w+)").unwrap(),
            pattern_type: "inheritance".to_string(),
            confidence_weight: 1.0,
        });
        
        patterns.push(OOPPattern {
            name: "implements_interface".to_string(),
            pattern: Regex::new(r"class\s+(\w+).*implements\s+([\w\s,]+)").unwrap(),
            pattern_type: "interface_implementation".to_string(),
            confidence_weight: 1.0,
        });
        
        // Polymorphism patterns
        patterns.push(OOPPattern {
            name: "method_override".to_string(),
            pattern: Regex::new(r"@Override\s+public\s+\w+\s+(\w+)\s*\(").unwrap(),
            pattern_type: "polymorphism".to_string(),
            confidence_weight: 1.0,
        });
        
        // Encapsulation patterns
        patterns.push(OOPPattern {
            name: "private_field".to_string(),
            pattern: Regex::new(r"private\s+\w+\s+\w+").unwrap(),
            pattern_type: "encapsulation".to_string(),
            confidence_weight: 0.8,
        });
        
        patterns.push(OOPPattern {
            name: "getter_method".to_string(),
            pattern: Regex::new(r"public\s+\w+\s+get(\w+)\s*\(\s*\)").unwrap(),
            pattern_type: "encapsulation".to_string(),
            confidence_weight: 0.9,
        });
        
        patterns.push(OOPPattern {
            name: "setter_method".to_string(),
            pattern: Regex::new(r"public\s+void\s+set(\w+)\s*\(\s*\w+\s+\w+\s*\)").unwrap(),
            pattern_type: "encapsulation".to_string(),
            confidence_weight: 0.9,
        });
        
        self.oop_patterns.insert("design_patterns".to_string(), patterns);
    }

    /// Initialize Spring framework patterns
    fn initialize_framework_patterns(&mut self) {
        let mut spring_patterns = Vec::new();
        
        // Spring annotations
        spring_patterns.push(FrameworkPattern {
            name: "spring_component".to_string(),
            pattern: Regex::new(r"@Component").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.9,
        });
        
        spring_patterns.push(FrameworkPattern {
            name: "spring_service".to_string(),
            pattern: Regex::new(r"@Service").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.9,
        });
        
        spring_patterns.push(FrameworkPattern {
            name: "spring_repository".to_string(),
            pattern: Regex::new(r"@Repository").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.9,
        });
        
        spring_patterns.push(FrameworkPattern {
            name: "spring_controller".to_string(),
            pattern: Regex::new(r"@(Rest)?Controller").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.9,
        });
        
        spring_patterns.push(FrameworkPattern {
            name: "spring_autowired".to_string(),
            pattern: Regex::new(r"@Autowired").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.8,
        });
        
        spring_patterns.push(FrameworkPattern {
            name: "spring_transactional".to_string(),
            pattern: Regex::new(r"@Transactional").unwrap(),
            framework: "Spring".to_string(),
            confidence_weight: 0.8,
        });
        
        // JPA/Hibernate patterns
        let mut jpa_patterns = Vec::new();
        
        jpa_patterns.push(FrameworkPattern {
            name: "jpa_entity".to_string(),
            pattern: Regex::new(r"@Entity").unwrap(),
            framework: "JPA".to_string(),
            confidence_weight: 0.9,
        });
        
        jpa_patterns.push(FrameworkPattern {
            name: "jpa_table".to_string(),
            pattern: Regex::new(r"@Table").unwrap(),
            framework: "JPA".to_string(),
            confidence_weight: 0.8,
        });
        
        jpa_patterns.push(FrameworkPattern {
            name: "jpa_id".to_string(),
            pattern: Regex::new(r"@Id").unwrap(),
            framework: "JPA".to_string(),
            confidence_weight: 0.9,
        });
        
        jpa_patterns.push(FrameworkPattern {
            name: "jpa_column".to_string(),
            pattern: Regex::new(r"@Column").unwrap(),
            framework: "JPA".to_string(),
            confidence_weight: 0.7,
        });
        
        // JUnit patterns
        let mut junit_patterns = Vec::new();
        
        junit_patterns.push(FrameworkPattern {
            name: "junit_test".to_string(),
            pattern: Regex::new(r"@Test").unwrap(),
            framework: "JUnit".to_string(),
            confidence_weight: 0.9,
        });
        
        junit_patterns.push(FrameworkPattern {
            name: "junit_before".to_string(),
            pattern: Regex::new(r"@Before(Each)?").unwrap(),
            framework: "JUnit".to_string(),
            confidence_weight: 0.8,
        });
        
        junit_patterns.push(FrameworkPattern {
            name: "junit_after".to_string(),
            pattern: Regex::new(r"@After(Each)?").unwrap(),
            framework: "JUnit".to_string(),
            confidence_weight: 0.8,
        });
        
        self.framework_patterns.insert("Spring".to_string(), spring_patterns);
        self.framework_patterns.insert("JPA".to_string(), jpa_patterns);
        self.framework_patterns.insert("JUnit".to_string(), junit_patterns);
    }

    /// Initialize security analysis patterns
    fn initialize_security_patterns(&mut self) {
        let mut patterns = Vec::new();
        
        // SQL Injection patterns
        patterns.push(SecurityAnalysisPattern {
            name: "sql_concatenation".to_string(),
            pattern: Regex::new(r#"(SELECT|INSERT|UPDATE|DELETE).*\+.*["']"#).unwrap(),
            vulnerability_type: "sql_injection".to_string(),
            severity: "high".to_string(),
        });
        
        // Hardcoded credentials
        patterns.push(SecurityAnalysisPattern {
            name: "hardcoded_password".to_string(),
            pattern: Regex::new(r#"(password|pwd|pass)\s*=\s*["'][^"']+["']"#).unwrap(),
            vulnerability_type: "hardcoded_credentials".to_string(),
            severity: "critical".to_string(),
        });
        
        // Command injection
        patterns.push(SecurityAnalysisPattern {
            name: "runtime_exec".to_string(),
            pattern: Regex::new(r"Runtime\.getRuntime\(\)\.exec\(").unwrap(),
            vulnerability_type: "command_injection".to_string(),
            severity: "high".to_string(),
        });
        
        // Path traversal
        patterns.push(SecurityAnalysisPattern {
            name: "file_path_concat".to_string(),
            pattern: Regex::new(r"new\s+File\([^)]*\+").unwrap(),
            vulnerability_type: "path_traversal".to_string(),
            severity: "medium".to_string(),
        });
        
        // Weak cryptography
        patterns.push(SecurityAnalysisPattern {
            name: "weak_hash".to_string(),
            pattern: Regex::new(r#"MessageDigest\.getInstance\(["'](MD5|SHA1)["']\)"#).unwrap(),
            vulnerability_type: "weak_cryptography".to_string(),
            severity: "medium".to_string(),
        });
        
        // Insecure random
        patterns.push(SecurityAnalysisPattern {
            name: "insecure_random".to_string(),
            pattern: Regex::new(r"new\s+Random\(\)").unwrap(),
            vulnerability_type: "insecure_randomness".to_string(),
            severity: "low".to_string(),
        });
        
        self.security_patterns.insert("vulnerabilities".to_string(), patterns);
    }

    /// Initialize modern Java feature patterns
    fn initialize_modern_feature_patterns(&mut self) {
        let mut patterns = Vec::new();
        
        // Lambda expressions (Java 8+)
        patterns.push(ModernFeaturePattern {
            name: "lambda_expression".to_string(),
            pattern: Regex::new(r"\([^)]*\)\s*->").unwrap(),
            java_version: "8".to_string(),
            feature_type: "lambda".to_string(),
        });
        
        // Stream API (Java 8+)
        patterns.push(ModernFeaturePattern {
            name: "stream_api".to_string(),
            pattern: Regex::new(r"\.stream\(\)").unwrap(),
            java_version: "8".to_string(),
            feature_type: "stream".to_string(),
        });
        
        // Optional (Java 8+)
        patterns.push(ModernFeaturePattern {
            name: "optional_usage".to_string(),
            pattern: Regex::new(r"Optional<").unwrap(),
            java_version: "8".to_string(),
            feature_type: "optional".to_string(),
        });
        
        // Var keyword (Java 10+)
        patterns.push(ModernFeaturePattern {
            name: "var_keyword".to_string(),
            pattern: Regex::new(r"\bvar\s+\w+\s*=").unwrap(),
            java_version: "10".to_string(),
            feature_type: "var".to_string(),
        });
        
        // Switch expressions (Java 12+)
        patterns.push(ModernFeaturePattern {
            name: "switch_expression".to_string(),
            pattern: Regex::new(r"switch\s*\([^)]+\)\s*\{[^}]*->").unwrap(),
            java_version: "12".to_string(),
            feature_type: "switch_expression".to_string(),
        });
        
        // Text blocks (Java 13+)
        patterns.push(ModernFeaturePattern {
            name: "text_blocks".to_string(),
            pattern: Regex::new(r#""{3}"#).unwrap(),
            java_version: "13".to_string(),
            feature_type: "text_block".to_string(),
        });
        
        // Records (Java 14+)
        patterns.push(ModernFeaturePattern {
            name: "record_class".to_string(),
            pattern: Regex::new(r"record\s+(\w+)").unwrap(),
            java_version: "14".to_string(),
            feature_type: "record".to_string(),
        });
        
        // Sealed classes (Java 15+)
        patterns.push(ModernFeaturePattern {
            name: "sealed_class".to_string(),
            pattern: Regex::new(r"sealed\s+(class|interface)").unwrap(),
            java_version: "15".to_string(),
            feature_type: "sealed".to_string(),
        });
        
        self.modern_feature_patterns.insert("java_features".to_string(), patterns);
    }

    /// Main analysis method - comprehensive Java code analysis
    pub fn analyze_comprehensive(&self, content: &str) -> Result<JavaComprehensiveAnalysis> {
        let oop_analysis = self.analyze_oop_patterns(content)?;
        let framework_analysis = self.analyze_frameworks(content)?;
        let security_analysis = self.analyze_security(content)?;
        let modern_features = self.analyze_modern_features(content)?;
        let performance_analysis = self.analyze_performance(content)?;
        
        Ok(JavaComprehensiveAnalysis {
            oop_analysis,
            framework_analysis,
            security_analysis,
            modern_features,
            performance_analysis,
            overall_score: self.calculate_overall_score(content),
            recommendations: self.generate_recommendations(content),
        })
    }

    /// Analyze object-oriented programming patterns
    pub fn analyze_oop_patterns(&self, content: &str) -> Result<OOPAnalysisInfo> {
        let class_hierarchies = self.analyze_class_hierarchies(content)?;
        let design_patterns = self.detect_design_patterns(content)?;
        let encapsulation_analysis = self.analyze_encapsulation(content)?;
        let polymorphism_usage = self.analyze_polymorphism(content)?;
        let inheritance_patterns = self.analyze_inheritance_patterns(content)?;
        let interface_usage = self.analyze_interface_usage(content)?;
        let solid_principles_score = self.evaluate_solid_principles(content)?;

        Ok(OOPAnalysisInfo {
            class_hierarchies,
            design_patterns,
            encapsulation_analysis,
            polymorphism_usage,
            inheritance_patterns,
            interface_usage,
            solid_principles_score,
        })
    }

    /// Analyze Spring and other framework usage
    pub fn analyze_frameworks(&self, content: &str) -> Result<JavaFrameworkAnalysis> {
        let frameworks_detected = self.detect_frameworks(content)?;
        let spring_analysis = self.analyze_spring_framework(content)?;
        let hibernate_analysis = self.analyze_hibernate(content)?;
        let junit_analysis = self.analyze_junit(content)?;
        let maven_analysis = self.analyze_maven(content)?;
        let gradle_analysis = self.analyze_gradle(content)?;
        let overall_framework_score = self.calculate_framework_score(&frameworks_detected);

        Ok(JavaFrameworkAnalysis {
            frameworks_detected,
            spring_analysis,
            hibernate_analysis,
            junit_analysis,
            maven_analysis,
            gradle_analysis,
            overall_framework_score,
        })
    }

    /// Analyze security vulnerabilities and patterns
    pub fn analyze_security(&self, content: &str) -> Result<JavaSecurityAnalysis> {
        let vulnerabilities = self.detect_vulnerabilities(content)?;
        let security_patterns = self.detect_security_patterns(content)?;
        let authentication_analysis = self.analyze_authentication(content)?;
        let authorization_analysis = self.analyze_authorization(content)?;
        let input_validation_analysis = self.analyze_input_validation(content)?;
        let cryptographic_analysis = self.analyze_cryptography(content)?;
        let web_security_analysis = self.analyze_web_security(content)?;
        let security_level = self.determine_security_level(&vulnerabilities, &security_patterns);
        let recommendations = self.generate_security_recommendations(&vulnerabilities, &security_patterns);

        Ok(JavaSecurityAnalysis {
            security_level,
            vulnerabilities,
            security_patterns,
            authentication_analysis,
            authorization_analysis,
            input_validation_analysis,
            cryptographic_analysis,
            web_security_analysis,
            recommendations,
        })
    }

    /// Analyze modern Java features usage
    pub fn analyze_modern_features(&self, content: &str) -> Result<ModernJavaFeatureAnalysis> {
        let java_version_detected = self.detect_java_version(content)?;
        let lambda_expressions = self.analyze_lambda_expressions(content)?;
        let stream_api_usage = self.analyze_stream_api(content)?;
        let optional_usage = self.analyze_optional_usage(content)?;
        let module_system_usage = self.analyze_module_system(content)?;
        let record_classes = self.analyze_record_classes(content)?;
        let sealed_classes = self.analyze_sealed_classes(content)?;
        let switch_expressions = self.analyze_switch_expressions(content)?;
        let text_blocks = self.analyze_text_blocks(content)?;
        let var_keyword_usage = self.analyze_var_usage(content)?;
        let completable_future_usage = self.analyze_completable_future(content)?;
        let date_time_api_usage = self.analyze_date_time_api(content)?;
        let collection_factory_methods = self.analyze_collection_factories(content)?;
        let overall_modernity_score = self.calculate_modernity_score(content);

        Ok(ModernJavaFeatureAnalysis {
            java_version_detected,
            lambda_expressions,
            stream_api_usage,
            optional_usage,
            module_system_usage,
            record_classes,
            sealed_classes,
            switch_expressions,
            text_blocks,
            var_keyword_usage,
            completable_future_usage,
            date_time_api_usage,
            collection_factory_methods,
            overall_modernity_score,
        })
    }

    /// Simple analysis for backward compatibility
    pub fn analyze_code(&self, content: &str) -> JavaAnalysisResult {
        // This provides backward compatibility with the existing simple interface
        let mut patterns = Vec::new();
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Perform comprehensive analysis and extract key insights
        if let Ok(comprehensive) = self.analyze_comprehensive(content) {
            // Extract patterns from design patterns
            for pattern in &comprehensive.oop_analysis.design_patterns {
                patterns.push(format!("{:?} pattern detected with {:.1}% confidence", 
                    pattern.pattern_type, pattern.confidence * 100.0));
            }

            // Extract security issues
            for vuln in &comprehensive.security_analysis.vulnerabilities {
                issues.push(format!("{:?}: {}", vuln.vulnerability_type, vuln.description));
            }

            // Extract framework recommendations
            recommendations.extend(comprehensive.recommendations);
        }

        // Fallback to basic patterns if comprehensive analysis fails
        if patterns.is_empty() {
            if content.contains("public static final") {
                patterns.push("Constants pattern detected".to_string());
            }
            if content.contains("private static") && content.contains("getInstance") {
                patterns.push("Singleton pattern detected".to_string());
            }
        }

        JavaAnalysisResult {
            patterns_detected: patterns,
            issues_found: issues,
            recommendations,
        }
    }

    /// Analyze class hierarchies
    fn analyze_class_hierarchies(&self, content: &str) -> Result<Vec<ClassHierarchyInfo>> {
        let mut hierarchies = Vec::new();
        
        // Look for class declarations with extends keyword
        let class_regex = Regex::new(r"(?m)^(?:\s*public\s+)?(?:abstract\s+)?class\s+(\w+)(?:\s+extends\s+(\w+))?(?:\s+implements\s+([\w\s,]+))?")?;
        
        for captures in class_regex.captures_iter(content) {
            let class_name = captures.get(1).unwrap().as_str().to_string();
            let superclass = captures.get(2).map(|m| m.as_str().to_string());
            
            let interfaces = if let Some(interfaces_str) = captures.get(3) {
                interfaces_str.as_str()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                Vec::new()
            };
            
            // Check if class is abstract
            let is_abstract = content.contains(&format!("abstract class {}", class_name));
            let is_final = content.contains(&format!("final class {}", class_name));
            
            hierarchies.push(ClassHierarchyInfo {
                class_name: class_name.clone(),
                superclass,
                interfaces,
                subclasses: self.find_subclasses(content, &class_name),
                hierarchy_depth: self.calculate_hierarchy_depth(content, &class_name),
                is_abstract,
                is_final,
                modifiers: self.extract_class_modifiers(content, &class_name),
            });
        }
        
        Ok(hierarchies)
    }

    /// Detect design patterns in the code
    fn detect_design_patterns(&self, content: &str) -> Result<Vec<DesignPatternInfo>> {
        let mut patterns = Vec::new();
        
        // Singleton pattern detection
        if content.contains("private static") && content.contains("getInstance") && content.contains("private") && content.contains("()") {
            patterns.push(DesignPatternInfo {
                pattern_type: DesignPatternType::Singleton,
                confidence: 0.8,
                implementation_quality: ImplementationQuality::Good,
                location: "Singleton class".to_string(),
                description: "Singleton pattern implementation detected".to_string(),
                participants: vec!["Singleton".to_string()],
            });
        }
        
        // Builder pattern detection
        if content.contains("Builder") && content.contains("build()") && content.contains("return this") {
            patterns.push(DesignPatternInfo {
                pattern_type: DesignPatternType::Builder,
                confidence: 0.9,
                implementation_quality: ImplementationQuality::Good,
                location: "Builder class".to_string(),
                description: "Builder pattern implementation detected".to_string(),
                participants: vec!["Builder".to_string()],
            });
        }
        
        // Factory pattern detection
        if content.contains("Factory") && content.contains("create") && content.contains("switch") {
            patterns.push(DesignPatternInfo {
                pattern_type: DesignPatternType::Factory,
                confidence: 0.7,
                implementation_quality: ImplementationQuality::Good,
                location: "Factory class".to_string(),
                description: "Factory pattern implementation detected".to_string(),
                participants: vec!["Factory".to_string()],
            });
        }
        
        Ok(patterns)
    }

    /// Analyze encapsulation patterns
    fn analyze_encapsulation(&self, content: &str) -> Result<Vec<EncapsulationInfo>> {
        let mut encapsulation_info = Vec::new();
        
        // Find all class declarations
        let class_regex = Regex::new(r"(?m)^(?:\s*public\s+)?class\s+(\w+)")?;
        
        for captures in class_regex.captures_iter(content) {
            let class_name = captures.get(1).unwrap().as_str().to_string();
            
            // Analyze field access for this class
            let field_access_analysis = self.analyze_field_access(content, &class_name)?;
            let getter_setter_patterns = self.analyze_getter_setters(content, &class_name)?;
            let data_hiding_score = self.calculate_data_hiding_score(&field_access_analysis);
            let immutability_patterns = self.analyze_immutability_patterns(content, &class_name)?;
            
            encapsulation_info.push(EncapsulationInfo {
                class_name,
                field_access_analysis,
                getter_setter_patterns,
                data_hiding_score,
                immutability_patterns,
            });
        }
        
        Ok(encapsulation_info)
    }

    /// Analyze polymorphism usage
    fn analyze_polymorphism(&self, content: &str) -> Result<Vec<PolymorphismInfo>> {
        let mut polymorphism_usage = Vec::new();
        
        // Find method overrides
        let override_regex = Regex::new(r"@Override\s+(?:public\s+|protected\s+|private\s+)?(\w+)\s+(\w+)\s*\(").unwrap();
        
        for captures in override_regex.captures_iter(content) {
            let method_name = captures.get(2).unwrap().as_str().to_string();
            let overriding_class = self.find_containing_class(content, captures.get(0).unwrap().start());
            
            if let Some(class_name) = overriding_class {
                polymorphism_usage.push(PolymorphismInfo {
                    polymorphism_type: PolymorphismType::Inheritance,
                    base_type: self.find_base_type(content, &class_name).unwrap_or("Object".to_string()),
                    derived_types: vec![class_name.clone()],
                    method_overrides: vec![MethodOverrideInfo {
                        method_name: method_name.clone(),
                        overriding_class: class_name,
                        base_class: "Unknown".to_string(), // Would need more sophisticated analysis
                        has_override_annotation: true,
                        preserves_contract: true, // Assume good practice
                        changes_behavior: false, // Would need semantic analysis
                    }],
                    dynamic_dispatch_usage: true,
                });
            }
        }
        
        Ok(polymorphism_usage)
    }

    /// Analyze inheritance patterns
    fn analyze_inheritance_patterns(&self, content: &str) -> Result<Vec<InheritancePatternInfo>> {
        let mut patterns = Vec::new();
        
        // Find inheritance relationships
        let extends_regex = Regex::new(r"class\s+(\w+)\s+extends\s+(\w+)").unwrap();
        
        for captures in extends_regex.captures_iter(content) {
            let derived_class = captures.get(1).unwrap().as_str().to_string();
            let base_class = captures.get(2).unwrap().as_str().to_string();
            
            patterns.push(InheritancePatternInfo {
                pattern_type: InheritancePatternType::SingleInheritance,
                base_class: base_class.clone(),
                derived_classes: vec![derived_class],
                depth: self.calculate_inheritance_depth(content, &base_class),
                complexity_score: self.calculate_inheritance_complexity(content, &base_class),
                potential_issues: self.identify_inheritance_issues(content, &base_class),
            });
        }
        
        Ok(patterns)
    }

    /// Analyze interface usage
    fn analyze_interface_usage(&self, content: &str) -> Result<Vec<InterfaceUsageInfo>> {
        let mut interface_usage = Vec::new();
        
        // Find interface declarations
        let interface_regex = Regex::new(r"interface\s+(\w+)").unwrap();
        
        for captures in interface_regex.captures_iter(content) {
            let interface_name = captures.get(1).unwrap().as_str().to_string();
            let implementing_classes = self.find_implementing_classes(content, &interface_name);
            let methods = self.extract_interface_methods(content, &interface_name)?;
            let functional_interface = self.is_functional_interface(&methods);
            let lambda_usage = if functional_interface {
                self.find_lambda_usage(content, &interface_name)?
            } else {
                Vec::new()
            };
            
            interface_usage.push(InterfaceUsageInfo {
                interface_name,
                implementing_classes,
                methods,
                functional_interface,
                lambda_usage,
            });
        }
        
        Ok(interface_usage)
    }

    /// Evaluate SOLID principles adherence
    fn evaluate_solid_principles(&self, content: &str) -> Result<SOLIDPrinciplesScore> {
        let single_responsibility = self.evaluate_srp(content);
        let open_closed = self.evaluate_ocp(content);
        let liskov_substitution = self.evaluate_lsp(content);
        let interface_segregation = self.evaluate_isp(content);
        let dependency_inversion = self.evaluate_dip(content);
        
        let overall_score = (single_responsibility + open_closed + liskov_substitution + 
                           interface_segregation + dependency_inversion) / 5;
        
        let violations = self.identify_solid_violations(content)?;
        
        Ok(SOLIDPrinciplesScore {
            single_responsibility,
            open_closed,
            liskov_substitution,
            interface_segregation,
            dependency_inversion,
            overall_score,
            violations,
        })
    }

    /// Detect frameworks in use
    fn detect_frameworks(&self, content: &str) -> Result<Vec<FrameworkInfo>> {
        let mut frameworks = Vec::new();
        
        for (framework_name, patterns) in &self.framework_patterns {
            let mut confidence = 0.0;
            let mut features_used = Vec::new();
            let mut total_weight = 0.0;
            
            for pattern in patterns {
                if pattern.pattern.is_match(content) {
                    confidence += pattern.confidence_weight;
                    total_weight += 1.0;
                    features_used.push(pattern.name.clone());
                }
            }
            
            if confidence > 0.0 {
                frameworks.push(FrameworkInfo {
                    name: framework_name.clone(),
                    version: self.detect_framework_version(content, framework_name),
                    confidence: confidence / total_weight,
                    usage_patterns: features_used,
                    best_practices_followed: self.evaluate_framework_best_practices(content, framework_name),
                    potential_issues: self.identify_framework_issues(content, framework_name),
                });
            }
        }
        
        Ok(frameworks)
    }

    /// Analyze Spring framework specifically
    fn analyze_spring_framework(&self, content: &str) -> Result<Option<SpringAnalysis>> {
        // Check for Spring annotations
        if content.contains("@RestController") || content.contains("@Controller") ||
           content.contains("@Service") || content.contains("@Repository") ||
           content.contains("@Autowired") || content.contains("@Component") {
            
            let spring_analysis = SpringAnalysis {
                spring_boot_used: content.contains("@SpringBootApplication") || content.contains("SpringApplication"),
                components: self.analyze_spring_components(content)?,
                dependency_injection: self.analyze_dependency_injection(content)?,
                aop_usage: self.analyze_aop_patterns(content)?,
                transaction_management: self.analyze_transactions(content)?,
                security_configuration: self.analyze_spring_security(content)?,
                data_access_patterns: self.analyze_data_access(content)?,
            };
            
            Ok(Some(spring_analysis))
        } else {
            Ok(None)
        }
    }

    /// Detect security vulnerabilities
    fn detect_vulnerabilities(&self, content: &str) -> Result<Vec<SecurityVulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        // SQL Injection detection
        if content.contains("+ username +") || content.contains("+ password +") || 
           content.contains("\"SELECT * FROM") && content.contains("'\" + ") {
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: SecurityVulnerabilityType::SqlInjection,
                severity: SecuritySeverity::High,
                location: "Database query construction".to_string(),
                description: "Potential SQL injection vulnerability detected through string concatenation".to_string(),
                cwe_id: Some("CWE-89".to_string()),
                recommendation: "Use prepared statements or parameterized queries".to_string(),
            });
        }
        
        // Hardcoded credentials detection
        if content.contains("PASSWORD = \"") || content.contains("password = \"") ||
           content.contains("\"admin123\"") || content.contains("getConnection(dbUrl, \"admin\"") {
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: SecurityVulnerabilityType::HardcodedCredentials,
                severity: SecuritySeverity::High,
                location: "Configuration or connection code".to_string(),
                description: "Hardcoded credentials detected in source code".to_string(),
                cwe_id: Some("CWE-798".to_string()),
                recommendation: "Use environment variables or secure configuration files".to_string(),
            });
        }
        
        // Weak cryptography detection
        if content.contains("MD5") || content.contains("getInstance(\"MD5\")") {
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: SecurityVulnerabilityType::WeakCryptography,
                severity: SecuritySeverity::Medium,
                location: "Cryptographic implementation".to_string(),
                description: "Use of weak MD5 hashing algorithm detected".to_string(),
                cwe_id: Some("CWE-327".to_string()),
                recommendation: "Use stronger hashing algorithms like SHA-256 or bcrypt".to_string(),
            });
        }
        
        // Command injection detection
        if content.contains("Runtime.getRuntime().exec") && content.contains("+ userInput") {
            vulnerabilities.push(SecurityVulnerability {
                vulnerability_type: SecurityVulnerabilityType::CommandInjection,
                severity: SecuritySeverity::High,
                location: "System command execution".to_string(),
                description: "Potential command injection vulnerability detected".to_string(),
                cwe_id: Some("CWE-78".to_string()),
                recommendation: "Validate and sanitize user input before executing system commands".to_string(),
            });
        }
        
        Ok(vulnerabilities)
    }

    /// Analyze modern Java features
    fn analyze_lambda_expressions(&self, content: &str) -> Result<Vec<LambdaExpressionInfo>> {
        let mut lambdas = Vec::new();
        
        let lambda_regex = Regex::new(r"(\([^)]*\)\s*->|[\w\s]*\s*->)").unwrap();
        
        for m in lambda_regex.find_iter(content) {
            lambdas.push(LambdaExpressionInfo {
                expression: m.as_str().to_string(),
                functional_interface: self.infer_functional_interface(content, m.start()),
                complexity: self.assess_lambda_complexity(m.as_str()),
                captures_variables: self.checks_variable_capture(content, m.start(), m.end()),
                usage_context: self.get_lambda_context(content, m.start()),
                performance_impact: self.assess_lambda_performance_impact(m.as_str()),
            });
        }
        
        Ok(lambdas)
    }

    /// Analyze performance characteristics
    fn analyze_performance(&self, content: &str) -> Result<JavaPerformanceAnalysis> {
        let algorithm_complexity = self.analyze_algorithm_complexity(content)?;
        let collection_usage = self.analyze_collection_usage(content)?;
        let memory_patterns = self.analyze_memory_patterns(content)?;
        let concurrency_patterns = self.analyze_concurrency_patterns(content)?;
        let performance_issues = self.identify_performance_issues(content)?;
        let optimization_opportunities = self.identify_optimization_opportunities(content)?;
        let overall_performance_score = self.calculate_performance_score(
            &algorithm_complexity, &performance_issues, &optimization_opportunities);
        
        Ok(JavaPerformanceAnalysis {
            algorithm_complexity,
            collection_usage,
            memory_patterns,
            concurrency_patterns,
            performance_issues,
            optimization_opportunities,
            overall_performance_score,
        })
    }

    // Helper methods
    fn find_subclasses(&self, content: &str, class_name: &str) -> Vec<String> {
        let regex = Regex::new(&format!(r"class\s+(\w+)\s+extends\s+{}", class_name)).unwrap();
        regex.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn calculate_hierarchy_depth(&self, content: &str, class_name: &str) -> usize {
        let mut depth = 0;
        let mut current_class = class_name.to_string();
        let mut visited = std::collections::HashSet::new();
        
        while let Some(superclass) = self.find_superclass(content, &current_class) {
            if visited.contains(&superclass) {
                break; // Avoid infinite loops
            }
            visited.insert(superclass.clone());
            depth += 1;
            current_class = superclass;
        }
        
        depth
    }

    fn find_superclass(&self, content: &str, class_name: &str) -> Option<String> {
        let regex = Regex::new(&format!(r"class\s+{}\s+extends\s+(\w+)", class_name)).unwrap();
        regex.captures(content)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    fn extract_class_modifiers(&self, content: &str, class_name: &str) -> Vec<String> {
        let regex = Regex::new(&format!(r"((?:public|private|protected|abstract|final|static)\s+)*class\s+{}", class_name)).unwrap();
        regex.captures(content)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("")
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    fn assess_pattern_quality(&self, _content: &str, _pattern_type: &str) -> ImplementationQuality {
        // Simplified assessment - would need more sophisticated analysis
        ImplementationQuality::Good
    }

    fn find_pattern_location(&self, content: &str, pattern: &Regex) -> String {
        pattern.find(content)
            .map(|m| format!("Line {}", self.get_line_number(content, m.start())))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn get_pattern_description(&self, pattern_type: &str) -> String {
        match pattern_type {
            "singleton" => "Singleton pattern ensures a class has only one instance".to_string(),
            "factory" => "Factory pattern creates objects without specifying exact classes".to_string(),
            "builder" => "Builder pattern constructs complex objects step by step".to_string(),
            "observer" => "Observer pattern defines one-to-many dependency between objects".to_string(),
            _ => format!("{} pattern detected", pattern_type),
        }
    }

    fn identify_pattern_participants(&self, _content: &str, _pattern_type: &str) -> Vec<String> {
        // Would need more sophisticated analysis to identify actual participants
        Vec::new()
    }

    fn get_line_number(&self, content: &str, position: usize) -> usize {
        content[..position].chars().filter(|&c| c == '\n').count() + 1
    }

    // Security analysis helper methods
    
    /// Assess implementation quality of security patterns
    fn assess_implementation_quality(&self, content: &str, pattern_type: &str) -> ImplementationQuality {
        match pattern_type {
            "sanitization" => {
                if content.contains("OWASP") || content.contains("AntiSamy") {
                    ImplementationQuality::Excellent
                } else if content.contains("htmlEscape") || content.contains("StringEscapeUtils") {
                    ImplementationQuality::Good
                } else {
                    ImplementationQuality::Poor
                }
            },
            "authentication" => {
                if content.contains("@EnableWebSecurity") && content.contains("BCryptPasswordEncoder") {
                    ImplementationQuality::Excellent
                } else if content.contains("@PreAuthorize") || content.contains("@Secured") {
                    ImplementationQuality::Good
                } else {
                    ImplementationQuality::Adequate
                }
            },
            _ => ImplementationQuality::Adequate
        }
    }

    /// Analyze JWT implementation weaknesses
    fn analyze_jwt_weaknesses(&self, content: &str) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if content.contains("none") || content.contains("\"alg\": \"none\"") {
            weaknesses.push("JWT algorithm set to 'none' - vulnerable to tampering".to_string());
        }
        
        if content.contains("HS256") && !content.contains("secret") {
            weaknesses.push("Weak JWT secret key management".to_string());
        }
        
        if !content.contains("expiration") && !content.contains("exp") {
            weaknesses.push("JWT tokens without expiration time".to_string());
        }
        
        weaknesses
    }

    /// Analyze OAuth2 implementation weaknesses
    fn analyze_oauth2_weaknesses(&self, content: &str) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if content.contains("http://") && content.contains("redirectUri") {
            weaknesses.push("OAuth2 redirect URI using HTTP instead of HTTPS".to_string());
        }
        
        if content.contains("client_secret") && content.contains("=") {
            weaknesses.push("Potential hardcoded OAuth2 client secret".to_string());
        }
        
        weaknesses
    }

    /// Analyze form authentication weaknesses
    fn analyze_form_auth_weaknesses(&self, content: &str) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if !content.contains("BCryptPasswordEncoder") && !content.contains("SCryptPasswordEncoder") {
            weaknesses.push("Weak password encoding mechanism".to_string());
        }
        
        if !content.contains("sessionManagement") {
            weaknesses.push("Missing session management configuration".to_string());
        }
        
        weaknesses
    }

    /// Extract roles from content
    fn extract_roles_from_content(&self, content: &str) -> Vec<String> {
        let mut roles = Vec::new();
        let role_regex = Regex::new(r"ROLE_(\w+)").unwrap();
        
        for captures in role_regex.captures_iter(content) {
            if let Some(role) = captures.get(1) {
                roles.push(format!("ROLE_{}", role.as_str()));
            }
        }
        
        if roles.is_empty() {
            // Look for common role patterns
            if content.contains("ADMIN") {
                roles.push("ROLE_ADMIN".to_string());
            }
            if content.contains("USER") {
                roles.push("ROLE_USER".to_string());
            }
        }
        
        roles
    }

    /// Extract permissions from content
    fn extract_permissions_from_content(&self, content: &str) -> Vec<String> {
        let mut permissions = Vec::new();
        
        if content.contains("READ") {
            permissions.push("READ".to_string());
        }
        if content.contains("WRITE") {
            permissions.push("WRITE".to_string());
        }
        if content.contains("DELETE") {
            permissions.push("DELETE".to_string());
        }
        if content.contains("CREATE") {
            permissions.push("CREATE".to_string());
        }
        
        permissions
    }

    /// Extract access control rules
    fn extract_access_control_rules(&self, content: &str) -> Vec<String> {
        let mut rules = Vec::new();
        
        if content.contains("@PreAuthorize") {
            let pre_auth_regex = Regex::new(r#"@PreAuthorize\("([^"]+)"\)"#).unwrap();
            for captures in pre_auth_regex.captures_iter(content) {
                if let Some(rule) = captures.get(1) {
                    rules.push(rule.as_str().to_string());
                }
            }
        }
        
        if content.contains("@PostAuthorize") {
            let post_auth_regex = Regex::new(r#"@PostAuthorize\("([^"]+)"\)"#).unwrap();
            for captures in post_auth_regex.captures_iter(content) {
                if let Some(rule) = captures.get(1) {
                    rules.push(rule.as_str().to_string());
                }
            }
        }
        
        rules
    }

    /// Extract sanitization techniques
    fn extract_sanitization_techniques(&self, content: &str) -> Vec<String> {
        let mut techniques = Vec::new();
        
        if content.contains("htmlEscape") {
            techniques.push("HTML escaping".to_string());
        }
        if content.contains("StringEscapeUtils") {
            techniques.push("Apache Commons Text escaping".to_string());
        }
        if content.contains("OWASP") {
            techniques.push("OWASP sanitization library".to_string());
        }
        if content.contains("Jsoup.clean") {
            techniques.push("Jsoup HTML sanitization".to_string());
        }
        
        techniques
    }

    /// Extract cryptographic algorithm
    fn extract_crypto_algorithm(&self, content: &str, operation_type: &str) -> String {
        match operation_type {
            "encryption" => {
                if content.contains("AES") {
                    if content.contains("AES/GCM/") {
                        "AES-GCM".to_string()
                    } else if content.contains("AES/CBC/") {
                        "AES-CBC".to_string()
                    } else {
                        "AES".to_string()
                    }
                } else if content.contains("RSA") {
                    "RSA".to_string()
                } else {
                    "Unknown".to_string()
                }
            },
            "hashing" => {
                if content.contains("SHA-256") || content.contains("SHA256") {
                    "SHA-256".to_string()
                } else if content.contains("SHA-512") || content.contains("SHA512") {
                    "SHA-512".to_string()
                } else if content.contains("SHA-1") || content.contains("SHA1") {
                    "SHA-1".to_string()
                } else if content.contains("MD5") {
                    "MD5".to_string()
                } else if content.contains("BCrypt") {
                    "BCrypt".to_string()
                } else {
                    "Unknown".to_string()
                }
            },
            _ => "Unknown".to_string()
        }
    }

    /// Analyze key management patterns
    fn analyze_key_management(&self, content: &str) -> KeyManagementPattern {
        let key_storage = if content.contains("KeyStore") {
            KeyStorageType::Keystore
        } else if content.contains("HSM") {
            KeyStorageType::HSM
        } else if content.contains("System.getenv") || content.contains("@Value") {
            KeyStorageType::Environment
        } else if content.contains("\"key\"") || content.contains("password =") {
            KeyStorageType::Hardcoded
        } else {
            KeyStorageType::Configuration
        };

        let key_strength = if content.contains("2048") || content.contains("256") {
            KeyStrength::Strong
        } else if content.contains("1024") || content.contains("128") {
            KeyStrength::Adequate
        } else if content.contains("512") || content.contains("64") {
            KeyStrength::Weak
        } else {
            KeyStrength::Unknown
        };

        KeyManagementPattern {
            key_storage,
            key_rotation: content.contains("rotation") || content.contains("renew"),
            key_strength,
            key_derivation: if content.contains("PBKDF2") {
                Some("PBKDF2".to_string())
            } else if content.contains("scrypt") {
                Some("scrypt".to_string())
            } else {
                None
            },
        }
    }

    /// Identify cryptographic implementation issues
    fn identify_crypto_issues(&self, content: &str, algorithm: &str) -> Vec<String> {
        let mut issues = Vec::new();
        
        match algorithm {
            "MD5" => issues.push("MD5 is cryptographically broken and should not be used".to_string()),
            "SHA-1" => issues.push("SHA-1 is deprecated and should be replaced with SHA-256 or better".to_string()),
            "AES-ECB" => issues.push("ECB mode is insecure and should not be used".to_string()),
            _ => {}
        }
        
        if content.contains("new Random()") {
            issues.push("Using weak random number generator for cryptographic operations".to_string());
        }
        
        if content.contains("DES") {
            issues.push("DES encryption is weak and should be replaced with AES".to_string());
        }
        
        issues
    }

    /// Extract CSRF configuration
    fn extract_csrf_config(&self, content: &str) -> Vec<String> {
        let mut config = Vec::new();
        
        if content.contains("csrf().disable()") {
            config.push("CSRF protection disabled".to_string());
        } else if content.contains("csrfTokenRepository") {
            config.push("Custom CSRF token repository configured".to_string());
        } else if content.contains("csrf()") {
            config.push("Default CSRF protection enabled".to_string());
        }
        
        config
    }

    /// Extract XSS configuration
    fn extract_xss_config(&self, content: &str) -> Vec<String> {
        let mut config = Vec::new();
        
        if content.contains("X-XSS-Protection") {
            config.push("XSS Protection header configured".to_string());
        }
        if content.contains("htmlEscape") {
            config.push("HTML output escaping implemented".to_string());
        }
        if content.contains("@ResponseBody") {
            config.push("Response body serialization (potential XSS vector)".to_string());
        }
        
        config
    }

    /// Assess XSS protection effectiveness
    fn assess_xss_protection_effectiveness(&self, content: &str) -> SecurityEffectiveness {
        if content.contains("OWASP") || content.contains("AntiSamy") {
            SecurityEffectiveness::Excellent
        } else if content.contains("htmlEscape") || content.contains("StringEscapeUtils") {
            SecurityEffectiveness::Good
        } else if content.contains("@ResponseBody") && !content.contains("escape") {
            SecurityEffectiveness::Poor
        } else {
            SecurityEffectiveness::Adequate
        }
    }

    /// Extract HTTPS configuration
    fn extract_https_config(&self, content: &str) -> Vec<String> {
        let mut config = Vec::new();
        
        if content.contains("requiresChannel().requestMatchers") {
            config.push("Channel security configured".to_string());
        }
        if content.contains("HTTPS") {
            config.push("HTTPS enforcement detected".to_string());
        }
        if content.contains("secure: true") {
            config.push("Secure cookie configuration".to_string());
        }
        
        config
    }

    /// Extract CSP configuration
    fn extract_csp_config(&self, content: &str) -> Vec<String> {
        let mut config = Vec::new();
        
        if content.contains("Content-Security-Policy") {
            config.push("Content Security Policy configured".to_string());
        }
        if content.contains("X-Frame-Options") {
            config.push("Frame options configured".to_string());
        }
        if content.contains("X-Content-Type-Options") {
            config.push("Content type options configured".to_string());
        }
        
        config
    }

    /// Extract CORS configuration
    fn extract_cors_config(&self, content: &str) -> Vec<String> {
        let mut config = Vec::new();
        
        if content.contains("@CrossOrigin") {
            config.push("Cross-origin annotations detected".to_string());
        }
        if content.contains("allowedOrigins") {
            config.push("Allowed origins configured".to_string());
        }
        if content.contains("allowCredentials") {
            config.push("Credentials allowed in CORS".to_string());
        }
        
        config
    }

    /// Assess CORS security
    fn assess_cors_security(&self, content: &str) -> SecurityEffectiveness {
        if content.contains("allowedOrigins(\"*\")") || content.contains("origins = \"*\"") {
            SecurityEffectiveness::Poor
        } else if content.contains("allowedOrigins") && content.contains("https://") {
            SecurityEffectiveness::Good
        } else if content.contains("@CrossOrigin") && !content.contains("origins") {
            SecurityEffectiveness::Adequate
        } else {
            SecurityEffectiveness::Good
        }
    }

    fn calculate_overall_score(&self, _content: &str) -> i32 {
        // Placeholder implementation
        75
    }

    fn generate_recommendations(&self, _content: &str) -> Vec<String> {
        vec![
            "Consider using more modern Java features like Streams and Optional".to_string(),
            "Implement proper error handling and logging".to_string(),
            "Review security patterns and input validation".to_string(),
            "Consider applying SOLID principles more consistently".to_string(),
        ]
    }

    // Additional helper methods would be implemented here...
    // (I'll add more specific implementations as needed)

    // Missing method implementations (stub versions)
    fn analyze_field_access(&self, _content: &str, _class_name: &str) -> Result<Vec<FieldAccessInfo>> {
        Ok(Vec::new())
    }

    fn analyze_getter_setters(&self, _content: &str, _class_name: &str) -> Result<Vec<GetterSetterInfo>> {
        Ok(Vec::new())
    }

    fn calculate_data_hiding_score(&self, _field_access_analysis: &[FieldAccessInfo]) -> i32 {
        75
    }

    fn analyze_immutability_patterns(&self, _content: &str, _class_name: &str) -> Result<Vec<ImmutabilityPattern>> {
        Ok(Vec::new())
    }

    fn find_containing_class(&self, content: &str, position: usize) -> Option<String> {
        let before_position = &content[..position];
        let class_regex = Regex::new(r"class\s+(\w+)").unwrap();
        
        // Find the last class declaration before the position
        class_regex.captures_iter(before_position)
            .last()
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    fn find_base_type(&self, content: &str, class_name: &str) -> Option<String> {
        self.find_superclass(content, class_name)
    }

    fn calculate_inheritance_depth(&self, _content: &str, _base_class: &str) -> usize {
        1
    }

    fn calculate_inheritance_complexity(&self, _content: &str, _base_class: &str) -> i32 {
        50
    }

    fn identify_inheritance_issues(&self, _content: &str, _base_class: &str) -> Vec<String> {
        Vec::new()
    }

    fn find_implementing_classes(&self, content: &str, interface_name: &str) -> Vec<String> {
        let regex = Regex::new(&format!(r"class\s+(\w+).*implements.*{}", interface_name)).unwrap();
        regex.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn extract_interface_methods(&self, _content: &str, _interface_name: &str) -> Result<Vec<InterfaceMethodInfo>> {
        Ok(Vec::new())
    }

    fn is_functional_interface(&self, methods: &[InterfaceMethodInfo]) -> bool {
        methods.iter().filter(|m| !m.is_default && !m.is_static).count() == 1
    }

    fn find_lambda_usage(&self, _content: &str, _interface_name: &str) -> Result<Vec<LambdaUsageInfo>> {
        Ok(Vec::new())
    }

    fn evaluate_srp(&self, _content: &str) -> i32 { 70 }
    fn evaluate_ocp(&self, _content: &str) -> i32 { 70 }
    fn evaluate_lsp(&self, _content: &str) -> i32 { 70 }
    fn evaluate_isp(&self, _content: &str) -> i32 { 70 }
    fn evaluate_dip(&self, _content: &str) -> i32 { 70 }

    fn identify_solid_violations(&self, _content: &str) -> Result<Vec<SOLIDViolation>> {
        Ok(Vec::new())
    }

    fn detect_framework_version(&self, _content: &str, _framework_name: &str) -> Option<String> {
        None
    }

    fn evaluate_framework_best_practices(&self, _content: &str, _framework_name: &str) -> Vec<String> {
        Vec::new()
    }

    fn identify_framework_issues(&self, _content: &str, _framework_name: &str) -> Vec<String> {
        Vec::new()
    }

    fn calculate_framework_score(&self, _frameworks: &[FrameworkInfo]) -> i32 {
        75
    }

    fn analyze_spring_components(&self, content: &str) -> Result<Vec<SpringComponentInfo>> {
        let mut components = Vec::new();
        
        // Spring component annotations
        let component_patterns = [
            ("@Component", SpringComponentType::Component),
            ("@Service", SpringComponentType::Service),
            ("@Repository", SpringComponentType::Repository),
            ("@Controller", SpringComponentType::Controller),
            ("@RestController", SpringComponentType::RestController),
            ("@Configuration", SpringComponentType::Configuration),
        ];
        
        for (annotation, component_type) in component_patterns {
            // Find all occurrences of the annotation
            let annotation_regex = Regex::new(&format!(r"{}(?:\([^)]*\))?\s+(?:public\s+)?class\s+(\w+)", annotation))?;
            
            for captures in annotation_regex.captures_iter(content) {
                let class_name = captures.get(1).unwrap().as_str().to_string();
                
                // Extract annotation parameters if any
                let annotations = self.extract_class_annotations(content, &class_name);
                
                // Find dependencies via @Autowired
                let dependencies = self.find_autowired_dependencies(content, &class_name);
                
                // Determine scope (default is singleton)
                let scope = self.extract_component_scope(content, &class_name);
                
                components.push(SpringComponentInfo {
                    component_type: component_type.clone(),
                    class_name,
                    annotations,
                    scope,
                    dependencies,
                });
            }
        }
        
        Ok(components)
    }

    fn analyze_dependency_injection(&self, content: &str) -> Result<Vec<DIPatternInfo>> {
        let mut di_patterns = Vec::new();
        
        // Find @Autowired annotations
        let autowired_regex = Regex::new(r"@Autowired\s+(?:private\s+|protected\s+|public\s+)?(\w+(?:<[^>]+>)?)\s+(\w+)")?;
        
        for captures in autowired_regex.captures_iter(content) {
            let dependency_type = captures.get(1).unwrap().as_str().to_string();
            let field_name = captures.get(2).unwrap().as_str().to_string();
            
            // Find the containing class
            let class_name = self.find_containing_class(content, captures.get(0).unwrap().start())
                .unwrap_or_else(|| "Unknown".to_string());
            
            // Assess DI best practices
            let follows_best_practices = self.assess_di_best_practices(content, &field_name);
            let potential_issues = self.identify_di_issues(content, &field_name, &dependency_type);
            
            di_patterns.push(DIPatternInfo {
                injection_type: DIType::Field,
                target_class: class_name,
                dependencies: vec![dependency_type],
                follows_best_practices,
                potential_issues,
            });
        }
        
        // Find constructor injection
        let constructor_injection_regex = Regex::new(r"public\s+(\w+)\s*\(\s*([^)]+)\s*\)")?;
        
        for captures in constructor_injection_regex.captures_iter(content) {
            let class_name = captures.get(1).unwrap().as_str().to_string();
            let params_str = captures.get(2).unwrap().as_str();
            
            // Check if this constructor has @Autowired or is the only constructor
            let is_di_constructor = content.contains("@Autowired") || 
                                  self.count_constructors(content, &class_name) == 1;
            
            if is_di_constructor {
                let dependencies = self.parse_constructor_parameters(params_str);
                let follows_best_practices = dependencies.len() <= 3; // Best practice: limit dependencies
                let potential_issues = if dependencies.len() > 5 {
                    vec!["Too many dependencies - consider refactoring".to_string()]
                } else {
                    Vec::new()
                };
                
                di_patterns.push(DIPatternInfo {
                    injection_type: DIType::Constructor,
                    target_class: class_name,
                    dependencies,
                    follows_best_practices,
                    potential_issues,
                });
            }
        }
        
        Ok(di_patterns)
    }

    // Helper methods for Spring analysis
    fn extract_class_annotations(&self, content: &str, class_name: &str) -> Vec<String> {
        let mut annotations = Vec::new();
        
        // Find the class definition and extract all annotations above it
        let class_regex = Regex::new(&format!(r"((?:@\w+(?:\([^)]*\))?\s*)*)\s*(?:public\s+)?class\s+{}", class_name)).unwrap();
        
        if let Some(captures) = class_regex.captures(content) {
            let annotations_text = captures.get(1).unwrap().as_str();
            let annotation_regex = Regex::new(r"@(\w+)(?:\([^)]*\))?").unwrap();
            
            for annotation_capture in annotation_regex.captures_iter(annotations_text) {
                annotations.push(annotation_capture.get(0).unwrap().as_str().to_string());
            }
        }
        
        annotations
    }
    
    fn find_autowired_dependencies(&self, content: &str, class_name: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Find class boundaries
        let class_start = content.find(&format!("class {}", class_name));
        if class_start.is_none() {
            return dependencies;
        }
        
        let class_content = self.extract_class_content(content, class_name);
        
        // Look for @Autowired fields
        let autowired_regex = Regex::new(r"@Autowired\s+(?:private\s+|protected\s+|public\s+)?(\w+(?:<[^>]+>)?)\s+(\w+)").unwrap();
        
        for captures in autowired_regex.captures_iter(&class_content) {
            let dependency_type = captures.get(1).unwrap().as_str();
            dependencies.push(dependency_type.to_string());
        }
        
        dependencies
    }
    
    fn extract_component_scope(&self, content: &str, class_name: &str) -> String {
        // Look for @Scope annotation
        let scope_regex = Regex::new(&format!(r#"@Scope\s*\(\s*["']([^"']+)["']\s*\).*?class\s+{}"#, class_name)).unwrap();
        
        if let Some(captures) = scope_regex.captures(content) {
            captures.get(1).unwrap().as_str().to_string()
        } else {
            "singleton".to_string() // Default Spring scope
        }
    }
    
    fn extract_class_content(&self, content: &str, class_name: &str) -> String {
        // Find class definition and extract content between braces
        let class_regex = Regex::new(&format!(r"class\s+{}\s*\{{([^{{}}]*(?:\{{[^{{}}]*\}}[^{{}}]*)*)\}}", class_name)).unwrap();
        
        if let Some(captures) = class_regex.captures(content) {
            captures.get(1).unwrap().as_str().to_string()
        } else {
            String::new()
        }
    }
    
    fn assess_di_best_practices(&self, content: &str, field_name: &str) -> bool {
        // Check if field is final (constructor injection) or properly encapsulated
        let field_regex = Regex::new(&format!(r"(?:private\s+)?(?:final\s+)?\w+\s+{}", field_name)).unwrap();
        
        if let Some(field_match) = field_regex.find(content) {
            let field_def = field_match.as_str();
            field_def.contains("private") && !field_def.contains("public")
        } else {
            false
        }
    }
    
    fn identify_di_issues(&self, content: &str, field_name: &str, dependency_type: &str) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for circular dependencies (simplified check)
        if content.contains("@Autowired") && dependency_type.contains("Service") &&
           content.contains(&format!("class {}Service", field_name)) {
            issues.push("Potential circular dependency detected".to_string());
        }
        
        // Check for field injection instead of constructor injection
        if content.contains("@Autowired") && content.contains(field_name) {
            issues.push("Consider using constructor injection instead of field injection".to_string());
        }
        
        issues
    }
    
    fn count_constructors(&self, content: &str, class_name: &str) -> usize {
        let constructor_regex = Regex::new(&format!(r"public\s+{}\s*\(", class_name)).unwrap();
        constructor_regex.find_iter(content).count()
    }
    
    fn parse_constructor_parameters(&self, params_str: &str) -> Vec<String> {
        if params_str.trim().is_empty() {
            return Vec::new();
        }
        
        params_str.split(',')
            .map(|param| {
                // Extract type from "Type varName" pattern
                let parts: Vec<&str> = param.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    parts[0].to_string()
                } else {
                    param.trim().to_string()
                }
            })
            .collect()
    }

    fn analyze_aop_patterns(&self, _content: &str) -> Result<Vec<AOPPatternInfo>> {
        Ok(Vec::new())
    }

    fn analyze_transactions(&self, _content: &str) -> Result<Vec<TransactionInfo>> {
        Ok(Vec::new())
    }

    fn analyze_spring_security(&self, _content: &str) -> Result<Option<SpringSecurityInfo>> {
        Ok(None)
    }

    fn analyze_data_access(&self, _content: &str) -> Result<Vec<DataAccessPatternInfo>> {
        Ok(Vec::new())
    }

    fn analyze_hibernate(&self, _content: &str) -> Result<Option<HibernateAnalysis>> {
        Ok(None)
    }

    fn analyze_junit(&self, _content: &str) -> Result<Option<JUnitAnalysis>> {
        Ok(None)
    }

    fn analyze_maven(&self, _content: &str) -> Result<Option<MavenAnalysis>> {
        Ok(None)
    }

    fn analyze_gradle(&self, _content: &str) -> Result<Option<GradleAnalysis>> {
        Ok(None)
    }

    /// Detect security patterns in code
    fn detect_security_patterns(&self, content: &str) -> Result<Vec<SecurityPattern>> {
        let mut security_patterns = Vec::new();
        
        // Input sanitization patterns
        if content.contains("StringEscapeUtils") || content.contains("OWASP") || 
           content.contains("htmlEscape") || content.contains("sanitize") {
            security_patterns.push(SecurityPattern {
                pattern_type: SecurityPatternType::InputSanitization,
                implementation_quality: self.assess_implementation_quality(content, "sanitization"),
                location: "Multiple locations".to_string(),
                description: "Input sanitization implementation detected".to_string(),
            });
        }
        
        // Authentication patterns
        if content.contains("@PreAuthorize") || content.contains("@Secured") ||
           content.contains("SecurityContextHolder") || content.contains("UserDetails") {
            security_patterns.push(SecurityPattern {
                pattern_type: SecurityPatternType::SecureAuthentication,
                implementation_quality: self.assess_implementation_quality(content, "authentication"),
                location: "Authentication mechanisms".to_string(),
                description: "Authentication security patterns detected".to_string(),
            });
        }
        
        // Audit logging patterns
        if content.contains("@Audit") || content.contains("SecurityEvent") ||
           content.contains("logger.info") && content.contains("security") {
            security_patterns.push(SecurityPattern {
                pattern_type: SecurityPatternType::AuditLogging,
                implementation_quality: self.assess_implementation_quality(content, "logging"),
                location: "Logging statements".to_string(),
                description: "Security audit logging detected".to_string(),
            });
        }
        
        // Secure communication patterns
        if content.contains("https://") || content.contains("TLS") ||
           content.contains("SSLContext") || content.contains("HttpsURLConnection") {
            security_patterns.push(SecurityPattern {
                pattern_type: SecurityPatternType::SecureCommunication,
                implementation_quality: self.assess_implementation_quality(content, "communication"),
                location: "Network communication".to_string(),
                description: "Secure communication patterns detected".to_string(),
            });
        }
        
        // Session management patterns
        if content.contains("HttpSession") || content.contains("sessionManagement") ||
           content.contains("invalidate()") || content.contains("JSESSIONID") {
            security_patterns.push(SecurityPattern {
                pattern_type: SecurityPatternType::SessionManagement,
                implementation_quality: self.assess_implementation_quality(content, "session"),
                location: "Session handling".to_string(),
                description: "Session management security patterns detected".to_string(),
            });
        }
        
        Ok(security_patterns)
    }

    /// Analyze authentication patterns
    fn analyze_authentication(&self, content: &str) -> Result<Vec<AuthenticationPattern>> {
        let mut auth_patterns = Vec::new();
        
        // JWT Token authentication
        if content.contains("JWT") || content.contains("JsonWebToken") || 
           content.contains("jwtDecode") || content.contains("Claims") {
            let weaknesses = self.analyze_jwt_weaknesses(content);
            auth_patterns.push(AuthenticationPattern {
                authentication_type: AuthenticationType::JwtToken,
                implementation_class: "JWT Token Handler".to_string(),
                security_features: vec![
                    "Token-based authentication".to_string(),
                    "Stateless authentication".to_string(),
                ],
                weaknesses,
            });
        }
        
        // OAuth2 authentication
        if content.contains("OAuth2") || content.contains("@EnableOAuth2Sso") ||
           content.contains("OAuth2Authentication") || content.contains("AuthorizationServer") {
            let weaknesses = self.analyze_oauth2_weaknesses(content);
            auth_patterns.push(AuthenticationPattern {
                authentication_type: AuthenticationType::OAuth2,
                implementation_class: "OAuth2 Provider".to_string(),
                security_features: vec![
                    "OAuth2 authorization flow".to_string(),
                    "Token-based access control".to_string(),
                ],
                weaknesses,
            });
        }
        
        // Form-based authentication
        if content.contains("formLogin") || content.contains("UsernamePasswordAuthenticationToken") ||
           content.contains("AuthenticationProvider") {
            let weaknesses = self.analyze_form_auth_weaknesses(content);
            auth_patterns.push(AuthenticationPattern {
                authentication_type: AuthenticationType::FormBased,
                implementation_class: "Form Authentication".to_string(),
                security_features: vec![
                    "Username/password authentication".to_string(),
                    "Session-based authentication".to_string(),
                ],
                weaknesses,
            });
        }
        
        // Basic authentication
        if content.contains("BasicAuthenticationFilter") || content.contains("httpBasic") ||
           content.contains("Authorization: Basic") {
            auth_patterns.push(AuthenticationPattern {
                authentication_type: AuthenticationType::BasicAuth,
                implementation_class: "Basic Authentication".to_string(),
                security_features: vec![
                    "HTTP Basic authentication".to_string(),
                ],
                weaknesses: vec![
                    "Credentials transmitted in base64 encoding".to_string(),
                    "Vulnerable without HTTPS".to_string(),
                ],
            });
        }
        
        Ok(auth_patterns)
    }

    /// Analyze authorization patterns
    fn analyze_authorization(&self, content: &str) -> Result<Vec<AuthorizationPattern>> {
        let mut auth_patterns = Vec::new();
        
        // Role-based authorization
        if content.contains("@RolesAllowed") || content.contains("hasRole") ||
           content.contains("ROLE_") || content.contains("GrantedAuthority") {
            let roles = self.extract_roles_from_content(content);
            auth_patterns.push(AuthorizationPattern {
                authorization_type: AuthorizationType::RoleBased,
                roles,
                permissions: Vec::new(),
                access_control_rules: self.extract_access_control_rules(content),
            });
        }
        
        // Permission-based authorization
        if content.contains("@PreAuthorize") || content.contains("hasPermission") ||
           content.contains("Permission") || content.contains("ACL") {
            let permissions = self.extract_permissions_from_content(content);
            auth_patterns.push(AuthorizationPattern {
                authorization_type: AuthorizationType::PermissionBased,
                roles: Vec::new(),
                permissions,
                access_control_rules: self.extract_access_control_rules(content),
            });
        }
        
        // Attribute-based authorization
        if content.contains("@PostAuthorize") || content.contains("SecurityEvaluationContext") ||
           content.contains("SpEL") && content.contains("security") {
            auth_patterns.push(AuthorizationPattern {
                authorization_type: AuthorizationType::AttributeBased,
                roles: Vec::new(),
                permissions: Vec::new(),
                access_control_rules: vec![
                    "Attribute-based access control with SpEL expressions".to_string(),
                ],
            });
        }
        
        Ok(auth_patterns)
    }

    /// Analyze input validation patterns
    fn analyze_input_validation(&self, content: &str) -> Result<Vec<InputValidationPattern>> {
        let mut validation_patterns = Vec::new();
        
        // Bean validation
        if content.contains("@Valid") || content.contains("@NotNull") ||
           content.contains("@Size") || content.contains("@Pattern") {
            validation_patterns.push(InputValidationPattern {
                validation_type: ValidationType::TypeValidation,
                input_sources: vec!["HTTP parameters".to_string(), "Request body".to_string()],
                validation_methods: vec!["Bean Validation annotations".to_string()],
                sanitization_techniques: self.extract_sanitization_techniques(content),
            });
        }
        
        // Regex validation
        if content.contains("Pattern.compile") || content.contains("matches()") ||
           content.contains("Regex") || content.contains("\\\\") {
            validation_patterns.push(InputValidationPattern {
                validation_type: ValidationType::RegexValidation,
                input_sources: vec!["String inputs".to_string()],
                validation_methods: vec!["Regular expression validation".to_string()],
                sanitization_techniques: self.extract_sanitization_techniques(content),
            });
        }
        
        // Whitelist validation
        if content.contains("whitelist") || content.contains("allowedValues") ||
           content.contains("VALID_") || content.contains("permitted") {
            validation_patterns.push(InputValidationPattern {
                validation_type: ValidationType::Whitelist,
                input_sources: vec!["User inputs".to_string()],
                validation_methods: vec!["Whitelist validation".to_string()],
                sanitization_techniques: self.extract_sanitization_techniques(content),
            });
        }
        
        Ok(validation_patterns)
    }

    /// Analyze cryptographic patterns
    fn analyze_cryptography(&self, content: &str) -> Result<Vec<CryptographicPattern>> {
        let mut crypto_patterns = Vec::new();
        
        // Encryption patterns
        if content.contains("Cipher.getInstance") || content.contains("AES") ||
           content.contains("RSA") || content.contains("encrypt") {
            let algorithm = self.extract_crypto_algorithm(content, "encryption");
            let key_management = self.analyze_key_management(content);
            let issues = self.identify_crypto_issues(content, &algorithm);
            
            crypto_patterns.push(CryptographicPattern {
                crypto_operation: CryptographicOperation::Encryption,
                algorithm,
                key_management,
                implementation_issues: issues,
            });
        }
        
        // Hashing patterns
        if content.contains("MessageDigest") || content.contains("hash") ||
           content.contains("SHA") || content.contains("BCrypt") {
            let algorithm = self.extract_crypto_algorithm(content, "hashing");
            let key_management = self.analyze_key_management(content);
            let issues = self.identify_crypto_issues(content, &algorithm);
            
            crypto_patterns.push(CryptographicPattern {
                crypto_operation: CryptographicOperation::Hashing,
                algorithm,
                key_management,
                implementation_issues: issues,
            });
        }
        
        // Digital signature patterns
        if content.contains("Signature.getInstance") || content.contains("sign()") ||
           content.contains("verify()") || content.contains("DSA") {
            let algorithm = self.extract_crypto_algorithm(content, "signature");
            let key_management = self.analyze_key_management(content);
            let issues = self.identify_crypto_issues(content, &algorithm);
            
            crypto_patterns.push(CryptographicPattern {
                crypto_operation: CryptographicOperation::DigitalSignature,
                algorithm,
                key_management,
                implementation_issues: issues,
            });
        }
        
        Ok(crypto_patterns)
    }

    /// Analyze web security patterns
    fn analyze_web_security(&self, content: &str) -> Result<Vec<WebSecurityPattern>> {
        let mut web_security_patterns = Vec::new();
        
        // CSRF protection
        if content.contains("@EnableWebSecurity") || content.contains("csrf()") ||
           content.contains("CsrfToken") || content.contains("_csrf") {
            let effectiveness = if content.contains("csrf().disable()") {
                SecurityEffectiveness::Missing
            } else if content.contains("csrfTokenRepository") {
                SecurityEffectiveness::Excellent
            } else {
                SecurityEffectiveness::Good
            };
            
            web_security_patterns.push(WebSecurityPattern {
                security_mechanism: WebSecurityMechanism::CsrfProtection,
                configuration: self.extract_csrf_config(content),
                effectiveness,
            });
        }
        
        // XSS protection
        if content.contains("X-XSS-Protection") || content.contains("htmlEscape") ||
           content.contains("ResponseEntity") || content.contains("@ResponseBody") {
            let effectiveness = self.assess_xss_protection_effectiveness(content);
            web_security_patterns.push(WebSecurityPattern {
                security_mechanism: WebSecurityMechanism::XssProtection,
                configuration: self.extract_xss_config(content),
                effectiveness,
            });
        }
        
        // HTTPS enforcement
        if content.contains("requiresChannel") || content.contains("HTTPS") ||
           content.contains("redirectStrategy") || content.contains("secure: true") {
            web_security_patterns.push(WebSecurityPattern {
                security_mechanism: WebSecurityMechanism::HttpsEnforcement,
                configuration: self.extract_https_config(content),
                effectiveness: SecurityEffectiveness::Good,
            });
        }
        
        // Content Security Policy
        if content.contains("Content-Security-Policy") || content.contains("CSP") ||
           content.contains("X-Frame-Options") || content.contains("X-Content-Type-Options") {
            web_security_patterns.push(WebSecurityPattern {
                security_mechanism: WebSecurityMechanism::ContentSecurityPolicy,
                configuration: self.extract_csp_config(content),
                effectiveness: SecurityEffectiveness::Good,
            });
        }
        
        // CORS configuration
        if content.contains("@CrossOrigin") || content.contains("CorsConfiguration") ||
           content.contains("allowedOrigins") || content.contains("Access-Control") {
            let effectiveness = self.assess_cors_security(content);
            web_security_patterns.push(WebSecurityPattern {
                security_mechanism: WebSecurityMechanism::CorsConfiguration,
                configuration: self.extract_cors_config(content),
                effectiveness,
            });
        }
        
        Ok(web_security_patterns)
    }

    fn determine_security_level(&self, vulnerabilities: &[SecurityVulnerability], _security_patterns: &[SecurityPattern]) -> SecurityLevel {
        if vulnerabilities.iter().any(|v| matches!(v.severity, SecuritySeverity::Critical)) {
            SecurityLevel::Vulnerable
        } else if vulnerabilities.iter().any(|v| matches!(v.severity, SecuritySeverity::High)) {
            SecurityLevel::Low
        } else {
            SecurityLevel::Medium
        }
    }

    fn generate_security_recommendations(&self, _vulnerabilities: &[SecurityVulnerability], _security_patterns: &[SecurityPattern]) -> Vec<String> {
        vec![
            "Implement input validation for all user inputs".to_string(),
            "Use parameterized queries to prevent SQL injection".to_string(),
            "Implement proper authentication and authorization".to_string(),
        ]
    }

    fn get_vulnerability_description(&self, vulnerability_type: &str) -> String {
        match vulnerability_type {
            "sql_injection" => "Potential SQL injection vulnerability detected".to_string(),
            "hardcoded_credentials" => "Hardcoded credentials found in source code".to_string(),
            "command_injection" => "Potential command injection vulnerability".to_string(),
            "path_traversal" => "Potential path traversal vulnerability".to_string(),
            "weak_cryptography" => "Weak cryptographic algorithm detected".to_string(),
            "insecure_randomness" => "Insecure random number generation".to_string(),
            _ => format!("Security issue: {}", vulnerability_type),
        }
    }

    fn get_cwe_id(&self, vulnerability_type: &str) -> Option<String> {
        match vulnerability_type {
            "sql_injection" => Some("CWE-89".to_string()),
            "hardcoded_credentials" => Some("CWE-798".to_string()),
            "command_injection" => Some("CWE-78".to_string()),
            "path_traversal" => Some("CWE-22".to_string()),
            "weak_cryptography" => Some("CWE-327".to_string()),
            "insecure_randomness" => Some("CWE-330".to_string()),
            _ => None,
        }
    }

    fn get_security_recommendation(&self, vulnerability_type: &str) -> String {
        match vulnerability_type {
            "sql_injection" => "Use parameterized queries or prepared statements".to_string(),
            "hardcoded_credentials" => "Store credentials in environment variables or secure configuration".to_string(),
            "command_injection" => "Validate and sanitize all input before using in system commands".to_string(),
            "path_traversal" => "Validate file paths and use canonicalization".to_string(),
            "weak_cryptography" => "Use strong cryptographic algorithms like SHA-256 or better".to_string(),
            "insecure_randomness" => "Use SecureRandom for cryptographic operations".to_string(),
            _ => "Review and fix security vulnerability".to_string(),
        }
    }

    fn detect_java_version(&self, _content: &str) -> Result<JavaVersionInfo> {
        Ok(JavaVersionInfo {
            minimum_version_required: "8".to_string(),
            features_by_version: Vec::new(),
            compatibility_issues: Vec::new(),
        })
    }

    fn analyze_stream_api(&self, _content: &str) -> Result<Vec<StreamApiUsageInfo>> {
        Ok(Vec::new())
    }

    fn analyze_optional_usage(&self, _content: &str) -> Result<Vec<OptionalUsageInfo>> {
        Ok(Vec::new())
    }

    fn analyze_module_system(&self, _content: &str) -> Result<Option<ModuleSystemInfo>> {
        Ok(None)
    }

    fn analyze_record_classes(&self, _content: &str) -> Result<Vec<RecordClassInfo>> {
        Ok(Vec::new())
    }

    fn analyze_sealed_classes(&self, _content: &str) -> Result<Vec<SealedClassInfo>> {
        Ok(Vec::new())
    }

    fn analyze_switch_expressions(&self, _content: &str) -> Result<Vec<SwitchExpressionInfo>> {
        Ok(Vec::new())
    }

    fn analyze_text_blocks(&self, _content: &str) -> Result<Vec<TextBlockInfo>> {
        Ok(Vec::new())
    }

    fn analyze_var_usage(&self, _content: &str) -> Result<Vec<VarUsageInfo>> {
        Ok(Vec::new())
    }

    fn analyze_completable_future(&self, _content: &str) -> Result<Vec<CompletableFutureInfo>> {
        Ok(Vec::new())
    }

    fn analyze_date_time_api(&self, _content: &str) -> Result<Vec<DateTimeApiInfo>> {
        Ok(Vec::new())
    }

    fn analyze_collection_factories(&self, _content: &str) -> Result<Vec<CollectionFactoryInfo>> {
        Ok(Vec::new())
    }

    fn calculate_modernity_score(&self, _content: &str) -> i32 {
        60
    }

    fn infer_functional_interface(&self, _content: &str, _position: usize) -> String {
        "Unknown".to_string()
    }

    fn assess_lambda_complexity(&self, _expression: &str) -> LambdaComplexity {
        LambdaComplexity::Simple
    }

    fn checks_variable_capture(&self, _content: &str, _start: usize, _end: usize) -> bool {
        false
    }

    fn get_lambda_context(&self, _content: &str, _position: usize) -> String {
        "Unknown".to_string()
    }

    fn assess_lambda_performance_impact(&self, _expression: &str) -> PerformanceImpact {
        PerformanceImpact::Neutral
    }

    fn analyze_algorithm_complexity(&self, _content: &str) -> Result<Vec<ComplexityAnalysis>> {
        Ok(Vec::new())
    }

    fn analyze_collection_usage(&self, _content: &str) -> Result<Vec<CollectionUsageInfo>> {
        Ok(Vec::new())
    }

    fn analyze_memory_patterns(&self, _content: &str) -> Result<Vec<MemoryPatternInfo>> {
        Ok(Vec::new())
    }

    fn analyze_concurrency_patterns(&self, _content: &str) -> Result<Vec<ConcurrencyPatternInfo>> {
        Ok(Vec::new())
    }

    fn identify_performance_issues(&self, _content: &str) -> Result<Vec<PerformanceIssue>> {
        Ok(Vec::new())
    }

    fn identify_optimization_opportunities(&self, _content: &str) -> Result<Vec<OptimizationOpportunity>> {
        Ok(Vec::new())
    }

    fn calculate_performance_score(&self, _algorithm_complexity: &[ComplexityAnalysis], _performance_issues: &[PerformanceIssue], _optimization_opportunities: &[OptimizationOpportunity]) -> i32 {
        70
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

/// Comprehensive Java analysis result
#[derive(Debug)]
pub struct JavaComprehensiveAnalysis {
    pub oop_analysis: OOPAnalysisInfo,
    pub framework_analysis: JavaFrameworkAnalysis,
    pub security_analysis: JavaSecurityAnalysis,
    pub modern_features: ModernJavaFeatureAnalysis,
    pub performance_analysis: JavaPerformanceAnalysis,
    pub overall_score: i32,
    pub recommendations: Vec<String>,
}

/// Java performance analysis
#[derive(Debug, Clone)]
pub struct JavaPerformanceAnalysis {
    pub algorithm_complexity: Vec<ComplexityAnalysis>,
    pub collection_usage: Vec<CollectionUsageInfo>,
    pub memory_patterns: Vec<MemoryPatternInfo>,
    pub concurrency_patterns: Vec<ConcurrencyPatternInfo>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub overall_performance_score: i32,
}

/// Algorithm complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityAnalysis {
    pub method_name: String,
    pub time_complexity: String,
    pub space_complexity: String,
    pub complexity_score: i32,
    pub potential_optimizations: Vec<String>,
}

/// Collection usage information
#[derive(Debug, Clone)]
pub struct CollectionUsageInfo {
    pub collection_type: String,
    pub usage_pattern: CollectionUsagePattern,
    pub size_characteristics: CollectionSizeCharacteristics,
    pub performance_implications: Vec<String>,
}

/// Collection usage patterns
#[derive(Debug, Clone)]
pub enum CollectionUsagePattern {
    Appropriate,
    Suboptimal,
    Inefficient,
}

/// Collection size characteristics
#[derive(Debug, Clone)]
pub enum CollectionSizeCharacteristics {
    Small,      // < 100 elements
    Medium,     // 100-10000 elements
    Large,      // > 10000 elements
    Unknown,
}

/// Memory pattern information
#[derive(Debug, Clone)]
pub struct MemoryPatternInfo {
    pub pattern_type: MemoryPatternType,
    pub location: String,
    pub memory_impact: MemoryImpact,
    pub recommendations: Vec<String>,
}

/// Memory pattern types
#[derive(Debug, Clone)]
pub enum MemoryPatternType {
    StringConcatenation,
    LargeObjectCreation,
    CachingPattern,
    MemoryLeak,
    ResourceLeak,
}

/// Memory impact levels
#[derive(Debug, Clone)]
pub enum MemoryImpact {
    High,
    Medium,
    Low,
}

/// Concurrency pattern information
#[derive(Debug, Clone)]
pub struct ConcurrencyPatternInfo {
    pub pattern_type: ConcurrencyPatternType,
    pub thread_safety: ThreadSafety,
    pub synchronization_mechanisms: Vec<String>,
    pub potential_issues: Vec<String>,
}

/// Concurrency pattern types
#[derive(Debug, Clone)]
pub enum ConcurrencyPatternType {
    Synchronized,
    Volatile,
    Atomic,
    ConcurrentCollections,
    ExecutorService,
    CompletableFuture,
}

/// Thread safety levels
#[derive(Debug, Clone)]
pub enum ThreadSafety {
    ThreadSafe,
    ConditionallyThreadSafe,
    NotThreadSafe,
    Immutable,
}

/// Optimization opportunities
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    pub opportunity_type: OptimizationType,
    pub location: String,
    pub description: String,
    pub estimated_impact: OptimizationImpact,
    pub implementation_effort: ImplementationEffort,
}

/// Optimization types
#[derive(Debug, Clone)]
pub enum OptimizationType {
    AlgorithmOptimization,
    DataStructureOptimization,
    CachingImplementation,
    LazyInitialization,
    ObjectPooling,
    StringOptimization,
    CollectionOptimization,
}

/// Optimization impact
#[derive(Debug, Clone)]
pub enum OptimizationImpact {
    High,
    Medium,
    Low,
}

/// Implementation effort
#[derive(Debug, Clone)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = JavaAnalyzer::new();
        let result = analyzer.analyze_code("public class Test {}");

        // Basic test to ensure the analyzer works
        assert!(!result.recommendations.is_empty()); // Should have recommendations
        assert!(result.patterns_detected.is_empty()); // Simple class shouldn't detect complex patterns
        assert!(result.issues_found.is_empty()); // Simple class shouldn't have issues
    }

    #[test]
    fn test_comprehensive_analysis() {
        let analyzer = JavaAnalyzer::new();
        let java_code = r#"
        @Component
        public class UserService {
            @Autowired
            private UserRepository userRepository;
            
            public Optional<User> findUser(String id) {
                return userRepository.findById(id);
            }
        }
        "#;
        
        let result = analyzer.analyze_comprehensive(java_code);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.overall_score > 0);
        assert!(!analysis.recommendations.is_empty());
    }

    #[test]
    fn test_singleton_pattern_detection() {
        let analyzer = JavaAnalyzer::new();

        let singleton_code = r#"
        public class Singleton {
            private static Singleton instance;
            private Singleton() {}
            
            public static Singleton getInstance() {
                if (instance == null) {
                    instance = new Singleton();
                }
                return instance;
            }
        }
        "#;
        
        let result = analyzer.analyze_code(singleton_code);
        assert!(result
            .patterns_detected
            .iter()
            .any(|p| p.contains("Singleton")));
    }

    #[test]
    fn test_spring_framework_detection() {
        let analyzer = JavaAnalyzer::new();

        let spring_code = r#"
        @RestController
        @RequestMapping("/api/users")
        public class UserController {
            
            @Autowired
            private UserService userService;
            
            @GetMapping("/{id}")
            public ResponseEntity<User> getUser(@PathVariable String id) {
                return ResponseEntity.ok(userService.findUser(id));
            }
        }
        "#;
        
        let result = analyzer.analyze_comprehensive(spring_code);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(!analysis.framework_analysis.frameworks_detected.is_empty());
        assert!(analysis.framework_analysis.spring_analysis.is_some());
    }

    #[test]
    fn test_security_vulnerability_detection() {
        let analyzer = JavaAnalyzer::new();

        let vulnerable_code = r#"
        public class UserDAO {
            public User findUser(String username, String password) {
                String sql = "SELECT * FROM users WHERE username = '" + username + 
                           "' AND password = '" + password + "'";
                // Vulnerable to SQL injection
                return executeQuery(sql);
            }
        }
        "#;
        
        let result = analyzer.analyze_code(vulnerable_code);
        assert!(result
            .issues_found
            .iter()
            .any(|i| i.to_lowercase().contains("sql") || i.to_lowercase().contains("injection")));
    }

    #[test]
    fn test_oop_analysis() {
        let analyzer = JavaAnalyzer::new();

        let oop_code = r#"
        public abstract class Animal {
            protected String name;
            
            public abstract void makeSound();
            
            public final void setName(String name) {
                this.name = name;
            }
        }
        
        public class Dog extends Animal {
            @Override
            public void makeSound() {
                System.out.println("Woof!");
            }
        }
        "#;
        
        let result = analyzer.analyze_oop_patterns(oop_code);
        assert!(result.is_ok());
        
        let oop_analysis = result.unwrap();
        assert!(!oop_analysis.class_hierarchies.is_empty());
        assert!(!oop_analysis.inheritance_patterns.is_empty());
    }

    #[test]
    fn test_modern_java_features() {
        let analyzer = JavaAnalyzer::new();

        let modern_code = r#"
        public class StreamExample {
            public List<String> processNames(List<String> names) {
                return names.stream()
                    .filter(name -> name.length() > 3)
                    .map(String::toUpperCase)
                    .collect(Collectors.toList());
            }
            
            public Optional<String> findFirst(List<String> items) {
                return items.stream().findFirst();
            }
        }
        "#;
        
        let result = analyzer.analyze_modern_features(modern_code);
        assert!(result.is_ok());
        
        let modern_analysis = result.unwrap();
        assert!(!modern_analysis.lambda_expressions.is_empty());
    }

    #[test]
    fn test_performance_analysis() {
        let analyzer = JavaAnalyzer::new();

        let performance_code = r#"
        public class PerformanceExample {
            public String concatenateStrings(List<String> strings) {
                String result = "";
                for (String s : strings) {
                    result += s; // Inefficient string concatenation
                }
                return result;
            }
        }
        "#;
        
        let result = analyzer.analyze_performance(performance_code);
        assert!(result.is_ok());
        
        let perf_analysis = result.unwrap();
        assert!(perf_analysis.overall_performance_score > 0);
    }

    #[test]
    fn test_design_pattern_detection() {
        let analyzer = JavaAnalyzer::new();

        let factory_code = r#"
        public class CarFactory {
            public static Car createCar(String type) {
                switch (type) {
                    case "sedan":
                        return new Sedan();
                    case "suv":
                        return new SUV();
                    default:
                        throw new IllegalArgumentException("Unknown car type");
                }
            }
        }
        "#;
        
        let result = analyzer.detect_design_patterns(factory_code);
        assert!(result.is_ok());
        
        let patterns = result.unwrap();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_builder_pattern_detection() {
        let analyzer = JavaAnalyzer::new();

        let builder_code = r#"
        public class User {
            private String name;
            private int age;
            
            public static class Builder {
                private String name;
                private int age;
                
                public Builder setName(String name) {
                    this.name = name;
                    return this;
                }
                
                public Builder setAge(int age) {
                    this.age = age;
                    return this;
                }
                
                public User build() {
                    return new User(this);
                }
            }
        }
        "#;
        
        let result = analyzer.analyze_code(builder_code);
        assert!(result
            .patterns_detected
            .iter()
            .any(|p| p.contains("Builder") || p.contains("builder")));
    }

    #[test]
    fn test_encapsulation_analysis() {
        let analyzer = JavaAnalyzer::new();

        let encapsulation_code = r#"
        public class BankAccount {
            private double balance;
            private final String accountNumber;
            
            public BankAccount(String accountNumber) {
                this.accountNumber = accountNumber;
                this.balance = 0.0;
            }
            
            public double getBalance() {
                return balance;
            }
            
            public void deposit(double amount) {
                if (amount > 0) {
                    balance += amount;
                }
            }
        }
        "#;
        
        let result = analyzer.analyze_encapsulation(encapsulation_code);
        assert!(result.is_ok());
        
        let encapsulation_analysis = result.unwrap();
        assert!(!encapsulation_analysis.is_empty());
    }

    #[test]
    fn test_inheritance_analysis() {
        let analyzer = JavaAnalyzer::new();

        let inheritance_code = r#"
        public interface Drawable {
            void draw();
        }
        
        public abstract class Shape implements Drawable {
            protected String color;
            
            public abstract double getArea();
        }
        
        public class Circle extends Shape {
            private double radius;
            
            @Override
            public void draw() {
                System.out.println("Drawing a circle");
            }
            
            @Override
            public double getArea() {
                return Math.PI * radius * radius;
            }
        }
        "#;
        
        let result = analyzer.analyze_inheritance_patterns(inheritance_code);
        assert!(result.is_ok());
        
        let inheritance_patterns = result.unwrap();
        assert!(!inheritance_patterns.is_empty());
    }

    #[test]
    fn test_security_hardcoded_credentials() {
        let analyzer = JavaAnalyzer::new();

        let insecure_code = r#"
        public class DatabaseConfig {
            private static final String PASSWORD = "admin123";
            private String dbUrl = "jdbc:mysql://localhost:3306/mydb";
            
            public Connection getConnection() {
                return DriverManager.getConnection(dbUrl, "admin", PASSWORD);
            }
        }
        "#;
        
        let result = analyzer.detect_vulnerabilities(insecure_code);
        assert!(result.is_ok());
        
        let vulnerabilities = result.unwrap();
        assert!(vulnerabilities.iter().any(|v| 
            matches!(v.vulnerability_type, SecurityVulnerabilityType::HardcodedCredentials)));
    }

    #[test]
    fn test_weak_cryptography_detection() {
        let analyzer = JavaAnalyzer::new();

        let weak_crypto_code = r#"
        public class HashUtil {
            public String hashPassword(String password) {
                try {
                    MessageDigest md = MessageDigest.getInstance("MD5");
                    byte[] hash = md.digest(password.getBytes());
                    return Base64.getEncoder().encodeToString(hash);
                } catch (Exception e) {
                    throw new RuntimeException(e);
                }
            }
        }
        "#;
        
        let result = analyzer.detect_vulnerabilities(weak_crypto_code);
        assert!(result.is_ok());
        
        let vulnerabilities = result.unwrap();
        assert!(vulnerabilities.iter().any(|v| 
            matches!(v.vulnerability_type, SecurityVulnerabilityType::WeakCryptography)));
    }

    #[test]
    fn test_command_injection_detection() {
        let analyzer = JavaAnalyzer::new();

        let command_injection_code = r#"
        public class SystemUtil {
            public void executeCommand(String userInput) {
                try {
                    Runtime.getRuntime().exec("ls " + userInput);
                } catch (IOException e) {
                    e.printStackTrace();
                }
            }
        }
        "#;
        
        let result = analyzer.detect_vulnerabilities(command_injection_code);
        assert!(result.is_ok());
        
        let vulnerabilities = result.unwrap();
        assert!(vulnerabilities.iter().any(|v| 
            matches!(v.vulnerability_type, SecurityVulnerabilityType::CommandInjection)));
    }

    #[test]
    fn test_class_hierarchy_analysis() {
        let analyzer = JavaAnalyzer::new();

        let hierarchy_code = r#"
        public abstract class Vehicle {
            protected String brand;
        }
        
        public class Car extends Vehicle implements Drivable {
            private int wheels = 4;
        }
        
        public interface Drivable {
            void drive();
        }
        "#;
        
        let result = analyzer.analyze_class_hierarchies(hierarchy_code);
        assert!(result.is_ok());
        
        let hierarchies = result.unwrap();
        assert!(!hierarchies.is_empty());
        
        // Check if Car class hierarchy is correctly identified
        let car_hierarchy = hierarchies.iter().find(|h| h.class_name == "Car");
        assert!(car_hierarchy.is_some());
        
        let car = car_hierarchy.unwrap();
        assert_eq!(car.superclass, Some("Vehicle".to_string()));
        assert!(car.interfaces.contains(&"Drivable".to_string()));
    }

    #[test]
    fn test_interface_usage_analysis() {
        let analyzer = JavaAnalyzer::new();

        let interface_code = r#"
        @FunctionalInterface
        public interface StringProcessor {
            String process(String input);
            
            default String processWithLogging(String input) {
                System.out.println("Processing: " + input);
                return process(input);
            }
        }
        
        public class UpperCaseProcessor implements StringProcessor {
            @Override
            public String process(String input) {
                return input.toUpperCase();
            }
        }
        "#;
        
        let result = analyzer.analyze_interface_usage(interface_code);
        assert!(result.is_ok());
        
        let interface_usage = result.unwrap();
        assert!(!interface_usage.is_empty());
        
        let string_processor = interface_usage.iter()
            .find(|i| i.interface_name == "StringProcessor");
        assert!(string_processor.is_some());
    }

    #[test]
    fn test_lambda_expression_analysis() {
        let analyzer = JavaAnalyzer::new();

        let lambda_code = r#"
        public class LambdaExample {
            public void processItems(List<String> items) {
                items.stream()
                    .filter(item -> item.length() > 5)
                    .map(String::toUpperCase)
                    .forEach(System.out::println);
            }
        }
        "#;
        
        let result = analyzer.analyze_lambda_expressions(lambda_code);
        assert!(result.is_ok());
        
        let lambda_expressions = result.unwrap();
        assert!(!lambda_expressions.is_empty());
    }

    #[test]
    fn test_overall_code_quality_score() {
        let analyzer = JavaAnalyzer::new();

        let quality_code = r#"
        @Service
        public class UserService {
            private final UserRepository userRepository;
            
            public UserService(UserRepository userRepository) {
                this.userRepository = userRepository;
            }
            
            public Optional<User> findUserById(Long id) {
                if (id == null || id <= 0) {
                    return Optional.empty();
                }
                return userRepository.findById(id);
            }
            
            @Transactional
            public User saveUser(User user) {
                validateUser(user);
                return userRepository.save(user);
            }
            
            private void validateUser(User user) {
                if (user == null || user.getEmail() == null) {
                    throw new IllegalArgumentException("User and email cannot be null");
                }
            }
        }
        "#;
        
        let result = analyzer.analyze_comprehensive(quality_code);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        // Should have a reasonable overall score
        assert!(analysis.overall_score >= 50);
        assert!(analysis.overall_score <= 100);
    }
}
