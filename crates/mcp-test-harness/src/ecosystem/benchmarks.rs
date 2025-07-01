//! Performance Benchmarking Framework

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration: Duration,
    pub memory_usage_mb: f64,
    pub operations_per_second: f64,
    pub success: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub name: String,
    pub version: String,
    pub baseline_results: HashMap<String, BenchmarkResult>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Benchmark suite for performance testing
#[derive(Debug)]
pub struct BenchmarkSuite {
    pub name: String,
    pub benchmarks: Vec<BenchmarkResult>,
    pub baseline: Option<PerformanceBaseline>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: String) -> Self {
        Self {
            name,
            benchmarks: Vec::new(),
            baseline: None,
        }
    }

    /// Add benchmark result
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.benchmarks.push(result);
    }

    /// Set performance baseline
    pub fn set_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baseline = Some(baseline);
    }

    /// Compare current results against baseline
    pub fn compare_to_baseline(&self) -> Result<Vec<String>> {
        let baseline = self
            .baseline
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No baseline set"))?;

        let mut comparisons = Vec::new();

        for result in &self.benchmarks {
            if let Some(baseline_result) = baseline.baseline_results.get(&result.test_name) {
                let duration_diff = ((result.duration.as_millis() as f64
                    - baseline_result.duration.as_millis() as f64)
                    / baseline_result.duration.as_millis() as f64)
                    * 100.0;

                let memory_diff = ((result.memory_usage_mb - baseline_result.memory_usage_mb)
                    / baseline_result.memory_usage_mb)
                    * 100.0;

                comparisons.push(format!(
                    "{}: Duration {:+.1}%, Memory {:+.1}%",
                    result.test_name, duration_diff, memory_diff
                ));
            }
        }

        Ok(comparisons)
    }
}
