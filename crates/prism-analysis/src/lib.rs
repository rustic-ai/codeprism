//! Language-agnostic code analysis tools for Prism

pub mod complexity;
pub mod duplicates;
pub mod security;
pub mod performance;
pub mod api_surface;
pub mod semantic;

pub use complexity::ComplexityAnalyzer;
pub use duplicates::DuplicateAnalyzer;
pub use security::SecurityAnalyzer;
pub use performance::PerformanceAnalyzer;
pub use api_surface::ApiSurfaceAnalyzer;

use anyhow::Result;
use serde_json::Value;

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