//! Service layer demonstrating business logic, error handling, and async patterns.

use crate::models::{User, UserRepository, Repository};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, error};

/// User service demonstrating service patterns and business logic
#[derive(Debug)]
pub struct UserService {
    repository: Arc<UserRepository>,
}

impl UserService {
    /// Create a new user service
    pub fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }

    /// Process a user through business logic
    pub async fn process_user(&self, user: User) -> Result<ProcessingResult> {
        info!("Processing user: {}", user.name());

        // Simulate business logic processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Validate user
        if let Err(e) = user.validate() {
            warn!("User validation failed: {}", e);
            return Ok(ProcessingResult::ValidationFailed(e.to_string()));
        }

        // Store user
        match self.repository.create(user.clone()).await {
            Ok(_) => {
                info!("User {} processed successfully", user.name());
                Ok(ProcessingResult::Success)
            }
            Err(e) => {
                error!("Failed to store user: {}", e);
                Ok(ProcessingResult::StorageFailed(e.to_string()))
            }
        }
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> Result<ServiceStats> {
        let repo_stats = self.repository.stats().await;
        
        Ok(ServiceStats {
            total_processed: repo_stats.total_users,
            active_users: repo_stats.active_users,
            inactive_users: repo_stats.inactive_users,
        })
    }

    /// Batch process multiple users
    pub async fn batch_process(&self, users: Vec<User>) -> Result<BatchResult> {
        info!("Batch processing {} users", users.len());

        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for user in users {
            match self.process_user(user).await? {
                ProcessingResult::Success => successful += 1,
                ProcessingResult::ValidationFailed(err) | ProcessingResult::StorageFailed(err) => {
                    failed += 1;
                    errors.push(err);
                }
            }
        }

        Ok(BatchResult {
            successful,
            failed,
            errors,
        })
    }

    /// Find user by name (business logic wrapper)
    pub async fn find_user_by_name(&self, name: &str) -> Result<Option<User>> {
        let all_users = self.repository.find_all().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch users: {}", e))?;

        Ok(all_users.into_iter().find(|user| user.name() == name))
    }

    /// Get active users count
    pub async fn count_active_users(&self) -> Result<usize> {
        let stats = self.repository.stats().await;
        Ok(stats.active_users)
    }
}

/// Processing result enumeration
#[derive(Debug, Clone)]
pub enum ProcessingResult {
    Success,
    ValidationFailed(String),
    StorageFailed(String),
}

/// Service statistics
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub total_processed: usize,
    pub active_users: usize,
    pub inactive_users: usize,
}

/// Batch processing result
#[derive(Debug)]
pub struct BatchResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserRepository;

    #[tokio::test]
    async fn test_user_service_processing() {
        let repo = Arc::new(UserRepository::new().await.unwrap());
        let service = UserService::new(repo);

        let user = User::new(
            "Test User".to_string(),
            "test@example.com".to_string(),
            25,
        );

        let result = service.process_user(user).await.unwrap();
        match result {
            ProcessingResult::Success => {},
            _ => panic!("Expected successful processing"),
        }

        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.total_processed, 1);
        assert_eq!(stats.active_users, 1);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let repo = Arc::new(UserRepository::new().await.unwrap());
        let service = UserService::new(repo);

        let users = vec![
            User::new("User 1".to_string(), "user1@example.com".to_string(), 25),
            User::new("User 2".to_string(), "user2@example.com".to_string(), 30),
            User::new("".to_string(), "invalid@example.com".to_string(), 25), // Invalid
        ];

        let result = service.batch_process(users).await.unwrap();
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 1);
        assert!(!result.errors.is_empty(), "Batch processing should report errors for invalid users");
        
        // Verify the errors contain meaningful information about validation failures
        assert_eq!(result.errors.len(), 1, "Should have exactly 1 error for the invalid user");
        let error = &result.errors[0];
        assert!(error.contains("validation") || error.contains("invalid"), "Error should describe validation failure: {}", error);
    }

    #[tokio::test]
    async fn test_find_user_by_name() {
        let repo = Arc::new(UserRepository::new().await.unwrap());
        let service = UserService::new(repo);

        let user = User::new(
            "Findable User".to_string(),
            "findable@example.com".to_string(),
            25,
        );

        service.process_user(user).await.unwrap();

        let found = service.find_user_by_name("Findable User").await.unwrap();
        assert!(found.is_some(), "Should find user by name");
        let found_user = found.unwrap();
        assert_eq!(found_user.name(), "Findable User", "Found user should have correct name");
        assert_eq!(found_user.email(), "findable@example.com", "Found user should have correct email");
        assert_eq!(found_user.age(), 25, "Found user should have correct age");
        // This assertion is now redundant as we verify it above

        let not_found = service.find_user_by_name("Non-existent User").await.unwrap();
        assert!(not_found.is_none());
    }
} 