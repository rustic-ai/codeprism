//! Quality testing module for CodePrism
//! 
//! This module contains quality assurance tests including:
//! - Code coverage analysis and reporting
//! - API compatibility testing
//! - Documentation accuracy validation
//! - Quality metrics collection

pub mod coverage_tests;
pub mod api_compatibility;
pub mod documentation_tests;
pub mod quality_metrics;

use std::collections::HashMap;

/// Quality test result
#[derive(Debug, Clone)]
pub struct QualityResult {
    pub test_name: String,
    pub coverage_percentage: Option<f64>,
    pub success: bool,
    pub details: String,
    pub metrics: HashMap<String, f64>,
}

impl QualityResult {
    pub fn new(test_name: &str, success: bool) -> Self {
        Self {
            test_name: test_name.to_string(),
            coverage_percentage: None,
            success,
            details: String::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn with_coverage(mut self, coverage: f64) -> Self {
        self.coverage_percentage = Some(coverage);
        self
    }

    pub fn with_metric(mut self, name: &str, value: f64) -> Self {
        self.metrics.insert(name.to_string(), value);
        self
    }
}

/// Quality testing harness
pub struct QualityTestHarness {
    results: Vec<QualityResult>,
    coverage_threshold: f64,
}

impl QualityTestHarness {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            coverage_threshold: 80.0, // 80% minimum coverage
        }
    }

    pub fn set_coverage_threshold(&mut self, threshold: f64) {
        self.coverage_threshold = threshold;
    }

    pub fn run_coverage_test<F>(&mut self, name: &str, test_fn: F) -> &QualityResult
    where
        F: FnOnce() -> Result<f64, Box<dyn std::error::Error>>,
    {
        let result = match test_fn() {
            Ok(coverage) => {
                let success = coverage >= self.coverage_threshold;
                QualityResult::new(name, success)
                    .with_coverage(coverage)
                    .with_metric("coverage", coverage)
            }
            Err(_) => QualityResult::new(name, false),
        };
        
        self.results.push(result);
        self.results.last().unwrap()
    }

    pub fn results(&self) -> &[QualityResult] {
        &self.results
    }

    pub fn overall_coverage(&self) -> Option<f64> {
        let coverages: Vec<f64> = self.results.iter()
            .filter_map(|r| r.coverage_percentage)
            .collect();
        
        if coverages.is_empty() {
            None
        } else {
            Some(coverages.iter().sum::<f64>() / coverages.len() as f64)
        }
    }

    pub fn print_summary(&self) {
        println!("\n📋 Quality Test Summary");
        println!("{}", "=".repeat(50));
        
        for result in &self.results {
            let status = if result.success { "✓" } else { "✗" };
            if let Some(coverage) = result.coverage_percentage {
                println!("{} {}: {:.1}% coverage", status, result.test_name, coverage);
            } else {
                println!("{} {}", status, result.test_name);
            }
        }
        
        if let Some(overall) = self.overall_coverage() {
            println!("{}", "=".repeat(50));
            println!("📊 Overall Coverage: {:.1}%", overall);
            
            if overall >= self.coverage_threshold {
                println!("✅ Coverage threshold met ({:.1}%)", self.coverage_threshold);
            } else {
                println!("❌ Coverage below threshold ({:.1}%)", self.coverage_threshold);
            }
        }
    }
}

impl Default for QualityTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_result_creation() {
        let result = QualityResult::new("test", true)
            .with_coverage(85.0)
            .with_metric("complexity", 3.2);
        
        assert_eq!(result.test_name, "test");
        assert!(result.success);
        assert_eq!(result.coverage_percentage, Some(85.0));
        assert_eq!(result.metrics.get("complexity"), Some(&3.2));
    }

    #[test]
    fn test_quality_harness() {
        let mut harness = QualityTestHarness::new();
        harness.set_coverage_threshold(75.0);
        
        harness.run_coverage_test("high_coverage", || Ok(85.0));
        harness.run_coverage_test("low_coverage", || Ok(60.0));
        
        assert_eq!(harness.results().len(), 2);
        assert!(harness.results()[0].success);
        assert!(!harness.results()[1].success);
        
        assert_eq!(harness.overall_coverage(), Some(72.5));
    }
} 