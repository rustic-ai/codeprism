//! Data models demonstrating Rust ownership, traits, and design patterns.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// User model demonstrating ownership, cloning, and validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    age: u32,
    active: bool,
}

impl User {
    /// Create a new user with validation
    pub fn new(name: String, email: String, age: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            age,
            active: true,
        }
    }

    /// Get user ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get user name (immutable borrow)
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get user email
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Get user age
    pub fn age(&self) -> u32 {
        self.age
    }

    /// Set user age (mutable operation)
    pub fn set_age(&mut self, age: u32) {
        self.age = age;
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activate user
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate user
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Validate user data
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName);
        }

        if !self.email.contains('@') {
            return Err(ValidationError::InvalidEmail(self.email.clone()));
        }

        if self.age > 150 {
            return Err(ValidationError::InvalidAge(self.age));
        }

        Ok(())
    }
}

/// Custom validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("User name cannot be empty")]
    EmptyName,
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    #[error("Invalid age: {0}")]
    InvalidAge(u32),
}

/// Repository trait demonstrating trait definitions and async patterns
#[async_trait::async_trait]
pub trait Repository<T> {
    type Error;

    async fn create(&self, item: T) -> Result<T, Self::Error>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<T>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;
    async fn update(&self, item: T) -> Result<T, Self::Error>;
    async fn delete(&self, id: &Uuid) -> Result<bool, Self::Error>;
}

