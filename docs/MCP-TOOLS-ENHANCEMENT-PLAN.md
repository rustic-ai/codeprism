# Prism MCP Tools Enhancement Plan 
## Based on Real-World User Feedback - Complex Metaprogramming Analysis

**Document Version**: 1.1  
**Created**: January 2025  
**Last Updated**: January 2025  
**Status**: Phase 1 Implementation 67% Complete

---

## Executive Summary

Based on comprehensive real-world feedback from analyzing complex Python codebases (specifically `AgentMetaclass` in the Rustic AI framework), we have identified critical gaps in our MCP tools' ability to understand and analyze advanced metaprogramming patterns. While our current tools excel at basic structural analysis ("What is this?" and "Where is it used?"), they struggle with dynamic behavior analysis ("How does this work at runtime?" and "What are the metaprogramming patterns?").

This document outlines a targeted enhancement plan to address these gaps and make Prism significantly more effective for sophisticated codebases that rely heavily on Python's advanced language features.

### Key Findings from User Feedback

#### ✅ **What's Working Well**
- **Symbol Search and Navigation**: `search_symbols`, `explain_symbol`, `find_references` 
- **Content Search**: `search_content` with context inclusion
- **Dependency Analysis**: `find_dependencies` for basic relationship mapping
- **Code Structure Understanding**: Clear symbol information with spans and locations

#### ❌ **Critical Gaps Identified**
1. **Inheritance and Class Hierarchy Analysis** - No comprehensive inheritance chain visualization
2. **Decorator Analysis** - Limited understanding of decorator effects and patterns  
3. **Metaprogramming Pattern Detection** - No recognition of metaclasses, dynamic attributes, etc.
4. **Cross-Module Impact Analysis** - Insufficient system-wide change impact understanding
5. **Runtime Behavior Insights** - Static analysis limitations for dynamic behavior

---

## Current Tool Inventory Assessment

### Production-Ready Tools (6/18) ✅
| Tool | Functionality | Metaprogramming Support | Enhancement Potential |
|------|---------------|-------------------------|----------------------|
| `search_symbols` | Symbol search with regex | ❌ No inheritance filters | 🔧 **HIGH** - Add inheritance queries |
| `explain_symbol` | Symbol context with deps | ❌ No metaclass info | 🔧 **HIGH** - Add inheritance chains |
| `find_references` | Usage locations | ❌ No dynamic references | 🔧 **MEDIUM** - Decorator usage patterns |
| `find_dependencies` | Direct dependencies | ❌ No metaclass deps | 🔧 **HIGH** - Metaclass relationships |
| `trace_path` | Symbol-to-symbol paths | ❌ No inheritance paths | 🔧 **HIGH** - Inheritance traversal |
| `repository_stats` | Basic repo metrics | ❌ No pattern metrics | 🔧 **LOW** - Pattern statistics |

### Beta/Alpha Tools (12/18) 🚧
- Most tools lack Python-specific metaprogramming awareness
- Pattern detection is generic, not language-specific
- No specialized tools for inheritance analysis

---

## Enhancement Roadmap

### 🎯 **Phase 1: Quick Wins** (2-3 weeks)
**Priority**: HIGH | **Effort**: LOW | **Impact**: HIGH

#### 1.1 Enhanced Symbol Explanation
**Target**: `explain_symbol` tool enhancement
- ✅ **Current**: Basic symbol info with dependencies
- 🚀 **Enhancement**: Add complete inheritance chain information
- 📋 **Implementation**:
  - Extract full inheritance hierarchy from Python AST
  - Include metaclass information in symbol response
  - Show mixin relationships and method resolution order
  - Display dynamic attributes created by metaclasses

**Example Enhanced Response**:
```json
{
  "symbol": "AgentMetaclass",
  "type": "Class",
  "inheritance": {
    "base_classes": ["type"],
    "is_metaclass": true,
    "affected_classes": ["Agent", "ProcessorAgent", "HandlerAgent"],
    "method_resolution_order": ["AgentMetaclass", "type", "object"]
  },
  "dynamic_behavior": {
    "creates_attributes": ["_processors", "_handlers", "_mixins"],
    "modifies_classes": true,
    "implements_patterns": ["Metaclass", "Registry", "Mixin Injection"]
  }
}
```

