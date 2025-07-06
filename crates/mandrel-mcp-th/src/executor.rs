use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Error,
    Timeout,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub suite_name: String,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub response_data: Option<serde_json::Value>,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub test_results: Vec<TestResult>,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub skipped: usize,
    pub total_tests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub memory_usage_bytes: Option<u64>,
    pub cpu_usage_percent: Option<f64>,
    pub network_requests: Option<u32>,
    pub file_operations: Option<u32>,
    pub response_time_ms: u64,
    pub retry_attempts: u32,
}
