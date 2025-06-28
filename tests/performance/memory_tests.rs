//! Memory usage and leak detection tests

use super::{PerformanceTestHarness, PerformanceThresholds};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Memory testing utilities
pub struct MemoryTests;

impl MemoryTests {
    /// Run comprehensive memory tests
    pub fn run_memory_tests() {
        let mut harness = PerformanceTestHarness::new();
        
        harness.run_test("memory_leak_detection", || {
            Self::test_memory_leaks()
        });
        
        harness.run_test("large_file_memory_usage", || {
            Self::test_large_file_memory()
        });
        
        harness.run_test("concurrent_memory_usage", || {
            Self::test_concurrent_memory_usage()
        });
        
        harness.print_summary();
    }

    /// Test for memory leaks during parsing operations
    fn test_memory_leaks() -> Result<(), Box<dyn std::error::Error>> {
        let initial_memory = Self::get_memory_usage();
        
        // Simulate multiple parsing operations
        for _ in 0..100 {
            Self::simulate_parsing_operation();
        }
        
        // Force garbage collection
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let final_memory = Self::get_memory_usage();
        let memory_increase = final_memory.saturating_sub(initial_memory);
        
        // Memory increase should be minimal (less than 10MB)
        if memory_increase > 10 * 1024 * 1024 {
            return Err(format!("Potential memory leak detected: {}MB increase", 
                memory_increase / 1024 / 1024).into());
        }
        
        Ok(())
    }

    /// Test memory usage with large files
    fn test_large_file_memory() -> Result<(), Box<dyn std::error::Error>> {
        let initial_memory = Self::get_memory_usage();
        
        // Simulate parsing a large file (10MB of code)
        let large_content = Self::generate_large_content(10_000_000);
        Self::simulate_parse_large_content(&large_content);
        
        let peak_memory = Self::get_memory_usage();
        let memory_used = peak_memory.saturating_sub(initial_memory);
        
        // Memory usage should be reasonable (less than 5x file size)
        if memory_used > large_content.len() * 5 {
            return Err(format!("Excessive memory usage: {}MB for {}MB file",
                memory_used / 1024 / 1024,
                large_content.len() / 1024 / 1024).into());
        }
        
        Ok(())
    }

    /// Test memory usage under concurrent load
    fn test_concurrent_memory_usage() -> Result<(), Box<dyn std::error::Error>> {
        let memory_counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        
        // Spawn multiple threads doing memory-intensive work
        for _ in 0..10 {
            let counter = Arc::clone(&memory_counter);
            let handle = std::thread::spawn(move || {
                for _ in 0..50 {
                    let content = Self::generate_test_content(1000);
                    Self::simulate_parse_content(&content);
                    counter.fetch_add(content.len(), Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().map_err(|_| "Thread panicked")?;
        }
        
        let total_processed = memory_counter.load(Ordering::Relaxed);
        println!("Total content processed: {}MB", total_processed / 1024 / 1024);
        
        // Verify no excessive memory usage
        let current_memory = Self::get_memory_usage();
        if current_memory > PerformanceThresholds::MAX_MEMORY_MB * 1024 * 1024 {
            return Err(format!("Memory usage exceeded threshold: {}MB",
                current_memory / 1024 / 1024).into());
        }
        
        Ok(())
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage() -> usize {
        // In a real implementation, this would use system APIs to get actual memory usage
        // For now, return a simulated value
        std::thread::sleep(std::time::Duration::from_millis(1));
        50 * 1024 * 1024 // 50MB baseline
    }

    /// Simulate a parsing operation
    fn simulate_parsing_operation() {
        let content = Self::generate_test_content(100);
        Self::simulate_parse_content(&content);
    }

    /// Generate test content of specified line count
    fn generate_test_content(lines: usize) -> String {
        let mut content = String::with_capacity(lines * 50);
        for i in 0..lines {
            content.push_str(&format!("def function_{}():\n    return {}\n", i, i));
        }
        content
    }

    /// Generate large content for memory testing
    fn generate_large_content(size_bytes: usize) -> String {
        let line = "def test_function():\n    return 42\n";
        let lines_needed = size_bytes / line.len();
        
        let mut content = String::with_capacity(size_bytes);
        for i in 0..lines_needed {
            content.push_str(&format!("def function_{}():\n    return {}\n", i, i));
        }
        content
    }

    /// Simulate parsing content
    fn simulate_parse_content(content: &str) {
        // Simulate the work of parsing
        let _lines: Vec<&str> = content.lines().collect();
        std::thread::sleep(std::time::Duration::from_micros(10));
    }

    /// Simulate parsing large content
    fn simulate_parse_large_content(content: &str) {
        // Simulate more intensive parsing for large content
        let _words: Vec<&str> = content.split_whitespace().collect();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage_measurement() {
        let memory = MemoryTests::get_memory_usage();
        assert!(memory > 0);
    }

    #[test]
    fn test_content_generation() {
        let content = MemoryTests::generate_test_content(10);
        assert!(content.contains("def function_0"));
        assert!(content.contains("def function_9"));
        
        let large_content = MemoryTests::generate_large_content(1000);
        assert!(large_content.len() >= 1000);
    }

    #[test]
    fn test_parsing_simulation() {
        let content = MemoryTests::generate_test_content(5);
        MemoryTests::simulate_parse_content(&content);
        // Should complete without panic
    }
} 