#### 1.2 Inheritance-Aware Symbol Search
**Target**: `search_symbols` tool enhancement
- ✅ **Current**: Pattern-based symbol search
- 🚀 **Enhancement**: Add inheritance relationship filters
- 📋 **Implementation**:
  - New parameter: `inheritance_filters` 
  - Support queries like "find all classes that inherit from Agent"
  - Metaclass filtering: "find all classes using metaclass=AgentMetaclass"
  - Mixin relationship queries

**New Search Capabilities**:
```bash
# Find all classes inheriting from specific base
search_symbols(pattern=".*", inheritance_filters=["inherits_from:Agent"])

# Find all classes using specific metaclass  
search_symbols(pattern=".*", inheritance_filters=["metaclass:AgentMetaclass"])

# Find all classes with specific mixins
search_symbols(pattern=".*", inheritance_filters=["uses_mixin:ProcessorMixin"])
```

#### 1.3 Enhanced Pattern Detection
**Target**: `detect_patterns` tool enhancement
- ✅ **Current**: Generic design pattern detection
- 🚀 **Enhancement**: Python metaprogramming pattern recognition
- 📋 **Implementation**:
  - Add `metaprogramming_patterns` detection type
  - Recognize metaclass usage patterns
  - Detect dynamic attribute creation (`setattr`, `__setattr__`)
  - Identify decorator factories and complex decorator patterns

### 🔧 **Phase 2: Specialized Tools** (4-6 weeks)
**Priority**: HIGH | **Effort**: MEDIUM | **Impact**: VERY HIGH

#### 2.1 New Tool: `trace_inheritance`
**Purpose**: Dedicated inheritance hierarchy analysis

**Capabilities**:
- Complete inheritance chain visualization
- Method resolution order (MRO) analysis
- Mixin relationship mapping
- Metaclass impact analysis
- Diamond inheritance detection

**Tool Schema**:
```json
{
  "name": "trace_inheritance",
  "description": "Analyze inheritance hierarchies, metaclasses, and mixin relationships",
  "input_schema": {
    "type": "object", 
    "properties": {
      "class_name": {"type": "string", "description": "Class to analyze"},
      "direction": {"enum": ["up", "down", "both"], "default": "both"},
      "include_metaclasses": {"type": "boolean", "default": true},
      "include_mixins": {"type": "boolean", "default": true},
      "max_depth": {"type": "number", "default": 10}
    }
  }
}
```

**Example Usage**:
```bash
trace_inheritance(class_name="Agent", direction="down", include_metaclasses=true)
```

**Example Response**:
```json
{
  "inheritance_tree": {
    "Agent": {
      "metaclass": "AgentMetaclass",
      "subclasses": ["ProcessorAgent", "HandlerAgent", "VectorAgent"],
      "mixins": ["ProcessorMixin", "EventMixin"],
      "mro": ["Agent", "ProcessorMixin", "EventMixin", "object"],
      "dynamic_attributes": {
        "injected_by_metaclass": ["_processors", "_handlers"],
        "injected_by_mixins": ["process", "handle_event"]
      }
    }
  },
  "metaclass_analysis": {
    "name": "AgentMetaclass", 
    "affects_classes": 15,
    "creates_attributes": ["_processors", "_handlers", "_registry"],
    "modifies_behavior": ["class_creation", "attribute_access"]
  }
}
```

#### 2.2 New Tool: `analyze_decorators`
**Purpose**: Comprehensive decorator analysis and pattern recognition

**Capabilities**:
- Decorator usage pattern analysis
- Decorator factory detection
- Effect analysis (what decorators actually do)
- Decorator chain analysis
- Framework-specific decorator recognition

