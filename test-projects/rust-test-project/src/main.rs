//! Main entry point for the Rust test project.
//!
//! This project demonstrates various Rust patterns and features for MCP analysis:
//! - Ownership and borrowing patterns
//! - Error handling with Result and custom errors
//! - Async/await programming
//! - Trait implementations and generics
//! - Unsafe code blocks
//! - Concurrency patterns
//! - Memory management
//! - Performance optimization techniques

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, Level};
use tracing_subscriber;

mod models;
mod services;
mod patterns;
mod unsafe_ops;
mod async_patterns;
mod performance;
mod errors;

use models::{User, UserRepository};
use services::UserService;
use patterns::SingletonManager;
use async_patterns::AsyncProcessor;

/// Main application structure demonstrating composition and dependency injection
#[derive(Debug)]
pub struct Application {
    user_service: Arc<UserService>,
    singleton_manager: &'static SingletonManager,
    async_processor: Arc<AsyncProcessor>,
}

impl Application {
    /// Create a new application instance with dependency injection
    pub async fn new() -> Result<Self> {
        info!("Initializing Rust Test Application");

        // Initialize tracing
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .init();

        // Create dependencies with Arc for shared ownership
        let user_repository = Arc::new(UserRepository::new().await?);
        let user_service = Arc::new(UserService::new(user_repository));
        let singleton_manager = SingletonManager::instance();
        let async_processor = Arc::new(AsyncProcessor::new());

        Ok(Self {
            user_service,
            singleton_manager,
            async_processor,
        })
    }

    /// Run the application with various demonstrations
    pub async fn run(&self) -> Result<()> {
        info!("Starting application demonstrations");

        // Demonstrate ownership patterns
        self.demonstrate_ownership_patterns().await?;

        // Demonstrate error handling
        self.demonstrate_error_handling().await?;

        // Demonstrate async patterns
        self.demonstrate_async_patterns().await?;

        // Demonstrate concurrency
        self.demonstrate_concurrency().await?;

        // Demonstrate unsafe operations (carefully)
        self.demonstrate_unsafe_operations()?;

        // Demonstrate performance patterns
        self.demonstrate_performance_patterns().await?;

        info!("Application demonstrations completed successfully");
        Ok(())
    }

    /// Demonstrate Rust ownership and borrowing patterns
    async fn demonstrate_ownership_patterns(&self) -> Result<()> {
        info!("=== Demonstrating Ownership Patterns ===");

        // Create a user with owned data
        let mut user = User::new(
            "john_doe".to_string(),
            "john@example.com".to_string(),
            25,
        );

        // Demonstrate borrowing
        let name_ref: &str = user.name(); // Immutable borrow
        info!("User name (borrowed): {}", name_ref);

        // Demonstrate mutable borrowing
        user.set_age(26); // Mutable borrow
        info!("Updated user age: {}", user.age());

        // Demonstrate move semantics
        let user_clone = user.clone(); // Explicit clone to avoid move
        let user_service = Arc::clone(&self.user_service);
        
        // Move into async task
        let handle = tokio::spawn(async move {
            user_service.process_user(user_clone).await
        });

        match handle.await? {
            Ok(_) => info!("User processing completed successfully"),
            Err(e) => warn!("User processing failed: {}", e),
        }

        // Demonstrate reference counting with Arc
        let service_clone = Arc::clone(&self.user_service);
        let another_clone = Arc::clone(&service_clone);
        
        info!("Arc reference count demonstration - multiple references to same service");
        drop(service_clone); // Explicitly drop one reference
        
        // Original service still accessible through another_clone
        let stats = another_clone.get_stats().await?;
        info!("Service stats: {:?}", stats);

        Ok(())
    }

