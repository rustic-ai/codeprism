# Issue #194: Mandrel MCP Test Harness Reporting System Design Document

## Problem Statement

The Mandrel MCP Test Harness (`moth` binary) currently lacks a comprehensive reporting system to present test results in structured, analyzable formats. While the executor module has rich `TestResult` structures, there's no way to:

- Generate reports for CI/CD integration (JUnit XML)
- Create human-readable reports (HTML/Markdown)
- Provide machine-readable outputs (JSON)
- Aggregate and filter test results
- Track performance metrics over time

This is critical for production usage of the test harness, as teams need to integrate results into their development workflows and CI/CD pipelines.

## Proposed Solution

Create a comprehensive reporting system built on existing executor structures using mature Rust ecosystem libraries:

### High-Level Architecture

```
Test Execution Results (TestResult, SuiteResult)
    ↓
Report Generator (ReportGenerator)
    ↓
Multiple Output Formats
    ├── JSON (serde_json)
    ├── JUnit XML (quick-junit)
    ├── HTML (tera templates)
    └── Markdown (pulldown-cmark)
```

### Core Components

```rust
// Core reporting structures
pub struct TestReport {
    pub metadata: ReportMetadata,
    pub summary: ExecutionSummary,
    pub test_results: Vec<TestResult>,
    pub server_info: ServerInfo,
    pub validation_details: Vec<ValidationDetail>,
    pub performance_metrics: PerformanceReport,
}

pub struct ReportMetadata {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub mandrel_version: String,
    pub mcp_protocol_version: String,
    pub environment: EnvironmentInfo,
}

pub struct ExecutionSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration: Duration,
    pub success_rate: f64,
}

pub struct PerformanceReport {
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub throughput: f64,
    pub memory_usage: MemoryStats,
}

// Main report generator
pub struct ReportGenerator {
    template_engine: Tera,
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn generate_json(&self, results: &SuiteResult) -> Result<String, ReportError>;
    pub fn generate_junit_xml(&self, results: &SuiteResult) -> Result<String, ReportError>;
    pub fn generate_html(&self, results: &SuiteResult) -> Result<String, ReportError>;
    pub fn generate_markdown(&self, results: &SuiteResult) -> Result<String, ReportError>;
}
```

### Output Formats

1. **JSON Report** - Machine-readable, structured data
2. **JUnit XML** - CI/CD integration (Jenkins, GitLab CI, GitHub Actions)
3. **HTML Report** - Human-readable with charts and filtering
4. **Markdown Report** - Documentation integration

### Library Dependencies

- `quick-junit` - Industry-standard JUnit XML generation
- `tera` - Powerful template engine for HTML generation
- `serde_json` - JSON serialization
- `pulldown-cmark` - Markdown processing
- `chrono` - Date/time handling (already included)

## Implementation Plan

### Phase 1: Core Report Structures (TDD)
1. **RED**: Write failing tests for `TestReport` serialization
2. **GREEN**: Implement basic report structures with JSON serialization
3. **REFACTOR**: Optimize structure design based on test feedback

### Phase 2: JSON Output Generation (TDD)
1. **RED**: Write comprehensive tests for JSON report generation
2. **GREEN**: Implement `generate_json()` method with proper formatting
3. **REFACTOR**: Add report filtering and aggregation capabilities

### Phase 3: JUnit XML Integration (TDD)
1. **RED**: Write tests for JUnit XML output validation
2. **GREEN**: Integrate `quick-junit` library for XML generation
3. **REFACTOR**: Ensure CI/CD compatibility with major platforms

### Phase 4: HTML Template System (TDD)
1. **RED**: Write tests for HTML report generation and template rendering
2. **GREEN**: Create Tera templates and implement HTML generation
3. **REFACTOR**: Add interactive features and responsive design

### Phase 5: Performance Metrics (TDD)
1. **RED**: Write tests for performance data collection and reporting
2. **GREEN**: Implement performance tracking and aggregation
3. **REFACTOR**: Add trend analysis and benchmarking features

### Phase 6: Integration and CLI (TDD)
1. **RED**: Write integration tests for CLI report generation
2. **GREEN**: Integrate reporting system with `moth` CLI
3. **REFACTOR**: Add configuration options and output customization

## API Design

### Report Generation API

```rust
// Primary interface for report generation
impl ReportGenerator {
    pub fn new(config: ReportConfig) -> Result<Self, ReportError>;
    
    pub fn generate_report(
        &self,
        results: &SuiteResult,
        format: OutputFormat,
        output_path: Option<&Path>,
    ) -> Result<GeneratedReport, ReportError>;
    
    pub fn generate_multiple_formats(
        &self,
        results: &SuiteResult,
        formats: &[OutputFormat],
        output_dir: &Path,
    ) -> Result<Vec<GeneratedReport>, ReportError>;
}

// Output format selection
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    JunitXml,
    Html { template: Option<String> },
    Markdown { style: MarkdownStyle },
}

// Configuration for report generation
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub include_performance_metrics: bool,
    pub include_validation_details: bool,
    pub html_template_dir: Option<PathBuf>,
    pub custom_fields: HashMap<String, String>,
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Template rendering failed: {0}")]
    TemplateError(#[from] tera::Error),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("JUnit XML generation failed: {0}")]
    JunitError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### CLI Integration

```rust
// CLI command structure
#[derive(clap::Parser)]
pub struct ReportCommand {
    /// Input test results file
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output format(s)
    #[arg(short, long, value_enum)]
    format: Vec<OutputFormat>,
    