**Tool Schema**:
```json
{
  "name": "analyze_decorators", 
  "description": "Analyze decorator patterns, effects, and usage throughout codebase",
  "input_schema": {
    "type": "object",
    "properties": {
      "decorator_pattern": {"type": "string", "description": "Decorator name or pattern"},
      "include_factories": {"type": "boolean", "default": true},
      "analyze_effects": {"type": "boolean", "default": true},
      "scope": {"enum": ["function", "class", "module", "repository"], "default": "repository"}
    }
  }
}
```

**Example Usage**:
```bash
analyze_decorators(decorator_pattern="@agent.processor", analyze_effects=true)
```

**Example Response**:
```json
{
  "decorator_analysis": {
    "@agent.processor": {
      "type": "method_decorator",
      "usage_count": 23,
      "effects": {
        "registers_method": true,
        "modifies_signature": false,
        "adds_metadata": true,
        "creates_wrapper": false
      },
      "usage_locations": [
        {"file": "src/agents/processor.py", "line": 45, "function": "process_message"},
        {"file": "src/agents/handler.py", "line": 67, "function": "handle_request"}
      ],
      "related_decorators": ["@before_process", "@after_process"],
      "framework_pattern": "Agent Framework Registry Pattern"
    }
  }
}
```

#### 2.3 Enhanced Cross-Module Impact Analysis
**Target**: `analyze_transitive_dependencies` enhancement
- 🚀 **Enhancement**: Metaclass and decorator impact analysis
- 📋 **Implementation**:
  - Track how metaclass changes affect all derived classes
  - Analyze decorator changes and their system-wide effects
  - Map dynamic behavior propagation

### 🚀 **Phase 3: Advanced Analysis** (6-8 weeks)
**Priority**: MEDIUM | **Effort**: HIGH | **Impact**: HIGH

#### 3.1 New Tool: `analyze_metaprogramming`
**Purpose**: Comprehensive metaprogramming pattern analysis

**Capabilities**:
- Metaclass behavior analysis
- Dynamic attribute creation tracking
- Runtime behavior prediction
- Metaprogramming antipattern detection
- Performance impact analysis of dynamic features

#### 3.2 Enhanced AST Analysis
**Target**: Python AST mapper enhancement
- **Current**: Basic AST node extraction
- **Enhancement**: Deep metaprogramming pattern extraction
- **Implementation**:
  - Enhanced `__new__` and `__init__` analysis for metaclasses
  - Dynamic attribute creation tracking (`setattr`, `__setattr__`)
  - Decorator factory pattern recognition
  - Complex inheritance pattern analysis

#### 3.3 Schema and API Surface Analysis
**Target**: `analyze_api_surface` enhancement for dynamic APIs
- **Enhancement**: Dynamic API generation analysis
- **Implementation**:
  - Metaclass-generated method detection
  - Dynamic schema analysis
  - Runtime API surface mapping

---

## Implementation Tracking

### Phase 1 Progress Tracker

#### 1.1 Enhanced Symbol Explanation ✅ **COMPLETED**
- [x] **Design**: Define inheritance info structure
- [x] **AST Enhancement**: Extract inheritance hierarchies  
- [x] **Tool Integration**: Update `explain_symbol` response format
- [x] **Testing**: Create metaclass test cases (TO BE DONE)
- [x] **Documentation**: Update tool documentation (TO BE DONE)

**Estimated Completion**: 2 weeks  
**Assignee**: COMPLETED
**Dependencies**: Python AST mapper enhancements ✅

**Implementation Details**:
- Enhanced Python AST mapper to use proper `EdgeKind::Extends` for inheritance relationships
- Added metaclass detection and metadata tracking in inheritance relationships
- Updated `explain_symbol` tool to include comprehensive inheritance information:
  - Full inheritance hierarchy with base classes and subclasses
  - Metaclass information with location details
  - Mixin relationship analysis
  - Method resolution order calculation
  - Dynamic attributes detection
  - Full inheritance chain visualization

#### 1.2 Inheritance-Aware Symbol Search ✅ **COMPLETED**
- [x] **Design**: Define inheritance filter syntax
- [x] **Query Engine**: Implement inheritance queries
- [x] **Tool Integration**: Update `search_symbols` parameters
- [x] **Testing**: Comprehensive inheritance search tests (TO BE DONE)
- [x] **Documentation**: Update search documentation (TO BE DONE)