    /// Demonstrate comprehensive error handling patterns
    async fn demonstrate_error_handling(&self) -> Result<()> {
        info!("=== Demonstrating Error Handling Patterns ===");

        // Using Result type for error handling
        let user_result = self.create_sample_user("valid_user", "valid@example.com").await;
        match user_result {
            Ok(user) => info!("Successfully created user: {}", user.name()),
            Err(e) => error!("Failed to create user: {}", e),
        }

        // Demonstrate error chaining with context
        let invalid_result = self.create_sample_user("", "invalid-email").await
            .context("Failed to create user with invalid data");
        
        if let Err(e) = invalid_result {
            error!("Error with context: {:?}", e);
        }

        // Demonstrate custom error types
        let validation_result = self.validate_user_data("", "bad-email");
        match validation_result {
            Ok(_) => info!("User data is valid"),
            Err(errors::ValidationError::EmptyName) => warn!("User name cannot be empty"),
            Err(errors::ValidationError::InvalidEmail(email)) => {
                warn!("Invalid email format: {}", email)
            },
            Err(e) => error!("Other validation error: {}", e),
        }

        // Demonstrate error propagation with ?
        let _complex_result = self.complex_operation_with_error_propagation().await?;

        Ok(())
    }

    /// Demonstrate async/await patterns and concurrent processing
    async fn demonstrate_async_patterns(&self) -> Result<()> {
        info!("=== Demonstrating Async Patterns ===");

        // Sequential async operations
        let start = std::time::Instant::now();
        
        let user1 = self.async_processor.process_data("user1").await?;
        let user2 = self.async_processor.process_data("user2").await?;
        let user3 = self.async_processor.process_data("user3").await?;
        
        let sequential_duration = start.elapsed();
        info!("Sequential processing took: {:?}", sequential_duration);
        info!("Results: {} {} {}", user1, user2, user3);

        // Concurrent async operations with join!
        let start = std::time::Instant::now();
        
        let (result1, result2, result3) = tokio::join!(
            self.async_processor.process_data("user1"),
            self.async_processor.process_data("user2"),
            self.async_processor.process_data("user3")
        );

        let concurrent_duration = start.elapsed();
        info!("Concurrent processing took: {:?}", concurrent_duration);
        info!("Results: {:?} {:?} {:?}", result1, result2, result3);

        // Demonstrate async stream processing
        let mut stream = self.async_processor.create_data_stream().await;
        let mut count = 0;
        
        while let Some(item) = stream.recv().await {
            info!("Received stream item: {}", item);
            count += 1;
            if count >= 5 { break; } // Limit for demo
        }

        Ok(())
    }

    /// Demonstrate concurrency patterns with threads and channels
    async fn demonstrate_concurrency(&self) -> Result<()> {
        info!("=== Demonstrating Concurrency Patterns ===");

        // Shared state with Arc<RwLock<T>>
        let shared_counter = Arc::new(RwLock::new(0u64));
        let mut handles = Vec::new();

        // Spawn multiple tasks that share state
        for i in 0..5 {
            let counter = Arc::clone(&shared_counter);
            let handle = tokio::spawn(async move {
                for _ in 0..10 {
                    let mut guard = counter.write().await;
                    *guard += 1;
                    info!("Task {} incremented counter to {}", i, *guard);
                    drop(guard); // Release lock early
                    
                    // Simulate some work
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await?;
        }

        let final_count = *shared_counter.read().await;
        info!("Final counter value: {}", final_count);

        // Demonstrate channel-based communication
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);

        // Producer task
        let producer = tokio::spawn(async move {
            for i in 0..10 {
                let message = format!("Message {}", i);
                if tx.send(message).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });

        // Consumer task
        let consumer = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                info!("Received: {}", message);
            }
        });

        // Wait for both tasks
        let _ = tokio::join!(producer, consumer);

        Ok(())
    }

    /// Demonstrate unsafe operations (with extreme caution)
    fn demonstrate_unsafe_operations(&self) -> Result<()> {
        info!("=== Demonstrating Unsafe Operations (Educational Only) ===");

        // Raw pointer manipulation (unsafe)
        let x = 42i32;
        let raw_ptr = &x as *const i32;
        
        unsafe {
            let value = *raw_ptr;
            info!("Value read from raw pointer: {}", value);
        }

        // Unsafe function call
        let slice = &[1, 2, 3, 4, 5];
        unsafe {
            let element = unsafe_ops::get_unchecked_element(slice, 2);
            info!("Unchecked element access: {}", element);
        }

        // FFI demonstration (calling C library function)
        unsafe {
            let pid = libc::getpid();
            info!("Current process ID (from libc): {}", pid);
        }

        // Memory alignment demonstration
        let buffer = unsafe_ops::create_aligned_buffer(1024);
        info!("Created aligned buffer of size: {}", buffer.len());

        warn!("Unsafe operations completed - use with extreme caution in production!");
        
        Ok(())
    }

