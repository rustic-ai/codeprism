mod integration;

// Re-export all integration tests for discovery
#[allow(unused_imports)]
pub use integration::test_issue_231_mandrel_codeprism_execution::*;