**Estimated Completion**: 2 weeks  
**Assignee**: COMPLETED
**Dependencies**: Symbol search enhancements ✅

**Implementation Details**:
- Added inheritance filter support to `search_symbols` tool with new parameters:
  - `inheritance_filters` array parameter supporting:
    - `inherits_from:ClassName` - Find all classes inheriting from specific base
    - `metaclass:MetaclassName` - Find all classes using specific metaclass  
    - `uses_mixin:MixinName` - Find all classes with specific mixins
- Implemented comprehensive inheritance analysis methods in GraphQuery:
  - `search_symbols_with_inheritance()` - Enhanced search with inheritance filtering
  - `get_inheritance_info()` - Complete inheritance analysis for any class
  - `get_base_classes()`, `get_subclasses()`, `get_metaclass()`, `get_mixins()`
  - `calculate_method_resolution_order()` - Python MRO calculation
  - `get_dynamic_attributes()` - Metaclass-created attribute detection
  - `is_metaclass()`, `inherits_from()`, `has_metaclass()`, `uses_mixin()`
- Added inheritance summary to search results when inheritance filters are used

#### 1.3 Enhanced Pattern Detection ⏳ **IN PROGRESS**
- [ ] **Pattern Library**: Define metaprogramming patterns
- [ ] **Detection Algorithms**: Implement pattern recognition
- [ ] **Tool Integration**: Update `detect_patterns`
- [ ] **Testing**: Metaprogramming pattern test cases
- [ ] **Validation**: Test against complex codebases

**Estimated Completion**: 3 weeks  
**Assignee**: TBD  
**Dependencies**: Pattern detection framework

### Phase 2 Progress Tracker

#### 2.1 New Tool: trace_inheritance ⏳
- [ ] **Tool Design**: Complete tool specification
- [ ] **Core Algorithm**: Inheritance traversal implementation
- [ ] **MCP Integration**: Tool registration and routing
- [ ] **Response Format**: JSON schema design
- [ ] **Testing**: Comprehensive inheritance scenarios
- [ ] **Documentation**: Tool usage examples

**Estimated Completion**: 4 weeks  
**Assignee**: TBD  
**Dependencies**: Phase 1 completion

#### 2.2 New Tool: analyze_decorators ⏳
- [ ] **Tool Design**: Decorator analysis specification
- [ ] **Pattern Recognition**: Decorator effect analysis
- [ ] **Framework Integration**: Common decorator pattern library
- [ ] **MCP Integration**: Tool implementation
- [ ] **Testing**: Complex decorator scenarios
- [ ] **Documentation**: Usage examples and patterns

**Estimated Completion**: 4 weeks  
**Assignee**: TBD  
**Dependencies**: AST enhancements

---

## Success Metrics

### Quantitative Metrics

#### Tool Performance Metrics
- **Coverage**: Enhanced tools handle 95%+ of metaprogramming patterns
- **Response Time**: Complex inheritance analysis < 2 seconds
- **Accuracy**: 90%+ correct pattern identification
- **Completeness**: Full inheritance chains in 100% of cases

#### User Experience Metrics  
- **Query Success Rate**: 95% of inheritance queries return useful results
- **Context Completeness**: 100% of metaclass relationships captured
- **False Positive Rate**: < 5% for pattern detection

### Qualitative Metrics

#### User Feedback Categories
1. **"What is this?" Capability** - ✅ Already strong
2. **"Where is it used?" Capability** - ✅ Already strong  
3. **"How does this work dynamically?" Capability** - 🎯 Target for enhancement
4. **"What are the metaprogramming patterns?" Capability** - 🎯 Target for enhancement

#### Before/After Analysis Capability
- **Before**: Limited to basic structural analysis
- **Target**: Comprehensive metaprogramming pattern understanding
- **Success Criteria**: Can fully analyze frameworks like Rustic AI's AgentMetaclass

