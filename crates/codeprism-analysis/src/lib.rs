//! Language-agnostic code analysis tools for CodePrism

pub mod api_surface;
pub mod complexity;
pub mod duplicates;
pub mod performance;
pub mod security;
pub mod semantic;

pub use api_surface::ApiSurfaceAnalyzer;
pub use complexity::ComplexityAnalyzer;
pub use duplicates::DuplicateAnalyzer;
pub use performance::PerformanceAnalyzer;
pub use security::SecurityAnalyzer;

// Remove unused imports

/// Main analysis coordinator
pub struct CodeAnalyzer {
    pub complexity: ComplexityAnalyzer,
    pub duplicates: DuplicateAnalyzer,
    pub security: SecurityAnalyzer,
    pub performance: PerformanceAnalyzer,
    pub api_surface: ApiSurfaceAnalyzer,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self {
            complexity: ComplexityAnalyzer::new(),
            duplicates: DuplicateAnalyzer::new(),
            security: SecurityAnalyzer::new(),
            performance: PerformanceAnalyzer::new(),
            api_surface: ApiSurfaceAnalyzer::new(),
        }
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
