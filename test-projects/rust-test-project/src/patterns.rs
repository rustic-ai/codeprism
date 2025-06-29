//! Design patterns implemented in Rust.

use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use tracing::info;

/// Singleton pattern implementation using Once for thread-safe initialization
#[derive(Debug)]
pub struct SingletonManager {
    data: Mutex<HashMap<String, String>>,
}

impl SingletonManager {
    /// Get the singleton instance
    pub fn instance() -> &'static SingletonManager {
        static mut INSTANCE: Option<SingletonManager> = None;
        static ONCE: Once = Once::new();
        
        unsafe {
            ONCE.call_once(|| {
                INSTANCE = Some(SingletonManager {
                    data: Mutex::new(HashMap::new()),
                });
            });
            INSTANCE.as_ref().unwrap()
        }
    }

    /// Store a key-value pair
    pub fn store(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
    }

    /// Retrieve a value by key
    pub fn retrieve(&self, key: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    /// Get all stored data
    pub fn get_all(&self) -> HashMap<String, String> {
        let data = self.data.lock().unwrap();
        data.clone()
    }
}

/// Observer pattern implementation
pub trait Observer {
    fn notify(&self, event: &Event);
}

/// Event types for observer pattern
#[derive(Debug, Clone)]
pub enum Event {
    UserCreated { user_id: String, name: String },
    UserUpdated { user_id: String, field: String },
    UserDeleted { user_id: String },
    SystemEvent { message: String },
}

/// Subject that can be observed
pub struct ObserverPattern {
    observers: Mutex<Vec<Arc<dyn Observer + Send + Sync>>>,
}

impl ObserverPattern {
    /// Create a new observable subject
    pub fn new() -> Self {
        Self {
            observers: Mutex::new(Vec::new()),
        }
    }

    /// Add an observer
    pub fn add_observer(&self, observer: Arc<dyn Observer + Send + Sync>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }

    /// Remove an observer (simplified - removes all matching Arc pointers)
    pub fn remove_observer(&self, observer: Arc<dyn Observer + Send + Sync>) {
        let mut observers = self.observers.lock().unwrap();
        observers.retain(|obs| !Arc::ptr_eq(obs, &observer));
    }

    /// Notify all observers of an event
    pub fn notify_all(&self, event: &Event) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.notify(event);
        }
    }
}

/// Logger observer implementation
pub struct LoggerObserver {
    name: String,
}

impl LoggerObserver {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Observer for LoggerObserver {
    fn notify(&self, event: &Event) {
        info!("[{}] Received event: {:?}", self.name, event);
    }
}

/// Metrics observer implementation
pub struct MetricsObserver {
    event_count: Mutex<u64>,
}

impl MetricsObserver {
    pub fn new() -> Self {
        Self {
            event_count: Mutex::new(0),
        }
    }

    pub fn get_count(&self) -> u64 {
        *self.event_count.lock().unwrap()
    }
}

impl Observer for MetricsObserver {
    fn notify(&self, _event: &Event) {
        let mut count = self.event_count.lock().unwrap();
        *count += 1;
        info!("Metrics: Event count now at {}", *count);
    }
}

/// Builder pattern for complex configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub ssl_enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            ssl_enabled: false,
        }
    }
}

/// Builder for DatabaseConfig
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: DatabaseConfig::default(),
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.config.database = database.into();
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.password = password.into();
        self
    }

    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    pub fn timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.config.timeout_seconds = timeout_seconds;
        self
    }

    pub fn ssl_enabled(mut self, ssl_enabled: bool) -> Self {
        self.config.ssl_enabled = ssl_enabled;
        self
    }

    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

/// Strategy pattern for different processing strategies
pub trait ProcessingStrategy {
    fn process(&self, data: &str) -> String;
}

/// Fast processing strategy
pub struct FastProcessingStrategy;

impl ProcessingStrategy for FastProcessingStrategy {
    fn process(&self, data: &str) -> String {
        format!("FAST: {}", data.to_uppercase())
    }
}

/// Thorough processing strategy
pub struct ThoroughProcessingStrategy;

impl ProcessingStrategy for ThoroughProcessingStrategy {
    fn process(&self, data: &str) -> String {
        let processed = data
            .chars()
            .map(|c| if c.is_ascii_alphabetic() { c.to_uppercase().next().unwrap() } else { c })
            .collect::<String>();
        format!("THOROUGH: {} (length: {})", processed, data.len())
    }
}

/// Context that uses a processing strategy
pub struct ProcessingContext {
    strategy: Box<dyn ProcessingStrategy>,
}

impl ProcessingContext {
    pub fn new(strategy: Box<dyn ProcessingStrategy>) -> Self {
        Self { strategy }
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn ProcessingStrategy>) {
        self.strategy = strategy;
    }

    pub fn execute(&self, data: &str) -> String {
        self.strategy.process(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton_manager() {
        let manager1 = SingletonManager::instance();
        let manager2 = SingletonManager::instance();
        
        // Both references should point to the same instance
        assert!(std::ptr::eq(manager1, manager2));
        
        manager1.store("key1".to_string(), "value1".to_string());
        
        // Value should be accessible from both references
        assert_eq!(manager2.retrieve("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_observer_pattern() {
        let subject = ObserverPattern::new();
        let logger = Arc::new(LoggerObserver::new("TestLogger".to_string()));
        let metrics = Arc::new(MetricsObserver::new());
        
        subject.add_observer(logger.clone());
        subject.add_observer(metrics.clone());
        
        let event = Event::UserCreated {
            user_id: "123".to_string(),
            name: "Test User".to_string(),
        };
        
        subject.notify_all(&event);
        
        assert_eq!(metrics.get_count(), 1);
    }

    #[test]
    fn test_builder_pattern() {
        let config = DatabaseConfigBuilder::new()
            .host("example.com")
            .port(3306)
            .database("myapp")
            .username("admin")
            .password("secret")
            .max_connections(50)
            .ssl_enabled(true)
            .build();
        
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database, "myapp");
        assert!(config.ssl_enabled);
    }

    #[test]
    fn test_strategy_pattern() {
        let mut context = ProcessingContext::new(Box::new(FastProcessingStrategy));
        
        let result = context.execute("hello world");
        assert!(result.contains("FAST"));
        assert!(result.contains("HELLO WORLD"));
        
        context.set_strategy(Box::new(ThoroughProcessingStrategy));
        
        let result = context.execute("hello world");
        assert!(result.contains("THOROUGH"));
        assert!(result.contains("HELLO WORLD"));
        assert!(result.contains("length:"));
    }
} 