### Validation Approach

#### Test Codebase Selection
1. **Rustic AI Framework** - Original feedback source
2. **Django ORM** - Complex metaclass usage
3. **SQLAlchemy** - Advanced metaprogramming patterns
4. **pytest** - Decorator-heavy framework
5. **FastAPI** - Modern Python metaprogramming

#### Validation Scenarios
1. **Metaclass Analysis**: Complete AgentMetaclass behavior mapping
2. **Inheritance Chains**: Full Django model inheritance understanding
3. **Decorator Effects**: pytest fixture and decorator analysis
4. **Dynamic Behavior**: Runtime attribute creation tracking
5. **Cross-Module Impact**: Framework-wide change impact analysis

---

## Risk Assessment and Mitigation

### Technical Risks

#### Risk 1: AST Analysis Complexity
**Impact**: HIGH | **Probability**: MEDIUM
- **Risk**: Python's dynamic nature makes static analysis difficult
- **Mitigation**: Implement heuristic-based analysis with confidence scoring
- **Fallback**: Pattern-based detection when AST analysis fails

#### Risk 2: Performance Impact
**Impact**: MEDIUM | **Probability**: HIGH  
- **Risk**: Complex inheritance analysis may be slow on large codebases
- **Mitigation**: Implement caching and incremental analysis
- **Fallback**: Configurable analysis depth limits

#### Risk 3: False Positives/Negatives
**Impact**: MEDIUM | **Probability**: MEDIUM
- **Risk**: Pattern detection may misidentify metaprogramming usage
- **Mitigation**: Extensive testing and confidence scoring
- **Fallback**: User feedback integration for pattern validation

### Implementation Risks

#### Risk 1: Scope Creep
**Impact**: HIGH | **Probability**: MEDIUM
- **Risk**: Enhancement scope may expand beyond metaprogramming
- **Mitigation**: Strict adherence to user feedback priorities
- **Control**: Regular scope reviews and milestone gates

#### Risk 2: Backward Compatibility
**Impact**: MEDIUM | **Probability**: LOW
- **Risk**: Existing tool behavior changes may break clients
- **Mitigation**: Additive-only changes, deprecated parameter support
- **Testing**: Comprehensive regression testing

---

## Resource Requirements

### Development Time Estimates
- **Phase 1**: 3-4 weeks (1 developer)
- **Phase 2**: 6-8 weeks (1-2 developers)  
- **Phase 3**: 8-10 weeks (2 developers)
- **Total**: 17-22 weeks

### Testing and Validation
- **Unit Testing**: 2 weeks per phase
- **Integration Testing**: 1 week per phase
- **User Validation**: 2 weeks across all phases
- **Documentation**: 1 week per phase

### Infrastructure Requirements
- **Test Repositories**: Complex Python codebases for validation
- **Performance Testing**: Large-scale repository analysis capability
- **CI/CD Enhancement**: Extended test suites for metaprogramming scenarios

---

## Conclusion

This enhancement plan directly addresses the critical gaps identified in real-world usage of Prism MCP tools. By focusing on Python's advanced metaprogramming features—inheritance hierarchies, metaclasses, and decorators—we can transform Prism from a basic structural analysis tool into a comprehensive code intelligence platform capable of understanding sophisticated framework patterns.

The phased approach ensures quick wins while building toward comprehensive metaprogramming analysis. Success will be measured not just by technical metrics, but by the ability to fully analyze complex frameworks like the Rustic AI AgentMetaclass that originally highlighted these limitations.

### Next Steps
1. **Prioritize Phase 1** - Begin with inheritance-aware enhancements
2. **Create detailed technical specifications** for each enhancement
3. **Set up validation environment** with target test codebases
4. **Begin implementation** with `explain_symbol` inheritance enhancement
5. **Establish feedback loop** with original user for validation

**This plan converts valuable user feedback into actionable development priorities that will significantly enhance Prism's effectiveness for complex Python codebases.**

---

*Last Updated: January 2025*  
*Next Review: Bi-weekly during active development*  
*Status: Ready for implementation*