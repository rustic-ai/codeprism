//! Unsafe operations for educational purposes - use with extreme caution!

/// Get element from slice without bounds checking (unsafe)
pub unsafe fn get_unchecked_element<T>(slice: &[T], index: usize) -> &T {
    slice.get_unchecked(index)
}

/// Create aligned buffer for performance-critical operations
pub fn create_aligned_buffer(size: usize) -> Vec<u8> {
    let layout = std::alloc::Layout::from_size_align(size, 64).unwrap();
    
    unsafe {
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        
        // Initialize with zeros
        std::ptr::write_bytes(ptr, 0, size);
        
        Vec::from_raw_parts(ptr, size, size)
    }
} 