    /// Output directory
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    
    /// Include performance metrics
    #[arg(long)]
    include_performance: bool,
    
    /// Custom template directory
    #[arg(long)]
    template_dir: Option<PathBuf>,
}
```

## Testing Strategy

### Unit Tests (90% Coverage Target)
- **Report Structure Tests**: Serialization/deserialization of all report types
- **JSON Generation Tests**: Validate JSON output format and structure
- **JUnit XML Tests**: Validate XML schema compliance and CI/CD compatibility
- **HTML Generation Tests**: Template rendering and content validation
- **Performance Metrics Tests**: Calculation accuracy and aggregation logic
- **Error Handling Tests**: All error scenarios and edge cases

### Integration Tests
- **End-to-End Report Generation**: Full workflow from `SuiteResult` to output files
- **CLI Integration Tests**: Command-line interface functionality
- **Template System Tests**: Custom template loading and rendering
- **Multi-Format Generation**: Simultaneous generation of multiple formats

### Performance Tests
- **Large Dataset Handling**: Report generation with 1000+ test results
- **Memory Usage Validation**: Ensure reasonable memory consumption
- **Generation Speed**: Sub-second report generation for typical workloads

### Test Data Fixtures
```rust
// Comprehensive test fixtures
fn create_comprehensive_test_suite() -> SuiteResult {
    SuiteResult {
        metadata: /* ... */,
        results: vec![
            create_passing_test_result(),
            create_failing_test_result(),
            create_skipped_test_result(),
            create_performance_test_result(),
        ],
        performance_metrics: create_performance_metrics(),
        validation_summary: create_validation_summary(),
    }
}
```

## Alternatives Considered

### Alternative 1: Simple JSON-Only Output
- **Pros**: Minimal implementation effort, lightweight
- **Cons**: Limited CI/CD integration, no human-readable reports
- **Rejected**: Insufficient for production usage requirements

### Alternative 2: Custom Template System
- **Pros**: Full control over templating
- **Cons**: Reinventing the wheel, maintenance burden
- **Rejected**: `tera` is mature, well-tested, and feature-complete

### Alternative 3: XML-First Approach
- **Pros**: Strong CI/CD integration
- **Cons**: Poor human readability, limited format flexibility
- **Rejected**: Multi-format approach provides better value

## Success Criteria

### Functional Requirements
- [ ] Generate valid JSON reports with comprehensive test result data
- [ ] Generate JUnit XML compatible with major CI/CD platforms
- [ ] Generate HTML reports with interactive filtering and visualization
- [ ] Generate Markdown reports suitable for documentation
- [ ] Support report aggregation and filtering capabilities
- [ ] Handle large test suites (1000+ tests) efficiently

### Quality Requirements
- [ ] 90%+ code coverage with comprehensive unit tests
- [ ] Sub-second report generation for typical workloads (<100 tests)
- [ ] Memory usage <50MB for large test suites (1000+ tests)
- [ ] All output formats validate against their respective schemas
- [ ] Comprehensive error handling with actionable error messages

### Integration Requirements
- [ ] Seamless integration with existing `moth` CLI
- [ ] Compatible with major CI/CD platforms (GitHub Actions, Jenkins, GitLab)
- [ ] Extensible template system for custom HTML reports
- [ ] Configuration through CLI arguments and config files

### Performance Benchmarks
- **JSON Generation**: <100ms for 100 test results
- **JUnit XML Generation**: <200ms for 100 test results
- **HTML Generation**: <500ms for 100 test results (including template rendering)
- **Memory Usage**: <10MB for 100 test results, <50MB for 1000 test results

## Implementation Dependencies

### New Cargo.toml Dependencies
```toml
[dependencies]
# Reporting and templating
quick-junit = "0.5"
tera = "1.19"
pulldown-cmark = "0.10"
```

### Template Assets
- HTML templates in `templates/` directory
- CSS/JS assets for interactive features
- Default themes and styling

### Configuration Files
- Default report configuration
- Template customization options
- CI/CD integration examples

## Breaking Changes

**None Expected** - This is a new feature addition that extends existing functionality without modifying current APIs.

## Future Enhancements

1. **Interactive Web Dashboard**: Real-time test result visualization
2. **Trend Analysis**: Historical performance tracking
3. **Custom Plugins**: Extensible report generation system
4. **Cloud Integration**: Direct upload to cloud reporting services
5. **Advanced Analytics**: Statistical analysis of test patterns

---

This design provides a comprehensive foundation for implementing a professional-grade reporting system that meets current needs while remaining extensible for future requirements. 