//! Execution strategies for test suite runner

use std::time::Duration;

/// Strategy for executing test cases
#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    /// Execute tests one at a time in dependency order
    Sequential,
    /// Execute tests in parallel groups respecting dependencies
    Parallel { max_concurrency: usize },
}

impl ExecutionStrategy {
    /// Create a sequential execution strategy
    pub fn sequential() -> Self {
        ExecutionStrategy::Sequential
    }

    /// Create a parallel execution strategy with specified concurrency
    pub fn parallel(max_concurrency: usize) -> Self {
        ExecutionStrategy::Parallel {
            max_concurrency: max_concurrency.clamp(1, 16),
        }
    }

    /// Check if this strategy supports parallel execution
    pub fn is_parallel(&self) -> bool {
        matches!(self, ExecutionStrategy::Parallel { .. })
    }

    /// Get the maximum concurrency for this strategy
    pub fn max_concurrency(&self) -> usize {
        match self {
            ExecutionStrategy::Sequential => 1,
            ExecutionStrategy::Parallel { max_concurrency } => *max_concurrency,
        }
    }
}

/// Execution context for a test case
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub test_name: String,
    pub dependencies: Vec<String>,
    pub timeout: Duration,
    pub retry_count: usize,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(test_name: String, dependencies: Vec<String>) -> Self {
        Self {
            test_name,
            dependencies,
            timeout: Duration::from_secs(30),
            retry_count: 0,
        }
    }

    /// Set timeout for this execution context
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry count for this execution context
    pub fn with_retry_count(mut self, retry_count: usize) -> Self {
        self.retry_count = retry_count;
        self
    }
}

/// Result of test execution with timing information
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub retry_attempts: usize,
}

impl ExecutionResult {
    /// Create a successful execution result
    pub fn success(test_name: String, duration: Duration) -> Self {
        Self {
            test_name,
            success: true,
            duration,
            error_message: None,
            retry_attempts: 0,
        }
    }

    /// Create a failed execution result
    pub fn failure(test_name: String, duration: Duration, error: String) -> Self {
        Self {
            test_name,
            success: false,
            duration,
            error_message: Some(error),
            retry_attempts: 0,
        }
    }

    /// Set the number of retry attempts
    pub fn with_retry_attempts(mut self, attempts: usize) -> Self {
        self.retry_attempts = attempts;
        self
    }
}

/// Execution scheduler for managing test case execution order and timing
#[derive(Debug)]
pub struct ExecutionScheduler {
    strategy: ExecutionStrategy,
    fail_fast: bool,
}

impl ExecutionScheduler {
    /// Create a new execution scheduler
    pub fn new(strategy: ExecutionStrategy, fail_fast: bool) -> Self {
        Self {
            strategy,
            fail_fast,
        }
    }

    /// Schedule execution of test cases
    ///
    /// Returns groups of test cases that can be executed together
    pub fn schedule_execution(&self, execution_order: &[String]) -> Vec<Vec<String>> {
        match &self.strategy {
            ExecutionStrategy::Sequential => {
                // For sequential execution, each test is its own group
                execution_order
                    .iter()
                    .map(|test| vec![test.clone()])
                    .collect()
            }
            ExecutionStrategy::Parallel { max_concurrency } => {
                // For parallel execution, group tests into batches of max_concurrency
                let mut groups = Vec::new();
                let mut current_group = Vec::new();

                for test in execution_order {
                    current_group.push(test.clone());
                    if current_group.len() >= *max_concurrency {
                        groups.push(current_group);
                        current_group = Vec::new();
                    }
                }

                if !current_group.is_empty() {
                    groups.push(current_group);
                }

                groups
            }
        }
    }

    /// Check if execution should stop after a failure
    pub fn should_stop_on_failure(&self, results: &[ExecutionResult]) -> bool {
        if !self.fail_fast {
            return false;
        }

        results.iter().any(|result| !result.success)
    }

    /// Get the execution strategy
    pub fn strategy(&self) -> &ExecutionStrategy {
        &self.strategy
    }

