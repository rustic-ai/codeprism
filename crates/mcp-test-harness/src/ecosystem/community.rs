//! Community Features for Test Sharing and Collaboration

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Community configuration for sharing and collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityConfig {
    pub sharing_enabled: bool,
    pub registry_url: Option<String>,
    pub user_credentials: Option<String>,
    pub local_cache_dir: String,
}

/// Shareable test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestShare {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub server_type: String,
    pub test_config: serde_json::Value,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Test result comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultComparison {
    pub test_name: String,
    pub baseline_result: serde_json::Value,
    pub current_result: serde_json::Value,
    pub differences: Vec<String>,
    pub improvement_percentage: f64,
}

impl CommunityConfig {
    /// Create default community configuration
    pub fn new() -> Self {
        Self {
            sharing_enabled: false,
            registry_url: Some("https://registry.mcp-test-harness.org".to_string()),
            user_credentials: None,
            local_cache_dir: "~/.mcp-test-harness/community".to_string(),
        }
    }

    /// Enable test sharing
    pub fn enable_sharing(&mut self, credentials: String) {
        self.sharing_enabled = true;
        self.user_credentials = Some(credentials);
    }

    /// Disable test sharing
    pub fn disable_sharing(&mut self) {
        self.sharing_enabled = false;
        self.user_credentials = None;
    }
}

impl TestShare {
    /// Create a new test share
    pub fn new(
        name: String,
        version: String,
        author: String,
        description: String,
        server_type: String,
        test_config: serde_json::Value,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            version,
            author,
            description,
            server_type,
            test_config,
            tags: Vec::new(),
            downloads: 0,
            rating: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add tags to the test share
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Export to shareable format
    pub fn export(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| anyhow::anyhow!("Export failed: {}", e))
    }

    /// Import from shareable format
    pub fn import(data: &str) -> Result<Self> {
        serde_yaml::from_str(data).map_err(|e| anyhow::anyhow!("Import failed: {}", e))
    }
}

impl ResultComparison {
    /// Create a new result comparison
    pub fn new(
        test_name: String,
        baseline_result: serde_json::Value,
        current_result: serde_json::Value,
    ) -> Self {
        let differences = Vec::new(); // FUTURE: Implement detailed diff analysis
        let improvement_percentage = 0.0; // FUTURE: Calculate actual improvement

        Self {
            test_name,
            baseline_result,
            current_result,
            differences,
            improvement_percentage,
        }
    }

    /// Get summary of differences
    pub fn summary(&self) -> String {
        format!(
            "Test '{}': {} differences, {:+.1}% improvement",
            self.test_name,
            self.differences.len(),
            self.improvement_percentage
        )
    }
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self::new()
    }
}
