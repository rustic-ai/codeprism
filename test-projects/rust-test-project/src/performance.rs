//! Performance optimization patterns and utilities.

use rayon::prelude::*;

/// Efficient data processing with pre-allocation
pub fn process_large_dataset(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len());
    
    for &item in data {
        result.push(item * 2);
    }
    
    result
}

/// Parallel processing with Rayon
pub fn parallel_sum(data: &[i32]) -> i32 {
    data.par_iter().sum()
}

/// String building with capacity
pub fn build_large_string(items: &[&str]) -> String {
    let total_length: usize = items.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(total_length + items.len() * 2);
    
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(item);
    }
    
    result
} 