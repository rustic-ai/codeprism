# analyze_code_quality Tool Design Document (Issue #181)

## Problem Statement

The current `analyze_code_quality` tool returns a placeholder "not_implemented" response instead of providing comprehensive code quality analysis. This tool needs to integrate multiple quality metrics including code smells, maintainability scoring, duplication analysis, documentation coverage, and technical debt assessment to provide actionable quality insights.

## Proposed Solution

### High-Level Approach

Implement a unified code quality analysis tool that combines multiple quality analysis techniques:
- **Code smell detection** (long methods, god classes, feature envy, etc.)
- **Maintainability scoring** using established metrics
- **Duplication percentage** calculation and detection
- **Documentation coverage** analysis
- **Technical debt assessment** with remediation guidance
- **Naming convention** analysis and compliance checking

### Component Architecture

```rust
// Parameter structure for configurable analysis
pub struct AnalyzeCodeQualityParams {
    pub target: String,                          // File path, symbol ID, or glob pattern
    pub quality_types: Option<Vec<String>>,      // Types of quality analysis to perform
    pub severity_threshold: Option<String>,      // Minimum severity level to report
    pub include_recommendations: Option<bool>,   // Include improvement recommendations
    pub detailed_analysis: Option<bool>,         // Include detailed analysis breakdown
}

// Main analysis orchestrator
impl CodePrismMcpServer {
    fn analyze_code_quality_comprehensive(
        &self,
        target: &str,
        quality_types: &[String],
        severity_threshold: &str,
        include_recommendations: bool,
        detailed_analysis: bool,
    ) -> Result<serde_json::Value>;
}
```

### Quality Analysis Types

1. **code_smells** - Detect common code smell patterns
2. **maintainability** - Calculate maintainability index and factors  
3. **duplication** - Find duplicate code patterns and calculate percentage
4. **documentation** - Analyze documentation coverage and quality
5. **naming_conventions** - Check adherence to naming standards
6. **technical_debt** - Assess and quantify technical debt
7. **all** - Perform comprehensive analysis across all categories

### API Design

#### Input Parameters
- `target`: Target for analysis (file path, symbol ID, or glob pattern)
- `quality_types`: Array of quality analysis types (default: ["all"])
- `severity_threshold`: Minimum severity ("low", "medium", "high", "critical")
- `include_recommendations`: Include actionable recommendations (default: true)
- `detailed_analysis`: Include detailed breakdown (default: false)

