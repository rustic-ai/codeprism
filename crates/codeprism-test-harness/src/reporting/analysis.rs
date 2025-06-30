//! Performance and coverage analysis for test reports

use crate::types::TestSuiteResult;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Performance analyzer for test execution data
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze performance across test results
    pub async fn analyze(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Result<super::PerformanceAnalysis> {
        let summary = self.calculate_performance_summary(test_results);
        let trends = self.generate_performance_trends(test_results);
        let regression_alerts = Vec::new(); // Will be populated by performance monitoring integration
        let baseline_comparisons = self.generate_baseline_comparisons(test_results);

        Ok(super::PerformanceAnalysis {
            summary,
            trends,
            regression_alerts,
            baseline_comparisons,
        })
    }

    /// Calculate overall performance summary statistics
    fn calculate_performance_summary(
        &self,
        test_results: &[TestSuiteResult],
    ) -> super::PerformanceSummary {
        let mut execution_times = Vec::new();
        let mut total_memory = 0.0;
        let mut _total_tests = 0;

        for suite in test_results {
            for test in &suite.test_results {
                execution_times.push(test.duration.as_millis() as f64);
                if let Some(memory) = test.memory_usage_mb {
                    total_memory += memory;
                }
                _total_tests += 1;
            }
        }

        let average_execution_time_ms = if !execution_times.is_empty() {
            execution_times.iter().sum::<f64>() / execution_times.len() as f64
        } else {
            0.0
        };

        // Calculate 95th percentile
        execution_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = (execution_times.len() as f64 * 0.95) as usize;
        let p95_execution_time_ms = execution_times.get(p95_index).copied().unwrap_or(0.0) as u64;

        let throughput_ops_per_sec = if average_execution_time_ms > 0.0 {
            1000.0 / average_execution_time_ms
        } else {
            0.0
        };

        super::PerformanceSummary {
            average_execution_time_ms,
            p95_execution_time_ms,
            total_memory_usage_mb: total_memory,
            throughput_ops_per_sec,
        }
    }

    /// Generate performance trend data over time
    fn generate_performance_trends(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Vec<super::PerformanceTrend> {
        // Create simplified trend with current test data
        // In a real implementation, this would compare against historical data
        let current_time = Utc::now();

        vec![
            super::PerformanceTrend {
                metric_name: "Average Execution Time".to_string(),
                data_points: vec![(
                    current_time,
                    self.calculate_average_execution_time(test_results),
                )],
                trend_direction: super::TrendDirection::Stable,
            },
            super::PerformanceTrend {
                metric_name: "Memory Usage".to_string(),
                data_points: vec![(
                    current_time,
                    self.calculate_average_memory_usage(test_results),
                )],
                trend_direction: super::TrendDirection::Stable,
            },
        ]
    }

    /// Generate baseline comparisons
    fn generate_baseline_comparisons(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Vec<super::BaselineComparison> {
        let mut comparisons = Vec::new();
        let mut tool_performances: HashMap<String, Vec<f64>> = HashMap::new();

        // Collect performance data by tool
        for suite in test_results {
            for test in &suite.test_results {
                tool_performances
                    .entry(test.test_case.tool_name.clone())
                    .or_default()
                    .push(test.duration.as_millis() as f64);
            }
        }

        // Create comparisons (using current performance as both current and baseline for now)
        for (tool_name, performances) in tool_performances {
            if !performances.is_empty() {
                let avg_performance = performances.iter().sum::<f64>() / performances.len() as f64;
                comparisons.push(super::BaselineComparison {
                    tool_name,
                    current_performance: avg_performance,
                    baseline_performance: avg_performance, // Would come from stored baselines
                    performance_delta_percent: 0.0,
                });
            }
        }

        comparisons
    }

    fn calculate_average_execution_time(&self, test_results: &[TestSuiteResult]) -> f64 {
        let mut total_time = 0.0;
        let mut count = 0;

        for suite in test_results {
            for test in &suite.test_results {
                total_time += test.duration.as_millis() as f64;
                count += 1;
            }
        }

        if count > 0 {
            total_time / count as f64
        } else {
            0.0
        }
    }

    fn calculate_average_memory_usage(&self, test_results: &[TestSuiteResult]) -> f64 {
        let mut total_memory = 0.0;
        let mut count = 0;

        for suite in test_results {
            for test in &suite.test_results {
                if let Some(memory) = test.memory_usage_mb {
                    total_memory += memory;
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_memory / count as f64
        } else {
            0.0
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Test coverage analyzer
pub struct TestCoverageAnalyzer;

impl TestCoverageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze test coverage across results
    pub async fn analyze(
        &self,
        test_results: &[TestSuiteResult],
    ) -> Result<super::CoverageAnalysis> {
        let tool_coverage = self.calculate_tool_coverage(test_results);
        let feature_coverage = self.calculate_feature_coverage(test_results);
        let coverage_trends = self.generate_coverage_trends();

        Ok(super::CoverageAnalysis {
            tool_coverage,
            feature_coverage,
            coverage_trends,
        })
    }

    /// Calculate tool coverage statistics
    fn calculate_tool_coverage(&self, test_results: &[TestSuiteResult]) -> super::ToolCoverage {
        let mut tested_tools = std::collections::HashSet::new();
        let mut all_tools = std::collections::HashSet::new();

        for suite in test_results {
            for test in &suite.test_results {
                all_tools.insert(test.test_case.tool_name.clone());
                if test.success {
                    tested_tools.insert(test.test_case.tool_name.clone());
                }
            }
        }

        let total_tools = all_tools.len();
        let tested_count = tested_tools.len();
        let coverage_percentage = if total_tools > 0 {
            (tested_count as f64 / total_tools as f64) * 100.0
        } else {
            100.0
        };

        let untested_tools: Vec<String> = all_tools.difference(&tested_tools).cloned().collect();

        super::ToolCoverage {
            total_tools,
            tested_tools: tested_count,
            coverage_percentage,
            untested_tools,
        }
    }

    /// Calculate feature coverage statistics
    fn calculate_feature_coverage(
        &self,
        test_results: &[TestSuiteResult],
    ) -> super::FeatureCoverage {
        let mut feature_breakdown = HashMap::new();
        let mut total_features = 0;
        let mut tested_features = 0;

        // Simple feature detection based on test case IDs
        for suite in test_results {
            for test in &suite.test_results {
                let feature_name = self.extract_feature_name(&test.test_case.id);
                total_features += 1;

                let is_tested = test.success;
                feature_breakdown.insert(feature_name, is_tested);

                if is_tested {
                    tested_features += 1;
                }
            }
        }

        let coverage_percentage = if total_features > 0 {
            (tested_features as f64 / total_features as f64) * 100.0
        } else {
            100.0
        };

        super::FeatureCoverage {
            total_features,
            tested_features,
            coverage_percentage,
            feature_breakdown,
        }
    }

    /// Extract feature name from test case ID
    fn extract_feature_name(&self, test_id: &str) -> String {
        // Simple extraction - split on underscore and take first part
        test_id.split('_').next().unwrap_or(test_id).to_string()
    }

    /// Generate coverage trends (simplified implementation)
    fn generate_coverage_trends(&self) -> Vec<super::CoverageTrend> {
        vec![super::CoverageTrend {
            timestamp: Utc::now(),
            tool_coverage_percent: 85.0, // Would be calculated from historical data
            feature_coverage_percent: 78.0,
        }]
    }
}

impl Default for TestCoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