/// In-memory user repository demonstrating Arc, RwLock, and async patterns
#[derive(Debug)]
pub struct UserRepository {
    storage: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl UserRepository {
    /// Create a new user repository
    pub async fn new() -> Result<Self> {
        Ok(Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get repository statistics
    pub async fn stats(&self) -> RepositoryStats {
        let storage = self.storage.read().await;
        let total_users = storage.len();
        let active_users = storage.values().filter(|u| u.is_active()).count();

        RepositoryStats {
            total_users,
            active_users,
            inactive_users: total_users - active_users,
        }
    }

    /// Clear all users (for testing)
    pub async fn clear(&self) {
        let mut storage = self.storage.write().await;
        storage.clear();
    }
}

/// Repository statistics
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    pub total_users: usize,
    pub active_users: usize,
    pub inactive_users: usize,
}

/// Repository errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("User not found with ID: {0}")]
    NotFound(Uuid),
    #[error("User already exists with ID: {0}")]
    AlreadyExists(Uuid),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[async_trait::async_trait]
impl Repository<User> for UserRepository {
    type Error = RepositoryError;

    async fn create(&self, user: User) -> Result<User, Self::Error> {
        // Validate user data
        user.validate()?;

        let mut storage = self.storage.write().await;
        
        if storage.contains_key(&user.id) {
            return Err(RepositoryError::AlreadyExists(user.id));
        }

        storage.insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, Self::Error> {
        let storage = self.storage.read().await;
        Ok(storage.get(id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<User>, Self::Error> {
        let storage = self.storage.read().await;
        Ok(storage.values().cloned().collect())
    }

    async fn update(&self, user: User) -> Result<User, Self::Error> {
        // Validate user data
        user.validate()?;

        let mut storage = self.storage.write().await;
        
        if !storage.contains_key(&user.id) {
            return Err(RepositoryError::NotFound(user.id));
        }

        storage.insert(user.id, user.clone());
        Ok(user)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, Self::Error> {
        let mut storage = self.storage.write().await;
        Ok(storage.remove(id).is_some())
    }
}

/// Generic filter trait demonstrating trait bounds and generics
pub trait Filter<T> {
    fn apply(&self, items: Vec<T>) -> Vec<T>;
}

/// Active user filter implementation
pub struct ActiveUserFilter;

impl Filter<User> for ActiveUserFilter {
    fn apply(&self, items: Vec<User>) -> Vec<User> {
        items.into_iter().filter(|user| user.is_active()).collect()
    }
}

/// Age range filter
pub struct AgeRangeFilter {
    min_age: u32,
    max_age: u32,
}

impl AgeRangeFilter {
    pub fn new(min_age: u32, max_age: u32) -> Self {
        Self { min_age, max_age }
    }
}

impl Filter<User> for AgeRangeFilter {
    fn apply(&self, items: Vec<User>) -> Vec<User> {
        items
            .into_iter()
            .filter(|user| user.age >= self.min_age && user.age <= self.max_age)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_creation() {
        let user = User::new("John Doe".to_string(), "john@example.com".to_string(), 30);
        assert_eq!(user.name(), "John Doe");
        assert_eq!(user.email(), "john@example.com");
        assert_eq!(user.age(), 30);
        assert!(user.is_active());
    }

    #[tokio::test]
    async fn test_user_validation() {
        let mut user = User::new("".to_string(), "invalid-email".to_string(), 200);
        
        match user.validate() {
            Err(ValidationError::EmptyName) => {},
            _ => panic!("Expected EmptyName error"),
        }

        user = User::new("John".to_string(), "invalid-email".to_string(), 200);
        
        match user.validate() {
            Err(ValidationError::InvalidEmail(_)) => {},
            _ => panic!("Expected InvalidEmail error"),
        }
    }

    #[tokio::test]
    async fn test_repository_operations() {
        let repo = UserRepository::new().await.unwrap();
        let user = User::new("Test User".to_string(), "test@example.com".to_string(), 25);
        let user_id = user.id;

        // Test create
        let created = repo.create(user).await.unwrap();
        assert_eq!(created.name(), "Test User");

        // Test find_by_id
        let found = repo.find_by_id(&user_id).await.unwrap();
        assert!(found.is_some(), "Should find user by ID");
        let found_user = found.unwrap();
        assert_eq!(found_user.name(), "Test User", "Found user should have correct name");
        assert_eq!(found_user.email(), "test@example.com", "Found user should have correct email");
        assert_eq!(found.unwrap().name(), "Test User");

        // Test find_all
        let all_users = repo.find_all().await.unwrap();
        assert_eq!(all_users.len(), 1, "Should have exactly 1 user in repository");
        assert_eq!(all_users[0].name(), "Test User", "User in repository should have correct name");
        assert!(all_users[0].is_active(), "User should be active by default");

        // Test delete
        let deleted = repo.delete(&user_id).await.unwrap();
        assert!(deleted);

        let not_found = repo.find_by_id(&user_id).await.unwrap();
        assert!(not_found.is_none(), "Should be none");
    }

    #[test]
    fn test_filters() {
        let users = vec![
            User::new("Active User".to_string(), "active@example.com".to_string(), 25),
            {
                let mut user = User::new("Inactive User".to_string(), "inactive@example.com".to_string(), 30);
                user.deactivate();
                user
            },
            User::new("Young User".to_string(), "young@example.com".to_string(), 20),
        ];

        let active_filter = ActiveUserFilter;
        let filtered = active_filter.apply(users.clone());
        assert_eq!(filtered.len(), 2, "Should have 2 active users after filtering");
        // Verify the filtered users are actually active
        assert!(filtered.iter().all(|u| u.is_active()), "All filtered users should be active");
        let names: Vec<&str> = filtered.iter().map(|u| u.name()).collect();
        assert!(names.contains(&"Active User"), "Should contain Active User");
        assert!(names.contains(&"Young User"), "Should contain Young User");

        let age_filter = AgeRangeFilter::new(22, 28);
        let age_filtered = age_filter.apply(users);
        assert_eq!(age_filtered.len(), 1, "Should have 1 user in age range 22-28");
        let filtered_user = &age_filtered[0];
        assert!(filtered_user.age() >= 22 && filtered_user.age() <= 28, "Filtered user should be within age range");
        assert_eq!(filtered_user.name(), "Active User", "Filtered user should be Active User");
        assert_eq!(age_filtered[0].age(), 25);
    }
} 