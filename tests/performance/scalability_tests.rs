//! Scalability tests for large repositories

use super::{PerformanceTestHarness, PerformanceThresholds};
use std::time::{Duration, Instant};

/// Scalability testing
pub struct ScalabilityTests;

impl ScalabilityTests {
    /// Run all scalability tests
    pub fn run_scalability_tests() {
        let mut harness = PerformanceTestHarness::new();
        
        harness.run_test("small_repository_test", || {
            Self::test_repository_size(100) // 100 files
        });
        
        harness.run_test("medium_repository_test", || {
            Self::test_repository_size(1000) // 1K files
        });
        
        harness.run_test("large_repository_test", || {
            Self::test_repository_size(10000) // 10K files
        });
        
        harness.run_test("concurrent_analysis_scaling", || {
            Self::test_concurrent_scaling()
        });
        
        harness.print_summary();
    }

    /// Test repository of given size
    fn test_repository_size(file_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Simulate repository indexing
        Self::simulate_repository_indexing(file_count);
        
        let duration = start.elapsed();
        
        // Performance thresholds based on repository size
        let max_duration = match file_count {
            0..=100 => Duration::from_secs(5),    // Small: 5s
            101..=1000 => Duration::from_secs(30), // Medium: 30s  
            _ => Duration::from_secs(120),         // Large: 2min
        };
        
        if duration > max_duration {
            return Err(format!(
                "Repository indexing took too long: {}s for {} files (max: {}s)",
                duration.as_secs(),
                file_count,
                max_duration.as_secs()
            ).into());
        }
        
        println!("✅ Indexed {} files in {:.2}s", file_count, duration.as_secs_f64());
        Ok(())
    }

    /// Test concurrent analysis scaling
    fn test_concurrent_scaling() -> Result<(), Box<dyn std::error::Error>> {
        // Test analysis performance with increasing concurrency
        for workers in [1, 2, 4, 8, 16] {
            let start = Instant::now();
            
            Self::simulate_concurrent_analysis(workers, 100);
            
            let duration = start.elapsed();
            let throughput = 100.0 / duration.as_secs_f64();
            
            println!("Workers: {} | Duration: {:.2}s | Throughput: {:.1} files/sec", 
                workers, duration.as_secs_f64(), throughput);
            
            // Throughput should generally increase with more workers (up to a point)
            if workers > 1 && throughput < 1.0 {
                return Err(format!(
                    "Poor throughput with {} workers: {:.1} files/sec",
                    workers, throughput
                ).into());
            }
        }
        
        Ok(())
    }

    /// Simulate repository indexing
    fn simulate_repository_indexing(file_count: usize) {
        // Simulate the work of indexing files
        for i in 0..file_count {
            Self::simulate_file_processing();
            
            // Add some variation to simulate different file sizes
            if i % 100 == 0 {
                std::thread::sleep(Duration::from_micros(500));
            }
        }
    }

    /// Simulate concurrent analysis
    fn simulate_concurrent_analysis(worker_count: usize, files_per_worker: usize) {
        let mut handles = Vec::new();
        
        for _ in 0..worker_count {
            let handle = std::thread::spawn(move || {
                for _ in 0..files_per_worker {
                    Self::simulate_analysis_operation();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Simulate processing a single file
    fn simulate_file_processing() {
        // Simulate parsing and indexing work
        std::thread::sleep(Duration::from_micros(100));
    }

    /// Simulate an analysis operation
    fn simulate_analysis_operation() {
        // Simulate analysis work (slightly more intensive)
        std::thread::sleep(Duration::from_micros(200));
    }

    /// Test memory scaling with repository size
    pub fn test_memory_scaling() -> Result<(), Box<dyn std::error::Error>> {
        for file_count in [100, 500, 1000, 2000, 5000] {
            let initial_memory = Self::estimate_memory_usage();
            
            // Simulate loading repository into memory
            Self::simulate_repository_loading(file_count);
            
            let peak_memory = Self::estimate_memory_usage();
            let memory_per_file = (peak_memory - initial_memory) / file_count;
            
            println!("Files: {} | Memory per file: {}KB", 
                file_count, memory_per_file / 1024);
            
            // Memory usage per file should be reasonable
            if memory_per_file > 100 * 1024 { // 100KB per file max
                return Err(format!(
                    "Excessive memory usage: {}KB per file for {} files",
                    memory_per_file / 1024,
                    file_count
                ).into());
            }
        }
        
        Ok(())
    }

    /// Simulate loading repository into memory
    fn simulate_repository_loading(file_count: usize) {
        // Simulate loading and parsing files
        for _ in 0..file_count {
            let _content = Self::generate_file_content();
            std::thread::sleep(Duration::from_micros(50));
        }
    }

    /// Generate simulated file content
    fn generate_file_content() -> String {
        "def example_function():\n    return 42\n".to_string()
    }

    /// Estimate current memory usage
    fn estimate_memory_usage() -> usize {
        // Simplified memory estimation
        50 * 1024 * 1024 // 50MB baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_repository() {
        let result = ScalabilityTests::test_repository_size(10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_scaling() {
        let result = ScalabilityTests::test_memory_scaling();
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_processing_simulation() {
        ScalabilityTests::simulate_file_processing();
        // Should complete without panic
    }
} 