#### Output Format
```json
{
  "status": "success",
  "target": "src/main.rs",
  "analysis_type": "comprehensive",
  "quality_metrics": {
    "overall_score": 7.8,
    "maintainability_index": 72.5,
    "technical_debt_ratio": 12.3,
    "documentation_coverage": 76.3
  },
  "code_smells": {
    "total_count": 14,
    "by_severity": {
      "critical": 0,
      "high": 2,
      "medium": 7,
      "low": 5
    },
    "by_category": {
      "long_methods": 3,
      "god_classes": 1,
      "feature_envy": 2,
      "data_clumps": 1,
      "primitive_obsession": 4,
      "large_parameter_lists": 3
    },
    "detailed_issues": [...]
  },
  "duplication_analysis": {
    "percentage": 3.2,
    "duplicate_blocks": 8,
    "similar_blocks": 12,
    "affected_files": 6
  },
  "naming_analysis": {
    "compliance_score": 89.2,
    "violations": 15,
    "conventions_checked": ["camelCase", "PascalCase", "snake_case"]
  },
  "recommendations": [...],
  "settings": {...}
}
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. **Add parameter structure** for configurable quality analysis
2. **Create quality analysis orchestrator** that coordinates different analysis types
3. **Implement helper methods** for each quality analysis category
4. **Add error handling** and validation for all analysis types

### Phase 2: Code Smell Detection  
1. **Long method detection** - Methods exceeding line/complexity thresholds
2. **God class detection** - Classes with too many responsibilities
3. **Feature envy detection** - Methods using another class's data excessively
4. **Data clumps detection** - Groups of data that appear together frequently
5. **Primitive obsession detection** - Overuse of primitive types
6. **Large parameter lists** - Methods with excessive parameters

### Phase 3: Maintainability Analysis
1. **Maintainability index calculation** using Halstead metrics and complexity
2. **Cyclomatic complexity integration** with existing analysis
3. **Cognitive complexity assessment** for readability
4. **Technical debt scoring** based on code smells and complexity

### Phase 4: Duplication & Documentation
1. **Code duplication detection** using AST similarity analysis
2. **Documentation coverage analysis** for public APIs
3. **Naming convention checking** based on language standards
4. **Integration with existing GraphStore** for cross-reference analysis

### Phase 5: Recommendations & Reporting
1. **Actionable recommendations** generation based on analysis results
2. **Severity-based filtering** and prioritization
3. **Detailed analysis breakdowns** for comprehensive reporting
4. **Integration with guidance system** for improvement workflows

## Testing Strategy

### Unit Tests
- Test each quality analysis method independently
- Verify parameter validation and error handling
- Test scoring calculations and threshold filtering
- Validate JSON response format and structure

### Integration Tests  
- Test with real codebase files and patterns
- Verify GraphStore integration for cross-references
- Test glob pattern handling for repository-wide analysis
- Validate performance with large codebases

### Property-Based Tests
- Test consistency of quality scores across similar code
- Verify monotonic behavior of severity thresholds
- Test duplication detection with known duplicate patterns
- Validate naming convention detection accuracy

## Success Criteria

### Functional Requirements
- [ ] Replaces placeholder with real analysis functionality
- [ ] Supports all quality analysis types (smells, maintainability, duplication, docs)
- [ ] Provides configurable severity thresholds and analysis scope
- [ ] Generates actionable improvement recommendations
- [ ] Integrates with existing complexity and performance analysis

### Quality Requirements
- [ ] Comprehensive unit test coverage (>90%)
- [ ] Integration with existing GraphStore and analysis infrastructure
- [ ] Performance optimization for large repositories
- [ ] Accurate detection of code quality issues with minimal false positives
- [ ] Clear, actionable recommendations with improvement guidance

### Integration Requirements
- [ ] Compatible with existing MCP tool parameter patterns
- [ ] Integrates with CodePrismMcpServer architecture
- [ ] Uses existing CodeAnalyzer and GraphStore infrastructure
- [ ] Follows established error handling and response formatting

## Performance Considerations

- **Caching analysis results** for repeated queries on same targets
- **Incremental analysis** for large repositories using file modification tracking
- **Parallel processing** for independent quality analysis types
- **Memory-efficient AST traversal** for duplication detection
- **Configurable analysis depth** to balance thoroughness vs. performance

## Security Considerations

- **Input validation** for all target paths and parameters
- **Safe file system access** with proper path traversal protection
- **Resource limits** to prevent denial-of-service through large analysis requests
- **Sanitized output** to prevent injection through error messages or file paths

## Alternative Approaches Considered

### Approach A: Separate Tools for Each Quality Type
**Trade-offs**: More granular control but increased complexity and API surface
**Why not chosen**: Users typically want comprehensive quality overview, not isolated metrics

### Approach B: External Tool Integration (SonarQube, CodeClimate)
**Trade-offs**: Mature analysis but external dependencies and setup complexity  
**Why not chosen**: Maintains self-contained analysis within CodePrism ecosystem

### Approach C: Statistical Analysis Only
**Trade-offs**: Faster execution but less actionable insights
**Why not chosen**: Users need specific, actionable recommendations for code improvement

## Dependencies

- **CodePrism Core**: GraphStore, NodeId, Graph traversal
- **CodePrism Analysis**: SecurityAnalyzer, PerformanceAnalyzer integration
- **Existing Infrastructure**: ContentSearchManager, RepositoryScanner
- **External Crates**: AST parsing libraries for duplication detection 