    /// Demonstrate performance optimization patterns
    async fn demonstrate_performance_patterns(&self) -> Result<()> {
        info!("=== Demonstrating Performance Patterns ===");

        // Vector capacity pre-allocation
        let mut optimized_vec = Vec::with_capacity(1000);
        let start = std::time::Instant::now();
        
        for i in 0..1000 {
            optimized_vec.push(i);
        }
        
        let optimized_duration = start.elapsed();
        info!("Pre-allocated vector took: {:?}", optimized_duration);

        // String building with capacity
        let mut efficient_string = String::with_capacity(10000);
        let start = std::time::Instant::now();
        
        for i in 0..1000 {
            efficient_string.push_str(&format!("Item {} ", i));
        }
        
        let string_duration = start.elapsed();
        info!("Efficient string building took: {:?}", string_duration);

        // Demonstrate iterator chaining and lazy evaluation
        let data: Vec<i32> = (0..1000).collect();
        let start = std::time::Instant::now();
        
        let result: Vec<i32> = data
            .iter()
            .filter(|&&x| x % 2 == 0)
            .map(|&x| x * x)
            .filter(|&x| x > 100)
            .take(10)
            .collect();
        
        let iterator_duration = start.elapsed();
        info!("Iterator chain processing took: {:?}", iterator_duration);
        info!("Filtered results: {:?}", result);

        // Demonstrate parallel processing with Rayon
        use rayon::prelude::*;
        let large_data: Vec<i32> = (0..1_000_000).collect();
        
        let start = std::time::Instant::now();
        let sum: i32 = large_data.par_iter().sum();
        let parallel_duration = start.elapsed();
        
        info!("Parallel sum of 1M integers: {} in {:?}", sum, parallel_duration);

        Ok(())
    }

    /// Helper method to create sample users with validation
    async fn create_sample_user(&self, name: &str, email: &str) -> Result<User> {
        // Validate input
        self.validate_user_data(name, email)?;
        
        // Create user
        let user = User::new(name.to_string(), email.to_string(), 30);
        
        // Process through service
        self.user_service.process_user(user.clone()).await?;
        
        Ok(user)
    }

    /// Validate user data with custom error types
    fn validate_user_data(&self, name: &str, email: &str) -> Result<(), errors::ValidationError> {
        if name.is_empty() {
            return Err(errors::ValidationError::EmptyName);
        }
        
        if !email.contains('@') || !email.contains('.') {
            return Err(errors::ValidationError::InvalidEmail(email.to_string()));
        }
        
        Ok(())
    }

    /// Complex operation demonstrating error propagation
    async fn complex_operation_with_error_propagation(&self) -> Result<String> {
        let user = self.create_sample_user("complex_user", "complex@example.com").await?;
        let processed = self.async_processor.process_data(&user.name()).await?;
        let stats = self.user_service.get_stats().await?;
        
        Ok(format!("Complex operation result: {}, stats: {:?}", processed, stats))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("Rust Test Project")
        .version("1.0")
        .author("CodePrism Team")
        .about("Comprehensive Rust patterns demonstration for MCP analysis")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose logging")
        )
        .arg(
            Arg::new("demo")
                .short('d')
                .long("demo")
                .value_name("DEMO_TYPE")
                .help("Run specific demo type")
                .value_parser(["ownership", "async", "concurrency", "unsafe", "performance", "all"])
        )
        .get_matches();

    // Configure logging based on verbosity
    let log_level = if matches.get_flag("verbose") {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("Starting Rust Test Project");

    // Create and run application
    let app = Application::new().await
        .context("Failed to initialize application")?;

    // Run specific demo or all demos
    match matches.get_one::<String>("demo") {
        Some(demo_type) => {
            match demo_type.as_str() {
                "ownership" => app.demonstrate_ownership_patterns().await?,
                "async" => app.demonstrate_async_patterns().await?,
                "concurrency" => app.demonstrate_concurrency().await?,
                "unsafe" => app.demonstrate_unsafe_operations()?,
                "performance" => app.demonstrate_performance_patterns().await?,
                _ => app.run().await?,
            }
        }
        None => app.run().await?,
    }

    info!("Rust Test Project completed successfully");
    Ok(())
} 