    /// Check if fail-fast is enabled
    pub fn is_fail_fast(&self) -> bool {
        self.fail_fast
    }
}

/// Execution statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
    pub total_retries: usize,
}

impl ExecutionStats {
    /// Create execution statistics from results
    pub fn from_results(results: &[ExecutionResult]) -> Self {
        if results.is_empty() {
            return Self::default();
        }

        let total_executions = results.len();
        let successful_executions = results.iter().filter(|r| r.success).count();
        let failed_executions = total_executions - successful_executions;

        let total_duration: Duration = results.iter().map(|r| r.duration).sum();
        let average_duration = total_duration / total_executions as u32;

        let max_duration = results.iter().map(|r| r.duration).max().unwrap_or_default();
        let min_duration = results.iter().map(|r| r.duration).min().unwrap_or_default();

        let total_retries = results.iter().map(|r| r.retry_attempts).sum();

        Self {
            total_executions,
            successful_executions,
            failed_executions,
            total_duration,
            average_duration,
            max_duration,
            min_duration,
            total_retries,
        }
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0
        }
    }

    /// Calculate failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_duration: Duration::from_secs(0),
            average_duration: Duration::from_secs(0),
            max_duration: Duration::from_secs(0),
            min_duration: Duration::from_secs(0),
            total_retries: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_strategy_creation() {
        let seq = ExecutionStrategy::sequential();
        assert!(!seq.is_parallel());
        assert_eq!(seq.max_concurrency(), 1);

        let par = ExecutionStrategy::parallel(4);
        assert!(par.is_parallel());
        assert_eq!(par.max_concurrency(), 4);
    }

    #[test]
    fn test_execution_strategy_concurrency_clamping() {
        let par = ExecutionStrategy::parallel(0);
        assert_eq!(par.max_concurrency(), 1); // Should be clamped to minimum

        let par = ExecutionStrategy::parallel(20);
        assert_eq!(par.max_concurrency(), 16); // Should be clamped to maximum
    }

    #[test]
    fn test_execution_scheduler_sequential() {
        let scheduler = ExecutionScheduler::new(ExecutionStrategy::sequential(), false);
        let order = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        let groups = scheduler.schedule_execution(&order);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], vec!["test1"]);
        assert_eq!(groups[1], vec!["test2"]);
        assert_eq!(groups[2], vec!["test3"]);
    }

    #[test]
    fn test_execution_scheduler_parallel() {
        let scheduler = ExecutionScheduler::new(ExecutionStrategy::parallel(2), false);
        let order = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        let groups = scheduler.schedule_execution(&order);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec!["test1", "test2"]);
        assert_eq!(groups[1], vec!["test3"]);
    }

    #[test]
    #[ignore = "Future work for Issue #227 - floating point precision issues in metrics calculations"]
    fn test_execution_stats() {
        let results = vec![
            ExecutionResult::success("test1".to_string(), Duration::from_millis(100)),
            ExecutionResult::failure(
                "test2".to_string(),
                Duration::from_millis(200),
                "error".to_string(),
            ),
            ExecutionResult::success("test3".to_string(), Duration::from_millis(150)),
        ];

        let stats = ExecutionStats::from_results(&results);
        assert_eq!(stats.total_executions, 3);
        assert_eq!(stats.successful_executions, 2);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.success_rate(), 66.66666666666667);
        assert_eq!(stats.total_duration, Duration::from_millis(450));
        assert_eq!(stats.average_duration, Duration::from_millis(150));
    }

    #[test]
    fn test_fail_fast_behavior() {
        let scheduler = ExecutionScheduler::new(ExecutionStrategy::sequential(), true);

        let results = vec![
            ExecutionResult::success("test1".to_string(), Duration::from_millis(100)),
            ExecutionResult::failure(
                "test2".to_string(),
                Duration::from_millis(200),
                "error".to_string(),
            ),
        ];

        assert!(scheduler.should_stop_on_failure(&results));

        let scheduler_no_fail_fast =
            ExecutionScheduler::new(ExecutionStrategy::sequential(), false);
        assert!(!scheduler_no_fail_fast.should_stop_on_failure(&results));
    }
}
